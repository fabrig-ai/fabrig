//! A connected Forgejo's one-shot bot tokens (temper/docs/design/client/forge.md, 5.4).
use super::{Limits, Request, Served, Side, forges, github};
use alloc::boxed::Box;
use jig_client::{Ask, Event, Family, Key, Told, contract as c};
use skein_lib::{Queue, Secret};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Api,
    Git,
    Webhook,
}
impl Field {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Api => 0,
            Self::Git => 1,
            Self::Webhook => 2,
        }
    }
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads presentation; the secrets stay private")]
pub struct Draft {
    pub written: [u64; 3],
    pub checked: bool,
    pub too_long: bool,
    values: [Option<Secret>; 3],
    oversized: [bool; 3],
}
impl Draft {
    pub(crate) fn new() -> Self {
        Self { written: [0; 3], checked: false, too_long: false, values: [None, None, None], oversized: [false; 3] }
    }
    #[must_use]
    pub fn has(&self, field: Field) -> bool {
        self.values.get(field.index()).expect("three secrets").is_some()
    }
    fn bundle(&self) -> Option<Secret> {
        let [api, git, secret] = &self.values;
        forges::pasted_tokens(
            match api {
                Some(value) => Some(value.expose()),
                None => None,
            },
            match git {
                Some(value) => Some(value.expose()),
                None => None,
            },
            match secret {
                Some(value) => Some(value.expose()),
                None => None,
            },
        )
    }
    pub(crate) fn clear(&mut self) {
        self.values = [None, None, None];
        self.oversized = [false; 3];
        self.checked = false;
        self.too_long = false;
        for written in &mut self.written {
            *written = written.checked_add(1).expect("secret writes fit");
        }
    }
}
/// What a sent connect has seen of its forge's row: the stop that empties the fields
/// again is its own (temper/docs/design/client/forge.md, 5.4).
#[derive(Debug, PartialEq, Eq)]
enum Seen {
    /// Not yet stopped or connected: the row's stop and its last connect's words, as last delivered.
    Open { stopped: bool, ended: Option<Box<[u8]>> },
    /// Stopped once, connected, or answered without a connect: no later row empties the fields.
    Settled,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads the connect's presentation; its seen row stays private")]
pub struct Pending {
    pub forge: u16,
    pub key: Key,
    pub answer: Option<github::Answer>,
    seen: Seen,
}
pub(crate) fn busy(side: &Side) -> bool {
    match &side.tokens {
        Some(pending) => pending.answer.is_none(),
        None => false,
    }
}
pub(crate) fn write(side: &mut Side, field: Field, value: Secret) {
    if busy(side) {
        return;
    }
    let draft = &mut side.tokens_form;
    *draft.oversized.get_mut(field.index()).expect("three secret bounds") = value.len() > forges::PASTED_PAYLOAD;
    *draft.values.get_mut(field.index()).expect("three secrets") =
        if value.is_empty() || value.len() > forges::PASTED_PAYLOAD { None } else { Some(value) };
    draft.too_long = draft.oversized.contains(&true)
        || (draft.bundle().is_none() && (draft.has(Field::Api) || draft.has(Field::Git) || draft.has(Field::Webhook)));
    draft.checked = false;
    side.changed();
}
pub(crate) fn connect_new(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if busy(side) || !forges::can_make(side) || side.playground_running() {
        return;
    }
    side.admin_form.place.checked = true;
    let Some(place) = side.admin_form.place.place() else {
        side.changed();
        return;
    };
    send_new(side, limits, place, out);
}
pub(crate) fn send_new(side: &mut Side, limits: Limits, place: forges::Managed, out: &mut Queue<Request>) {
    if !place.valid() || !forges::can_make(side) || side.playground_running() {
        return;
    }
    send_target(side, limits, 0, Some(place), out);
}
pub(crate) fn send(side: &mut Side, limits: Limits, forge: u16, out: &mut Queue<Request>) {
    send_target(side, limits, forge, None, out);
}
fn send_target(side: &mut Side, limits: Limits, forge: u16, place: Option<forges::Managed>, out: &mut Queue<Request>) {
    let Some(_party) = side.party else { return };
    if busy(side) || side.served == Served::OutOfDate {
        return;
    }
    let seen = if forge != 0 {
        let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) else { return };
        if !offered(row) {
            return;
        }
        Seen::Open { stopped: stop(row), ended: row.ended.clone() }
    } else {
        if !forges::can_make(side) || side.playground_running() || place.is_none() {
            return;
        }
        Seen::Open { stopped: false, ended: None }
    };
    let draft = &mut side.tokens_form;
    draft.checked = true;
    if !draft.has(Field::Api) || !draft.has(Field::Git) || draft.too_long {
        side.changed();
        return;
    }
    let value = draft.bundle().expect("validated bundle fits");
    if forge == 0 {
        forges::new_record(side, forges::Kind::Forgejo, forges::Mode::Connected);
    }
    let bounds = super::wire_limits(&limits).expect("checked limits");
    let Some(target) = forges::making(side, forge, &bounds, place) else { return };
    let request = w::Request::Connect(
        w::Connect::new(
            &bounds,
            w::ConnectParts {
                forge: target,
                how: w::How::ForgejoTokens(
                    w::ForgejoTokens::new(&bounds, w::ForgejoTokensParts {}).expect("empty tokens fits"),
                ),
            },
        )
        .expect("connect fits"),
    );
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).expect("request fits");
    let request = Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    };
    side.tokens_form.clear();
    side.serial = side.serial.checked_add(1).expect("keys fit");
    let key = Key { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    let name = crate::key::named(
        b"temper-tokens",
        &crate::key::Name::new(skein_lib::Decimal::of(side.serial).as_bytes()).expect("serial fits"),
    );
    side.tokens = Some(Pending { forge, key, answer: None, seen });
    out.push(Request::Client { asked: Event::Carry { key, name, request, value: Some(value) } });
    side.changed();
}
pub(crate) fn told(side: &mut Side, client: &jig_client::Client, told: &Told, limits: Limits) -> bool {
    let Some(pending) = &mut side.tokens else { return false };
    let answer = match told {
        Told::Answered { key, outcome } if *key == pending.key => github::decode(outcome, false, limits),
        Told::Pending { key } if *key == pending.key => {
            if let Some(value) = client.pending(*key)
                && value.progress == jig_client::Progress::Unknown
            {
                Some(github::Answer::Unknown)
            } else {
                return true;
            }
        }
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return false,
    };
    if pending.answer.is_none() {
        let answer = answer.unwrap_or(github::Answer::Unknown);
        match &answer {
            github::Answer::Connecting | github::Answer::Unknown => {}
            github::Answer::Link(_)
            | github::Answer::Page(_)
            | github::Answer::NoAccess
            | github::Answer::NotFound
            | github::Answer::Refused(_)
            | github::Answer::Entrance(_) => pending.seen = Seen::Settled,
        }
        pending.answer = Some(answer);
        side.tokens_form.clear();
        if pending.forge == 0
            && pending.answer == Some(github::Answer::Connecting)
            && let Some(new) = &mut side.setup_form.making
            && new.kind == forges::Kind::Forgejo
            && new.mode == forges::Mode::Connected
        {
            new.accepted = true;
        }
        side.changed();
    }
    true
}
pub(crate) fn stopped(side: &mut Side) {
    let Some(pending) = &mut side.tokens else { return };
    if pending.forge == 0 {
        return;
    }
    let Some(row) = super::connect::row(pending.forge, &side.forges, None) else { return };
    let Seen::Open { stopped, ended } = &pending.seen else { return };
    let now = stop(row);
    if now && (!stopped || row.ended != *ended) {
        side.tokens_form.clear();
        if pending.answer.is_none() {
            pending.answer = Some(github::Answer::Connecting);
        }
        pending.seen = Seen::Settled;
    } else {
        pending.seen = match row.connection {
            forges::Connection::Connected { .. } => Seen::Settled,
            forges::Connection::Stopped { .. }
            | forges::Connection::NotConnected
            | forges::Connection::NotSetUp
            | forges::Connection::SettingUp { .. }
            | forges::Connection::SetUpFailed { .. }
            | forges::Connection::Connecting => Seen::Open { stopped: now, ended: row.ended.clone() },
        };
    }
}
fn stop(row: &forges::Row) -> bool {
    match row.connection {
        forges::Connection::Stopped { .. } => true,
        forges::Connection::NotConnected => row.ended.is_some(),
        forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting
        | forges::Connection::Connected { .. } => false,
    }
}

/// Only a disconnected Forgejo of the connected mode offers this Connect.
#[must_use]
pub fn offered(row: &forges::Row) -> bool {
    if row.kind != forges::Kind::Forgejo || row.mode != forges::Mode::Connected {
        return false;
    }
    match row.connection {
        forges::Connection::NotConnected | forges::Connection::Stopped { .. } => true,
        forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting
        | forges::Connection::Connected { .. } => false,
    }
}
