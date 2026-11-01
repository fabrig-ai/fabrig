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
fn offered_acts_fill_their_bound_exactly() {
    use crate::contract::{ActKind, MAX_OFFERED_ACTS};
    let acts = [
        ActKind::Message,
        ActKind::Answer,
        ActKind::Take,
        ActKind::HandBack,
        ActKind::Accept,
        ActKind::Reject,
        ActKind::PassUp,
        ActKind::Release,
        ActKind::LeaveHeld,
        ActKind::Amend,
        ActKind::Stop,
        ActKind::Cancel,
        ActKind::Move,
        ActKind::Reorder,
        ActKind::Seen,
        ActKind::EditNote,
        ActKind::DeleteNote,
        ActKind::Adopt,
        ActKind::ChangePolicy,
        ActKind::Rename,
        ActKind::SetRoles,
        ActKind::MakeService,
        ActKind::SetPool,
        ActKind::Withdraw,
        ActKind::RemoveProject,
    ];
    assert_eq!(u32::try_from(acts.len()).unwrap(), MAX_OFFERED_ACTS);
    for (at, act) in acts.into_iter().enumerate() {
        let position = match act {
            ActKind::Message => 0,
            ActKind::Answer => 1,
            ActKind::Take => 2,
            ActKind::HandBack => 3,
            ActKind::Accept => 4,
            ActKind::Reject => 5,
            ActKind::PassUp => 6,
            ActKind::Release => 7,
            ActKind::LeaveHeld => 8,
            ActKind::Amend => 9,
            ActKind::Stop => 10,
            ActKind::Cancel => 11,
            ActKind::Move => 12,
            ActKind::Reorder => 13,
            ActKind::Seen => 14,
            ActKind::EditNote => 15,
            ActKind::DeleteNote => 16,
            ActKind::Adopt => 17,
            ActKind::ChangePolicy => 18,
            ActKind::Rename => 19,
            ActKind::SetRoles => 20,
            ActKind::MakeService => 21,
            ActKind::SetPool => 22,
            ActKind::Withdraw => 23,
            ActKind::RemoveProject => 24,
        };
        assert_eq!(at, position);
    }
}
