//! Home's status agent through the root and its durable document (temper/docs/design/domain/forge/attention.md, 3).
use super::*;
use jig_domain_notes as notes;
use skein_lib::{List, Wall};

const CHANNEL: Token = Token::new(7);
const PROJECT: u32 = 1;
const SENTENCE: &[u8] = b"The widget fix has landed.";
const REPORT: &[u8] = b"The widget fix has landed.\n\nThe review passed and the branch is ready.";

#[path = "status/shared.rs"]
mod shared;

fn status_world(allowed: bool, funded: bool) -> World {
    status_world_with_spec(allowed, funded, None)
}
fn status_world_with_spec(allowed: bool, funded: bool, spec_bytes: Option<u32>) -> World {
    status_world_with_sessions(allowed, funded, spec_bytes, Duration::from_secs(7200))
}
fn status_world_with_sessions(allowed: bool, funded: bool, spec_bytes: Option<u32>, lifetime: Duration) -> World {
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    if let Some(bytes) = spec_bytes {
        limits.tasks.spec_bytes = bytes;
    }
    limits.people.sign_in_lifetime = lifetime;
    configured_status_world(allowed, funded, &limits)
}
fn status_world_with_limits(allowed: bool, funded: bool, mut limits: engine::Limits) -> World {
    // The same signed-in person returns for the next sitting.
    limits.people.sign_in_lifetime = Duration::from_secs(7200);
    configured_status_world(allowed, funded, &limits)
}
fn configured_status_world(allowed: bool, funded: bool, limits: &engine::Limits) -> World {
    let model = walking::models()[0].clone();
    let llm = jig_protocol_config::Llm {
        models: (Box::new([model.clone()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),

        endpoints: Box::new([]),
        providers: Box::new([]),
    };
    let mut defaults = temper_engine_service_stock::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    // Scripted workers use the same small run bounds as the other forge stories.
    for charter in &mut defaults.charters {
        let number = charter.number;
        *charter = walking::charter();
        charter.number = number;
    }
    if !funded {
        // The chat reserves the looking owner's entire pool, not the period.
        defaults.policies.get_mut(&1).expect("stock project policy").policy.roles[0].period_spend = 400;
    }
    // The reference's policy may withhold this charter; the procedure still gets made.
    if !allowed {
        for authority in [
            &mut defaults.rules.ceiling,
            &mut defaults.policies.get_mut(&1).expect("stock project policy").policy.ceiling,
        ] {
            authority.delegation.kinds = authority
                .delegation
                .kinds
                .iter()
                .copied()
                .filter(|kind| *kind != authority::Executor::Charter(9))
                .collect();
        }
        for role in &mut defaults.policies.get_mut(&1).expect("stock project policy").policy.roles {
            role.authority.delegation.kinds = role
                .authority
                .delegation
                .kinds
                .iter()
                .copied()
                .filter(|kind| *kind != authority::Executor::Charter(9))
                .collect();
        }
    }
    World::configured_stock_limits(&defaults, &model, limits)
}
fn settle(world: &mut World) {
    world.oracle_settle();
    assert!(world.store.pending.is_empty());
}
fn look_request(world: &mut World, number: u8) {
    world.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: world.signed_in.expect("owner session"),
        key: [number; 16],
        scope: people::ApplicationScope::Project(PROJECT),
        digest: [number; 32],
        carried: None,
        request: engine::Application::Looked { project: PROJECT },
    }));
}
fn looked(world: &mut World, number: u8) {
    look_request(world, number);
    settle(world);
    let looks: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => {
                if let forge_top::Stored::Looked(row) = row.as_ref() {
                    Some(row)
                } else {
                    None
                }
            }
            Record::Core(_) => None,
        })
        .collect();
    assert!(!looks.is_empty(), "look saved; replies={:?}", world.web_replies);
}
fn facts(world: &World) -> &forge_top::ProjectFacts {
    match world.store.rows.get(&Key::Forge(forge_top::Key::ProjectFacts { project: PROJECT })) {
        Some(Record::Forge { row }) => match row.as_ref() {
            forge_top::Stored::ProjectFacts(row) => row,
            other => panic!("facts: {other:?}"),
        },
        other => panic!("durable project facts: {other:?}"),
    }
}
fn person_brief(world: &World, party: u64) -> &forge_top::PersonBrief {
    let Some(Record::Forge { row }) =
        world.store.rows.get(&Key::Forge(forge_top::Key::PersonBrief { party, project: PROJECT }))
    else {
        panic!("durable person's brief")
    };
    let forge_top::Stored::PersonBrief(row) = row.as_ref() else { panic!("person brief row") };
    row
}
fn own_brief(world: &World) -> &forge_top::PersonBrief {
    person_brief(world, 1)
}
fn next_sitting(world: &mut World, number: u8) {
    world.env.now = world.env.now.saturating_add(forge_top::SITTING);
    world.env.wall = Wall::from_nanos(world.env.now.as_nanos());
    looked(world, number);
}
fn ready(allowed: bool) -> World {
    ready_with(allowed, true)
}
fn ready_with(allowed: bool, funded: bool) -> World {
    let (mut world, requester) = opened_chat(allowed, funded);
    land_and_look(&mut world, &requester);
    world
}
fn opened_chat(allowed: bool, funded: bool) -> (World, engine::Assignment) {
    opened_chat_in(status_world(allowed, funded))
}
fn opened_chat_in(mut world: World) -> (World, engine::Assignment) {
    world.adopt_with_checks(false, Box::new([1]));
    let mut hello_event = protocol::hello_slots(CHANNEL, 3, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) =
        &mut hello_event
    else {
        panic!("worker hello")
    };
    hello.bounds.message_bytes = world.env.limits.tasks.result_bytes.max(world.env.limits.tasks.message_bytes);
    hello.bounds.inbox = world.env.limits.tasks.run_inbox;
    world.send(hello_event);
    looked(&mut world, 170);
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Make the widget fix") else {
        panic!("chat request")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].tracked = Some(3);
    batch[0].label = 2;
    let requester = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, requester)
}
fn land_and_look(world: &mut World, requester: &engine::Assignment) {
    land(world, requester);
    next_sitting(world, 171);
}
fn land(world: &mut World, requester: &engine::Assignment) {
    assert!(world.assigned.iter().all(|run| run.charter != 9), "the first look never spends");
    land_numbered(world, requester, 93, 1);
}
fn land_numbered(world: &mut World, requester: &engine::Assignment, number: u64, landed: u64) {
    let change = super::questions::make_change(world, requester, number, engine::ChangeQuestions::None);
    let producer = super::questions::next_producer(world, change);
    let branch = super::questions::row(world, change).branch.clone();
    let raw::Answer::Commit(base) =
        world.external(1, raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() }))
    else {
        panic!("landing head")
    };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    let body = format!("widget fixed {number}");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", body.as_bytes(), 1)
        .expect("producer pushed");
    let reviewer_index = world.assigned.len();
    answer(
        world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
            connector: 1,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    let raw::Answer::Commit(head) = world.external(1, raw::Op::Read(raw::Read::Branch { branch: branch.clone() }))
    else {
        panic!("pushed head")
    };
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let reviewer = super::reviews::next_assignment(world, reviewer_index);
    answer(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    for _ in 0..200 {
        world.tick();
        if world.store.rows.contains_key(&Key::Forge(forge_top::Key::ProjectFacts { project: PROJECT }))
            && facts(world).landed == landed
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(
        facts(world).landed,
        landed,
        "change={:?}; replies={:?}",
        super::questions::row(world, change),
        world.call_errors
    );
}
fn run(world: &World) -> engine::Assignment {
    world
        .assigned
        .iter()
        .rev()
        .find(|run| run.charter == 9)
        .unwrap_or_else(|| panic!("status agent assigned; facts={:?}", facts(world)))
        .clone()
}
fn answer(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
    world.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 5, None, b"Finished the task"));
    settle(world);
    let mut event = world.protocol.answer(
        CHANNEL,
        run.task,
        run.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("worker answer")
    };
    answer.turns = 1;
    world.send(event);
}
fn finish(world: &mut World, run: &engine::Assignment) {
    let party = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::PersonBrief(row)
                    if row
                        .running
                        .as_ref()
                        .and_then(|running| running.task)
                        .is_some_and(|task| live_task(world, task).delegates.contains(&run.task)) =>
                {
                    Some(row.party)
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("run's person brief");
    answer(world, run, tasks::TaskResult::Report { words: REPORT.into() });
    for _ in 0..200 {
        world.tick();
        if person_brief(world, party).running.is_none() && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("status report heard; facts={:?}; task={:?}", facts(world), live_task(world, run.task).phase);
}
fn assert_brief(world: &mut World) {
    let brief = own_brief(world).brief.as_ref().expect("saved status brief");
    assert_eq!(brief.sentence.as_ref(), SENTENCE);
    assert_eq!(brief.lede.as_ref(), b"The review passed and the branch is ready.");
    assert_eq!(brief.landed, 1);
    assert_eq!(own_brief(world).running, None);
    let expected = brief.clone();
    let document =
        world.oracle_document(world.signed_in.expect("owner session"), PROJECT, forge_top::status_subject(PROJECT));
    let Some(forge_top::Document::Status(status)) = document else { panic!("status document") };
    assert_eq!(status.brief, Some(expected));
}
#[test]
fn a_stock_project_with_its_owners_pool_open_writes_its_brief() {
    // The walking model takes K's 256-unit floor: a chat is 24K.
    const CHAT_SPEND: u64 = 24 * 256;
    const POOL_SPEND: u64 = 80 * CHAT_SPEND;
    let (mut world, requester) = opened_chat(true, true);
    let period = tasks::Funder::Period { project: PROJECT, period: 1 };
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(ledger)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(period))))
    else {
        panic!("open period's funding")
    };
    let numbers = ledger.numbers;
    land_and_look(&mut world, &requester);
    let agent = run(&world);
    let tasks::Party::Task(procedure) = live_task(&world, agent.task).requester else { panic!("status procedure") };
    assert_eq!(live_task(&world, procedure).funder, tasks::Funder::Pool { project: PROJECT, person: 1, period: 1 });
    assert_eq!(world.root.tasks().payer(agent.task), Some(tasks::Payer::Party(1)));
    finish(&mut world, &agent);
    assert_brief(&mut world);
    assert_eq!(numbers.reserved, POOL_SPEND, "the owner's open pool reserves eighty chats");
    assert_eq!(
        numbers.budget - numbers.reserved - numbers.spent - numbers.spent_below,
        CHAT_SPEND,
        "one chat remains outside the pool"
    );
    world.close();
}
#[test]
fn a_look_after_a_landing_runs_the_agent_and_saves_its_brief() {
    let mut world = ready(true);
    let agent = run(&world);
    let parent = live_task(&world, agent.task).requester;
    let tasks::Party::Task(parent) = parent else { panic!("procedure parent") };
    assert_eq!(live_task(&world, parent).executor, tasks::Executor::Procedure { connector: 1, code: 5 });
    let words = core::str::from_utf8(&live_task(&world, agent.task).spec.words).expect("status words are UTF-8");
    assert!(words.contains("Small fix"));
    assert!(words.contains("Make the widget fix: 0 of 1 parts done"), "{words}");
    assert!(!words.contains("Milestones:"));
    assert!(!words.contains("Standing roles:"));
    assert!(words.len() <= usize::try_from(world.env.limits.tasks.spec_bytes).expect("spec bound fits usize"));
    finish(&mut world, &agent);
    assert_brief(&mut world);
    world.close();
}
fn roles_world() -> (World, engine::Assignment) {
    roles_world_with_spec(None)
}
fn roles_world_with_spec(spec_bytes: Option<u32>) -> (World, engine::Assignment) {
    let (mut world, requester) = opened_chat_in(status_world_with_spec(true, true, spec_bytes));
    settle(&mut world);
    drain_api(&mut world);
    let mut header = world.store.header();
    // Keep the role numbers in the brief's example, without creating unused tasks.
    header.tasks = 40;
    world.store.seed(
        Box::new([Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header)))]),
        Box::new([]),
    );
    restart_status_world(&mut world);
    (world, requester)
}

