//! A green head ends only CI repair (temper/docs/design/domain/forge.md, 8.4).
use super::*;

fn repairing(why: Repair, requested: bool) -> Change {
    let mut c = change(State::Repairing { requested, why });
    c.repairs = 1;
    c.clean = Box::new([BASE]);
    c.owns_turn = true;
    c
}

#[test]
fn green_ci_ends_due_and_failed_repairs_at_the_current_head() {
    for why in [Repair::Ci, Repair::Semantic] {
        for (requested, status) in [(false, Status::Unknown), (true, Status::Failed)] {
            for (way, state) in [
                (crate::Way::Queue, State::Checking { head: NEW }),
                (crate::Way::Turn, State::Gating { head: NEW, asked: Box::new([]) }),
            ] {
                let mut c = repairing(why, requested);
                c.way = way;
                let mut f = facts();
                f.branch = Some(NEW);
                f.ci = Ci { head: NEW, status: Status::Passed };
                let next = decide(&c, &f, &Heard { delegate: status, ..heard() });
                assert_eq!(next.decision, Decision::RepairEnded { head: NEW });
                assert_eq!(next.change.state, state);
                assert_eq!(next.change.last_head, Some(NEW));
                assert_eq!(next.change.repairs, 1);
                assert!(next.change.clean.is_empty());
                assert!(!next.change.owns_turn);
                assert_eq!(next.change.since, at(1));
            }
        }
    }
}

#[test]
fn green_ci_cancels_a_live_repair_without_a_run() {
    for why in [Repair::Ci, Repair::Semantic] {
        let c = repairing(why, true);
        let mut f = facts();
        f.delegate = Some(Delegate::Repair(why));
        let next = decide(&c, &f, &heard());
        assert_eq!(next.decision, Decision::CancelRepair { head: HEAD });
        assert_eq!(next.change, c);
    }
}

#[test]
fn green_ci_leaves_running_and_unmade_tasks_alone() {
    let c = repairing(Repair::Ci, true);
    for (delegate, running) in [(None, false), (Some(Delegate::Repair(Repair::Ci)), true)] {
        let mut f = facts();
        f.delegate = delegate;
        let next = decide(&c, &f, &Heard { delegate_running: running, ..heard() });
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
        assert_eq!(next.change, c);
    }
}

#[test]
fn a_successful_repair_delivers_its_head_as_ever() {
    let c = repairing(Repair::Ci, true);
    let mut f = facts();
    f.delegate = Some(Delegate::Repair(Repair::Ci));
    f.branch = Some(NEW);
    f.ci = Ci { head: NEW, status: Status::Passed };
    let next = decide(&c, &f, &Heard { delegate: Status::Passed, delegate_running: true, ..heard() });
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.state, State::Checking { head: NEW });
    assert_eq!(next.change.repairs, 1);
}

#[test]
fn ci_on_another_head_or_not_passed_keeps_the_repair() {
    for ci in [
        Ci { head: BASE, status: Status::Passed },
        Ci { head: HEAD, status: Status::Unknown },
        Ci { head: HEAD, status: Status::Pending },
        Ci { head: HEAD, status: Status::Failed },
    ] {
        let c = repairing(Repair::Ci, true);
        let mut f = facts();
        f.ci = ci;
        f.delegate = Some(Delegate::Repair(Repair::Ci));
        assert_eq!(decide(&c, &f, &heard()).decision, Decision::Wait { until: Some(at(30)) });
        assert_eq!(
            decide(&c, &f, &Heard { delegate: Status::Failed, ..heard() }).decision,
            Decision::Hold(Hold::Failed)
        );
    }
}

#[test]
fn green_ci_leaves_review_and_landing_repairs_alone() {
    for why in [Repair::Gate(9), Repair::Landing { gate: 3 }, Repair::Review { head: BASE }] {
        let c = repairing(why, true);
        let mut f = facts();
        f.delegate = Some(Delegate::Repair(why));
        assert_eq!(decide(&c, &f, &heard()).decision, Decision::Wait { until: Some(at(30)) });
        assert_eq!(
            decide(&c, &f, &Heard { delegate: Status::Failed, ..heard() }).decision,
            Decision::Hold(Hold::Failed)
        );
    }
}

#[test]
fn green_ci_after_gates_end_requests_no_repair() {
    let c = change(State::CancellingGates { head: HEAD });
    let mut f = facts();
    f.live_gate = Some(HEAD);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Wait { until: Some(at(30)) });
    f.live_gate = None;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.state, State::Checking { head: HEAD });
    assert_eq!(next.change.repairs, 0);
}

#[test]
fn a_release_at_green_resets_the_count_and_ends_ci_repair() {
    for why in
        [Hold::Failed, Hold::Stalled, Hold::Repairs, Hold::Questions, Hold::Resolutions, Hold::Updates, Hold::Rejected]
    {
        let c = change(State::Held { was: Box::new(repairing(Repair::Ci, true).state), why });
        let released = decide(&c, &facts(), &Heard { released: true, ..heard() });
        let next = decide(&released.change, &facts(), &Heard { delegate: Status::Failed, ..heard() });
        assert_eq!(next.decision, Decision::RepairEnded { head: HEAD });
        assert_eq!(next.change.state, State::Checking { head: HEAD });
        assert_eq!(next.change.repairs, 0);
    }
}

