//! A goal's dependent changes land under its branch's rules and keep their reports
//! (temper/docs/design/domain/agent.md, 4.3.1; temper/docs/design/domain/forge.md, 8.2, 8.7 and 11.1).
use super::goal_branch::{branch_tip, forge_hold, goal_request};
use super::*;

const CHANNEL: Token = Token::new(7);
const GOAL_REQUEST: u8 = 92;
const CHANGE_CALL: u64 = 93;
const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const TWO_CHANGES: usize = 2;
const GOAL_AND_A: usize = 2;
const GOAL_AND_BOTH_PRODUCERS: usize = 3;
// Two result reservations leave room for the goal's messages in its 1024-byte inbox.
const RESULT_BYTES: u32 = 384;
const A_REPORT: &[u8] = b"Done: added A.\n## Left\n- none";
const B_REPORT: &[u8] = b"Done: added B.\n## Left\n- none";

fn world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(RESULT_BYTES), ..StoryLimits::default() },
    );
    world.env.limits.tasks.batch = u32::try_from(TWO_CHANGES).expect("two changes fit u32");
    let segments = [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from);
    world = world.with_landing_deployment(Box::new([
        engine::LandingRule {
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.clone().into(),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.into(),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ]));
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    world
}

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn finish(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
    let event = world.protocol.answer(
        CHANNEL,
        run.task,
        run.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
}

fn push(world: &mut World, producer: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer's destination");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 2)
        .expect("producer pushed its change");
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state: raw::Check::Pending,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
    head
}

fn producer_result(title: &[u8], report: &[u8]) -> tasks::TaskResult {
    tasks::TaskResult::Change {
        title: tasks::ChangeTitle::new(title.into()),
        connector: 0,
        kind: 2,
        resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        words: report.into(),
    }
}

fn assert_landed(world: &mut World, goal: u64, change: u64, base: u64, head: u64, report: &[u8]) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, change).is_some() {
            break;
        }
    }
    let row = change_row(world, change);
    assert!(
        matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }),
        "change {change}: {row:?}"
    );
    assert_eq!(row.report.as_ref(), report);
    assert_eq!((row.change.updates, row.change.repairs, row.change.resolutions), (0, 0, 0));
    let number = row.pull.expect("landed pull");
    let branch = row.base.clone();
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) =
        &ended(world, change).expect("change ended").phase
    else {
        panic!("change ended done")
    };
    let words = core::str::from_utf8(words).expect("UTF-8 change result");
    let landing = format!(
        "Landed: pull request #{number} merged into {} as ",
        core::str::from_utf8(&branch).expect("goal branch")
    );
    let lines: Vec<_> = words.lines().collect();
    assert!(lines[2].starts_with(&landing), "{words}");
    assert_eq!(lines[3], "Report:");
    assert_eq!(&lines[4..7], core::str::from_utf8(report).expect("UTF-8 report").lines().collect::<Vec<_>>());
    assert!(
        live_task(world, goal).inbox.iter().any(|message| {
            message.from == tasks::Party::Task(change)
                && matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
                && message.words.as_ref() == words.as_bytes()
        }),
        "the goal receives its change's report"
    );
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number: pull, head: merged, how: skein_forge::api::MergeHow::Style }
            if *pull == number && *merged == translate::commit(head))));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("landed pull")
    };
    assert_eq!(pull.base, branch);
    assert_eq!(pull.commit, head, "the pending head lands without an update");
    assert!(
        pull.statuses.iter().any(|status| status.context.as_ref() == b"build" && status.state == raw::Check::Pending)
    );
    let merge = pull.merged.expect("merged with CI still pending");
    let raw::Answer::Commits { commits } =
        world.external(1, raw::Op::Read(raw::Read::Commits { before: base, after: merge }))
    else {
        panic!("merge ancestry")
    };
    let commit = commits.iter().find(|commit| commit.id == merge).expect("merge commit");
    assert_eq!(commit.parents.as_ref(), &[base, head], "the landing is a merge commit at the producer's head");
    assert_eq!(branch_tip(world, &branch), merge);
    assert_eq!(forge_hold(world, &branch).expect("goal still holds its branch").task, goal);
    merge
}

#[test]
#[ignore = "the fake forge squashes Style merges; a goal's landing requires a merge commit (C5)"]
fn a_goal_makes_two_changes_into_its_branch_the_second_after_the_first() {
    let mut world = world();
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    let row = live_task(&world, goal.task);
    assert_eq!(row.tracked, Some(0));
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })));
    assert_eq!(forge_hold(&world, &branch).expect("G holds g").task, goal.task);
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, &branch), main, "g is made at main's tip");

    let input = smith_domain_run::HostInput::attested(
        br#"{"changes":[{"name":"a","words":"Add A","landing":"goal"},{"words":"Add B","landing":"goal","after":["a"]}]}"#.as_slice().into(),
    ).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let event = world.protocol.call(CHANNEL, goal.task, goal.attempt, CHANGE_CALL, call);
    world.send(event);
    world.until(Until::Ready);
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
        panic!("both changes admitted: {:?}; {:?}", world.answers, world.call_errors)
    };
    assert_eq!(made.len(), TWO_CHANGES);
    let (a, b) = (made[0].task, made[1].task);
    assert_eq!(live_task(&world, a).spec.words.as_ref(), b"Add A");
    assert_eq!(live_task(&world, b).spec.words.as_ref(), b"Add B");
    assert_eq!(live_task(&world, b).dependencies.as_ref(), &[a]);
    let producer_a = reviews::next_assignment(&mut world, 1);
    world.until(Until::Ready);
    assert_eq!(world.assigned.len(), GOAL_AND_A, "B waits while a worker slot is free");
    assert_eq!(live_task(&world, producer_a.task).requester, tasks::Party::Task(a));
    assert_eq!(change_row(&world, a).base, branch);
    let a_head = push(&mut world, &producer_a, main, b"A");
    let moved_main = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"main moved", 1)
        .expect("main advances while G's first change is being produced");
    finish(&mut world, &producer_a, producer_result(b"Add A", A_REPORT));
    let a_merge = assert_landed(&mut world, goal.task, a, main, a_head, A_REPORT);

    let producer_b = reviews::next_assignment(&mut world, 2);
    assert!(ended(&world, a).is_some(), "B starts only after A is done");
    assert_eq!(live_task(&world, producer_b.task).requester, tasks::Party::Task(b));
    assert_eq!(change_row(&world, b).base, branch);
    let b_head = push(&mut world, &producer_b, a_merge, b"B");
    finish(&mut world, &producer_b, producer_result(b"Add B", B_REPORT));
    let b_merge = assert_landed(&mut world, goal.task, b, a_merge, b_head, B_REPORT);
    assert_eq!(world.merges, 2);
    assert_eq!(world.assigned.len(), GOAL_AND_BOTH_PRODUCERS, "neither change asks a review or repair");
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    assert_eq!(branch_tip(&mut world, b"main"), moved_main);

    finish(&mut world, &goal, tasks::TaskResult::Report { words: b"Done: both changes landed.".as_slice().into() });
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, goal.task).is_some() {
            break;
        }
    }
    assert!(matches!(ended(&world, goal.task).expect("G ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    world.until(Until::Ready);
    assert!(forge_hold(&world, &branch).is_none(), "G releases its hold as it ends");
    assert_eq!(branch_tip(&mut world, &branch), b_merge, "g survives G's ending");
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: deleted } if *deleted == branch)));
    world.close();
}
