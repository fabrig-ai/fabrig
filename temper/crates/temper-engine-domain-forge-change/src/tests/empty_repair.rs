//! Empty CI repairs check their head again (domain/forge/change/repairs.md).
use super::*;

fn repairing(why: Repair) -> Change {
    let mut c = change(State::Repairing { requested: true, why });
    c.repairs = 1;
    c.owns_turn = true;
    c
}

#[test]
fn an_empty_ci_repair_runs_one_job_again_at_its_head_without_updating() {
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

        // A push may already have updated last_head, but NEW isn't the repaired head.
        let moved = decide(
            &Change { last_head: Some(NEW), ..c },
            &Facts { branch: Some(NEW), repair_head: Some(HEAD), ci: Ci { head: NEW, status: Status::Failed }, ..f },
            &Heard { delegate: Status::Passed, ..heard() },
        );
        assert_eq!(moved.decision, Decision::None);
        assert_eq!(moved.change.state, State::Checking { head: NEW });
        assert_eq!(moved.change.repairs, 1);
    }
}

#[test]
fn an_empty_ci_repair_without_a_job_holds_even_behind_its_base() {
    for why in [Repair::Ci, Repair::Semantic] {
        for updates in [0, LIMITS.updates] {
            let c = Change { updates, ready_since: Some(at(0)), ..repairing(why) };
            let f = Facts { ci: Ci { head: HEAD, status: Status::Failed }, contains_base: Status::Failed, ..facts() };
            let next = decide(&c, &f, &Heard { delegate: Status::Passed, ..heard() });
            assert_eq!(next.decision, Decision::Hold(Cause::RedStands));
            assert_eq!(next.change.state, ci_hold(why));
            assert_eq!(next.change.last_head, Some(HEAD));
            assert_eq!((next.change.repairs, next.change.updates), (1, updates));
            let red = Facts { rerun: true, ..f };
            let owed = decide(&released(&next.change, at(1)), &red, &heard());
            assert_eq!(owed.decision, Decision::Effect(Effect::Rerun { head: HEAD }));
            assert_eq!((owed.change.repairs, owed.change.updates), (1, updates));
        }
    }
}

fn ci_hold(why: Repair) -> State {
    State::Held {
        was: Box::new(State::Repairing { requested: false, why }),
        hold: crate::Hold {
            judged: Some(crate::Judged { head: HEAD, spec: 0 }),
            ..crate::Hold::new(Cause::RedStands, at(1))
        },
    }
}

