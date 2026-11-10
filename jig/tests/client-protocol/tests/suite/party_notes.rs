//! A person's preferences through the client wire (client/domain/contract/README.md, 4).
use jig_client::{Answered, Ask, Asker, Handle, Key, Outcome, Readable, Readout, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_test_domain as root;
use skein_lib::{Duration, List, Wall};

const WAIT: Duration = Duration::from_secs(1);
const FIRST: u64 = 41;
const SECOND: u64 = 42;
const PERSON: u64 = fixtures::FIRST_OWNER;

fn scope() -> c::NoteScope {
    c::NoteScope::Party { party: PERSON }
}

fn seeded() -> Story {
    let mut story = Story::new(fixtures::SEED);
    let mut rows = Vec::new();
    for (name, description) in [(FIRST, b"Use short answers".as_slice()), (SECOND, b"Show examples".as_slice())] {
        let scope = notes::Scope::Party { party: PERSON };
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            name,
            scope: scope.clone(),
            description: description.into(),
            revision: 2,
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
            name,
            scope,
            description: description.into(),
            body: Box::new([]),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: PERSON },
            changed: Wall::from_nanos(name),
            revision: 2,
        }))));
    }
    story.seed(rows.into_boxed_slice());
    story.restart_engine(Duration::ZERO);
    story.sign_in();
    assert!(story.run_until(|s| s.client.link().state == jig_client::Link::Live, WAIT));
    assert_eq!(story.client.link().party, Some(PERSON));
    story
}

fn read(story: &mut Story, number: u64) -> Readout {
    let handle = Handle { asker: Asker::JIG, number };
    story.read(handle, Readable::Jig(c::Read::Notes { scope: scope(), description: None, after: None }));
    assert!(story.run_until(|s| !s.pages(handle).is_empty(), WAIT));
    let pages = story.pages(handle);
    let [answer] = pages.as_slice() else { panic!("one notes answer: {pages:?}") };
    answer.clone()
}

fn act(story: &mut Story, number: u64, request: c::Ask, note: u64) {
    let key = Key { asker: Asker::JIG, number };
    story.send(key, Ask::Jig(Box::new(request)));
    assert!(story.run_until(|s| !s.answered(key).is_empty(), WAIT));
    assert_eq!(
        story.answered(key),
        [Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::NoteEdited {
            name: note
        },))))]
    );
}

#[test]
fn a_person_reads_corrects_and_deletes_preferences_through_the_client() {
    let mut story = seeded();
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) = read(&mut story, 31) else {
        panic!("P's notes cross the wire")
    };
    assert!(!page.cut);
    assert_eq!((page.before, page.after), (None, None));
    assert_eq!(
        page.rows.iter().map(|note| (note.number, note.description.bytes())).collect::<Vec<_>>(),
        [(FIRST, b"Use short answers".as_slice()), (SECOND, b"Show examples".as_slice()),]
    );
    for note in &page.rows {
        assert_eq!(note.scope, scope());
        assert_eq!(note.revision, c::Revision { value: 2 });
        assert_eq!(note.writer, c::NoteAuthor::Party { party: PERSON });
        assert_eq!(note.changed, Some(Wall::from_nanos(note.number)));
        assert_eq!(
            note.acts.iter().map(|offer| (offer.act, offer.admission.clone())).collect::<Vec<_>>(),
            [(c::ActKind::EditNote, c::Admission::Allowed), (c::ActKind::DeleteNote, c::Admission::Allowed),]
        );
    }
    // Q signs in on the same application-held client, so a cached P page cannot answer Q.
    story.sign_in_as(fixtures::identity(2));
    assert_ne!(story.client.link().party, Some(PERSON));
    assert_eq!(read(&mut story, 32), Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)));
    story.sign_in_as(fixtures::identity(PERSON));
    assert_eq!(story.client.link().party, Some(PERSON));
    act(
        &mut story,
        41,
        c::Ask::EditNote {
            project: 0,
            note: FIRST,
            scope: scope(),
            revision: c::Revision { value: 2 },
            description: c::Text::new(b"Use concise answers".as_slice().into(), 32).expect("short preference"),
            body: c::Text::new(Box::new([]), 32).expect("empty body"),
            references: List::with_capacity(0),
        },
        FIRST,
    );
    act(
        &mut story,
        42,
        c::Ask::DeleteNote { project: 0, note: SECOND, scope: scope(), revision: c::Revision { value: 2 } },
        SECOND,
    );
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) = read(&mut story, 33) else {
        panic!("P's remaining line")
    };
    let [note] = page.rows.as_slice() else { panic!("one preference after deletion") };
    assert_eq!(note.number, FIRST);
    assert_eq!(note.description.bytes(), b"Use concise answers");
    assert_eq!(note.revision, c::Revision { value: 3 });
}
