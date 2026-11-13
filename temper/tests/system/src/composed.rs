//! Engine and worker roots with Smith's agent and fake LLM, on jig's real
//! store and fake parties and skein's fake forge (jig's domain/testing.md,
//! sections 2 and 8).
//!
//! The world plays what the two sides' own worlds play: the channel between
//! the roots, the protocol layers' translations and hand-outs (`link`), and
//! the agent process (`process`). Smith's payloads cross encoded by smith's
//! codecs; no scripted record chooses a turn, call or ending. Smith's own
//! world supplies filesystem and completion IO below its domain. Each tick
//! advances each domain once, so durable turn ACKs and relayed answers
//! traverse both roots.
use std::collections::VecDeque;

use jig_domain_authority as authority;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_fake_parties as parties;
use skein_fake_forge_domain::api as fake;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use smith_agent_world::World as Agent;
use smith_domain_run as run;
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Key};
use temper_engine_domain_world::protocol::Protocol;
use temper_engine_domain_world::store::Store;
use temper_engine_domain_world::walking;
use temper_engine_forge_world::translate as api;
use temper_fake_checkout::Checkout;
use temper_fake_checkout::git::Tree;
use temper_worker_checkout_world::{forge::Forge, translate as git};
use temper_worker_domain as worker;
use temper_worker_domain_world::protocol as worker_protocol;

mod agreement;
#[cfg(test)]
mod chat;
pub mod close_job;
#[cfg(test)]
mod design_delivery;
mod link;
mod long_chat;
pub mod merge_job;
mod next_turn;
mod parked;
mod process;
mod review;

pub use agreement::call;
use process::{Around, Process};

const CHANNEL: Token = Token::new(7);
const TICK: Duration = Duration::from_millis(1);
/// The party's request that adopts the repository.
const ADOPTION: Token = Token::new(900);
const RUN_WATCH: Token = Token::new(901);

/// What crossed the independently driven roots and process channel.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Observed {
    pub assignments: usize,
    pub checkouts: usize,
    pub spawns: usize,
    /// Turns the worker sent the engine.
    pub turns: usize,
    /// Live pieces the worker sent the engine.
    pub pieces: usize,
    /// Turns the engine told the worker it keeps.
    pub turn_acks: usize,
    pub relays: usize,
    pub replies: usize,
    /// Settled host replies replayed byte for byte after a lost delivery.
    pub replays: usize,
    pub messages: Vec<(Token, Box<[u8]>)>,
    pub cancels: usize,
    pub answers: usize,
    pub gone: usize,
}

/// The party's acknowledged provisional reply, played without a client
/// (testing/README.md, 2.1). The turn is its attempt, number and retry.
#[derive(Default, Debug)]
pub struct RunWatch {
    pub turn: Option<(u64, u32, u32)>,
    pub text: Vec<u8>,
}

/// Whether the chat works in an adopted repository, and how far its
/// adoption got.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Adoption {
    /// The chat works in no repository.
    Without,
    /// The owner adopts the repository once signed in; the chat waits.
    Pending,
    Adopted,
}

/// A composed deployment: all roots retain their own state, with IO outside.
pub struct World {
    seed: u64,
    merge_job: Option<merge_job::Story>,
    close_job: Option<close_job::Story>,
    scripts: VecDeque<Script>,
    parked: std::collections::BTreeMap<u64, Process>,
    period_budget: u64,
    creating: jig_test_profile::fixtures::Creating,
    chat_ending: tasks::EndedBy,
    goal_branch: bool,
    root: engine::Domain,
    env: Env<engine::Limits>,
    root_out: Queue<engine::Request>,
    root_events: VecDeque<engine::Event>,
    pub store: Store,
    pub party: parties::Party,
    protocol: Protocol,
    worker: worker::Domain,
    worker_env: Env<worker::Limits>,
    worker_out: Queue<worker::Request>,
    worker_events: VecDeque<worker::Event>,
    process: Process,
    replay: Option<(Option<host::LinkUp>, Option<host::LinkDown>)>,
    assignment: Option<engine::Assignment>,
    inspecting_runs: std::collections::BTreeMap<(u64, u64), bool>,
    adoption: Adoption,
    /// The root heard its funding period and the git identity once ready.
    funded: bool,
    /// The root was quiescent once: the worker and the party may begin.
    opened: bool,
    forge: Forge,
    disk: Checkout,
    pub observed: Observed,
    pub run_watch: Option<RunWatch>,
}

/// One text-only reply, with no finish call.
#[must_use]
pub fn text_script(reply: &[u8]) -> Script {
    Script {
        // The host instructions stay in system text when an Open brief moves to user.
        cue: Box::from(&b"Complete the task within its authority."[..]),
        turns: Box::new([Turn {
            lines: Box::new([Line::Text { text: reply.into() }]),
            finish: Finish::Stop,
            tokens: 20,
        }]),
    }
}

