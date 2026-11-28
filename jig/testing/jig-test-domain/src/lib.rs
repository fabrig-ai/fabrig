#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
//! The testing application's root. It owns two numbered test connectors and
//! one commit door; the core owns every cross-child decision. The root only
//! translates connector values and wraps records (`domain/root.md`, 3–4).
//!
//! | State | Input | Next state | Output |
//! | --- | --- | --- | --- |
//! | open | admitted core or connector event | decision held | at most one commit |
//! | decision held | committed number | durable | held outputs may leave |
//! | open | failed number | stopped | stop, no later delivery |

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
use jig_domain_contract::{self as jig, Connector, Declaration, resources::ResourceKind};
use jig_domain_core as core;
use jig_domain_core::engine::{self, Admit, Engine, In, store as engine_store};
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decision, Env, Journal, List, Map, Queue, ReplyTo, Token, Wall};

/// A store address owned by the core or one numbered connector.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Key {
    /// A core or child address.
    Core(core::Key),
    /// A connector-owned address.
    Connector { number: u16, key: connector::RecordKey },
}

/// A durable row owned by the core or one numbered connector.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Record {
    /// A core or child row.
    Core(core::Record),
    /// A connector-owned row.
    Connector { number: u16, record: connector::Record },
}

impl Record {
    /// The row's stable store address.
    #[must_use]
    pub fn key(&self) -> Key {
        match self {
            Self::Core(row) => Key::Core(row.key()),
            Self::Connector { number, record } => Key::Connector { number: *number, key: record.key() },
        }
    }
}

/// One mutation in an atomic decision.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Write {
    /// Drop a task transcript whole (jig/docs/design/domain/people/acts.md, 2; jig/docs/design/protocol/store/transcripts.md, 5, jig/docs/design/protocol/store/growth.md, 1.1).
    DropTranscript { task: u64 },
    /// Drop a task history log whole (jig/docs/design/protocol/store/logs.md, 3 and 9).
    DropHistory { task: u64 },
    /// Copy the old settings revision into history atomically.
    CopySettingsRevision(u32),
    /// The shell-supplied wires, in the start’s own decision.
    Wires,
    /// Replace the row under its key.
    Save(Record),
    /// Remove the row under its key.
    Erase(Key),
}

/// What the journal releases after its commit, or through its read door.
#[derive(Debug)]
pub enum Delivery {
    /// A job and its opaque workspace items, held behind the assignment's commit.
    AssignedJob { channel: Token, job: jig_domain_host::JobAssignment, items: Option<Box<[u8]>> },

    /// The root heard this project made after its commit.
    ProjectMade { project: u32 },
    /// A typed hub record for a worker channel.
    Link { channel: Token, down: jig_domain_host::LinkDown },
    /// The host retries this call after the engine drains.
    CallBusy { channel: Token, task: u64, attempt: u64, name: Box<[u8]> },
    /// A whole connector-owned document released after every reflected commit.
    ApplicationDocument { watcher: Token, number: u16, document: Box<connector::ClientDocument> },
    /// One connector-owned read terminal, held until durable.
    ApplicationRead {
        reply: ReplyTo,
        number: u16,
        answer: Result<connector::ClientDocument, connector::DocumentMissing>,
    },
    /// The connector's own terminal, released by the journal after durability.
    Application { to: ReplyTo, number: u16, outcome: connector::ApplicationOutcome },
    /// One committed whole-tree projection handoff.
    Projection { connector: u16, feed: Box<core::ProjectionFeed> },
    /// One committed inbox word for its current host.
    Message { channel: Token, task: u64, attempt: u64, word: tasks::Word },
    /// The start's continuation, released with the writes of its decision.
    Resume(engine::Resume),
    /// Core-held party, host, task or view output.
    Core(core::Held),
    /// A connector continuation released back to its owner after the commit.
    Connector(Connector, jig::Released),
    /// A held engine load, released after preceding commits.
    Store(engine_store::Out),
    /// A connector-owned load, released after preceding commits.
    ConnectorLoad { number: u16, after: Option<Box<[u8]>> },
    /// An answer with its original opaque call name and durability class.
    CallAnswer {
        channel: Token,
        task: u64,
        attempt: u64,
        name: Box<[u8]>,
        call: core::SettledCall,
        /// Writes wait for commitment; reads use the journal's door.
        writes: bool,
    },
    /// A prepared worker assignment, after the claim is durable.
    Assigned { channel: Token, assignment: Assignment },
    /// A post-commit continuation inside the core's fleet.
    Fleet(fleet::Event),
    /// Connector output to its fake system.
    System { connector: u16, call: connector::SystemRequest },
    /// A fresh read owed after this run's loss committed.
    LostRead { connector: u16, task: u64, attempt: u64, resource: tasks::Name, call: connector::SystemRequest },
    /// A procedure is due at its owning connector.
    Procedure { task: u64, step: u64, connector: u16, code: u16, context: Box<tasks::RunContext> },
}

pub use jig_domain_hub_link::Assignment;

/// What the root asks its surrounding world to do.
#[derive(Debug)]
pub enum Request {
    Workers(Box<[core::settings::WorkerSetting]>),
    /// The store's names, replaced before the catalogue decision commits (domain/engine/settings-catalogue.md, 1).
    StoreCatalogue {
        names: Box<[core::settings::ModelName]>,
    },
    /// The clients' catalogue, after the store's replacement (domain/engine/settings-catalogue.md, 1).
    ClientsCatalogue {
        models: Box<[core::settings::CatalogueModel]>,
    },
    /// Read a fenced opaque terminal in the protocol routing.
    ReadResult {
        payload: Token,
        outcome: Box<[u8]>,
    },
    /// Return a connector input to its owner during the drain.
    Busy {
        event: Box<Event>,
    },
    /// Apply one numbered transaction as a whole.
    Commit {
        number: u64,
        writes: Queue<Write>,
    },
    /// Release one output only after its prerequisite commit.
    Deliver(Delivery),
    /// An immediate observation with no durability dependency.
    Now(core::Now),
    /// A failed store commit stopped this root.
    Stop,
}

