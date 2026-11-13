//! Settings travel through the real restart, store and commit barrier (domain/engine/settings.md, 7).
use jig_core_world::client_peer::ClientPeer;
use jig_core_world::fixtures;
pub(super) use jig_core_world::fixtures::PARTY as OWNER;
use jig_core_world::fixtures::sign_in as signin;
use jig_core_world::peers;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_link::Hello;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

pub(super) const SEED: u64 = 37_001;

pub(super) fn configuration() -> (root::Config, root::Limits) {
    let document = jig_protocol_config::document(
        include_bytes!("engine.json"),
        &jig_protocol_config::DocumentLimits { bytes: 16384, tokens: 4096, depth: 16 },
    )
    .expect("settings story fixture");
    let mut engine = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Document,
    )
    .expect("settings story fixture");
    engine.charters[0].policy.instructions = Box::from(b"Answer within the granted authority.".as_slice());
    let mut config = jig_test_profile::config(SEED);
    config.core = jig_protocol_config::core(&engine, [31; 16], SEED);
    // Auxiliary parties have configured roles even when they cannot decide settings.
    for project in &mut config.core.seed_projects {
        auxiliary_roles(&mut project.policy.policy);
    }
    let numbers: Vec<_> = config.core.defaults.policies.iter().map(|(number, _)| *number).collect();
    for number in numbers {
        auxiliary_roles(&mut config.core.defaults.policies.get_mut(&number).expect("fixture policy").policy);
    }
    let mut limits = jig_test_profile::limits();
    limits.core = engine.limits.core;
    peers::journal(&mut limits);
    (config, limits)
}
fn auxiliary_roles(policy: &mut jig_domain_authority::Policy) {
    let mut roles = policy.roles.to_vec();
    for number in [people::Role::Member.number(), people::Role::Observer.number()] {
        let mut role = roles[0].clone();
        role.number = number;
        role.requests = jig_domain_authority::Requests(0);
        role.decides = jig_domain_authority::Proposals(0);
        roles.push(role);
    }
    policy.roles = roles.into_boxed_slice();
}
pub(super) fn start(history: u32) -> World {
    let (config, mut limits) = configuration();
    limits.core.settings.history = history;
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        boot: jig_core_world::world::Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    assert!(world.domain.ready());
    world
}
pub(super) fn sign_in(world: &mut World) -> u64 {
    world.send(signin(Token::new(SEED), 1, b"owner-a", &[], &[]));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { person, sign_in, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("owner signs in")
    };
    assert_eq!(person, OWNER);
    sign_in
}
/// Encode a distinct key for every numbered deciding request.
pub(super) fn request_key(number: u32) -> [u8; 16] {
    u128::from(number).to_be_bytes()
}
pub(super) fn ask(world: &mut World, signed: u64, key: u8, ask: people::Ask) -> people::Reply {
    ask_key(world, signed, [key; 16], ask)
}
/// Submit a deciding act with a full-width key and return its reply after the world settles.
pub(super) fn ask_key(world: &mut World, signed: u64, key: [u8; 16], ask: people::Ask) -> people::Reply {
    world.send(ask_event(signed, key, ask));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("request reply")
    };
    reply
}
pub(super) fn frame(world: &mut World, signed: u64) -> documents::Frame {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        read: reads::Read::Frame,
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })) =
        world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("me reply")
    };
    let documents::Document::Frame(frame) = *document else { panic!("me document") };
    frame
}
pub(super) fn key(key: settings::Key) -> root::Key {
    root::Key::Core(core::Key::Core(core::CoreKey::Settings(key)))
}
pub(super) fn record(record: settings::Record) -> root::Record {
    root::Record::Core(core::Record::Core(core::CoreRecord::Settings(record)))
}
pub(super) fn revision(world: &World, number: u32, history: bool) -> &settings::Revision {
    let address = if history { settings::Key::History(number) } else { settings::Key::Held(number) };
    let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Revision {
        revision, ..
    }))) = &world.store.rows[&key(address)]
    else {
        panic!("settings revision")
    };
    revision
}
pub(super) fn restart(world: &mut World, config: root::Config) {
    world.restart_with(config);
    assert!(
        world.domain.ready(),
        "restart trace: {:?}; steps: {:?}; failure: {:?}",
        world.trace,
        world.restart_steps,
        world.domain.start_failure()
    );
}

