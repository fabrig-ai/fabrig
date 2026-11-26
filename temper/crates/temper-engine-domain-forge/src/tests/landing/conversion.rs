//! Link restores a predecessor's separate change parts and its per-member landing slots.
use super::*;

const LANDING: u64 = 61;

#[test]
fn conversion_does_not_repair_a_head_that_moved() {
    let mut d = domain();
    let mut row = fixtures::opening_row();
    row.repository = REPO;
    row.change.last_head = Some(fixtures::HEAD);
    let (cold, hot) = row.split();
    let mut landing = merge_landing_row(false);
    landing.landing.task = LANDING;
    landing.landing.phase = policy::Phase::Proving;
    landing.landing.members[0].candidate = Some(fixtures::BASE);
    landing.landing.members[0].gates = Box::new([policy::GateProof {
        proof: policy::Proof::Failed { key: [8; 32] },
        posted: policy::Posted::Verdict,
        faults: 0,
    }]);
    landing.previous_asks = Box::new([None]);
    for record in [
        Stored::Repository(adopted()),
        Stored::ChangeCold(Box::new(cold)),
        Stored::ChangeHot(Box::new(hot)),
        Stored::Landing(landing),
    ] {
        outputs(&mut d, Event::Restore { record });
    }
    let mut out = Queue::uncapped();
    let tasks = jig_tasks_kit::domain(&jig_tasks_kit::LIMITS, 71);
    assert_eq!(d.start_links(&tasks, &env(), &mut out), Ok(()));
    let row = d.change(fixtures::CHANGE).expect("member kept by the unmade pass");
    assert!(row.asks.is_empty());
}