/// A test input at the root boundary.
#[derive(Debug)]
pub enum Event {
    /// An input from a scripted engine peer.
    Engine(In),
    /// A channel opening, record or loss in the hub vocabulary.
    Link { channel: Token, link: link::Event },
    /// The protocol routing read a fenced result.
    ResultRead { payload: Token, end: tasks::End },
    /// The service begins its drain.
    Stop,
    /// Prepare and watch the application's typed subject in the client's declared scope.
    WatchApplication {
        watcher: Token,
        sign_in: u64,
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        subject: connector::ClientSubject,
        window: core::documents::Window,
    },
    /// Prepare and read one typed connector document without a key.
    ReadApplication {
        reply: ReplyTo,
        sign_in: u64,
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        subject: connector::ClientSubject,
        window: core::documents::Window,
    },
    /// A protocol-decoded application value with its independently computed
    /// digest, and the number and digest of a value it carried, already kept
    /// in the secrets' store (`protocol/client/README.md`, section 3).
    Application {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        digest: [u8; 32],
        value: connector::ApplicationRequest,
        carried: Option<core::Carried>,
    },
    /// A correlated reply to the connector's fresh read after a lost writer.
    WriterRead { connector: u16, resource: tasks::Name, event: connector::SystemEvent },
    /// A numbered connector's independent retry deadline elapsed.
    ConnectorTimer { number: u16 },
    /// Begin the start at the store's open.
    StartBegin { pending: bool },
    /// A page of a numbered connector's own live records.
    ConnectorPage { number: u16, records: Box<[connector::Record]>, after: Option<Box<[u8]>> },
    /// The store refused a page with its own typed cause; engine loads also receive `Failed`.
    StartPageRefused { load: Option<Token>, failure: jig_protocol_store::restore::Failure },
    /// Drive one due child timer through the same decision barrier.
    Timer(core::Timer),
    /// A party, host, account or other event in jig's vocabulary.
    Core(core::Event),
    /// An answer on the engine’s store port.
    Store(engine_store::In),
    /// A generation named by the application's credential source.
    Credential { number: u32, generation: u64, valid: skein_lib::Duration },
    /// One numbered connector's input from its system or a scripted peer.
    Connector { number: u16, event: connector::Event },
    /// A scripted connector's projection write for a tracked goal.
    ProjectionEffect { goal: u64, number: u16, effect: connector::Effect },
    /// A decoded call for an offered topic.
    TopicCall { to: ReplyTo, key: core::CallKey, number: u16, topic: u16 },
    /// A decoded named effect call, whose payload stays with its connector.
    EffectCall {
        to: ReplyTo,
        key: core::CallKey,
        number: u16,
        effect: connector::Effect,
        deadline: Wall,
        proposal: Option<Box<[u8]>>,
    },
    /// The rendered answer to a read or an undecodable call, with no durable call record.
    CallAnswered { to: ReplyTo, call: core::SettledCall },
    /// A scripted worker's provisional writing, named by its host.
    Text { task: u64, attempt: u64, turn: u32, offset: u32, text: Box<[u8]> },
    /// A scripted worker's retained turn, including its opaque body.
    Turn {
        channel: Token,
        task: u64,
        attempt: u64,
        turn: u32,
        cumulative: u64,
        read: Option<u64>,
        opens_window: bool,
        refused: Box<[u64]>,
        transcript: Box<[u8]>,
    },
    /// A scripted worker's retained terminal answer.
    Answer {
        channel: Token,
        task: u64,
        attempt: u64,
        cumulative: u64,
        end: tasks::End,
        read: Option<u64>,
        refused: Box<[u64]>,
    },
    /// A bounded store page of prior opaque turns; pages arrive oldest first.
    /// Store acknowledgement of this exact commit number.
    Committed { number: u64, fill: u64 },
    /// A store failure of this exact commit number.
    Failed { number: u64 },
}

impl Event {
    /// A test peer's answer or observation on the connector port.
    #[must_use]
    pub fn connector_up(number: u16, up: jig::Up) -> Self {
        Self::Engine(In::Connector(Connector(number), up))
    }
}

/// The journal's sizes (`domain/root.md` 4, 10).
#[derive(Clone, Copy, Debug)]
pub struct JournalSizes {
    /// Commits in flight at once: the store's `sizes::Commits::in_flight` and the watermark's first term.
    pub commits: u32,
    /// Unique writes in one commit: the store's commit edge, `sizes::Commits::writes`.
    pub writes: u32,
    /// Outputs the journal's door holds (`Journal::now`): the watermark's second term.
    pub door: u32,
    /// Held outputs the journal releases in one call (`Journal::uncapped`).
    pub release: u32,
}

/// Child bounds, synchronous route capacity and admission pacing for the test root.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// The core's bounded state and route buffers.
    pub core: core::Limits,
    /// Each test connector's bounded state and output.
    pub connector: connector::Limits,
    /// Journal release pacing and capacities used for conservative pricing.
    pub journal: JournalSizes,
    /// New external work waits while the backlog is at or past this many
    /// entries (skein's `programming-model.md`, 7.1); a pacing limit,
    /// not a memory one.
    pub watermark: u32,
    /// Public admission bounds (jig/docs/design/domain/root.md, 10).
    pub public_watermark: u32,
    pub public_reads: u32,
    /// At most this many synchronous child continuations per decision.
    pub routes: u32,
}

/// The watermark's default: the journal's commits plus the outputs its
/// door holds (`domain/root.md` 10).
#[must_use]
pub fn default_watermark(journal: &JournalSizes) -> u32 {
    journal.commits.checked_add(journal.door).expect("journal commits plus door outputs fit")
}

