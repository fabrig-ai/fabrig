use skein_forge::api::{Error, Write};
use skein_forge_client::{Condition, Effect, Entry, Made, Outcome, Writer};
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{REPO, World};

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
