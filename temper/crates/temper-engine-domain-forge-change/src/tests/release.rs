use super::*;
use crate::{Ask, Handed, Took, Way};

#[test]
fn a_release_resets_only_its_causes_allowance() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let authority = AuthorityHold {
        recovery: AuthorityRecovery { task: 7, spec: 0, hold: 2, tasks: 2 },
        lack: AuthorityLack::Tasks,
        had: 0,
        released: false,
    };
    for (cause, repairs, expected) in [
        (Cause::Repairs, 80, (0, 2, 3)),
        (Cause::Resolutions, 5, (5, 0, 3)),
        (Cause::Updates, 5, (5, 2, 0)),
        (Cause::Stalled, 5, (5, 2, 3)),
        (Cause::Runner, 5, (5, 2, 3)),
        (Cause::RedStands, 5, (5, 2, 3)),
        (Cause::CloseRefused, 5, (5, 2, 3)),
        (Cause::Rejected, 5, (5, 2, 3)),
        (Cause::Questions, 5, (5, 2, 3)),
        (Cause::Authority(authority), 5, (5, 2, 3)),
        (Cause::BranchMoved, 5, (5, 2, 3)),
        (Cause::BranchMissing, 5, (5, 2, 3)),
        (Cause::PullClosed, 5, (5, 2, 3)),
        (Cause::Retargeted, 5, (5, 2, 3)),
        (Cause::Failed, 5, (5, 2, 3)),
    ] {
        let mut held = change(State::Held {
            was: Box::new(State::Gating { head: HEAD, asked: Box::new([9]) }),
            hold: crate::Hold::new(cause, at(3)),
        });
        (held.repairs, held.resolutions, held.updates) = (repairs, 2, 3);
        held.gate_failures = Box::new([GateFailure { number: 9, count: 2 }]);
        held.clean = Box::new([HEAD]);
        let lifted = released(&held, at(10));
        let released = if cause.kind() == crate::Kind::Judgement {
            step(&lifted, &facts(), &heard(), at(10), &LIMITS).change
        } else {
            lifted
        };
        assert_eq!((released.repairs, released.resolutions, released.updates), expected, "{cause:?}");
        assert!(released.gate_failures.is_empty(), "{cause:?}");
        assert!(released.clean.is_empty(), "{cause:?}");
        assert_eq!(released.since, at(10), "{cause:?}");
    }
}

#[test]
fn a_release_takes_an_owed_landed_prefix_before_resetting() {
    let mut held = change(State::Held {
        was: Box::new(State::Ready { head: HEAD, ready_since: at(1) }),
        hold: crate::Hold::new(Cause::Repairs, at(3)),
    });
    held.way = Way::Turn;
    held.repairs = 80;
    let fresh = Facts {
        ask: Some(Handed { number: 1, landing: 19, round: 1, ask: Ask::Landed { prefix: MERGE } }),
        landing: Some(19),
        pull: Pull::Closed,
        ..facts()
    };
    let next = decide(&held, &fresh, &Heard { released: true, ..heard() });
    assert_eq!(next.took, Some(Took::Taken(1)));
    assert_eq!(next.decision, Decision::Finish { merge: MERGE });
    assert_eq!(next.change.repairs, 80, "the landing settled before starting another episode");
}

#[test]
fn a_release_keeps_a_landings_repair_ask() {
    let mut held = change(State::Held {
        was: Box::new(State::Ready { head: HEAD, ready_since: at(1) }),
        hold: crate::Hold::new(Cause::Repairs, at(3)),
    });
    held.way = Way::Turn;
    held.repairs = LIMITS.repairs;
    let why = Repair::LandingAt { gate: 1, candidate: MERGE, below: None };
    let fresh = Facts {
        ask: Some(Handed {
            number: 1,
            landing: 19,
            round: 1,
            ask: Ask::Repair { gate: 1, candidate: MERGE, below: None },
        }),
        landing: Some(19),
        ..facts()
    };
    let recovered = decide(&held, &fresh, &Heard { released: true, ..heard() });
    let asked = decide(&recovered.change, &fresh, &heard());
    assert_eq!(asked.took, Some(Took::Taken(1)));
    assert_eq!(asked.decision, Decision::Delegate(Delegate::Repair(why)));
    assert_eq!(asked.change.repairs, 1);
}

#[test]
fn a_persons_rejection_needs_a_new_answer_and_a_release() {
    let gate = Gate { number: 9, kind: GateKind::Person, freshness: Freshness::Exact, blocking: true, eager: false };
    let mut live = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    live.gates = Box::new([gate]);
    let mut read = Facts {
        gates: Box::new([GateReport { number: 9, head: HEAD, status: Status::Failed, revision: 0 }]),
        ..facts()
    };
    let stopped = decide(&live, &read, &heard());
    assert_eq!(stopped.decision, Decision::Hold(Cause::Rejected));
    let held = stopped.change;
    for measure in [false, true] {
        read.measure = measure;
        let replay = decide(&released(&held, at(1)), &read, &heard());
        assert_eq!(replay.decision, Decision::Hold(Cause::Rejected));
        let State::Held { hold, .. } = replay.change.state else { panic!("replayed rejection") };
        assert!(hold.words.starts_with(b"not measured again: its rejection at 010101010101 stands;"));
    }
    read.gates[0].status = Status::Passed;
    let read_only = decide(&held, &read, &heard());
    assert_eq!(read_only.decision, Decision::None);
    assert_eq!(read_only.change, held, "even a fresh green forge read and a new answer don't release it");
    let lifted = decide(&released(&held, at(1)), &read, &heard());
    assert_eq!(lifted.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
    assert!(matches!(decide(&lifted.change, &read, &heard()).change.state, State::Queued { head: HEAD, .. }));
}