/// The testing application's two numbered connectors.
#[derive(Debug)]
pub struct Config {
    /// The core's configuration.
    pub core: core::Config,
    /// Connector one.
    pub first: connector::Config,
    /// Connector two.
    pub second: connector::Config,
    /// Resource kinds for connectors one and two, respectively.
    pub resource_kinds: [Box<[ResourceKind]>; 2],
}

impl Config {
    /// Declare just the connectors the core configuration names.
    #[must_use]
    pub fn declarations(&self) -> Box<[Declaration]> {
        let [first, second] = &self.resource_kinds;
        let mut declarations = List::with_capacity(2);
        for number in &self.core.connectors {
            let (config, resources) = match *number {
                1 => (&self.first, first),
                2 => (&self.second, second),
                _ => continue,
            };
            declarations
                .push(routes::declaration(Connector(*number), config, resources))
                .expect("the testing application has two connectors");
        }
        declarations.into_boxed()
    }
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "catalogue release and restart progress are independent")]
pub struct Domain {
    scripted_workspaces: Option<Queue<core::Event>>,
    brief_failures: Queue<(u64, jig_domain_brief::GatherFailure)>,
    hub_link: jig_domain_hub_link::Link,
    engine: Engine,
    first: connector::Domain,
    second: connector::Domain,
    journal: Journal<Write, Delivery>,
    now: Queue<Request>,
    catalogue_pending: bool,
    store_answers: Queue<Box<Event>>,
    busy: Queue<Box<Event>>,
    workspace_writes: Map<u64, Box<[tasks::Name]>>,
    public_reads: u32,
    decision_wrote: bool,
    decision_numbering: Option<u64>,
    stopped_reported: bool,
    start_opened: bool,
    restored_deployment: bool,
    start_phase: Option<core::start::Phase>,
    start_failure: Option<(core::start::Phase, core::start::Failure)>,
    rehearsing: bool,
    rehearsed: bool,
    start_page_failure: Option<(core::start::Phase, jig_protocol_store::restore::Failure)>,
}

impl Domain {
    /// Retain holdings reports and answers so a world can delay, lose or interleave
    /// connector traffic (jig/docs/design/domain/tasks/holds.md, 2).
    /// Delay selected resource observations while still answering holdings in the decision.
    pub fn delay_holding_reports(&mut self, number: u16, choices: Box<[bool]>) {
        numbered(self, number).delay_holding_reports(choices);
    }

    pub fn take_holding_report(&mut self, number: u16) -> Option<In> {
        let up = numbered(self, number).take_holding_report()?;
        Some(In::Connector(Connector(number), jig::Up::Resources(up)))
    }

    /// Hold the test connector's per-answer part asks until the world answers.
    pub fn script_parts(&mut self) {
        self.first.script_parts();
        self.second.script_parts();
    }

    pub fn take_parts(&mut self) -> Option<(u16, core::PartsAsk)> {
        if let Some(ask) = self.first.take_parts() {
            return Some((1, ask));
        }
        Some((2, self.second.take_parts()?))
    }

    /// Hold the fake connector's wait reports to expose the view before delivery.
    pub fn script_wait_reports(&mut self) {
        self.first.script_wait_reports();
        self.second.script_wait_reports();
    }

    pub fn take_wait_report(&mut self) -> Option<In> {
        if let Some(up) = self.first.take_wait_report() {
            return Some(In::Connector(Connector(1), jig::Up::Procedures(up)));
        }
        Some(In::Connector(Connector(2), jig::Up::Procedures(self.second.take_wait_report()?)))
    }

    /// The latest report refusal delivered to the selected fake connector.
    #[must_use]
    pub const fn wait_refusal(&self, number: u16) -> Option<(u64, &'static str)> {
        match number {
            1 => self.first.wait_refusal(),
            2 => self.second.wait_refusal(),
            _ => None,
        }
    }

    /// Hold workspace completions for deterministic preparation interleavings.
    pub fn script_workspaces(&mut self) {
        self.scripted_workspaces = Some(Queue::uncapped());
    }

    /// Fail each brief assembly until the world clears the script.
    pub fn script_brief_failure(&mut self, failure: Option<jig_domain_brief::GatherFailure>) {
        self.first.script_brief_failure(failure);
    }

    /// Take one failure observed by the application's log route.
    pub fn take_brief_failure(&mut self) -> Option<(u64, jig_domain_brief::GatherFailure)> {
        self.brief_failures.pop()
    }

    /// Take one held completion; the world delivers it through the root's entrance.
    pub fn take_workspace(&mut self) -> Option<core::Event> {
        self.scripted_workspaces.as_mut()?.pop()
    }

    /// The composed engine, for the world's timer queries.
    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Drive the engine past the root in a world that scripts a hand-off.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// The composed core, for access not yet provided by the engine's functions.
    #[must_use]
    pub fn core(&self) -> &core::Core {
        self.engine.core()
    }

    /// Let a world supply connector release answers (jig/docs/design/domain/people/acts.md, 2).
    pub fn script_project_releases(&mut self) {
        self.first.script_project_releases();
        self.second.script_project_releases();
    }

    /// One configured connector asked to release a project during this start.
    pub fn take_project_release(&mut self) -> Option<(u16, u32)> {
        if let Some(project) = self.first.take_project_release() {
            return Some((1, project));
        }
        Some((2, self.second.take_project_release()?))
    }

    /// Configure the composed core before the testing application's first step.
    pub fn core_mut(&mut self) -> &mut core::Core {
        self.engine.core_mut()
    }

    /// Borrow the core's settings; the testing root only translates this lookup.
    #[must_use]
    pub fn settings(&self) -> &core::settings::Current {
        &self.engine.core().current
    }

