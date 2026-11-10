//! Party reads and acts on their own preferences (`jig/docs/design/domain/engine/briefs.md`, 2.1).
use jig_core_world::{
    fixtures::{ask_as, second_party},
    reads::{Parties, ask as read, latest},
    world::{Setup, World},
};
use jig_domain_core::{self as core, acts, documents, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token, Wall};

const SEED: u64 = 768;
const PERSON: u64 = 3;
const FIRST: u64 = 41;
const SECOND: u64 = 42;
const PROJECT_NOTE: u64 = 43;
const REVISION: u32 = 2;

fn scope() -> notes::Scope {
    notes::Scope::Party { party: PERSON }
}

fn entry(name: u64, scope: notes::Scope, description: &[u8]) -> notes::Entry {
    let mut references = List::with_capacity(1);
    references.push(99).expect("one reference");
    notes::Entry {
        name,
        scope,
        description: description.into(),
        body: Box::new([]),
        references,
        author: notes::Author::Task { task: 99, attempt: 1 },
        revision: REVISION,
        changed: Wall::from_nanos(name),
    }
}

fn fixture(role: Option<people::Role>) -> (World, u64, u64, u64) {
    let (_, mut limits) = jig_core_world::world::fixture(SEED, false);
    let config = world_config();
    limits.core.people.sign_ins = 4;
    limits.core.people.people = 3;
    limits.core.people.holdings = 3;
    limits.core.people.requests = 16;
    limits.core.documents.rows = 4;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits,
        initial: Some(people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } }),
        require_assignment: false,
        ..Setup::default()
    });
    let owner = world.session();
    let person = second_party(&mut world, b"pref-p", people::Role::Member);
    assert_eq!(world.domain.core().people.kind(PERSON), Some(people::Kind::Person), "{:?}", world.people_answers);
    let other = second_party(&mut world, b"pref-q", people::Role::Member);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
    })));
    let mut holdings = vec![
        people::Holding { person: 1, role: people::Role::Owner },
        people::Holding { person: 4, role: people::Role::Member },
    ];
    if let Some(role) = role {
        holdings.push(people::Holding { person: PERSON, role });
    }
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: holdings.into_boxed_slice(),
    })));
    let mut rows = Vec::new();
    for entry in [
        entry(FIRST, scope(), b"Use short answers"),
        entry(SECOND, scope(), b"Show examples"),
        entry(PROJECT_NOTE, notes::Scope::Goal { project: 1, goal: 99 }, b"Project knowledge"),
    ] {
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            name: entry.name,
            scope: entry.scope.clone(),
            description: entry.description.clone(),
            revision: entry.revision,
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(entry))));
    }
    world.seed(rows.into_boxed_slice());
    // The story starts cold with durable rows, not a run's write or a warmed notes index.
    let config = world_config();
    world.restart_with(config);
    assert!(world.domain.ready(), "cold start: {:?}; {:?}", world.domain.start_phase(), world.domain.start_failure());
    assert_eq!(world.domain.core().people.kind(PERSON), Some(people::Kind::Person));
    assert_eq!(world.domain.core().people.person(person, world.env().now, world.env().wall), Some(PERSON));
    world.reads = Some(Parties::new(world.store.applied));
    (world, person, other, owner)
}

fn world_config() -> root::Config {
    let mut config = jig_core_world::world::fixture(SEED, false).0;
    config.core.grants = Box::new([people::ActGrant { connector: 1, may: Box::new([people::ActKind::RemoveProject]) }]);
    config
}

fn query() -> reads::Read {
    reads::Read::ScopedNotes { scope: scope(), description: None, from: None }
}

fn correction(recalled: u32) -> people::NoteChange {
    people::NoteChange::Correct {
        description: b"Use concise answers".as_slice().into(),
        body: Box::new([]),
        references: Box::new([99]),
        recalled,
    }
}

fn edit(world: &mut World, sign_in: u64, key: u8, name: u64, change: people::NoteChange) {
    ask_as(
        world,
        sign_in,
        key,
        people::Ask::EditNote {
            project: 0,
            name,
            scope: Box::new(people::NoteScope::Party { party: PERSON }),
            change: Box::new(change),
        },
    );
}

fn outcome(world: &World) -> &people::Outcome {
    let people::Reply::Outcome(outcome) = world.people_answers.last().expect("edit answered") else {
        panic!("note outcome")
    };
    outcome
}

fn stored_entry(world: &World, name: u64) -> Option<&notes::Entry> {
    match world.store.rows.get(&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))) {
        Some(root::Record::Core(core::Record::Notes(notes::Record::Entry(entry)))) => Some(entry),
        None => None,
        value => panic!("note entry: {value:?}"),
    }
}

fn stored_line(world: &World, name: u64) -> Option<&notes::Line> {
    match world.store.rows.get(&root::Key::Core(core::Key::Notes(notes::Key::Line { scope: scope(), name }))) {
        Some(root::Record::Core(core::Record::Notes(notes::Record::Line(line)))) => Some(line),
        None => None,
        value => panic!("note index line: {value:?}"),
    }
}

