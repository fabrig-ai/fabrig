//! A goal's delivery leaves its proving landing; keys and runs survive the departure
//! (temper/docs/design/domain/landing.md, 4.3, 6, 7.1 and 15–16).
use super::super::goal_branch::{branch_tip, running_goal_in};
use super::super::goal_landing::{finish_goal, land_into_goal, mine, row};
use super::*;
use temper_engine_domain_forge_landing::Proof;

const STEPS: u32 = 1200;
const WORKFLOW_LATENCY: Duration = Duration::from_secs(300);

struct ProvingGoal {
    world: World,
    goal: engine::Assignment,
    branch: Box<[u8]>,
    change: u64,
    departed: u64,
    candidate: api::Commit,
    key: [u8; 32],
    run: u64,
    main: u64,
}

fn proving_goal() -> ProvingGoal {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            slots: Some(3),
            inbox_messages: Some(16),
            inbox_bytes: Some(4096),
            stall: Some(Duration::from_secs(3600)),
            ..StoryLimits::default()
        },
    );
    // Main, the goal, the proving round and a second child's preparation coexist.
    world.fake_env.limits.limits.branches = 6;
    world.fake_env.limits.limits.commits = 32;
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            id: 2,
            name: b"org/repo".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                contexts: Box::new([]),
                cues: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            workflows: Box::new([raw::Workflow {
                file: b"gate.yml".as_slice().into(),
                jobs: Box::new([b"test".as_slice().into()]),
                latency_min: WORKFLOW_LATENCY,
                latency_max: WORKFLOW_LATENCY,
                requirements: Box::new([]),
                fails: Box::new([]),
                flaky: Box::new([]),
                report: None,
                select: None,
                silent: 0,
                passes: 1000,
                cue: None,
            }]),
            protection: None,
            hooked: false,
        },
    );
    fake::grant(&mut world.fake, b"org/repo", 1, raw::Permission::Admin);
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    fake::manual_merges(
        &mut world.fake,
        b"org/repo",
        raw::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true },
    );
    world.adopt();
    world.send(hello_prepare(3));
    let (mut world, goal, branch) = running_goal_in(world);
    let main = branch_tip(&mut world, b"main");
    land_into_goal(&mut world, &goal, &branch, 95, b"plan");
    assert_eq!(fast_forwards(&world), 1, "the first change landed only into the goal");
    assert_eq!(branch_tip(&mut world, b"main"), main);
    assert_eq!(dispatches(&world), 0, "a goal train has no test gate");
    let answer = mine(&mut world, &goal, 96, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
    let change = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if let Some(landing) = landing_rows(&world).into_iter().find(|landing| {
            landing.landing.members.iter().any(|member| member.change == change)
                && landing.entries.iter().any(|entry| entry.run.is_some_and(|run| !run.ended))
        }) {
            let member = &landing.landing.members[0];
            let Proof::Dispatched { key, .. } = member.gates[0].proof else { panic!("gate dispatched") };
            let run = landing.entries.iter().find_map(|entry| entry.run).expect("run found").id;
            let departed = landing.landing.task;
            let candidate = member.candidate.expect("prepared candidate");
            assert_eq!(landing.landing.phase, forge_top::LandingPhase::Proving);
            assert_eq!(dispatches(&world), 1);
            assert!(receipt(&world, key).is_none(), "run remains in flight");
            return ProvingGoal { world, goal, branch, change, departed, candidate, key, run, main };
        }
    }
    panic!("goal landing proves by turn: {:?}", landing_rows(&world));
}

fn dispatches(world: &World) -> usize {
    world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count()
}

fn fast_forwards(world: &World) -> usize {
    world
        .writes
        .iter()
        .filter(|write| matches!(write, api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))
        .count()
}

fn receipt(world: &World, key: [u8; 32]) -> Option<&forge_top::Receipt> {
    match world.store.rows.get(&Key::Forge(forge_top::Key::Receipt(forge_top::ReceiptKey {
        project: 1,
        repository: forge_world::REPO,
        check: forge_top::Check::Gate(b"gate".as_slice().into()),
        key,
    }))) {
        Some(Record::Forge { row }) => match row.as_ref() {
            forge_top::Stored::Receipt { receipt, .. } => Some(receipt),
            _ => None,
        },
        _ => None,
    }
}

fn landing(world: &World, task: u64) -> &forge_top::LandingRow {
    landing_rows(world).into_iter().find(|row| row.landing.task == task).expect("landing kept")
}

fn deliver_second(setup: &mut ProvingGoal, file: &[u8]) -> u64 {
    let head = land_into_goal(&mut setup.world, &setup.goal, &setup.branch, 97, file);
    for _ in 0..STEPS {
        setup.world.tick();
        if landing(&setup.world, setup.departed).landing.phase == forge_top::LandingPhase::Landed {
            break;
        }
    }
    let old = landing(&setup.world, setup.departed);
    assert_eq!(old.landing.phase, forge_top::LandingPhase::Landed, "departed round ends");
    assert_eq!(old.landing.members[0].left, Some(forge_top::LandingLeave::HeadMoved { head: translate::commit(head) }));
    assert!(old.landing.members.iter().all(|member| member.landed.is_none()), "nothing landed");
    assert_eq!(branch_tip(&mut setup.world, b"main"), setup.main, "departure leaves main unchanged");
    assert!(receipt(&setup.world, setup.key).is_none(), "departed run still in flight");
    head
}