#[test]
fn a_bare_authority_release_at_green_does_not_rehold_ci() {
    let cause = crate::AuthorityHold {
        recovery: crate::AuthorityRecovery { task: 7, spec: 0, hold: 1, tasks: 2 },
        lack: crate::AuthorityLack::Tasks,
        had: 2,
        released: false,
    };
    for was in [
        State::Repairing { requested: false, why: Repair::Ci },
        State::Repairing { requested: false, why: Repair::Semantic },
        State::Checking { head: HEAD },
        State::CancellingGates { head: HEAD },
    ] {
        let c = change(State::Held { was: Box::new(was), why: Hold::Authority(cause) });
        let next = decide(&c, &facts(), &Heard { released: true, lacking: Some(cause), ..heard() });
        assert_eq!(next.decision, Decision::None);
        assert_eq!(next.change.released, None);
        assert!(!matches!(next.change.state, State::Held { .. }));
    }
    for was in [
        State::Gating { head: HEAD, asked: Box::new([]) },
        State::Repairing { requested: false, why: Repair::Gate(9) },
        State::Repairing { requested: false, why: Repair::Landing { gate: 3 } },
    ] {
        let c = change(State::Held { was: Box::new(was), why: Hold::Authority(cause) });
        assert_eq!(
            decide(&c, &facts(), &Heard { released: true, lacking: Some(cause), ..heard() }).decision,
            Decision::Hold(Hold::Authority(crate::AuthorityHold { released: true, ..cause }))
        );
    }
}

fn authority() -> Hold {
    Hold::Authority(crate::AuthorityHold {
        recovery: crate::AuthorityRecovery { task: 7, spec: 0, hold: 1, tasks: 2 },
        lack: crate::AuthorityLack::Tasks,
        had: 2,
        released: false,
    })
}

#[test]
fn green_ci_ends_only_its_own_hold_without_resetting_counters() {
    for reason in [Repair::Ci, Repair::Semantic] {
        for why in [Hold::Failed, Hold::Stalled, Hold::Repairs, authority()] {
            let mut c = repairing(reason, true);
            c.state = State::Held { was: Box::new(c.state), why };
            c.resolutions = 2;
            c.updates = 3;
            let next = decide(&c, &facts(), &heard());
            assert_eq!(next.decision, Decision::EndHold);
            assert!(!matches!(next.change.state, State::Held { .. }));
            assert_eq!((next.change.repairs, next.change.resolutions, next.change.updates), (1, 2, 3));
            assert_eq!(next.change.since, at(1));
            assert!(!next.change.owns_turn);
            assert_eq!(next.change.released, None);
        }
    }
    for was in [State::Checking { head: HEAD }, State::CancellingGates { head: HEAD }] {
        for why in [Hold::Repairs, authority()] {
            let c = change(State::Held { was: Box::new(was.clone()), why });
            let next = decide(&c, &facts(), &heard());
            assert_eq!(next.decision, Decision::EndHold);
            assert_eq!(next.change.state, was);
        }
    }
}

#[test]
fn green_ci_leaves_other_holds_and_reasons_held() {
    for was in [
        State::Repairing { requested: true, why: Repair::Gate(9) },
        State::Repairing { requested: true, why: Repair::Landing { gate: 3 } },
        State::Gating { head: HEAD, asked: Box::new([]) },
    ] {
        for why in [Hold::Failed, Hold::Stalled, Hold::Repairs, authority()] {
            let c = change(State::Held { was: Box::new(was.clone()), why });
            assert_eq!(decide(&c, &facts(), &heard()).change, c);
        }
    }
    for why in [
        Hold::BranchMoved,
        Hold::BranchMissing,
        Hold::Retargeted,
        Hold::PullClosed,
        Hold::Questions,
        Hold::Rejected,
        Hold::Resolutions,
        Hold::Updates,
    ] {
        let c = change(State::Held { was: Box::new(repairing(Repair::Ci, true).state), why });
        assert_eq!(decide(&c, &facts(), &heard()).change, c);
    }
    for was in [State::Checking { head: HEAD }, State::CancellingGates { head: HEAD }] {
        for why in [Hold::Failed, Hold::Stalled] {
            let c = change(State::Held { was: Box::new(was.clone()), why });
            assert_eq!(decide(&c, &facts(), &heard()).change, c);
        }
    }
}

#[test]
fn a_held_repair_needs_green_at_head_and_no_run_or_drift() {
    let c = change(State::Held { was: Box::new(repairing(Repair::Ci, true).state), why: Hold::Failed });
    for (ci, drift, writer, pull, running) in [
        (Ci { head: BASE, status: Status::Passed }, None, false, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Unknown }, None, false, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Pending }, None, false, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Failed }, None, false, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Passed }, Some(Hold::BranchMoved), false, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Passed }, None, true, Pull::Missing, false),
        (Ci { head: HEAD, status: Status::Passed }, None, false, Pull::Closed, false),
        (Ci { head: HEAD, status: Status::Passed }, None, false, Pull::Missing, true),
    ] {
        let f = Facts { ci, drift, writer_taken: writer, pull, ..facts() };
        let next = decide(&c, &f, &Heard { delegate_running: running, ..heard() });
        assert_eq!(next.decision, Decision::None);
        assert_eq!(next.change, c);
    }
}