    /// Read the newest value through its owning connector (connectors.md, 17).
    pub fn setting(
        &mut self,
        env: &Env<Limits>,
        connector: u16,
        kind: u32,
        scope: u32,
        number: u32,
    ) -> Option<Box<[u8]>> {
        let token = Token::new(0);
        let mut out = Queue::uncapped();
        connector::step_system(
            numbered(self, connector),
            &env.with(env.limits.connector),
            connector::Event::ReadSetting { token, kind, scope, number },
            &mut out,
        );
        let Some(connector::Request::SettingValue { token: answered, value }) = out.pop() else {
            unreachable!("a setting read returns its value synchronously")
        };
        assert_eq!(answered, token, "the connector answers this read");
        assert!(out.is_empty(), "a setting read has one answer");
        value
    }

    /// The fixture's journal, for numbered writes made outside a root decision.
    #[must_use]
    pub fn journal_mut(&mut self) -> &mut Journal<Write, Delivery> {
        &mut self.journal
    }

    /// Stop this start at the switch, before its mark or any external effects
    /// (`jig/docs/design/domain/testing-fakes.md`, 1).
    pub fn rehearse(&mut self) {
        self.rehearsing = true;
    }

    /// The rehearsal served every phase through the sweep and reached the switch.
    #[must_use]
    pub const fn rehearsed(&self) -> bool {
        self.rehearsed
    }

    /// Whether startup has accepted a stored deployment header.
    #[must_use]
    pub const fn restored_deployment(&self) -> bool {
        self.restored_deployment
    }

    /// The phase last reported on the engine's application port.
    #[must_use]
    pub const fn start_phase(&self) -> Option<core::start::Phase> {
        self.start_phase
    }

    /// The engine's start refusal with the phase it refused at.
    #[must_use]
    pub fn start_failure(&self) -> Option<(Option<core::start::Phase>, core::start::Failure)> {
        let (phase, failure) = self.start_failure.as_ref()?;
        Some((Some(*phase), failure.clone()))
    }

    /// The application's store-page cause and the phase at which it stopped.
    #[must_use]
    pub fn start_page_failure(&self) -> Option<&(core::start::Phase, jig_protocol_store::restore::Failure)> {
        self.start_page_failure.as_ref()
    }

    /// Project the two connector owners' existing absolute deadlines unchanged.
    #[must_use]
    pub fn connector_deadlines(&self) -> [(u16, Option<Wall>); 2] {
        [(1, connector::next_deadline(&self.first)), (2, connector::next_deadline(&self.second))]
    }

    /// The core's retained effect reply deadline, including stale rights.
    #[must_use]
    pub fn effect_deadline(&self) -> Option<Wall> {
        self.engine.core().effect_deadline()
    }

    /// The accounts child, for a world that reads what a restart restored.
    #[must_use]
    pub fn accounts(&self) -> &jig_domain_accounts::Domain {
        &self.engine.core().accounts
    }

    /// Pieces whose kind or offset the core cannot keep (domain/engine/views.md).
    #[must_use]
    pub fn pieces_dropped(&self) -> u64 {
        self.hub_link.pieces_dropped()
    }

    /// Build the testing application without issuing a store request.
    #[must_use]
    pub fn new(config: Config, limits: &Limits) -> Domain {
        assert!(limits.routes > 0, "synchronous route bound is positive");
        let declarations = config.declarations();
        let mut first = connector::Domain::new(config.first, &limits.connector);
        let mut second = connector::Domain::new(config.second, &limits.connector);
        second.address(Connector(2));
        for kind in &config.core.kinds {
            match kind.connector {
                1 => first.settings_owner(),
                2 => second.settings_owner(),
                _ => {} // Engine configuration checks the owners.
            }
        }
        Domain {
            scripted_workspaces: None,
            brief_failures: Queue::uncapped(),
            hub_link: jig_domain_hub_link::Link::new(
                jig_domain_hub_link::Limits {
                    assignments: limits.core.tasks.tasks,
                    payloads: limits.core.fleet.turns.checked_add(limits.core.fleet.attempts).expect("payload room"),
                    turns: limits.core.fleet.turns,
                },
                skein_lib::Rng::new(config.core.seed ^ 0x5455_524e_4255_5359),
            ),
            engine: Engine::new(config.core, &declarations, &limits.core).expect("fixture core configuration"),
            first,
            second,
            journal: Journal::uncapped(limits.journal.release, 0),
            now: Queue::uncapped(),
            catalogue_pending: false,
            store_answers: Queue::uncapped(),
            busy: Queue::uncapped(),
            workspace_writes: Map::with_capacity(limits.core.tasks.tasks),
            public_reads: 0,
            decision_wrote: false,
            decision_numbering: None,
            stopped_reported: false,
            start_opened: false,
            restored_deployment: false,
            start_phase: None,
            start_failure: None,
            rehearsing: false,
            rehearsed: false,
            start_page_failure: None,
        }
    }

    #[must_use]
    pub fn ready(&self) -> bool {
        self.start_opened
    }

    /// The client's current held-task context, admitted by the core's visibility rule.
    #[must_use]
    pub fn escalation(&self, person: u64, task: u64) -> Option<Box<tasks::EscalationContext>> {
        let context = self.engine.core().tasks.escalation(task)?;
        let role = self.engine.core().people.role(person, context.project);
        if self.engine.core().escalation_visible(&context, person, role) { Some(context) } else { None }
    }

    /// Release retired child slots after every complete application iteration.
    pub fn reclaim(&mut self) {
        self.engine.core_mut().reclaim();
    }

    /// Whether no work can advance without another input. An assignment may
    /// still wait for a host whose next hello supplies its capacity.
    #[must_use]
    pub fn quiescent(&self) -> bool {
        self.journal.idle()
            && self.now.is_empty()
            && (!self.ready() || (!self.engine.core().due_ready() && !self.engine.core().finish_owed()))
    }
}