/// A two-completion report script; optional host write precedes the report.
#[must_use]
pub fn report_script(host_call: bool) -> Script {
    let finish = Turn {
        lines: Box::new([Line::Call {
            name: Box::from(&b"finish"[..]),
            arguments: Box::from(&br#"{"report":"The composed run answered."}"#[..]),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let turns = if host_call {
        vec![
            Turn {
                lines: Box::new([Line::Call {
                    name: Box::from(&b"message"[..]),
                    arguments: Box::from(&br#"{"target":1,"form":"words","words":"check the plan"}"#[..]),
                }]),
                finish: Finish::ToolCalls,
                tokens: 20,
            },
            finish,
        ]
    } else {
        vec![finish]
    };
    Script { cue: Box::from(&b"@system"[..]), turns: turns.into_boxed_slice() }
}

/// [`report_script`] with its host write, finishing again once the person's
/// words arrive: a wait-granted run rejects a finish that crosses words it
/// has not read, and offers them.
#[must_use]
pub fn crossed_script() -> Script {
    let mut script = report_script(true);
    let mut turns = script.turns.to_vec();
    turns.push(turns.last().expect("a finishing turn").clone());
    script.turns = turns.into_boxed_slice();
    script
}

/// The engine's configuration: the narrow fixtures' charter on smith's one
/// endpoint as the worker world's codecs name it (its account included), a
/// hub window of two turns, and room to resume a conversation.
fn config(seed: u64) -> engine::Config {
    configure(crate::world::config(seed))
}

fn open_config(seed: u64) -> engine::Config {
    configure(crate::world::open_config(seed))
}

fn configure(mut config: engine::Config) -> engine::Config {
    let mut charter = crate::world::charter();
    charter.policy.model.window = 1_048_576;
    config.core.settings.models = Box::new([charter.policy.model.clone()]);
    config.core.defaults.charters = Box::new([charter]);
    config.core.settings.resume_bytes = 32768;
    config.core.hub_turns = 2;
    config
}

/// The worker's limits: one slot, room for smith's concrete charters, turns
/// and transcripts, a turn window of two, and the bounds its hello says,
/// which fit the engine's charter.
fn worker_limits() -> worker::Limits {
    let mut limits = temper_worker_domain_world::limits();
    limits.host.slots = 1;
    limits.host.charter_bytes = 65536;
    limits.agent.charter_bytes = 65536;
    limits.host.turn_bytes = 32768;
    limits.agent.turn_bytes = 32768;
    limits.host.turns = 2;
    limits.agent.turns = 2;
    limits.host.turn_queue_bytes = 65536;
    limits.agent.unacknowledged_bytes = 65536;
    limits.host.transcript_bytes = 32768;
    limits.agent.transcript_bytes = 32768;
    limits.host.event_bytes = 1024;
    limits.agent.message_bytes = 2048;
    limits.host.inbox = 4;
    limits.host.held = 4;
    limits.agent.messages = 4;
    limits.agent.call_bytes = 4096;
    limits.checkout.message_bytes = 4096;
    limits.checkout.conflicts = 2;
    limits.agent.conflicts = 2;
    limits.checkout.path_bytes = 64;
    limits.agent.path_bytes = 64;
    limits.agent.no_progress = Duration::from_secs(120);
    limits.bounds = crate::world::bounds();
    limits.bounds.endpoints[0].window = 1_048_576;
    limits.bounds.inbox = 4;
    limits.bounds.message_bytes = 1024;
    limits.bounds.unacknowledged = 2;
    limits
}

impl World {
    /// Start a keyed chat through jig's independently scripted party.
    #[must_use]
    pub fn chat(seed: u64, script: Script) -> Self {
        Self::configured(seed, script, false)
    }

    /// Read a large family design with unequal engine and receiving allowances.
    #[must_use]
    pub fn forge_reads(seed: u64, mut script: Script, content: &[u8], refused: bool) -> Self {
        let mut world = Self::configured_limits(
            seed,
            report_script(false),
            true,
            config,
            if refused { refused_forge_limits } else { forge_read_limits },
        );
        world.worker_env.limits.reply_bytes = 8192;
        world.worker = worker::Domain::new(&world.worker_env.limits, Box::new([]));
        let before = world.forge.branch(b"org/repo", b"main").expect("base commit");
        world.forge.create_branch(b"org/repo", b"formats", before).expect("head branch");
        let head = world.forge.advance(b"org/repo", b"formats", b"docs/design/formats.md", content);
        let pull = world
            .forge
            .api(
                b"org/repo",
                1,
                fake::Op::Write(fake::Write::OpenPull {
                    opens_empty: false,
                    title: b"Family numbers".as_slice().into(),
                    body: Box::default(),
                    head: b"formats".as_slice().into(),
                    base: b"main".as_slice().into(),
                }),
            )
            .expect("open pull");
        assert!(matches!(pull, fake::Answer::Created(1)));
        let hex =
            |commit| String::from_utf8(engine::commit_text(&api::commit(commit)).into_vec()).expect("commit ASCII");
        for turn in &mut script.turns {
            for line in &mut turn.lines {
                if let Line::Call { arguments, .. } = line {
                    *arguments = String::from_utf8(arguments.to_vec())
                        .expect("fixture JSON")
                        .replace("$BEFORE", &hex(before))
                        .replace("$HEAD", &hex(head))
                        .into_bytes()
                        .into_boxed_slice();
                }
            }
        }
        world.process = Process::new(seed, script);
        world.replay = Some((None, None));
        world
    }

    /// Start a chat whose task is ended by its requester.
    #[must_use]
    pub fn open_chat(seed: u64, script: Script) -> Self {
        Self::configured_with(seed, script, false, open_config)
    }

    /// Watch an open chat, holding its reply before each later live piece
    /// and before its turn crosses the process channel (testing/README.md, 2.1).
    #[must_use]
    pub fn open_chat_in_pieces(seed: u64, script: Script, pieces: usize) -> Self {
        let mut world = Self::open_chat(seed, script);
        world.process.in_pieces(pieces);
        world.run_watch = Some(RunWatch::default());
        world
    }

    /// Release one later reply piece, or its turn after all pieces arrived.
    pub fn release_reply(&mut self) {
        self.process.release_reply(&mut self.worker_events);
    }

    /// Wait at an event-controlled hold until the run watch has this prefix.
    pub fn until_reply(&mut self, expected: &[u8]) {
        for _ in 0..10000 {
            self.tick();
            if self.run_watch.as_ref().expect("a run watch").text == expected {
                return;
            }
        }
        panic!("run watch did not reach {expected:?}: {:?}; {:?}", self.run_watch, self.observed);
    }

    /// Prepare an adopted repository through the worker before starting Smith.
    #[must_use]
    pub fn with_repository(seed: u64, script: Script) -> Self {
        Self::configured(seed, script, true)
    }

    #[cfg(test)]
    pub(crate) fn signed_in_chat(seed: u64, script: Script) -> Self {
        use jig_domain_accounts::{Holder, Kind, Stored};
        let mut world = Self::chat(seed, script);
        let rows = world
            .store
            .rows
            .values()
            .filter_map(|row| {
                let engine::Record::Core(jig_domain_core::Record::Accounts(mut row)) = row.clone() else {
                    return None;
                };
                match &mut row {
                    Stored::Account { holder, .. } => *holder = Holder::Party(1),
                    Stored::Source { kind, .. } => *kind = Kind::SignIn,
                    Stored::Issued { .. } | Stored::Next { .. } | Stored::Facts { .. } => (),
                }
                Some(engine::Record::Core(jig_domain_core::Record::Accounts(row)))
            })
            .collect();
        world.store.seed(rows, Box::new([]));
        world.protocol.sign_in_sources.insert(1);
        world
    }

    /// Two real durable source identities belonging to one billing account.
    #[must_use]
    pub fn two_source_chat(seed: u64, script: Script) -> Self {
        use jig_domain_accounts::{Holder, Kind, Stored};
        let mut world = Self::chat(seed, script);
        let erases = world
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let engine::Record::Core(jig_domain_core::Record::Accounts(_)) = row {
                    Some(row.key())
                } else {
                    None
                }
            })
            .collect();
        let rows = [
            Stored::Account {
                list: None,
                number: BILLING_ACCOUNT,
                provider: 1,
                holder: Holder::Deployment,
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
            },
            Stored::Source {
                account: BILLING_ACCOUNT,
                number: FIRST_SOURCE,
                place: 0,
                kind: Kind::Key,
                name: Box::new([]),
                held: None,
            },
            Stored::Source {
                account: BILLING_ACCOUNT,
                number: SECOND_SOURCE,
                place: 1,
                kind: Kind::Key,
                name: Box::new([]),
                held: None,
            },
            Stored::Next { number: SECOND_SOURCE + 1 },
        ]
        .into_iter()
        .map(|row| engine::Record::Core(jig_domain_core::Record::Accounts(row)))
        .collect();
        world.store.seed(rows, erases);
        world
    }

    /// Feed the agent channel's notice through the worker's real process boundary.
    pub fn notice(&mut self, notice: worker::agent::Up) {
        self.process.notice(notice);
    }

    fn configured(seed: u64, script: Script, repository: bool) -> Self {
        Self::configured_with(seed, script, repository, config)
    }

    fn configured_with(seed: u64, script: Script, repository: bool, config: fn(u64) -> engine::Config) -> Self {
        Self::configured_limits(seed, script, repository, config, |_| {})
    }

    fn configured_limits(
        seed: u64,
        script: Script,
        repository: bool,
        config: fn(u64) -> engine::Config,
        limits: fn(&mut engine::Limits),
    ) -> Self {
        let [(bootstrap, mut limits), (config, _)] = std::array::from_fn(|_| {
            let (config, mut bounds) = configured_parts(config(seed), repository);
            limits(&mut bounds);
            (config, bounds)
        });
        let forge = repository_forge(seed);
        let mut worker_limits =
            if config.core.defaults.charters.len() > 1 { agreement::worker_limits(&limits) } else { worker_limits() };
        if !config.core.jobs.is_empty() {
            // One completed answer can remain on the link while the other slot advances the goal.
            worker_limits.host.slots = 2;
        }
        // The engine waits for a lost worker longer than it may take to stop.
        limits.fleet.grace = worker_limits.stop_bound.saturating_add(Duration::from_secs(1));
        assert!(engine::valid(&limits));
        assert!(worker::valid(&worker_limits));
        let kinds = close_job::worker_kinds(&config.core.jobs, &mut worker_limits);
        let store =
            temper_engine_domain_world::fixtures::restored_over(Store::with_limits(&limits), bootstrap, &limits);
        Self {
            seed,
            merge_job: None,
            close_job: None,
            scripts: VecDeque::new(),
            parked: std::collections::BTreeMap::new(),
            period_budget: config.core.defaults.rules.period_spend,
            goal_branch: false,
            creating: jig_test_profile::fixtures::Creating::new(&config.core, limits.tasks.tree_tasks),
            chat_ending: chat_ending(&config),
            root: engine::Domain::new(config, &limits).expect("valid engine configuration"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            root_out: Queue::uncapped(),
            root_events: VecDeque::from([
                engine::Event::Restart { pending: false },
                engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
                    forge: 1,
                    state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
                })),
            ]),
            store,
            party: signed_in_party(),
            protocol: Protocol::default(),
            worker: worker::Domain::new(&worker_limits, kinds),
            worker_out: Queue::uncapped(),
            worker_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: worker_limits },
            // The worker's link has a channel open to the engine.
            worker_events: VecDeque::from([worker::Event::Ready]),
            process: Process::new(seed, script),
            replay: None,
            assignment: None,
            inspecting_runs: std::collections::BTreeMap::new(),
            adoption: if repository { Adoption::Pending } else { Adoption::Without },
            funded: false,
            opened: false,
            forge,
            disk: Checkout::new(),
            observed: Observed::default(),
            run_watch: None,
        }
    }

    /// Offer one input through admission; one the journal cannot take
    /// returns to the back of the queue (jig's `domain/root.md`, 3).
    fn root_step(&mut self, event: engine::Event) {
        if let Err(event) = engine::try_step(&mut self.root, &self.env, event) {
            self.root_events.push_back(event);
        }
        engine::release(&mut self.root, &self.env, &mut self.root_out);
        while let Some(request) = self.root_out.pop() {
            self.root_request(request);
        }
        self.root.reclaim();
    }

    /// Drive one domain iteration and the independently observed IO terminals.
    pub fn tick(&mut self) {
        self.env.now = self.env.now.saturating_add(TICK);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        self.worker_env.now = self.env.now;
        self.worker_env.wall = self.env.wall;
        if !self.funded && self.root.ready() {
            self.funded = true;
            self.fund();
        }
        if let Some(event) = self.root_events.pop_front() {
            self.root_step(event);
        }
        if let Some(number) = self.store.tick(false).expect("calm store") {
            self.root_step(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
        engine::fire(&mut self.root, &self.env);
        for pass in 0..32 {
            if pass == 0 {
                engine::release(&mut self.root, &self.env, &mut self.root_out);
            } else {
                temper_engine_domain_world::poll(&mut self.root, &self.env, &mut self.root_out);
            }
            if self.root_out.is_empty() && pass > 0 {
                break;
            }
            while let Some(request) = self.root_out.pop() {
                self.root_request(request);
            }
        }
        self.opened |= self.root.quiescent();
        if self.opened
            && (self.adoption != Adoption::Pending || self.party.sign_in.is_none())
            && let Some(up) = self.party.tick(self.env.now)
        {
            self.party_request(up);
        }
        if self.opened {
            self.worker_tick();
        }
        let mut around = Around {
            events: &mut self.worker_events,
            observed: &mut self.observed,
            disk: &mut self.disk,
            store: &self.store,
            assignment: self.assignment.as_ref(),
            repository: self.adoption != Adoption::Without,
        };
        self.process.tick(self.worker_env.now, &mut around);
        self.worker.reclaim();
        self.root.reclaim();
    }

    /// Advance controlled time, driving both roots and the process at each tick.
    pub fn advance(&mut self, duration: Duration) {
        let until = self.env.now.saturating_add(duration);
        while self.env.now < until {
            self.tick();
        }
    }

    /// The deployment's first period and its git identity's credential, from
    /// outside the root (jig's domain/authority.md, 7; domain/root.md, 6.1).
    fn fund(&mut self) {
        let period = engine::PartyInput::Period { project: 1, period: 1, budget: self.period_budget, next: None };
        self.root_events.push_back(engine::Event::Party(period));
        let identity = engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) };
        self.root_events.push_back(identity);
    }

    fn worker_tick(&mut self) {
        if let Some(event) = self.worker_events.pop_front() {
            worker::step(&mut self.worker, &self.worker_env, event, &mut self.worker_out);
        }
        if self.worker.is_due(self.worker_env.now) {
            worker::fire(&mut self.worker, &self.worker_env, &mut self.worker_out);
        }
        for _ in 0..16 {
            if !self.worker.is_ready() {
                break;
            }
            worker::resume(&mut self.worker, &self.worker_env, &mut self.worker_out);
            while let Some(request) = self.worker_out.pop() {
                self.worker_request(request);
            }
        }
        while let Some(request) = self.worker_out.pop() {
            self.worker_request(request);
        }
    }

    /// Finish one activation, including its worker answer and committed ACK.
    pub fn run(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.process.answered.is_some()
                && self.party.quiescent()
                && self.observed.answers != 0
                && self.observed.gone != 0
                && self.worker_events.is_empty()
                && self.root_events.is_empty()
                && !self.committing()
                && self.root.quiescent()
            {
                return;
            }
        }
        panic!(
            "composed run did not settle: {:?}; answered={:?}; agent_tail={:?}",
            self.observed,
            self.process.answered,
            self.agent().trace().iter().rev().take(8).collect::<Vec<_>>()
        );
    }

    /// Whether the store has commits not yet applied.
    fn committing(&self) -> bool {
        !self.store.pending.is_empty()
    }

    /// Resume a parked activation from concrete history whose bytes the
    /// engine made durable and returned through the worker's next start.
    pub fn resume(&mut self, words: &[u8], script: Script) {
        self.process.resume(script);
        let ask = parties::Ask::Say { task: parties::Task::Last, words: words.into(), release: None };
        self.act(ask, [44; 16]);
    }

    /// Release withheld store terminals in their independently retained order.
    pub fn release_commits(&mut self) {
        while let Some(number) = self.store.release_held() {
            self.root_events.push_back(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
    }

    /// Add a new authenticated action to the independent party.
    pub fn act(&mut self, ask: parties::Ask, key: [u8; 16]) {
        self.party.extend(Box::new([parties::Act::Request { key, ask }]));
    }

    /// A notice crosses worker output, link, engine input and durable acknowledgement.
    /// Each boundary may have a complete output cohort queued ahead of it.
    #[must_use]
    pub fn notice_steps(&self) -> u32 {
        const BOUNDARIES: u32 = 4;
        // One process notice crosses its channel, the worker root and link;
        // allow the peer driver's 1,000 settling steps at each boundary.
        const WORKER_NOTICE_STEPS: u32 = 3 * 1_000;
        // Steps for one complete root output cohort queued ahead of the notice at a boundary.
        const ROOT_COHORT_STEPS: u32 = 22;
        BOUNDARIES * (WORKER_NOTICE_STEPS + ROOT_COHORT_STEPS)
    }

    /// Wait until the process receives the first Smith host operation.
    pub fn until_call(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.process.called() {
                return;
            }
        }
        panic!("Smith host call did not arrive");
    }

    #[must_use]
    pub fn agent(&self) -> &Agent {
        self.process.agent()
    }

    #[must_use]
    pub fn assignment(&self) -> &engine::Assignment {
        self.assignment.as_ref().expect("committed assignment")
    }

    fn chat_request(&self, project: u32, words: &[u8]) -> people::Ask {
        let people::Ask::Create { project, mut batch } = walking::configured_chat(
            &self.creating,
            &self.store.rows,
            self.env.limits.tasks.tree_tasks,
            project,
            words,
        ) else {
            unreachable!("chat creates a task")
        };
        if !self.scripts.is_empty() {
            agreement::quiet(&mut batch[0]);
            if self.merge_job.as_ref().is_some_and(|story| story.row == merge_job::Row::Clean) {
                batch[0].wake.as_mut().expect("goal wake policy").results = people::ResultsWake::LastOrFailure;
            }
        }
        if self.goal_branch {
            batch[0].tracked = Some(1);
            batch[0].symbols = Box::new([people::Symbol { connector: 1, symbol: 1 }]);
        }
        // As jig_client::shapes::chat does, copy the template's ending into Create.
        batch[0].flags.ended_by = match self.chat_ending {
            tasks::EndedBy::Result => people::EndedBy::Result,
            tasks::EndedBy::Requester => people::EndedBy::Requester,
            tasks::EndedBy::Standing => people::EndedBy::Standing,
            tasks::EndedBy::Closer(closer) => {
                people::EndedBy::Closer(people::Closer { connector: closer.connector, code: closer.code })
            }
        };
        people::Ask::Create { project, batch }
    }

    fn party_request(&mut self, request: parties::Up) {
        let event = match request {
            parties::Up::SignIn { to, provider, subject } => engine::Event::Party(engine::PartyInput::SignedIn {
                held: None,
                reply_to: ReplyTo::new(Token::new(to)),
                identity: people::Identity {
                    key: people::IdentityKey { provider, subject },
                    login: Box::from(&b"person"[..]),
                    name: Box::from(&b"Person"[..]),
                },
            }),
            parties::Up::Request { to, sign_in, project, key, ask } => {
                let ask = match ask {
                    parties::Ask::Chat { words } => self.chat_request(project, &words),
                    parties::Ask::Say { task: parties::Task::Number(task), words, release } => {
                        say(project, task, words, release)
                    }
                    parties::Ask::Stop { task: parties::Task::Number(task) } => people::Ask::Stop { project, task },
                    parties::Ask::Decision { proposer: parties::Proposer::Task(task), proposal, accept } => {
                        people::Ask::DecideProposal {
                            project: people::DEPLOYMENT,
                            proposer: people::Proposer::Task(task),
                            proposal,
                            decision: if accept {
                                people::ProposalDecision::Accept
                            } else {
                                people::ProposalDecision::Reject { reason: Box::default() }
                            },
                            running: Some(people::Running::Stays),
                        }
                    }
                    other @ (parties::Ask::Close { .. }
                    | parties::Ask::Cancel { .. }
                    | parties::Ask::SeeResults { .. }
                    | parties::Ask::SeeConversation { .. }
                    | parties::Ask::Goal { .. }
                    | parties::Ask::Say { .. }
                    | parties::Ask::Stop { .. }
                    | parties::Ask::Release { .. }
                    | parties::Ask::Take { .. }
                    | parties::Ask::Choose { .. }
                    | parties::Ask::Decide { .. }
                    | parties::Ask::Decision { .. }) => panic!("unscripted party request: {other:?}"),
                };
                engine::Event::Party(engine::PartyInput::Ask {
                    models: Box::new([]),
                    reply_to: ReplyTo::new(Token::new(to)),
                    sign_in,
                    key,
                    ask,
                })
            }
        };
        self.root_events.push_back(event);
    }

    /// One root output: the protocol layer's hand-outs first, then the
    /// store, the forge, and deliveries to the worker and the party.
    fn root_request(&mut self, request: engine::Request) {
        let request = match self.play(request) {
            Ok(events) => {
                self.root_events.extend(events);
                return;
            }
            Err(request) => *request,
        };
        match request {
            engine::Request::Store(engine::StoreRequest::Catalogue { .. })
            | engine::Request::Party(engine::PartyRequest::Catalogue { .. }) => {}
            engine::Request::ToChild(released) => self.root_events.push_back(engine::Event::Released(released)),
            engine::Request::Store(engine::StoreRequest::Switch) => {
                let answer = self.store.switch();
                self.root_events.push_back(engine::Event::Store(answer));
            }
            engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                let delay = u32::try_from(self.seed % 3).expect("small seeded delay");
                self.store.submit(number, writes, delay);
            }
            engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. }) => {
                let (rows, next, texts) = self.store.page_with_texts(&range, after.as_ref(), most);
                self.root_events.push_back(engine::Event::Store(engine::StoreInput::Loaded {
                    owner,
                    rows,
                    next,
                    texts,
                }));
            }
            engine::Request::Forge { call, repository, op } => self.forge(call, repository, &op),
            engine::Request::Worker(engine::WorkerRequest::Deliver(delivery))
            | engine::Request::Party(engine::PartyRequest::Deliver(delivery)) => self.delivery(delivery),
            engine::Request::Party(engine::PartyRequest::RunText {
                watcher,
                turn,
                from,
                to,
                text,
                cut,
                call,
                writing,
            }) => {
                assert_eq!(watcher, RUN_WATCH);
                let watch = self.run_watch.as_mut().expect("a run watch");
                assert_eq!(usize::try_from(from).expect("bounded offset"), watch.text.len());
                assert_eq!(usize::try_from(to).expect("bounded offset"), watch.text.len() + text.len());
                assert!(call.is_none() && writing.is_none(), "the reply contains text only");
                assert!(!cut, "the short reply fits the stream buffer");
                let name = (turn.attempt, turn.turn, turn.retry);
                if let Some(previous) = watch.turn {
                    assert_eq!(previous, name, "one reply turn");
                }
                watch.turn = Some(name);
                watch.text.extend_from_slice(&text);
                self.root_events.push_back(engine::Event::Party(engine::PartyInput::StreamDelivered { watcher }));
            }
            other @ (engine::Request::Store(engine::StoreRequest::Read { .. })
            | engine::Request::Worker(
                engine::WorkerRequest::Host(_) | engine::WorkerRequest::ReadResult { .. },
            )
            | engine::Request::Party(
                engine::PartyRequest::SignInRefused { .. } | engine::PartyRequest::WatchRefused { .. },
            )
            | engine::Request::Account(_)
            | engine::Request::ForgeKept(_)
            | engine::Request::ForgeGone { .. }
            | engine::Request::BoxGroup { .. }
            | engine::Request::PlaygroundActs { .. }
            | engine::Request::ResetAct { .. }
            | engine::Request::BoxRemove { .. }
            | engine::Request::Connect { .. }
            | engine::Request::Replace { .. }
            | engine::Request::Manifest { .. }
            | engine::Request::AdoptAccess { .. }
            | engine::Request::Reachable { .. }
            | engine::Request::Install { .. }
            | engine::Request::Accept { .. }
            | engine::Request::EraseCarried { .. }
            | engine::Request::MakingCopied { .. }
            | engine::Request::SetUp { .. }
            | engine::Request::Bound { .. }
            | engine::Request::Setup { .. }
            | engine::Request::Busy(_)
            | engine::Request::Unsettled(_)
            | engine::Request::Drained
            | engine::Request::Stop) => panic!("unscripted engine output: {other:?}"),
        }
    }

    /// The protocol layer's part: the engine's host calls decoded and their
    /// answers rendered, ended answers' outcomes read, by `link`; accounts
    /// and core reads as the engine world's protocol plays them.
    #[expect(clippy::too_many_lines, reason = "the protocol dispatcher lists every host delivery beside settings")]
    fn play(&mut self, request: engine::Request) -> Result<Vec<engine::Event>, Box<engine::Request>> {
        let event = match request {
            engine::Request::Worker(engine::WorkerRequest::Host(host_request)) => match *host_request {
                engine::HostRequest::Decode { to, call, .. } => self.decode_close_call(to, &call),
                engine::HostRequest::SettingsNote { to, call, body } => {
                    return Ok(vec![temper_engine_service_vocabulary::tools::settings_note(
                        to,
                        *call,
                        &body,
                        &self.root.catalogue(),
                    )]);
                }
                engine::HostRequest::Dropped { .. } => return Ok(Vec::new()),
                ask @ engine::HostRequest::Ask { .. } => {
                    return Err(Box::new(engine::Request::Worker(engine::WorkerRequest::Host(Box::new(ask)))));
                }
            },
            engine::Request::Worker(engine::WorkerRequest::Deliver(Delivery::Host(delivery))) => match *delivery {
                engine::HostDelivery::Render { to, answer, tool, key, job, page, ask, .. } => {
                    engine::WorkerInput::RenderedHostCall {
                        to,
                        answer: link::render(
                            &answer,
                            &tool,
                            self.env.limits.call_answer_bytes,
                            key.task,
                            self.worker_env.limits.reply_bytes,
                            job,
                            page,
                            ask,
                        ),
                    }
                }
                engine::HostDelivery::See { to, line, from, answer } => {
                    let (body, error) = temper_engine_smith::see_answer(
                        &line,
                        from,
                        &answer,
                        &self.root.catalogue(),
                        self.env.limits.call_answer_bytes.saturating_sub(3),
                        temper_engine_smith::ForgeReadBounds::STOCK,
                    )
                    .into_parts();
                    engine::WorkerInput::RenderedHostCall {
                        to,
                        answer: jig_domain_core::SettledAnswer::Host { body, error },
                    }
                }
                engine::HostDelivery::Settings {
                    to,
                    answer,
                    repositories,
                    people,
                    forge_people,
                    offer,
                    application,
                    from,
                    requested,
                } => {
                    let (body, error) = temper_engine_smith::settings_repository_answer(
                        &answer,
                        from,
                        requested,
                        &self.root.catalogue(),
                        application.as_ref(),
                        &repositories,
                        &people,
                        &forge_people,
                        &offer,
                        self.env.limits.call_answer_bytes.saturating_sub(8),
                    )
                    .into_parts();
                    engine::WorkerInput::RenderedHostCall {
                        to,
                        answer: jig_domain_core::SettledAnswer::Host { body, error },
                    }
                }
                other @ engine::HostDelivery::Answer { .. } => {
                    panic!("unscripted delivery answer: {other:?}")
                }
            },
            engine::Request::Worker(engine::WorkerRequest::ReadResult { payload, outcome }) => {
                engine::WorkerInput::ResultRead {
                    payload,
                    end: match self.root.change_resource(payload) {
                        Some((connector, kind, resource)) => link::end_with(
                            &outcome,
                            Some(temper_engine_smith::ChangeResource { connector, kind, resource }),
                        ),
                        None => link::end(&outcome),
                    },
                }
            }
            other @ (engine::Request::Store(_)
            | engine::Request::Worker(_)
            | engine::Request::Party(_)
            | engine::Request::Forge { .. }
            | engine::Request::Account(_)
            | engine::Request::ForgeKept(_)
            | engine::Request::ForgeGone { .. }
            | engine::Request::BoxGroup { .. }
            | engine::Request::PlaygroundActs { .. }
            | engine::Request::ResetAct { .. }
            | engine::Request::BoxRemove { .. }
            | engine::Request::Connect { .. }
            | engine::Request::Replace { .. }
            | engine::Request::Manifest { .. }
            | engine::Request::AdoptAccess { .. }
            | engine::Request::Reachable { .. }
            | engine::Request::Install { .. }
            | engine::Request::Accept { .. }
            | engine::Request::EraseCarried { .. }
            | engine::Request::MakingCopied { .. }
            | engine::Request::SetUp { .. }
            | engine::Request::Bound { .. }
            | engine::Request::Setup { .. }
            | engine::Request::ToChild(_)
            | engine::Request::Busy(_)
            | engine::Request::Unsettled(_)
            | engine::Request::Drained
            | engine::Request::Stop) => return self.protocol.play(other),
        };
        Ok(vec![engine::Event::Worker(event)])
    }

    fn forge(&mut self, call: Token, repository: skein_forge::api::Repository, op: &skein_forge::api::Op) {
        assert_eq!(repository.forge, temper_engine_forge_world::REPO.forge);
        assert!(repository.repository == 0 || repository == temper_engine_forge_world::REPO);
        self.forge.at(self.env.now);
        let client = &self.env.limits.forge.client;
        let located = temper_engine_forge_world::translate::locate_name(op);
        let name = located.as_deref().unwrap_or(b"org/repo");
        let result = match self.forge.api(name, 1, api::op(op, client)) {
            Ok(answer) => {
                let answer = api::answer(op, answer, client);
                if let Ok(answer) = &answer
                    && skein_forge::measure::answer(answer).expect("fake answer size") > u64::from(client.answer_bytes)
                {
                    Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()))
                } else {
                    answer
                }
            }
            Err(error) => Err(api::error(
                error,
                skein_fake_forge_domain::time(&temper_worker_checkout_world::forge::config(), self.env.now),
            )),
        };
        self.root_events.push_back(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
            skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result },
        ))));
        let _moves = self.forge.moves();
    }

    fn assigned(&mut self, channel: Token, assignment: engine::Assignment) {
        if self.merge_job.is_some() {
            self.prepare_clean_goal(assignment.task);
        }
        if let Some(story) = &mut self.merge_job {
            if story.row == merge_job::Row::Clean {
                // One process serves the world's agents. Its answer must cross
                // the worker before the next assignment selects another one.
                if self.assignment.is_some() && !story.clean_answered {
                    assert!(story.pending.is_none(), "one next agent");
                    story.pending = Some((channel, assignment));
                    return;
                }
                story.clean_answered = false;
            }
            // The goal's wait must cross the worker before its producer uses
            // this story's single concrete process.
            if self.assignment.as_ref().is_some_and(|previous| previous.task == 1)
                && self.process.answered.is_none()
                && !self.process.prepared()
            {
                assert!(story.pending.is_none(), "one next agent");
                story.pending = Some((channel, assignment));
                return;
            }
            // This story ends when the resolution is made, before its agent runs.
            if assignment.task > 5 {
                return;
            }
            if assignment.task == 5 && story.held.is_none() && story.base.is_none() {
                story.pending = Some((channel, assignment));
                return;
            }
        }
        assert_eq!(channel, CHANNEL);
        let live = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task)));
        assert!(self.store.rows.contains_key(&live), "assignment follows durable task");
        self.bind_review_script();
        self.select_process(assignment.task);
        self.observed.assignments += 1;
        let repositories = usize::from(self.adoption != Adoption::Without);
        assert_eq!(assignment.workspace.repositories.len(), repositories);
        let (hub, items) = link::assignment(&assignment);
        if self.run_watch.is_some() {
            self.root_events.push_back(engine::Event::Party(engine::PartyInput::WatchRun {
                watcher: RUN_WATCH,
                sign_in: self.party.sign_in.expect("authenticated party"),
                task: assignment.task,
                turn: None,
                offset: 0,
            }));
        }
        self.worker_events.push_back(worker::Event::Assign { assignment: hub, items: Ok(items) });
        self.inspecting_runs.insert(
            (assignment.task, assignment.attempt),
            assignment.workspace.repositories.iter().all(|repository| repository.push.is_none()),
        );
        self.assignment = Some(assignment);
    }

    fn delivery(&mut self, delivery: Delivery) {
        match delivery {
            // A run watch first supplies its current document, then streams
            // provisional suffixes. Acknowledge both kinds of delivery.
            Delivery::Document { watcher, .. } => self.acknowledge_run(watcher),
            Delivery::Push(_) => panic!("this story has no push devices"),
            Delivery::SourceSignedOut { .. } => panic!("this story requests no source sign-out"),
            Delivery::Application { to, outcome } => {
                assert_eq!(to.into_token(), ADOPTION);
                assert!(matches!(
                    outcome,
                    Some(temper_engine_domain::ApplicationAnswer::Adopt(
                        temper_engine_domain_forge::ApplicationOutcome::Adopted(_)
                    ))
                ));
                self.adoption = Adoption::Adopted;
            }
            Delivery::WebReply { to, reply } => self.web_reply(to.into_token(), reply),
            Delivery::Assigned { channel, assignment } => self.assigned(channel, assignment),
            Delivery::Link { channel, down } => {
                assert_eq!(channel, CHANNEL);
                if let host::LinkDown::TurnKept { .. } = down {
                    self.observed.turn_acks += 1;
                }
                self.deliver_link(down);
            }
            // The link sends a turn again once the engine's backoff passed.
            Delivery::TurnBusy { channel, task, attempt, turn } => {
                assert_eq!(channel, CHANNEL);
                let (run, attempt) = (Token::new(task), Token::new(attempt));
                self.worker_events.push_back(worker::Event::Resend { run, attempt, turn });
            }
            // A live notice to the requester, with no reply right.
            Delivery::Result { .. } => {}
            Delivery::AssignedJob { channel, job, workspace } => {
                self.assign_story_job(channel, job, workspace.as_ref());
            }
            other @ (Delivery::PlaygroundNote { .. }
            | Delivery::Host(_)
            | Delivery::Procedure { .. }
            | Delivery::Forges { .. }
            | Delivery::ForgeDocument { .. }
            | Delivery::Trials { .. }
            | Delivery::Ways { .. }
            | Delivery::Approvals { .. }
            | Delivery::Panel { .. }
            | Delivery::Refuse { .. }
            | Delivery::ForgeRead { .. }
            | Delivery::Read { .. }
            | Delivery::WatchEnded { .. }
            | Delivery::ProjectMade { .. }
            | Delivery::ProjectReleased { .. }
            | Delivery::MoveStanding { .. }
            | Delivery::FindingsOwned { .. }
            | Delivery::ForgeCommitted { .. }
            | Delivery::ForgeCall { .. }
            | Delivery::Bound { .. }
            | Delivery::Relay { .. }
            | Delivery::ReadEscalationDecision { .. }
            | Delivery::Fleet(_)
            | Delivery::ReadLoad { .. }
            | Delivery::Public { .. }
            | Delivery::PublicWait { .. }
            | Delivery::Drained
            | Delivery::Start(_)
            | Delivery::NotesLoad { .. }
            | Delivery::Load { .. }
            | Delivery::CarriedUnused { .. }) => panic!("unscripted released delivery: {other:?}"),
        }
    }

    fn acknowledge_run(&mut self, watcher: Token) {
        assert_eq!(watcher, RUN_WATCH);
        assert!(self.run_watch.is_some(), "the party's run watch");
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Delivered { watcher }));
    }

    fn web_reply(&mut self, to: Token, reply: people::Reply) {
        if next_turn::picked(to, &reply) {
            return;
        }
        if agreement::answered(to, &reply) {
            return;
        }
        let signed_in = matches!(reply, people::Reply::SignedIn { .. });
        let reply = match reply {
            people::Reply::SignedIn { person, sign_in, .. } => parties::Reply::SignedIn { person, sign_in },
            people::Reply::Outcome(people::Outcome::Created { first: task, last, given }) => {
                assert_eq!(task, last, "one template task");
                assert!(given.is_some_and(|given| given.task == task && given.amount > 0), "template funding receipt");
                parties::Reply::Started { task }
            }
            people::Reply::Outcome(people::Outcome::Said { task, message }) => parties::Reply::Said { task, message },
            people::Reply::Outcome(people::Outcome::Stopped { task }) => parties::Reply::Changed { task },
            people::Reply::Outcome(people::Outcome::ProposalDecided {
                proposer: people::Proposer::Task(task),
                proposal,
                by,
                at,
                choice,
                ..
            }) => parties::Reply::ProposalDecided {
                proposer: parties::Proposer::Task(task),
                proposal,
                by,
                at,
                choice: match choice {
                    people::ProposalChoice::Accepted => parties::Choice::Accepted,
                    people::ProposalChoice::Rejected => parties::Choice::Rejected,
                    people::ProposalChoice::Withdrawn => parties::Choice::Withdrawn,
                    people::ProposalChoice::Passed => parties::Choice::Passed,
                    people::ProposalChoice::Stale => parties::Choice::Stale,
                },
            },
            other @ (people::Reply::Outcome(_)
            | people::Reply::SignedOut
            | people::Reply::Again(_)
            | people::Reply::Full { .. }
            | people::Reply::Signer(_)
            | people::Reply::Refused(_)) => panic!("unscripted party reply: {other:?}"),
        };
        self.party.reply(to.raw(), reply);
        if signed_in && self.adoption == Adoption::Pending {
            self.root_events.push_back(engine::Event::Party(engine::PartyInput::Application {
                scope: jig_domain_people::ApplicationScope::Project(1),
                reply_to: ReplyTo::new(ADOPTION),
                sign_in: self.party.sign_in.expect("authenticated party"),
                key: [90; 16],
                digest: [5; 32],
                carried: None,
                request: temper_engine_domain::Application::Forge(temper_engine_domain_forge::Ask::Adopt(adoption())),
            }));
        }
    }

    /// One of the worker's requests: up the link to the engine, to the
    /// protocol layer's routing, to io's git, or to the agent process.
    fn worker_answer(
        &mut self,
        run: Token,
        attempt: Token,
        answer: host::RunAnswer,
        work: &worker::wire::Work,
        preparation: Option<worker::wire::Preparation>,
    ) {
        self.observed.answers += 1;
        // An older terminal may cross the next assignment on the worker link.
        let process = if self.assignment().task == run.raw() {
            &self.process
        } else {
            self.parked.get(&run.raw()).expect("the answering run's retained process")
        };
        let (turns, spent) = process.answered.unwrap_or((0, 0));
        assert_eq!(
            (answer.turns, answer.spent),
            (turns, spent),
            "answer {run:?}; current assignment {:?}",
            self.assignment.as_ref().map(|assignment| assignment.task)
        );
        if let host::Ending::Refused(_) = &answer.ending {
            assert!(process.prepared(), "refusal precedes a process");
        } else {
            match (&answer.ending, process.agent().answer()) {
                (host::Ending::Ended { outcome, .. }, run::Answer::Accepted { outcome: declared, .. }) => {
                    let decoded = smith_protocol_channel::decode_declared(outcome, &smith_charter::CEILINGS);
                    assert_eq!(decoded.as_ref(), Ok(declared), "the answer carries smith's own result");
                }
                (host::Ending::Parked { .. }, run::Answer::Parked { .. })
                | (host::Ending::Failed { .. }, run::Answer::Failed { .. }) => {}
                (ending, settled) => panic!("the worker's answer {ending:?} follows smith's {settled:?}"),
            }
        }
        if *self.inspecting_runs.get(&(run.raw(), attempt.raw())).expect("answering run's workspace") {
            assert!(work.landed.is_empty(), "an inspecting run lands nothing");
        }
        let parts = link::parts(work, preparation);
        let link = engine::Link::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts };
        if let Some(story) = &mut self.merge_job
            && run.raw() == 4
            && story.base.is_none()
        {
            story.held = Some(link);
        } else {
            self.up(link);
        }
        if let Some(story) = &mut self.merge_job
            && (story.row == merge_job::Row::Clean || run.raw() == 1)
        {
            story.clean_answered = true;
            if let Some((channel, assignment)) = story.pending.take() {
                self.assigned(channel, assignment);
            }
        }
    }

    fn remember_call(&mut self, up: &host::LinkUp) {
        if let Some((pending, _)) = &mut self.replay
            && let host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline } = up
        {
            *pending = Some(host::LinkUp::Call {
                contract: *contract,
                run: *run,
                attempt: *attempt,
                call: call.clone(),
                tool: tool.clone(),
                writes: *writes,
                input: input.clone(),
                deadline: *deadline,
            });
        }
    }

    fn deliver_link(&mut self, down: host::LinkDown) {
        if matches!(down, host::LinkDown::Relayed { answer: host::Relayed::Answer { .. }, .. })
            && let Some((pending, previous)) = &mut self.replay
        {
            if let Some(first) = previous.take() {
                assert_eq!(down, first, "settled-call replay preserves the fitted answer");
                self.observed.replays += 1;
            } else if let Some(up) = pending.take() {
                *previous = Some(down);
                self.up(engine::Link::Up { up, parts: engine::Parts::none() });
                return;
            }
        }
        self.worker_events.push_back(worker::Event::Link(down));
    }

    fn worker_request(&mut self, request: worker::Request) {
        match request {
            worker::Request::ReadList { .. }
            | worker::Request::CancelRead { .. }
            | worker::Request::Source { .. }
            | worker::Request::SourceFault => {
                panic!("this fixture declares no program sources")
            }
            worker::Request::Job { owner, op } => self.story_job_operation(owner, op),
            worker::Request::Hello(hello) => {
                self.close_hello(&hello);
                if let Some(story) = &mut self.merge_job {
                    story.hello = hello.jobs.to_vec();
                }
                self.up(engine::Link::Hello { worker: 0, hello });
            }
            worker::Request::Link(up) => {
                self.remember_call(&up);
                match &up {
                    host::LinkUp::Turn { .. } => self.observed.turns += 1,
                    host::LinkUp::Piece { .. } => self.observed.pieces += 1,
                    host::LinkUp::Call { .. } => self.observed.relays += 1,
                    host::LinkUp::ListRead { .. }
                    | host::LinkUp::Draining
                    | host::LinkUp::Logins { .. }
                    | host::LinkUp::SignedOut { .. }
                    | host::LinkUp::Activity { .. }
                    | host::LinkUp::Unauthorized { .. }
                    | host::LinkUp::Limited { .. }
                    | host::LinkUp::AccountRefused { .. }
                    | host::LinkUp::JobAnswer { .. }
                    | host::LinkUp::Ask { .. }
                    | host::LinkUp::Answer { .. } => {}
                }
                self.up(engine::Link::Up { up, parts: engine::Parts::none() });
            }
            worker::Request::Answer { run, attempt, answer, work, preparation } => {
                self.worker_answer(run, attempt, answer, &work, preparation);
            }
            worker::Request::JobAnswer { run, attempt, answer, work, preparation } => {
                let parts = link::parts(&work, preparation);
                self.up(engine::Link::Up { up: host::LinkUp::JobAnswer { run, attempt, answer }, parts });
            }
            // The link cancels a relay's local wait at once.
            worker::Request::CancelRelay { run, attempt, call } => {
                self.worker_events.push_back(worker::Event::RelayCancelled { run, attempt, call });
            }
            worker::Request::Stop | worker::Request::Abandon => panic!("the worker stops only once shut down"),
            worker::Request::Process(request) => self.process_request(*request),
            worker::Request::Decode { agent, run, start } => {
                let start = worker_protocol::decode(run, *start, &self.worker_env.limits.agent);
                self.worker_events.push_back(worker::Event::Decoded { agent, start, endpoints: Box::new([]) });
            }
            worker::Request::Encode { owner, value } => {
                let encoded = worker_protocol::encode(value);
                self.worker_events.push_back(worker::Event::Encoded { owner, encoded });
            }
            worker::Request::Fact { owner, body } => {
                let activity = worker_protocol::activity(&body);
                self.worker_events.push_back(worker::Event::Activity { owner, activity });
            }
            worker::Request::Io { owner, op, .. } => {
                self.observed.checkouts += 1;
                self.forge.at(self.env.now);
                let op = forge_operation(op);
                self.observe_merge_fetch(&op);
                self.observe_close_io(&op);
                let done =
                    git::perform_within(&mut self.forge, &mut self.disk, op, self.worker_env.limits.checkout.conflicts);
                self.worker_events.push_back(worker::Event::Done { owner, done });
                let _moves = self.forge.moves();
            }
            // io ends each operation at once: its outcome has won the race.
            worker::Request::CancelIo { .. } => {}
        }
    }

    fn process_request(&mut self, request: worker::agent::process::Request) {
        let mut around = Around {
            events: &mut self.worker_events,
            observed: &mut self.observed,
            disk: &mut self.disk,
            store: &self.store,
            assignment: self.assignment.as_ref(),
            repository: self.adoption != Adoption::Without,
        };
        self.process.request(request, &mut around);
    }

    fn up(&mut self, link: engine::Link) {
        self.root_events.push_back(engine::Event::Worker(engine::WorkerInput::Link { channel: CHANNEL, link }));
    }
}

