//! Home's status agent through the root and its durable document (temper/docs/design/domain/forge/attention.md, 3).
use super::*;
use skein_lib::Wall;

const CHANNEL: Token = Token::new(7);
const PROJECT: u32 = 1;
const SENTENCE: &[u8] = b"The widget fix has landed.";
const REPORT: &[u8] = b"The widget fix has landed.\n\nThe review passed and the branch is ready.";

fn status_world(allowed: bool, funded: bool) -> World {
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
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    // The same signed-in person returns for the next sitting.
    limits.people.sign_in_lifetime = Duration::from_secs(7200);
    let mut defaults = temper_engine_service::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    // Scripted workers use the same small run bounds as the other forge stories.
    for charter in &mut defaults.charters {
        let number = charter.number;
        *charter = walking::charter();
        charter.number = number;
    }
    if !funded {
        let policy = &mut defaults.policies.get_mut(&1).expect("stock project policy").policy;
        policy.period_spend = policy.roles[0].period_spend;
        // This world's calendar opens its period from the rules' spend.
        defaults.rules.period_spend = policy.period_spend;
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
    World::configured_stock_limits(&defaults, &model, &limits)
}
fn settle(world: &mut World) {
    world.until(Until::Ready);
    assert!(world.store.pending.is_empty());
}
fn looked(world: &mut World, number: u8) {
    world.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: world.signed_in.expect("owner session"),
        key: [number; 16],
        scope: people::ApplicationScope::Project(PROJECT),
        digest: [number; 32],
        carried: None,
        request: engine::Application::Looked { project: PROJECT },
    }));
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
    let mut world = status_world(allowed, funded);
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
    let change = super::questions::make_change(world, requester, 93, engine::ChangeQuestions::None);
    let producer = super::questions::next_producer(world, change);
    let branch = super::questions::row(world, change).branch.clone();
    assert!(world.assigned.iter().all(|run| run.charter != 9), "the first look never spends");
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"widget fixed", 1)
        .expect("producer pushed");
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
    let reviewer = super::reviews::next_assignment(world, 2);
    answer(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    for _ in 0..200 {
        world.tick();
        if world.store.rows.contains_key(&Key::Forge(forge_top::Key::ProjectFacts { project: PROJECT }))
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(facts(world).landed, 1);
    next_sitting(world, 171);
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
    answer(world, run, tasks::TaskResult::Report { words: REPORT.into() });
    for _ in 0..200 {
        world.tick();
        if facts(world).running.is_none() && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("status report heard; facts={:?}; task={:?}", facts(world), live_task(world, run.task).phase);
}
fn assert_brief(world: &mut World) {
    let brief = facts(world).brief.as_ref().expect("saved status brief");
    assert_eq!(brief.sentence.as_ref(), SENTENCE);
    assert_eq!(brief.lede.as_ref(), b"The review passed and the branch is ready.");
    assert_eq!(brief.landed, 1);
    assert_eq!(facts(world).running, None);
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
    assert!(words.contains("Make the widget fix: 0 of 1 delegates done"), "{words}");
    assert!(words.len() <= usize::try_from(world.env.limits.tasks.spec_bytes).expect("spec bound fits usize"));
    finish(&mut world, &agent);
    assert_brief(&mut world);
    world.close();
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
    assert!(facts(&world).running.is_none());
    assert!(facts(&world).brief.is_none());
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
    assert!(facts(&world).running.is_none());
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
    assert_eq!(facts(&world).running, None);
    assert_eq!(facts(&world).brief, None);
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
            if facts(&world).running.is_none() && world.store.pending.is_empty() {
                break;
            }
        }
        assert_eq!(facts(&world).running, None);
        assert_eq!(facts(&world).brief, None);
        world.close();
    }
}
