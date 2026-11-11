//! The session's approvals (temper/docs/design/client/pages/README.md, temper/docs/design/client/forge/records.md).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Family, Handle, Latest, Link, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

/// Whether the person's review counts for the approval the change waits on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Yours {
    Counts,
    NoAccount,
    TooFew,
    OtherRole,
}

/// A change waiting on a person's review on its forge
/// (temper/docs/design/client/forge/records.md; temper/docs/design/client/forge/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Row {
    pub task: u64,
    pub project: u32,
    pub goal: Option<u64>,
    pub forge: u16,
    pub kind: super::forges::Kind,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    /// The files page on the forge's web address, empty for none.
    pub address: Box<[u8]>,
    pub title: Box<[u8]>,
    /// When the change began waiting at its head.
    pub since: Wall,
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: u32,
    pub sets: Box<[ApprovalSet]>,
    pub whole: bool,
    pub yours: Yours,
}
/// A path set touched by this approval's change (temper/docs/design/client/forge/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApprovalSet {
    pub number: u8,
    pub name: Box<[u8]>,
}

#[derive(Debug)]
enum State {
    Unknown,
    Rows { rows: Box<[Row]>, more: u32 },
    Ended,
}
/// The session's approvals document: no rows until delivered, and none once refused or ended.
#[derive(Debug)]
pub struct Approvals {
    state: State,
}
impl Default for Approvals {
    fn default() -> Self {
        Self { state: State::Unknown }
    }
}
impl Approvals {
    /// The delivered rows, oldest wait first; empty until delivered or once ended.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        match &self.state {
            State::Rows { rows, .. } => rows,
            State::Unknown | State::Ended => &[],
        }
    }
    /// The rows past the document's bound, or zero while unknown or ended.
    #[must_use]
    pub fn more(&self) -> u32 {
        match self.state {
            State::Rows { more, .. } => more,
            State::Unknown | State::Ended => 0,
        }
    }
}
impl Side {
    /// The session's approvals, independent of the current page.
    #[must_use]
    pub const fn approvals(&self) -> &Approvals {
        &self.approvals
    }
}
#[must_use]
pub fn watch(limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let record = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Approvals }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}

pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::APPROVALS }
}
pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.approvals.state = State::Unknown;
        side.changed();
        return;
    }
    if side.link == Link::Live
        && side.party.is_some()
        && !side.approvals_open
        && let State::Unknown = side.approvals.state
    {
        if let Some(subject) = watch(&limits) {
            side.approvals_open = true;
            out.push(Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.approvals.state = State::Ended;
        }
        side.changed();
    }
}

/// A whole approvals delivery, or the end of its watch (client/forge/records.md).
#[derive(Debug)]
pub enum Event {
    Rows { rows: Box<[Row]>, more: u32 },
    Ended,
}
pub(crate) fn told(client: &jig_client::Client, told: &Told, limits: Limits) -> Option<Event> {
    match told {
        Told::Changed { handle: changed } if *changed == handle() => {
            let document = client.document(*changed)?;
            if document.state == jig_client::State::Ended {
                return Some(Event::Ended);
            }
            match document.latest {
                Some(Latest::Family(record)) => Some(match rows(record, limits) {
                    Some(event) => event,
                    None => Event::Ended,
                }),
                Some(Latest::Jig(_)) => Some(Event::Ended),
                None => None,
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => Some(Event::Ended),
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => None,
    }
}
pub(crate) fn receive(side: &mut Side, event: Event, out: &mut Queue<Request>) {
    if !side.approvals_open {
        return;
    }
    side.approvals.state = match event {
        Event::Rows { rows, more } => State::Rows { rows, more },
        Event::Ended => {
            end(side, out);
            State::Ended
        }
    };
    side.changed();
}
fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.approvals_open {
        side.approvals_open = false;
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}
fn rows(record: &Family, limits: Limits) -> Option<Event> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let document = wire::decode_approvals(&record.bytes, &wire_limits(&limits)?)?.into_parts();
    let mut rows = List::with_capacity(document.rows.len());
    for value in document.rows.into_boxed() {
        let value = value.into_parts();
        let mut sets = List::with_capacity(value.sets.len());
        for set in value.sets.into_boxed() {
            let set = set.into_parts();
            sets.push(ApprovalSet { number: set.number, name: set.name }).expect("wire bounds approval sets");
        }
        rows.push(Row {
            task: value.task,
            project: value.project,
            goal: value.goal,
            forge: value.forge,
            kind: match value.kind {
                w::Kind::Forgejo => super::forges::Kind::Forgejo,
                w::Kind::Github => super::forges::Kind::GitHub,
            },
            owner: value.owner,
            name: value.name,
            pull: value.pull,
            address: value.address,
            title: value.title,
            since: Wall::from_nanos(value.since),
            scope: value.scope,
            rule: value.rule,
            role: value.role,
            required: value.required,
            counted: value.counted,
            sets: sets.into_boxed(),
            whole: value.whole,
            yours: match value.yours {
                w::ApprovalYours::Counts => Yours::Counts,
                w::ApprovalYours::NoAccount => Yours::NoAccount,
                w::ApprovalYours::TooFew => Yours::TooFew,
                w::ApprovalYours::OtherRole => Yours::OtherRole,
            },
        })
        .expect("wire bounds approval rows");
    }
    Some(Event::Rows { rows: rows.into_boxed(), more: document.more })
}