#[test]
fn only_the_taken_ci_measurement_judges_an_empty_repair() {
    use crate::{Fault, Recheck};
    let c = Change { repairs: 1, ..change(State::Checking { head: HEAD }) };
    let red = Facts { ci: Ci { head: HEAD, status: Status::Failed }, recheck: Recheck::Owed, ..facts() };
    let waiting = decide(&c, &red, &heard());
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(waiting.change, c);
    assert_eq!(step(&c, &red, &heard(), at(30), &LIMITS).decision, Decision::Hold(Cause::Stalled));
    let taken = Facts { recheck: Recheck::Taken, ..red };
    let again = decide(&c, &taken, &heard());
    assert_eq!(again.decision, Decision::Hold(Cause::RedStands));
    assert_eq!(again.change.state, ci_hold(Repair::Ci));
    assert_eq!(again.change.repairs, 1);
    let green = decide(&c, &Facts { recheck: Recheck::Taken, ..facts() }, &heard());
    assert_eq!(green.decision, Decision::None);
    assert_eq!(green.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
    assert_eq!(green.change.repairs, 1);
    for (reruns, expected) in
        [(0, Decision::Effect(Effect::Rerun { head: HEAD })), (LIMITS.reruns, Decision::Hold(Cause::Runner))]
    {
        let failed = decide(&c, &Facts { fault: Fault::Runner { reruns }, ..taken.clone() }, &heard());
        assert_eq!(failed.decision, expected);
        assert_eq!(failed.change.repairs, 1);
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

const LANDING_REASONS: [Repair; 2] =
    [Repair::Landing { gate: 9 }, Repair::LandingAt { candidate: NEW, gate: 9, below: None }];

fn landing_endings() -> [Heard; 3] {
    let passed = Heard { delegate: Status::Passed, ..heard() };
    [passed, Heard { flake: true, ..passed }, Heard { no_test: true, ..passed }]
}

#[test]
fn unchanged_landing_repairs_check_again_at_the_same_head() {
    for why in LANDING_REASONS {
        for terminal in landing_endings() {
            let next = decide(&repairing(why), &Facts { regate: true, ..facts() }, &terminal);
            assert_eq!(next.decision, Decision::None);
            assert_eq!(next.change.state, State::Checking { head: HEAD });
            assert_eq!(next.change.last_head, Some(HEAD));
            assert_eq!(next.change.repairs, 1);
        }
    }
}

#[test]
fn a_landing_repair_without_its_receipt_holds_red_stands() {
    for why in LANDING_REASONS {
        for terminal in landing_endings() {
            let held = decide(&repairing(why), &facts(), &terminal);
            assert_eq!(held.decision, Decision::Hold(Cause::RedStands));
            assert_eq!(
                held.change.state,
                State::Held {
                    was: Box::new(State::Repairing { requested: false, why }),
                    hold: crate::Hold {
                        judged: Some(crate::Judged { head: HEAD, spec: 0 }),
                        ..crate::Hold::new(Cause::RedStands, at(1))
                    },
                }
            );
            assert_eq!(held.change.repairs, 1);
            assert!(!held.change.owns_turn);
            assert!(!crate::held_on_ci(&held.change.state));
            assert_eq!(decide(&held.change, &facts(), &terminal).decision, Decision::None);
        }
    }
}

#[test]
fn a_flake_release_checks_then_gates_without_a_repair() {
    let mut c = repairing(Repair::Landing { gate: 9 });
    c.state = State::Held { was: Box::new(c.state), hold: crate::Hold::new(Cause::RedStands, Wall::EPOCH) };
    c.clean = Box::new([BASE]);
    let lifted = released(&c, at(1));
    let checked = decide(&lifted, &Facts { regate: true, ..facts() }, &heard());
    assert_eq!(checked.decision, Decision::None);
    assert_eq!(checked.change.state, State::Checking { head: HEAD });
    assert!(checked.change.clean.is_empty());
    assert_eq!(checked.change.repairs, 1);
    let gated = decide(&checked.change, &facts(), &heard());
    assert_eq!(gated.decision, Decision::None);
    assert_eq!(gated.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
}

#[test]
fn a_failed_landing_repair_release_asks_a_fresh_repair() {
    let why = Repair::Landing { gate: 9 };
    let held = decide(&repairing(why), &facts(), &Heard { delegate: Status::Failed, ..heard() });
    assert_eq!(held.decision, Decision::Hold(Cause::Failed));
    let lifted = decide(&held.change, &facts(), &Heard { released: true, ..heard() });
    assert_eq!(lifted.change.state, State::Repairing { requested: false, why });
    let repair = decide(&lifted.change, &facts(), &heard());
    assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(why)));
    assert_eq!(repair.change.repairs, 2);
}

#[test]
fn failed_repairs_and_non_ci_empty_repairs_still_hold() {
    // Failed and cancelled terminals both arrive as Failed; non-CI and
    // non-landing no_change results keep that coercion at the forge boundary.
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
        assert_eq!(next.decision, Decision::Hold(Cause::Failed));
    }
}

#[test]
fn a_moved_landing_repair_is_a_push_not_a_regate() {
    for why in LANDING_REASONS {
        for terminal in landing_endings() {
            let mut c = repairing(why);
            // A push report can already have updated the policy's last head.
            c.last_head = Some(NEW);
            let f = Facts { branch: Some(NEW), repair_head: Some(HEAD), ..facts() };
            let pushed = decide(&c, &f, &terminal);
            assert_eq!(pushed.decision, Decision::None);
            assert_eq!(pushed.change.state, State::Checking { head: NEW });
            assert_eq!(pushed.change.repairs, 1);
        }
    }
}

fn failed_landing_ask(recheck: crate::Recheck) -> Facts {
    Facts { ask: Some(handed(crate::Ask::Repair { gate: 9, candidate: NEW, below: None })), recheck, ..facts() }
}

#[test]
fn only_the_taken_measurement_holds_at_the_landing_ask() {
    let c = change(State::Ready { head: HEAD, ready_since: at(0) });
    let why = LANDING_REASONS[1];
    for (recheck, expected) in [
        (crate::Recheck::None, Decision::Delegate(Delegate::Repair(why))),
        (crate::Recheck::Owed, Decision::Delegate(Delegate::Repair(why))),
        (crate::Recheck::Taken, Decision::Hold(Cause::RedStands)),
    ] {
        let next = decide(&c, &failed_landing_ask(recheck), &heard());
        assert_eq!(next.decision, expected);
        assert_eq!(next.took, Some(crate::Took::Taken(17)));
    }
}

#[test]
fn an_amended_release_asks_a_fresh_repair_for_the_judged_reason() {
    for why in [LANDING_REASONS[1], Repair::Ci, Repair::Semantic] {
        let was = State::Repairing { requested: false, why };
        let mut held = change(ci_hold(why));
        (held.repairs, held.resolutions, held.updates) = (1, LIMITS.resolutions, LIMITS.updates);
        let released = released(&held, at(1));
        assert_eq!(released.state, held.state, "the judgement waits for the fresh gather");
        assert_eq!((released.repairs, released.resolutions, released.updates), (1, LIMITS.resolutions, LIMITS.updates));
        let red = Facts {
            revision: 1,
            recheck: crate::Recheck::Taken,
            ci: Ci { head: HEAD, status: Status::Failed },
            ..facts()
        };
        let lifted = decide(&released, &red, &heard());
        assert_eq!(lifted.change.state, was);
        let repair = decide(&lifted.change, &red, &heard());
        assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(why)));
        assert_eq!(repair.change.repairs, 2);
    }
}