pub(super) fn change(
    world: &mut World,
    signed: u64,
    key_number: u8,
    against: u32,
    items: Box<[settings::Item]>,
) -> people::Reply {
    change_key(world, signed, [key_number; 16], against, items)
}
/// Apply settings with a full-width key and a digest stable across retries.
pub(super) fn change_key(
    world: &mut World,
    signed: u64,
    key: [u8; 16],
    against: u32,
    items: Box<[settings::Item]>,
) -> people::Reply {
    world.send(change_event(signed, key, against, items));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("settings reply")
    else {
        panic!("settings reply")
    };
    reply
}
pub(super) fn template_edit(world: &World, grace: u64) -> settings::Item {
    let mut value = world.domain.settings().template(1).expect("chat template").clone();
    value.grace = Some(Duration::from_secs(grace));
    settings::Item {
        setting: settings::Setting::Template(1),
        edit: settings::Edit::Set(settings::Value::Template(Box::new(value))),
    }
}
pub(super) fn changed(reply: &people::Reply, revision: u32) {
    assert_eq!(reply, &people::Reply::Outcome(people::Outcome::SettingsChanged { revision }));
}

pub(super) fn settings_rows(world: &World) -> Vec<(root::Key, root::Record)> {
    world
        .store
        .rows
        .iter()
        .filter(|(key, _)| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Settings(_)))))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

