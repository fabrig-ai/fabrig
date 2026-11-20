//! Both selections at the end of fresh reads retain their named gate limit.
use super::*;
use crate::tests::{env, fixtures};

fn finishing(follow: bool) -> Queue<Request> {
    let (mut d, mut row) = crate::tests::follow_check::configured();
    fixtures::connect(&mut d);
    crate::tests::outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    crate::tests::outputs(&mut d, Event::ReadAfresh);
    for _ in 0..env().limits.client.pending {
        if fresh_reads_in(&d) {
            break;
        }
        fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(fixtures::BASE)));
    }
    assert!(fresh_reads_in(&d));
    row.pending = None;
    let own = change::Gate {
        number: 1,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    };
    row.change.gates = Box::new([own]);
    if !follow {
        row.follows = None;
        row.change.gates = Box::new([own, change::Gate { number: 2, ..own }]);
    }
    d.changes.insert(row.task, row).unwrap();
    let mut out = Queue::uncapped();
    step(&mut d, &env(), fixtures::change_step(), &mut out);
    let Some((owner, _)) = d.steps.first() else { panic!("gathering step: {out:?}") };
    let owner = *owner;
    let mut pending = d.steps.remove(&owner).unwrap();
    pending.branch = Some(fixtures::HEAD);
    pending.base_tip = Some(fixtures::BASE);
    pending.woken = false;
    let mut env = env();
    env.limits.change_policy.gates = 1;
    let mut out = Queue::uncapped();
    finish_step(&mut d, &env, pending, &mut out);
    out
}
fn named(out: &Queue<Request>) {
    let mut named = false;
    for request in out {
        if let Request::ResourceRefused {
            operation: b"step change",
            cause: crate::RefusalCause::Limit { member: "forge.change_policy.gates", configured: 1, needed: 2 },
            ..
        } = request
        {
            named = true;
        }
        assert!(!matches!(request, Request::Refused { .. }), "{out:?}");
    }
    assert!(named, "{out:?}");
}
#[test]
fn finishing_a_follow_selection_names_the_gate_limit() {
    named(&finishing(true));
}
#[test]
fn finishing_a_standing_gate_selection_names_the_gate_limit() {
    named(&finishing(false));
}
