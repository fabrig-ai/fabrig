//! The playground read, start and reset (temper/docs/design/draft/playground-reset.md, 6.1–6.2).
//! The retained halves follow temper/docs/design/client/README.md, 4.8.
use super::{Demo, Limits, Request, Side, family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Link, Readable, Readout, Told, contract as c};
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
    pub settled: bool,
    pub deployment: Option<(Handle, u32)>,
    pub ended: Option<RetiredDemo>,
    pub model_ended: bool,
    pub forge_ended: bool,
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
/// Identity retained when the playground ends (client/README.md, 5.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredDemo {
    pub forge: u16,
    pub provider: u32,
    pub project: u32,
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

    /// The demo forge serves no pages, including after its end (playground.md, 6).
    #[must_use]
    pub fn serves_pages(&self, forge: u16) -> bool {
        match self.demo.answer.as_ref() {
            Some(demo) => demo.forge != forge,
            None => match self.demo.ended {
                Some(demo) => demo.forge != forge,
                None => true,
            },
        }
    }

    /// The shared client's key for the playground's request, while it retries.
    #[must_use]
    pub fn demo_request(&self) -> Option<Token> {
        Some(self.demo.start.as_ref()?.request)
    }
    /// The retired playground, including an end observed in this session.
    #[must_use]
    pub fn retired_demo(&self) -> Option<RetiredDemo> {
        if self.demo.forge_ended { self.demo.ended } else { None }
    }

    /// The playground's provider, retained after its end for account-offer policy.
    #[must_use]
    #[expect(clippy::manual_map, reason = "step code reads retained identity without closures")]
    pub fn demo_provider(&self) -> Option<u32> {
        match self.demo.answer.as_ref() {
            Some(demo) => Some(demo.provider),
            None => match self.demo.ended {
                Some(demo) => Some(demo.provider),
                None => None,
            },
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
            settled: false,
            deployment: None,
            ended: None,
            model_ended: false,
            forge_ended: false,
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
            if result == Some(StartAnswer::Running) && side.demo.selection == ProjectSelection::Starting {
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
/// Whether the demo's model and forge still supply the deployment's work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Halves {
    pub model: bool,
    pub forge: bool,
}
#[must_use]
pub fn halves(
    answer: Option<&Demo>,
    accounts: Option<&c::OwnerAccounts>,
    deployment: Option<&c::OwnerDeployment>,
    forges: &super::forges::State,
) -> Halves {
    let mut halves = Halves { model: false, forge: false };
    let Some(answer) = answer else { return halves };
    if answer.state == DemoState::Ended || answer.state == DemoState::Resetting {
        return halves;
    }
    if let super::forges::State::Rows(rows) = forges {
        halves.forge = true;
        for row in rows {
            if row.number != answer.forge
                && let super::forges::Connection::Connected { .. } = row.connection
            {
                halves.forge = false;
                break;
            }
        }
    }
    let Some(accounts) = accounts else { return halves };
    if accounts.cut {
        return halves;
    }
    for line in &accounts.accounts {
        if line.provider != Some(answer.provider) && line.cut {
            return halves;
        }
    }
    halves.model = !other_model_usable(accounts, deployment, answer.provider);
    halves
}

pub(crate) fn current(side: &Side, accounts: Option<&c::OwnerAccounts>, client: &Client) -> Halves {
    let mut halves = halves(side.demo.answer.as_ref(), accounts, owner_deployment(side, client), &side.forges);
    halves.model &= !side.demo.model_ended;
    halves.forge &= !side.demo.forge_ended;
    halves
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

/// A usable model of a provider other than the demo's: one of the person's `accounts`
/// lines, or, for an owner, a deployment-held account of its delivered, uncut
/// `deployment` document (client/README.md, 3.5 and 4.8).
#[must_use]
pub fn other_model_usable(
    accounts: &c::OwnerAccounts,
    deployment: Option<&c::OwnerDeployment>,
    demo_provider: u32,
) -> bool {
    for line in &accounts.accounts {
        if usable_account(line, Some(demo_provider)) {
            return true;
        }
    }
    match deployment {
        Some(deployment) => deployment_model(deployment, demo_provider).is_some(),
        None => false,
    }
}

/// The first deployment-held account of another provider than the demo's that is usable
/// and uncut, in an owner's delivered, uncut `deployment` (client/README.md, 3.5, 4.8).
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn deployment_model(deployment: &c::OwnerDeployment, demo_provider: u32) -> Option<&c::OwnerAccount> {
    if deployment.cut {
        return None;
    }
    for account in &deployment.accounts {
        if !account.cut
            && account.holder == c::OwnerAccountHolder::Deployment
            && account.provider != Some(demo_provider)
            && usable_sources(&account.sources)
        {
            return Some(account);
        }
    }
    None
}

/// The owner's delivered, uncut deployment, through the demo's one live watch.
#[must_use]
pub fn owner_deployment<'a>(side: &Side, client: &'a Client) -> Option<&'a c::OwnerDeployment> {
    if !owner(client) {
        return None;
    }
    let (handle, _) = side.demo.deployment?;
    let document = client.document(handle)?;
    if document.state != jig_client::State::Live {
        return None;
    }
    let Some(Latest::Jig(c::Delivery::OwnerDeployment(deployment))) = document.latest else { return None };
    if deployment.cut {
        return None;
    }
    Some(deployment)
}

/// Observe positive evidence of an end; unavailable or cut documents are not ends.
/// Session latches keep a lost credential from bringing the model half back.
pub(crate) fn progress(
    side: &mut Side,
    accounts: Option<&c::OwnerAccounts>,
    client: &Client,
    out: &mut Queue<Request>,
) {
    watch_deployment(side, client, out);
    observe_end(side, accounts, client, out);
    remove(side, client, out);
    watch_deployment(side, client, out);
}

fn observe_end(side: &mut Side, accounts: Option<&c::OwnerAccounts>, client: &Client, out: &mut Queue<Request>) {
    let Some(demo) = side.demo.answer.as_ref() else { return };
    if demo.state == DemoState::Ended || demo.state == DemoState::Resetting {
        return;
    }
    let model_ended = demo.state == DemoState::Running
        && match accounts {
            Some(accounts) => other_model_usable(accounts, owner_deployment(side, client), demo.provider),
            None => false,
        };
    let mut forge_ended = false;
    if let super::forges::State::Rows(rows) = &side.forges {
        for row in rows {
            if row.number != demo.forge
                && let super::forges::Connection::Connected { .. } = row.connection
            {
                forge_ended = true;
                break;
            }
        }
    }
    let reread = forge_ended && !side.demo.forge_ended;
    let changed = (model_ended && !side.demo.model_ended) || reread;
    side.demo.model_ended |= model_ended;
    side.demo.forge_ended |= forge_ended;
    if model_ended || forge_ended {
        // Retain the identity after the engine retires the demo or the owner watch closes.
        side.demo.ended = Some(RetiredDemo { forge: demo.forge, provider: demo.provider, project: demo.project });
    }
    if reread {
        read(side, out);
    }
    if changed {
        side.changed();
    }
}

fn watch_deployment(side: &mut Side, client: &Client, out: &mut Queue<Request>) {
    let needed = owner(client) && (side.demo_running() || (side.demo.model_ended && !side.demo.settled));
    if !needed {
        if let Some((handle, _)) = side.demo.deployment.take() {
            out.push(Request::Client { asked: jig_client::Event::Unwatch { handle } });
        }
        return;
    }
    if side.demo.deployment.is_some() {
        return;
    }
    let provider = match side.demo.answer.as_ref() {
        Some(demo) => demo.provider,
        None => match side.demo.ended {
            Some(demo) => demo.provider,
            None => return,
        },
    };
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.demo.deployment = Some((handle, provider));
    out.push(Request::Client {
        asked: jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
    });
}

fn owner(client: &Client) -> bool {
    let Some(document) = client.document(super::me::handle()) else { return false };
    if document.state != jig_client::State::Live {
        return false;
    }
    let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return false };
    me.deployment_role == Some(c::HeldRole::Owner)
}