use link::Work;

/// Admit one input and route all synchronous continuations inside one decision.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event) {
    let Ok(event) = stopping::defer(domain, event) else { return };
    match event {
        Event::StartBegin { pending } => {
            decide(domain, env, Work::Engine(In::Store(engine_store::In::Opened { pending })));
        }
        Event::ConnectorPage { number, records, after } => {
            let event = restored_page(domain, number, records, after);
            decide(domain, env, Work::Connector { number, event });
        }
        Event::StartPageRefused { load, failure } => {
            if !domain.start_stopped() && !domain.stopping() {
                let phase = domain.start_phase().expect("page belongs to the start");
                if let Some(load) = load {
                    let result = engine_store::Answer::Failed(engine_store::Failure::Failed);
                    decide(domain, env, Work::Engine(In::Store(engine_store::In::Loaded { load, result })));
                }
                domain.start_page_failure = Some((phase, failure));
            }
        }
        Event::Link { channel, link } => decide(domain, env, Work::Link { channel, link }),
        Event::ResultRead { payload, end } => link::read(domain, env, payload, end),
        Event::Stop => decide(domain, env, Work::Core(core::Event::Stopping)),
        Event::Application { reply_to, sign_in, key, scope, number, digest, value, carried } => {
            decide(domain, env, Work::Application { reply_to, sign_in, key, scope, number, digest, value, carried });
        }
        Event::ConnectorTimer { number } => decide(domain, env, Work::ConnectorFire { number }),
        Event::WriterRead { connector, resource, event } => {
            decide(
                domain,
                env,
                Work::Connector { number: connector, event: connector::Event::WriterRead { resource, event } },
            );
        }
        Event::Committed { number, fill } => {
            domain.journal.committed(number);
            decide(domain, env, Work::Engine(In::Store(engine_store::In::Fill { bytes: fill })));
        }
        Event::Failed { number } => domain.journal.failed(number),
        Event::ProjectionEffect { goal, number, effect } => {
            decide(domain, env, Work::ProjectionEffect { goal, number, effect });
        }
        Event::TopicCall { to, key, number, topic } => decide(domain, env, Work::TopicCall { to, key, number, topic }),
        Event::EffectCall { to, key, number, effect, deadline, proposal } => {
            decide(domain, env, Work::EffectCall { to, key, number, effect, deadline, proposal });
        }
        Event::Timer(timer) => decide(domain, env, Work::Timer(timer)),
        Event::Core(event) | Event::Engine(In::Core(event)) => decide(domain, env, Work::Core(event)),
        Event::Engine(input) => decide(domain, env, Work::Engine(input)),
        Event::Store(answer) => decide(domain, env, Work::Engine(In::Store(answer))),
        Event::Credential { number, generation, valid } => decide(
            domain,
            env,
            Work::Core(core::Event::Account(jig_domain_accounts::Event::Credential { number, generation, valid })),
        ),
        Event::WatchApplication { watcher, sign_in, scope, number, subject, window } => {
            decide(domain, env, Work::WatchApplication { watcher, sign_in, scope, number, subject, window });
        }
        Event::ReadApplication { reply, sign_in, scope, number, subject, window } => {
            decide(domain, env, Work::ReadApplication { reply, sign_in, scope, number, subject, window });
        }
        Event::CallAnswered { to, call } => decide(domain, env, Work::CallAnswered { to, call }),
        Event::Connector { number, event } => decide(domain, env, Work::Connector { number, event }),
        Event::Text { task, attempt, turn, offset, text } => decide(
            domain,
            env,
            Work::Core(core::Event::RunText {
                run: Token::new(task),
                attempt: Token::new(attempt),
                turn,
                // This finite scripted source supplies one initial completion.
                retry: 0,
                offset,
                text,
                call: None,
            }),
        ),
        Event::Turn { channel, task, attempt, turn, cumulative, read, transcript, opens_window, refused } => {
            decide(
                domain,
                env,
                Work::Turn { channel, task, attempt, turn, cumulative, read, transcript, opens_window, refused },
            );
        }
        Event::Answer { channel, task, attempt, cumulative, end, read, refused } => {
            decide(domain, env, Work::Answer { channel, task, attempt, cumulative, end, read, refused });
        }
    }
}

/// Route a scripted engine decision's outputs through the normal journal boundary.
/// Test peers may step the engine several times before giving the decision back.
pub fn accept_engine_outputs(domain: &mut Domain, env: &Env<Limits>, out: Queue<engine::Out>) {
    decide(domain, env, Work::EngineOutputs(out));
}