fn drain_api(world: &mut World) {
    for _ in 0..200 {
        if world.pending.is_empty() && world.store.pending.is_empty() {
            return;
        }
        world.tick();
    }
    panic!("forge answers drained before restart");
}

fn restart_status_world(world: &mut World) {
    drain_api(world);
    world.restart_same();
    for event in &mut world.events {
        if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = event
        {
            hello.bounds.message_bytes = world.env.limits.tasks.result_bytes.max(world.env.limits.tasks.message_bytes);
            hello.bounds.inbox = world.env.limits.tasks.run_inbox;
        }
    }
    settle(world);
}

fn standing_role(world: &mut World, key: u8) -> u64 {
    let assignment = start_standing_role(world, key);
    park_chat(world, &assignment);
    assignment.task
}

fn start_standing_role(world: &mut World, key: u8) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Watch the tests") else {
        panic!("role defaults")
    };
    let role = &mut batch[0];
    role.label = 3;
    role.tracked = None;
    role.flags.conversation = false;
    role.flags.ended_by = people::EndedBy::Standing;
    // Sixteen funds the same eight-unit run after keeping half for delegates.
    role.budget = people::Budget::AsAllowed { cap: Some(16) };
    assigned_chat(world, key, people::Ask::Create { project, batch })
}

fn person_question(world: &mut World, requester: &engine::Assignment, call: u64) -> u64 {
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"May I continue?", Box::new([]));
    delegate.executor = tasks::Executor::Person(tasks::PersonAddress::Person(1));
    delegate.authority.tools = tasks::Tools(0);
    delegate.authority.budget.spend = 1;
    let engine::CallAnswer::Delegated(made) =
        world.scripted_call(requester, call, engine::Tool::Delegate { batch: Box::new([delegate]) })
    else {
        panic!("person question delegated")
    };
    made[0].task
}

