//! Task notes use the document pane's scope and retain whole bodies.
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::{List, bytes::find};

const CALL: &[&str] = &["task", "notes", "41", "--project", "shop"];
const HEADING: &[u8] = b"note 1 revision 4: Away mode\n";

fn note(number: u64, body: &[u8]) -> c::Note {
    c::Note {
        number,
        revision: c::Revision { value: 4 },
        scope: c::NoteScope::Task { project: f::PROJECT, task: f::TASK },
        description: f::document_text(b"Away mode"),
        body: c::DocumentText::complete(c::Text::new(body.into(), crate::limits::LONG_TEXT).expect("bounded body")),
        writer: c::NoteAuthor::Task { task: f::TASK, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: None,
        acts: List::with_capacity(0),
        cut: false,
    }
}

fn page(notes: &[c::Note], after: Option<c::Cursor>) -> c::ReadResult {
    let mut rows = List::with_capacity(u32::try_from(notes.len()).expect("test notes"));
    for note in notes {
        rows.push(note.clone()).expect("test notes");
    }
    c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage { rows, cut: false, before: None, after })))
}

fn answer(notes: &[c::Note], after: Option<c::Cursor>, json: bool) -> crate::jig::Printed {
    let argv = f::extended(CALL, if json { &["--json"] } else { &[] });
    let printed = f::read_answer(argv.as_slice(), page(notes, after));
    assert_eq!(printed.status, 0);
    assert!(printed.stderr.is_empty());
    printed
}

#[test]
fn task_notes_asks_the_panes_read() {
    for project in ["shop", "1"] {
        for after in [false, true] {
            let argv = f::extended(
                &["task", "notes", "41", "--project", project],
                if after { &["--after", "1122"] } else { &[] },
            );
            let mut root = f::signed_in(f::PARTY);
            let watches = root.watches.len();
            let call = root.submit(argv.as_slice());
            assert_eq!(root.reads.len(), 1);
            assert_eq!(
                root.reads.get(&call).expect("notes read").1,
                c::Read::Notes {
                    scope: c::NoteScope::Task { project: f::PROJECT, task: f::TASK },
                    description: None,
                    after: if after { Some(f::cursor()) } else { None },
                }
            );
            assert_eq!(root.watches.len(), watches, "notes add no watch");
        }
    }
    assert!(f::parse(&["task", "notes", "41"]).is_err(), "project is required");
    assert!(f::parse(&["task", "notes", "41", "--project", ""]).is_err(), "project is nonempty");
}

#[test]
fn task_notes_prints_each_note_whole() {
    let printed = answer(&[note(2, b"second\nbody\n"), note(1, b"first\nbody")], None, false);
    assert_eq!(
        printed.stdout.as_ref(),
        b"note 1 revision 4: Away mode\nfirst\nbody\nnote 2 revision 4: Away mode\nsecond\nbody\n"
    );
}

#[test]
fn task_notes_prints_a_body_past_a_part_whole() {
    let part = usize::try_from(crate::limits::PART).unwrap();
    let text = usize::try_from(crate::limits::TEXT).unwrap();
    let long = usize::try_from(crate::limits::LONG_TEXT).unwrap();
    for length in [part + 1, text + 1, long] {
        let body = "é".repeat(length / 2) + if length % 2 == 1 { "x" } else { "" };
        for json in [false, true] {
            let printed = answer(&[note(1, body.as_bytes())], None, json);
            if json {
                let prefix = br#"{"notes":[{"number":1,"revision":4,"description":"Away mode","body":""#;
                let suffix = br#"","cut":false}],"after":null}"#;
                let mut expected = skein_lib::Writer::new(prefix.len() + length + suffix.len());
                expected.put(prefix).unwrap();
                expected.put(body.as_bytes()).unwrap();
                expected.put(suffix).unwrap();
                assert_eq!(printed.stdout, expected.finish());
            } else {
                assert_eq!(printed.stdout.len(), HEADING.len() + length + 1);
                assert!(find(&printed.stdout, body.as_bytes()).is_some(), "the whole body is printed");
                assert!(printed.stdout.starts_with(HEADING));
                assert!(printed.stdout.ends_with(b"\n"));
            }
        }
    }
}

#[test]
fn task_notes_says_a_cut_body() {
    let mut cut = note(1, b"the supplied prefix");
    cut.body.cut = true;
    let printed = answer(&[cut.clone()], None, false);
    assert_eq!(printed.stdout.as_ref(), b"note 1 revision 4: Away mode\nthe supplied prefix\nbody cut; report it\n");
    let printed = answer(&[cut], None, true);
    assert!(find(&printed.stdout, br#""body":"the supplied prefix","cut":true"#).is_some());
}

#[test]
fn task_notes_says_how_to_read_on() {
    let printed = answer(&[note(1, b"THE BODY")], Some(f::cursor()), false);
    assert_eq!(
        printed.stdout.as_ref(),
        b"note 1 revision 4: Away mode\nTHE BODY\nread on with: task notes 41 --project shop --after 1122\n"
    );
    let printed = answer(&[note(1, b"THE BODY")], Some(f::cursor()), true);
    assert!(printed.stdout.ends_with(br#""after":"1122"}"#));
}

#[test]
fn task_notes_says_no_notes() {
    assert_eq!(answer(&[], None, false).stdout.as_ref(), b"no notes\n");
    assert_eq!(answer(&[], None, true).stdout.as_ref(), br#"{"notes":[],"after":null}"#);
}

#[test]
fn task_notes_json_records_one_note() {
    let mut row = note(1, b"THE BODY\n\"quoted\"\\end");
    row.cut = true;
    let printed = answer(&[row], None, true);
    assert_eq!(
        printed.stdout.as_ref(),
        br#"{"notes":[{"number":1,"revision":4,"description":"Away mode","body":"THE BODY\n\"quoted\"\\end","cut":false}],"after":null}"#
    );
}

#[test]
fn task_notes_refusals_are_task_text_refusals() {
    for json in [false, true] {
        let argv = f::extended(CALL, if json { &["--json"] } else { &[] });
        let printed = f::read_answer(argv.as_slice(), c::ReadResult::None(c::ReadNone::Refused));
        let source = f::extended(&["task", "text", "41"], if json { &["--json"] } else { &[] });
        let expected = f::read_answer(source.as_slice(), c::ReadResult::None(c::ReadNone::Refused));
        assert_eq!(printed.status, expected.status);
        assert_eq!(printed.stderr, expected.stderr);
        assert!(printed.stdout.is_empty());
    }
}
