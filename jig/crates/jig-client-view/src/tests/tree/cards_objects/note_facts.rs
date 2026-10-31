use crate::tests::fixtures::AnsweredByEngine;
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;
#[test]
fn note_cards_name_the_whole_stored_attempt_and_only_render_supplied_times() {
    let mut goldens = Goldens::new("tree/cards_objects/note_facts/note_cards_name");
    for writer in [NoteAuthor::Party { party: u64::MAX }, NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX }] {
        for changed in [None, Some(Wall::EPOCH)] {
            let note = Note {
                number: 77,
                revision: Revision { value: u64::MAX },
                scope: NoteScope::Task { project: u32::MAX, task: u64::MAX },
                description: DocumentText::complete(text(b"note title")),
                body: DocumentText { bytes: text(b"note body"), cut: true },
                writer,
                tasks: list([91]),
                changed,
                acts: offered(ActKind::DeleteNote),
                cut: true,
            };
            let (domain, id) = note_loaded(note);
            let words = words();
            let mut bindings = JigBindings::new(256, CLIENT.client.text);
            let mut builder = Builder::new(&limits());
            cards::card(&mut builder, &domain.view(), id, &cards::Place::Elsewhere, words, &mut bindings);
            let tree = builder.finish();
            goldens.check(&tree);
            assert_eq!(
                tree.nodes().iter().filter(|node| node.role == Role::Time).count(),
                usize::from(changed.is_some())
            );
        }
    }
}

fn settled_note_answer(act: ActKind, cut: bool, goldens: &mut Goldens) {
    let note = Note {
        number: 77,
        revision: Revision { value: 123 },
        scope: NoteScope::Project(4),
        description: DocumentText { bytes: text(b"stored note title"), cut },
        body: DocumentText { bytes: text(b"**stored note body**"), cut },
        writer: NoteAuthor::Task { task: 91, attempt: 12 },
        tasks: list([91, u64::MAX]),
        changed: Some(Wall::EPOCH),
        acts: list([
            Offered { effect: None, act: ActKind::EditNote, admission: Admission::Allowed },
            Offered { effect: None, act: ActKind::DeleteNote, admission: Admission::Allowed },
        ]),
        cut,
    };
    let expected_note = note.clone();
    let (mut domain, id) = note_loaded(note);
    let words = words();
    let mut bindings = JigBindings::new(256, CLIENT.client.text);
    let tree = render(&domain, id, words, &mut bindings);
    goldens.check(&tree);
    let label = if act == ActKind::EditNote { b"Edit note".as_slice() } else { b"Delete note".as_slice() };
    let action = press(&tree, label, &mut bindings);
    drive(&mut domain, Event::Act { action });
    let outgoing = drive(&mut domain, Event::Act { action: Action::Confirm });
    let request = outgoing
        .iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { key: request, .. } } => Some(*request),
            _ => None,
        })
        .unwrap();
    goldens.check(&render(&domain, id, words, &mut bindings));
    let answer = Answer::Done(Outcome::Document(DocumentOutcome::NoteEdited { name: 77 }));
    drive(&mut domain, AnsweredByEngine { request, answer: answer.clone() });
    let facts = domain.objects().get(id).unwrap();
    assert_eq!(facts.answer.as_ref(), Some(&answer));
    match &facts.body {
        Some(crate::state::objects::Body::Note { document, .. }) => assert_eq!(document.as_ref(), &expected_note),
        other => panic!("the answer must preserve the supplied note: {other:?}"),
    }
    let mut envelope = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
    envelope.nodes = crate::card_nodes(&CLIENT, &(CLIENT).client).unwrap();
    let mut builder = Builder::new(&envelope);
    bindings.begin();
    assert!(cards::card(&mut builder, &domain.view(), id, &cards::Place::Elsewhere, words, &mut bindings));
    let tree = builder.finish();
    goldens.check(&tree);
    assert!(tree.len() <= envelope.nodes);

    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Time).count(), 1);
}

#[test]
fn a_note_card_shows_its_durable_edit_answer_with_the_stored_document_and_offers() {
    let mut goldens = Goldens::new("tree/cards_objects/note_facts/a_note_card_shows_its_durable_edit");
    settled_note_answer(ActKind::EditNote, false, &mut goldens);
}

#[test]
fn a_note_card_shows_its_durable_deletion_answer_with_the_stored_loss_facts() {
    let mut goldens = Goldens::new("tree/cards_objects/note_facts/a_note_card_shows_its_durable_deletion");
    settled_note_answer(ActKind::DeleteNote, true, &mut goldens);
}