fn answer_person_question(world: &mut World, task: u64, key: u8) {
    let revision = live_task(world, task).revisions.person;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask: people::Ask::AnswerPerson {
            project: PROJECT,
            task,
            revision,
            result: people::PersonResult::Report { words: b"Go ahead.".as_slice().into() },
        },
    }));
    settle(world);
    assert!(goal_landing::ended(world, task).is_some(), "person question answered");
}

fn seed_note(world: &mut World, name: u64, role: u64, description: &[u8], body: &[u8]) {
    let scope = notes::Scope::Goal { project: PROJECT, goal: role };
    let entry = notes::Entry {
        name,
        scope: scope.clone(),
        description: description.into(),
        body: body.into(),
        references: List::with_capacity(0),
        author: notes::Author::Task { task: role, attempt: 1 },
        revision: 1,
        changed: world.env.wall,
    };
    let line = notes::Line { scope, name, description: description.into(), revision: 1 };
    world.store.seed(
        Box::new([
            Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(entry))),
            Record::Core(jig_domain_core::Record::Notes(notes::Record::Line(line))),
        ]),
        Box::new([]),
    );
}

fn status_words(world: &World) -> &str {
    let agent = run(world);
    core::str::from_utf8(&live_task(world, agent.task).spec.words).expect("UTF-8 status words")
}

