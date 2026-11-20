//! Approval codes and secret prints (jig/docs/design/protocol/client/sign-in.md).
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
    let take_bytes = secret_bytes(TAKE_CANARY.as_bytes(), b"\0");
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

#[test]
#[expect(clippy::disallowed_macros, reason = "the eight approval declarations are the client wire's contract")]
fn approval_records_follow_the_sign_in_contract() {
    let schema = include_str!("../../schema/client-v39.schema");
    for line in [
        "record ApprovalAsk { port: option u16 }",
        "enum ApprovalKind { listener, returned, typed }",
        "record ApprovalWaiting { asked: u64 kind: ApprovalKind source: option fixed 16 reader: option fixed 16 }",
        "record ApprovalLoopback { port: u16 grant: secret 32 }",
        "record ApprovalApproved { loopback: option ApprovalLoopback }",
        "enum ApprovalDecided { approved: ApprovalApproved, refused, used, expired, unknown, not_a_browser, signed_out, not_ready, busy }",
        "record ApprovalSignIn { secret: secret 64 grant: option secret 32 }",
        "enum ApprovalSignInAnswer { signed_in: SignedIn, waiting, refused, expired, used, unknown, not_ready, busy, full: ApprovalFull, not_granted }",
    ] {
        let mut present = false;
        for found in schema.lines() {
            present |= found == line;
        }
        assert!(present, "missing declaration: {line}");
    }
}

#[test]
fn approval_records_round_trip_optional_returns_grants_and_sources() {
    use crate::*;
    use skein_lib::Secret;
    macro_rules! round_trip {
        ($ty:ident, $value:expr) => {{
            let value = $value;
            let bytes = super::fixtures::encoded(&value, $ty::measure, $ty::encode);
            let decoded = $ty::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("round trip");
            assert_eq!(decoded, value);
            assert_eq!(super::fixtures::encoded(&decoded, $ty::measure, $ty::encode), bytes);
            bytes
        }};
    }
    for port in [None, Some(43210)] {
        let ask = ApprovalAsk::new(&CEILINGS, ApprovalAskParts { port }).expect("ask");
        assert_eq!(ask.port(), &port);
        round_trip!(ApprovalAsk, ask);
    }
    for returned in [false, true] {
        let grant = if returned { Some(Secret::new(Box::new([9; 32]))) } else { None };
        let take =
            ApprovalSignIn::new(&CEILINGS, ApprovalSignInParts { secret: Secret::new(Box::new([7; 32])), grant })
                .expect("take");
        assert_eq!(take.grant().is_some(), returned);
        round_trip!(ApprovalSignIn, take);
        let loopback = if returned {
            Some(
                ApprovalLoopback::new(
                    &CEILINGS,
                    ApprovalLoopbackParts { port: 43210, grant: Secret::new(Box::new([9; 32])) },
                )
                .expect("loopback"),
            )
        } else {
            None
        };
        let approved = ApprovalApproved::new(&CEILINGS, ApprovalApprovedParts { loopback }).expect("approved");
        round_trip!(ApprovalDecided, ApprovalDecided::Approved(approved));
    }
    for (tag, kind) in [(0, ApprovalKind::Listener), (1, ApprovalKind::Returned), (2, ApprovalKind::Typed)] {
        for source in [None, Some([7; 16])] {
            for reader in [None, Some([8; 16])] {
                let waiting = ApprovalWaiting::new(
                    &CEILINGS,
                    ApprovalWaitingParts { asked: 100, kind: kind.clone(), source, reader },
                )
                .expect("waiting");
                let bytes = round_trip!(ApprovalWaiting, waiting);
                assert_eq!(bytes.get(8), Some(&tag), "approval kind's tag");
            }
        }
    }
    round_trip!(ApprovalSignInAnswer, ApprovalSignInAnswer::NotGranted);
}
