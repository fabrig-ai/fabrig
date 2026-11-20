use super::fixtures::panel as p;
use crate::forge::{
    change::{self, Ordinal as O, Slots},
    words::ChangeWord as W,
};
use skein_lib::List;
use skein_ui::phrases::Hole;
use temper_client_protocol_wire::v1 as w;
fn repair(reason: w::Reason) -> w::Step {
    w::Step::Repairing(w::Repairing { reason })
}
#[test]
fn each_step_supplies_its_slot_line_and_held_word() {
    let cases = [
        (w::Step::Ready(w::InLanding { landing: Some(61) }), Some(4), W::ReadyLine, W::HeldReady),
        (w::Step::Ready(w::InLanding { landing: None }), Some(4), W::ReadyAloneLine, W::HeldReady),
        (w::Step::Preparing(w::InLanding { landing: Some(61) }), Some(4), W::PrepareLine, W::HeldPreparing),
        (w::Step::Producing, Some(0), W::ProduceLine, W::HeldProducing),
        (w::Step::Opening, Some(1), W::OpenLine, W::HeldOpening),
        (w::Step::Recreating, Some(1), W::RecreateLine, W::HeldRecreating),
        (w::Step::Reopening, Some(1), W::ReopenLine, W::HeldReopening),
        (w::Step::Checking, Some(2), W::CheckLine, W::HeldChecking),
        (w::Step::Gating, Some(3), W::GateLine, W::HeldGating),
        (p::queued(), Some(4), W::QueueTh, W::HeldQueued),
        (w::Step::First, Some(4), W::FirstLine, W::HeldFirst),
        (w::Step::Updating, Some(4), W::UpdateLine, W::HeldUpdating),
        (w::Step::Resolving, Some(4), W::ResolveLine, W::HeldResolving),
        (repair(w::Reason::Ci), Some(2), W::RepairCi, W::HeldRepairing),
        (repair(w::Reason::Gate(w::GateReason { gate: 123 })), Some(3), W::RepairGate, W::HeldRepairing),
        (repair(w::Reason::Semantic), Some(4), W::RepairSemantic, W::HeldRepairing),
        (repair(w::Reason::Review), Some(3), W::RepairReview, W::HeldRepairing),
        (w::Step::Landing, Some(5), W::LandingLine, W::HeldLanding),
        (w::Step::Landed(w::Landed { merge: [0xab; 32] }), None, W::LandedLine, W::HeldLanded),
    ];
    for (step, current, word, held) in cases {
        assert_eq!(change::slot(&step), Slots { current, done: current.unwrap_or(6) });
        assert_eq!(change::held_step(&step), held);
        let panel = p::panel(17, step);
        let (line, holes) = change::line(&panel);
        assert_eq!(line, word);
        if word == W::RepairReview {
            let mut found = false;
            for (candidate, literals, kinds) in crate::forge::words::CHANGES {
                if candidate == word {
                    assert_eq!(literals, &[b"Repairing: a person asked for changes on the forge.".as_slice()]);
                    assert_eq!((kinds).len(), 0);
                    found = true;
                }
            }
            assert!(found);
        }
        let expected: Box<[Hole]> = if word == W::QueueTh {
            Box::new([Hole::Number(65), Hole::Name(panel.landing.clone())])
        } else if word == W::PrepareLine || word == W::FirstLine || word == W::UpdateLine || word == W::ResolveLine {
            Box::new([Hole::Name(panel.landing.clone())])
        } else if word == W::ReadyLine {
            Box::new([Hole::Number(61)])
        } else if word == W::RepairGate {
            Box::new([Hole::Number(123)])
        } else if word == W::LandedLine {
            Box::new([Hole::Name(Box::from(*b"abababa"))])
        } else {
            Box::new([])
        };
        assert_eq!(holes.len(), expected.len());
        for (actual, expected) in holes.iter().zip(expected.iter()) {
            match (actual, expected) {
                (Hole::Number(actual), Hole::Number(expected)) => assert_eq!(actual, expected),
                (Hole::Name(actual), Hole::Name(expected)) => assert_eq!(actual, expected),
                _ => panic!("line hole kind"),
            }
        }
    }
}
#[test]
fn a_panel_supplies_its_outbox_wait_phrase() {
    for (wait, expected) in [(Some(w::ChangeWait::OutboxRoom), Some(W::OutboxRoom)), (None, None)] {
        let mut parts = p::panel(17, w::Step::Opening);
        parts.wait = wait;
        let state = crate::forge::panel::State::Shown(Box::new(parts));
        assert_eq!(change::shown(&state).unwrap().wait(), expected);
    }
    let mut found = false;
    for row in crate::forge::words::CHANGES {
        if row.0 == W::OutboxRoom {
            found = true;
            assert_eq!(row.1, &[b"Waiting for room in the forge outbox.".as_slice()]);
            assert_eq!(row.2, []);
        }
    }
    assert!(found);
}
#[test]
fn holds_supply_each_reason() {
    for (hold, word) in [
        (w::Hold::Stalled, W::Stalled),
        (w::Hold::Repairs, W::RepairsSpent),
        (w::Hold::Questions, W::QuestionsSpent),
        (w::Hold::Resolutions, W::ResolutionsSpent),
        (w::Hold::Updates, W::UpdatesSpent),
        (w::Hold::Moved, W::Moved),
        (w::Hold::Missing, W::Missing),
        (w::Hold::Closed, W::Closed),
        (w::Hold::Retargeted, W::Retargeted),
        (w::Hold::Failed, W::FailedHold),
        (w::Hold::Rejected, W::Rejected),
    ] {
        assert_eq!(change::hold(&hold), word);
    }
}
#[test]
fn queue_names_stay_nearest_first_and_unnamed_counts_saturate() {
    for (place, names, unnamed) in
        [(1, &[][..], 0), (3, &[7, 9][..], 0), (4, &[7, 9][..], 1), (3, &[][..], 2), (0, &[7, 9][..], 0)]
    {
        let mut ahead = List::with_capacity(2);
        for name in names {
            ahead.push(*name).unwrap();
        }
        let step = w::Step::Queued(w::Queued { place, ahead });
        let queue = change::queue(&step).unwrap();
        assert_eq!((queue.place, queue.ahead, queue.unnamed), (place, names, unnamed));
    }
    let first = change::queue(&w::Step::First).unwrap();
    assert_eq!((first.place, first.ahead, first.unnamed), (1, &[][..], 0));
    assert!(change::queue(&w::Step::Producing).is_none());
}
#[test]
fn ordinals_keep_the_teens_and_the_last_digit() {
    for (number, expected) in [
        (1, O::St),
        (2, O::Nd),
        (3, O::Rd),
        (4, O::Th),
        (11, O::Th),
        (12, O::Th),
        (13, O::Th),
        (21, O::St),
        (22, O::Nd),
        (101, O::St),
        (111, O::Th),
        (112, O::Th),
        (23, O::Rd),
        (113, O::Th),
        (u32::MAX, O::Th),
    ] {
        assert_eq!(change::ordinal(number), expected);
    }
}
#[test]
fn merge_digits_are_lower_case_and_stop_at_seven_nibbles() {
    let mut merge = [0; 32];
    for (byte, value) in merge.iter_mut().zip([0x01, 0x23, 0xab, 0xcd]) {
        *byte = value;
    }
    let panel = p::panel(17, w::Step::Landed(w::Landed { merge }));
    let (_, holes) = change::line(&panel);
    let [Hole::Name(digits)] = holes.as_ref() else { panic!("merge digits") };
    assert_eq!(digits.as_ref(), b"0123abc");
}