#[test]
fn a_goals_status_line_counts_its_open_person_questions() {
    let (mut world, requester) = opened_chat(true, true);
    person_question(&mut world, &requester, 100);
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Land the other fix") else {
        panic!("landing chat")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    let landing = assigned_chat(&mut world, 110, people::Ask::Create { project, batch });
    land_and_look(&mut world, &landing);
    let words = status_words(&world);
    assert!(words.contains("Make the widget fix: 0 of 1 parts done, 1 question open to a person\n"), "{words}");
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn a_roles_status_counts_live_person_questions_not_its_unchanged_now() {
    const NOW: &[u8] = b"Waiting on my person's answers.";
    let (mut world, requester) = roles_world();
    let role = start_standing_role(&mut world, 110);
    assert_eq!(role.task, 41);
    let first = person_question(&mut world, &role, 100);
    let second = person_question(&mut world, &role, 101);
    park_chat(&mut world, &role);
    seed_note(&mut world, 1, role.task, b"now", NOW);
    restart_status_world(&mut world);
    land_and_look(&mut world, &requester);
    let words = status_words(&world);
    assert!(
        words.ends_with("Standing roles:\nRole 41, 2 questions open to a person:\nWaiting on my person's answers.\n"),
        "{words}"
    );
    let agent = run(&world);
    finish(&mut world, &agent);
    answer_person_question(&mut world, first, 120);
    answer_person_question(&mut world, second, 121);
    let resumed = world.assigned.iter().rev().find(|run| run.task == role.task).expect("role heard answers").clone();
    park_chat(&mut world, &resumed);
    world.env.now = world.env.now.saturating_add(Duration::from_secs(3601));
    world.env.wall = Wall::from_nanos(world.env.now.as_nanos());
    // The fake credential provider renews the forge grant across clock jumps.
    world.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
    land_numbered(&mut world, &requester, 94, 2);
    looked(&mut world, 172);
    let words = status_words(&world);
    assert!(words.ends_with("Standing roles:\nRole 41:\nWaiting on my person's answers.\n"), "{words}");
    let agent = run(&world);
    finish(&mut world, &agent);
    // Let the live branch watch hear the second landing before the restart oracle.
    super::goal_branch::observe_goal_branch(&mut world, b"main");
    world.close();
}

#[test]
fn the_status_words_carry_each_standing_roles_now_after_the_goals() {
    const NOW: &[u8] = b"Repairing the flaky test.\nWaiting for change 52.";
    let (mut world, requester) = roles_world();
    assert_eq!(standing_role(&mut world, 110), 41);
    assert_eq!(standing_role(&mut world, 111), 42);
    seed_note(&mut world, 1, 41, b"now extra", b"Not an exact description");
    seed_note(&mut world, 9, 41, b"now", b"Not the lowest-numbered Now");
    seed_note(&mut world, 4, 41, b"now", NOW);
    restart_status_world(&mut world);
    land_and_look(&mut world, &requester);
    let words = status_words(&world);
    assert!(words.ends_with(
        "Make the widget fix: 0 of 1 parts done\nOpen findings: none\nStanding roles:\nRole 41:\nRepairing the flaky test.\nWaiting for change 52.\nRole 42:\nno Now yet\n"
    ), "{words}");
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn status_nows_stop_at_a_character_edge_and_at_the_eight_lowest_roles() {
    let (mut world, requester) = roles_world();
    for key in 110..120 {
        standing_role(&mut world, key);
    }
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(240)),
        sign_in: world.signed_in.expect("owner session"),
        key: [240; 16],
        ask: people::Ask::Cancel {
            project: PROJECT,
            task: 43,
            reason: b"No longer needed".as_slice().into(),
            grace: None,
        },
    }));
    settle(&mut world);
    assert!(goal_landing::ended(&world, 43).is_some(), "role cancelled before the look");
    world.assigned.retain(|run| run.task != 43);
    let body = "界".repeat(400); // 1,200 bytes; byte 1,024 is inside a character.
    seed_note(&mut world, 1, 41, b"now", body.as_bytes());
    restart_status_world(&mut world);
    land_and_look(&mut world, &requester);
    let words = status_words(&world);
    let roles = words.split_once("Standing roles:\n").expect("roles part").1;
    let (first, rest) = roles
        .strip_prefix("Role 41:\n")
        .expect("lowest role first")
        .split_once("\nRole 42:\n")
        .expect("next role after Now");
    assert_eq!(first, "界".repeat(341)); // 1,023 complete UTF-8 bytes.
    assert!(rest.starts_with("no Now yet\nRole 44:\n"));
    let numbers: Vec<_> = roles.lines().filter(|line| line.starts_with("Role ")).collect();
    assert_eq!(
        numbers,
        ["Role 41:", "Role 42:", "Role 44:", "Role 45:", "Role 46:", "Role 47:", "Role 48:", "Role 49:"]
    );
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn a_failed_now_read_omits_only_its_role_and_still_starts_status() {
    let (mut world, requester) = roles_world();
    standing_role(&mut world, 110);
    standing_role(&mut world, 111);
    seed_note(&mut world, 1, 41, b"now", b"This read fails");
    seed_note(&mut world, 2, 42, b"now", b"Still working");
    restart_status_world(&mut world);
    land(&mut world, &requester);
    world.env.now = world.env.now.saturating_add(forge_top::SITTING);
    world.env.wall = Wall::from_nanos(world.env.now.as_nanos());
    look_request(&mut world, 171);
    let mut failed = false;
    for _ in 0..200 {
        for event in &mut world.events {
            if let engine::Event::Store(engine::StoreInput::Loaded { owner, rows, .. }) = event
                && rows.iter().any(|row| {
                    matches!(row,
                    Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(entry))) if entry.name == 1)
                })
            {
                *event = engine::Event::Store(engine::StoreInput::Unloaded { owner: *owner });
                failed = true;
            }
        }
        world.tick();
        if world.assigned.iter().any(|run| run.charter == 9) {
            break;
        }
    }
    assert!(failed, "the role's entry load was failed");
    settle(&mut world);
    let words = status_words(&world);
    assert!(!words.contains("Role 41:"), "{words}");
    assert!(words.ends_with("Standing roles:\nRole 42:\nStill working\n"), "{words}");
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn status_words_trim_roles_from_the_end_before_the_goals() {
    const BOUND: u32 = 1024;
    let (mut world, requester) = roles_world_with_spec(Some(BOUND));
    standing_role(&mut world, 110);
    standing_role(&mut world, 111);
    seed_note(&mut world, 1, 41, b"now", &vec![b'a'; 1024]);
    seed_note(&mut world, 2, 42, b"now", &vec![b'b'; 1024]);
    restart_status_world(&mut world);
    land(&mut world, &requester);
    next_sitting(&mut world, 171);
    let words = status_words(&world);
    assert_eq!(words.len(), usize::try_from(BOUND).expect("spec bound"));
    assert!(words.contains("Small fix"), "{words}");
    assert!(
        words.contains("Make the widget fix: 0 of 1 parts done\nOpen findings: none\nStanding roles:\nRole 41:\n"),
        "{words}"
    );
    assert!(!words.contains("Role 42:"));
    assert!(words.ends_with("aaa..."));
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn its_words_name_the_people_who_signed_after_the_goals() {
    for has_signers in [false, true] {
        let (mut world, requester) = opened_chat(true, true);
        land(&mut world, &requester);
        let mut saves = Vec::new();
        for (person, login, name) in [
            (4, b"sam".as_slice(), b"Sam".as_slice()),
            (7, b"seven".as_slice(), b"".as_slice()),
            (9, b"former".as_slice(), b"Former member".as_slice()),
        ] {
            saves.push(Record::Core(jig_domain_core::Record::People(people::Stored::Person {
                number: person,
                identity: people::Identity {
                    key: forge_top::user_key(1, person + 100),
                    login: login.into(),
                    name: name.into(),
                },
            })));
        }
        for record in world.store.rows.values() {
            let mut record = record.clone();
            match &mut record {
                Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: PROJECT, holdings })) => {
                    let mut members = holdings.to_vec();
                    for person in [4, 7] {
                        members.push(people::Holding { person, role: people::Role::Member });
                    }
                    *holdings = members.into_boxed_slice();
                    saves.push(record);
                }
                Record::Forge { row } => {
                    if let forge_top::Stored::LandingLog(log) = row.as_mut() {
                        for landing in &mut log.landings {
                            for change in &mut landing.changes {
                                if has_signers {
                                    change.signed = [4, 7, 9]
                                        .into_iter()
                                        .map(|person| forge_top::SignOff {
                                            person,
                                            head: landing.commit,
                                            at: landing.at,
                                        })
                                        .collect();
                                }
                            }
                        }
                        saves.push(record);
                    }
                }
                Record::Core(_) => {}
            }
        }
        let mut header = world.store.header();
        header.people = header.people.max(9);
        saves.push(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))));
        world.store.seed(saves.into_boxed_slice(), Box::new([]));
        world.restart_same();
        for event in &mut world.events {
            if let engine::Event::Worker(engine::WorkerInput::Link {
                link: engine::Link::Hello { hello, .. }, ..
            }) = event
            {
                hello.bounds.message_bytes =
                    world.env.limits.tasks.result_bytes.max(world.env.limits.tasks.message_bytes);
                hello.bounds.inbox = world.env.limits.tasks.run_inbox;
            }
        }
        settle(&mut world);
        next_sitting(&mut world, 171);
        let agent = run(&world);
        let words = core::str::from_utf8(&live_task(&world, agent.task).spec.words).expect("status words are UTF-8");
        let (_, after_goals) = words.split_once("Live goals:\n").expect("root adds goals");
        assert!(after_goals.starts_with("Make the widget fix: 1 of 1 parts done\n"), "{words}");
        if has_signers {
            assert!(
                after_goals.ends_with(
                    "People who signed:\n- person 4: Sam\n- person 7: seven\n- person 9: no longer a member\nOpen findings: none\n"
                ),
                "{words}"
            );
        } else {
            assert!(!words.contains("People who signed:"), "{words}");
        }
        finish(&mut world, &agent);
        world.close();
    }
}