fn assert_landed(setup: &mut ProvingGoal, second_file: &[u8], successor: u64) {
    setup.world.env.now = setup.world.env.now.saturating_add(WORKFLOW_LATENCY);
    for _ in 0..STEPS {
        setup.world.tick();
        if ended(&setup.world, setup.change) && ended(&setup.world, setup.departed) && ended(&setup.world, successor) {
            break;
        }
    }
    assert!(
        ended(&setup.world, setup.change),
        "goal's landing change ends: {:?}",
        row(&setup.world, setup.change).change.state
    );
    let observed = setup.world.external(1, raw::Op::Read(raw::Read::Run { run: setup.run }));
    assert!(receipt(&setup.world, setup.key).is_some(), "departed run's receipt kept: {observed:?}");
    assert!(ended(&setup.world, setup.departed), "departed landing releases after following its run");
    assert_eq!(landing(&setup.world, successor).landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row(&setup.world, setup.change).drift, None);
    assert_eq!(row(&setup.world, setup.change).change.repairs, 0);
    assert!(matches!(
        row(&setup.world, setup.change).change.state,
        temper_engine_domain_forge_change::State::Landed { .. }
    ));
    assert_eq!(fast_forwards(&setup.world), 3, "two children into the goal, then the whole goal into main");
    let main = setup.world.fake.branch(b"org/repo", b"main").expect("main");
    let tree = &setup.world.fake.object(main).expect("landed tree").tree;
    for file in [b"plan".as_slice(), second_file] {
        assert_eq!(tree.get(file).map(AsRef::as_ref), Some(b"added".as_slice()), "both edits land");
    }
    let receipt = receipt(&setup.world, setup.key).expect("departed run's receipt kept for its key");
    assert!(receipt.passed);
    assert_eq!(receipt.run.id, setup.run);
    assert_eq!(receipt.candidate, setup.candidate);
    finish_goal(&mut setup.world, &setup.goal);
    setup.world.close();
}

#[test]
fn a_goal_delivering_mid_proof_leaves_and_lands_its_new_head() {
    let mut setup = proving_goal();
    let head = deliver_second(&mut setup, b"implementation");
    let mut successor = None;
    for _ in 0..STEPS {
        setup.world.tick();
        if let Some(row) = landing_rows(&setup.world).into_iter().find(|row| {
            row.landing.task != setup.departed
                && row.landing.members.iter().any(|member| member.change == setup.change)
                && row.entries.iter().any(|entry| entry.run.is_some_and(|run| !run.ended))
                && matches!(row.landing.members[0].gates[0].proof, Proof::Dispatched { .. })
        }) {
            successor = Some(row.landing.task);
            let member = &row.landing.members[0];
            assert_eq!(member.head, Some(translate::commit(head)));
            assert_ne!(member.candidate, Some(setup.candidate));
            let Proof::Dispatched { key, .. } = member.gates[0].proof else { unreachable!() };
            assert_ne!(key, setup.key, "a gate-read edit moves the key");
            break;
        }
    }
    let successor = successor.expect("new head dispatches under a new key");
    assert_landed(&mut setup, b"implementation", successor);
    assert_eq!(dispatches(&setup.world), 2, "one dispatch for each key");
}

#[test]
fn a_goal_delivering_mid_proof_takes_a_departed_landings_shared_run() {
    let mut setup = proving_goal();
    let head = deliver_second(&mut setup, b"docs/guide.md");
    let mut successor = None;
    for _ in 0..STEPS {
        setup.world.tick();
        if let Some(row) = landing_rows(&setup.world).into_iter().find(|row| {
            row.landing.task != setup.departed
                && row.landing.members.iter().any(|member| member.change == setup.change)
                && row.landing.members[0].gates[0].proof == Proof::Keyed { key: setup.key }
        }) {
            successor = Some(row.landing.task);
            assert_eq!(row.landing.members[0].head, Some(translate::commit(head)));
            assert_ne!(row.landing.members[0].candidate, Some(setup.candidate));
            assert_eq!(row.landing.phase, forge_top::LandingPhase::Proving);
            break;
        }
    }
    let successor = successor.expect("different candidate waits on the same key");
    assert!(receipt(&setup.world, setup.key).is_none(), "successor waits for shared run");
    assert_eq!(dispatches(&setup.world), 1);
    setup.world.drain_cut();
    assert!(receipt(&setup.world, setup.key).is_none(), "restart cuts while the shared run is in flight");
    assert_eq!(landing(&setup.world, successor).landing.members[0].gates[0].proof, Proof::Keyed { key: setup.key });
    assert_eq!(dispatches(&setup.world), 1);
    setup.world.restart_same();
    assert_landed(&mut setup, b"docs/guide.md", successor);
    assert_eq!(landing(&setup.world, successor).landing.members[0].gates[0].proof, Proof::Passed { key: setup.key });
    assert_eq!(dispatches(&setup.world), 1, "the departed run proves both candidates, including after restart");
}
