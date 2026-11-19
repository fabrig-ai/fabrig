//! A goal's reds (temper/docs/design/domain/red-findings.md, 10).
use super::{Stands, stands, uncleared_first, w};
use alloc::boxed::Box;

fn row_parts(state: w::FindingState) -> w::FindingRowParts {
    w::FindingRowParts {
        number: 8,
        kind: w::FindingKind::Red,
        severity: w::Severity::Medium,
        area: None,
        state,
        title: b"A red test".as_slice().into(),
        holder: w::Holder::Role(
            w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number: 3 }).expect("a role within wire limits"),
        ),
        at: 17,
        why: None,
        hander: None,
        goal: None,
        reason: None,
        cleared: None,
    }
}

fn row(parts: w::FindingRowParts) -> w::FindingRow {
    w::FindingRow::new(&w::CEILINGS, parts).expect("a finding within wire limits")
}

fn clearing() -> w::FindingClearing {
    w::FindingClearing::new(
        &w::CEILINGS,
        w::FindingClearingParts { tier: b"nightly".as_slice().into(), run: 412, at: 23 },
    )
    .expect("a clearing within wire limits")
}

#[test]
fn clearing_takes_precedence_over_every_triage_state() {
    for state in [w::FindingState::New, w::FindingState::Routed, w::FindingState::Backlog, w::FindingState::Dropped] {
        let finding = row(w::FindingRowParts {
            goal: Some(830),
            reason: Some(b"No repair planned".as_slice().into()),
            cleared: Some(clearing()),
            ..row_parts(state)
        });
        assert_eq!(stands(&finding), Stands::Cleared { tier: b"nightly".as_slice().into(), run: 412 });
    }
}

#[test]
fn new_is_waiting() {
    assert_eq!(stands(&row(row_parts(w::FindingState::New))), Stands::Waiting);
}

#[test]
fn routed_is_repairing_its_optional_goal() {
    for goal in [Some(830), None] {
        let finding = row(w::FindingRowParts { goal, ..row_parts(w::FindingState::Routed) });
        assert_eq!(stands(&finding), Stands::Repairing(goal));
    }
}

#[test]
fn backlog_stands_in_the_backlog() {
    assert_eq!(stands(&row(row_parts(w::FindingState::Backlog))), Stands::Backlog);
}

#[test]
fn dropped_keeps_its_reason_or_empty_bytes() {
    for reason in [Some(Box::from(b"No repair planned".as_slice())), None] {
        let finding = row(w::FindingRowParts { reason: reason.clone(), ..row_parts(w::FindingState::Dropped) });
        let expected = reason.unwrap_or_default();
        assert_eq!(stands(&finding), Stands::Dropped(expected));
    }
}

#[test]
fn uncleared_first_preserves_order_within_both_groups() {
    let rows = [
        row(w::FindingRowParts { number: 9, cleared: Some(clearing()), ..row_parts(w::FindingState::Routed) }),
        row(w::FindingRowParts { number: 8, ..row_parts(w::FindingState::Dropped) }),
        row(w::FindingRowParts { number: 7, cleared: Some(clearing()), ..row_parts(w::FindingState::New) }),
        row(w::FindingRowParts { number: 6, ..row_parts(w::FindingState::Backlog) }),
    ];
    let ordered = uncleared_first(&rows);
    let [nine, eight, seven, six] = &rows;
    assert_eq!(ordered.as_ref(), &[eight.clone(), six.clone(), nine.clone(), seven.clone()]);
}
