//! Per-row replacement of carried credentials (temper/docs/design/client/forge.md, 5.4).
use super::{Limits, Request, Served, Side, forges, github};
use alloc::boxed::Box;
use jig_client::{Ask, Event, Family, Key, Told, contract as c};
use skein_lib::{Queue, Secret};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Key,
    Api,
    Git,
    Webhook,
}
impl Field {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Key => 0,
            Self::Api => 1,
            Self::Git => 2,
            Self::Webhook => 3,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Sent,
    Connecting,
    Observed,
    Finished,
}

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads presentation while credentials stay private")]
pub struct Draft {
    pub forge: u16,
    pub written: [u64; 4],
    pub checked: bool,
    pub too_long: bool,
    pub pending: Option<Key>,
    pub answer: Option<github::Answer>,
    values: [Option<Secret>; 4],
    oversized: [bool; 4],
    stopped: bool,
    ended: Option<Box<[u8]>>,
    phase: Phase,
}
impl Draft {
    fn new(row: &forges::Row) -> Self {
        Self {
            forge: row.number,
            written: [0; 4],
            checked: false,
            too_long: false,
            pending: None,
            answer: None,
            values: [None, None, None, None],
            oversized: [false; 4],
            stopped: stopped_row(row),
            ended: row.ended.clone(),
            phase: Phase::Idle,
        }
    }
    #[must_use]
    pub fn has(&self, field: Field) -> bool {
        self.values.get(field.index()).expect("four fields").is_some()
    }
    #[must_use]
    pub fn busy(&self) -> bool {
        self.pending.is_some() && self.answer.is_none()
    }
    #[must_use]
    pub fn connecting(&self) -> bool {
        match self.phase {
            Phase::Connecting | Phase::Observed => true,
            Phase::Idle | Phase::Sent | Phase::Finished => false,
        }
    }
    fn bytes(&self, field: Field) -> Option<&[u8]> {
        match self.values.get(field.index()).expect("four fields") {
            Some(value) => Some(value.expose()),
            None => None,
        }
    }
    fn bundle(&self, kind: forges::Kind) -> Option<Secret> {
        match kind {
            forges::Kind::GitHub => forges::restored_app_key(self.bytes(Field::Key)?),
            forges::Kind::Forgejo => {
                forges::pasted_tokens(self.bytes(Field::Api), self.bytes(Field::Git), self.bytes(Field::Webhook))
            }
        }
    }
    fn clear(&mut self) {
        self.values = [None, None, None, None];
        self.oversized = [false; 4];
        self.checked = false;
        self.too_long = false;
        for written in &mut self.written {
            *written = written.checked_add(1).expect("secret writes fit");
        }
    }
}
#[must_use]
pub fn offered(row: &forges::Row) -> bool {
    if row.mode == forges::Mode::Fake || (row.kind == forges::Kind::Forgejo && row.mode != forges::Mode::Connected) {
        return false;
    }
    match row.connection {
        forges::Connection::Connected { .. } | forges::Connection::Stopped { .. } => true,
        forges::Connection::NotConnected
        | forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting => false,
    }
}
pub(crate) fn open(side: &mut Side, forge: u16) {
    let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) else { return };
    if !offered(row) || side.replacement(forge).is_some() {
        return;
    }
    side.replacements.push(Draft::new(row)).expect("one draft per bounded row");
    side.changed();
}
pub(crate) fn close(side: &mut Side, forge: u16) {
    let drafts = core::mem::replace(&mut side.replacements, skein_lib::List::with_capacity(forges::ROWS));
    for draft in drafts.into_boxed() {
        if draft.forge != forge {
            side.replacements.push(draft).expect("bounded drafts");
        }
    }
    side.changed();
}
pub(crate) fn write(side: &mut Side, forge: u16, field: Field, value: Secret) {
    let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) else { return };
    if !offered(row) || ((row.kind == forges::Kind::GitHub) != (field == Field::Key)) {
        return;
    }
    let kind = row.kind;
    let Some(index) = index(side, forge) else { return };
    let draft = side.replacements.get_mut(index).expect("draft index");
    if draft.busy() {
        return;
    }
    let bytes = if field == Field::Key { forges::trimmed_app_key(value.expose()) } else { value.expose() };
    let oversized = value.len() > forges::PASTED_PAYLOAD;
    let empty = bytes.is_empty();
    *draft.oversized.get_mut(field.index()).expect("four bounds") = oversized;
    *draft.values.get_mut(field.index()).expect("four values") = if empty || oversized { None } else { Some(value) };
    draft.too_long = draft.oversized.contains(&true)
        || ((draft.has(Field::Key) || draft.has(Field::Api) || draft.has(Field::Git) || draft.has(Field::Webhook))
            && draft.bundle(kind).is_none());
    draft.checked = false;
    draft.answer = None;
    draft.pending = None;
    draft.phase = Phase::Idle;
    side.changed();
}
pub(crate) fn send(side: &mut Side, limits: Limits, forge: u16, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == Served::OutOfDate {
        return;
    }
    let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) else { return };
    if !offered(row) {
        return;
    }
    let kind = row.kind;
    let Some(index) = index(side, forge) else { return };
    let draft = side.replacements.get_mut(index).expect("draft index");
    if draft.busy() {
        return;
    }
    draft.checked = true;
    let required = match kind {
        forges::Kind::GitHub => draft.has(Field::Key),
        forges::Kind::Forgejo => draft.has(Field::Api) || draft.has(Field::Git),
    };
    if !required || draft.too_long {
        side.changed();
        return;
    }
    let value = draft.bundle(kind).expect("validated replacement bundle");
    let bounds = super::wire_limits(&limits).expect("checked limits");
    let request = w::Request::Replace(w::Replace::new(&bounds, w::ReplaceParts { forge }).expect("forge fits"));
    let record = w::Requesting::new(&bounds, w::RequestingParts { request }).expect("request fits");
    let request = Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    };
    side.serial = side.serial.checked_add(1).expect("keys fit");
    let key = Key { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    let name = crate::key::named(
        b"temper-replace",
        &crate::key::Name::new(skein_lib::Decimal::of(side.serial).as_bytes()).expect("serial fits"),
    );
    draft.clear();
    draft.pending = Some(key);
    draft.answer = None;
    draft.phase = Phase::Sent;
    out.push(Request::Client { asked: Event::Carry { key, name, request, value: Some(value) } });
    side.changed();
}
pub(crate) fn told(side: &mut Side, client: &jig_client::Client, told: &Told, limits: Limits) -> bool {
    let key = match told {
        Told::Answered { key, .. } | Told::Pending { key } => *key,
        Told::Link { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return false,
    };
    let mut found = None;
    for index in 0..side.replacements.len() {
        if side.replacements.get(index).expect("draft index").pending == Some(key) {
            found = Some(index);
            break;
        }
    }
    let Some(index) = found else { return false };
    let draft = side.replacements.get_mut(index).expect("draft index");
    if draft.answer.is_some() {
        return true;
    }
    let answer = match told {
        Told::Answered { outcome, .. } => github::decode(outcome, false, limits).unwrap_or(github::Answer::Unknown),
        Told::Pending { .. } => {
            match client.pending(key) {
                Some(value) if value.progress == jig_client::Progress::Unknown => {}
                Some(_) | None => return true,
            }
            github::Answer::Unknown
        }
        Told::Link { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return false,
    };
    draft.clear();
    if answer == github::Answer::Connecting {
        if draft.phase != Phase::Observed && draft.phase != Phase::Finished {
            draft.phase = Phase::Connecting;
        }
    } else {
        draft.phase = Phase::Idle;
    }
    draft.answer = Some(answer);
    side.changed();
    true
}
pub(crate) fn stopped(side: &mut Side) {
    let forges::State::Rows(_) = &side.forges else { return };
    let drafts = core::mem::replace(&mut side.replacements, skein_lib::List::with_capacity(forges::ROWS));
    for mut draft in drafts.into_boxed() {
        let Some(row) = super::connect::row(draft.forge, &side.forges, None) else { continue };
        let stopped = stopped_row(row);
        if draft.pending.is_some() {
            if row.connection == forges::Connection::Connecting {
                draft.phase = Phase::Observed;
            } else if draft.connecting() || (stopped && !draft.stopped) || row.ended != draft.ended {
                draft.phase = Phase::Finished;
            }
        }
        if (stopped && !draft.stopped) || row.ended != draft.ended {
            draft.clear();
        }
        draft.stopped = stopped;
        draft.ended.clone_from(&row.ended);
        side.replacements.push(draft).expect("bounded row drafts");
    }
}
fn stopped_row(row: &forges::Row) -> bool {
    match row.connection {
        forges::Connection::Stopped { .. } => true,
        forges::Connection::Connected { .. }
        | forges::Connection::NotConnected
        | forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting => false,
    }
}

fn index(side: &Side, forge: u16) -> Option<u32> {
    for index in 0..side.replacements.len() {
        if side.replacements.get(index)?.forge == forge {
            return Some(index);
        }
    }
    None
}