#[expect(clippy::too_many_lines, reason = "one bounded decision routes the complete root work vocabulary")]
fn decide(domain: &mut Domain, env: &Env<Limits>, first: Work) {
    // A refused start leaves its decision unaccepted; the root takes no other.
    if domain.journal.stopped() || domain.start_stopped() {
        return;
    }
    let mut decision = domain.journal.decide();
    domain.decision_wrote = false;
    let mut finish_only = false;
    let mut work = Queue::uncapped();
    work.push(first);
    for _ in 0..env.limits.routes {
        let Some(next) = work.pop() else {
            if !domain.ready() {
                break;
            }
            let requests = core::finish_decision(domain.engine.core_mut(), &env.with(env.limits.core));
            let done = requests.is_empty();
            route_core(domain, env, &mut decision, requests, &mut work);
            if done && work.is_empty() {
                break;
            }
            continue;
        };
        match next {
            Work::Link { channel, link } => link::receive(domain, channel, link, &mut work),
            Work::WatchApplication { watcher, sign_in, scope, number, subject, window } => {
                let token = domain.hub_link.fresh();
                if number == 1 || number == 2 {
                    work.push(Work::Connector { number, event: connector::Event::DocumentPrepare { token, subject } });
                }
                work.push(Work::Core(core::Event::Watch {
                    watcher,
                    sign_in,
                    subject: core::documents::Watched::Application { scope, connector: number, subject: token },
                    window,
                }));
            }
            Work::ReadApplication { reply, sign_in, scope, number, subject, window } => {
                let token = domain.hub_link.fresh();
                if number == 1 || number == 2 {
                    work.push(Work::Connector { number, event: connector::Event::DocumentPrepare { token, subject } });
                }
                work.push(Work::Core(core::Event::Read {
                    reply,
                    sign_in,
                    read: core::reads::Read::Application { scope, connector: number, read: token, window },
                }));
            }
            Work::Application { reply_to, sign_in, key, scope, number, digest, value, carried } => {
                let owner = domain.hub_link.fresh();
                if number == 1 || number == 2 {
                    work.push(Work::Connector { number, event: connector::Event::ApplicationPrepare { owner, value } });
                }
                work.push(Work::Core(core::Event::Application {
                    reply_to,
                    sign_in,
                    key,
                    scope,
                    connector: number,
                    digest,
                    owner,
                    carried,
                }));
            }
            Work::ConnectorFire { number } => {
                let mut out = Queue::uncapped();
                connector::fire(numbered(domain, number), &env.with(env.limits.connector), &mut out);
                route_connector(domain, &mut decision, number, &mut out, &mut work);
            }
            Work::ProjectionEffect { goal, number, effect } => {
                let owner = domain.engine.core_mut().effect_owner();
                work.push(Work::Connector {
                    number,
                    event: connector::Event::StageEffect {
                        owner,
                        effect,
                        start: Some(jig::effects::EffectOrigin::Projection { goal, entry: None }),
                    },
                });
            }
            Work::TopicCall { to, key, number, topic } => topics::start(domain, to, key, number, topic, &mut work),
            Work::EffectCall { to, key, number, effect, deadline, proposal } => {
                let owner = domain.engine.core_mut().effect_owner();
                work.push(Work::Connector {
                    number,
                    event: connector::Event::StageEffect { owner, effect, start: None },
                });
                let origin = match proposal {
                    Some(reason) => core::EffectOrigin::Propose { to, key, reason, as_holder: false },
                    None => core::EffectOrigin::Call { to, key, deadline },
                };
                work.push(Work::Core(core::Event::EffectStart { owner, connector: number, origin }));
            }
            Work::Timer(timer) => {
                let mut out = Queue::uncapped();
                engine::fire(&mut domain.engine, &env.with(env.limits.core), timer, &mut out);
                route_engine(domain, env, &mut decision, out, &mut work);
            }
            Work::CallAnswered { to, call } => {
                let answer = payload(domain, Payload::SettledCall(call));
                work.push(Work::Core(core::Event::Fleet(fleet::Event::Relayed { to, answer })));
            }
            Work::Core(core::Event::PublicRead { reply, read }) => {
                client_documents::public_start(domain, env, &mut decision, reply, read, &mut work);
            }
            Work::Core(event) => {
                let mut out = Queue::uncapped();
                engine::step(&mut domain.engine, &env.with(env.limits.core), In::Core(event), &mut out);
                route_engine(domain, env, &mut decision, out, &mut work);
            }
            Work::Engine(input) => routes::step_engine(domain, env, &mut decision, input, &mut work),
            Work::EngineOutputs(out) => route_engine(domain, env, &mut decision, out, &mut work),
            Work::Finish => finish_only = true,
            Work::ResumeFleet => {
                let requests = core::resume_fleet(domain.engine.core_mut(), &env.with(env.limits.core));
                route_core(domain, env, &mut decision, requests, &mut work);
            }
            Work::Connector { number, event } => {
                let mut requests = Queue::uncapped();
                let connector = numbered(domain, number);
                connector::step(connector, &env.with(env.limits.connector), event, &mut requests);
                route_connector(domain, &mut decision, number, &mut requests, &mut work);
            }
            Work::ConnectorResume { number } => {
                let mut requests = Queue::uncapped();
                connector::resume(numbered(domain, number), &mut requests);
                route_connector(domain, &mut decision, number, &mut requests, &mut work);
            }
            Work::Turn { channel, task, attempt, turn, cumulative, read, transcript, opens_window, refused } => {
                let body = Payload::Turn { task, attempt, turn, cumulative, read, transcript, opens_window, refused };
                messages::receive(domain, &mut work, channel, body);
            }
            Work::Answer { channel, task, attempt, cumulative, end, read, refused } => {
                let body =
                    Payload::Answer { task, attempt, cumulative, end: link::Ending::Scripted(end), read, refused };
                messages::receive(domain, &mut work, channel, body);
            }
        }
    }
    assert!(work.is_empty(), "decision_work_finished: synchronous hand-offs finish before the next entrance");
    if finish_only {
        assert!(
            !domain.decision_wrote && !domain.engine.core().counters.dirty(),
            "stopping continuation makes no commit"
        );
    }
    if let Some(header) = domain.engine.core_mut().decision_header(domain.decision_wrote) {
        write(domain, &mut decision, Write::Save(Record::Core(header)));
    }
    if !domain.start_stopped() {
        domain.journal.accept(decision);
        if let Some(commits) = domain.decision_numbering.take() {
            domain.journal.restore_numbering(commits);
            domain.restored_deployment = true;
        }
    }
}

impl Domain {
    fn start_stopped(&self) -> bool {
        self.start_page_failure.is_some() || self.start_failure.is_some()
    }
}

fn payload(domain: &mut Domain, value: Payload) -> Token {
    domain.hub_link.retain(value)
}

fn numbered(domain: &mut Domain, number: u16) -> &mut connector::Domain {
    match number {
        1 => &mut domain.first,
        2 => &mut domain.second,
        _ => unreachable!("testing application has exactly two connectors"),
    }
}