#[test]
fn status_findings_follow_signers_precede_roles_and_trim_from_the_end() {
    let mut complete = String::new();
    for bound in [None, Some(1024)] {
        let (mut world, requester) = roles_world_with_spec(bound);
        let role = standing_role(&mut world, 110);
        land(&mut world, &requester);
        drain_api(&mut world);
        seed_note(&mut world, 1, role, b"now", &vec![b'z'; 1024]);
        let mut saves = Vec::new();
        for record in world.store.rows.values() {
            if let Record::Forge { row } = record
                && let forge_top::Stored::LandingLog(log) = row.as_ref()
            {
                let mut log = log.clone();
                for landing in &mut log.landings {
                    for change in &mut landing.changes {
                        change.signed =
                            Box::new([forge_top::SignOff { person: 4, head: landing.commit, at: landing.at }]);
                    }
                }
                saves.push(Record::Forge { row: Box::new(forge_top::Stored::LandingLog(log)) });
            }
        }
        for (number, title) in [(1, b'a'), (2, b'b'), (3, b'c')] {
            saves.push(Record::Forge {
                row: Box::new(forge_top::Stored::Finding(forge_top::Finding {
                    project: PROJECT,
                    number,
                    kind: forge_top::FindingKind::Qa,
                    severity: forge_top::Severity::High,
                    area: None,
                    state: forge_top::FindingState::New,
                    title: vec![title; 200].into_boxed_slice(),
                    body: Box::new([]),
                    filer: forge_top::Filer::Party(1),
                    at: world.env.wall,
                    why: None,
                    hander: None,
                    goal: None,
                    reason: None,
                    signature: None,
                    seen: None,
                    blame: None,
                    repair: None,
                    cleared: None,
                })),
            });
        }
        world.store.seed(saves.into_boxed_slice(), Box::new([]));
        restart_status_world(&mut world);
        next_sitting(&mut world, 171);
        let words = status_words(&world);
        assert!(words.contains("Make the widget fix: 1 of 1 parts done\nPeople who signed:\n- person 4: no longer a member\nOpen findings: 3\n- qa: 3 high\nHigh ones, newest first:\n"), "{words}");
        match bound {
            None => {
                assert!(
                    words.contains(&format!(
                        "- {} (qa)\n- {} (qa)\n- {} (qa)\nStanding roles:\nRole {role}:\n",
                        "c".repeat(200),
                        "b".repeat(200),
                        "a".repeat(200)
                    )),
                    "{words}"
                );
                complete = words.to_owned();
            }
            Some(bound) => {
                let bound = usize::try_from(bound).unwrap();
                assert_eq!(words.len(), bound);
                assert_eq!(words, format!("{}...", &complete[..bound - 3]));
                if bound == 1024 {
                    assert!(words.contains("Standing roles:\nRole 41:\n"), "only roles are cut: {words}");
                    assert!(words.ends_with("zzz..."));
                } else {
                    assert!(!words.contains("Standing roles:"), "findings are cut after all roles: {words}");
                    assert!(words.ends_with("bbb..."), "the findings' end is cut: {words}");
                }
            }
        }
        let agent = run(&world);
        finish(&mut world, &agent);
        world.close();
    }
}

