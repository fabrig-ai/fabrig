//! A dependent change produces on a pending change's pushed head (domain/forge.md, 8.11).
use super::goal_branch::{branch_tip, goal_request};
use super::*;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // A full change landing, including fresh reads and hold release.
const RESULT_BYTES: u32 = 384; // Two change results fit the goal's inbox.
const LANDINGS: usize = 2; // C2 and C3 each land once into g.

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
    world.env.limits.tasks.batch = 2;
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

fn ended(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
}

fn finish(world: &mut World, producer: &engine::Assignment, title: &[u8]) {
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(title.into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"Done: pushed.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn push(world: &mut World, producer: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer destination");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: base, expected: None })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 2).expect("producer push")
}

fn assert_landed_tree(world: &World, main: u64, tip: u64, c2_head: u64, c3_head: u64) {
    let tree = &world.fake.object(tip).expect("goal tip").tree;
    let pushed_tree = &world.fake.object(c3_head).expect("C3 pushed head").tree;
    assert_eq!(tree.len(), pushed_tree.len());
    for (path, content) in pushed_tree {
        assert_eq!(tree.get(path), Some(content), "the goal tip holds C3's pushed tree");
    }
    let dependency_tree = &world.fake.object(c2_head).expect("C2 pushed head").tree;
    let base_tree = &world.fake.object(main).expect("main tip").tree;
    for (path, content) in dependency_tree {
        if !base_tree.contains_key(path) {
            assert_eq!(tree.get(path), Some(content), "C2's added path is kept");
        }
    }
}

#[test]
fn a_dependent_change_starts_early_on_its_dependencys_pushed_head() {
    let mut world = world();
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    let main = branch_tip(&mut world, &branch);
    let input = smith_domain_run::HostInput::attested(
        br#"{"changes":[{"name":"c2","words":"Add C2","landing":"goal"},{"words":"Add C3","landing":"goal","after":["c2"]}]}"#.as_slice().into(),
    ).expect("bounded changes");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let event = world.protocol.call(CHANNEL, goal.task, goal.attempt, 93, call);
    world.send(event);
    world.until(Until::Ready);
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
        panic!("two changes: {:?}", world.answers)
    };
    assert_eq!(made.len(), 2);
    let (c2, c3) = (made[0].task, made[1].task);
    assert!(matches!(live_task(&world, c3).phase, tasks::Phase::Active(_)));
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    assert_eq!(change_row(&world, c3).delegate, None);
    let producer_c2 = reviews::next_assignment(&mut world, 1);
    let c2_head = push(&mut world, &producer_c2, main, b"C2");
    // Scripted workers deliver with their terminal answer. Mergeability keeps C2 pending
    // after that answer, while C3 produces and opens its own pull.
    world.fake_env.limits.settle_min = Duration::from_secs(20);
    world.fake_env.limits.settle_max = Duration::from_secs(20);
    finish(&mut world, &producer_c2, b"Add C2");
    let producer_c3 = reviews::next_assignment(&mut world, 2);
    assert!(!ended(&world, c2));
    assert_eq!(live_task(&world, producer_c3.task).requester, tasks::Party::Task(c3));
    assert_eq!(
        producer_c3.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(c2_head)]) }
    );
    assert_eq!(live_task(&world, c3).started_from.as_ref(), &[c2]);
    let files = world
        .fake
        .object(c2_head)
        .expect("dependency head")
        .tree
        .iter()
        .map(|(path, content)| raw::File { path: path.clone(), content: content.clone() })
        .collect();
    let start = fake::merge_commit(&mut world.fake, &world.fake_env.limits, main, c2_head, files, b"Merge C2")
        .expect("prepared start");
    let c3_head = push(&mut world, &producer_c3, start, b"C3");
    world.fake_env.limits.settle_min = Duration::ZERO;
    world.fake_env.limits.settle_max = Duration::ZERO;
    finish(&mut world, &producer_c3, b"Add C3");
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, c3).pull.is_some() {
            break;
        }
    }
    assert!(change_row(&world, c3).pull.is_some(), "C3 opens while C2 is pending");
    assert!(!ended(&world, c2));
    assert_eq!(live_task(&world, c3).waiting_on.as_ref(), &[c2]);
    assert!(change_row(&world, c3).change.ready_since.is_none(), "a kept change is not ready");
    assert_eq!(world.merges, 0);
    let pull = change_row(&world, c2).pull.expect("C2 opened");
    fake::check(&mut world.fake, &world.fake_env, b"org/repo", pull, Duration::ZERO);
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, c3) {
            break;
        }
    }
    assert!(ended(&world, c2), "C2 lands");
    assert!(ended(&world, c3), "C3 lands after C2");
    assert_eq!(world.merges, 2);
    let change::State::Landed { merge, .. } = change_row(&world, c3).change.state else { panic!("C3 landed") };
    let merged_tip = branch_tip(&mut world, &branch);
    assert_eq!(translate::commit(merged_tip), merge);
    let raw::Answer::Commits { commits } =
        world.external(1, raw::Op::Read(raw::Read::Commits { before: main, after: merged_tip }))
    else {
        panic!("goal history")
    };
    let change::State::Landed { merge: c2_merge, .. } = change_row(&world, c2).change.state else {
        panic!("C2 landed")
    };
    assert_eq!(commits.len(), LANDINGS, "each change contributes one squash landing");
    let c2_landing = commits.iter().find(|commit| translate::commit(commit.id) == c2_merge).expect("C2 landing");
    let c3_landing = commits.iter().find(|commit| commit.id == merged_tip).expect("C3 landing");
    assert_eq!(c2_landing.parents.as_ref(), &[main]);
    assert_eq!(c3_landing.parents.as_ref(), &[c2_landing.id]);
    assert_landed_tree(&world, main, merged_tip, c2_head, c3_head);
    assert_eq!(change_row(&world, c3).change.last_head, Some(translate::commit(c3_head)));
    world.close();
}
