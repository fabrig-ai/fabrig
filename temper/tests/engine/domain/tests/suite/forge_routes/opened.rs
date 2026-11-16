//! Opened changes delegate their gates and trouble as any change (forge/change/opened.md, 5).
use super::light_rules::change_row;
use super::*;
use temper_engine_domain_forge_change as change;

const SOURCE: &[u8] = b"docs/x";
const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000;

fn open(world: &mut World, connector: u16, spend: u64) -> (u64, u64) {
    world.external(2, raw::Op::Git(raw::Git::Create { branch: SOURCE.into(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", SOURCE, b"file", b"fixed", 2)
        .expect("person pushed source");
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Opened from docs/x") else { panic!("batch") };
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Procedure { connector, code: 2 };
    member.contract = people::Contract::Change { connector, kind: 1, words: world.env.limits.tasks.result_bytes };
    member.parameters = Box::new([
        people::Parameter::Resource {
            name: 1,
            connector,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        people::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
        people::Parameter::Bytes { name: 10, value: b"Opened fix".as_slice().into() },
        people::Parameter::Bytes { name: 19, value: SOURCE.into() },
    ]);
    member.symbols = Box::new([]);
    member.tracked = None;
    member.flags = people::Flags {
        ended_by: people::EndedBy::Result,
        conversation: false,
        listed: true,
        retention: people::Retention::Kept,
    };
    member.budget = people::Budget::Named;
    member.authority.spend = spend;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::Create { project, batch },
    }));
    for _ in 0..STEPS {
        world.tick();
        if let Some(outcome) = world.store.rows.values().find_map(|row| match row {
            Record::Core(jig_domain_core::Record::People(people::Stored::Answer { key, outcome, .. }))
                if key.key == [92; 16] =>
            {
                Some(outcome)
            }
            _ => None,
        }) {
            let people::Outcome::Created { first, last, .. } = outcome else { panic!("opened: {outcome:?}") };
            assert_eq!(first, last);
            return (*first, head);
        }
    }
    panic!("person's batch did not answer")
}

fn delegate(world: &mut World, task: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(run) = world.assigned.first() {
            return run.clone();
        }
    }
    panic!("delegate missing for change {task}: {:?}", world.store.change(task))
}

fn trouble_world(red: bool) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            create_branches: true,
            forge_charter: Some(2),
            dispatch_tests: red.then_some(b"file"),
            ..StoryLimits::default()
        },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    world
}

fn bounded_delegate(world: &World, task: u64, run: &engine::Assignment, head: u64) {
    let row = change_row(world, task);
    assert_eq!(row.source, Some(forge_top::ChangeSource { branch: SOURCE.into(), at: None }));
    assert_eq!(row.kept, Some(translate::commit(head)));
    assert_eq!((row.producing_charter, row.review_charter), (2, 4));
    assert_eq!(run.charter, row.producing_charter);
    let child = live_task(world, run.task);
    assert_eq!(child.executor, tasks::Executor::Agent { charter: row.producing_charter });
    assert_eq!(child.requester, tasks::Party::Task(task));
    let parent = live_task(world, task);
    let remaining = parent.numbers.budget - parent.numbers.spent - parent.numbers.spent_below;
    assert!(child.authority.budget.spend > 0);
    assert!(child.authority.budget.spend <= remaining);
    assert!(parent.spec.parameters.iter().all(|parameter| !matches!(
        parameter,
        tasks::Parameter::Bytes { name: 11, .. } | tasks::Parameter::Number { name: 11, .. }
    )));
}

