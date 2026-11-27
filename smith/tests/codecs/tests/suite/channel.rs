//! Channel kind and body checks (protocol/channel.md, sections 2-4).

use crate::support::fixed_values::FixedValues;
use smith_channel::Record;

use skein_channel::Direction;
use skein_lib::Reader;
use smith_channel::{Answer, CEILINGS, Start, V14, schema};

#[test]
fn channel_fixed_values_decode_at_their_measured_length() {
    let fixed_values = FixedValues::new("channel", smith_channel::wire::VERSION);
    fixed_values.assert_measured("list_ask", |bytes| {
        smith_channel::ListAsk::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("list ask decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
    fixed_values.assert_measured("list_row", |bytes| {
        smith_channel::ListRow::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("list row decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
    fixed_values.assert_measured("start", |bytes| {
        Start::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("start decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
    fixed_values.assert_measured("drain", |bytes| {
        smith_channel::Drain::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("drain decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
    fixed_values.assert_measured("signalled", |bytes| {
        smith_channel::Signalled::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("signal notice decodes")
            .measure(&CEILINGS)
            .expect("bounded signal notice")
    });
    fixed_values.assert_measured("answer", |bytes| {
        Answer::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("answer decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
}

#[test]
fn usage_round_trips_present_and_absent_window_fields() {
    for state in
        [smith_channel::UsageState::Open, smith_channel::UsageState::Warned, smith_channel::UsageState::Limited]
    {
        for mask in 0..8 {
            let window = smith_channel::UsageWindow {
                label: b"seven_day".as_slice().into(),
                length: (mask & 1 != 0).then_some(skein_lib::Duration::from_secs(604_800)),
                used: (mask & 2 != 0).then_some(8123),
                resets: (mask & 4 != 0).then_some(1_781_845_200),
                state: state.clone(),
            };
            let record = smith_channel::Usage { account: 7, window };
            let frame = smith_channel::frame(smith_channel::Body::Usage(record.clone())).expect("usage frame");
            assert_eq!(frame.kind(), smith_channel::kind::USAGE);
            let body = &frame.bytes()[8..];
            let read = smith_channel::Usage::decode(&CEILINGS, &mut Reader::new(body)).expect("usage decodes");
            assert_eq!(read, record, "{state:?}, field mask {mask}");
            assert_eq!(usize::try_from(read.measure(&CEILINGS).expect("bounded usage")).expect("measure"), body.len());
        }
    }
}

#[test]
fn every_required_kind_has_its_direction_and_bound_in_the_schema() {
    let table = schema(&CEILINGS).expect("schema bounds fit");
    assert_eq!(table.magic, *b"smth");
    assert_eq!(smith_channel::KindRule::FORMAT_VERSION, u64::from(smith_channel::wire::VERSION));
    assert_eq!(table.version, smith_channel::wire::VERSION);
    assert_eq!(table.kinds.len(), 26);
    for (index, rule) in V14.iter().enumerate() {
        assert!(rule.required, "all initial kinds are required");
        let kind = table.kind(rule.kind).expect("required kind is listed");
        assert_eq!(kind.direction, rule.direction);
        if [0x0105, 0x0106, 0x010c].contains(&rule.kind) {
            assert_eq!(kind.largest, 0);
        } else {
            assert!(kind.largest > 0);
        }
        assert_eq!(usize::from(rule.kind), 0x0100 + index);
        if index < 6
            || [smith_channel::kind::DRAIN, smith_channel::kind::RENEWALS, smith_channel::kind::PART_KEPT]
                .contains(&rule.kind)
        {
            assert_eq!(rule.direction, Direction::FromInitiator);
        } else {
            assert_eq!(rule.direction, Direction::FromResponder);
        }
    }
}

#[test]
fn message_refusal_fixed_values_keep_each_typed_reason() {
    let fixed_values = FixedValues::new("channel", smith_channel::wire::VERSION);
    fixed_values.assert_measured("message_refused", |bytes| {
        smith_channel::MessageRefused::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("refusal decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
}

#[test]
fn channel_start_fixed_values_seal_the_optional_fixed_sixteen_byte_identity() {
    let absent = Start::decode(
        &CEILINGS,
        &mut Reader::new(&crate::support::fixed_values::fixed_value(
            "channel",
            smith_channel::wire::VERSION,
            "record_start_smallest.bin",
        )),
    )
    .expect("absent identity fixed-rule value");
    assert_eq!((&absent.identity), &None);
    let present = Start::decode(
        &CEILINGS,
        &mut Reader::new(&crate::support::fixed_values::fixed_value(
            "channel",
            smith_channel::wire::VERSION,
            "record_start_full.bin",
        )),
    )
    .expect("present identity fixed-rule value");
    assert_eq!((&present.identity), &Some([165; 16]));
}

#[test]
fn channel_turn_fixed_values_keep_the_declared_window_flag() {
    let first = smith_channel::Turn::decode(
        &CEILINGS,
        &mut Reader::new(&crate::support::fixed_values::fixed_value(
            "channel",
            smith_channel::wire::VERSION,
            "record_turn_full.bin",
        )),
    )
    .expect("window flag fixed-rule value");
    assert!(first.opens_window);
    let ordinary = smith_channel::Turn::decode(
        &CEILINGS,
        &mut Reader::new(&crate::support::fixed_values::fixed_value(
            "channel",
            smith_channel::wire::VERSION,
            "record_turn_smallest.bin",
        )),
    )
    .expect("ordinary flag fixed-rule value");
    assert!(!ordinary.opens_window);
}

#[test]
fn live_fixed_values_decode_at_their_measured_length() {
    let fixed_values = FixedValues::new("channel", smith_channel::wire::VERSION);
    fixed_values.assert_measured("live", |bytes| {
        smith_channel::Live::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("live decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
}

#[test]
fn each_grant_value_form_reads_back_by_its_tag() {
    use smith_channel::{GrantValue, Scheme};
    let rows = [
        (credential_value(Scheme::Bearer), (0, Some(Scheme::Bearer))),
        (credential_value(Scheme::Key), (0, Some(Scheme::Key))),
        (program_value(b"program"), (1, None)),
    ];
    for (value, expected) in rows {
        let bytes = value.encode(&CEILINGS).expect("bounded record");
        let read = GrantValue::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("grant decodes");
        let scheme = match read {
            GrantValue::Credential(value) => Some(value.scheme.clone()),
            GrantValue::Program(_) => None,
        };
        assert_eq!((bytes[0], scheme), expected);
    }
}

fn credential_value(scheme: smith_channel::Scheme) -> smith_channel::GrantValue {
    smith_channel::GrantValue::Credential(smith_channel::CredentialValue {
        scheme,
        bytes: skein_lib::Secret::new(Box::from(*b"fixture")),
    })
}

fn program_value(name: &[u8]) -> smith_channel::GrantValue {
    smith_channel::GrantValue::Program(smith_channel::ProgramValue { name: Box::from(name) })
}

#[test]
fn a_grant_value_past_its_bounds_is_refused() {
    use skein_codec::Reason;
    let program_bound = usize::try_from(CEILINGS.program_value_name).expect("program ceiling");
    let rows = [
        ("long program", program_value(&vec![b'a'; program_bound + 1]).encode(&CEILINGS), Reason::Bound),
        ("invalid program text", program_value(&[255]).encode(&CEILINGS), Reason::Utf8),
    ];
    for (name, result, reason) in rows {
        assert_eq!(result.err().map(|problem| problem.reason), Some(reason), "{name}");
    }
}

#[test]
fn grant_values_at_their_bounds_are_admitted() {
    let program_bound = usize::try_from(CEILINGS.program_value_name).expect("program ceiling");
    assert!(program_value(&vec![b'a'; program_bound]).encode(&CEILINGS).is_ok(), "program");
}

#[test]
fn limited_resets_distinguish_zero_one_and_not_said() {
    use skein_lib::Duration;
    for reset in [Some(Duration::ZERO), Some(Duration::from_nanos(1)), None] {
        let value = smith_channel::Limited { account: 7, reset };
        let bytes = value.encode(&CEILINGS).expect("bounded record");
        let read = smith_channel::Limited::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("limited decodes");
        assert_eq!((read.account, read.reset), (7, reset));
    }
}

#[test]
fn refused_notice_fields_read_back_at_both_bounds() {
    let code_bound = usize::try_from(CEILINGS.account_refused_code).expect("code ceiling");
    let reason_bound = usize::try_from(CEILINGS.account_refused_reason).expect("reason ceiling");
    for (code_length, reason_length) in [(0, 0), (code_bound, reason_bound)] {
        let value = smith_channel::AccountRefused {
            account: 7,
            code: vec![b'c'; code_length].into_boxed_slice(),
            reason: vec![b'r'; reason_length].into_boxed_slice(),
        };
        let bytes = value.encode(&CEILINGS).expect("bounded record");
        let read = smith_channel::AccountRefused::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("refusal decodes");
        assert_eq!(
            (read.account, read.code.as_ref().len(), read.reason.as_ref().len()),
            (7, code_length, reason_length)
        );
    }
}

#[test]
fn a_list_asks_debug_keeps_the_credentials_value_private() {
    const VALUE: &[u8] = b"private-list-value";
    for scheme in [smith_channel::Scheme::Key, smith_channel::Scheme::Bearer] {
        let credential = smith_channel::CredentialValue { scheme, bytes: skein_lib::Secret::new(VALUE.into()) };
        let ask = smith_channel::ListAsk { endpoint: 17, credential: Some(credential) };
        let words = format!("{ask:?}");
        assert!(!words.contains(std::str::from_utf8(VALUE).expect("fixture text")), "no secret text in Debug");
        assert!(!words.contains(&format!("{VALUE:?}")), "no secret bytes in Debug");
    }
}

#[test]
fn parts_and_grace_keep_their_fields_and_kinds() {
    for kind in [smith_channel::PartKind::Answer, smith_channel::PartKind::Result, smith_channel::PartKind::Window] {
        let part = smith_channel::Part {
            number: 3,
            turn: 2,
            thread: 7,
            call: 4,
            kind: kind.clone(),
            body: b"turn".as_slice().into(),
        };
        let frame = smith_channel::frame(smith_channel::Body::Part(part.clone())).expect("part frame");
        assert_eq!(frame.kind(), smith_channel::kind::PART);
        assert_eq!(
            smith_channel::Part::decode(&CEILINGS, &mut Reader::new(&frame.bytes()[8..]))
                .expect("bounded version-fourteen record"),
            part
        );
        let kept = smith_channel::KeptPart { activation: 42, turn: 2, thread: 7, call: 4, kind, body: part.body };
        let bytes = kept.encode(&CEILINGS).expect("bounded version-fourteen record");
        assert_eq!(
            smith_channel::KeptPart::decode(&CEILINGS, &mut Reader::new(&bytes))
                .expect("bounded version-fourteen record"),
            kept
        );
    }
    let kept = smith_channel::PartKept { number: 19 };
    let frame =
        smith_channel::frame(smith_channel::Body::PartKept(kept.clone())).expect("bounded version-fourteen record");
    assert_eq!(frame.kind(), smith_channel::kind::PART_KEPT);
    assert_eq!(
        smith_channel::PartKept::decode(&CEILINGS, &mut Reader::new(&frame.bytes()[8..]))
            .expect("bounded version-fourteen record"),
        kept
    );
    for grace in [None, Some(skein_lib::Duration::from_secs(17))] {
        let drain = smith_channel::Drain { grace };
        let frame =
            smith_channel::frame(smith_channel::Body::Drain(drain.clone())).expect("bounded version-fourteen record");
        let body = &frame.bytes()[8..];
        assert_eq!(
            smith_channel::Drain::decode(&CEILINGS, &mut Reader::new(body)).expect("bounded version-fourteen record"),
            drain
        );
        if grace.is_none() {
            assert_eq!(body, &[0], "no grace writes the absent option");
        }
    }
}
