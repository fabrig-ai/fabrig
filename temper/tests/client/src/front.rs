//! Real engine root on its direct store driver, with jig's protocol correlation.
//! Forge records and playground scripts (temper/docs/design/client/forge/deployment.md; temper/docs/design/client/testing.md).
pub mod account_acts;
mod away;
mod dismissed;
mod status;
use crate::app::Event;
use crate::{app::Temper, fixtures, translate};
use jig_client_kit::{
    harness::{ClientId, Front, FrontFault, Observed},
    joint::transport::Transport,
};
use jig_domain_link::{EndpointBound, Phase};
use skein_lib::{Duration, RequestEnvelope, Time, Token, Wall};
use smith_domain_session::llm::Block;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use temper_client_domain::{self as client, Event as RootEvent, Request};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

pub mod ways;

pub const PRINTED: [u8; 32] = [9; 32];

#[must_use]
pub fn printed_link() -> Box<[u8]> {
    link_url(&PRINTED)
}

fn link_url(secret: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let prefix = b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=";
    let encoded = base64::encoded_len(Alphabet::Url, 32).expect("printed secret size");
    let mut out = skein_lib::Writer::new(prefix.len() + usize::try_from(encoded).expect("short encoding"));
    out.put(prefix).expect("measured prefix");
    base64::encode(Alphabet::Url, secret, &mut out).expect("measured secret");
    out.finish()
}

/// The common client Front backed by temper's own engine root.
#[expect(clippy::struct_excessive_bools, reason = "independent fixture switches keep existing worlds unchanged")]
pub struct Joint {
    pub engine: Driver,
    pub sign_ins: u32,
    dismissal_sends: usize,
    status_looks: BTreeMap<ClientId, usize>,
    /// The ctl caller in this world; existing stories use person 1.
    pub caller: u64,
    sign_in_identity: Option<jig_domain_people::Identity>,
    pub leaves: Vec<Box<[u8]>>,
    pub tabs: Vec<client::Tab>,
    pub shows: Vec<(client::Address, bool)>,
    set_ups: Vec<(u16, Token)>,
    pub admin_connects: Vec<(u16, Token)>,
    pub replacements: Vec<(u16, Token)>,
    pub playground: Option<temper_client_protocol_wire::v1::PlaygroundAnswer>,
    pub deployment: (bool, bool),
    pub program_environment: Box<[u8]>,
    pub hold_playground_end: bool,
    held_playground_end: Vec<(ClientId, Event)>,
    pub playground_ends: Vec<skein_lib::RequestKey>,
    pub playground_starts: Vec<skein_lib::RequestKey>,
    pub playground_removals: Vec<skein_lib::RequestKey>,
    pub playground_reads: usize,
    pub note_reads: usize,
    pub note_read_ends: VecDeque<jig_client::contract::ReadNone>,
    /// Terminal replies for successive task-document attempts, before the real engine serves one.
    pub task_read_ends: VecDeque<jig_client::contract::ReadNone>,
    /// Terminal replies for activity reads, before the real engine serves one.
    pub activity_read_ends: VecDeque<jig_client::contract::ReadNone>,
    pub reachable_reads: Vec<(u16, Option<u64>, u32)>,
    pub reachable_empty: bool,
    pub reachable_pages: bool,
    pub installs: Vec<(u16, Token)>,
    pub playground_outcomes: VecDeque<temper_client_protocol_wire::v1::Outcome>,
    pub answer_runs: bool,
    pub reply_runs: bool,
    pub runs_answered: u32,
    /// Delivered me documents, including the initial watch answer.
    pub me_deliveries: usize,
    /// Worker assignments retained for assertions about requester notifications.
    pub assignments: Vec<engine::Assignment>,
    trial_config: Option<engine::Config>,
    reply_script: Option<ReplyScript>,
    held_reply: Option<HeldReply>,
    hold_playground_read: bool,
    held_playground_reads: VecDeque<(ClientId, Event)>,
    pub park_runs: bool,
    settles_with_a_deadline_armed: bool,
    keyed_fault: Option<RequestEnvelope>,
    reads: usize,
    read_loads: BTreeSet<Token>,
    transport: Transport,
    component: crate::component::Composition,
    taking: BTreeMap<ClientId, jig_client::Taking>,
    limits: client::Limits,
    client_limits: BTreeMap<ClientId, client::Limits>,
    next_limits: Option<client::Limits>,
    callbacks: Vec<(ClientId, Event)>,
    parked: BTreeMap<ClientId, Vec<Request>>,
    offline_until: Option<Time>,
    linked_heard: BTreeMap<ClientId, Vec<jig_client::Event>>,
    due: Option<Time>,
    observed: Vec<Observed>,
    lose_answers: u32,
    lost_answer: Option<(ClientId, Token)>,
    hosting: BTreeMap<u64, u64>,
    pub(crate) seed: u64,
    now: Time,
    last_commit: u64,
    sign_outs: BTreeMap<ClientId, VecDeque<Option<u64>>>,
    truth: jig_client_kit::joint::truth::Truth,
}
/// Typed replies observed by a scripted run (temper/docs/design/testing/checks.md, 3).
#[derive(Debug)]
pub enum HostAnswer {
    See(Box<engine::SeeAnswer>),
    Tool(engine::CallAnswer),
}
enum ReplyScript {
    AtOnce(&'static [u8]),
    Pieces(&'static [&'static [u8]]),
}

struct HeldReply {
    task: u64,
    attempt: u64,
    read: Option<u64>,
    pieces: &'static [&'static [u8]],
    next: usize,
    words: Vec<u8>,
}
fn split_chat_allowance(config: &mut engine::Config, chats: u32) {
    if chats > 1 {
        for template in &mut config.core.defaults.templates {
            if template.number == 5 {
                template.budget = jig_domain_people::TemplateBudget::AsAllowed { cap: Some(500 / u64::from(chats)) };
            }
        }
    }
}

fn asking_limits(projects: u32, chats: u32) -> engine::Limits {
    let mut engine_limits = engine_limits();
    engine_limits.people.projects = projects + 1;
    engine_limits.authority.projects = projects;
    engine_limits.forge.judge_projects = projects;
    engine_limits.people.holdings = projects + 1;
    engine_limits.tasks.charters = 2;
    engine_limits.tasks.executor_kinds = 2;
    engine_limits.authority.executors = 2;
    engine_limits.authority.roles = 2;
    engine_limits.documents.executors = 2;
    engine_limits.templates = 3;
    engine_limits.people.initial_owners = projects + 1;
    engine_limits.people.requests = 16;
    // Two fixture sign-ins, a member's ctl, its owner's ctl and a fresh replay.
    engine_limits.people.sign_ins = 5;
    engine_limits.people.words = 512;
    engine_limits.tasks.message_bytes = 512;
    engine_limits.tasks.inbox_bytes = 1024;
    engine_limits.journal.result_bytes = 512;
    // Stock charter 11's instructions and framed run policy need 10,766 bytes.
    engine_limits.journal.run_bytes = 12_288;
    // A stored stock charter 11 has 9,542 payload bytes, including model cache lifetimes.
    engine_limits.loads.transcript_bytes = 9_542;
    engine_limits.documents.text_bytes = 4096;
    limits_for_chats(&mut engine_limits, chats);
    engine_limits
}

fn limits_for_chats(limits: &mut engine::Limits, chats: u32) {
    limits.tasks.tasks = limits.tasks.tasks.max(chats);
    limits.tasks.project_tasks = limits.tasks.project_tasks.max(chats);
    limits.people.inbox_tasks = limits.people.inbox_tasks.max(chats * 2);
    if chats > 1 {
        limits.documents.rows = 2;
    }
}

#[must_use]
pub fn engine_limits() -> engine::Limits {
    let mut limits = temper_engine_domain_world::fixtures::forge_limits();
    // The world's engine admits the whole starters and cues its client offers.
    limits.people.words = limits.people.words.max(temper_client_protocol_wire::CUE_BYTES);
    limits.tasks.spec_bytes = limits.tasks.spec_bytes.max(temper_client_protocol_wire::CUE_BYTES);
    limits.tasks.message_bytes = limits.tasks.message_bytes.max(limits.people.words);
    limits.tasks.inbox_bytes = limits.tasks.inbox_bytes.max(limits.tasks.message_bytes);
    // The world's own bound, not production's: automatic Seen takes a key per arrival.
    limits.people.requests = 5;
    // A goal's driver plus two clients, as a fresh client at an address beside the person's.
    limits.people.sign_ins = 3;
    let tools = temper_client_tools::limits::client().client.domain;
    // Frame project allocations must also fit the smaller tools project bound.
    limits.documents.lines = limits.documents.lines.min(tools.projects);
    limits.watermark = engine::default_watermark(&limits.journal);
    limits
}
// First-run client worlds cover the seeded project, a new project and its owner.
const FIRST_RUN_WRITES: u32 = 2120;

/// The first-run stories need the seeded project and one made at runtime.
/// Use the real `MakeProject` path, with a configured policy and room for its owner.
pub fn first_run_projects(config: &mut engine::Config, limits: &mut engine::Limits) {
    limits.authority.projects = 2;
    limits.people.projects = 3; // The deployment roster and two projects.
    limits.people.holdings = 3;
    limits.journal.writes = FIRST_RUN_WRITES;
    let seed = &config.core.seed_projects[0].policy;
    let policy = jig_domain_core::ConfiguredPolicy {
        policy: seed.policy.clone(),
        // The seed's permission probes name unoffered roles; a new project's
        // owner needs no such bindings. Settings check all policy references.
        permissions: seed
            .permissions
            .iter()
            .filter(|permission| seed.policy.roles.iter().any(|role| role.number == permission.role))
            .copied()
            .collect(),
        templates: seed.templates.clone(),
    };
    config.core.defaults.policies.insert(1, policy).expect("first-run policy");
}
fn restored_forges(store: Store, bootstrap: engine::Config, config: &engine::Config, limits: &engine::Limits) -> Store {
    if !store.rows.is_empty() {
        return store;
    }
    let mut store = temper_engine_domain_world::fixtures::restored_over(store, bootstrap, limits);
    let mut records = Vec::new();
    let mut last = 0;
    for forge in &config.forges {
        last = last.max(forge.number);
        let row = temper_engine_domain_forge::ForgeRecord {
            person: None,
            git_program: None,
            number: forge.number,
            kind: forge.kind,
            mode: forge.mode,
            state: temper_engine_domain_forge::ForgeState::Kept,
            place: temper_engine_domain_forge::Place {
                api: temper_engine_domain_forge::Address { host: fixtures::FORGE_HOST.into(), port: 443, tls: None },
                git: forge.git.clone(),
                web: forge.web.clone(),
            },
            read_only: forge.read_only,
            bot: temper_engine_domain_forge::Bot { login: forge.bot.clone(), name: Box::new([]), email: Box::new([]) },
            asked: forge.asked,
            first: None,
        };
        records.push(engine::Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::Forge(row)) });
    }
    if last != 0 {
        records.push(engine::Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::ForgeNumber { last }) });
        store.seed(records.into_boxed_slice(), Box::new([]));
    }
    store
}
impl Joint {
    /// Shipped page watch sets beside a third client (temper/docs/design/protocol/configuration/limits.md).
    #[must_use]
    pub fn with_watch_room(seed: u64, limits: &client::Limits) -> Self {
        Self::with_watch_room_streams(seed, limits, 8)
    }