fn milestone_record(world: &World, task: u64, name: &[u8]) -> tasks::TaskRecord {
    let mut row = live_task(world, task).clone();
    row.executor =
        tasks::Executor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    row.label = 8;
    row.tracked = Some(0);
    row.flags.ended_by = tasks::EndedBy::Requester;
    row.session = None;
    row.activation_open = false;
    row.llm = None;
    row.kept_account = None;
    row.spec.words = name.into();
    row
}

fn live_record(row: tasks::TaskRecord) -> Record {
    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(row))))
}

fn minute(hour: u64, minute: u64) -> Wall {
    // 2026-11-16, UTC, with seconds to show the words discard them.
    Wall::from_nanos((1_794_787_200 + hour * 3600 + minute * 60 + 17) * 1_000_000_000)
}

fn ended_milestone(task: u64, name: &[u8], at: Wall, outcome: forge_top::MilestoneOutcome) -> forge_top::MilestoneRow {
    forge_top::MilestoneRow {
        project: PROJECT,
        task,
        name: name.into(),
        last_landing: None,
        ended: Some(forge_top::MilestoneEnd { at, outcome, done: 5, failed: 0, closed: 1, spent: 0 }),
    }
}

fn milestone_row(row: forge_top::MilestoneRow) -> Record {
    Record::Forge { row: Box::new(forge_top::Stored::Milestone(row)) }
}

fn live_milestone_records(world: &World, goal: u64, store: u64, moves: u64, delegate: u64, moving: u64) -> Vec<Record> {
    let mut goal = live_task(world, goal).clone();
    goal.spec.words = b"S7 history in task logs".as_slice().into();
    goal.requester = tasks::Party::Task(store);
    goal.root = store;
    goal.depth = 1;
    goal.ended_delegates.done = 2;
    goal.delegates = Box::new([delegate]);
    let mut child = live_task(world, delegate).clone();
    child.requester = tasks::Party::Task(goal.number);
    child.root = store;
    child.depth = 2;
    child.label = 0;
    child.flags = tasks::Flags::PLAIN;
    let mut store = milestone_record(world, store, b"Store shape");
    store.concluded = Some(tasks::Concluded { by: 1, at: minute(21, 37) });
    store.ended_delegates.done = 7;
    store.delegates = Box::new([goal.number]);
    store.last_activity = minute(21, 38);
    let mut moves = milestone_record(world, moves, b"Moves");
    moves.ended_delegates = tasks::Endings { done: 2, failed: 1, cancelled: 0 };
    moves.delegates = Box::new([moving]);
    moves.last_activity = minute(21, 36);
    let mut moving = live_task(world, moving).clone();
    moving.requester = tasks::Party::Task(moves.number);
    moving.root = moves.number;
    moving.depth = 1;
    moving.label = 0;
    moving.flags = tasks::Flags::PLAIN;
    vec![live_record(store), live_record(moves), live_record(goal), live_record(child), live_record(moving)]
}

#[test]
fn status_names_milestone_goals_and_reports_live_and_recently_ended_milestones() {
    let (mut world, requester) = roles_world();
    let store = standing_role(&mut world, 110);
    let moves = standing_role(&mut world, 111);
    let delegate = standing_role(&mut world, 112);
    let moving = standing_role(&mut world, 113);
    let role = standing_role(&mut world, 114);
    land(&mut world, &requester);
    settle(&mut world);
    drain_api(&mut world);
    let mut saves = live_milestone_records(&world, requester.task, store, moves, delegate, moving);
    let mut search = ended_milestone(200, b"Search", minute(21, 40), forge_top::MilestoneOutcome::Complete);
    search.last_landing = Some(forge_top::MilestoneLanding { at: minute(21, 40), goal: 198, change: 199 });
    let old = ended_milestone(201, b"Old", minute(19, 59), forge_top::MilestoneOutcome::Complete);
    let mut facts = forge_top::PersonBrief { party: 1, project: PROJECT, brief: None, running: None, unpaid: None };
    facts.brief = Some(forge_top::Brief {
        sentence: b"Earlier brief".as_slice().into(),
        lede: Box::new([]),
        at: minute(20, 0),
        landed: 0,
        findings: None,
    });
    saves.extend([
        milestone_row(forge_top::MilestoneRow {
            project: PROJECT,
            task: store,
            name: b"Store shape".as_slice().into(),
            last_landing: Some(forge_top::MilestoneLanding { at: minute(21, 38), goal: 10, change: 11 }),
            ended: None,
        }),
        milestone_row(search),
        milestone_row(old),
        Record::Forge { row: Box::new(forge_top::Stored::PersonBrief(facts)) },
    ]);
    for record in world.store.rows.values() {
        if let Record::Forge { row } = record
            && let forge_top::Stored::LandingLog(log) = row.as_ref()
        {
            let mut log = log.clone();
            for landing in &mut log.landings {
                for change in &mut landing.changes {
                    landing.at = minute(21, 39);
                    change.signed = Box::new([forge_top::SignOff { person: 4, head: landing.commit, at: landing.at }]);
                }
            }
            saves.push(Record::Forge { row: Box::new(forge_top::Stored::LandingLog(log)) });
        }
        // The session remains live on the examples' wall date.
        if let Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { number, person, .. })) = record {
            saves.push(Record::Core(jig_domain_core::Record::People(people::Stored::SignIn {
                number: *number,
                person: *person,
                expires: minute(23, 0),
            })));
        }
    }
    world.store.seed(saves.into_boxed_slice(), Box::new([]));
    restart_status_world(&mut world);
    // The sitting is two hours after the previous brief.
    world.env.now = world.env.now.saturating_add(forge_top::SITTING);
    world.env.wall = minute(22, 0);
    looked(&mut world, 171);
    let words = status_words(&world);
    assert!(words.contains("Live goals:\nStore shape › S7 history in task logs: 2 of 3 parts done\nMilestones:\n- Store shape: whole; 7 of 8 goals done; last landing 2026-11-16T21:38Z\n- Moves: open; 2 of 4 goals done, 1 failed\n- Search: complete at 2026-11-16T21:40Z; 5 of 5 goals done, 1 closed; last landing 2026-11-16T21:40Z\nPeople who signed:\n- person 4: no longer a member\nOpen findings: none\nStanding roles:\n"), "{words}");
    assert!(!words.contains("Old"), "{words}");
    assert!(words.ends_with(&format!("Role {role}:\nno Now yet\n")), "{words}");
    let agent = run(&world);
    finish(&mut world, &agent);
    world.close();
}

