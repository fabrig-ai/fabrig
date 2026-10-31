#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect complete typed cards and loaded language data")]
use crate::JigBindings;
use crate::state::objects::{Object, question::facts};
use crate::tests::fixtures::Fixture;
use crate::tests::tree::{
    cards::{CLIENT, contains, text, words},
    cards_objects::{loaded, question, render},
};
use crate::words::Words;
use crate::words::phrases::english;
use crate::words::phrases::jig::{JigPhrase, shapes};
use jig_client::contract::*;
use skein_lib::{Id, Wall, bytes::copy_of};
use skein_ui::phrases::{HoleKind, PhraseLimits, Phrasebook, Segment, WallFormat};
use skein_ui::{Builder, Tree};
fn fixture(from: Option<OwnerRequester>) -> (Fixture, Id<Object>, Question) {
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
    render(d, id, w, &mut JigBindings::new(64, CLIENT.text))
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
            shapes(CLIENT.text),
            Box::new([first, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::owner_words::phrase_count(),
                holes: 4,
                segments: 6000,
                text_bytes: 100_000,
                hole_bytes: CLIENT.text,
            },
        )
        .unwrap(),
        1,
        60,
    )
}
#[test]
fn question_sender_cards_render_exact_closed_kinds_full_widths_and_no_claim_for_absence() {
    for (from, expected) in [
        (None, b"".as_slice()),
        (Some(OwnerRequester::Task(0)), b"Question from task 0"),
        (Some(OwnerRequester::Task(u64::MAX)), b"Question from task 18446744073709551615"),
        (Some(OwnerRequester::Person(0)), b"Question from person 0"),
        (Some(OwnerRequester::Person(u64::MAX)), b"Question from person 18446744073709551615"),
        (Some(OwnerRequester::Deployment { project: 0 }), b"Question from deployment project 0"),
        (Some(OwnerRequester::Deployment { project: u32::MAX }), b"Question from deployment project 4294967295"),
    ] {
        let (d, id, value) = fixture(from);
        let shown = tree(&d, id, words());
        if from.is_some() {
            assert!(contains(&shown, expected));
        }
        let (task, person, deployment) = match from {
            None => (false, false, false),
            Some(from) => match from {
                OwnerRequester::Task(_) => (true, false, false),
                OwnerRequester::Person(_) => (false, true, false),
                OwnerRequester::Deployment { .. } => (false, false, true),
            },
        };
        assert_eq!(contains(&shown, b"Question from task "), task);
        assert_eq!(contains(&shown, b"Question from person "), person);
        assert_eq!(contains(&shown, b"Question from deployment project "), deployment);
        assert!(contains(&shown, value.words.bytes()), "whole opaque words survive markdown");
        assert!(contains(&shown, b"Waiting "));
        assert!(
            shown
                .nodes()
                .iter()
                .any(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Answer"))
        );
        assert_eq!(facts(&d.objects().get(id).unwrap()), Some(&value));
    }
}
#[test]
fn question_sender_selected_language_keeps_reordered_numeric_holes_whole_words_and_waiting_clock() {
    let w = selected();
    for (from, expected) in [
        (OwnerRequester::Task(u64::MAX), b"18446744073709551615 task/source".as_slice()),
        (OwnerRequester::Person(u64::MAX), b"18446744073709551615 person/source"),
        (OwnerRequester::Deployment { project: u32::MAX }, b"4294967295 deployment/source"),
    ] {
        let (d, id, value) = fixture(Some(from));
        let shown = tree(&d, id, &w);
        assert!(contains(&shown, expected));
        assert!(!contains(&shown, b"Question from "));
        assert!(contains(&shown, value.words.bytes()));
        let mut b = Builder::new(&crate::sizes::limits::of_words(&CLIENT, &w).unwrap());
        w.waiting(&mut b, d.wall(), value.since);
        for node in b.finish().nodes() {
            for run in &node.text {
                assert!(contains(&shown, &run.text));
            }
        }
        assert_eq!(facts(&d.objects().get(id).unwrap()), Some(&value));
    }
    let (d, id, _) = fixture(None);
    assert!(!contains(&tree(&d, id, &w), b"/source"));
}

const UTF8_TEXTS: [&[u8]; 3] = [b"w~\0^?", &[b'x'; 128], &[b't'; 128]];