impl World {
    /// A goal's producer delivers, asks its change a question, then continues when answered.
    #[must_use]
    pub fn questioning_producer(seed: u64) -> Self {
        let goal = Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"changes":[{"words":"Make two edits","landing":"goal","questions":"up"},{"words":"Advance the goal","landing":"goal"}]}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let mut world = Self::configured_with(seed, goal.clone(), true, questioning_config);
        world.goal_branch = true;
        let moving = Script {
            cue: b"Run only what your change needs".as_slice().into(),
            turns: Box::new([
                call(b"write", br#"{"path":"goal.txt","content":"goal moved\n"}"#),
                call(b"finish", br#"{"title":"Advance the goal","body":"Give the waiting producer new context."}"#),
            ]),
        };
        world.scripts = VecDeque::from([goal, questioning_script(), moving]);
        world
    }

    /// Advance the goal while its producer parks, then verify that producer's next delivery.
    pub fn continue_delivered_producer(&mut self) {
        self.until_producer_answer(0);
        assert_eq!(self.assignment().task, 4);
        assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }), "{:?}", self.agent().answer());
        let attempt = self.assignment().attempt;
        let branch = self.assignment().workspace.repositories[0].push.clone().expect("producer push branch");
        let first = self.forge.branch(b"org/repo", &branch).expect("first delivery exists");
        let base = self.reviewed_change().base.clone();
        let goal = self.advance_goal_for_producer(&base);
        self.answer_producer_question();
        self.until_producer_answer(attempt);
        assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }));
        let second = self.forge.branch(b"org/repo", &branch).expect("second delivery exists");
        self.assert_producer_history(first, goal, second);
        self.run();
        self.answer_producer_question();
        let attempt = self.assignment().attempt;
        self.until_producer_answer(attempt);
        assert!(
            matches!(self.agent().answer(), run::Answer::Accepted { .. }),
            "task {}: {:?}",
            self.assignment().task,
            self.agent().answer()
        );
        let third = self.forge.branch(b"org/repo", &branch).expect("third delivery exists");
        self.assert_producer_history(second, goal, third);
        self.assert_producer_file(third, b"third.txt", b"third delivery\n");
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(4)));
        for _ in 0..10000 {
            if self.store.rows.contains_key(&key) {
                break;
            }
            self.tick();
        }
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            self.store.rows.get(&key)
        else {
            panic!("the same producer finished")
        };
        assert_eq!(row.tries, tasks::Tries::NONE, "question resumes spend no failure tries");
        let producers = self
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(record,
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.executor == (tasks::Executor::Agent { charter: 2 }))
            })
            .count();
        assert_eq!(producers, 2, "the waiting producer and the independent producer; no replacement");
    }

    fn until_producer_answer(&mut self, after: u64) {
        for _ in 0..10000 {
            self.tick();
            if self.assignment.as_ref().is_some_and(|assignment| assignment.task == 4 && assignment.attempt > after)
                && self.process.answered.is_some()
            {
                return;
            }
        }
        panic!("producer did not answer: {:?}", self.observed);
    }

    fn assert_producer_history(&mut self, previous: u64, goal: u64, head: u64) {
        assert_ne!(previous, head);
        assert!(self.forge.is_ancestor(previous, head), "the delivery retains the preceding delivery");
        assert!(self.forge.is_ancestor(goal, head), "the producer merges the moved goal tip");
        self.assert_producer_file(head, b"first.txt", b"first delivery\n");
        self.assert_producer_file(head, b"second.txt", b"second delivery\n");
        self.assert_producer_file(head, b"goal.txt", b"goal moved\n");
    }

    fn assert_producer_file(&mut self, head: u64, path: &[u8], content: &[u8]) {
        assert_eq!(
            self.forge.api(b"org/repo", 1, fake::Op::Read(fake::Read::File { commit: head, path: path.into() })),
            Ok(fake::Answer::File(content.into()))
        );
    }

    fn advance_goal_for_producer(&mut self, base: &[u8]) -> u64 {
        let before = self.forge.branch(b"org/repo", base).expect("goal branch exists");
        for _ in 0..10000 {
            if self.process.answered.is_some()
                && self.root_events.is_empty()
                && self.worker_events.is_empty()
                && !self.committing()
            {
                self.env.now = self.env.now.saturating_add(Duration::from_secs(1));
            }
            self.tick();
            let head = self.forge.branch(b"org/repo", base).expect("goal branch stays live");
            if head != before {
                return head;
            }
        }
        panic!("the independent change did not advance the goal");
    }

    fn answer_producer_question(&mut self) {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            self.store.rows.get(&key)
        else {
            panic!("live change")
        };
        let question = row.questions[0].number;
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::default(),
            reply_to: ReplyTo::new(Token::new(902)),
            sign_in: self.party.sign_in.expect("signed in"),
            key: [u8::try_from(question).expect("fixture question number"); 16],
            ask: people::Ask::AnswerQuestion {
                project: 1,
                task: 1,
                question,
                words: b"Continue with the second edit.".as_slice().into(),
            },
        }));
    }
}