/// Drain at most one commit and all currently durable outputs.
pub fn release(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    admission::resume(domain, env);
    stopping::release(domain, out);

    if domain.journal.stopped() || domain.start_stopped() {
        if !domain.stopped_reported {
            domain.stopped_reported = true;
            out.push(Request::Stop);
        }
        return;
    }
    if domain.catalogue_pending {
        domain.catalogue_pending = false;
        for _ in 0..domain.now.len() {
            let request = domain.now.pop().expect("catalogue output count");
            out.push(request);
        }
    }
    if let Some(commit) = domain.journal.commit() {
        out.push(Request::Commit { number: commit.number, writes: commit.writes });
    }
    let mut ready = Queue::uncapped();
    let _released = domain.journal.release(&mut ready);
    for _ in 0..ready.len() {
        let delivery = ready.pop().expect("release count");
        match delivery {
            Delivery::Resume(resume) => decide(domain, env, Work::Engine(In::Resume(resume))),
            Delivery::Store(engine_store::Out::Switch) if domain.rehearsing => domain.rehearsed = true,
            Delivery::Core(held) => release_core(domain, env, held, out),
            Delivery::Connector(number, released) => match number.0 {
                1 => decide(domain, env, Work::Connector { number: 1, event: connector::Event::Released(released) }),
                2 => decide(domain, env, Work::Connector { number: 2, event: connector::Event::Released(released) }),
                _ => unreachable!("engine releases to a configured connector"),
            },
            Delivery::Fleet(event) => step(domain, env, Event::Core(core::Event::Fleet(event))),
            other @ (Delivery::Store(_)
            | Delivery::ConnectorLoad { .. }
            | Delivery::ApplicationDocument { .. }
            | Delivery::AssignedJob { .. }
            | Delivery::Link { .. }
            | Delivery::ApplicationRead { .. }
            | Delivery::Application { .. }
            | Delivery::ProjectMade { .. }
            | Delivery::Projection { .. }
            | Delivery::Message { .. }
            | Delivery::CallBusy { .. }
            | Delivery::CallAnswer { .. }
            | Delivery::Assigned { .. }
            | Delivery::System { .. }
            | Delivery::LostRead { .. }
            | Delivery::Procedure { .. }) => out.push(Request::Deliver(other)),
        }
    }
    for _ in 0..domain.now.len() {
        let now = domain.now.pop().expect("bounded now count");
        out.push(now);
    }
    continue_ready(domain, env);
}

fn release_core(domain: &mut Domain, env: &Env<Limits>, held: core::Held, out: &mut Queue<Request>) {
    if let core::Held::PublicWait { read } = held {
        step(domain, env, Event::Core(core::Event::PublicDurable { read }));
    } else if let core::Held::PublicRead { .. } = held {
        domain.public_reads = domain.public_reads.checked_sub(1).expect("one answer per admitted read");
        out.push(Request::Deliver(Delivery::Core(held)));
    } else if let core::Held::TaskPartKept { key } = held {
        step(
            domain,
            env,
            Event::Core(core::Event::Fleet(fleet::Event::PartKept {
                run: Token::new(key.task),
                attempt: Token::new(key.attempt),
                part: key.number,
            })),
        );
    } else if let core::Held::TaskTurnKept { task, attempt, turn } = held {
        step(
            domain,
            env,
            Event::Core(core::Event::Fleet(jig_domain_fleet::Event::TurnKept {
                run: Token::new(task),
                attempt: Token::new(attempt),
                turn,
            })),
        );
    } else if let core::Held::TaskTerminalAcknowledged { task, attempt } = held {
        step(
            domain,
            env,
            Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Acknowledge {
                run: Token::new(task),
                attempt: Token::new(attempt),
            })),
        );
    } else if let core::Held::StopRun { task, attempt } = held {
        step(
            domain,
            env,
            Event::Core(core::Event::Fleet(fleet::Event::Cancel {
                run: Token::new(task),
                attempt: Token::new(attempt),
            })),
        );
    } else if let core::Held::Relay { task, attempt, word, .. } = held {
        let event = relay_message(domain, task, attempt, word);
        step(domain, env, event);
    } else if let core::Held::Store(_) = held {
        unreachable!("store loads cross the engine port (jig/docs/design/domain/engine/vocabulary.md, 2.4)")
    } else {
        out.push(Request::Deliver(Delivery::Core(held)));
    }
}

fn continue_ready(domain: &mut Domain, env: &Env<Limits>) {
    if domain.stopping() {
        if domain.engine.core().finish_owed() {
            decide(domain, env, Work::Finish);
        }
        return;
    }
    if resume_restart_outbox(domain, env) {
        return;
    }
    if domain.journal.idle() && domain.ready() {
        if let Some(number) = closing_connector(domain) {
            decide(domain, env, Work::ConnectorResume { number });
            return;
        }
        if let Some(context) = domain.engine.core_mut().take_due() {
            decide(domain, env, Work::Core(core::Event::Activate { context, ready: true }));
            return;
        }
        if domain.engine.core().fleet.ready_to_resume(!domain.engine.core().settling())
            || domain.engine.core().finish_owed()
            || timers_due(domain, env)
        {
            decide(domain, env, Work::ResumeFleet);
        }
    }
}

fn timers_due(domain: &Domain, env: &Env<Limits>) -> bool {
    if engine::due(&domain.engine, env.now, env.wall, Admit::All).is_some() {
        return true;
    }
    for (_, due) in domain.connector_deadlines() {
        if let Some(due) = due
            && due <= env.wall
        {
            return true;
        }
    }
    match domain.effect_deadline() {
        Some(due) => due <= env.wall,
        None => false,
    }
}

