//! Offer fences across every restart moment (`jig/docs/design/domain/testing.md`, 5).

use crate::decision_reads::{Restart, decision_restart_cases};

#[test]
fn decisions_keep_their_offered_fence_and_room_across_each_restart_cut() {
    for proposal in [false, true] {
        decision_restart_cases(
            proposal,
            &[Restart::Uncut, Restart::BeforeDecision, Restart::AfterDecision, Restart::AfterTurn],
        );
    }
}
