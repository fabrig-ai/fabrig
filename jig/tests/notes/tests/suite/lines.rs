use jig_domain_notes::{Entry, Event, LineRange, Recall, Refusal, Scope};
use jig_notes_world::{Answer, World, task_entry};
use skein_lib::Token;

fn first(world: &mut World) {
    assert_eq!(
        world.call(Event::Write {
            owner: Token::new(1),
            entry: task_entry(20, Scope::Project { project: 7 }, b"draft", b"a\nb", 9),
            recalled: None,
        }),
        Answer::Written { name: 20, revision: 1 }
    );
}

fn append(world: &mut World, recalled: u32, first: u32, text: &[u8]) -> Answer {
    world.call(Event::WriteLines {
        owner: Token::new(2),
        entry: task_entry(20, Scope::Project { project: 7 }, b"draft", text, 9),
        recalled,
        range: LineRange { first, remove: 0, join: false },
    })
}

fn recall(world: &mut World) -> Entry {
    let answer = world.call(Event::Recall { owner: Token::new(3), by: Recall::Name { name: 20 }, page: 0 });
    let Answer::Recalled { entries, more: false } = answer else { panic!("recall: {answer:?}") };
    assert_eq!(entries.len(), 1);
    entries.get(0).expect("whole note").clone()
}

#[test]
fn a_stale_revision_says_the_current_one() {
    let mut world = World::default();
    first(&mut world);
    assert_eq!(append(&mut world, 1, 3, b"c"), Answer::Written { name: 20, revision: 2 });
    let before = recall(&mut world);
    assert_eq!(append(&mut world, 1, 4, b"d"), Answer::Refused { why: Refusal::Moved { revision: 2 } });
    assert_eq!(recall(&mut world), before);
}
#[test]
fn a_range_past_the_end_appends_at_the_end() {
    let mut world = World::default();
    first(&mut world);
    assert_eq!(append(&mut world, 1, 4, b"c"), Answer::Written { name: 20, revision: 2 });
    assert_eq!(&*recall(&mut world).body, b"a\nb\nc");
}
#[test]
fn a_result_past_the_bound_says_so() {
    let mut world = World::default();
    first(&mut world);
    let before = recall(&mut world);
    assert_eq!(
        append(&mut world, 1, 3, &[b'x'; 61]),
        Answer::Refused { why: Refusal::PastBound { bytes: 65, bound: 64 } }
    );
    assert_eq!(recall(&mut world), before);
}
#[test]
fn a_note_built_in_parts_recalls_whole() {
    let mut world = World::default();
    first(&mut world);
    assert_eq!(append(&mut world, 1, 3, b"c\nd\n"), Answer::Written { name: 20, revision: 2 });
    assert_eq!(append(&mut world, 2, 5, b"e"), Answer::Written { name: 20, revision: 3 });
    let entry = recall(&mut world);
    assert_eq!(&*entry.body, b"a\nb\nc\nd\ne");
    assert_eq!(entry.revision, 3);
}
#[test]
fn parts_survive_a_restart() {
    let mut world = World::default();
    first(&mut world);
    assert_eq!(append(&mut world, 1, 3, b"c\n"), Answer::Written { name: 20, revision: 2 });
    let durable = world.records().clone();
    world.restart();
    assert_eq!(world.records(), &durable);
    assert_eq!(append(&mut world, 2, 4, b"d"), Answer::Written { name: 20, revision: 3 });
    world.restart();
    let entry = recall(&mut world);
    assert_eq!(&*entry.body, b"a\nb\nc\nd");
    assert_eq!(entry.revision, 3);
}
