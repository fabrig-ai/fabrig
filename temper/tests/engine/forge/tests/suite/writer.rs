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
