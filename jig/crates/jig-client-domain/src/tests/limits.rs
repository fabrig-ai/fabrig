use crate::Limits;

#[test]
fn turn_text_at_either_receiving_bound_is_valid() {
    let base = super::fixtures::limits();
    // Leave room between the field bound and the existing turn-byte bound.
    let text = base.text.checked_sub(1).unwrap();
    for turn_text in [text, base.turn_bytes] {
        let limits = Limits { text, turn_text, ..base };
        assert!(crate::valid(&limits), "turn_text={turn_text}");
    }
}

#[test]
fn turn_text_outside_either_receiving_bound_is_refused() {
    let base = super::fixtures::limits();
    for turn_text in [base.text.checked_sub(1).unwrap(), base.turn_bytes.checked_add(1).unwrap()] {
        let limits = Limits { turn_text, ..base };
        assert!(!crate::valid(&limits), "turn_text={turn_text}");
    }
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    use crate::{Backoff, valid};
    use skein_lib::{Duration, RequestLimits};
    let base = super::fixtures::limits();
    assert!(valid(&base));
    let cases = [
        Limits { turn_text: base.text - 1, ..base },
        Limits { turn_text: base.turn_bytes + 1, ..base },
        Limits { watches: 0, ..base },
        Limits { backoff: Backoff { first: Duration::ZERO, ..base.backoff }, ..base },
        Limits { backoff: Backoff { first: Duration::from_secs(1), ..base.backoff }, ..base },
        Limits { requests: RequestLimits { out: 0, ..base.requests }, ..base },
        Limits { requests: RequestLimits { first: Duration::ZERO, ..base.requests }, ..base },
        Limits { requests: RequestLimits { first: Duration::from_secs(1), ..base.requests }, ..base },
    ];
    for (case, limits) in cases.into_iter().enumerate() {
        assert!(!valid(&limits), "case {case}");
    }
}

#[test]
fn validity_does_not_price_retained_payloads() {
    let base = super::fixtures::limits();
    for limits in [
        Limits { document_bytes: u64::MAX, ..base },
        Limits { requests: skein_lib::RequestLimits { bytes: u64::MAX, ..base.requests }, ..base },
    ] {
        assert!(crate::valid(&limits));
    }
}

#[test]
fn long_text_at_either_receiving_bound_is_valid() {
    let base = super::fixtures::limits();
    for long_text in [base.text, u32::try_from(base.document_bytes).unwrap()] {
        let ask_bytes = u64::from(long_text) + u64::try_from(size_of::<crate::contract::Ask>()).unwrap();
        assert!(crate::valid(&Limits { ask_bytes, long_text, ..base }), "long_text={long_text}");
    }
}

#[test]
fn long_text_outside_either_receiving_bound_is_refused() {
    let base = super::fixtures::limits();
    for long_text in [base.text - 1, u32::try_from(base.document_bytes).unwrap() + 1] {
        assert!(!crate::valid(&Limits { long_text, ask_bytes: u64::MAX, ..base }), "long_text={long_text}");
    }
}

#[test]
fn an_ask_without_room_for_long_text_and_its_record_is_refused() {
    let base = super::fixtures::limits();
    let ask_bytes = u64::from(base.long_text) + u64::try_from(size_of::<crate::contract::Ask>()).unwrap() - 1;
    assert!(!crate::valid(&Limits { ask_bytes, ..base }));
}
