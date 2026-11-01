//! The playground read, start and reset (temper/docs/design/draft/playground-reset.md, 6.1–6.2).
use super::{Demo, Limits, Request, Side, family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Duration, Queue, RequestProgress, Time, Token};

use temper_client_protocol_wire::{self as wire, v1 as w};

const RESET_WAIT: Duration = Duration::from_secs(2);

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent session lifecycle facts (client/README.md, 4.8)")]
pub(super) struct State {
    pub answer: Option<Demo>,
    pub pending: Option<Handle>,
    pub live: bool,
    pub answered: bool,
    pub start: Option<Starting>,
    pub failure: Option<StartFailure>,
    pub ending: Option<Ending>,
    pub retry_at: Option<Time>,
    pub reset_navigation: bool,
    pub reset_opened: bool,
    pub selection: ProjectSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProjectSelection {
    None,
    Starting,
    Reading,
    Ready(u32),
}
#[derive(Debug)]
pub(super) struct Starting {
    pub request: Token,
}
/// The reset request retained through its terminal answer (temper/docs/design/draft/playground-reset.md, 6.2).
#[derive(Debug)]
pub struct Ending {
    pub request: Token,
    pub progress: Option<RequestProgress>,
    pub answer: Option<EndAnswer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndAnswer {
    Reset,
    NotOffered,
    Unknown,
    EntranceRefused(c::EntranceRefusal),
}
impl Side {
    #[must_use]
    pub fn leaving(&self) -> bool {
        if let Some(ending) = &self.demo.ending
            && ending.progress.is_some()
        {
            return true;
        }
        resetting(self)
    }
    #[must_use]
    pub fn playground_running(&self) -> bool {
        let running = match self.demo.answer.as_ref() {
            Some(demo) => demo.state == DemoState::Running,
            None => false,
        };
        running && !self.leaving()
    }
    #[must_use]
    pub fn ending(&self) -> Option<&Ending> {
        self.demo.ending.as_ref()
    }
    pub(crate) fn select_playground_on_start(&mut self) {
        self.demo.selection = ProjectSelection::Starting;
    }
    pub(crate) fn playground_project(&self) -> Option<u32> {
        match self.demo.selection {
            ProjectSelection::Ready(project) => Some(project),
            ProjectSelection::None | ProjectSelection::Starting | ProjectSelection::Reading => None,
        }
    }
    pub(crate) fn playground_project_selected(&mut self) {
        self.demo.selection = ProjectSelection::None;
    }
    pub(crate) fn take_reset_navigation(&mut self) -> bool {
        core::mem::replace(&mut self.demo.reset_navigation, false)
    }

    /// The playground's forge serves no pages while it runs or leaves.
    #[must_use]
    pub fn serves_pages(&self, forge: u16) -> bool {
        match self.demo.answer.as_ref() {
            Some(demo) if self.demo_running() || self.leaving() => demo.forge != forge,
            Some(_) | None => true,
        }
    }

    /// The shared client's key for the playground's request, while it retries.
    #[must_use]
    pub fn demo_request(&self) -> Option<Token> {
        Some(self.demo.start.as_ref()?.request)
    }
    /// The playground's provider from its current answer.
    #[must_use]
    #[expect(clippy::manual_map, reason = "step code reads the answer without closures")]
    pub fn demo_provider(&self) -> Option<u32> {
        match self.demo.answer.as_ref() {
            Some(demo) => Some(demo.provider),
            None => None,
        }
    }
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
            ending: None,
            retry_at: None,
            reset_navigation: false,
            reset_opened: false,
            selection: ProjectSelection::None,
        }
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
                read_answer(side, page, limits);
            }
            true
        }
        Told::Answered { key, outcome } if end_key(side, *key) => {
            let answer = match outcome {
                jig_client::Outcome::Answered(value) => end_answer(value, &limits).unwrap_or(EndAnswer::Unknown),
                jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Held
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Unsent
                | jig_client::Outcome::Full { .. }
                | jig_client::Outcome::Lost
                | jig_client::Outcome::Unconfirmed => EndAnswer::Unknown,
            };
            if answer == EndAnswer::Reset {
                reset_done(side);
                side.demo.answer = None;
                side.demo.retry_at = None;
            }
            let ending = side.demo.ending.as_mut().expect("matching end request");
            ending.progress = None;
            ending.answer = Some(answer);
            read(side, out);
            side.changed();
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
            let select_project = match &result {
                Some(StartAnswer::Running | StartAnswer::Failed(_)) => true,
                Some(StartAnswer::NotOffered | StartAnswer::Unknown | StartAnswer::EntranceRefused(_)) | None => false,
            };
            if select_project && side.demo.selection == ProjectSelection::Starting {
                side.demo.selection = ProjectSelection::Reading;
            }
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
    let record =
        w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).expect("fieldless demo read");
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