#[test]
fn status_milestones_keep_eight_live_then_eight_ended_newest_first() {
    let mut complete = String::new();
    for bound in [None, Some(900)] {
        let (mut world, requester) = roles_world_with_spec(bound);
        land(&mut world, &requester);
        settle(&mut world);
        let prototype = standing_role(&mut world, 110);
        let mut header = world.store.header();
        let mut saves = Vec::new();
        for n in 0..9u8 {
            let activity = 8 - n;
            let mut row = milestone_record(&world, prototype, format!("Live {activity}").as_bytes());
            if n != 0 {
                header.tasks += 1;
                row.number = header.tasks;
                row.root = row.number;
                // Open gatherings need no run allotment or agent execution history.
                row.numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
                row.authority.budget.spend = 0;
                row.run_spent = 0;
                row.run_reserved = 0;
            }
            row.last_activity = minute(21, u64::from(activity));
            saves.push(live_record(row));
            let mut row = ended_milestone(
                208 - u64::from(n),
                format!("Ended {n}").as_bytes(),
                minute(21, u64::from(n)),
                if n == 8 { forge_top::MilestoneOutcome::Ended } else { forge_top::MilestoneOutcome::Closed },
            );
            if n == 8 {
                row.ended.as_mut().unwrap().failed = 1;
            }
            saves.push(milestone_row(row));
        }
        saves.push(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))));
        world.store.seed(saves.into_boxed_slice(), Box::new([]));
        restart_status_world(&mut world);
        next_sitting(&mut world, 171);
        let words = status_words(&world);
        if let Some(bound) = bound {
            let bound = usize::try_from(bound).expect("spec bound");
            assert_eq!(words.len(), bound);
            assert_eq!(words, format!("{}...", &complete[..bound - 3]));
            assert!(words.contains("Milestones:\n- Live 8:"), "{words}");
            assert!(!words.contains("Open findings:"), "the end is trimmed first: {words}");
        } else {
            complete = words.to_owned();
            let milestones =
                words.split_once("Milestones:\n").expect("milestone part").1.split_once("Open findings:").unwrap().0;
            let lines: Vec<_> = milestones.lines().collect();
            assert_eq!(lines.len(), 16, "{words}");
            for (line, n) in lines[..8].iter().zip((1..9).rev()) {
                assert_eq!(*line, format!("- Live {n}: open; 0 of 0 goals done"));
            }
            assert_eq!(lines[8], "- Ended 8: ended at 2026-11-16T21:08Z; 5 of 6 goals done, 1 failed, 1 closed");
            for (line, n) in lines[9..].iter().zip((1..8).rev()) {
                assert_eq!(*line, format!("- Ended {n}: closed at 2026-11-16T21:0{n}Z; 5 of 5 goals done, 1 closed"));
            }
        }
        let agent = run(&world);
        finish(&mut world, &agent);
        world.close();
    }
}

#[test]
fn the_run_holds_no_tools_no_grants_and_no_workspace() {
    let mut world = ready(true);
    let agent = run(&world);
    let task = live_task(&world, agent.task);
    assert_eq!(task.authority.tools.0, 0);
    assert!(task.authority.grants.is_empty());
    assert_eq!(task.authority.notes.0, 0);
    assert!(task.authority.note_resources.is_empty());
    assert!(task.authority.delegation.kinds.is_empty());
    assert_eq!(task.authority.delegation.tasks, 0);
    assert_eq!(task.authority.delegation.depth, 0);
    assert!(agent.workspace.repositories.is_empty());
    finish(&mut world, &agent);
    world.close();
}
#[test]
fn its_task_is_unlisted_and_asks_nobody() {
    let mut world = ready(true);
    let agent = run(&world);
    let child = live_task(&world, agent.task);
    assert_eq!(child.flags, tasks::Flags::PLAIN);
    assert_eq!(child.tracked, None);
    assert!(!child.spec.asks);
    assert_eq!(child.wake, tasks::WakePolicy::DEFAULT);
    let tasks::Party::Task(parent) = child.requester else { panic!("procedure parent") };
    assert!(!live_task(&world, parent).spec.asks);
    assert_eq!(live_task(&world, parent).flags, tasks::Flags::PLAIN);
    assert!(world.store.rows.values().all(|record| !matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if matches!(row.executor, tasks::Executor::Person(_)))));
    finish(&mut world, &agent);
    world.close();
}
#[test]
fn a_refused_run_lets_the_next_sitting_try() {
    let mut world = ready(false);
    assert!(own_brief(&world).running.is_none());
    assert!(own_brief(&world).brief.is_none());
    assert!(world.assigned.iter().all(|run| run.charter != 9));
    let before = world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
        if row.executor == (tasks::Executor::Procedure { connector: 1, code: 5 }))
        })
        .count();
    assert_eq!(before, 1, "the refused procedure still terminates");
    next_sitting(&mut world, 172);
    let after = world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
        if row.executor == (tasks::Executor::Procedure { connector: 1, code: 5 }))
        })
        .count();
    assert_eq!(after, 2);
    assert!(own_brief(&world).running.is_none());
    world.close();
}
#[test]
fn a_project_task_limit_refusal_lets_the_next_sitting_try() {
    admission_refusal_retries(true);
}

