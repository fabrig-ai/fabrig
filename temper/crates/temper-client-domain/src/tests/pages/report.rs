use crate::pages::report::{self, Report};
use crate::tests::fixtures::chat::document_text;
use jig_client::contract::{self as c, Read};
use skein_lib::{List, Wall};

const PROJECT: u32 = 7;
const GOAL: u64 = 42;
const CHANGED: Wall = Wall::from_nanos(540);

fn note(description: &[u8], body: &[u8]) -> c::Note {
    c::Note {
        number: 1,
        revision: c::Revision { value: 3 },
        scope: c::NoteScope::Task { project: PROJECT, task: GOAL },
        description: document_text(description),
        body: document_text(body),
        writer: c::NoteAuthor::Task { task: GOAL, attempt: 2 },
        tasks: List::with_capacity(0),
        changed: Some(CHANGED),
        acts: List::with_capacity(0),
        cut: false,
    }
}

fn parts(words: &[&[u8]]) -> List<Box<[u8]>> {
    let mut parts = List::with_capacity(u32::try_from(words.len()).expect("fixture count fits"));
    for word in words {
        parts.push(Box::from(*word)).expect("fixture count fits");
    }
    parts
}

#[test]
fn a_whole_report_reads_in_its_parts() {
    let note = note(
        b"report: The helpers landed.\nNot the headline.",
        "  I built the helper.  \n It reads notes.\r\n\nI checked its parts.\nLeft: Draw the report.\nRisk: None known.\nDecided: first, 05:40 · second, 05:12\n".as_bytes(),
    );
    assert_eq!(
        report::parse(&note),
        Report {
            headline: Box::from(&b"The helpers landed."[..]),
            paragraphs: parts(&[b"I built the helper. It reads notes.", b"I checked its parts."]),
            left: Some(Box::from(&b"Draw the report."[..])),
            risk: Some(Box::from(&b"None known."[..])),
            decided: parts(&[b"first, 05:40", b"second, 05:12"]),
            at: Some(CHANGED),
        }
    );
}

#[test]
fn every_line_is_optional() {
    for (body, left, risk, decided) in [
        (&b""[..], None, None, parts(&[])),
        (&b"Left: Later."[..], Some(Box::from(&b"Later."[..])), None, parts(&[])),
        (&b"Risk: Rain."[..], None, Some(Box::from(&b"Rain."[..])), parts(&[])),
        (&b"Decided: wait, 05:40"[..], None, None, parts(&[b"wait, 05:40"])),
        ("Decided: a · · b".as_bytes(), None, None, parts(&[b"a", "· b".as_bytes()])),
        (
            &b"Left: old\nLeft: \nRisk: old\nRisk: \nDecided: old\nDecided: "[..],
            Some(Box::from(&b""[..])),
            Some(Box::from(&b""[..])),
            parts(&[]),
        ),
    ] {
        let mut note = note(b"report: Ready.", body);
        note.changed = None;
        assert_eq!(
            report::parse(&note),
            Report { headline: Box::from(&b"Ready."[..]), paragraphs: parts(&[]), left, risk, decided, at: None }
        );
    }
}

#[test]
fn only_a_lines_start_makes_a_field() {
    let note = note(
        b"report: Ready.",
        b"We saw Left: inside a line.\nAnd Risk: inside too.\nAlso Decided: in prose.\nLeft: first\nA paragraph follows.\nRisk: old\nLeft: last\nRisk: last risk\nDecided: old\nDecided: last, 05:12\nThe final paragraph.",
    );
    let report = report::parse(&note);
    assert_eq!(
        report.paragraphs,
        parts(&[
            b"We saw Left: inside a line. And Risk: inside too. Also Decided: in prose.",
            b"A paragraph follows.",
            b"The final paragraph.",
        ])
    );
    assert_eq!(report.left.as_deref(), Some(&b"last"[..]));
    assert_eq!(report.risk.as_deref(), Some(&b"last risk"[..]));
    assert_eq!(report.decided, parts(&[b"last, 05:12"]));
}

#[test]
fn a_cut_body_reads_as_far_as_it_goes() {
    let mut cut_field = note(b"report: In progress.", "I kept café.\nLeft: finish caf".as_bytes());
    cut_field.body.cut = true;
    let field = report::parse(&cut_field);
    assert_eq!(field.paragraphs, parts(&["I kept café.".as_bytes()]));
    assert_eq!(field.left.as_deref(), Some(&b"finish caf"[..]));

    let mut cut_paragraph = note(b"report: In progress.", "A whole paragraph.\n\nI kept café\nand a par".as_bytes());
    cut_paragraph.body.cut = true;
    let paragraph = report::parse(&cut_paragraph);
    assert_eq!(paragraph.paragraphs, parts(&[b"A whole paragraph.", "I kept café and a par".as_bytes()]));
    assert_eq!(paragraph.left, None);
}

#[test]
fn a_description_without_the_opening_reads_whole() {
    for (description, expected) in [
        (&b"The report is ready."[..], &b"The report is ready."[..]),
        (&b"We saw report: here.\nAnother line."[..], &b"We saw report: here."[..]),
        (&b""[..], &b""[..]),
        (&b"rep"[..], &b"rep"[..]),
        (&b"  A headline.  \nNext."[..], &b"  A headline.  "[..]),
    ] {
        assert_eq!(&*report::parse(&note(description, b"")).headline, expected);
    }
}

#[test]
fn newest_takes_the_opening_and_the_latest_change() {
    let mut rows = List::with_capacity(5);
    assert_eq!(report::newest(&rows), None);
    let mut incidental = note(b"We saw report: inside.", b"");
    incidental.number = 99;
    incidental.changed = Some(Wall::from_nanos(999));
    rows.push(incidental).unwrap();
    assert_eq!(report::newest(&rows), None);
    let mut undated = note(b"report: Undated.", b"");
    undated.changed = None;
    undated.number = 90;
    rows.push(undated).unwrap();
    assert_eq!(report::newest(&rows).unwrap().number, 90);
    let mut latest = note(b"report: Latest.", b"");
    latest.number = 3;
    rows.push(latest.clone()).unwrap();
    assert_eq!(report::newest(&rows).unwrap().number, 3);
    latest.number = 4;
    rows.push(latest).unwrap();
    assert_eq!(report::newest(&rows).unwrap().number, 4);
    let mut older = note(b"report: Older.", b"");
    older.number = 80;
    older.changed = Some(Wall::EPOCH);
    rows.push(older).unwrap();
    assert_eq!(report::newest(&rows).unwrap().number, 4);
}

#[test]
fn its_read_names_the_goal_and_the_opening() {
    assert_eq!(report::OPENING, b"report: ");
    assert_eq!(
        report::read(PROJECT, GOAL),
        Read::Notes {
            scope: c::NoteScope::Task { project: PROJECT, task: GOAL },
            description: Some(c::Text::new(Box::from(&b"report: "[..]), 256).unwrap()),
            after: None,
        }
    );
}