/// The engine's offered demo state, independent of a start sent by this session.
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
        w::Outcome::Made(_)
        | w::Outcome::Full
        | w::Outcome::NotOffered
        | w::Outcome::Removing
        | w::Outcome::Playground
        | w::Outcome::Reset
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
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
    if side.party.is_none() || side.served == super::Served::OutOfDate || side.demo.start.is_some() || side.leaving() {
        return;
    }
    let Some(demo) = side.demo.answer.as_ref() else { return };
    if demo.state == DemoState::Running && side.demo.failure.is_none() {
        return;
    }
    let Some(ask) = ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.demo.start = Some(Starting { request });
    side.demo.failure = None;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}

fn reset_done(side: &mut Side) {
    if !side.demo.reset_opened {
        side.demo.reset_navigation = true;
        side.demo.reset_opened = true;
    }
}
/// Poll a resetting read without overlapping reads (temper/docs/design/draft/playground-reset.md, 6.2).
pub(crate) fn poll(side: &mut Side, now: Time, out: &mut Queue<Request>) {
    if !side.demo.live || side.party.is_none() || side.served == super::Served::OutOfDate {
        side.demo.retry_at = None;
        return;
    }
    if !resetting(side) || side.demo.pending.is_some() {
        return;
    }
    match side.demo.retry_at {
        Some(at) if now >= at => {
            side.demo.retry_at = None;
            read(side, out);
        }
        Some(_) => {}
        None => side.demo.retry_at = now.checked_add(RESET_WAIT),
    }
}
#[must_use]
pub fn end_ask(limits: &Limits) -> Option<jig_client::Ask> {
    let limits = wire_limits(limits)?;
    let record = w::Requesting::new(&limits, w::RequestingParts { request: w::Request::EndPlayground }).ok()?;
    Some(jig_client::Ask::Family {
        scope: c::Scope::Deployment,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::requesting(&record),
        },
    })
}
#[must_use]
pub fn end_answer(value: &jig_client::Answered, limits: &Limits) -> Option<EndAnswer> {
    let record = match value {
        jig_client::Answered::Family(record) => record,
        jig_client::Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(EndAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?));
        }
        jig_client::Answered::Jig(c::Answer::Done(_)) => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    match answer.into_parts().outcome {
        w::Outcome::Reset => Some(EndAnswer::Reset),
        w::Outcome::NotOffered => Some(EndAnswer::NotOffered),
        w::Outcome::Unknown => Some(EndAnswer::Unknown),
        w::Outcome::Made(_)
        | w::Outcome::Full
        | w::Outcome::Removing
        | w::Outcome::Playground
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Returning => None,
    }
}
fn end_key(side: &Side, key: jig_client::Key) -> bool {
    if key.asker != crate::owner::Owner::Forge.asker() {
        return false;
    }
    match &side.demo.ending {
        Some(ending) => ending.request.raw() == key.number && ending.progress.is_some(),
        None => false,
    }
}
pub(crate) fn end(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == super::Served::OutOfDate || !side.playground_running() {
        return;
    }
    let Some(ask) = end_ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.demo.ending = Some(Ending { request, progress: Some(RequestProgress::Parked), answer: None });
    side.demo.reset_opened = false;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
pub(crate) fn end_progress(side: &mut Side, request: Token, progress: RequestProgress) {
    if let Some(ending) = &mut side.demo.ending
        && ending.request == request
        && ending.progress.is_some()
    {
        ending.progress =
            Some(if side.served == super::Served::OutOfDate { RequestProgress::Parked } else { progress });
        side.changed();
    }
}

fn resetting(side: &Side) -> bool {
    match side.demo.answer.as_ref() {
        Some(demo) => demo.state == DemoState::Resetting,
        None => false,
    }
}
fn read_answer(side: &mut Side, page: &Readout, limits: Limits) {
    let answer = match page {
        Readout::Family { family: number, record: Some(record) } if *number == wire::FORGE => {
            match wire_limits(&limits) {
                Some(limits) => family::demo(record, &limits),
                None => None,
            }
        }
        Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
    };
    let was_resetting = resetting(side);
    if let Some(demo) = &answer {
        if was_resetting && demo.state == DemoState::NotStarted {
            reset_done(side);
        }
        if demo.state == DemoState::Running {
            side.demo.reset_opened = false;
            if side.demo.selection == ProjectSelection::Reading && demo.project != 0 {
                side.demo.selection = ProjectSelection::Ready(demo.project);
            }
        }
    }
    side.demo.retry_at = None;
    let first = !side.demo.answered;
    side.demo.answered = true;
    if first || side.demo.answer != answer {
        side.demo.answer = answer;
        side.changed();
    }
}
