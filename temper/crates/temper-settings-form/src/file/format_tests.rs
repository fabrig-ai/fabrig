use super::{Read, proposal_items, read, read_with};
use crate::{
    Limits, Text,
    fields::{
        Shape,
        formats::{Conversion, Destination, Formats, Move},
    },
};
use alloc::format;
use skein_json::{Document, Kind, Token, document};

pub(crate) const TEST: Formats = Formats {
    newest: 2,
    moves: &[
        Move {
            format: 2,
            shape: Shape::Overlay,
            member: b"legacy-shell",
            to: Destination::Renamed { member: b"shell" },
        },
        Move {
            format: 2,
            shape: Shape::Overlay,
            member: b"legacy-agents",
            to: Destination::Converted {
                member: b"agents",
                conversion: Conversion::LegacySwitch,
                words: b"agents, a boolean",
            },
        },
        Move {
            format: 2,
            shape: Shape::Overlay,
            member: b"legacy-check",
            to: Destination::MovedOut { words: b"check mode lives in the worker" },
        },
        Move { format: 2, shape: Shape::Overlay, member: b"legacy-mode", to: Destination::Gone { release: b"0.2" } },
    ],
};

pub(super) fn legacy_switch(doc: &Document, at: u32) -> Result<Document, &'static [u8]> {
    let value = doc.token(at).expect("old member value");
    let token = match (value.kind, doc.text(value)) {
        (Kind::String, Some(b"yes")) => Token::True,
        (Kind::String, Some(b"no")) => Token::False,
        (
            Kind::String
            | Kind::ObjectStart
            | Kind::ObjectEnd
            | Kind::ArrayStart
            | Kind::ArrayEnd
            | Kind::Key
            | Kind::Number
            | Kind::True
            | Kind::False
            | Kind::Null
            | Kind::Long,
            _,
        ) => {
            return Err(b"yes or no required");
        }
    };
    Ok(Document::from_tokens(&[token], &document::Limits { tokens: 1, text: 0 }).expect("one boolean"))
}

#[test]
fn absent_format_is_one() {
    let limits = Limits::standard();
    let read_one = read(br#"{"format":1,"items":[{"setting":{"kind":"charter","number":2},"edit":"reset"}]}"#, &limits)
        .expect("explicit format 1");
    let absent = read(br#"{"items":[{"setting":{"kind":"charter","number":2},"edit":"reset"}]}"#, &limits)
        .expect("implicit format 1");
    assert_eq!(read_one.format, 1);
    assert_eq!(absent.format, 1);
    assert!(read_one.notices.is_empty() && absent.notices.is_empty());
    assert_eq!(read_one.items, absent.items);
}

#[test]
fn bad_or_newer_format_is_refused_before_other_members() {
    let limits = Limits::standard();
    assert_eq!(
        read(br#"{"items":[{"setting":{"kind":"nope"}}],"unexpected":true,"format":2}"#, &limits)
            .expect_err("newer file")
            .as_ref(),
        b"settings file: format 2; this temper ctl reads format 1: upgrade it, or write the file at format 1"
    );
    assert_eq!(
        read_with(br#"{"format":3}"#, &limits, TEST).expect_err("newer test file").as_ref(),
        b"settings file: format 3; this temper ctl reads formats 1 to 2: upgrade it, or write the file at format 2"
    );
    for bytes in [
        br#"{"unknown":0,"format":0}"#.as_slice(),
        br#"{"unknown":0,"format":"1"}"#,
        br#"{"unknown":0,"format":1.5}"#,
        br#"{"unknown":0,"format":null}"#,
    ] {
        assert_eq!(
            read(bytes, &limits).expect_err("invalid format").as_ref(),
            b"settings file: format: a whole number from 1"
        );
    }
}

#[test]
fn inline_proposal_items_take_no_format() {
    assert_eq!(
        proposal_items(br#"{"items":[],"format":1}"#, &Limits::standard()).expect_err("not a person's file").as_ref(),
        b"settings file: format: unknown member"
    );
}

#[test]
fn old_members_move_with_a_notice_each_in_file_order() {
    let limits = Limits::standard();
    let old = br#"{"items":[{"setting":{"kind":"charter","number":2},"overlay":{"legacy-mode":true,"legacy-agents":"no"}},{"setting":{"kind":"charter","number":5},"overlay":{"legacy-check":7,"legacy-shell":false}}]}"#;
    let new = br#"{"format":2,"items":[{"setting":{"kind":"charter","number":2},"overlay":{"agents":false}},{"setting":{"kind":"charter","number":5},"overlay":{"shell":false}}]}"#;
    let Read { format, items, notices } = read_with(old, &limits, TEST).expect("older file");
    assert_eq!(format, 1);
    let expected = read_with(new, &limits, TEST).expect("newest file");
    assert_eq!(items, expected.items);
    assert!(expected.notices.is_empty());
    let expected: &[&[u8]] = &[
        b"settings file item 1: overlay.legacy-mode: ignored; 0.2 took it away".as_slice(),
        b"settings file item 1: overlay.legacy-agents: format 1; format 2 writes it as agents, a boolean",
        b"settings file item 2: overlay.legacy-check: moved: check mode lives in the worker",
        b"settings file item 2: overlay.legacy-shell: format 1; format 2 writes it as shell",
    ];
    assert_eq!(notices.as_slice().len(), expected.len());
    for (notice, expected) in notices.iter().zip(expected) {
        assert_eq!(notice.as_ref(), *expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "generate the old and new JSON refusals for each move")]
fn unknown_members_are_still_unknown() {
    for (format, name, value) in [
        (1_u64, "legacy-typo", "7"),
        (2, "legacy-mode", "true"),
        (2, "legacy-check", "7"),
        (2, "legacy-agents", "\"no\""),
        (2, "legacy-shell", "false"),
    ] {
        let source = format!(
            r#"{{"format":{format},"items":[{{"setting":{{"kind":"charter","number":2}},"overlay":{{"{name}":{value}}}}}]}}"#
        );
        let why = read_with(source.as_bytes(), &Limits::standard(), TEST).expect_err("unknown at this format");
        assert_eq!(why.as_ref(), format!("settings file item 1: {name}: unknown member").as_bytes());
    }
}

#[test]
fn the_formats_reference_lists_each_move() {
    let mut measure = Text::measure();
    TEST.write(&mut measure);
    let mut out = Text::writing_exact(measure);
    TEST.write(&mut out);
    assert_eq!(out.finish().as_ref(), b"\n## Formats\n\nNewest format `temper ctl` reads: 2; no `format`: 1.\nformat 2: overlay.legacy-shell: renamed shell\nformat 2: overlay.legacy-agents: converted to agents: agents, a boolean\nformat 2: overlay.legacy-check: moved out: check mode lives in the worker\nformat 2: overlay.legacy-mode: gone in 0.2\n");
}