#[test]
fn an_opened_changes_prepare_conflict_delegates_a_bounded_resolution_under_the_forges_charter() {
    let mut world = trouble_world(false);
    world.manual_prepare_jobs();
    let (task, head) = open(&mut world, 0, 100);
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base conflict", 2)
        .expect("base moved");
    let job = catch_up::next_job(&mut world, task);
    assert_eq!(job.kind, jig_domain_link::JobKind(1));
    catch_up::finish_job(
        &mut world,
        &job,
        br#"{"conflicts":[{"member":1,"with":"branch","files":["file"],"cut":false}]}"#,
    );
    let run = delegate(&mut world, task);
    bounded_delegate(&world, task, &run, head);
    let row = change_row(&world, task);
    assert_eq!(row.delegate, Some((run.task, change::Delegate::Resolve { base: translate::commit(base) })));
    assert_eq!(row.change.resolutions, 1);
    assert_eq!(
        run.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: row.branch.clone(), base: translate::commit(base) }
    );
    assert_eq!(run.workspace.repositories[0].push.as_deref(), Some(row.branch.as_ref()));
}

#[test]
fn an_opened_changes_blamed_red_delegates_a_bounded_repair_under_the_forges_charter() {
    let mut world = trouble_world(true);
    let (task, head) = open(&mut world, 0, 100);
    for _ in 0..STEPS {
        world.prepare_tick();
        if !world.assigned.is_empty() {
            break;
        }
    }
    let run = world.assigned.first().expect("repair assigned after red proof").clone();
    bounded_delegate(&world, task, &run, head);
    let row = change_row(&world, task);
    assert!(matches!(row.delegate, Some((child, change::Delegate::Repair(_))) if child == run.task));
    assert_eq!(row.change.repairs, 1);
    assert!(world.store.rows.values().any(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Landing(row) if row.landing.members.iter().any(|member| member.change == task && matches!(member.left, Some(temper_engine_domain_forge_landing::Leave::Failed { .. } | temper_engine_domain_forge_landing::Leave::Blamed { .. })))))));
    assert_eq!(run.workspace.repositories[0].start, engine::ForgeStart::Branch(row.branch.clone()));
    assert_eq!(world.merges, 0);
}

#[test]
fn an_opened_change_waits_for_stocks_gate_one_review_then_lands() {
    let defaults = goal_branch::stock_defaults();
    let model = stock::box_model();
    let mut world =
        World::configured_stock_turn(&defaults, &model, &temper_engine_domain_world::forge_routes::box_limits());
    world.adopt();
    let mut bounds = protocol::bounds();
    bounds.turns = 800;
    bounds.time = Duration::from_secs(28_800);
    bounds.spend = defaults.rules.maximum_run_spend;
    bounds.message_bytes = world.env.limits.tasks.message_bytes.max(world.env.limits.tasks.result_bytes);
    bounds.inbox = world.env.limits.tasks.run_inbox;
    bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    let mut hello = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: offer, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    offer.bounds = bounds;
    world.send(hello);
    let (task, head) = open(&mut world, 1, 43_600);
    let reviewer = delegate(&mut world, task);
    assert_eq!(reviewer.charter, 4);
    let row = change_row(&world, task);
    assert_eq!((row.producing_charter, row.review_charter), (2, 4));
    assert_eq!(row.kept, Some(translate::commit(head)));
    assert_eq!(
        row.delegate,
        Some((reviewer.task, change::Delegate::Gate { number: 1, head: translate::commit(head) }))
    );
    assert!(row.change.gates.iter().any(|gate| gate.number == 1
        && gate.blocking
        && gate.kind == change::GateKind::Agent
        && gate.freshness == change::Freshness::Clean));
    assert_eq!(live_task(&world, reviewer.task).executor, tasks::Executor::Agent { charter: row.review_charter });
    for _ in 0..10 {
        world.prepare_tick();
    }
    assert_eq!(world.merges, 0, "no landing before the review approves");
    reviews::finish_review(
        &mut world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    for _ in 0..STEPS {
        world.prepare_tick();
        if goal_landing::ended(&world, task).is_some() {
            break;
        }
    }
    assert!(
        matches!(change_row(&world, task).change.state, change::State::Landed { .. }),
        "{:?}",
        change_row(&world, task)
    );
    assert_eq!(world.assigned.len(), 1, "no producing task and no second review of a clean prefix");
    assert!(goal_landing::ended(&world, task).is_some(), "opened change ended");
    world.close();
}