fn questioning_script() -> Script {
    Script {
        cue: b"Run only what your change needs".as_slice().into(),
        turns: Box::new([
            call(b"write", br#"{"path":"first.txt","content":"first delivery\n"}"#),
            call(b"deliver", br#"{"title":"First edit","body":"Keep this work."}"#),
            call(b"message", br#"{"target":2,"form":"question","words":"Add the second edit? I recommend yes."}"#),
            call(b"wait", b"{}"),
            call(b"write", br#"{"path":"second.txt","content":"second delivery\n"}"#),
            call(b"deliver", br#"{"title":"Both edits","body":"Keep both edits."}"#),
            call(b"message", br#"{"target":2,"form":"question","words":"Keep both edits? I recommend yes."}"#),
            call(b"wait", b"{}"),
            call(b"write", br#"{"path":"third.txt","content":"third delivery\n"}"#),
            call(b"finish", br#"{"title":"Both edits","body":"Both edits are ready."}"#),
        ]),
    }
}

fn questioning_config(seed: u64) -> engine::Config {
    let mut config = agreement::config(seed);
    let rule = engine::LandingRule {
        sets: Box::new([]),
        connector: 1,
        kind: 4,
        enforce: true,
        ci: false,
        up_to_date: false,
        pattern: authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: authority::Last::Open(Box::default()),
        },
        gates: Box::default(),
        approvals: Box::default(),
    };
    config.kinds_stock = engine::landing_stock(&[rule], &skein_lib::Map::with_capacity(0), 2, 4);
    config
}

fn signed_in_party() -> parties::Party {
    parties::Party::new(
        1,
        parties::Role::Owner,
        400,
        Box::new([
            parties::Act::SignIn { provider: 0, subject: temper_engine_domain_forge::user_key(1, 7).subject },
            parties::Act::Request { key: [41; 16], ask: parties::Ask::Chat { words: Box::from(&b"@system"[..]) } },
        ]),
    )
}

/// Map the service's git remote to the fake forge's repository name.
fn forge_operation(mut op: worker::checkout::git::Op) -> worker::checkout::git::Op {
    use worker::checkout::git::Op;
    let remote = match &mut op {
        Op::Store { remote, .. }
        | Op::Clone { remote, .. }
        | Op::Fetch { remote, .. }
        | Op::Create { remote, .. }
        | Op::Push { remote, .. } => remote,
        Op::Commit { snapshot: Some(snapshot), .. } => &mut snapshot.remote,
        Op::Point { .. }
        | Op::Make { .. }
        | Op::CheckOut { .. }
        | Op::Look { .. }
        | Op::DiffPaths { .. }
        | Op::Brings { .. }
        | Op::IsAncestor { .. }
        | Op::Merge { .. }
        | Op::Commit { snapshot: None, .. } => return op,
    };
    *remote = remote
        .strip_prefix(link::GIT_BASE)
        .and_then(|path| path.strip_prefix(b"/"))
        .and_then(|path| path.strip_suffix(b".git"))
        .expect("the world's configured git remote")
        .into();
    op
}

/// The owner's adoption of the forge's one repository.
fn adoption() -> temper_engine_domain_forge::Adoption {
    temper_engine_domain_forge::Adoption {
        project: 1,
        home: true,
        forge: temper_engine_forge_world::REPO.forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: temper_engine_domain_forge::Role::Owned,
        checks: Box::new([]),
        ci: true,
        landing: Box::from(&b"main"[..]),
    }
}

/// Read access to the repository for the chat's role, and room for it.
fn grant_read(config: &mut engine::Config, limits: &mut engine::Limits) {
    let tools = authority::Tools(1 | jig_domain_core::ToolFamilies::JIG.read.0);
    let grant = authority::Grant {
        connector: 0,
        kind: 1,
        pattern: authority::Pattern {
            segments: Box::new([Box::from(&b"forge"[..]), Box::from(&b"forge.example"[..]), Box::from(&b"org"[..])]),
            last: authority::Last::Open(Box::from(&b"repo"[..])),
        },
    };
    limits.people.people = 4;
    limits.people.holdings = 4;
    limits.authority.roles = 4;
    limits.journal.writes += 128;
    limits.authority.segments = 8;
    limits.authority.segment_bytes = 64;
    limits.tasks.authority_segments = 9;
    limits.tasks.authority_bytes = 128;
    limits.notes.scopes = 4;
    let mut authority_rules = config.core.defaults.rules.clone();
    authority_rules.ceiling.grants = Box::new([grant.clone()]);
    authority_rules.ceiling.tools = tools;
    let mut policy = config.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.grants = Box::new([grant.clone()]);
    policy.ceiling.tools = tools;
    policy.projections = Box::new([grant.clone()]);
    // A chat's authority is its role's, one level shallower.
    policy.roles[0].authority.grants = Box::new([grant]);
    policy.roles[0].authority.tools = tools;
    let mut roles = policy.roles.to_vec();
    for number in 1..4 {
        let mut role = roles[0].clone();
        role.number = number;
        roles.push(role);
    }
    policy.roles = roles.into_boxed_slice();
    let mut domain = authority::Domain::new(authority_rules, limits.authority).expect("read authority");
    let mut out = Queue::uncapped();
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
    config.core.defaults.rules = domain.rules().clone();
}

fn say(project: u32, task: u64, words: Box<[u8]>, release: Option<u64>) -> people::Ask {
    match release {
        Some(revision) => {
            let mut operations = skein_lib::List::with_capacity(2);
            operations.push(people::Operation::Message { task, words }).expect("two operations");
            operations.push(people::Operation::Release { task, revision, again: None }).expect("two operations");
            people::Ask::Compound { project, operations }
        }
        None => people::Ask::Say { project, task, words },
    }
}

// Independent durable identities exercise the account/source translation in both roots.
pub const BILLING_ACCOUNT: u32 = 43;
pub const FIRST_SOURCE: u32 = 17;
pub const SECOND_SOURCE: u32 = 71;

fn configured_parts(mut config: engine::Config, repository: bool) -> (engine::Config, engine::Limits) {
    let mut limits = if config.core.defaults.charters.len() > 1 {
        temper_engine_domain_world::forge_routes::box_limits()
    } else {
        walking::limits()
    };
    if config.core.defaults.charters.len() == 1 {
        limits.journal.transcript_bytes = 32768;
        limits.loads.transcript_bytes = 32768;
        limits.fleet.turns = 64;
        limits.tasks.inbox_bytes = 1024;
        limits.journal.result_bytes = 4096;
    }
    if repository && config.core.defaults.charters.len() == 1 {
        grant_read(&mut config, &mut limits);
    }
    (config, limits)
}

fn forge_read_limits(limits: &mut engine::Limits) {
    limits.call_answer_bytes = 32768;
    limits.forge.client.answer_bytes = 65_536;
    limits.journal.transcript_bytes = 131_072;
    limits.loads.transcript_bytes = 131_072;
}

fn refused_forge_limits(limits: &mut engine::Limits) {
    forge_read_limits(limits);
    limits.forge.client.answer_bytes = 4096;
}

fn repository_forge(seed: u64) -> Forge {
    let mut forge = Forge::new(seed);
    forge.repository_with_checks(
        b"org/repo",
        b"main",
        Tree::from([
            (b"README.md".to_vec(), b"The answer is 42.\n".to_vec()),
            (b"AGENTS.md".to_vec(), b"Report what you found.\n".to_vec()),
            (b"src/lib.rs".to_vec(), b"pub fn answer() -> u32 { 42 }\n".to_vec()),
            (b"src/main.rs".to_vec(), b"fn main() {}\n".to_vec()),
        ]),
        agreement::checks(),
    );
    forge.grant(b"org/repo", 1, fake::Permission::Admin);
    forge.grant(b"org/repo", 7, fake::Permission::Read);
    forge
}

fn chat_ending(config: &engine::Config) -> tasks::EndedBy {
    config.core.defaults.templates.iter().find(|template| template.number == 1).expect("chat template").flags.ended_by
}