/// Only the model's end needs client cleanup; the engine owns cleanup at the forge's end.
fn remove(side: &mut Side, client: &Client, out: &mut Queue<Request>) {
    if side.demo.forge_ended {
        side.demo.settled = true;
    }
    if side.demo.settled || !side.demo.model_ended || !owner(client) {
        return;
    }
    let Some((_, provider)) = side.demo.deployment else { return };
    let Some(deployment) = owner_deployment(side, client) else { return };
    for account in &deployment.accounts {
        if account.provider == Some(provider) && account.holder == c::OwnerAccountHolder::Deployment {
            side.serial = side.serial.checked_add(1).expect("forge request serial fits");
            // No pending page operation owns this key: every outcome is dropped.
            out.push(Request::Client {
                asked: jig_client::Event::Send {
                    key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: side.serial },
                    name: None,
                    request: jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                        holder: c::AccountHolder::Deployment,
                        change: c::AccountChange::Remove { account: account.number },
                    })),
                },
            });
            break;
        }
    }
    side.demo.settled = true;
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
    Ended,
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
    if side.party.is_none()
        || side.served == super::Served::OutOfDate
        || side.demo.start.is_some()
        || side.retired_demo().is_some()
        || side.leaving()
    {
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
    let mut answer = match page {
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
    let retired = match answer.as_ref() {
        Some(demo) if demo.state == DemoState::Ended => {
            Some(RetiredDemo { forge: demo.forge, provider: demo.provider, project: demo.project })
        }
        _ => None,
    };
    let retirement_changed = retired.is_some() && side.retired_demo() != retired;
    if let Some(retired) = retired {
        side.demo.ended = Some(retired);
        side.demo.model_ended = true;
        side.demo.forge_ended = true;
        side.demo.settled = true;
        side.demo.start = None;
        side.demo.failure = None;
        answer = None;
    }
    let first = !side.demo.answered;
    side.demo.answered = true;
    if first || side.demo.answer != answer || retirement_changed {
        side.demo.answer = answer;
        side.changed();
    }
}