#[test]
fn a_live_task_limit_refusal_lets_the_next_sitting_try() {
    admission_refusal_retries(false);
}

fn admission_refusal_retries(project_limit: bool) {
    const TASKS: u32 = 6;
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    assert!(limits.tasks.tasks > TASKS);
    assert!(limits.tasks.project_tasks > TASKS);
    if project_limit {
        limits.tasks.project_tasks = TASKS;
    } else {
        limits.tasks.tasks = TASKS;
        limits.tasks.exemptions = limits.tasks.exemptions.min(TASKS);
        limits.people.operations = limits.people.operations.min(TASKS);
    }
    let (mut world, requester) = opened_chat_in(status_world_with_limits(true, true, limits));
    land(&mut world, &requester);
    settle(&mut world);
    let mut fillers = Vec::new();
    for key in 180..186 {
        let live = world
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(record,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
            if row.project == PROJECT)
            })
            .count();
        if live == usize::try_from(TASKS).expect("fixture task bound fits usize") {
            break;
        }
        let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Keep a place until the next sitting")
        else {
            panic!("filler task template")
        };
        let filler = &mut batch[0];
        filler.executor = people::TaskExecutor::Person(people::PersonAddress::Role(0));
        filler.flags.conversation = false;
        filler.budget = people::Budget::Named;
        filler.authority.spend = 32;
        filler.authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: world.signed_in.expect("owner session"),
            key: [key; 16],
            ask: people::Ask::Create { project, batch },
        }));
        settle(&mut world);
        let task = world.store.header().tasks;
        assert_eq!(live_task(&world, task).executor, tasks::Executor::Person(tasks::PersonAddress::Role(0)));
        fillers.push(task);
    }
    assert!(fillers.len() >= 2, "two places can return to the status procedure and its delegate");
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|record| matches!(record, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
            .count(),
        usize::try_from(TASKS).expect("fixture task bound fits usize")
    );

    next_sitting(&mut world, 171);
    assert_eq!(own_brief(&world).running, None);
    assert_eq!(own_brief(&world).brief, None);
    assert!(world.assigned.iter().all(|run| run.charter != 9));
    assert!(world.store.rows.values().all(|record| !matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
        if row.executor == (tasks::Executor::Procedure { connector: 1, code: 5 }))));

    // Return two places without changing either admission bound.
    for (at, task) in fillers.into_iter().take(2).enumerate() {
        let key = 190_u8 + u8::try_from(at).expect("two filler cancellations");
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: world.signed_in.expect("owner session"),
            key: [key; 16],
            ask: people::Ask::Cancel {
                project: PROJECT,
                task,
                reason: b"Make room for status".as_slice().into(),
                grace: None,
            },
        }));
        settle(&mut world);
    }
    next_sitting(&mut world, 172);
    let agent = run(&world);
    assert_eq!(agent.charter, 9);
    finish(&mut world, &agent);
    assert_brief(&mut world);
    world.close();
}

#[test]
fn a_restart_mid_run_still_saves_the_brief() {
    let mut world = ready(true);
    let agent = run(&world);
    world.restart_same();
    for event in &mut world.events {
        if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = event
        {
            hello.bounds.message_bytes = world.env.limits.tasks.result_bytes.max(world.env.limits.tasks.message_bytes);
            hello.bounds.inbox = world.env.limits.tasks.run_inbox;
        }
    }
    settle(&mut world);
    assert_eq!(world.assigned.iter().filter(|run| run.charter == 9).count(), 1, "restart delegates once");
    finish(&mut world, &agent);
    assert_brief(&mut world);
    world.close();
}

#[test]
fn no_unreserved_spend_refuses_the_run_without_a_task() {
    let mut world = ready_with(true, false);
    assert_eq!(own_brief(&world).running, None);
    assert_eq!(own_brief(&world).brief, None);
    assert!(own_brief(&world).unpaid.is_some());
    let pool = world
        .root
        .tasks()
        .funding(tasks::Funder::Pool { project: PROJECT, person: 1, period: 1 })
        .expect("owner's open pool");
    assert_eq!(pool.numbers.budget, pool.numbers.reserved + pool.numbers.spent + pool.numbers.spent_below);
    assert!(world.store.rows.values().all(|record| !matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
        if matches!(row.executor, tasks::Executor::Person(_)))));
    assert!(world.store.rows.values().all(|record| !matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
        if row.executor == (tasks::Executor::Procedure { connector: 1, code: 5 }))));
    world.close();
}
#[test]
fn failed_and_empty_reports_clear_the_run_and_save_nothing() {
    for result in [
        tasks::TaskResult::Failure { reason: b"cannot summarize".as_slice().into() },
        tasks::TaskResult::Report { words: b" \n\n ".as_slice().into() },
        tasks::TaskResult::Report { words: b"\n\nNo first paragraph".as_slice().into() },
        tasks::TaskResult::Report { words: b" \r\n\r\nNo first paragraph".as_slice().into() },
    ] {
        let mut world = ready(true);
        let agent = run(&world);
        answer(&mut world, &agent, result);
        for _ in 0..200 {
            world.tick();
            if own_brief(&world).running.is_none() && world.store.pending.is_empty() {
                break;
            }
        }
        assert_eq!(own_brief(&world).running, None);
        assert_eq!(own_brief(&world).brief, None);
        world.close();
    }
}