    /// A bounded room for the Busy and recovery stories (protocol/configuration/limits.md).
    #[must_use]
    pub fn with_watch_room_streams(seed: u64, limits: &client::Limits, streams: u32) -> Self {
        let delegates = fixtures::WATCH_ROOM_DELEGATES;
        let tasks = delegates + 1;
        let configs = std::array::from_fn(|_| {
            use jig_domain_authority as authority;
            let mut config = walking::config(seed);
            config.core.defaults.rules.ceiling.delegation.tasks = tasks;
            config.core.defaults.rules.ceiling.delegation.depth = 2;
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.ceiling.delegation.tasks = tasks;
            policy.ceiling.delegation.depth = 2;
            policy.roles[0].authority.delegation.tasks = tasks;
            policy.roles[0].authority.delegation.depth = 2;
            let mut authority_limits = walking::limits().authority;
            authority_limits.batch = delegates;
            let mut domain = authority::Domain::new(config.core.defaults.rules.clone(), authority_limits)
                .expect("watch-room authority");
            let mut out = skein_lib::Queue::uncapped();
            authority::step(
                &mut domain,
                authority::Event::Policy { project: fixtures::QUESTION_PROJECT, policy: policy.clone() },
                &mut out,
            );
            assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: fixtures::QUESTION_PROJECT }));
            config.core.seed_projects[0].policy.policy =
                domain.policy(fixtures::QUESTION_PROJECT).expect("watch-room project policy").clone();
            config.core.defaults.rules = domain.rules().clone();
            config
        });
        let mut engine_limits = engine_limits();
        engine_limits.streams = streams;
        // The goal's driver and three independent client roots.
        engine_limits.people.sign_ins = 4;
        engine_limits.tasks.tasks = tasks;
        engine_limits.tasks.project_tasks = tasks;
        engine_limits.tasks.tree_tasks = tasks;
        engine_limits.tasks.delegates = delegates;
        engine_limits.tasks.batch = delegates;
        engine_limits.tasks.inbox_messages = tasks;
        engine_limits.tasks.inbox_bytes = engine_limits.tasks.message_bytes * tasks;
        engine_limits.authority.batch = delegates;
        engine_limits.people.inbox_tasks = tasks * 2;
        engine_limits.people.inbox_recipients = tasks * 2 + engine_limits.authority.roles + 4;
        engine_limits.documents.rows = limits.client.domain.rows;
        // The delegated batch retains its writes and deliveries together.
        engine_limits.journal.writes = 4096;
        engine_limits.journal.deliveries = 64;
        engine_limits.journal.held = 192;
        engine_limits.fleet.slots = tasks;
        engine_limits.fleet.attempts = tasks * 2;
        let mut front =
            Self::with_engine(seed, limits, Store::with_limits(&engine_limits), configs, &engine_limits, tasks);
        // A reachable engine can refuse admission. This story checks readiness through
        // its person's waits and tools call, rather than promising every link is admitted.
        front.observed.clear();
        front
    }

    /// The page retains its address and its unlinked `Start` supplies `SignedIn` itself.
    pub fn resume_watch_room_page(&mut self, client: ClientId) {
        self.callbacks.retain(|(owner, event)| {
            *owner != client
                || !matches!(
                    event,
                    Event::Root(
                        RootEvent::Went { .. }
                            | RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }
                    )
                )
        });
    }

    /// Close the tab's transport and drain its engine watches.
    pub fn close_client(&mut self, client: ClientId) {
        self.detach(client);
    }

    /// Open protocol watches, leaving closing ones out (temper/docs/design/protocol/configuration/limits.md).
    #[must_use]
    pub fn open_watches(&self) -> usize {
        self.transport.watches().len()
    }

    pub fn finish_playground_reset(&mut self) {
        if !self.held_playground_end.is_empty() {
            self.playground = None;
            self.callbacks.append(&mut self.held_playground_end);
            self.publish_playground_forges();
        }
    }
    /// Hold the next playground read's encoded answer, including its original client and read token.
    /// Arming again lets a story hold a newer answer behind the older one.
    pub fn hold_next_playground_read(&mut self) {
        assert!(!self.hold_playground_read, "the next playground read is already armed");
        self.hold_playground_read = true;
    }

    /// Queue the oldest held answer unchanged; the normal client pump delivers it.
    pub fn release_playground_read(&mut self) -> bool {
        let Some(answer) = self.held_playground_reads.pop_front() else { return false };
        self.callbacks.push(answer);
        true
    }

    /// The scripted setup host failed to create the requested first repository.
    pub fn fail_first_repository(&mut self, forge: u16) {
        assert!(self.set_ups.iter().any(|(number, _)| *number == forge), "setup must reach the host first");
        self.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::FirstFailed {
            forge,
            step: temper_engine_domain_forge::CreateStep::Repository,
        })));
        self.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
            forge,
            state: temper_engine_domain_forge::Connection::SetUpFailed {
                why: b"making the repository".as_slice().into(),
            },
        })));
        self.engine.settle();
    }

    /// The GitHub first run: real project/adoption engine, fixed reachable page.
    #[must_use]
    pub fn github_fixture(seed: u64) -> (Self, crate::app::Root) {
        Self::github_fixture_with(seed, true, false)
    }
    /// GitHub at the setup boundary: optionally uninstalled, or with no repositories.
    #[must_use]
    pub fn github_fixture_with(seed: u64, installed: bool, empty: bool) -> (Self, crate::app::Root) {
        use temper_engine_domain_forge as forge;
        let limits = fixtures::domain_limits();
        let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let mut engine_limits = engine_limits();
            first_run_projects(&mut config, &mut engine_limits);
            config.core.connectors = Box::new([fixtures::FORGE]);
            config.forges = Box::new([forge::Configured {
                admin_connect: false,
                bot: Box::new([]),
                owned: None,
                number: 1,
                kind: forge::Kind::GitHub,
                mode: forge::Mode::Connected,
                git: Box::new([]),
                web: Some(b"https://github.test".as_slice().into()),
                read_only: false,
                asked: false,
            }]);
            (config, engine_limits)
        });
        let mut front = Self::with_engine(
            seed,
            &limits,
            Store::with_limits(&engine_limits),
            [bootstrap, config],
            &engine_limits,
            1,
        );
        front.reachable_empty = empty;
        front.engine.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[
            (b"acme", b"shop", 7),
            (b"acme", b"second", 8),
        ]));
        front.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"app[bot]".as_slice().into() },
        })));
        if installed {
            front.engine.send(engine::Event::Forge(Box::new(forge::Event::Installation {
                forge: 1,
                installation: 1,
                state: Some(forge::Installation {
                    account: Some((1, b"acme".as_slice().into())),
                    stray: false,
                    suspended: false,
                }),
            })));
        }
        front.engine.settle();
        let who = ClientId(0);
        front.bind(who, fixtures::PARTY);
        let mut root = crate::app::Root::Person(Box::new(client::unlinked(&limits, seed)));
        account_acts::event(
            &mut front,
            &mut root,
            <Temper as jig_client_kit::harness::App>::start(
                client::Address::ConnectForge { forge: Some(client::Choice::Forge(1)) },
                Vec::new(),
                0,
            ),
        );
        (front, root)
    }
    /// Commit the next assigned chat's reply before the client can read its activity.
    pub fn reply_at_once(&mut self, words: &'static [u8]) {
        assert!(self.reply_script.is_none() && self.held_reply.is_none());
        self.reply_script = Some(ReplyScript::AtOnce(words));
    }

    /// The next chat run's reply, written in these pieces: the first is sent at the
    /// run's assignment and each later part at one `release_reply` (testing/checks.md 3).
    pub fn reply_in_pieces(&mut self, pieces: &'static [&'static [u8]]) {
        assert!(self.reply_script.is_none() && self.held_reply.is_none(), "one scripted reply at a time");
        assert!(!pieces.is_empty() && pieces.iter().all(|piece| !piece.is_empty()), "a reply has nonempty pieces");
        self.reply_script = Some(ReplyScript::Pieces(pieces));
    }

    /// Send the held reply's next part: its next piece, or, once every piece is sent,
    /// the committed turn with the whole text and the run's answer. False when no reply
    /// is held.
    pub fn release_reply(&mut self) -> bool {
        let Some(mut reply) = self.held_reply.take() else { return false };
        if let Some(bytes) = reply.pieces.get(reply.next) {
            self.engine.send(temper_engine_domain_world::protocol::up(
                Token::new(7),
                jig_domain_host::LinkUp::Piece {
                    run: Token::new(reply.task),
                    attempt: Token::new(reply.attempt),
                    piece: jig_domain_host::Piece {
                        turn: 1,
                        retry: 0,
                        kind: jig_domain_host::LiveKind::Text,
                        offset: skein_lib::len_u64(reply.words.len()),
                        bytes: (*bytes).into(),
                    },
                },
                engine::Parts::none(),
            ));
            reply.words.extend_from_slice(bytes);
            reply.next += 1;
            self.held_reply = Some(reply);
        } else {
            self.finish_reply(reply.task, reply.attempt, reply.read, Some(&reply.words));
        }
        self.settle();
        true
    }

    fn finish_reply(&mut self, task: u64, attempt: u64, read: Option<u64>, words: Option<&[u8]>) {
        self.runs_answered += 1;
        if let Some(words) = words {
            let mut turn = temper_worker_domain_world::protocol::session_turn(1, 1, words.len(), true);
            turn.messages[0].content = Box::new([Block::Text { text: words.into(), replay: None }]);
            let body = smith_protocol_channel::encode_turn(
                &turn,
                &smith_transcript::CEILINGS,
                &temper_worker_domain_world::protocol::endpoints(),
            )
            .expect("scripted reply encodes");
            self.engine.send(temper_engine_domain_world::protocol::turn(
                Token::new(7),
                task,
                attempt,
                1,
                1,
                read,
                &body,
            ));
            self.engine.settle();
        }
        self.engine.answer(
            Token::new(7),
            task,
            attempt,
            u64::from(words.is_some()),
            jig_domain_tasks::End::Finished {
                result: jig_domain_tasks::TaskResult::Report { words: b"Answered by the run".as_slice().into() },
                cancel_delegates: false,
            },
            engine::Parts::none(),
        );
    }

    #[must_use]
    pub fn set_ups(&self) -> &[(u16, Token)] {
        &self.set_ups
    }
    #[must_use]
    pub fn answers_to_lose(&self) -> u32 {
        self.lose_answers
    }
    #[must_use]
    pub fn engine_adopted(&self, forge: u16, repository: u32) -> bool {
        use temper_engine_domain_forge::{Key, Role, Stored};
        let key = engine::Key::Forge(Key::Repository(skein_forge::api::Repository { forge, repository }));
        let Some(engine::Record::Forge { row }) = self.engine.store.rows.get(&key) else { return false };
        matches!(row.as_ref(), Stored::Repository(value) if value.role == Role::Owned)
    }
    #[must_use]
    pub fn applications_admitted(&self) -> usize {
        self.engine
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(record,
                    engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Answer { ask, .. }))
                    if matches!(ask.as_ref(), jig_domain_people::Ask::Application { .. })
                )
            })
            .count()
    }
    #[must_use]
    pub fn adoption(seed: u64, limits: client::Limits) -> Joint {
        let mut engine_limits = engine_limits();
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            first_run_projects(&mut config, &mut engine_limits);
            config.core.connectors = Box::new([fixtures::FORGE]);
            config
        });
        let mut front =
            Self::with_engine(seed, &limits, Store::with_limits(&engine_limits), configs, &engine_limits, 1);
        front.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
            forge: 1,
            state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        })));
        front.engine.settle();
        front
    }
    #[must_use]
    pub fn new(seed: u64, limits: client::Limits) -> Joint {
        let engine_limits = engine_limits();
        Self::with_engine(
            seed,
            &limits,
            Store::with_limits(&engine_limits),
            [walking::config(seed), walking::config(seed)],
            &engine_limits,
            1,
        )
    }
    /// A client world with the component's browser approval enabled.
    #[must_use]
    pub fn with_approval(seed: u64, limits: client::Limits) -> Self {
        let mut joint = Self::new(seed, limits);
        joint.component =
            crate::component::Composition::new_with_approval(&walking::config(seed), limits.client.domain);
        joint
    }

    pub fn as_browser(&mut self, client: ClientId) {
        self.component.as_browser(client);
    }

    pub fn ask_approval(&mut self, client: ClientId) -> (jig_client::ApprovalCode, Wall) {
        self.component.ask_approval(client);
        self.pump_component();
        self.component.asked(client).expect("the component answered asked")
    }

    pub fn take_approval(&mut self, client: ClientId) {
        self.taking.remove(&client);
        self.component.take_approval(client);
        self.pump_component();
    }

    #[must_use]
    pub fn taking(&self, client: ClientId) -> Option<jig_client::Taking> {
        self.taking.get(&client).copied()
    }

    #[must_use]
    pub fn with_printed_link(seed: u64, limits: client::Limits) -> Joint {
        let mut engine_limits = engine_limits();
        // Sign-outs in TUI stories must precede the host's minute of expiry slack.
        engine_limits.people.sign_in_lifetime = Duration::from_secs(120);
        let mut joint = Self::with_engine(
            seed,
            &limits,
            Store::with_limits(&engine_limits),
            [walking::config(seed), walking::config(seed)],
            &engine_limits,
            1,
        );
        joint.offer_printed_link(limits);
        joint
    }

    /// Four retained Ask temper chats sharing the fixture's allowance, with two-row
    /// activity pages and the TUI's pasted-link sign-in (client/testing.md).
    #[must_use]
    pub fn with_asking_printed_link(seed: u64, limits: client::Limits) -> Self {
        let mut joint = Self::asking_chats(seed, limits, false, 1, 4);
        joint.offer_printed_link(limits);
        joint
    }

    /// Replaces the fixture's client component, ending its setup clients first.
    pub(crate) fn offer_printed_link(&mut self, limits: client::Limits) {
        for who in self.transport.clients() {
            if let Some(sign_in) = self.transport.sign_in(who) {
                self.component.sign_out(who, sign_in);
                self.pump_component();
            }
            self.detach(who);
        }
        self.component = crate::component::Composition::with_printed_link(
            &walking::config(self.seed),
            limits.client.domain,
            jig_domain_people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
            skein_lib::Secret::new(Box::new(PRINTED)),
        );
        self.pump_component();
    }

    pub fn link_client(&mut self, who: ClientId, limits: jig_client::DomainLimits) {
        self.advance(self.now);
        self.component.link_client(who, limits);
        self.linked_heard.entry(who).or_default();
    }

    #[must_use]
    pub fn latest_link(&self) -> Option<Box<[u8]>> {
        self.component.latest_link.as_ref().map(|secret| link_url(secret.expose()))
    }

    #[must_use]
    pub fn local_link(&self) -> Option<Box<[u8]>> {
        self.component.local_link.as_ref().map(|secret| link_url(secret.expose()))
    }

    pub fn carry(&mut self, who: ClientId, now: Time, request: jig_client::Request) {
        self.advance(now);
        if self.offline_until.is_some_and(|until| now < until) {
            let heard = self.linked_heard.get_mut(&who).expect("linked client");
            heard.extend(self.component.heard(who));
            match request {
                jig_client::Request::Exchange { exchange, .. } => heard.push(jig_client::Event::Lost { exchange }),
                jig_client::Request::OpenStream { .. } => heard.push(jig_client::Event::StreamGone),
                jig_client::Request::Save { .. } | jig_client::Request::Erase { .. } => {
                    heard.push(jig_client::Event::Stored);
                }
                jig_client::Request::CloseStream
                | jig_client::Request::Told { .. }
                | jig_client::Request::Approval(_)
                | jig_client::Request::Authorized(_)
                | jig_client::Request::ServiceCredential { .. } => {}
            }
            return;
        }
        self.component.carry(who, request);
        self.pump_component();
    }

    pub fn heard(&mut self, who: ClientId, now: Time) -> Vec<jig_client::Event> {
        self.advance(now);
        self.pump_component();
        self.drain();
        let mut heard = std::mem::take(self.linked_heard.get_mut(&who).expect("linked client"));
        heard.extend(self.component.heard(who));
        heard
    }
    /// A chat with authority to write its document, read through the real client component.
    #[must_use]
    pub fn with_chat_draft(seed: u64, limits: client::Limits) -> Self {
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            config.core.connectors = Box::new([fixtures::FORGE]);
            for permission in &mut config.core.seed_projects[0].policy.permissions {
                permission.connector = fixtures::FORGE;
            }
            let kinds: Box<[jig_domain_authority::Executor]> =
                Box::new([jig_domain_authority::Executor::Charter(1), jig_domain_authority::Executor::Role(0)]);
            config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
            config.core.defaults.rules.ceiling.delegation.depth = 2;
            config.core.defaults.rules.ceiling.notes = jig_domain_authority::Scopes(1);
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.ceiling.delegation.kinds.clone_from(&kinds);
            policy.roles[0].authority.delegation.kinds = kinds;
            policy.ceiling.delegation.depth = 2;
            policy.roles[0].authority.delegation.depth = 2;
            policy.ceiling.notes = jig_domain_authority::Scopes(1);
            policy.roles[0].authority.notes = jig_domain_authority::Scopes(1);
            for template in &mut config.core.defaults.templates {
                if template.tracked.is_some() {
                    template.deadline = jig_domain_people::TemplateDeadline::None;
                }
            }
            config
        });
        // The chat snapshot includes its instructions, goal line and provenance.
        let mut engine_limits = engine_limits();
        // One slot for the chat and one for its goal.
        engine_limits.fleet.slots = 2;
        // The chat, its goal and one person answer; charter and role executors.
        engine_limits.tasks.tasks = 3;
        engine_limits.tasks.project_tasks = 3;
        engine_limits.people.inbox_tasks = 6;
        engine_limits.authority.executors = 2;
        engine_limits.tasks.executor_kinds = 2;
        engine_limits.documents.executors = 2;
        // The goal asks with the report contract's 4096-byte answer bound.
        engine_limits.tasks.result_bytes = 4096;
        engine_limits.forge.client.op_bytes = engine_limits.forge.client.op_bytes.max(engine_limits.tasks.result_bytes);
        engine_limits.journal.result_bytes = engine_limits.tasks.result_bytes;
        engine_limits.journal.transcript_bytes = 32768;
        engine_limits.tasks.spec_bytes = 1024;
        engine_limits.people.words = 1024;
        engine_limits.tasks.message_bytes = engine_limits.people.words;
        // A tracked goal reserves its news beside the child's report.
        engine_limits.tasks.inbox_bytes = engine_limits
            .tasks
            .result_bytes
            .checked_add(engine_limits.tasks.message_bytes)
            .expect("report and news fit");
        engine_limits.journal.result_bytes = engine_limits.journal.result_bytes.max(engine_limits.people.words);
        engine_limits.watermark = engine::default_watermark(&engine_limits.journal);
        Self::with_engine(seed, &limits, Store::with_limits(&engine_limits), configs, &engine_limits, 2)
    }
    /// Temper's stock kinds through the joined client and engine roots.
    #[must_use]
    pub fn with_kinds(seed: u64, limits: client::Limits) -> Self {
        let mut engine_limits = engine_limits();
        let kinds_limits = temper_engine_domain_world::kinds::limits();
        engine_limits.tasks.charters = kinds_limits.tasks.charters;
        engine_limits.people.amendment_bytes = kinds_limits.people.amendment_bytes;
        let [bootstrap, config, restart] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let kinds = temper_engine_domain_world::kinds::configuration(&engine_limits);
            config.core.kinds = kinds.core.kinds;
            config.core.defaults.charters = kinds.core.defaults.charters;
            config.kinds_stock = kinds.kinds_stock;
            config
        });
        // Stock kinds add their defaults during cold start. Transfer the real
        // store without the generic fixture's fixed first-start commit count.
        let mut driver = Driver::cold(Store::with_limits(&engine_limits), bootstrap, &engine_limits);
        driver.settle();
        let store = driver.take_store();
        Self::with_engine(seed, &limits, store, [config, restart], &engine_limits, 1)
    }
    /// Ask temper's stock charter and template in the joined world
    /// (temper/docs/design/domain/agent/ask-temper.md).
    #[must_use]
    pub fn with_asking(seed: u64, limits: client::Limits, member: bool) -> Self {
        Self::with_asking_projects(seed, limits, member, 1)
    }

    /// Asking proposals read across this many seeded projects.
    #[must_use]
    pub fn with_asking_projects(seed: u64, limits: client::Limits, member: bool, projects: u32) -> Self {
        Self::asking_chats(seed, limits, member, projects, 1)
    }

    fn asking_chats(seed: u64, limits: client::Limits, member: bool, projects: u32, chats: u32) -> Self {
        let engine_limits = asking_limits(projects, chats);
        let mut config = fixtures::asking_configuration(seed, member);
        split_chat_allowance(&mut config, chats);
        let policy = config.core.seed_projects[0].policy.clone();
        config.core.seed_projects = (1..=projects)
            .map(|number| jig_domain_core::SeedProject {
                number,
                name: format!("Project {number}").into_boxed_str(),
                policy: policy.clone(),
            })
            .collect();
        let owner = config.core.owners[0].clone();
        let mut owners = config.core.owners.into_vec();
        for project in 2..=projects {
            let mut owner = owner.clone();
            owner.project = project;
            owners.push(owner);
        }
        config.core.owners = owners.into_boxed_slice();
        let mut driver = Driver::cold(Store::with_limits(&engine_limits), config, &engine_limits);
        driver.settle();
        // The account component plays a fake key; no secret bytes enter this world.
        driver.send(engine::Event::Account(jig_domain_accounts::Event::Change {
            request: Token::new(2000),
            holder: jig_domain_accounts::Holder::Deployment,
            change: jig_domain_accounts::Change::AddKey { provider: 1, join: None, carried: 0 },
        }));
        driver.settle();
        let caller = if member { fixtures::asking_member(&mut driver) } else { fixtures::PARTY };
        let (transport, component) = Self::client_parts(&walking::config(seed), &limits);
        driver.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(fixtures::REPOSITORIES));
        let mut joint = Self::with_driver(seed, &limits, driver, transport, component, false);
        joint.caller = caller;
        // The worker offers stock charter 11's run bounds.
        let mut hello = temper_engine_domain_world::protocol::hello(fixtures::QUESTION_WORKER, Box::new([]));
        let engine::Event::Worker(engine::WorkerInput::Link {
            link: engine::Link::Hello { hello: offered, .. }, ..
        }) = &mut hello
        else {
            unreachable!("worker hello")
        };
        offered.bounds.time = Duration::from_secs(3600);
        offered.bounds.turns = 64;
        offered.bounds.message_bytes = engine_limits.tasks.message_bytes;
        joint.engine.send(hello);
        joint.engine.settle();
        joint.last_commit = joint.engine.store.header().commits;
        joint.truth = jig_client_kit::joint::truth::Truth::after(core_rows(&joint.engine));
        joint.engine.delivered.clear();
        joint.engine.immediate.clear();
        joint
    }

    /// Subsequent ctl clients sign in as the asking world's seeded owner.
    pub fn asking_owner(&mut self) {
        self.caller = fixtures::PARTY;
        self.sign_in_identity = Some(jig_domain_people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8),
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        });
    }

    /// Play a decoded run host call through the worker link. Decode and render
    /// are scripted here, as in the domain world's protocol stand.
    pub fn host_tool(&mut self, task: u64, completion: u32, tool: engine::Tool) -> HostAnswer {
        let attempt = self.hosted_attempt(task).expect("the task reached its worker");
        self.engine.protocol.defer_render = true;
        let event = self.engine.protocol.call(
            fixtures::QUESTION_WORKER,
            task,
            attempt,
            u64::from(completion),
            engine::Call { completion, position: 0, tool },
        );
        self.engine.send(event);
        self.engine.pump();
        let (to, answer) = if let Some(index) = self.engine.delivered.iter().position(|delivery| {
            matches!(delivery, engine::Delivery::Host(value)
                if matches!(value.as_ref(), engine::HostDelivery::See { .. }))
        }) {
            let engine::Delivery::Host(value) = self.engine.delivered.remove(index) else { unreachable!() };
            let engine::HostDelivery::See { to, answer, .. } = *value else { unreachable!() };
            (to, HostAnswer::See(answer))
        } else {
            let (to, name) = self.engine.protocol.pending_renders.pop().expect("host answer to render");
            let (answered, answer) = self.engine.protocol.answers.last().expect("typed host answer");
            assert_eq!(*answered, name);
            (to, HostAnswer::Tool(answer.clone()))
        };
        self.engine.send(engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
            to,
            answer: jig_domain_core::SettledAnswer::Host { error: false, body: Box::new([]) },
        }));
        self.settle();
        self.engine.protocol.defer_render = false;
        answer
    }

    /// Commit the run's one-line reply after it hears its proposal's outcome.
    pub fn asking_reply(&mut self, task: u64, words: &[u8]) {
        let run = self.engine.task(task).expect("Ask temper chat");
        let read = run.inbox.last().map(|message| message.number);
        self.finish_reply(task, run.attempt, read, Some(words));
        self.settle();
    }

    /// Chats end only when their requester closes them.
    #[must_use]
    pub fn open_chats(seed: u64, limits: client::Limits) -> Joint {
        Self::open_chats_with_refused_hello(seed, limits, false)
    }
    /// This worker names an unconfigured position, so the engine refuses its hello.
    #[must_use]
    pub fn open_chats_with_a_refused_hello(seed: u64, limits: client::Limits) -> Joint {
        Self::open_chats_with_refused_hello(seed, limits, true)
    }
    fn open_chats_with_refused_hello(seed: u64, limits: client::Limits, refuse_hello: bool) -> Joint {
        let engine_limits = engine_limits();
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            config.core.defaults.templates =
                temper_engine_domain_world::fixtures::templates_ending(jig_domain_tasks::EndedBy::Requester);
            config
        });
        Self::with_engine_with_refused_hello(
            seed,
            &limits,
            Store::with_limits(&engine_limits),
            configs,
            &engine_limits,
            1,
            refuse_hello,
        )
    }
    /// The small profile carries a whole turn while an activity page can size it.
    #[must_use]
    pub fn with_large_turns(seed: u64, limits: &client::Limits) -> Self {
        let profile = temper_service::limits::profile(temper_service::limits::Profile::Small).root;
        Self::with_turn_bytes(seed, limits, profile.journal.transcript_bytes, profile.loads.read_bytes, None)
    }
    /// A committed turn past jig small's cap, with a configured whole-read allowance.
    #[must_use]
    pub fn with_long_turns(seed: u64, limits: &client::Limits) -> Self {
        Self::with_turn_bytes(seed, limits, 131_072, 262_144, None)
    }
    /// Whole display turns and three large tool results fit the stored transcript.
    #[must_use]
    pub fn with_whole_turns(seed: u64, limits: &client::Limits) -> Self {
        Self::with_turn_bytes(seed, limits, 1_048_576, 2_097_152, None)
    }
    /// A whole-turn engine with two activity rows per page.
    #[must_use]
    pub fn with_reading_pages(seed: u64, limits: &client::Limits) -> Self {
        Self::with_turn_bytes(seed, limits, 1_048_576, 2_097_152, Some(2))
    }
    fn with_turn_bytes(seed: u64, limits: &client::Limits, bytes: u32, read_bytes: u64, rows: Option<u32>) -> Self {
        let mut engine_limits = engine_limits();
        engine_limits.journal.transcript_bytes = bytes;
        engine_limits.loads.bytes = bytes;
        engine_limits.loads.reply_bytes = bytes;
        engine_limits.loads.transcript_bytes = bytes;
        engine_limits.loads.read_bytes = read_bytes;
        if let Some(rows) = rows {
            engine_limits.documents.rows = rows;
        }
        Self::with_engine(
            seed,
            limits,
            Store::with_limits(&engine_limits),
            [walking::config(seed), walking::config(seed)],
            &engine_limits,
            1,
        )
    }
    #[must_use]
    pub fn with_goal_proposals(seed: u64, limits: client::Limits) -> Self {
        let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let rules = config.core.defaults.rules.clone();
            let mut policy = config.core.seed_projects[0].policy.policy.clone();
            policy.roles[0].authority.budget.spend = 150;
            let mut domain =
                jig_domain_authority::Domain::new(rules, temper_engine_domain_world::walking::limits().authority)
                    .expect("goal authority");
            let mut out = skein_lib::Queue::uncapped();
            jig_domain_authority::step(
                &mut domain,
                jig_domain_authority::Event::Policy { project: 1, policy },
                &mut out,
            );
            assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
            config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
            config.core.defaults.rules = domain.rules().clone();
            let mut engine_limits = engine_limits();
            engine_limits.fleet.slots = 2;
            engine_limits.fleet.attempts = 4;
            engine_limits.journal.writes = 4096;
            engine_limits.journal.deliveries = 64;
            engine_limits.journal.held = 192;
            (config, engine_limits)
        });
        Self::with_engine(seed, &limits, Store::with_limits(&engine_limits), [bootstrap, config], &engine_limits, 2)
    }
    pub(crate) fn with_questions(seed: u64, limits: &client::Limits) -> Self {
        Self::with_question_text_bound(seed, limits, None, false)
    }
    pub(crate) fn with_question_text_bound(seed: u64, limits: &client::Limits, text: Option<u32>, agree: bool) -> Self {
        Self::with_question_bounds(seed, limits, text, agree, None)
    }
    pub(crate) fn with_question_composer(seed: u64, limits: &client::Limits) -> Self {
        Self::with_question_bounds(seed, limits, None, false, Some(8192))
    }
    fn with_question_bounds(
        seed: u64,
        limits: &client::Limits,
        text: Option<u32>,
        agree: bool,
        words: Option<u32>,
    ) -> Self {
        use jig_domain_authority as authority;
        let depth = if agree { 3 } else { 2 };
        let tasks = if agree { 3 } else { 2 };
        let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let mut rules = config.core.defaults.rules.clone();
            rules.ceiling.delegation.depth = depth;
            rules.ceiling.delegation.tasks = tasks;
            let mut policy = config.core.seed_projects[0].policy.policy.clone();
            policy.ceiling.delegation.depth = depth;
            policy.ceiling.delegation.tasks = tasks;
            policy.roles[0].authority.delegation.depth = depth;
            policy.roles[0].authority.delegation.tasks = tasks;
            let mut domain = authority::Domain::new(rules, temper_engine_domain_world::walking::limits().authority)
                .expect("question authority");
            let mut out = skein_lib::Queue::uncapped();
            authority::step(
                &mut domain,
                authority::Event::Policy { project: fixtures::QUESTION_PROJECT, policy },
                &mut out,
            );
            assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: fixtures::QUESTION_PROJECT }));
            config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
            config.core.defaults.rules = domain.rules().clone();
            let mut engine_limits = engine_limits();
            if let Some(words) = words {
                engine_limits.people.words = words;
                engine_limits.tasks.message_bytes = words;
                engine_limits.journal.result_bytes = engine_limits.journal.result_bytes.max(words);
                engine_limits.journal.transcript_bytes = 1_048_576;
                engine_limits.loads.bytes = 1_048_576;
                engine_limits.loads.reply_bytes = 1_048_576;
                engine_limits.loads.transcript_bytes = 1_048_576;
                engine_limits.loads.read_bytes = 2_097_152;
            }
            if agree {
                // The 306-byte agreed brief plus a quarter of headroom, rounded up.
                engine_limits.tasks.spec_bytes = 384;
                // The chat, its goal and the goal's possible delegate fit the tree.
                engine_limits.tasks.tree_tasks = tasks;
            }
            engine_limits.tasks.inbox_bytes =
                engine_limits.tasks.message_bytes.checked_mul(8).expect("question and answer slots fit");
            if let Some(text) = text {
                engine_limits.documents.text_bytes = text;
            }
            engine_limits.fleet.slots = 2;
            engine_limits.fleet.attempts = 4;
            engine_limits.journal.writes = 4096;
            engine_limits.journal.deliveries = 64;
            engine_limits.journal.held = 192;
            (config, engine_limits)
        });
        Self::with_engine(seed, limits, Store::with_limits(&engine_limits), [bootstrap, config], &engine_limits, 2)
    }
    /// A goal and its delegated authority fit, with bounds chosen by the story.
    pub(crate) fn with_delegates(seed: u64, limits: &client::Limits, delegates: u32, page_rows: u32) -> Self {
        use jig_domain_authority as authority;
        let tasks = delegates.checked_add(1).expect("goal plus its delegates");
        let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let mut rules = config.core.defaults.rules.clone();
            rules.ceiling.delegation.tasks = tasks;
            rules.ceiling.delegation.depth = 2;
            let mut policy = config.core.seed_projects[0].policy.policy.clone();
            policy.ceiling.delegation.tasks = tasks;
            policy.ceiling.delegation.depth = 2;
            policy.roles[0].authority.delegation.tasks = tasks;
            policy.roles[0].authority.delegation.depth = 2;
            let mut authority_limits = temper_engine_domain_world::walking::limits().authority;
            authority_limits.batch = delegates;
            let mut domain = authority::Domain::new(rules, authority_limits).expect("delegate authority");
            let mut out = skein_lib::Queue::uncapped();
            authority::step(
                &mut domain,
                authority::Event::Policy { project: fixtures::QUESTION_PROJECT, policy },
                &mut out,
            );
            assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: fixtures::QUESTION_PROJECT }));
            config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
            config.core.defaults.rules = domain.rules().clone();
            let mut engine_limits = engine_limits();
            engine_limits.documents.rows = page_rows;
            engine_limits.tasks.tasks = tasks;
            engine_limits.tasks.project_tasks = tasks;
            engine_limits.tasks.tree_tasks = tasks;
            engine_limits.tasks.delegates = delegates;
            engine_limits.tasks.batch = delegates;
            engine_limits.authority = authority_limits;
            // The core prices two inbox projections for each live task.
            engine_limits.people.inbox_tasks = tasks * 2;
            engine_limits.tasks.inbox_bytes =
                engine_limits.tasks.message_bytes.checked_mul(8).expect("question and answer slots fit");
            engine_limits.fleet.slots = 2;
            engine_limits.fleet.attempts = 4;
            engine_limits.journal.writes = 4096;
            engine_limits.journal.deliveries = 64;
            engine_limits.journal.held = 192;
            (config, engine_limits)
        });
        Self::with_engine(seed, limits, Store::with_limits(&engine_limits), [bootstrap, config], &engine_limits, 2)
    }
    /// A real forge procedure is admitted by this scenario's policy.
    #[must_use]
    pub(crate) fn with_procedures(seed: u64, limits: &client::Limits) -> Self {
        const SLOTS: u32 = 1;
        // The unknown-panel fixture names its repository and a malformed role choice.
        const PARAMETERS: u32 = 2;
        let [(bootstrap, mut engine_limits), (config, _)] = std::array::from_fn(|_| Self::procedure_config(seed));
        engine_limits.tasks.parameters = PARAMETERS;
        Self::with_engine(seed, limits, Store::with_limits(&engine_limits), [bootstrap, config], &engine_limits, SLOTS)
    }
    fn procedure_config(seed: u64) -> (engine::Config, engine::Limits) {
        use jig_domain_authority as a;
        const PROJECT: u32 = 1;
        const PROCEDURE: u32 = 2;
        const EXECUTORS: u32 = 1;
        let mut config = walking::config(seed);
        config.core.connectors = Box::new([1]);
        for role in &mut config.core.seed_projects[0].policy.permissions {
            role.connector = 1;
        }
        let mut rules = config.core.defaults.rules.clone();
        let mut policy = config.core.seed_projects[0].policy.policy.clone();
        let kinds: Box<[a::Executor]> = Box::new([a::Executor::Procedure(PROCEDURE)]);
        rules.ceiling.delegation.kinds.clone_from(&kinds);
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.roles[0].authority.delegation.kinds = kinds;
        let mut authority_limits = temper_engine_domain_world::walking::limits().authority;
        authority_limits.executors = EXECUTORS;
        let mut domain = a::Domain::new(rules, authority_limits).expect("procedure policy limits");
        let mut out = skein_lib::Queue::uncapped();
        a::step(&mut domain, a::Event::Policy { project: PROJECT, policy }, &mut out);
        assert_eq!(out.pop(), Some(a::PolicyFact::Added { project: PROJECT }));
        config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
        config.core.defaults.rules = domain.rules().clone();
        let mut engine_limits = engine_limits();
        engine_limits.authority = authority_limits;
        (config, engine_limits)
    }
    /// The last attempt assigned to the joint's scripted worker, if any.
    #[must_use]
    pub fn hosted_attempt(&self, task: u64) -> Option<u64> {
        self.hosting.get(&task).copied()
    }

    /// Whether the client's change panel still has its application route.
    #[must_use]
    pub fn watching_change(&self, client: ClientId, task: u64) -> bool {
        use jig_client_kit::joint::transport::Watched;
        use jig_client_test_domain::family::Kind;
        self.transport.watches().into_iter().any(|watcher| {
            let route = self.transport.watch(watcher).expect("live watch");
            route.client == client
                && matches!(route.subject, Watched::Application { subject, .. } if subject.kind == Kind::Panel(task))
                && self.transport.application_watch_token(client, route.native) == Some(watcher)
        })
    }

    /// client/engine.md and client/testing.md: a pushed change lands through CI and an agent gate.
    #[must_use]
    pub fn with_change_landing(seed: u64, limits: &client::Limits) -> Self {
        Self::change_landing(seed, limits, None, false, false)
    }
    /// testing/checks.md, 3: the deployment also waits for the project's owner's review.
    #[must_use]
    pub fn with_change_approval(seed: u64, limits: &client::Limits) -> Self {
        Self::change_landing(seed, limits, None, false, true)
    }
    /// Settings and a following change through the same joined engine.
    #[must_use]
    pub fn with_settings_changes(seed: u64, limits: &client::Limits) -> Self {
        Self::change_landing(seed, limits, None, true, false)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the shared change fixture keeps its policy and settings variant together"
    )]
    fn change_landing(
        seed: u64,
        limits: &client::Limits,
        session: Option<Duration>,
        settings: bool,
        approval: bool,
    ) -> Self {
        // Chat, reviewer, foreman and change procedure; chat/change/producer depth;
        // the same owner in the project and deployment.
        const SETTINGS_EXECUTORS: u32 = 4;
        const SETTINGS_DEPTH: u32 = 3;
        const SETTINGS_OWNERS: u32 = 2;
        const SLOTS: u32 = 1;
        // domain/forge/change/repairs.md: six descendants, plus the change itself.
        const CHANGE_DESCENDANTS: u32 = 6;
        const TREE_TASKS: u32 = CHANGE_DESCENDANTS + 1;
        use jig_domain_authority as a;
        let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
            let (mut config, mut limits) = Self::procedure_config(seed);
            if let Some(session) = session {
                limits.people.sign_in_lifetime = session;
            }
            let connector = config.core.connectors[0];
            // Same agent gate and rule as forge_routes.rs's configured_policy
            // (domain/forge/change/gates.md, 1 and 2), on the client world's repository.
            let mut landing: Box<[engine::LandingRule]> = Box::new([engine::LandingRule {
                connector,
                kind: 4,
                sets: Box::new([]),
                enforce: true,
                pattern: a::Pattern {
                    segments: [b"forge".as_slice(), fixtures::FORGE_HOST, b"acme", b"shop", b"branch"]
                        .map(Box::from)
                        .into(),
                    last: a::Last::Exact(b"main".as_slice().into()),
                },
                ci: true,
                up_to_date: true,
                gates: Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Exact }]),
                approvals: Box::new([]),
            }]);
            let mut projects = skein_lib::Map::with_capacity(1);
            projects.insert(1, landing.clone()).expect("one project landing rule");
            if approval {
                landing[0].approvals =
                    Box::new([engine::Approval { role: 0, people: 1, freshness: engine::Freshness::Clean }]);
            }
            config.kinds_stock = engine::landing_stock(&landing, &projects, 1, 4);
            let mut rules = config.core.defaults.rules.clone();
            let mut policy = config.core.seed_projects[0].policy.policy.clone();
            rules.ceiling.delegation.kinds =
                Box::new([a::Executor::Charter(1), a::Executor::Charter(4), a::Executor::Procedure(2)]);
            // The requesting chat, its change and the review gate each consume a level.
            rules.ceiling.delegation.depth = 3;
            rules.ceiling.delegation.tasks = TREE_TASKS;
            // This pushed-branch path needs read, push, open, land and its goal's issue.
            let grants: Box<[a::Grant]> = [1, 2, 3, 4, 8]
                .map(|kind| a::Grant {
                    connector,
                    kind,
                    pattern: a::Pattern {
                        segments: [b"forge".as_slice(), fixtures::FORGE_HOST, b"acme"].map(Box::from).into(),
                        last: a::Last::Open(b"shop".as_slice().into()),
                    },
                })
                .into();
            // As in configured_policy, goal projections can write their issue and comments.
            let projections = [7, 13].map(|kind| a::Grant { kind, ..grants[0].clone() });
            rules.ceiling.grants = grants.iter().cloned().chain(projections).collect();
            policy.ceiling.grants.clone_from(&rules.ceiling.grants);
            policy.projections.clone_from(&rules.ceiling.grants);
            policy.roles[0].authority.grants = grants;
            policy.ceiling.delegation = rules.ceiling.delegation.clone();
            policy.roles[0].authority.delegation = rules.ceiling.delegation.clone();
            let mut maintainer = policy.roles[0].clone();
            maintainer.number = 1;
            let mut member = maintainer.clone();
            member.number = 2;
            policy.roles = Box::new([policy.roles[0].clone(), maintainer, member]);
            limits.authority.roles = 3;
            limits.authority.requirements = 8;
            limits.authority.facts = 8;
            limits.authority.executors = 3;
            limits.authority.grants = 7;
            limits.authority.segments = 8;
            limits.authority.segment_bytes = 64;
            limits.documents.grants = limits.authority.grants;
            limits.documents.executors = limits.authority.executors;
            // Task offers name all three roles; this joint serves the person's page.
            limits.documents.lines = 3;
            limits.tasks.tasks = 3;
            limits.tasks.project_tasks = 3;
            limits.tasks.tree_tasks = TREE_TASKS;
            limits.tasks.depth = 3;
            limits.tasks.parameters = 5;
            // A gate's brief names its pull, branch, base and hexadecimal head,
            // alongside the task's byte parameters (domain/forge/change/gates.md, 1).
            limits.tasks.spec_bytes = 384;
            limits.tasks.contract_choices = 2;
            limits.tasks.executor_kinds = 3;
            limits.tasks.charters = 4;
            Self::landing_reviewer(&mut config);
            limits.tasks.authority_grants = 5;
            limits.tasks.authority_segments = 9;
            limits.tasks.authority_bytes = 512;
            limits.tasks.inbox_bytes = 1024;
            limits.people.inbox_tasks = 6;
            // configured_policy uses jig's Small profile's creation-request bound.
            limits.people.amendment_bytes = 8_192;
            // The core prices deployment, project, task and every grant's note scope.
            limits.notes.scopes = 3 + limits.authority.grants;
            limits.journal.writes = 8192;
            limits.journal.held = 1024;
            limits.journal.deliveries = 256;
            limits.journal.result_bytes = 4096;
            // The deep task row includes its optional session and talk, 32 bytes each.
            limits.journal.transcript_bytes = 8256;
            limits.journal.run_bytes = 8192;
            limits.brief.sections = 5;
            limits.brief.brief_bytes = 1024;
            limits.brief.budgets.task = 512;
            limits.brief.read_bytes = 512;
            limits.brief.budgets.pull = 384;
            limits.brief.budgets.ci = 384;
            if settings {
                config.core.kinds = temper_service::stock::kinds(&limits, connector);
                let owner = config.core.owners[0].clone();
                config.core.owners =
                    Box::new([owner.clone(), jig_domain_people::InitialOwner { project: 0, identity: owner.identity }]);
                limits.people.initial_owners = SETTINGS_OWNERS;
                let mut foreman = config.core.defaults.charters[0].clone();
                foreman.number = 6;
                let mut charters = config.core.defaults.charters.into_vec();
                charters.push(foreman);
                config.core.defaults.charters = charters.into_boxed_slice();
                rules.ceiling.delegation.kinds = Box::new([
                    a::Executor::Charter(1),
                    a::Executor::Charter(4),
                    a::Executor::Charter(6),
                    a::Executor::Procedure(2),
                ]);
                policy.ceiling.delegation.kinds.clone_from(&rules.ceiling.delegation.kinds);
                policy.roles[0].authority.delegation.kinds.clone_from(&rules.ceiling.delegation.kinds);
                rules.ceiling.delegation.depth = SETTINGS_DEPTH;
                policy.ceiling.delegation.depth = SETTINGS_DEPTH;
                policy.roles[0].authority.delegation.depth = SETTINGS_DEPTH;
                limits.tasks.depth = SETTINGS_DEPTH;
                limits.authority.executors = SETTINGS_EXECUTORS;
                limits.tasks.executor_kinds = SETTINGS_EXECUTORS;
                limits.documents.executors = SETTINGS_EXECUTORS;
                // The task's closer, optional session and talk each occupy 32 bytes in the deep row.
                limits.journal.transcript_bytes = 8264;
            }
            let mut domain = a::Domain::new(rules, limits.authority).expect("landing authority");
            let mut out = skein_lib::Queue::uncapped();
            a::step(&mut domain, a::Event::Policy { project: 1, policy }, &mut out);
            assert_eq!(out.pop(), Some(a::PolicyFact::Added { project: 1 }));
            config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
            config.core.defaults.rules = domain.rules().clone();
            (config, limits)
        });
        if settings {
            let mut driver = Driver::cold(Store::with_limits(&engine_limits), bootstrap, &engine_limits);
            driver.settle();
            let mut store = driver.take_store();
            store.seed(
                temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers),
                Box::new([]),
            );
            let restart = temper_engine_domain_world::oracle::config::copy(&config);
            Self::with_engine(seed, limits, store, [config, restart], &engine_limits, SLOTS)
        } else {
            Self::with_engine(
                seed,
                limits,
                Store::with_limits(&engine_limits),
                [bootstrap, config],
                &engine_limits,
                SLOTS,
            )
        }
    }
    fn landing_reviewer(config: &mut engine::Config) {
        let mut reviewer = config.core.defaults.charters[0].clone();
        reviewer.number = 4;
        reviewer.policy.modify = false;
        reviewer.policy.agents = false;
        config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), reviewer]);
    }
    /// Join a story's engine configuration to the client protocol and its observations.
    #[must_use]
    pub fn with_engine(
        seed: u64,
        limits: &client::Limits,
        store: Store,
        configs: [engine::Config; 2],
        engine_limits: &engine::Limits,
        slots: u32,
    ) -> Self {
        Self::with_engine_with_refused_hello(seed, limits, store, configs, engine_limits, slots, false)
    }
    fn worker_hello(
        config: &engine::Config,
        engine_limits: &engine::Limits,
        slots: u32,
        refuse_hello: bool,
    ) -> engine::Event {
        let worker_models = config
            .core
            .settings
            .models
            .iter()
            .map(|model| EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output })
            .collect();
        let mut hello =
            temper_engine_domain_world::protocol::hello_slots(fixtures::QUESTION_WORKER, slots, Box::new([]));
        let engine::Event::Worker(engine::WorkerInput::Link {
            link: engine::Link::Hello { worker, hello: value }, ..
        }) = &mut hello
        else {
            unreachable!("worker hello")
        };
        value.bounds.endpoints = worker_models;
        value.bounds.message_bytes = engine_limits.tasks.message_bytes;
        if refuse_hello {
            // The fixture configures only worker zero.
            const UNCONFIGURED_WORKER: u32 = 7;
            *worker = UNCONFIGURED_WORKER;
        }
        hello
    }
    fn with_engine_with_refused_hello(
        seed: u64,
        limits: &client::Limits,
        store: Store,
        configs: [engine::Config; 2],
        engine_limits: &engine::Limits,
        slots: u32,
        refuse_hello: bool,
    ) -> Self {
        let (transport, component) = Self::client_parts(&configs[1], limits);
        let hello = Self::worker_hello(&configs[1], engine_limits, slots, refuse_hello);
        // protocol/configuration/limits.md: documents fit the client this world serves.
        // Smaller text projections still retain the full task spec and brief.
        let mut engine_limits = *engine_limits;
        engine_limits.documents.text_bytes = engine_limits.documents.text_bytes.min(limits.client.domain.text);
        let [bootstrap, config] = configs;
        let store = restored_forges(store, bootstrap, &config, &engine_limits);
        let mut engine = Driver::cold(store, config, &engine_limits);
        engine.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(fixtures::REPOSITORIES));
        engine.settle();
        engine.send(hello);
        engine.settle();
        Self::with_driver(seed, limits, engine, transport, component, refuse_hello)
    }
    fn client_parts(config: &engine::Config, limits: &client::Limits) -> (Transport, crate::component::Composition) {
        let mut transport = Transport::new(jig_client::contract::VersionRange {
            first: jig_client_protocol_wire::VERSION,
            last: jig_client_protocol_wire::VERSION,
        });
        transport.models.clone_from(&config.core.settings.models);
        transport.providers = config.core.settings.llm_providers.clone();
        (transport, crate::component::Composition::new(config, limits.client.domain))
    }
    fn with_driver(
        seed: u64,
        limits: &client::Limits,
        mut engine: Driver,
        transport: Transport,
        component: crate::component::Composition,
        refuse_hello: bool,
    ) -> Self {
        let last_commit = engine.store.header().commits;
        let truth = jig_client_kit::joint::truth::Truth::after(core_rows(&engine));
        engine.delivered.clear();
        engine.immediate.clear();
        Joint {
            engine,
            sign_ins: 0,
            dismissal_sends: 0,
            status_looks: BTreeMap::new(),
            caller: fixtures::PARTY,
            sign_in_identity: None,
            leaves: Vec::new(),
            tabs: Vec::new(),
            shows: Vec::new(),
            set_ups: Vec::new(),
            admin_connects: Vec::new(),
            replacements: Vec::new(),
            playground: None,
            deployment: (true, true),
            program_environment: Box::new([]),
            hold_playground_end: false,
            held_playground_end: Vec::new(),
            playground_ends: Vec::new(),
            playground_starts: Vec::new(),
            playground_removals: Vec::new(),
            playground_reads: 0,
            note_reads: 0,
            note_read_ends: VecDeque::new(),
            task_read_ends: VecDeque::new(),
            activity_read_ends: VecDeque::new(),
            reachable_reads: Vec::new(),
            reachable_empty: false,
            reachable_pages: false,
            installs: Vec::new(),
            playground_outcomes: VecDeque::new(),
            answer_runs: false,
            reply_runs: false,
            runs_answered: 0,
            me_deliveries: 0,
            assignments: Vec::new(),
            trial_config: None,
            reply_script: None,
            held_reply: None,
            hold_playground_read: false,
            held_playground_reads: VecDeque::new(),
            park_runs: false,
            settles_with_a_deadline_armed: refuse_hello,
            keyed_fault: None,
            reads: 0,
            read_loads: BTreeSet::new(),
            transport,
            component,
            taking: BTreeMap::new(),
            limits: *limits,
            client_limits: BTreeMap::new(),
            next_limits: None,
            callbacks: Vec::new(),
            parked: BTreeMap::new(),
            offline_until: None,
            linked_heard: BTreeMap::new(),
            due: None,
            observed: vec![Observed::Reachable { live: true }],
            lose_answers: 0,
            lost_answer: None,
            hosting: BTreeMap::new(),
            seed,
            now: Time::ZERO,
            last_commit,
            truth,
            sign_outs: BTreeMap::new(),
        }
    }
    /// Supply one typed provider answer and let the normal watch path deliver it.
    pub fn answer_read(&mut self, outcome: jig_domain_link::ListOutcome) {
        let event = self.engine.protocol.answer_read(outcome);
        self.send(event);
        self.drain();
    }
    /// Send a family's ask through the same component as a page, retaining its answer for the story.
    /// The synchronous caller owns the reserved attempt, outside the page's request table.
    pub fn family_request(
        &mut self,
        who: ClientId,
        key: skein_lib::RequestKey,
        request: jig_client::Ask,
    ) -> jig_client::Family {
        let attempt = Token::new(u64::MAX);
        self.request(
            who,
            Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, key, request, value: None },
                },
            },
        );
        self.drain();
        let position = self
            .callbacks
            .iter()
            .position(|(client, event)| {
                *client == who
                    && matches!(event, Event::Root(RootEvent::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::Answer { attempt: answered, .. } },
                }) if *answered == attempt)
            })
            .expect("family request answered");
        let (_, event) = self.callbacks.remove(position);
        let Event::Root(RootEvent::Client {
            event:
                jig_client::Event::Up {
                    up:
                        jig_client::Up::Answer {
                            envelope: RequestEnvelope::Final,
                            answer: Some(jig_client::Answered::Family(record)),
                            ..
                        },
                },
        }) = event
        else {
            panic!("family request: {event:?}")
        };
        record
    }
    pub fn next_client_limits(&mut self, limits: client::Limits) {
        self.next_limits = Some(limits);
    }
    fn hello(&mut self, who: ClientId) {
        self.callbacks.push((
            who,
            translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })
            .into(),
        ));
        self.callbacks.push((
            who,
            RootEvent::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyServed {
                        family: fixtures::FORGE,
                        versions: Some(jig_client::contract::VersionRange {
                            first: temper_client_protocol_wire::CLIENT_VERSION,
                            last: temper_client_protocol_wire::CLIENT_VERSION,
                        }),
                    },
                },
            }
            .into(),
        ));
    }
    fn advance(&mut self, now: Time) {
        self.now = now;
        self.engine.env.now = now;
        self.engine.env.wall = Wall::from_nanos(now.as_nanos());
        self.component.clock(now, self.engine.env.wall);
        if let Some(deadline) = self.engine.root.next_deadline(&self.engine.env)
            && deadline <= now
        {
            engine::fire(&mut self.engine.root, &self.engine.env);
        }
    }
    fn send(&mut self, mut event: engine::Event) {
        // The deployment front serves a window within the engine's configured page bound.
        if let engine::Event::Party(engine::PartyInput::Watch { window, .. }) = &mut event {
            window.rows = window.rows.min(self.engine.env.limits.documents.rows);
        }
        if matches!(event, engine::Event::Party(engine::PartyInput::SignedIn { .. })) {
            self.sign_ins += 1;
        }
        self.engine.send(event);
    }
    fn reachable(&mut self, who: ClientId, request: &Request) -> bool {
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
        else {
            return false;
        };
        if record.family != wire::FORGE {
            return false;
        }
        let Some(request) = wire::decode_requesting(&record.bytes, &w::CEILINGS) else { return false };
        let w::Request::Reachable(read) = request.request() else { return false };
        self.reachable_reads.push((read.forge(), *read.installation(), read.page()));
        let mut rows = skein_lib::List::with_capacity(if self.reachable_pages { 10 } else { 2 });
        if self.reachable_pages && read.page() == 1 {
            for id in 100..108 {
                rows.push(
                    w::Reached::new(
                        &w::CEILINGS,
                        w::ReachedParts {
                            id,
                            owner: w::Account::new(
                                &w::CEILINGS,
                                w::AccountParts { id: 1, login: b"noise".as_slice().into() },
                            )
                            .expect("owner"),
                            name: skein_lib::Decimal::of(id).as_bytes().into(),
                            private: false,
                        },
                    )
                    .expect("row"),
                )
                .expect("page");
            }
        }
        for (page, id, name) in [(1, 7, b"shop".as_slice()), (2, 8, b"second".as_slice())] {
            if self.reachable_pages && read.page() != page {
                continue;
            }
            if self.reachable_empty {
                break;
            }
            rows.push(
                w::Reached::new(
                    &w::CEILINGS,
                    w::ReachedParts {
                        id,
                        owner: w::Account::new(
                            &w::CEILINGS,
                            w::AccountParts { id: 1, login: b"acme".as_slice().into() },
                        )
                        .expect("fixed owner"),
                        name: name.into(),
                        private: id == 8,
                    },
                )
                .expect("fixed row"),
            )
            .expect("fixed page");
        }
        let outcome = w::Outcome::Page(
            w::Page::new(&w::CEILINGS, w::PageParts { rows, more: self.reachable_pages && read.page() < 3 })
                .expect("fixed page"),
        );
        self.callbacks.push((
            who,
            RootEvent::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt: *attempt,
                        envelope: RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Family(jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::answer(
                                &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).expect("fixed answer"),
                            ),
                        })),
                    },
                },
            }
            .into(),
        ));
        true
    }
    fn start_playground(&mut self, who: ClientId, request: &Request) -> bool {
        // The front scripts the playground lifecycle beside the real engine.
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, key, request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
        {
            use temper_client_protocol_wire::{self as wire, v1 as w};
            let decoded = wire::decode_requesting(&record.bytes, &w::CEILINGS);
            let ending = decoded.as_ref().is_some_and(|request| *request.request() == w::Request::EndPlayground);
            if record.family == wire::FORGE
                && decoded.is_some_and(|request| {
                    matches!(request.request(), w::Request::StartPlayground | w::Request::EndPlayground)
                })
            {
                let outcome = if ending {
                    self.playground_ends.push(*key);
                    if !self.hold_playground_end {
                        self.playground = None;
                    }
                    w::Outcome::Reset
                } else {
                    self.playground_starts.push(*key);
                    self.playground_outcomes.pop_front().unwrap_or(w::Outcome::Running)
                };
                if outcome == w::Outcome::Running {
                    let mut parts =
                        self.playground.take().unwrap_or_else(|| account_acts::playground_answer(true)).into_parts();
                    parts.state = w::PlaygroundState::Running;
                    self.playground = Some(w::PlaygroundAnswer::new(&w::CEILINGS, parts).expect("playground fixture"));
                }
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt: *attempt,
                                envelope: RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Family(jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::answer(
                                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome })
                                            .expect("scripted start answer"),
                                    ),
                                })),
                            },
                        },
                    }
                    .into(),
                ));
                if ending && self.hold_playground_end {
                    self.held_playground_end.push(self.callbacks.pop().expect("queued reset answer"));
                }
                self.publish_playground_forges();
                return true;
            }
        }
        false
    }
    fn note_read_end(&mut self, who: ClientId, request: &Request) -> bool {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what: jig_client::Readable::Jig(jig_client::contract::Read::Notes { .. }),
                        },
                },
        } = request
        {
            self.note_reads += 1;
            if let Some(why) = self.note_read_ends.pop_front() {
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read: *read,
                                result: jig_client::Readout::Jig(jig_client::contract::ReadResult::None(why)),
                            },
                        },
                    }
                    .into(),
                ));
                return true;
            }
        }
        false
    }
    fn task_read_end(&mut self, who: ClientId, request: &Request) -> bool {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what: jig_client::Readable::Jig(jig_client::contract::Read::Task { .. }),
                        },
                },
        } = request
            && let Some(why) = self.task_read_ends.pop_front()
        {
            self.callbacks.push((
                who,
                RootEvent::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(jig_client::contract::ReadResult::None(why)),
                        },
                    },
                }
                .into(),
            ));
            return true;
        }
        false
    }
    fn activity_read_end(&mut self, who: ClientId, request: &Request) -> bool {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what:
                                jig_client::Readable::Jig(
                                    jig_client::contract::Read::Activity { .. }
                                    | jig_client::contract::Read::ActivityBeside { .. },
                                ),
                        },
                },
        } = request
            && let Some(why) = self.activity_read_ends.pop_front()
        {
            self.callbacks.push((
                who,
                RootEvent::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(jig_client::contract::ReadResult::None(why)),
                        },
                    },
                }
                .into(),
            ));
            return true;
        }
        false
    }
    fn playground_read(&mut self, who: ClientId, request: &Request) -> bool {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
        {
            if temper_client_protocol_wire::decode_reading(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
                .is_some_and(|reading| *reading.read() == temper_client_protocol_wire::v1::Read::Deployment)
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                let (welcome, owned) = self.deployment;
                let answer = w::DeploymentAnswer::new(
                    &w::CEILINGS,
                    w::DeploymentAnswerParts { welcome, owned, program_login: None },
                )
                .expect("deployment answer");
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read: *read,
                                result: jig_client::Readout::Family {
                                    family: wire::FORGE,
                                    record: Some(wire::deployment_answer(&answer)),
                                },
                            },
                        },
                    }
                    .into(),
                ));
                return true;
            }
            if temper_client_protocol_wire::decode_reading(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
                .is_some_and(|reading| *reading.read() == temper_client_protocol_wire::v1::Read::ProgramEnvironment)
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                let answer = w::ProgramEnvironmentAnswer::new(
                    &w::CEILINGS,
                    w::ProgramEnvironmentAnswerParts { environment: self.program_environment.clone() },
                )
                .expect("program environment");
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read: *read,
                                result: jig_client::Readout::Family {
                                    family: wire::FORGE,
                                    record: Some(wire::program_environment_answer(&answer)),
                                },
                            },
                        },
                    }
                    .into(),
                ));
                return true;
            }
            if !temper_client_protocol_wire::decode_reading(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
                .is_some_and(|reading| *reading.read() == temper_client_protocol_wire::v1::Read::Playground)
            {
                return false;
            }
            // The playground is scripted independently of the engine.
            self.playground_reads += 1;
            self.callbacks.push((
                who,
                RootEvent::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Family {
                                family: record.family,
                                record: self.playground.as_ref().map(temper_client_protocol_wire::playground_answer),
                            },
                        },
                    },
                }
                .into(),
            ));
            if self.hold_playground_read {
                self.hold_playground_read = false;
                self.held_playground_reads.push_back(self.callbacks.pop().expect("just queued playground answer"));
            }
            return true;
        }
        false
    }
    fn request(&mut self, who: ClientId, request: Request) {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::ApprovalAct(act) } = request {
            self.component.approval(who, act);
            self.pump_component();
            return;
        }
        if self.offline_until.is_some() || !self.transport.is_bound(who) || self.transport.sign_in(who).is_none() {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, value: Some(_), .. } },
            } = &request
            {
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt: *attempt,
                                envelope: RequestEnvelope::Again,
                                answer: None,
                                why: None,
                            },
                        },
                    }
                    .into(),
                ));
                return;
            }
            self.parked.entry(who).or_default().push(request);
            return;
        }
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = &request
            && let Some(envelope) = self.keyed_fault.take()
        {
            if envelope == RequestEnvelope::SignedOut {
                self.transport.signed_out(who);
                self.sign_outs.entry(who).or_default().push_back(None);
                self.callbacks.push((
                    who,
                    RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }.into(),
                ));
            }
            self.callbacks.push((
                who,
                RootEvent::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer { attempt: *attempt, envelope, answer: None, why: None },
                    },
                }
                .into(),
            ));
            return;
        }
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { key, request: jig_client::Ask::Jig(ask), .. },
                },
        } = &request
            && matches!(
                ask.as_ref(),
                jig_client::contract::Ask::AccountChange {
                    holder: jig_client::contract::AskAccountHolder::Deployment,
                    change: jig_client::contract::AccountChange::Remove { .. },
                }
            )
        {
            self.playground_removals.push(*key);
        }
        if self.reachable(who, &request) || self.start_playground(who, &request) {
            return;
        }
        if self.playground_read(who, &request)
            || self.note_read_end(who, &request)
            || self.task_read_end(who, &request)
            || self.activity_read_end(who, &request)
        {
            return;
        }
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else {
            panic!("unlinked down request")
        };
        self.component.send(who, down);
        self.pump_component();
    }
    fn pump_component(&mut self) {
        self.component.playground.clone_from(&self.playground);
        loop {
            let packets = self.component.pump(&mut self.transport);
            let events = self.component.effects();
            let linked_changed = self.component.take_linked_changed();
            if linked_changed {
                self.engine.root.mark_forges();
            }
            let active = linked_changed || !events.is_empty();
            for event in events {
                self.send(event);
            }
            if active {
                self.settle();
                let refused: Vec<_> = self
                    .engine
                    .immediate
                    .iter()
                    .filter_map(|request| {
                        if let engine::Request::Party(engine::PartyRequest::WatchRefused { watcher, .. }) = request {
                            Some(*watcher)
                        } else {
                            None
                        }
                    })
                    .collect();
                self.component.admitted(&refused);
            }
            for packet in packets {
                self.incoming(packet);
            }
            if !active {
                break;
            }
        }
    }
    fn publish_playground_forges(&mut self) {
        self.engine.root.mark_forges();
        self.drain();
    }
    fn playground_forges(&self, event: &mut RootEvent) {
        let RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { record, .. } } } =
            event
        else {
            return;
        };
        Self::with_playground_forges(&mut record.bytes, self.playground.as_ref());
    }
    pub(crate) fn with_playground_forges(
        bytes: &mut Box<[u8]>,
        playground: Option<&temper_client_protocol_wire::v1::PlaygroundAnswer>,
    ) {
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let Some(playground) = playground else { return };
        let Some(rows) = wire::decode_forges(bytes, &w::CEILINGS) else { return };
        let rows = rows.into_parts();
        let linked = rows.linked;
        let mut forges = skein_lib::List::with_capacity(wire::FORGES);
        for row in rows.forges.into_boxed() {
            // A change story can use the engine's fake stand as its playground forge.
            if row.number() != playground.forge() {
                forges.push(row).expect("scripted deployment row");
            }
        }
        let connection = if *playground.state() == w::PlaygroundState::Running {
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 2, login: b"playground".as_slice().into() })
                    .expect("fake connection"),
            )
        } else {
            w::Connection::NotConnected
        };
        forges
            .push(
                w::ForgeRow::new(
                    &w::CEILINGS,
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        makings: skein_lib::List::with_capacity(0),
                        host: b"git.example:3000".as_slice().into(),
                        number: playground.forge(),
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Fake,
                        connection,
                        offer: w::Offer::NotOffered,
                        first: None,
                        installations: skein_lib::List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: skein_lib::List::with_capacity(0),
                        owned: None,
                        admin: false,
                        read_only: false,
                        ended_stop: None,
                    },
                )
                .expect("scripted playground forge"),
            )
            .expect("playground row fits");
        *bytes =
            wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges, linked }).expect("scripted forges"));
    }
    // Runtime records have no host-supplied version or password until the setup host reports them.
    fn host_forges(event: &mut RootEvent) {
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { record, .. } } } =
            event
        else {
            return;
        };
        let Some(rows) = wire::decode_forges(&record.bytes, &w::CEILINGS) else { return };
        let rows = rows.into_parts();
        let linked = rows.linked;
        let mut forges = skein_lib::List::with_capacity(wire::FORGES);
        for row in rows.forges.into_boxed() {
            let mut parts = row.into_parts();
            if parts.owned.as_ref().is_some_and(|owned| {
                owned.forgejo().is_empty() && owned.runner().is_none() && owned.password().is_empty()
            }) {
                parts.owned = None;
            }
            forges.push(w::ForgeRow::new(&w::CEILINGS, parts).expect("host forge row")).expect("bounded forge rows");
        }
        record.bytes =
            wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges, linked }).expect("host forges"));
    }
    fn callback(&mut self, client: ClientId, mut event: RootEvent) {
        Self::host_forges(&mut event);
        self.playground_forges(&mut event);
        if let RootEvent::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Answer { attempt, envelope, answer, why } },
        } = &mut event
            && self.lose_answers != 0
        {
            self.lost_answer = Some((client, *attempt));
            self.lose_answers -= 1;
            *envelope = RequestEnvelope::Lost;
            *answer = None;
            *why = None;
        }
        self.callbacks.push((client, event.into()));
    }
    fn incoming(&mut self, packet: crate::component::Incoming) {
        match packet {
            crate::component::Incoming::Taking { client, taking } => {
                self.taking.insert(client, taking);
            }
            crate::component::Incoming::Authenticated { client, person, .. } => {
                if self.component.bind_authenticated(client, person, &mut self.transport) {
                    let limits = *self.client_limits.entry(client).or_insert(self.limits);
                    self.component.receiving(client, limits.client.domain);
                    self.hello(client);
                    self.callbacks.push((
                        client,
                        RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }.into(),
                    ));
                }
                for request in self.parked.remove(&client).unwrap_or_default() {
                    self.request(client, request);
                }
            }
            crate::component::Incoming::Callback { client, event } => {
                self.callback(client, translate::callback(event));
            }
            crate::component::Incoming::SignedOut { client, sign_in } => {
                self.observed.push(Observed::SignInEnded { client });
                self.sign_outs.entry(client).or_default().push_back(Some(sign_in));
                self.callbacks.push((
                    client,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }),
                ));
            }
            crate::component::Incoming::SignInAgain { why, .. } => {
                panic!("the deployment owner signs in again: {why:?}")
            }
            crate::component::Incoming::SignInFailed { failure, .. } => {
                panic!("the deployment owner signs in: {failure:?}")
            }
        }
    }
    fn packet(&mut self, request: engine::Request) {
        self.component.playground.clone_from(&self.playground);
        self.component.incoming(&mut self.transport, request);
        self.pump_component();
    }
    fn settle(&mut self) {
        for step in 0..200 {
            if self.trial_config.is_some() {
                // Trial stories supply the worker's status explicitly; an
                // empty inventory from the direct driver's default is held.
                self.engine.events.retain(|event| {
                    !matches!(event,
                    engine::Event::Worker(engine::WorkerInput::Link {
                        link: engine::Link::Up { up: jig_domain_host::LinkUp::Logins { logins }, .. }, ..
                    }) if logins.is_empty())
                });
            }
            for event in &mut self.engine.events {
                if let engine::Event::Store(engine::StoreInput::ReadLoaded { load, result }) = event
                    && self.read_loads.insert(*load)
                    && let Some(range) = self.engine.protocol.reads.get(self.reads)
                {
                    *result = fixtures::read(&self.engine.store, range);
                    self.reads += 1;
                }
            }
            self.engine.advance(true);
            // The refused hello keeps the fleet's placement deadline armed.
            if (self.engine.root.quiescent() || (self.settles_with_a_deadline_armed && step == 199))
                && self.engine.events.is_empty()
                && self.engine.store.pending.is_empty()
                && self.engine.forge.as_ref().is_none_or(temper_engine_domain_world::forge_stand::ForgeStand::quiescent)
            {
                return;
            }
        }
        panic!(
            "joint engine did not settle: quiescent={}; returned={}; events={:?}; pending={:?}",
            self.engine.root.quiescent(),
            self.engine.returned,
            self.engine.events,
            self.engine.store.pending
        );
    }
    fn assigned_reply(&mut self, assignment: &engine::Assignment) {
        self.assignments.push(assignment.clone());
        let read = assignment.messages.last().map(|word| word.number);
        match self.reply_script.take() {
            Some(ReplyScript::AtOnce(words)) => {
                self.finish_reply(assignment.task, assignment.attempt, read, Some(words));
            }
            Some(ReplyScript::Pieces(pieces)) => {
                self.held_reply = Some(HeldReply {
                    task: assignment.task,
                    attempt: assignment.attempt,
                    read,
                    pieces,
                    next: 0,
                    words: Vec::new(),
                });
                assert!(self.release_reply(), "the assignment holds its first piece");
            }
            None => {
                if self.answer_runs {
                    let words = self.reply_runs.then(|| format!("Answered, run {}", self.runs_answered + 1));
                    self.finish_reply(assignment.task, assignment.attempt, read, words.as_deref().map(str::as_bytes));
                } else {
                    self.hosting.insert(assignment.task, assignment.attempt);
                }
            }
        }
    }
    fn drain(&mut self) {
        self.settle();
        for models in std::mem::take(&mut self.engine.protocol.catalogues) {
            self.packet(engine::Request::Party(engine::PartyRequest::Catalogue { models }));
        }
        let commit = self.engine.store.header().commits;
        if commit != self.last_commit {
            self.observed.extend(self.truth.applied(jig_core_kit::store_peer::DurableSnapshot {
                commit,
                at: self.now,
                rows: core_rows(&self.engine),
            }));
            self.last_commit = commit;
        }
        let deliveries = std::mem::take(&mut self.engine.delivered);
        for delivery in deliveries {
            if let engine::Delivery::Link { channel, down: jig_domain_host::LinkDown::Cancel { run, attempt } } =
                &delivery
            {
                self.hosting.remove(&run.raw());
                self.engine.answer(
                    *channel,
                    run.raw(),
                    attempt.raw(),
                    0,
                    jig_domain_tasks::End::Parked,
                    engine::Parts::none(),
                );
            }
            if self.park_runs
                && let engine::Delivery::Assigned { assignment, .. } = &delivery
            {
                if let Some(word) = assignment.messages.last() {
                    self.engine.send(temper_engine_domain_world::protocol::turn(
                        Token::new(7),
                        assignment.task,
                        assignment.attempt,
                        1,
                        0,
                        Some(word.number),
                        b"read messages",
                    ));
                    self.engine.settle();
                }
                self.engine.answer(
                    Token::new(7),
                    assignment.task,
                    assignment.attempt,
                    0,
                    jig_domain_tasks::End::Parked,
                    engine::Parts::none(),
                );
            }
            if let engine::Delivery::Assigned { assignment, .. } = &delivery {
                self.assigned_reply(assignment);
            }
            self.packet(engine::Request::Party(engine::PartyRequest::Deliver(delivery)));
        }
        self.immediate();
    }
    fn immediate(&mut self) {
        let immediate = std::mem::take(&mut self.engine.immediate);
        for request in immediate {
            if let engine::Request::Manifest { forge, request, .. } = &request {
                self.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
                    forge: *forge,
                    state: temper_engine_domain_forge::Connection::NotConnected,
                })));
                self.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::ConnectAnswered {
                    forge: *forge,
                    request: *request,
                    answer: temper_engine_domain_forge::ConnectAnswer::Link(
                        b"https://github.test/settings/apps/new".as_slice().into(),
                    ),
                })));
            }
            if let engine::Request::Install { forge, request, .. } = &request {
                self.installs.push((*forge, *request));
            }
            if let engine::Request::Connect { forge, request: attempt, carried, how } = &request {
                let codes: &[u16] = match how {
                    temper_engine_domain_forge::How::ForgejoAdmin { .. } => &[1],
                    temper_engine_domain_forge::How::ForgejoTokens => &[2, 3],
                    temper_engine_domain_forge::How::GitHubApp { .. }
                    | temper_engine_domain_forge::How::GitHubManifest { .. } => &[],
                };
                if !codes.is_empty() {
                    let value = self.component.carried.get(&carried.number).expect("connect carrier");
                    let items = temper_client_domain::forge::forges::decode_pasted(value.expose())
                        .expect("pasted connect bundle");
                    for code in codes {
                        assert!(items.iter().any(|(kind, _)| kind == code));
                    }
                    for (code, _) in &items {
                        assert!(codes.contains(code) || (*code == 5 && codes == [2, 3]));
                    }
                    self.admin_connects.push((*forge, *attempt));
                    self.engine.send(engine::Event::Forge(Box::new(
                        temper_engine_domain_forge::Event::ConnectAnswered {
                            forge: *forge,
                            request: *attempt,
                            answer: temper_engine_domain_forge::ConnectAnswer::Connecting,
                        },
                    )));
                }
            }
            if let engine::Request::Replace { forge, request: attempt, carried } = &request {
                let value = self.component.carried.get(&carried.number).expect("replace carrier");
                let items =
                    temper_client_domain::forge::forges::decode_pasted(value.expose()).expect("replacement bundle");
                assert!(!items.is_empty());
                if items.iter().any(|(code, _)| *code == 11) {
                    assert_eq!(items.len(), 1);
                } else {
                    assert!(items.iter().any(|(code, _)| *code == 2 || *code == 3));
                    for (code, _) in &items {
                        assert!([2, 3, 5].contains(code));
                    }
                }
                self.replacements.push((*forge, *attempt));
                self.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::ConnectAnswered {
                    forge: *forge,
                    request: *attempt,
                    answer: temper_engine_domain_forge::ConnectAnswer::Connecting,
                })));
            }
            if let engine::Request::EraseCarried { number } = &request {
                self.component.carried.remove(number);
            }

            if let engine::Request::SetUp { forge, request, .. } = &request {
                self.set_ups.push((*forge, *request));
            }
            self.packet(request);
        }
    }
    fn detach(&mut self, who: ClientId) {
        self.sign_outs.remove(&who);
        self.callbacks.retain(|(client, _)| *client != who);
        self.parked.remove(&who);
        self.component.detach(who);
        self.transport.detach(who);
        self.pump_component();
    }
    fn drop_watches(&mut self) {
        self.component.drop_watches();
        self.pump_component();
    }
}
impl Drop for Joint {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            // End the clients' volatile watches before the engine's restart oracle opens its own.
            // Its immediate-work settle accepts live fleet deadlines left by the story.
            self.component.drop_watches();
            drop(self.component.pump(&mut self.transport));
            for event in self.component.effects() {
                self.engine.send(event);
            }
        }
    }
}