#[test]
fn after_landing_steps_keep_the_merge_tiers_and_completed_slots() {
    let mut tiers = List::with_capacity(1);
    tiers.push(Box::from(b"fast run 2".as_slice())).unwrap();
    for (step, word, held) in [
        (
            w::Step::Verifying(w::Verifying { merge: [0xab; 32], tiers: tiers.clone() }),
            W::VerifyingLine,
            W::HeldVerifying,
        ),
        (w::Step::Red(w::Red { merge: [0xab; 32], tiers: tiers.clone() }), W::RedLine, W::HeldRed),
        (
            w::Step::Cleared(w::Cleared { merge: [0xab; 32], tiers: tiers.clone(), uncleared: false }),
            W::ClearedLine,
            W::HeldCleared,
        ),
        (
            w::Step::Cleared(w::Cleared { merge: [0xab; 32], tiers: tiers.clone(), uncleared: true }),
            W::UnclearedLine,
            W::HeldUncleared,
        ),
    ] {
        assert_eq!(change::slot(&step), Slots { current: None, done: 6 });
        assert_eq!(change::held_step(&step), held);
        let panel = p::panel(17, step);
        assert_eq!(change::line(&panel).0, word);
        let state = crate::forge::panel::State::Shown(Box::new(panel));
        let shown = change::shown(&state).unwrap();
        assert_eq!(shown.tiers(), tiers.as_slice());
    }
}

#[test]
fn a_no_test_hold_names_the_repairs_finding() {
    let word = change::hold(&w::Hold::NoTest);
    assert_eq!(word, W::NoTestHold);
    let mut found = false;
    for row in crate::forge::words::CHANGES {
        if row.0 == word {
            assert_eq!(row.1, &[b"its repair found no failed test".as_slice()]);
            assert_eq!(row.2, []);
            found = true;
        }
    }
    assert!(found);
}
