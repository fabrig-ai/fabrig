use super::fixtures::{
    Frame,
    charter::{Case, Charter, LEAF, NAME, Overlay, Part, Target},
};
use jig_client::contract::REASON_NAMES;
use jig_client_protocol_wire as w;
use skein_io::kernel::Exit;

const COPY: &[&[u8]] = &[b"charter", b"copy", NAME];
const WORDS: &[u8] = b"Build it.";
const EXISTS: &[u8] =
    b"temper ctl: .temper/charters/implementer.md exists; charter copy never writes over a charter's file\n";

fn whole() -> Charter {
    Charter { name: Some(NAME), instructions: WORDS, parts: &[], overlay: None }
}
fn copy(words: &[&[u8]], case: Case) -> Frame {
    let mut frame = Frame::act(words);
    frame.charter = Some(case);
    frame.answer_sign_in_then_me();
    assert_eq!(frame.act_attempts, [], "copies send no act");
    frame
}
fn read(frame: &Frame) -> temper_charter_set::CharterFile {
    let bytes = frame.stored(LEAF).expect("new charter file");
    temper_charter_set::read_charter(std::str::from_utf8(bytes).expect("UTF-8 file"), &REASON_NAMES)
        .expect("project can read the charter file")
}

#[test]
fn copy_uses_the_default_words_and_the_chosen_directory() {
    const EXPECTED: &[u8] = b"Build it.\n";
    type Written = (&'static [&'static [u8]], &'static [u8], &'static [u8], &'static [u8]);
    let cases: &[Written] = &[
        (COPY, b".", b".temper/charters", b".temper/charters/implementer.md (10 bytes)\n"),
        (
            &[b"charter", b"copy", b"2", b"--into", b"work/project"],
            b".",
            b"work/project/.temper/charters",
            b"work/project/.temper/charters/implementer.md (10 bytes)\n",
        ),
        (
            &[b"charter", b"copy", b"2", b"--into", b"/work/project"],
            b"/",
            b"work/project/.temper/charters",
            b"/work/project/.temper/charters/implementer.md (10 bytes)\n",
        ),
        (
            &[b"charter", b"copy", NAME, b"--into", b"../left/../project"],
            b"./..",
            b"project/.temper/charters",
            b"../project/.temper/charters/implementer.md (10 bytes)\n",
        ),
    ];
    for &(words, anchor, parent, output) in cases {
        let frame = copy(
            words,
            Case::new(Charter {
                instructions: b"Build it.\n\nUse the box.",
                overlay: Some(Overlay { instructions: b"Use the box.", parts: &[] }),
                ..whole()
            }),
        );
        let parsed = crate::arguments::parse(words).expect("copy command");
        assert_eq!(crate::arguments::file_root(&parsed).as_deref(), Some(anchor));
        let case = frame.charter.as_ref().expect("charter fixture");
        assert!(matches!(case.asked.as_slice(), [w::Read::Settings, w::Read::Setting(_)]));
        assert_eq!(case.placed_in.as_deref(), Some(parent));
        assert_eq!(frame.exit(), Some(Exit::Code(0)));
        assert_eq!(frame.stdout(), output);
        assert_eq!(frame.stored(LEAF), Some(EXPECTED));
        let file = read(&frame);
        assert_eq!((file.instructions.as_ref(), file.settings, file.parts.len()), ("Build it.", None, 0));
    }
}

#[test]
fn copy_writes_ordered_parts_from_the_source_charter() {
    const PARTS: &[Part] =
        &[Part { on: (1 << 13) | 1, words: b"Take notes." }, Part { on: 1 << 8, words: b"Read the result." }];
    const BOX_PART: &[Part] = &[Part { on: 1 << 8, words: b"Use the log." }];
    const FILE: &[u8] =
        b"Build it.\n\n## part, on: first, words\nTake notes.\n\n## part, on: result\nRead the result.\n";
    let cases = [
        Charter { parts: PARTS, ..whole() },
        Charter { parts: PARTS, overlay: Some(Overlay { instructions: b"", parts: BOX_PART }), ..whole() },
    ];
    for charter in cases {
        let frame = copy(COPY, Case::new(charter));
        let file = read(&frame);
        assert_eq!(frame.exit(), Some(Exit::Code(0)));
        assert_eq!(frame.stored(LEAF), Some(FILE));
        let parts: Vec<_> = file
            .parts
            .iter()
            .map(|part| (part.reasons.iter().map(AsRef::as_ref).collect::<Vec<_>>(), part.text.as_ref()))
            .collect();
        assert_eq!(parts, vec![(vec!["first", "words"], "Take notes."), (vec!["result"], "Read the result.")]);
    }
}

#[test]
fn copy_does_not_overwrite_an_existing_name() {
    for target in [Target::File, Target::Link, Target::Raced] {
        let mut case = Case::new(whole());
        case.target = Some(target);
        let frame = copy(COPY, case);
        assert_eq!(frame.exit(), Some(Exit::Code(1)));
        assert_eq!(frame.stderr(), EXISTS);
        assert_eq!(frame.stdout(), b"");
        assert_eq!(frame.stored(LEAF), None);
    }
}

#[test]
fn copy_does_not_follow_a_linked_directory() {
    let mut case = Case::new(whole());
    case.linked_directory = Some(b".temper");
    let frame = copy(COPY, case);
    assert_eq!(frame.exit(), Some(Exit::Code(1)));
    assert_eq!(frame.stored(LEAF), None);
    assert_eq!(frame.stdout(), b"");
}

#[test]
fn refused_charter_words_leave_the_destination_untouched() {
    const BAD_PART: &[Part] = &[Part { on: 1, words: b"Read this.\n## part, on: first" }];
    type Refused = (&'static [&'static [u8]], Charter, &'static [u8]);
    let cases: &[Refused] = &[
        (&[b"charter", b"copy", b"2"], Charter { name: None, ..whole() }, b"charter 2 has no name"),
        (COPY, Charter { parts: BAD_PART, ..whole() }, b"charter 2:2: part 1: a line begins `## part, on:`"),
        (&[b"charter", b"copy", b"99"], whole(), b"unknown"),
        (&[b"charter", b"copy", b"absent"], whole(), b"no charter is named absent"),
        (
            COPY,
            Charter { overlay: Some(Overlay { instructions: b"Use the box.", parts: &[] }), ..whole() },
            b"charter 2 does not end with its overlay's words",
        ),
    ];
    for (words, charter, why) in cases {
        let frame = copy(words, Case::new(charter.clone()));
        assert_eq!(frame.exit(), Some(Exit::Code(1)));
        assert!(skein_lib::bytes::find(frame.stderr(), why).is_some(), "{:?}", std::str::from_utf8(frame.stderr()));
        assert_eq!(frame.stdout(), b"");
        assert!(!frame.charter.as_ref().expect("charter fixture").destination_opened);
    }
}

#[test]
fn a_json_call_cannot_choose_a_charter_destination() {
    for input in [
        br#"{"tool":"charter copy","arguments":{"number":"implementer"}}"#.as_slice(),
        br#"{"tool":"charter copy","arguments":{"number":"implementer","into":"checkout"}}"#,
    ] {
        let mut frame = Frame::read(&["--call", "-"]);
        frame.input = input;
        frame.drain();
        assert!(!frame.opened(b"directory"), "refuse before opening DIRECTORY");
        assert_eq!(frame.exit(), Some(Exit::Code(1)));
        assert_eq!(frame.requests(), 0);
    }
}
