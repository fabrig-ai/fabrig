//! A goal's delivery lands by a change while its run keeps writing
//! (temper/docs/design/domain/forge.md, 8.1, 8.5 and 11.1;
//! temper/docs/design/domain/agent.md, 4.3 and 4.3.1).
use super::goal_branch::{branch_tip, forge_hold, goal_request, writer};
use super::goal_landing::{assert_goal_holds, ended, finish_run, land_into_goal, mine, row, status};
use super::*;
use temper_engine_domain_forge_change::State;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const WORKER_SLOTS: u32 = 3; // Room for the goal, a sibling producer and an early producer.
const RESULT_BYTES: u32 = 224; // Three results, a landing subscription and 128 bytes of core news fit 1024 bytes.
const GOAL_REQUEST: u8 = 92;
const SIBLING_CALL: u64 = 93;
const DELIVERY_CALL: u64 = 94;
const DEPENDENT_CALL: u64 = 95;
const SIBLING_MERGES: u32 = 1;
const DELIVERY_MERGES: u32 = 2; // The sibling and C1 each land into g.
const DELIVERY_FILE: &[u8] = b"design";
const DELIVERY_CONTENT: &[u8] = b"the design";

fn running_goal() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { slots: Some(WORKER_SLOTS), result_bytes: Some(RESULT_BYTES), ..StoryLimits::default() },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: authority::Last::Open(b"temper".as_slice().into()),
        },
        ci: true,
        // The sibling moves g; this story lands the delivery without asking for an update of r.
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
}

fn await_end(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            return;
        }
    }
    panic!("task {task} ended")
}

#[test]
#[ignore = "a goal's run delivers once goal-delivery lands: temper/docs/design/domain/forge.status.md"]
fn a_goals_delivery_lands_into_its_goal_branch_by_a_change() {
    let (mut world, goal, branch) = running_goal();

    // 1. Delivering leaves the goal's report contract intact.
    let charter = temper_engine_smith::charter(&goal);
    assert!(charter.grants.deliver.is_some(), "the goal's charter grants deliver");
    assert!(charter.grants.tools.modify);
    let deliver = charter.grants.deliver.as_ref().expect("delivery grant");
    assert!(deliver.checks_must_pass);
    assert_eq!(
        deliver.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(),
        [b"title".as_slice(), b"body"]
    );
    assert!(charter.outcome.change.is_none());
    assert!(charter.outcome.report.is_some());

    // 2. The run writes r and leaves g's slot free for merges.
    let run_branch = format!("temper/{}/r{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    assert_eq!(goal.workspace.repositories[0].start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(goal.workspace.repositories[0].push, Some(run_branch.clone()));
    let goal_writer = Some(tasks::Writer::Run { task: goal.task, attempt: goal.attempt });
    assert_eq!(writer(&world, &run_branch), goal_writer);
    assert_eq!(writer(&world, &branch), None);

    // 3. The scripted worker pushes H1 and hints its run branch.
    let base = branch_tip(&mut world, &branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: run_branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &run_branch, DELIVERY_FILE, DELIVERY_CONTENT, 1)
            .expect("the goal pushes its delivery");
    assert_ne!(head, base);
    assert_eq!(branch_tip(&mut world, &run_branch), head);
    status(&mut world, head, raw::Check::Pending);
    early_start::observe_push(&mut world, &goal);

    // 4. A sibling lands into g while the goal keeps its run slot.
    let sibling_head = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"sibling");
    assert_ne!(sibling_head, base);
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_eq!(writer(&world, &run_branch), goal_writer);
    assert_goal_holds(&world, goal.task, &branch);

    // 5. C1 lands the run branch without making a producer.
    let assigned = world.assigned.len();
    let answer =
        mine(&mut world, &goal, DELIVERY_CALL, br#"{"words":"Land the design","branch":"run","landing":"goal"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("C1 delegated: {answer:?}") };
    let c1 = made[0].task;
    assert_eq!(row(&world, c1).branch, run_branch);
    assert_eq!(row(&world, c1).base, branch);
    assert_eq!(row(&world, c1).delegate, None);
    assert_eq!(world.assigned.len(), assigned, "C1 has no producer");

    // 6. C2 starts early from H1 while C1 waits for its checks.
    let input = format!(r#"{{"words":"Build on the design","landing":"goal","after":[{c1}]}}"#);
    let answer = mine(&mut world, &goal, DEPENDENT_CALL, input.as_bytes());
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("C2 delegated: {answer:?}") };
    let c2 = made[0].task;
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(c2));
    assert_eq!(
        producer.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(head)]) }
    );
    assert_eq!(live_task(&world, c2).started_from.as_ref(), &[c1]);
    assert!(ended(&world, c1).is_none(), "C2 starts before C1 lands");

    // 7. C1 lands H1 while the goal's run still writes r.
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        assert_eq!(writer(&world, &run_branch), goal_writer);
        if ended(&world, c1).is_some() {
            break;
        }
    }
    assert!(ended(&world, c1).is_some(), "C1 closes while the goal runs");
    assert!(matches!(row(&world, c1).change.state, State::Landed { .. }), "{:?}", row(&world, c1));
    assert_eq!(row(&world, c1).change.last_head, Some(translate::commit(head)));
    assert_eq!(world.merges, DELIVERY_MERGES);
    let tip = branch_tip(&mut world, &branch);
    let tree = &world.fake.object(tip).expect("goal tip").tree;
    assert_eq!(tree.get(DELIVERY_FILE).map(AsRef::as_ref), Some(DELIVERY_CONTENT));
    assert_eq!(tree.get(b"sibling".as_slice()).map(AsRef::as_ref), Some(b"added".as_slice()));

    // 8. C1's close keeps the run branch and the goal's holds.
    world.until(Until::Ready);
    assert_eq!(branch_tip(&mut world, &run_branch), head);
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: deleted } if *deleted == run_branch)));
    assert_eq!(forge_hold(&world, &run_branch).expect("run branch hold").task, goal.task);
    assert_goal_holds(&world, goal.task, &branch);

    // 9. The early producer completes, then the goal finishes with a report.
    let base = branch_tip(&mut world, &branch);
    let implementation_head = early_start::push(&mut world, &producer, base, b"implementation");
    status(&mut world, implementation_head, raw::Check::Passed);
    finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Build on the design".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    await_end(&mut world, c2);
    let report = tasks::TaskResult::Report { words: b"Done".as_slice().into() };
    finish_run(&mut world, &goal, report.clone());
    await_end(&mut world, goal.task);
    assert_eq!(ended(&world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(report)));
    world.close();
}
