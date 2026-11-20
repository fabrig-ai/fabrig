//! A goal's reds (temper/docs/design/domain/red-findings.md, 10).
use super::{Stands, stands, uncleared_first, w};
use alloc::boxed::Box;

fn row(state: w::FindingState) -> w::FindingRow {
    w::FindingRow {
        number: 8,
        kind: w::FindingKind::Red,
        severity: w::Severity::Medium,
        area: None,
        state,
        title: b"A red test".as_slice().into(),
        holder: w::Holder::Role(w::HolderRole { number: 3 }),
        at: 17,
        why: None,
        hander: None,
        goal: None,
        reason: None,
        cleared: None,
    }
}

fn clearing() -> w::FindingClearing {
    w::FindingClearing { tier: b"nightly".as_slice().into(), run: 412, at: 23 }
}

#[test]
fn clearing_takes_precedence_over_every_triage_state() {
    for state in [w::FindingState::New, w::FindingState::Routed, w::FindingState::Backlog, w::FindingState::Dropped] {
        let finding = w::FindingRow {
            goal: Some(830),
            reason: Some(b"No repair planned".as_slice().into()),
            cleared: Some(clearing()),
            ..row(state)
        };
        assert_eq!(stands(&finding), Stands::Cleared { tier: b"nightly".as_slice().into(), run: 412 });
    }
}

#[test]
fn new_is_waiting() {
    assert_eq!(stands(&row(w::FindingState::New)), Stands::Waiting);
}

#[test]
fn routed_is_repairing_its_optional_goal() {
    for goal in [Some(830), None] {
        let finding = w::FindingRow { goal, ..row(w::FindingState::Routed) };
        assert_eq!(stands(&finding), Stands::Repairing(goal));
    }
}

#[test]
fn backlog_stands_in_the_backlog() {
    assert_eq!(stands(&row(w::FindingState::Backlog)), Stands::Backlog);
}

#[test]
fn dropped_keeps_its_reason_or_empty_bytes() {
    for reason in [Some(Box::from(b"No repair planned".as_slice())), None] {
        let finding = w::FindingRow { reason: reason.clone(), ..row(w::FindingState::Dropped) };
        let expected = reason.unwrap_or_default();
        assert_eq!(stands(&finding), Stands::Dropped(expected));
    }
}

#[test]
fn uncleared_first_preserves_order_within_both_groups() {
    let rows = [
        w::FindingRow { number: 9, cleared: Some(clearing()), ..row(w::FindingState::Routed) },
        w::FindingRow { number: 8, ..row(w::FindingState::Dropped) },
        w::FindingRow { number: 7, cleared: Some(clearing()), ..row(w::FindingState::New) },
        w::FindingRow { number: 6, ..row(w::FindingState::Backlog) },
    ];
    let ordered = uncleared_first(&rows);
    let [nine, eight, seven, six] = &rows;
    assert_eq!(ordered.as_ref(), &[eight.clone(), six.clone(), nine.clone(), seven.clone()]);
}
