//! Playground lifecycle (temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/family.md, 7).
use super::{Limits, Request, Side, family, wire_limits};
pub use super::{Playground, PlaygroundState};
use alloc::boxed::Box;
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
#[derive(Debug)]
pub(super) struct State {
    pub answer: Option<Playground>,
    pub pending: Option<Handle>,
    pub live: bool,
    pub answered: bool,
    pub start: Option<Starting>,
    pub failure: Option<StartFailure>,
    pub end: Option<Token>,
    pub leaving: bool,
    pub deployment: Option<(Handle, u32)>,
}
#[derive(Debug)]
pub(super) struct Starting {
    pub request: Token,
}
impl State {
    pub const fn new() -> Self {
        Self {
            answer: None,
            pending: None,
            live: false,
            answered: false,
            start: None,
            failure: None,
            end: None,
            leaving: false,
            deployment: None,
        }
    }
}
pub(crate) fn told(side: &mut Side, told: &Told, limits: Limits, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.playground.live {
                side.playground.live = true;
                read(side, out);
            } else if *link != Link::Live {
                side.playground.live = false;
            }
            false
        }
        Told::Page { handle, page } if Some(*handle) == side.playground.pending => {
            side.playground.pending = None;
            let answer = match page {
                Readout::Family { family: number, record: Some(record) } if *number == wire::FORGE => {
                    family::playground(
                        record,
                        &match wire_limits(&limits) {
                            Some(limits) => limits,
                            None => return true,
                        },
                    )
                }
                Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
            };
            let empty = match page {
                Readout::Family { family, record: None } => *family == wire::FORGE,
                Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => false,
            };
            let changed = !side.playground.answered || side.playground.answer != answer;
            let was_leaving = side.playground.leaving;
            if empty && side.playground.leaving {
                reset(side, out);
            }
            if let Some(answer) = &answer
                && answer.state == PlaygroundState::Resetting
            {
                side.playground.leaving = true;
            }
            side.playground.answer = answer;
            side.playground.answered = true;
            if changed || was_leaving != side.playground.leaving {
                side.changed();
            }
            // Transitional rows are read until running or erased (4.8).
            if let Some(answer) = &side.playground.answer {
                match answer.state {
                    PlaygroundState::Starting | PlaygroundState::Resetting => read(side, out),
                    PlaygroundState::Running => {}
                }
            }
            true
        }
        Told::Answered { key, outcome } if owns(side, *key, true) => {
            side.playground.start = None;
            side.playground.failure = match outcome {
                jig_client::Outcome::Answered(value) => match answer(value, &limits) {
                    Some(StartAnswer::Failed(why)) => Some(why),
                    Some(
                        StartAnswer::Running
                        | StartAnswer::NotOffered
                        | StartAnswer::Unknown
                        | StartAnswer::EntranceRefused(_),
                    )
                    | None => None,
                },
                jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Held
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Unsent
                | jig_client::Outcome::Full { .. }
                | jig_client::Outcome::Lost
                | jig_client::Outcome::Unconfirmed => None,
            };
            read(side, out);
            side.changed();
            true
        }
        Told::Answered { key, outcome } if owns(side, *key, false) => {
            side.playground.end = None;
            match end_answer(outcome, limits) {
                Some(EndAnswer::Reset) => reset(side, out),
                Some(EndAnswer::NotOffered) => side.playground.leaving = false,
                None => {}
            }
            read(side, out);
            side.changed();
            true
        }
        Told::Page { handle, .. } if handle.asker == crate::owner::Owner::Forge.asker() => true,
        Told::Page { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => false,
    }
}
fn owns(side: &Side, key: jig_client::Key, start: bool) -> bool {
    if key.asker != crate::owner::Owner::Forge.asker() {
        return false;
    }
    let token = if start {
        let Some(start) = &side.playground.start else { return false };
        start.request
    } else {
        let Some(end) = side.playground.end else { return false };
        end
    };
    token.raw() == key.number
}
fn reset(side: &mut Side, out: &mut Queue<Request>) {
    side.playground.leaving = false;
    side.playground.end = None;
    side.playground.start = None;
    side.playground.failure = None;
    side.playground.answer = None;
    out.push(Request::PlaygroundReset);
}
pub(crate) fn read(side: &mut Side, out: &mut Queue<Request>) {
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.playground.pending = Some(handle);
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground })
        .expect("fieldless playground read");
    out.push(Request::Client {
        asked: jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Deployment,
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        },
    });
}
pub(crate) fn start(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.party.is_none()
        || side.served == super::Served::OutOfDate
        || side.playground.start.is_some()
        || (side.playground.answer.is_some() && side.playground.failure.is_none())
        || side.playground.leaving
    {
        return;
    }
    let Some(ask) = ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.playground.start = Some(Starting { request });
    side.playground.failure = None;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
pub(crate) fn end(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.party.is_none()
        || side.served == super::Served::OutOfDate
        || side.playground.answer.is_none()
        || side.playground.leaving
    {
        return;
    }
    let Some(ask) = end_ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.playground.end = Some(request);
    side.playground.leaving = true;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndAnswer {
    Reset,
    NotOffered,
}
#[must_use]
pub fn end_ask(limits: &Limits) -> Option<jig_client::Ask> {
    let record =
        w::Requesting::new(&wire_limits(limits)?, w::RequestingParts { request: w::Request::EndPlayground }).ok()?;
    Some(jig_client::Ask::Family {
        scope: c::Scope::Deployment,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::requesting(&record),
        },
    })
}
fn end_answer(outcome: &jig_client::Outcome, limits: Limits) -> Option<EndAnswer> {
    let record = match outcome {
        jig_client::Outcome::Answered(value) => match value {
            jig_client::Answered::Family(record) => record,
            jig_client::Answered::Jig(_) => return None,
        },
        jig_client::Outcome::Unknown
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Held
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost
        | jig_client::Outcome::Unconfirmed => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(&limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    match answer.into_parts().outcome {
        w::Outcome::Reset => Some(EndAnswer::Reset),
        w::Outcome::EndRefused(why) => match why.into_parts().why {
            w::EndRefusal::NotOffered => Some(EndAnswer::NotOffered),
        },
        w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::Unknown
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Removed
        | w::Outcome::Looked
        | w::Outcome::Dismissed
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged
        | w::Outcome::FindingRefused(_)
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::RemoveRefused(_) => None,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartFailure {
    Group(Box<[u8]>),
    Account,
    Adoption,
    NotConnected,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartAnswer {
    Running,
    NotOffered,
    Failed(StartFailure),
    Unknown,
    EntranceRefused(c::EntranceRefusal),
}
#[must_use]
pub fn ask(limits: &Limits) -> Option<jig_client::Ask> {
    let limits = wire_limits(limits)?;
    let record = w::Requesting::new(&limits, w::RequestingParts { request: w::Request::StartPlayground }).ok()?;
    Some(jig_client::Ask::Family {
        scope: c::Scope::Deployment,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::requesting(&record),
        },
    })
}
/// Outcomes belonging to another operation are undecodable here.
#[must_use]
pub fn answer(value: &jig_client::Answered, limits: &Limits) -> Option<StartAnswer> {
    let record = match value {
        jig_client::Answered::Family(record) => record,
        jig_client::Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(StartAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?));
        }
        jig_client::Answered::Jig(c::Answer::Done(_)) => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let value = w::Answer::decode(&wire_limits(limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match value.into_parts().outcome {
        w::Outcome::Running => StartAnswer::Running,
        w::Outcome::Unknown => StartAnswer::Unknown,
        w::Outcome::StartRefused(value) => match value.into_parts().why {
            w::StartRefusal::NotOffered => StartAnswer::NotOffered,
        },
        w::Outcome::StartFailed(value) => StartAnswer::Failed(match value.into_parts().why {
            w::StartFailure::Group(value) => StartFailure::Group(value.into_parts().why),
            w::StartFailure::Account => StartFailure::Account,
            w::StartFailure::Adoption => StartFailure::Adoption,
            w::StartFailure::NotConnected => StartFailure::NotConnected,
        }),
        w::Outcome::Removed
        | w::Outcome::Looked
        | w::Outcome::Dismissed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::DeviceAdded(_)
        | w::Outcome::DeviceRemoved
        | w::Outcome::DeviceRefused(_)
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged
        | w::Outcome::FindingRefused(_)
        | w::Outcome::Drafted(_)
        | w::Outcome::DraftRefused(_)
        | w::Outcome::Follow(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_) => return None,
    })
}
