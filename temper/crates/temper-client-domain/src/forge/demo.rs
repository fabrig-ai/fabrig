//! The playground's read and start (temper/docs/design/draft/playground-reset.md, 6.1).
//! Its reset belongs to pg-playground-banner.
use super::{Demo, Limits, Request, Side, family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub(super) struct State {
    pub answer: Option<Demo>,
    pub pending: Option<Handle>,
    pub live: bool,
    pub answered: bool,
    pub start: Option<Starting>,
    pub failure: Option<StartFailure>,
    pub select: bool,
}
#[derive(Debug)]
pub(super) struct Starting {
    pub request: Token,
}
impl Side {
    /// The playground's forge serves no pages (playground.md, 6).
    #[must_use]
    pub fn serves_pages(&self, forge: u16) -> bool {
        match self.demo.answer.as_ref() {
            Some(demo) => demo.forge != Some(forge),
            None => true,
        }
    }

    pub(crate) fn playground_project(&self) -> Option<u32> {
        if !self.demo.select {
            return None;
        }
        let answer = self.demo.answer.as_ref()?;
        if answer.state != DemoState::Running {
            return None;
        }
        answer.project
    }

    pub(crate) fn playground_selected(&mut self) {
        self.demo.select = false;
    }

    /// The shared client's key for the playground's request, while it retries.
    #[must_use]
    pub fn demo_request(&self) -> Option<Token> {
        Some(self.demo.start.as_ref()?.request)
    }
    /// The playground's provider, for account-offer policy.
    #[must_use]
    pub fn demo_provider(&self) -> Option<u32> {
        Some(self.demo.answer.as_ref()?.provider)
    }
}

impl State {
    pub const fn new() -> Self {
        Self { answer: None, pending: None, live: false, answered: false, start: None, failure: None, select: false }
    }
}
/// A link tell remains available to the other children; every forge read page is consumed.
pub(crate) fn told(side: &mut Side, told: &Told, limits: Limits, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.demo.live {
                side.demo.live = true;
                read(side, out);
            } else if *link != Link::Live {
                side.demo.live = false;
            }
            false
        }
        Told::Page { handle, page } if handle.asker == crate::owner::Owner::Forge.asker() => {
            if Some(*handle) == side.demo.pending {
                side.demo.pending = None;
                let answer = match page {
                    Readout::Family { family: number, record: Some(record) } if *number == wire::FORGE => {
                        match wire_limits(&limits) {
                            Some(limits) => family::playground(record, &limits),
                            None => None,
                        }
                    }
                    Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
                };
                let first = !side.demo.answered;
                side.demo.answered = true;
                if first || side.demo.answer != answer {
                    side.demo.answer = answer;
                    side.changed();
                }
            }
            true
        }
        Told::Answered { key, outcome } if start_key(side, *key) => {
            let result = match outcome {
                jig_client::Outcome::Answered(value) => answer(value, &limits),
                jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Held
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Unsent
                | jig_client::Outcome::Full { .. }
                | jig_client::Outcome::Lost
                | jig_client::Outcome::Unconfirmed => Some(StartAnswer::Unknown),
            };
            side.demo.start = None;
            side.demo.failure = match result {
                Some(StartAnswer::Failed(why)) => Some(why),
                Some(
                    StartAnswer::Running
                    | StartAnswer::NotOffered
                    | StartAnswer::Unknown
                    | StartAnswer::EntranceRefused(_),
                )
                | None => None,
            };
            read(side, out);
            side.changed();
            true
        }
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
/// A usable source of a provider other than the demo's, or any provider without a demo.
#[must_use]
pub fn usable_account(line: &c::OwnerAccountLine, demo_provider: Option<u32>) -> bool {
    if demo_provider.is_some() && line.provider == demo_provider {
        return false;
    }
    usable_sources(&line.sources)
}

fn usable_sources(sources: &skein_lib::List<c::OwnerAccountSource>) -> bool {
    for source in sources {
        if source.usable {
            return true;
        }
    }
    false
}

fn read(side: &mut Side, out: &mut Queue<Request>) {
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.demo.pending = Some(handle);
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground })
        .expect("fieldless playground read");
    out.push(Request::Client {
        asked: jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Deployment,
                // Window has no minimum row count; this read has no paged rows.
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

/// The engine's offered playground state, independent of a start sent by this session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoState {
    NotStarted,
    Running,
    Resetting,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartFailure {
    Group(Box<[u8]>),
    Account,
    Adoption,
    NotConnected,
    Project,
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
            w::StartFailure::Project => StartFailure::Project,
        }),
        w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Removing
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::EndRefused(_)
        | w::Outcome::Returning => return None,
    })
}
fn start_key(side: &Side, key: jig_client::Key) -> bool {
    if key.asker != crate::owner::Owner::Forge.asker() {
        return false;
    }
    match &side.demo.start {
        Some(start) => start.request.raw() == key.number,
        None => false,
    }
}
pub(crate) fn start(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == super::Served::OutOfDate || side.demo.start.is_some() {
        return;
    }
    let Some(demo) = side.demo.answer.as_ref() else { return };
    if demo.state == DemoState::Resetting || (demo.state == DemoState::Running && side.demo.failure.is_none()) {
        return;
    }
    let Some(ask) = ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.demo.start = Some(Starting { request });
    side.demo.select = true;
    side.demo.failure = None;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
