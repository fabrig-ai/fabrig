//! Work keeps one project's lists across its filters, then releases them for another
//! (temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 3;
//! temper/docs/design/testing/checks.md, 3).
use jig_client::{Handle, Latest};
use jig_client_kit::harness::{ClientId, Settings, Until, World};
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_person::{Find, Step, Target};
use skein_lib::{ReplyTo, Token};
use skein_ui::{Role, State};
use skein_world::domain::Verdict;
use temper_client_domain::{Address, ChatsFilter, GoalsFilter, WorkFilter};
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_world::{store::Store, walking};

type ClientWorld = World<Temper, Joint>;

fn client_limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.reads = 16; // Three lists, their refreshes, notes, reports and the change's task.
    // Session watches and both projects' lists overlap while the first project's watches close.
    limits.client.domain.handles = 24;
    limits.client.domain.watches = 24;
    limits
}
fn client(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    let limits = client_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    (
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        temper_client_world::app::View::Person(Box::new(temper_client_view::View::new(
            &bounds,
            &limits,
            words,
            fixtures::FORGE,
        ))),
    )
}
const FIRST: u32 = 1;
const SECOND: u32 = 2;
const FIRST_GOAL: &[u8] = b"Plan the orchard";
const SECOND_GOAL: &[u8] = b"Plan the harbour";
const FIRST_CHAT: &[u8] = b"Sketch the orchard";
const SECOND_CHAT: &[u8] = b"Sketch the harbour";
const CHANGE: &[u8] = b"Plant the first tree";
const BRANCH: &[u8] = b"temper/orchard";
const WORKER: Token = Token::new(7);
// Two goals, two chats and the first chat's change.
const TASKS: u32 = 5;
// Two running chats; the listed goals reserve no spend and the change is a procedure.
const SLOTS: u32 = 2;
// Each chat reserves 200 of its project's 500.
const CHAT_SPEND: u64 = 200;
const CHANGE_SPEND: u64 = 100;
// Forge read, push, open, land, issue and comment grants, as the change-panel fixture.
const GRANTS: [u16; 6] = [1, 2, 3, 4, 7, 13];

fn configuration(limits: &mut engine::Limits) -> engine::Config {
    let mut config = walking::config(fixtures::SEED);
    config.core.connectors = Box::new([fixtures::FORGE]);
    config.core.defaults.rules.ceiling.notes = authority::Scopes(1);
    let executors: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(2)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&executors);
    config.core.defaults.rules.ceiling.delegation.depth = 2;
    let grants = GRANTS
        .map(|kind| authority::Grant {
            connector: fixtures::FORGE,
            kind,
            pattern: authority::Pattern {
                segments: [b"forge".as_slice(), fixtures::FORGE_HOST, b"acme"].map(Box::from).into(),
                last: authority::Last::Open(b"shop".as_slice().into()),
            },
        })
        .into();
    config.core.defaults.rules.ceiling.grants = grants;
    let project = &mut config.core.seed_projects[0].policy;
    project.permissions = Box::new([]);
    project.policy.ceiling = config.core.defaults.rules.ceiling.clone();
    project.policy.projections.clone_from(&project.policy.ceiling.grants);
    let owner = &mut project.policy.roles[0].authority;
    owner.notes = authority::Scopes(1);
    owner.delegation.kinds = executors;
    owner.delegation.depth = 2;
    owner.grants.clone_from(&project.policy.ceiling.grants);
    for template in &mut config.core.defaults.templates {
        if template.tracked.is_some() {
            template.deadline = people::TemplateDeadline::None;
        }
    }
    // The deployment owner makes the second project through the same entrance as the switcher.
    let owner = config.core.owners[0].clone();
    config.core.owners = Box::new([owner.clone(), people::InitialOwner { project: 0, identity: owner.identity }]);
    limits.people.initial_owners = 2;
    temper_client_world::front::first_run_projects(&mut config, limits);
    // A pushed change waits outside the queue until the owner reviews it on the forge.
    let landing = engine::LandingRule {
        sets: Box::new([]),
        connector: fixtures::FORGE,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), fixtures::FORGE_HOST, b"acme", b"shop", b"branch"].map(Box::from).into(),
            last: authority::Last::Exact(b"main".as_slice().into()),
        },
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([engine::Approval { role: 0, people: 1, freshness: engine::Freshness::Clean }]),
    };
    let mut projects: skein_lib::Map<u32, Box<[engine::LandingRule]>> = skein_lib::Map::with_capacity(1);
    projects.insert(FIRST, Box::new([landing.clone()])).expect("first project's landing");
    config.kinds_stock = engine::landing_stock(&[landing], &projects, 1, 4);
    let mut stock = config.kinds_stock.into_vec();
    for project in [FIRST, SECOND] {
        stock.push(temper_engine_domain_forge::settings::Stock {
            kind: 7,
            scope: project,
            number: 1,
            value: temper_engine_domain_forge::settings::SizedValue {
                size: 0,
                value: temper_engine_domain_forge::settings::Value::Ways(
                    temper_engine_domain_forge::settings::ProjectWays {
                        drafts: true,
                        goals: true,
                        changes: true,
                        way: 0,
                    },
                ),
            },
        });
    }
    config.kinds_stock = stock.into_boxed_slice();
    config
}