pub(super) fn propose(
    world: &mut World,
    signed: u64,
    request_key: u8,
    against: u32,
    items: Box<[settings::Item]>,
) -> people::Reply {
    world.send(root::Event::Core(core::Event::ProposeSettings {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [request_key; 16],
        digest: [request_key; 32],
        change: Box::new(settings::Proposed {
            against,
            items,
            reason: b"Please review these settings".as_slice().into(),
        }),
    }));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("proposal reply")
    else {
        panic!("proposal reply")
    };
    reply
}
pub(super) fn restore(world: &mut World, signed: u64, key: u8, revision: u32, reason: &[u8]) -> people::Reply {
    ask(world, signed, key, people::Ask::Restore { revision, reason: reason.into() })
}
pub(super) fn restored(reply: &people::Reply) -> (u64, Box<[settings::Setting]>) {
    let people::Reply::Outcome(people::Outcome::Restored { proposal, to, left_out }) = reply else {
        panic!("restored: {reply:?}")
    };
    assert_eq!(*to, people::Role::Owner.number());
    (*proposal, left_out.clone())
}
pub(super) fn proposed(reply: &people::Reply) -> u64 {
    let people::Reply::Outcome(people::Outcome::Proposed { proposal, to }) = reply else {
        panic!("proposed: {reply:?}")
    };
    assert_eq!(*to, people::Role::Owner.number());
    *proposal
}
pub(super) fn party(world: &mut World, subject: &[u8], role: Option<(u32, people::Role)>) -> (u64, u64) {
    world.send(signin(Token::new(SEED), 1, subject, subject, subject));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { person, sign_in, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("client").delivered.pop().expect("signed in")
    else {
        panic!("signed in")
    };
    if let Some((project, role)) = role {
        let root::Record::Core(core::Record::People(people::Stored::Roles { holdings, .. })) =
            &world.store.rows[&root::Key::Core(core::Key::People(people::Key::Roles(project)))]
        else {
            panic!("seeded roster")
        };
        let mut holdings = holdings.to_vec();
        holdings.push(people::Holding { person, role });
        world.send(root::Event::Core(core::Event::People(people::Event::Roles {
            project,
            holdings: holdings.into_boxed_slice(),
        })));
    }
    (person, sign_in)
}
pub(super) fn decide(
    world: &mut World,
    signed: u64,
    request_key: u8,
    proposer: u64,
    proposal: u64,
    decision: people::ProposalDecision,
    running: Option<people::Running>,
) -> people::Reply {
    ask(
        world,
        signed,
        request_key,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Party(proposer),
            proposal,
            decision,
            running,
        },
    )
}
pub(super) fn read(world: &mut World, signed: u64, value: reads::Read) -> reads::Answer {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        read: value,
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer, .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("read reply")
    else {
        panic!("read reply")
    };
    answer
}
pub(super) fn proposal_document(world: &mut World, signed: u64, proposer: u64, proposal: u64) -> documents::Proposal {
    let reads::Answer::Document(document) =
        read(world, signed, reads::Read::Proposal { proposer: people::Proposer::Party(proposer), proposal })
    else {
        panic!("pending proposal document")
    };
    let documents::Document::Proposal(value) = *document else { panic!("proposal document") };
    *value
}
pub(super) fn inventory(world: &mut World, signed: u64) -> documents::Settings {
    let reads::Answer::Document(document) = read(world, signed, reads::Read::Settings) else {
        panic!("settings document")
    };
    let documents::Document::Settings(value) = *document else { panic!("settings inventory") };
    *value
}
pub(super) fn project_policy_item(world: &World, project: u32, period_spend: u64) -> settings::Item {
    let mut policy = world.domain.settings().policy(1).expect("fixture policy").clone();
    policy.policy.period_spend = period_spend;
    settings::Item {
        setting: settings::Setting::ProjectPolicy { project },
        edit: settings::Edit::Set(settings::Value::Policy(Box::new(policy))),
    }
}
pub(super) fn project_document(world: &mut World, signed: u64, project: u32) -> documents::Project {
    let reads::Answer::Document(document) = read(world, signed, reads::Read::Project { project }) else {
        panic!("project document")
    };
    let documents::Document::Project(value) = *document else { panic!("project document") };
    *value
}
pub(super) fn proposal_set_key(proposer: u64, proposal: u64) -> root::Key {
    key(settings::Key::Proposal {
        proposer: people::Proposer::Party(proposer),
        proposal: u32::try_from(proposal).expect("proposal fits"),
    })
}
pub(super) fn charter_and_template(world: &World) -> Box<[settings::Item]> {
    let mut charter = world.domain.settings().charter(1).expect("configured charter").clone();
    charter.policy.instructions = b"Changed by review".as_slice().into();
    Box::new([
        settings::Item {
            setting: settings::Setting::Charter(1),
            edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))),
        },
        settings::Item { setting: settings::Setting::Template(2), edit: settings::Edit::Removed },
    ])
}
pub(super) fn removing_charter(world: &World) -> Box<[settings::Item]> {
    let mut rules = world.domain.settings().rules().clone();
    rules.ceiling.delegation.kinds = Box::new([]);
    Box::new([
        settings::Item { setting: settings::Setting::Charter(1), edit: settings::Edit::Removed },
        settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Removed },
        settings::Item { setting: settings::Setting::Policy(1), edit: settings::Edit::Removed },
        settings::Item {
            setting: settings::Setting::Rules,
            edit: settings::Edit::Set(settings::Value::Rules(Box::new(rules))),
        },
    ])
}
pub(super) fn updates(world: &mut World, signed: u64) -> Vec<documents::InboxEntry> {
    let reads::Answer::Page(page) = read(
        world,
        signed,
        reads::Read::Inbox {
            group: reads::Group::Updates,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        },
    ) else {
        panic!("updates page")
    };
    page.rows
        .into_boxed()
        .into_vec()
        .into_iter()
        .map(|row| {
            let reads::PageRow::Inbox(entry) = row else { panic!("inbox entry") };
            entry
        })
        .collect()
}

