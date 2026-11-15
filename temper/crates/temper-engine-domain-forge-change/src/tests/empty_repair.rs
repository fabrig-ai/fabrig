//! Empty CI repairs check their head again (domain/forge/change/repairs.md).
use super::*;

fn repairing(why: Repair) -> Change {
    let mut c = change(State::Repairing { requested: true, why });
    c.repairs = 1;
    c.owns_turn = true;
    c
}

#[test]
fn an_empty_ci_repair_runs_one_job_again_before_updating() {
    for why in [Repair::Ci, Repair::Semantic] {
        let c = repairing(why);
        let f = Facts {
            taken: None,
            rerun: true,
            ci: Ci { head: HEAD, status: Status::Failed },
            contains_base: Status::Failed,
            ..facts()
        };
        let next = decide(&c, &f, &Heard { delegate: Status::Passed, ..heard() });
        assert_eq!(next.decision, Decision::Effect(Effect::Rerun { head: HEAD }));
        assert_eq!(next.change.state, State::Checking { head: HEAD });
        assert_eq!(next.change.last_head, Some(HEAD));
        assert!(next.change.owns_turn);
        assert_eq!((next.change.repairs, next.change.updates), (1, 0));
        assert_eq!(next.change.since, at(1));
    }
}

#[test]
fn an_empty_ci_repair_without_a_job_updates_a_behind_head() {
    let mut c = repairing(Repair::Ci);
    c.ready_since = Some(at(0));
    let f = Facts { ci: Ci { head: HEAD, status: Status::Failed }, contains_base: Status::Failed, ..facts() };
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    assert_eq!(next.change.state, State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
    assert_eq!((next.change.repairs, next.change.updates), (1, 1));
    let updated =
        decide(&next.change, &Facts { branch: Some(NEW), ..f.clone() }, &Heard { effect: EffectResult::Made, ..h });
    assert_eq!(updated.change.state, State::Checking { head: NEW });
    assert_eq!(updated.change.last_head, Some(NEW));
    c.updates = LIMITS.updates;
    assert_eq!(decide(&c, &f, &h).decision, Decision::Hold(Hold::Updates));
}

#[test]
fn empty_ci_repairs_leave_red_standing_and_spend_the_repair_bound() {
    let f = Facts { ci: Ci { head: HEAD, status: Status::Failed }, ..facts() };
    let h = Heard { delegate: Status::Passed, ..heard() };
    let mut c = repairing(Repair::Ci);
    for count in 1..=LIMITS.repairs {
        let checked = decide(&c, &f, &h);
        assert_eq!(checked.change.state, State::Checking { head: HEAD });
        assert_eq!(checked.decision, Decision::None);
        assert_eq!(checked.change.repairs, count);
        let next = decide(&checked.change, &f, &heard());
        if count < LIMITS.repairs {
            assert_eq!(next.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
            assert_eq!(next.change.repairs, count + 1);
        } else {
            assert_eq!(next.decision, Decision::Hold(Hold::Repairs));
            assert_eq!(next.change.repairs, LIMITS.repairs);
        }
        c = next.change;
    }
}

#[test]
fn an_empty_ci_repair_at_green_ends_as_a_green_repair() {
    for why in [Repair::Ci, Repair::Semantic] {
        let next = decide(&repairing(why), &facts(), &Heard { delegate: Status::Passed, ..heard() });
        assert_eq!(next.decision, Decision::RepairEnded { head: HEAD });
        assert_eq!(next.change.state, State::Checking { head: HEAD });
        assert_eq!(next.change.repairs, 1);
    }
}

#[test]
fn failed_repairs_and_non_ci_empty_repairs_still_hold() {
    // Failed and cancelled terminals both arrive as Failed; non-CI
    // no_change results keep that coercion at the forge boundary.
    for why in [
        Repair::Ci,
        Repair::Semantic,
        Repair::Gate(9),
        Repair::Review { head: HEAD },
        Repair::Landing { gate: 9 },
        Repair::LandingAt { candidate: HEAD, gate: 9, below: None },
    ] {
        let f = Facts { rerun: true, ci: Ci { head: HEAD, status: Status::Failed }, ..facts() };
        let next = decide(&repairing(why), &f, &Heard { delegate: Status::Failed, ..heard() });
        assert_eq!(next.decision, Decision::Hold(Hold::Failed));
    }
}