fn ask(front: &mut Joint, key: u8, ask: people::Ask) -> people::Outcome {
    let driver = &mut front.engine;
    let to = ReplyTo::new(Token::new(u64::from(key)));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply { to: reply_to, reply: people::Reply::Outcome(outcome) } = delivery
                && *reply_to == to
            {
                Some(outcome.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("engine outcome for {key}: {:?}", driver.delivered))
}

fn task(front: &mut Joint, project: u32, key: u8, words: &[u8], goal: bool) -> u64 {
    // Both projects use the first project's configured policy; the driver's fixture
    // knows seeded projects, while the second was made through the engine.
    let people::Ask::Create { mut batch, .. } = front.engine.chat_ask(FIRST, words) else { panic!("create") };
    let spec = &mut batch[0];
    spec.budget = people::Budget::Named;
    spec.authority.spend = if goal { 0 } else { CHAT_SPEND };
    if goal {
        spec.tracked = Some(1);
        spec.label = 2;
        spec.flags.conversation = false;
        spec.flags.listed = true;
    }
    let people::Outcome::Created { first, .. } = ask(front, key, people::Ask::Create { project, batch }) else {
        panic!("created task")
    };
    first
}

fn draft(front: &mut Joint, project: u32, key: u8, words: &[u8]) -> u64 {
    let task = task(front, project, key, words, false);
    let run = front.engine.last_assignment();
    let input = format!(
        r##"{{"scope":{{"kind":"goal","project":{project},"goal":{task}}},"description":"Draft for project {project}","body":"# Project {project}\nOur next steps\n"}}"##
    );
    let (answer, failed) =
        temper_engine_domain_world::host_tool::host_tool(&mut front.engine, &run, 1, b"note", input.as_bytes());
    assert!(!failed, "document write: {}", String::from_utf8_lossy(&answer));
    fixtures::reading_turn(front, task, b"Our draft is ready");
    task
}

fn change(front: &mut Joint, chat: u64) -> u64 {
    front.engine.protocol.defer_render = false;
    front.engine.forge.as_mut().expect("forge").push_branch(7, BRANCH);
    let mut child = temper_engine_domain_world::fixtures::report_delegate(CHANGE, Box::new([]));
    child.executor = tasks::Executor::Procedure { connector: fixtures::FORGE, code: 2 };
    child.authority.tools = tasks::Tools(0);
    child.spec.parameters = Box::new([
        tasks::Parameter::Resource {
            name: 1,
            connector: fixtures::FORGE,
            resource: temper_engine_domain_forge::tag(1, 7),
        },
        tasks::Parameter::Bytes { name: 3, value: BRANCH.into() },
        tasks::Parameter::Bytes { name: 10, value: CHANGE.into() },
    ]);
    child.contract = tasks::Contract::Change { connector: fixtures::FORGE, kind: 4, words: 128 };
    let owner = &front.engine.task(chat).expect("requesting chat").authority;
    child.authority.grants.clone_from(&owner.grants);
    child.authority.budget.spend = CHANGE_SPEND;
    let attempt = front.engine.task(chat).expect("running chat").attempt;
    front.engine.call(
        WORKER,
        chat,
        attempt,
        2,
        engine::Call { completion: 2, position: 0, tool: engine::Tool::Delegate { batch: Box::new([child]) } },
    );
    front.engine.settle();
    let Some(engine::CallAnswer::Delegated(children)) = front.engine.answered(2) else {
        panic!("change delegated: {:?}; deliveries: {:?}", front.engine.answered(2), front.engine.delivered)
    };
    let change = children[0].task;
    // The requesting run yields its repository while its delegated change lands.
    let spent = front.engine.task(chat).expect("requesting chat").run_spent;
    front.engine.answer(WORKER, chat, attempt, spent, tasks::End::Parked, engine::Parts::none());
    front.engine.settle();
    change
}

fn world() -> ClientWorld {
    let mut limits = temper_client_world::front::engine_limits();
    limits.tasks.tasks = TASKS;
    limits.tasks.project_tasks = 3; // First goal, chat and change.
    limits.people.inbox_tasks = TASKS * 2;
    limits.people.requests = 12; // Adoption, project and four task creations plus client automatic Seen.
    limits.people.amendment_bytes = 8192; // Six forge grants in the creation's authority.
    limits.authority.executors = 2;
    limits.tasks.executor_kinds = 2;
    limits.documents.executors = 2;
    limits.tasks.depth = 2;
    limits.tasks.parameters = 3; // Change repository, pushed branch and title.
    limits.forge.client.pending = 8; // Goal projections, branch watches and the change's fresh reads overlap.
    limits.authority.grants = u32::try_from(GRANTS.len()).expect("fixture grants");
    limits.tasks.authority_grants = limits.authority.grants;
    limits.documents.grants = limits.authority.grants;
    limits.documents.segments = 9; // A change's held forge resource and its grants fit their full paths.
    limits.authority.segments = 8; // Full branch paths, including their slash-separated suffixes.
    limits.authority.segment_bytes = 64;
    limits.tasks.authority_segments = 9;
    limits.tasks.authority_bytes = 512;
    limits.notes.scopes = 3 + limits.authority.grants; // Deployment, project, task and grant scopes.
    limits.fleet.slots = SLOTS;
    limits.call_records = 3; // Two retained note answers and the change's delegation.
    limits.forge.named_calls = limits.call_records;
    limits.tasks.spec_bytes = 384; // The landing procedure's approval question.
    limits.tasks.inbox_bytes = 1024;
    limits.journal.result_bytes = 4096;
    limits.journal.run_bytes = 8192;
    limits.journal.transcript_bytes = 32768; // Whole task rows with the change's forge grants.
    let config = configuration(&mut limits);
    let restart = temper_engine_domain_world::oracle::config::copy(&config);
    let mut front = Joint::with_engine(
        fixtures::SEED,
        &client_limits(),
        Store::with_limits(&limits),
        [config, restart],
        &limits,
        SLOTS,
    );
    fixtures::adopt_change_repository(&mut front);
    let outcome = ask(&mut front, 61, people::Ask::MakeProject { name: "Harbour".into(), policy: 1 });
    assert!(matches!(outcome, people::Outcome::ProjectMade { project: SECOND, .. }), "{outcome:?}");
    task(&mut front, FIRST, 62, FIRST_GOAL, true);
    task(&mut front, SECOND, 63, SECOND_GOAL, true);
    let chat = draft(&mut front, FIRST, 64, FIRST_CHAT);
    draft(&mut front, SECOND, 65, SECOND_CHAT);
    change(&mut front, chat);
    World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front)
}

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}
fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: fixtures::WITHIN }
}
fn run(world: &mut ClientWorld, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn filter(label: &[u8]) -> Find {
    Find::named(Role::Link, label)
        .within(Box::new([Target::ExactRole { role: Role::Navigation, name: b"Work's filters".as_slice().into() }]))
}

fn handles(world: &ClientWorld, person: ClientId) -> [Handle; 3] {
    let root = world.root(person).person();
    let pages = root.pages();
    let goals = pages.goals().expect("goals open").live;
    let chats = pages.chats().expect("chats open").live;
    // This world's first page has allocated at most its handle bound. Find the
    // changes document by its type, without depending on its allocation order.
    let bound = u64::from(client_limits().client.domain.handles);
    let changes = (goals.number..goals.number + bound)
        .find_map(|number| {
            let handle = Handle { number, ..goals };
            if let Some(document) = root.client().document(handle)
                && let Some(Latest::Family(record)) = document.latest
                && temper_client_domain::forge::changes::panel(record, client_limits().forge).is_some()
            {
                Some(handle)
            } else {
                None
            }
        })
        .expect("changes open");
    [goals, changes, chats]
}

fn first_project(world: &mut ClientWorld, person: ClientId) {
    run(
        world,
        person,
        vec![
            see(Find::text("Project 1 · Work".as_bytes())),
            see(Find::exactly_named(Role::Heading { level: 1 }, b"One in flight, one draft.")),
            see(filter(b"All")),
            see(filter(b"Drafts")),
            see(filter(b"Goals")),
            gone(filter(b"Changes")),
            gone(filter(b"Chats")),
            see(Find::named(Role::Link, FIRST_GOAL)),
            gone(Find::named(Role::Link, CHANGE)),
            see(Find::text(FIRST_CHAT)),
            see(Find::text(b"draft-for-project-1")),
            gone(Find::text(SECOND_GOAL)),
            gone(Find::text(SECOND_CHAT)),
            gone(Find::text(b"draft-for-project-2")),
        ],
    );
}

#[test]
fn work_keeps_one_projects_lists_across_filters_then_switches_projects() {
    let mut world = world();
    let all = Address::Work { project: FIRST, filter: WorkFilter::All };
    world.front().next_client_limits(client_limits());
    let person = world.client_with(fixtures::PARTY, all.clone(), client_limits(), client);
    first_project(&mut world, person);
    let held = handles(&world, person);
    let reads = (world.front().engine.loads.len(), world.front().note_reads, world.front().open_watches());
    for (label, address, shown, hidden) in [
        (b"Drafts".as_slice(), Address::Work { project: FIRST, filter: WorkFilter::Drafts }, FIRST_CHAT, FIRST_GOAL),
        (b"Goals".as_slice(), Address::Goals { project: FIRST, filter: GoalsFilter::All }, FIRST_GOAL, FIRST_CHAT),
        (b"All".as_slice(), all.clone(), FIRST_GOAL, SECOND_GOAL),
    ] {
        run(
            &mut world,
            person,
            vec![Step::Press { find: filter(label) }, see(Find::text(shown)), gone(Find::text(hidden))],
        );
        assert_eq!(world.root(person).person().pages().address(), &address, "{label:?} is a move");
        assert_eq!(handles(&world, person), held, "{label:?} keeps all three reads");
        assert_eq!(
            (world.front().engine.loads.len(), world.front().note_reads, world.front().open_watches()),
            reads,
            "{label:?} reads nothing again",
        );
    }
    // The sign-off wait supplies no Work filter; the shared chats route still opens.
    world.go(person, Address::Chats { filter: ChatsFilter::All });
    run(
        &mut world,
        person,
        vec![see(Find::text("Project 1 · chats".as_bytes())), see(Find::named(Role::Link, b"+ New chat"))],
    );
    assert_eq!(world.root(person).person().pages().address(), &Address::Chats { filter: ChatsFilter::All });
    world.go(person, all);
    run(
        &mut world,
        person,
        vec![
            see(Find::named(Role::Link, FIRST_GOAL)),
            see(Find::text(FIRST_CHAT)),
            Step::Press { find: filter(b"Drafts") },
            see(Find::text(FIRST_CHAT)),
            gone(Find::text(FIRST_GOAL)),
        ],
    );
    let first_handles = handles(&world, person);
    run(
        &mut world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Harbour") },
            see(Find::exactly_named(Role::Disclosure, b"Harbour")),
            see(Find::text(SECOND_CHAT)),
            gone(Find::text(SECOND_GOAL)),
            see(Find::text(b"draft-for-project-2")),
            gone(Find::text(FIRST_GOAL)),
            gone(Find::text(FIRST_CHAT)),
            gone(Find::text(CHANGE)),
            gone(Find::text(b"draft-for-project-1")),
        ],
    );
    let root = world.root(person).person();
    assert_eq!(root.pages().address(), &Address::Work { project: SECOND, filter: WorkFilter::Drafts });
    assert!(
        world.tree(person).nodes().iter().any(|node| {
            node.role == Role::Button
                && node.name.as_deref() == Some(b"Harbour")
                && node.states.as_slice().contains(&State::Pressed)
        }),
        "switcher names the second project as open"
    );
    for handle in first_handles {
        assert!(root.client().document(handle).is_none(), "first project's read released: {handle:?}");
    }
    run(
        &mut world,
        person,
        vec![
            Step::Press { find: filter(b"All") },
            see(Find::named(Role::Link, SECOND_GOAL)),
            see(Find::text(SECOND_CHAT)),
            gone(Find::text(FIRST_GOAL)),
            gone(Find::text(FIRST_CHAT)),
            gone(Find::text(CHANGE)),
        ],
    );
}