pub(super) fn run_configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    config.core.defaults.rules.ceiling.tools.0 |= 1024 | core::ToolFamilies::JIG.ask.0;
    config.core.defaults.rules.maximum_run_spend = 20;
    for seed in &mut config.core.seed_projects {
        seed.policy.policy.ceiling.tools.0 |= 1024 | core::ToolFamilies::JIG.ask.0;
        for role in &mut seed.policy.policy.roles {
            role.authority.tools.0 |= 1024 | core::ToolFamilies::JIG.ask.0;
        }
    }
    let policy = config.core.defaults.policies.get_mut(&1).expect("project policy");
    policy.policy.ceiling.tools.0 |= 1024 | core::ToolFamilies::JIG.ask.0;
    for role in &mut policy.policy.roles {
        role.authority.tools.0 |= 1024 | core::ToolFamilies::JIG.ask.0;
    }
    limits.core.settings.proposals = 2;
    peers::journal(&mut limits);
    (config, limits)
}
pub(super) fn proposing_run(settings_family: bool) -> (World, u64, u64) {
    settings_run(settings_family, None)
}
pub(super) fn settings_run(settings_family: bool, history: Option<u32>) -> (World, u64, u64) {
    settings_run_with_limits(settings_family, history, run_configuration().1)
}
/// Start a proposing run with explicit core bounds and the usual fake peers.
pub(super) fn settings_run_with_limits(
    settings_family: bool,
    history: Option<u32>,
    mut limits: root::Limits,
) -> (World, u64, u64) {
    let config = run_configuration().0;
    if let Some(history) = history {
        limits.core.settings.history = history;
    }
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        boot: jig_core_world::world::Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let first = settings_goal(&mut world, signed, 1, 101, settings_family);
    (world, signed, first)
}
pub(super) fn settings_goal(world: &mut World, signed: u64, project: u32, key: u8, settings_family: bool) -> u64 {
    let mut goal = fixtures::chat(world, 1, b"Review the settings".as_slice().into());
    let people::Ask::Create { project: goal_project, batch } = &mut goal else { panic!("goal creation") };
    *goal_project = project;
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 40;
    batch[0].authority.tools = if settings_family { 2047 } else { 1023 };
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = ask(world, signed, key, goal) else {
        panic!("goal admitted")
    };
    assert_eq!(world.assigned.as_ref().expect("assigned proposing goal").task, first);
    first
}

pub(super) fn run_charter(world: &World, instructions: &[u8]) -> settings::Item {
    let mut charter = world.domain.settings().charter(1).expect("fixture charter").clone();
    charter.policy.instructions = instructions.into();
    settings::Item {
        setting: settings::Setting::Charter(1),
        edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))),
    }
}

// Peak 16,224,965 bytes with the stock workloads; a quarter above, rounded up.
const REQUEST_HEAP_CEILING: u64 = 21_000_000;

/// A stock profile's whole room of deciding requests answered, a cold
/// restart, and the first request replayed (the heap ceiling checks each step).
pub(super) fn whole_room_of_answers(profile: jig_protocol_config::Profile) {
    let (config, mut limits) = configuration();
    let people = jig_protocol_config::profile(profile).core.people;
    limits.core.people.requests = people.requests;
    limits.core.people.request_retention = people.request_retention;
    let mut world = RequestWorld::new(config, &limits);
    let signed = world.sign_in();
    let requests = people.requests;
    for number in 1..requests {
        world.change(signed, number);
        assert_answer(&world.store.rows, number, &people::Outcome::SettingsChanged { revision: number + 1 });
    }
    world.visibility(signed, requests);
    assert_answer(&world.store.rows, requests, &people::Outcome::VisibilitySet { project: 1, public: true });
    world.restart();
    world.change(signed, 1);
}

pub(super) fn assert_answer(
    rows: &std::collections::BTreeMap<root::Key, root::Record>,
    number: u32,
    expected: &people::Outcome,
) {
    let address = root::Key::Core(core::Key::People(people::Key::Answer(people::RequestKey {
        person: OWNER,
        key: request_key(number),
    })));
    let root::Record::Core(core::Record::People(people::Stored::Answer { outcome, .. })) = &rows[&address] else {
        panic!("durable deciding answer")
    };
    assert_eq!(outcome, expected);
}