fn closing_connector(domain: &Domain) -> Option<u16> {
    if connector::closing_ready(&domain.first) {
        Some(1)
    } else if connector::closing_ready(&domain.second) {
        Some(2)
    } else {
        None
    }
}

fn resume_restart_outbox(domain: &mut Domain, env: &Env<Limits>) -> bool {
    if !domain.journal.idle() {
        return false;
    }
    let number = if connector::restart_ready(&domain.first, &env.with(env.limits.connector)) {
        1
    } else if connector::restart_ready(&domain.second, &env.with(env.limits.connector)) {
        2
    } else {
        return false;
    };
    decide(
        domain,
        env,
        Work::Connector { number, event: connector::Event::Restart(connector::RestartStep::SettleOutbox) },
    );
    true
}

fn write(domain: &mut Domain, decision: &mut Decision<Write, Delivery>, value: Write) {
    domain.decision_wrote = true;
    decision.push_write(value);
}

fn hold(decision: &mut Decision<Write, Delivery>, value: Delivery) {
    decision.push_held(value);
}

// The translations below are deliberately exhaustive: a new boundary variant
// must make the testing application say how it crosses its root.
mod client_documents;
pub mod link;
mod messages;
pub mod parts;
mod routes;
pub use connector::procedure_input;
#[cfg(test)]
mod tests;
mod topics;
use messages::Payload;
mod stopping;
use routes::{route_connector, route_core, route_engine};

mod limits;
pub use limits::valid;
mod admission;
pub use admission::{Class, class, try_step};

fn relay_message(domain: &mut Domain, task: u64, attempt: u64, word: tasks::Word) -> Event {
    let name = Token::new(word.number);
    let body = payload(domain, Payload::InboxWord(word));
    Event::Core(core::Event::Fleet(fleet::Event::Inbound {
        run: Token::new(task),
        attempt: Token::new(attempt),
        message: fleet::Message { name, sender: body, words: body },
    }))
}

/// Supply the core's binding numbers along with the connector's own rows.
fn restored_page(
    domain: &Domain,
    number: u16,
    records: Box<[connector::Record]>,
    after: Option<Box<[u8]>>,
) -> connector::Event {
    let mut rows = List::with_capacity(u32::try_from(records.len()).expect("bounded restore page"));
    for record in records {
        let mut interests = List::uncapped();
        if let Some((topic, task)) = record.subscription()
            && let Some(row) = domain.engine.core().tasks.task(task)
        {
            for interest in &row.subscriptions {
                if interest.kind == (tasks::SubscriptionKind::Topic { connector: number, topic: u64::from(topic) }) {
                    interests.push(interest.number).expect("restored subscription numbers");
                }
            }
        }
        rows.push(connector::RestoreRow { record, interests: interests.into_boxed() }).expect("one entry per row");
    }
    connector::Event::RestoredPage { rows: rows.into_boxed(), after }
}

/// Settings rows accompanying a deployment header in a seeded test store.
#[must_use]
pub fn initial_settings(defaults: &core::settings::Defaults) -> Box<[core::Record]> {
    use core::settings as s;
    let count = u32::try_from(defaults.charters.len())
        .expect("fixture charters")
        .checked_add(u32::try_from(defaults.templates.len()).expect("fixture templates"))
        .expect("fixture settings count")
        .checked_add(defaults.policies.len())
        .expect("fixture settings count")
        .checked_add(4)
        .expect("fixture settings count");
    let mut rows = List::with_capacity(count);
    for charter in &defaults.charters {
        rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
            setting: s::Setting::Charter(charter.number),
            default: s::Default { value: s::Value::Charter(Box::new(charter.clone())), revision: 1 },
        })))
        .expect("fixture settings count");
    }
    for template in &defaults.templates {
        rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
            setting: s::Setting::Template(template.number),
            default: s::Default { value: s::Value::Template(Box::new(template.clone())), revision: 1 },
        })))
        .expect("fixture settings count");
    }
    for (&number, policy) in &defaults.policies {
        rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
            setting: s::Setting::Policy(number),
            default: s::Default { value: s::Value::Policy(Box::new(policy.clone())), revision: 1 },
        })))
        .expect("fixture settings count");
    }
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
        setting: s::Setting::Rules,
        default: s::Default { value: s::Value::Rules(Box::new(defaults.rules.clone())), revision: 1 },
    })))
    .expect("fixture settings count");
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
        setting: s::Setting::Limits,
        default: s::Default { value: s::Value::Limits(defaults.limits.clone()), revision: 1 },
    })))
    .expect("fixture settings count");
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
        history: false,
        revision: s::Revision {
            applications: Box::default(),
            head: s::Head {
                number: 1,
                against: 0,
                at: Wall::EPOCH,
                by: s::Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
        },
    })))
    .expect("fixture settings count");
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Extent { oldest: 0, keys: 1 })))
        .expect("fixture settings count");
    rows.into_boxed()
}

/// The same fixture rows from an unmodified core's already loaded binary defaults.
#[must_use]
pub fn initial_settings_current(current: &core::settings::Current) -> Box<[core::Record]> {
    use core::settings as s;
    let names = current.offered();
    let mut rows = List::with_capacity(names.len().checked_add(2).expect("fixture settings count"));
    for &setting in &names {
        rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
            setting,
            default: s::Default {
                value: current.value(setting).expect("offered fixture default").clone(),
                revision: 1,
            },
        })))
        .expect("fixture settings count");
    }
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
        history: false,
        revision: s::Revision {
            applications: Box::default(),
            head: s::Head {
                number: 1,
                against: 0,
                at: Wall::EPOCH,
                by: s::Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
        },
    })))
    .expect("fixture settings count");
    rows.push(core::Record::Core(core::CoreRecord::Settings(s::Record::Extent { oldest: 0, keys: 1 })))
        .expect("fixture settings count");
    rows.into_boxed()
}