#[test]
fn a_no_test_release_checks_then_gates_without_a_repair() {
    let mut c = repairing(Repair::Landing { gate: 9 });
    c.state = State::Held { was: Box::new(c.state), hold: crate::Hold::new(Cause::RedStands, Wall::EPOCH) };
    c.clean = Box::new([BASE]);
    let lifted = released(&c, at(1));
    let checked = decide(&lifted, &Facts { regate: true, ..facts() }, &heard());
    assert_eq!(checked.decision, Decision::None);
    assert_eq!(checked.change.state, State::Checking { head: HEAD });
    assert!(checked.change.clean.is_empty());
    assert_eq!(checked.change.repairs, 1);
    let gated = decide(&checked.change, &facts(), &heard());
    assert_eq!(gated.decision, Decision::None);
    assert_eq!(gated.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
}

#[test]
fn a_release_checks_a_new_head_instead_of_measuring_the_held_red() {
    let mut held = change(ci_hold(Repair::Semantic));
    held.repairs = 1;
    let fresh = Facts {
        branch: Some(NEW),
        ci: Ci { head: NEW, status: Status::Failed },
        pull: Pull::Open { head: NEW, base: BASE },
        measure: true,
        rerun: true,
        ..facts()
    };
    let next = decide(&released(&held, at(1)), &fresh, &heard());
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.state, State::Checking { head: NEW });
    assert_eq!(next.change.last_head, Some(NEW));
    assert_eq!(next.change.repairs, 1);
}