/// The core and journal with a domain store fake, without protocol copies or histories.
pub(super) struct RequestWorld {
    /// The core and its commit barrier under test.
    pub(super) domain: root::Domain,
    /// Durable answers retained by the domain store fake.
    pub(super) store: jig_fake_store::Store<root::Key, root::Record>,
    env: skein_lib::Env<root::Limits>,
    meter: skein_world::domain::heap::Meter,
}
impl RequestWorld {
    /// Start a deployment with the caller's limits and empty durable rows.
    pub(super) fn new(config: root::Config, limits: &root::Limits) -> Self {
        let meter = skein_world::domain::heap::Meter::new();
        let mut world = Self {
            domain: request_domain(config, limits),
            store: jig_fake_store::Store::new(),
            env: skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: *limits },
            meter,
        };
        world.send(root::Event::StartBegin { pending: false });
        assert!(
            world.domain.ready(),
            "phase {:?}; failure {:?}",
            world.domain.start_phase(),
            world.domain.start_failure()
        );
        world
    }

    /// Drive each commit whole before releasing its replies, without advancing time.
    pub(super) fn send(&mut self, event: root::Event) -> Option<people::Reply> {
        root::step(&mut self.domain, &self.env, event);
        let mut reply = None;
        for _ in 0..300 {
            let mut out = skein_lib::Queue::uncapped();
            root::release(&mut self.domain, &self.env, &mut out);
            if out.is_empty() {
                assert!(self.domain.quiescent());
                return reply;
            }
            while let Some(request) = out.pop() {
                match request {
                    root::Request::Commit { number, writes } => self.commit(number, writes),
                    root::Request::Start { phase, census } => self.restore(phase, census),
                    root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { reply: value, .. })) => {
                        assert!(reply.replace(value).is_none(), "one reply per call");
                    }
                    root::Request::StoreCatalogue { .. }
                    | root::Request::ClientsCatalogue { .. }
                    | root::Request::Deliver(_)
                    | root::Request::Now(_) => {}
                    other @ (root::Request::Stop | root::Request::Busy { .. } | root::Request::ReadResult { .. }) => {
                        panic!("unexpected request: {other:?}");
                    }
                }
            }
            self.domain.reclaim();
            assert!(self.meter.held() <= REQUEST_HEAP_CEILING, "retained answer workload heap");
        }
        panic!("request world settles");
    }

    fn commit(&mut self, number: u64, mut writes: skein_lib::Queue<root::Write>) {
        let mut changes = Vec::new();
        while let Some(write) = writes.pop() {
            match write {
                root::Write::Save(row) => changes.push(jig_fake_store::Write::Save { key: row.key(), row }),
                root::Write::Erase(key) => changes.push(jig_fake_store::Write::Erase(key)),
                root::Write::CopySettingsRevision(number) => {
                    let mut row = self.store.rows[&key(settings::Key::Held(number))].clone();
                    let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                        settings::Record::Revision { history, .. },
                    ))) = &mut row
                    else {
                        panic!("held settings revision")
                    };
                    *history = true;
                    changes.push(jig_fake_store::Write::Save { key: row.key(), row });
                }
                root::Write::Wires => {}
                root::Write::DropTranscript { .. } => panic!("no tasks in the answer workload"),
            }
        }
        self.store.submit(number, changes.into_boxed_slice(), 0);
        assert_eq!(self.store.tick(false), Ok(Some(number)));
        root::step(&mut self.domain, &self.env, root::Event::Committed { number, fill: 0 });
    }

    fn restore(&mut self, phase: core::start::Phase, census: Option<core::Census>) {
        use core::start::Phase;
        let ranges = match phase {
            Phase::LoadCore => core::RestoreRange::LIVE.to_vec(),
            Phase::LoadInbox => vec![core::RestoreRange::Census(census.expect("inbox census"))],
            Phase::Sweep => vec![core::RestoreRange::People],
            Phase::RestoreConnector { .. } | Phase::Switch => Vec::new(),
            other @ (Phase::Link
            | Phase::Wires
            | Phase::AdoptRuns
            | Phase::ReadAfresh { .. }
            | Phase::SettleOutbox { .. }
            | Phase::Open) => panic!("root owns phase {other:?}"),
        };
        for range in ranges {
            for (key, row) in &self.store.rows {
                if let root::Key::Core(key) = key
                    && range.admits(key)
                {
                    root::start_record(&mut self.domain, &self.env, row.clone());
                }
            }
        }
        root::step(&mut self.domain, &self.env, root::Event::StartDone(phase));
    }

    /// Sign in the configured owner through the same authenticated request entrance.
    pub(super) fn sign_in(&mut self) -> u64 {
        let reply = self.send(signin(Token::new(SEED), 1, b"owner-a", &[], &[]));
        let Some(people::Reply::SignedIn { person: OWNER, sign_in, .. }) = reply else {
            panic!("owner signs in: {reply:?}")
        };
        sign_in
    }

    /// Apply a numbered predecessor or replay the same change under its retained key.
    pub(super) fn change(&mut self, signed: u64, number: u32) {
        let mut template = self.domain.settings().template(1).expect("chat template").clone();
        template.grace = Some(Duration::from_secs(6 + u64::from(number % 2)));
        let against = number;
        let items = Box::new([settings::Item {
            setting: settings::Setting::Template(1),
            edit: settings::Edit::Set(settings::Value::Template(Box::new(template))),
        }]);
        let reply = self.send(change_event(signed, request_key(number), against, items)).expect("settings reply");
        changed(&reply, against + 1);
    }

    /// Continue ordinary deciding work through the same shared answer entrance.
    pub(super) fn visibility(&mut self, signed: u64, number: u32) {
        let public = number.is_multiple_of(2);
        assert_eq!(
            self.send(ask_event(signed, request_key(number), people::Ask::SetVisibility { project: 1, public })),
            Some(people::Reply::Outcome(people::Outcome::VisibilitySet { project: 1, public }))
        );
    }

    /// Restore retained keys from the fake's durable rows and preserve the wall clock.
    pub(super) fn restart(&mut self) {
        let newest = self.domain.settings().newest();
        self.store.crash();
        self.domain = request_domain(configuration().0, &self.env.limits);
        self.send(root::Event::StartBegin { pending: false });
        assert!(self.domain.ready());
        assert_eq!(self.domain.settings().newest(), newest);
    }
}

