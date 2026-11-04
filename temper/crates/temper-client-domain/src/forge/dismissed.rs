//! A person's dismissal facts (temper/docs/design/client/README.md, 4.6).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Handle, Latest, Link, Told, Watchable, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dismissal {
    ToBegin { project: u32 },
}

/// An ended watch stays ended until the next sign-in.
#[derive(Debug)]
pub enum State {
    Unknown,
    Rows(Box<[Dismissal]>),
    Ended,
}

#[must_use]
pub fn watch(limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let record = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Dismissed }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}

/// The request "Skip" will send: of `what`'s project's scope.
#[must_use]
pub fn ask(what: Dismissal, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let Dismissal::ToBegin { project } = what;
    let what = w::Dismissal::ToBegin(w::ToBegin::new(&bounds, w::ToBeginParts { project }).ok()?);
    let dismiss = w::Dismiss::new(&bounds, w::DismissParts { what }).ok()?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request: w::Request::Dismiss(dismiss) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}

impl Side {
    /// `None` until the first document, or once the watch is refused or ended.
    #[must_use]
    pub fn dismissed(&self, what: Dismissal) -> Option<bool> {
        match &self.dismissed {
            State::Rows(rows) => Some(rows.contains(&what)),
            State::Unknown | State::Ended => None,
        }
    }
}

pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::DISMISSED }
}

pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.dismissed = State::Unknown;
        side.changed();
        return;
    }
    if side.party.is_some()
        && !side.dismissed_open
        && let State::Unknown = side.dismissed
    {
        if let Some(subject) = watch(&limits) {
            side.dismissed_open = true;
            out.push(Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.dismissed = State::Ended;
        }
        side.changed();
    }
}

pub(crate) fn told(
    side: &mut Side,
    client: &jig_client::Client,
    told: &Told,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    match told {
        Told::Changed { handle: changed } if *changed == handle() => {
            if !side.dismissed_open {
                return true;
            }
            side.dismissed = match client.document(*changed) {
                Some(document) if document.state == jig_client::State::Ended => State::Ended,
                Some(document) => match document.latest {
                    Some(Latest::Family(record)) => match rows(record, limits) {
                        Some(rows) => State::Rows(rows),
                        None => State::Ended,
                    },
                    Some(Latest::Jig(_)) => State::Ended,
                    None => State::Unknown,
                },
                None => State::Unknown,
            };
            if let State::Ended = side.dismissed {
                end(side, out);
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            side.dismissed = State::Ended;
            end(side, out);
        }
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    }
    side.changed();
    true
}

fn rows(record: &Family, limits: Limits) -> Option<Box<[Dismissal]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let document = wire::decode_dismissals(&record.bytes, &wire_limits(&limits)?)?;
    let mut rows = List::with_capacity(wire::DISMISSED);
    for what in document.into_parts().dismissed.into_boxed() {
        let what = match what {
            w::Dismissal::ToBegin(value) => Dismissal::ToBegin { project: value.project() },
        };
        rows.push(what).expect("wire bounds dismissal rows");
    }
    Some(rows.into_boxed())
}

fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.dismissed_open {
        side.dismissed_open = false;
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}
