//! The follow check is an exact gate of the change, before either landing way.
use super::*;
use crate::{FOLLOW_CHECK_GATE, Way};
fn follow(state: State, way: Way) -> Change {
    let mut c = change(state);
    c.way = way;
    c.gates = Box::new([Gate {
        number: FOLLOW_CHECK_GATE,
        kind: GateKind::Follow,
        blocking: true,
        freshness: Freshness::Exact,
        eager: false,
    }]);
    c
}
#[test]
fn a_follow_check_waits_for_ci_then_for_its_exact_verdict() {
    let mut f = facts();
    f.ci.status = Status::Pending;
    let checking = follow(State::Checking { head: HEAD }, Way::Queue);
    assert!(matches!(decide(&checking, &f, &heard()).decision, Decision::Wait { .. }));
    f.ci.status = Status::Passed;
    let gating = decide(&checking, &f, &heard());
    let no_agent_room = Heard {
        lacking: Some(crate::AuthorityHold {
            recovery: crate::AuthorityRecovery { task: checking.task, spec: 0, hold: 1, tasks: 1 },
            lack: crate::AuthorityLack::Tasks,
            had: 0,
            released: false,
        }),
        ..heard()
    };
    let asked = decide(&gating.change, &f, &no_agent_room);
    assert_eq!(asked.decision, Decision::Delegate(Delegate::Gate { number: FOLLOW_CHECK_GATE, head: HEAD }));
    assert!(matches!(decide(&asked.change, &f, &heard()).decision, Decision::Wait { .. }));
    let mut c = asked.change;
    c.clean = Box::new([BASE]);
    f.gates = Box::new([GateReport { number: FOLLOW_CHECK_GATE, head: BASE, status: Status::Passed }]);
    assert!(matches!(decide(&c, &f, &heard()).decision, Decision::Wait { .. }), "clean predecessors cannot answer it");
    f.gates = Box::new([GateReport { number: FOLLOW_CHECK_GATE, head: HEAD, status: Status::Passed }]);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Ready);
}
#[test]
fn a_failed_follow_check_repairs_before_joining_a_landing_and_a_new_head_checks_again() {
    for way in [Way::Queue, Way::Turn] {
        let c = follow(State::Gating { head: HEAD, asked: Box::new([FOLLOW_CHECK_GATE]) }, way);
        let mut f = facts();
        f.gates = Box::new([GateReport { number: FOLLOW_CHECK_GATE, head: HEAD, status: Status::Failed }]);
        assert_eq!(
            decide(&c, &f, &heard()).decision,
            Decision::Delegate(Delegate::Repair(Repair::Gate(FOLLOW_CHECK_GATE)))
        );
        let mut repaired = follow(State::Gating { head: NEW, asked: Box::new([]) }, way);
        repaired.last_head = Some(NEW);
        f.branch = Some(NEW);
        f.ci = Ci { head: NEW, status: Status::Passed };
        assert_eq!(
            decide(&repaired, &f, &heard()).decision,
            Decision::Delegate(Delegate::Gate { number: FOLLOW_CHECK_GATE, head: NEW })
        );
        f.gates = Box::new([GateReport { number: FOLLOW_CHECK_GATE, head: NEW, status: Status::Passed }]);
        assert_eq!(decide(&repaired, &f, &heard()).decision, Decision::Ready);
    }
}