fn preference_rows(world: &World) -> Vec<root::Record> {
    world
        .store
        .rows
        .values()
        .filter(|row| {
            matches!(
                row,
                root::Record::Core(core::Record::Notes(
                    notes::Record::Entry(notes::Entry { scope: notes::Scope::Party { .. }, .. })
                        | notes::Record::Line(notes::Line { scope: notes::Scope::Party { .. }, .. })
                ))
            )
        })
        .cloned()
        .collect()
}

fn keep_own_preferences(world: &mut World, person: u64) {
    let reads::Answer::Page(page) = latest(world) else { panic!("P reads their scope: {:?}", latest(world)) };
    assert_eq!(page.next, None);
    assert_eq!(page.rows.len(), 2);
    for (row, expected) in
        page.rows.iter().zip([entry(FIRST, scope(), b"Use short answers"), entry(SECOND, scope(), b"Show examples")])
    {
        let reads::PageRow::Note(note) = row else { panic!("note row") };
        assert_eq!(note.number, expected.name);
        assert_eq!(note.scope, documents::NoteLocation::Party { party: PERSON });
        assert_eq!(note.description.bytes, expected.description);
        assert!(note.body.bytes.is_empty());
        assert_eq!(note.writer, expected.author);
        assert_eq!(note.tasks, expected.references);
        assert_eq!(note.changed, Some(expected.changed));
        assert_eq!(note.revision, REVISION);
        assert!(!note.cut);
        assert_eq!(
            note.acts.iter().map(|offer| (offer.act, offer.admission.clone())).collect::<Vec<_>>(),
            [(acts::Act::EditNote, acts::Admission::Allowed), (acts::Act::DeleteNote, acts::Admission::Allowed),]
        );
    }
    edit(world, person, 41, FIRST, correction(REVISION));
    assert_eq!(outcome(world), &people::Outcome::NoteEdited { name: FIRST });
    let changed = stored_entry(world, FIRST).expect("corrected row");
    assert_eq!(changed.description.as_ref(), b"Use concise answers");
    assert_eq!(changed.revision, REVISION + 1);
    assert_eq!(changed.author, notes::Author::Party { party: PERSON });
    assert!(changed.changed > Wall::from_nanos(FIRST));
    assert!(changed.changed <= world.wall_time());
    assert_eq!(stored_line(world, FIRST).expect("corrected index").description, changed.description);
    assert_eq!(stored_line(world, FIRST).expect("corrected index").revision, changed.revision);
    let before = preference_rows(world);
    edit(world, person, 42, FIRST, correction(REVISION));
    assert_eq!(
        outcome(world),
        &people::Outcome::Refused(people::Refusal::Moved { now: people::Moved::Note { revision: REVISION + 1 } })
    );
    assert_eq!(preference_rows(world), before);
    edit(world, person, 43, SECOND, people::NoteChange::Delete { recalled: REVISION });
    assert_eq!(outcome(world), &people::Outcome::NoteEdited { name: SECOND });
    assert_eq!(stored_entry(world, SECOND), None);
    assert_eq!(stored_line(world, SECOND), None);
}

#[test]
fn a_person_alone_reads_and_keeps_their_preferences() {
    for role in [None, Some(people::Role::Observer)] {
        let (mut world, person, other, owner) = fixture(role);
        for (label, sign_in, allowed) in [("P", person, true), ("Q", other, false), ("deployment owner", owner, false)]
        {
            world.sign_in = Some(sign_in);
            let before = world.store.rows.clone();
            read(&mut world, 1, query());
            assert_eq!(world.store.rows, before, "{label}'s read writes nothing");
            if allowed {
                keep_own_preferences(&mut world, person);
            } else {
                assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Unknown), "{label}'s read");
                let before = preference_rows(&world);
                for (key, change) in
                    [(44, correction(REVISION + 1)), (45, people::NoteChange::Delete { recalled: REVISION + 1 })]
                {
                    edit(&mut world, sign_in, key, FIRST, change);
                    assert_eq!(outcome(&world), &people::Outcome::Refused(people::Refusal::Unknown), "{label}'s edit");
                    assert_eq!(preference_rows(&world), before, "{label} changes no preferences");
                }
            }
        }
        world.sign_in = Some(other);
        let before = preference_rows(&world);
        read(&mut world, 2, reads::Read::Notes { project: 1, from: None });
        let reads::Answer::Page(page) = latest(&world) else { panic!("project's task notes") };
        let [reads::PageRow::Note(note)] = page.rows.as_slice() else { panic!("only project knowledge") };
        assert_eq!(note.number, PROJECT_NOTE);
        assert_eq!(preference_rows(&world), before);
        world.send(root::Event::Core(core::Event::Act {
            reply_to: ReplyTo::new(Token::new(90)),
            connector: 1,
            key: [90; 16],
            act: Box::new(people::Act::RemoveProject { project: 1, grace: None }),
            carried: None,
        }));
        super::deletion::finish(&mut world);
        assert_eq!(
            world.act_answers.last().expect("project removed").1,
            people::Outcome::ProjectRemoved { project: 1 }
        );
        assert_eq!(stored_entry(&world, PROJECT_NOTE), None);
        assert_eq!(preference_rows(&world), before, "project deletion leaves its person's scope");
        world.sign_in = Some(person);
        read(&mut world, 3, query());
        let reads::Answer::Page(page) = latest(&world) else { panic!("P reads after project deletion") };
        assert_eq!(page.rows.len(), 1);
        world.close();
    }
}