fn request_domain(mut config: root::Config, limits: &root::Limits) -> root::Domain {
    // No task or connector resource participates in settings and answer admission.
    config.first.resources = Box::new([]);
    config.second.resources = Box::new([]);
    root::Domain::new(config, limits)
}

fn ask_event(signed: u64, key: [u8; 16], ask: people::Ask) -> root::Event {
    root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key,
        ask,
    }))
}

fn change_event(signed: u64, key: [u8; 16], against: u32, items: Box<[settings::Item]>) -> root::Event {
    root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key,
        digest: [key, key].concat().try_into().expect("digest width"),
        change: Box::new(settings::Change { against, items, running: settings::Running::Stays }),
    })
}

pub(super) fn run_propose(world: &mut World, against: u32, items: Box<[settings::Item]>) -> core::CallPart {
    run_propose_reason(world, against, items, "Please review these settings")
}
pub(super) fn run_propose_reason(
    world: &mut World,
    against: u32,
    items: Box<[settings::Item]>,
    reason: &str,
) -> core::CallPart {
    use std::fmt::Write as _;
    let (config, mut limits) = run_configuration();
    // The fake transport carries names up to its schema ceiling; admission is the core's.
    limits.core.authority.segments = jig_protocol_store::settings_v3::CEILINGS.document_name;
    limits.core.authority.segment_bytes = jig_protocol_store::settings_v3::CEILINGS.segment_bytes;
    let models = settings::model_names(&config.core.settings.models);
    let bytes = jig_protocol_store::codec::settings::encode(
        &settings::Record::Proposal {
            proposer: people::Proposer::Task(1),
            proposal: 1,
            value: settings::Proposal { key: None, applications: Box::default(), against, items },
        },
        &jig_protocol_store::codec::settings::limits(&limits.core).expect("settings codec limits"),
        &models,
    )
    .expect("encoded whole settings set");
    let mut encoded = String::new();
    for byte in &bytes {
        write!(&mut encoded, "{byte:02x}").expect("hex string");
    }
    let input = format!("{{\"against\":{against},\"set\":\"{encoded}\",\"reason\":\"{reason}\"}}");
    world.tool_call(b"propose_settings", input.as_bytes(), true);
    world
        .answers
        .last()
        .unwrap_or_else(|| {
            panic!(
                "typed tool answer; input bytes {}; host answers {:?}; host calls {:?}; busy {:?}",
                input.len(),
                world.host_answers,
                world.host_calls,
                world.busy_calls
            )
        })
        .2
        .clone()
}
