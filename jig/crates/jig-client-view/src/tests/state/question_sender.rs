use crate::JigBindings;
use crate::state::objects::{Object, question::facts};
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::tests::tree::{
    cards::{CLIENT, text, words},
    cards_objects::{loaded, question, render},
};
use crate::words::Words;
use crate::words::phrases::english;
use crate::words::phrases::jig::{JigPhrase, shapes};
use jig_client::contract::*;
use skein_lib::{Id, Wall, bytes::copy_of};
use skein_ui::phrases::{HoleKind, PhraseLimits, Phrasebook, Segment, WallFormat};
use skein_ui::{Builder, Tree};
fn fixture(from: Option<Requester>) -> (Fixture, Id<Object>, Question) {
    let mut value = question();
    value.from = from;
    value.words = DocumentText::complete(text(UTF8_TEXTS[0]));
    value.since = Wall::from_nanos(23);
    let (d, id) = loaded(
        ObjectKey::Question { task: value.task.task, message: value.message },
        ReadValue::Question(Box::new(value.clone())),
    );
    assert_eq!(facts(&d.objects().get(id).unwrap()), Some(&value));
    (d, id, value)
}
fn tree(d: &Fixture, id: Id<Object>, w: &Words) -> Tree {
    render(d, id, w, &mut JigBindings::new(64, CLIENT.client.text))
}
fn selected() -> Words {
    let mut first = english::language();
    first.wall = WallFormat::Nanos;
    let mut second = first.clone();
    for phrase in &mut second.phrases {
        let suffix = if phrase.id == JigPhrase::QuestionFromTask.id() {
            Some(b" task/source".as_slice())
        } else if phrase.id == JigPhrase::QuestionFromPerson.id() {
            Some(b" person/source".as_slice())
        } else if phrase.id == JigPhrase::QuestionFromDeployment.id() {
            Some(b" deployment/source".as_slice())
        } else {
            None
        };
        if let Some(suffix) = suffix {
            for form in &mut phrase.forms {
                form.segments =
                    Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(suffix))]);
            }
        }
    }
    Words::new(
        Phrasebook::load(
            shapes(CLIENT.client.text),
            Box::new([first, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: 6000,
                text_bytes: 100_000,
                hole_bytes: CLIENT.client.text,
            },
        )
        .unwrap(),
        1,
        60,
    )
}
#[test]
fn question_sender_cards_render_exact_closed_kinds_full_widths_and_no_claim_for_absence() {
    let mut goldens = Goldens::new("state/question_sender/question_sender_cards");
    for from in [
        None,
        Some(Requester::Task(0)),
        Some(Requester::Task(u64::MAX)),
        Some(Requester::Person(0)),
        Some(Requester::Person(u64::MAX)),
        Some(Requester::Deployment { project: 0 }),
        Some(Requester::Deployment { project: u32::MAX }),
    ] {
        let (d, id, value) = fixture(from);
        let shown = tree(&d, id, words());
        goldens.check(&shown);
        assert_eq!(facts(&d.objects().get(id).unwrap()), Some(&value));
    }
}
#[test]
fn question_sender_selected_language_keeps_reordered_numeric_holes_whole_words_and_waiting_clock() {
    let mut goldens = Goldens::new("state/question_sender/question_sender_selected");
    let w = selected();
    for from in [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }] {
        let (d, id, value) = fixture(Some(from));
        let shown = tree(&d, id, &w);
        goldens.check(&shown);
        let mut b = Builder::new(&crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, &w).unwrap());
        w.waiting(&mut b, d.wall(), value.since);

        assert_eq!(facts(&d.objects().get(id).unwrap()), Some(&value));
    }
    let (d, id, _) = fixture(None);
    goldens.check(&tree(&d, id, &w));
}

const UTF8_TEXTS: [&[u8]; 3] = [b"w~\0^?", &[b'x'; 128], &[b't'; 128]];