impl Front<Temper> for Joint {
    fn bind(&mut self, who: ClientId, party: u64) {
        self.client_limits.insert(who, self.next_limits.take().unwrap_or(self.limits));
        self.component.receiving(who, self.client_limits[&who].client.domain);
        // A story can establish the person's membership before opening its Home.
        if self.transport.clients().contains(&who) {
            assert_eq!(self.transport.party(who), party, "the fixture-bound person");
            return;
        }
        assert_eq!(party, self.caller, "the configured caller");
        self.transport.bind(
            who,
            party,
            self.sign_in_identity.clone().unwrap_or_else(|| jig_domain_people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            }),
        );
        self.callbacks.push((
            who,
            Event::Root(translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })),
        ));
        self.component.sign_in(
            who,
            self.sign_in_identity.clone().unwrap_or_else(|| jig_domain_people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            }),
        );
        self.pump_component();
    }
    fn wire(&mut self, who: ClientId, now: Time, _wall: Wall, request: Request) {
        self.advance(now);
        self.count_dismissal(&request);
        self.count_status_look(who, &request);
        self.request(who, request);
    }
    fn poll(&mut self, now: Time, _wall: Wall) -> Vec<(ClientId, Event)> {
        self.advance(now);
        if let Some(until) = self.offline_until {
            if now < until {
                return std::mem::take(&mut self.callbacks);
            }
            self.offline_until = None;
            self.observed.push(Observed::Reachable { live: true });
            for who in self.transport.clients() {
                self.hello(who);
                if self.transport.sign_in(who).is_none() {
                    self.sign_outs.entry(who).or_default().push_back(None);
                    self.callbacks.push((
                        who,
                        Event::Root(RootEvent::Client {
                            event: jig_client::Event::Up { up: jig_client::Up::SignedOut },
                        }),
                    ));
                }
            }
            for (who, requests) in std::mem::take(&mut self.parked) {
                for request in requests {
                    self.request(who, request);
                }
            }
        }
        self.drain();
        if self.due.is_some_and(|due| now < due) {
            return Vec::new();
        }
        self.due = None;
        std::mem::take(&mut self.callbacks)
    }
    fn next_due(&self) -> Option<Time> {
        self.offline_until.or(self.due).or_else(|| {
            // Engine replies left by the component are ready work for the next poll.
            if self.engine.delivered.is_empty() && self.engine.immediate.is_empty() {
                Some(self.now.saturating_add(Duration::from_millis(100)))
            } else {
                Some(self.now)
            }
        })
    }
    fn observed(&mut self) -> Vec<Observed> {
        std::mem::take(&mut self.observed)
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        self.advance(now);
        match fault {
            FrontFault::Restart { offline } => {
                let limits = self.engine.env.limits;
                let hosting = self
                    .hosting
                    .iter()
                    .map(|(task, attempt)| {
                        temper_engine_domain_world::protocol::hosting(*task, *attempt, Phase::Active)
                    })
                    .collect();
                let store = self.engine.take_store();
                let config = walking::config(self.seed);
                // The restarted engine runs the walking settings: judge asks by the same ones, as jig's kit does.
                self.transport.models.clone_from(&config.core.settings.models);
                self.transport.providers = config.core.settings.llm_providers.clone();
                self.engine = Driver::with_repositories(
                    store,
                    [walking::config(self.seed), config],
                    &limits,
                    fixtures::REPOSITORIES,
                );
                self.engine.pump();
                let mut hello = temper_engine_domain_world::protocol::hello_slots(
                    fixtures::QUESTION_WORKER,
                    limits.fleet.slots,
                    hosting,
                );
                if let engine::Event::Worker(engine::WorkerInput::Link {
                    link: engine::Link::Hello { hello, .. },
                    ..
                }) = &mut hello
                {
                    hello.bounds.message_bytes = limits.tasks.message_bytes;
                }
                self.engine.send(hello);
                self.engine.settle();
                self.reads = 0;
                self.read_loads.clear();
                self.offline_until = Some(now.saturating_add(offline));
                self.observed.push(Observed::Reachable { live: false });
                self.drop_watches();
            }
            FrontFault::Slow { span } => self.due = Some(now.saturating_add(span)),
            // The engine receives the request but its answer is lost on a live link.
            // Report Lost immediately: this front has no exchange deadline to wait for
            // (jig's client/domain/client/README.md, 13); jig-client owns the retry backoff.
            FrontFault::NoAnswer => self.lose_answers = self.lose_answers.checked_add(1).expect("finite lost answers"),
            FrontFault::DropWatch => self.drop_watches(),
            FrontFault::Busy => self.keyed_fault = Some(RequestEnvelope::Again),
            FrontFault::SignedOut => self.keyed_fault = Some(RequestEnvelope::SignedOut),
            FrontFault::NotReady | FrontFault::Versions | FrontFault::ObjectGone => {
                panic!("not a temper joint story fault: {fault:?}")
            }
        }
    }
    fn reload(&mut self, who: ClientId, now: Time) {
        self.advance(now);
        self.detach(who);
        // A terminal restart has no retained address, unlike a browser reload.
        self.callbacks.push((who, RootEvent::Went { place: client::Place::Address(fixtures::inbox()) }.into()));
        if self.offline_until.is_none() {
            if self.transport.sign_in(who).is_some() {
                self.callbacks.push((
                    who,
                    RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }.into(),
                ));
            }
            self.hello(who);
            if self.transport.sign_in(who).is_none() {
                self.sign_outs.entry(who).or_default().push_back(None);
                self.callbacks.push((
                    who,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }),
                ));
            }
        }
    }
    fn received(&mut self, _who: ClientId, event: &Event) {
        if matches!(
            event,
            Event::Root(RootEvent::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(_)),
                        ..
                    }
                }
            })
        ) {
            self.me_deliveries += 1;
        }
    }
    fn sign_in(&mut self, who: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        self.callbacks.retain(|(client, event)| {
            *client != who
                || !matches!(
                    event,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } })
                )
        });
        self.sign_outs.remove(&who);
        self.callbacks.push((
            who,
            Event::Root(translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })),
        ));
        self.component.sign_in(
            who,
            self.sign_in_identity.clone().unwrap_or_else(|| jig_domain_people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            }),
        );
        self.pump_component();
    }
    fn delivers(&mut self, client: ClientId, event: &Event) -> bool {
        if !matches!(
            event,
            Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } })
        ) {
            return true;
        }
        let sign_in = self
            .sign_outs
            .get_mut(&client)
            .and_then(VecDeque::pop_front)
            .expect("sign-out callback retains its exchange");
        self.transport.sign_in(client).is_none_or(|current| Some(current) == sign_in)
    }
    fn sign_out(&mut self, who: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        if self.offline_until.is_some() {
            self.callbacks.push((who, Event::Root(RootEvent::Client { event: jig_client::Event::SignOutLost })));
            return;
        }
        let Some(sign_in) = self.transport.sign_in(who) else { return };
        self.component.sign_out(who, sign_in);
        self.pump_component();
    }
}

// The kit's truth reads the unchanged core family of the externally applied store.
// Temper's other durable families are not jig client-referee facts.
fn core_rows(engine: &Driver) -> BTreeMap<jig_test_domain::Key, jig_test_domain::Record> {
    engine
        .store
        .rows
        .iter()
        .filter_map(|(key, record)| {
            if let (engine::Key::Core(key), engine::Record::Core(record)) = (key, record) {
                Some((jig_test_domain::Key::Core(key.clone()), jig_test_domain::Record::Core(record.clone())))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;

mod accounts;
