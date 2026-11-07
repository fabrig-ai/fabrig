//! Approval codes and secret prints (jig/docs/design/protocol/client.md, 8).
use crate::{ApprovalAsked, ApprovalCode, ApprovalSignIn, CEILINGS};
use alloc::fmt::format;
use skein_lib::{Reader, Writer};

const FORMS: &[(&[u8], Option<&[u8; 8]>)] = &[
    (b"WDJB-MJHT", Some(b"WDJBMJHT")),
    (b"WDJBMJHT", Some(b"WDJBMJHT")),
    (b"wdjb mjht", Some(b"WDJBMJHT")),
    (b" WDJB - MJHT ", Some(b"WDJBMJHT")),
    (b"W-D J-BM-JH T", Some(b"WDJBMJHT")),
    (b"------------WDJB-MJHT-----------", Some(b"WDJBMJHT")),
    (b"WDJB-MJH", None),
    (b"WDJB-MJHTB", None),
    (b"WDJA-MJHT", None),
    (b"WDJ1-MJHT", None),
    (b"WDJY-MJHT", None),
    (b"WDJB_MJHT", None),
    (b"WDJB\tMJHT", None),
    (b"WDJ\xff-MJHT", None),
    (b"------------WDJB-MJHT------------", None),
];

fn letters(code: Option<&ApprovalCode>) -> Option<&[u8; 8]> {
    match code {
        Some(code) => Some(code.letters()),
        None => None,
    }
}

#[test]
fn an_approval_code_reads_input_and_carried_forms() {
    for (input, expected) in FORMS {
        assert_eq!(letters(ApprovalCode::parse(input).as_ref()), *expected, "{input:?}");
    }
}

#[test]
fn an_approval_code_is_shown_as_four_a_hyphen_and_four() {
    for (input, expected) in FORMS {
        if expected.is_some() {
            let code = ApprovalCode::parse(input).expect("readable form");
            assert_eq!(code.shown(), *b"WDJB-MJHT", "{input:?}");
            assert_eq!(ApprovalCode::parse(&code.shown()), Some(code), "{input:?}");
        }
    }
}

#[test]
fn from_indexes_takes_the_alphabet_and_refuses_past_it() {
    let rows: &[([u8; 8], Option<&[u8; 8]>)] = &[
        ([0; 8], Some(b"BBBBBBBB")),
        ([19; 8], Some(b"ZZZZZZZZ")),
        ([0, 1, 2, 3, 4, 5, 6, 7], Some(b"BCDFGHJK")),
        ([20, 0, 0, 0, 0, 0, 0, 0], None),
        ([0, 0, 0, 0, 20, 0, 0, 0], None),
        ([0, 0, 0, 0, 0, 0, 0, 20], None),
        ([u8::MAX; 8], None),
    ];
    for (indexes, expected) in rows {
        assert_eq!(letters(ApprovalCode::from_indexes(*indexes).as_ref()), *expected, "{indexes:?}");
    }
}

fn secret_bytes(secret: &[u8], tail: &[u8]) -> Box<[u8]> {
    let size = size_of::<u32>().checked_add(secret.len()).expect("canary size fits usize");
    let size = size.checked_add(tail.len()).expect("canary and tail fit usize");
    let mut writer = Writer::new(size);
    writer.put(&u32::try_from(secret.len()).expect("canary length fits u32").to_be_bytes()).expect("length fits");
    writer.put(secret).expect("canary fits");
    writer.put(tail).expect("tail fits");
    writer.finish()
}

#[test]
fn debug_of_an_approval_shows_the_secrets_length_never_its_bytes() {
    const ASKED_CANARY: &str = "canary-approval-32-byte-secret!!";
    const TAKE_CANARY: &str = "canary-c42e8b";
    let asked_bytes = secret_bytes(ASKED_CANARY.as_bytes(), b"WDJBMJHT\0\0\0\0\0\0\0\x1d");
    let asked = ApprovalAsked::decode(&CEILINGS, &mut Reader::new(&asked_bytes)).expect("bounded ask secret");
    let take_bytes = secret_bytes(TAKE_CANARY.as_bytes(), b"");
    let take = ApprovalSignIn::decode(&CEILINGS, &mut Reader::new(&take_bytes)).expect("bounded take secret");
    for (debug, pretty, canary, length) in [
        (format(format_args!("{asked:?}")), format(format_args!("{asked:#?}")), ASKED_CANARY, "Secret { len: 32 }"),
        (format(format_args!("{take:?}")), format(format_args!("{take:#?}")), TAKE_CANARY, "Secret { len: 13 }"),
    ] {
        let byte_list = format(format_args!("{:?}", canary.as_bytes()));
        assert!(!debug.contains(canary));
        assert!(!debug.contains(&byte_list));
        assert!(!pretty.contains(canary));
        assert!(!pretty.replace([' ', '\n'], "").contains(&byte_list.replace(' ', "")));
        assert!(debug.contains(length), "{debug}");
        assert!(pretty.replace([' ', '\n', ','], "").contains(&length.replace(' ', "")), "{pretty}");
    }
    let encoded = super::fixtures::encoded(&asked, ApprovalAsked::measure, ApprovalAsked::encode);
    assert_eq!(encoded, asked_bytes);
    let decoded = ApprovalAsked::decode(&CEILINGS, &mut Reader::new(&encoded)).expect("round trip ask");
    assert_eq!(decoded.secret().expose(), ASKED_CANARY.as_bytes());
    assert_eq!(decoded, asked);
    let encoded = super::fixtures::encoded(&take, ApprovalSignIn::measure, ApprovalSignIn::encode);
    assert_eq!(encoded, take_bytes);
    let decoded = ApprovalSignIn::decode(&CEILINGS, &mut Reader::new(&encoded)).expect("round trip take");
    assert_eq!(decoded.secret().expose(), TAKE_CANARY.as_bytes());
    assert_eq!(decoded, take);
}
