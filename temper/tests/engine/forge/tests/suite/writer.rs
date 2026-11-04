use skein_forge::api::{Commit, Error, MergeHow, Op, Read, Write};
use skein_forge_client::{Condition, Effect, Entry, Made, Outcome, Writer};
use skein_lib::{Duration, Wall};
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{REPO, World, changes, translate};

const CHANGE: u64 = 51;
const MERGING: u64 = 1;
const CALL_DELAY: Duration = Duration::from_millis(10);
const READ_STEPS: usize = 16; // Goal reads plus restored holds and the client's fresh-read chain.

fn branch_name(branch: &[u8]) -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect()),
    }
}

fn land_into_main(world: &mut World) -> Commit {
    world.produce(b"temper/51");
    let pull = world.open_pull(b"temper/51", b"main");
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = changes::change_row();
    row.pull = Some(pull);
    row.pending = Some(MERGING);
    row.change.state = temper_engine_domain_forge_change::State::Landing { head };
    world.event(top::Event::Change { row });
    world.event(top::Event::Enqueue {
        entry: Entry {
            number: MERGING,
            task: CHANGE,
            repository: REPO,
            effect: Effect {
                write: Write::Merge { number: pull, head, how: MergeHow::Style },
                condition: Condition::Merge { base: b"main".as_slice().into() },
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    world.event(top::Event::Committed { entry: MERGING });
    world.run_for(0);
    let tip = translate::commit(world.branch(b"main"));
    assert!(
        world.seen().iter().any(|request| matches!(
            request,
            top::Request::Outcome { entry: MERGING, outcome: Outcome::Made { made: Made::Merged(commit), .. }, .. }
                if *commit == tip
        )),
        "the connector's merge lands before the goal starts"
    );
    tip
}

fn take_goal_at_tip(world: &mut World, task: u64, entry: u64, branch: &[u8], tip: Commit) {
    world.take_calls();
    world.take_seen();
    world.slow_calls(CALL_DELAY);
    world.event(top::Event::Hold { entry, task, resource: branch_name(branch), from: None });
    // Stop at the durable creation, before the outbox's next call finishes.
    for _ in 0..READ_STEPS {
        if world.stored().contains_key(&top::Key::Entry(entry)) {
            break;
        }
        world.run_to(Wall::from_nanos(world.wall().as_nanos() + CALL_DELAY.as_nanos()));
    }
    let calls = world.take_calls();
    assert!(
        calls.contains(&Op::Read(Read::Branch { branch: b"main".as_slice().into() })),
        "each missing goal reads the default branch despite its saved head: {calls:?}"
    );
    let Some(top::Stored::Entry { entry: saved, held: false }) = world.stored().get(&top::Key::Entry(entry)) else {
        panic!("goal {task} creation remains durable while its forge call is pending: {calls:?}")
    };
    assert_eq!(saved.effect.write, Write::CreateBranch { branch: branch.into(), commit: tip });
    world.slow_calls(Duration::ZERO);
    world.run_to(Wall::from_nanos(world.wall().as_nanos() + CALL_DELAY.as_nanos()));
    assert_eq!(translate::commit(world.branch(branch)), tip);
    assert!(world.take_calls().contains(&Op::Write(Write::CreateBranch { branch: branch.into(), commit: tip })));
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::Outcome { entry: found, outcome: Outcome::Made { made: Made::Branch(commit), .. }, .. }
            if *found == entry && *commit == tip
    )));
}

#[test]
fn a_goal_started_after_a_landing_reads_the_current_default_tip() {
    const BRANCHES: u32 = 5; // main, the change, and three goals.
    let mut world = World::with_branches(292, BRANCHES);
    world.adopt();
    world.slow_calls(Duration::ZERO);
    let main = branch_name(b"main");
    world.event(top::Event::Names { task: CHANGE, resources: Box::new([main.clone()]) });
    world.run_for(0);
    let old = top::BranchHead { name: main.clone(), commit: translate::commit(world.branch(b"main")), owned: false };
    let tip = land_into_main(&mut world);
    assert_ne!(old.commit, tip);
    for (task, entry, branch) in [(27, 2, b"temper/27/g27".as_slice()), (29, 3, b"temper/29/g29")] {
        assert_eq!(
            world.stored().get(&top::Key::BranchHead(main.clone())),
            Some(&top::Stored::BranchHead(old.clone()))
        );
        take_goal_at_tip(&mut world, task, entry, branch, tip);
    }
    world.restart();
    assert_eq!(world.stored().get(&top::Key::BranchHead(main)), Some(&top::Stored::BranchHead(old)));
    take_goal_at_tip(&mut world, 31, 4, b"temper/31/g31", tip);
}

#[test]
fn a_runtime_writer_is_kept_once_and_refused_writers_keep_nothing() {
    let mut world = World::writerless(104);
    let writer = Writer { forge: REPO.forge, author: 1 };
    assert!(!world.give_writer(Writer { author: 0, ..writer }));
    assert_eq!(world.writer_saves(), 0);
    assert!(world.stored().is_empty());
    assert!(world.give_writer(writer));
    assert_eq!(world.writer_saves(), 1);
    assert_eq!(world.stored().get(&top::Key::Writer(REPO.forge)), Some(&top::Stored::Writer(writer)));
    assert!(world.give_writer(writer));
    for refused in [Writer { author: 2, ..writer }, Writer { author: 0, ..writer }] {
        assert!(!world.give_writer(refused));
    }
    assert_eq!(world.writer_saves(), 1);
    assert_eq!(world.stored().len(), 1);
}

#[test]
fn a_restored_writer_precedes_the_restored_effect_on_a_writerless_restart() {
    let mut world = World::writerless(104);
    let writer = Writer { forge: REPO.forge, author: 1 };
    assert!(world.give_writer(writer));
    world.adopt();
    world.event(top::Event::Enqueue {
        entry: Entry {
            number: 1,
            task: 51,
            repository: REPO,
            effect: Effect {
                write: Write::CreateIssue {
                    key: Box::from(b"restored-writer".as_slice()),
                    title: Box::from(b"Writer restored".as_slice()),
                    body: Box::from(b"An effect after restart".as_slice()),
                },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    assert!(world.stored().contains_key(&top::Key::Entry(1)));
    assert_eq!(world.writes(), 0);
    world.take_seen();
    world.restart();
    assert_eq!(world.writer_saves(), 1, "restore emits no save");
    world.run_for(20);
    let mut made = false;
    for request in world.seen() {
        if let top::Request::Outcome { entry: 1, outcome, .. } = request {
            assert_ne!(*outcome, Outcome::Failed(Error::Forbidden), "restored effect has its writer");
            if let Outcome::Made { made: Made::Created(_), .. } = outcome {
                made = true;
            }
        }
    }
    assert!(made, "the restored effect was made");
    assert_eq!(world.writes(), 1);
    assert!(world.give_writer(writer));
    assert!(!world.give_writer(Writer { author: 2, ..writer }));
    assert_eq!(world.writer_saves(), 1, "the restored writer was already held by the client");
    assert!(!world.stored().contains_key(&top::Key::Entry(1)));
}

#[test]
fn goal_branch_reservation_blocks_later_projection() {
    use skein_lib::Duration;
    use temper_engine_domain_forge_issues::{GoalView, Phase};
    let mut world = World::new(166);
    world.adopt();
    let tip = world.branch(b"main");
    let name = top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([
            Box::from(b"temper".as_slice()),
            Box::from(b"1".as_slice()),
            Box::from(b"g1".as_slice()),
        ])),
    };
    world.slow_calls(Duration::from_secs(2));
    world.event(top::Event::Names { task: 1, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { entry: 1, task: 1, resource: name, from: None });
    world.event(top::Event::Project {
        entry: 2,
        repository: REPO,
        view: GoalView {
            phase: Phase::Waiting,
            goal: 1,
            repository: u64::from(REPO.repository),
            title: Box::from("Goal"),
            goal_text: Box::from("Plan"),
            plan: Box::new([]),
            milestones: Box::new([]),
            finished: None,
        },
    });
    world.event(top::Event::Committed { entry: 2 });
    world.run_for(1);
    assert!(world.stored().contains_key(&top::Key::Entry(2)), "projection remains pending during the read");
    world.run_for(30);
    let mut completed = Vec::new();
    for request in world.seen() {
        if let top::Request::Outcome { entry: 1, outcome: Outcome::Made { .. }, .. } = request {
            completed.push(1);
        }
        if let top::Request::ProjectionOutcome { entry: 2, outcome: Outcome::Made { .. } } = request {
            completed.push(2);
        }
        assert!(!matches!(request, top::Request::MakeFailed { .. } | top::Request::ProjectionFailed { .. }));
    }
    assert_eq!(completed, [1, 2], "branch completes before the later projection");
    assert_eq!(world.branch(b"temper/1/g1"), tip);
    assert_eq!(world.writes(), 2);
    assert!(!world.stored().contains_key(&top::Key::Entry(1)));
    assert!(!world.stored().contains_key(&top::Key::Entry(2)));
}
