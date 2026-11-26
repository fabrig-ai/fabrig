//! Reviews judge the asked spec (temper/docs/design/domain/forge/change/gates.md, 1, Valid at a spec revision).
use super::*;
use crate::{Ask, Judgement, Way, prefix_verdict};

fn gate(kind: GateKind, freshness: Freshness, blocking: bool) -> Gate {
    Gate { number: 9, kind, freshness, blocking, eager: false }
}

#[test]
fn only_agent_verdicts_expire_at_an_amended_spec() {
    for kind in [GateKind::Agent, GateKind::Person, GateKind::Check] {
        for freshness in [Freshness::Exact, Freshness::Clean] {
            let gate = gate(kind, freshness, true);
            let mut c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
            c.gates = Box::new([gate]);
            c.clean = Box::new([BASE]);
            let report_head = if freshness == Freshness::Clean { BASE } else { HEAD };
            for status in [Status::Passed, Status::Failed] {
                let mut f = facts();
                f.gates = Box::new([GateReport { number: 9, head: report_head, status, revision: 0 }]);
                for revision in [0, 1] {
                    f.revision = revision;
                    let stale = kind == GateKind::Agent && revision == 1;
                    let next = decide(&c, &f, &heard());
                    if stale {
                        assert_eq!(next.decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }));
                        assert_eq!(next.change.state, c.state, "re-asking does not duplicate the asked number");
                        assert_eq!(next.change.repairs, 0);
                        assert_eq!(next.change.gate_failures, c.gate_failures);
                    } else if status == Status::Passed {
                        assert_eq!(next.decision, Decision::Ready);
                    } else if kind == GateKind::Person {
                        assert_eq!(next.decision, Decision::Hold(Hold::Rejected));
                    } else {
                        assert_eq!(next.decision, Decision::Delegate(Delegate::Repair(Repair::Gate(9))));
                    }
                    // Exact judgements are at their report head; clean agent/person
                    // verdicts also carry from the member head to the prefix.
                    let prefix =
                        if freshness == Freshness::Clean && kind != GateKind::Check { NEW } else { report_head };
                    assert_eq!(
                        prefix_verdict(&c, gate, &f.gates, HEAD, prefix, revision),
                        if stale { Status::Unknown } else { status }
                    );
                }
            }
        }
    }
}

#[test]
fn stale_reviews_use_the_existing_gate_paths_before_merge() {
    let target = Judgement {
        landing: 50,
        round: 1,
        pull: 2,
        branch: b"candidate".as_slice().into(),
        base: b"main".as_slice().into(),
        prefix: HEAD,
    };
    for blocking in [false, true] {
        for status in [Status::Passed, Status::Failed] {
            for state in [
                State::Ready { head: HEAD, ready_since: at(0) },
                State::Queued { head: HEAD, ready_since: at(0) },
                State::First { head: HEAD, ready_since: at(0) },
                State::Judging { head: HEAD, ready_since: at(0), target: target.clone(), asked: Box::new([9]) },
            ] {
                let mut c = change(state.clone());
                c.gates = Box::new([gate(GateKind::Agent, Freshness::Exact, blocking)]);
                c.gate_failures = Box::new([GateFailure { number: 9, count: 1 }]);
                let mut f = facts();
                f.revision = 1;
                f.gates = Box::new([GateReport { number: 9, head: HEAD, status, revision: 0 }]);
                if matches!(state, State::Ready { .. } | State::Judging { .. }) {
                    c.way = Way::Turn;
                    f.landing = Some(50);
                }
                if matches!(state, State::Judging { .. }) {
                    f.ask = Some(handed(Ask::Judge(target.clone())));
                }
                let mut next = decide(&c, &f, &heard());
                if next.decision == Decision::None {
                    next = decide(&next.change, &f, &heard());
                }
                assert_eq!(next.decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }), "{state:?}");
                assert_eq!(next.change.repairs, 0);
                assert_eq!(next.change.gate_failures, c.gate_failures);
                f.live_gate = Some(HEAD);
                let live = decide(&next.change, &f, &heard());
                if !blocking && matches!(next.change.state, State::Gating { .. }) {
                    assert_eq!(live.decision, Decision::Ready, "a running advisory review does not block");
                } else {
                    assert!(
                        matches!(live.decision, Decision::Wait { .. }),
                        "the old review runs on until its terminal"
                    );
                }
            }
        }
    }
}
