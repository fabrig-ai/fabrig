//! Every durable cut of the project deletion story resumes to its checked ending.
use jig_conformance::scenarios::Scenario;
use jig_conformance_world::replay;
use std::{cell::RefCell, rc::Rc, time::Instant};

#[test]
fn project_deleted_survives_every_cut() {
    const SEED: u64 = 31;
    let started = Instant::now();
    let cuts = Rc::new(RefCell::new(Vec::new()));
    let reached = Rc::clone(&cuts);
    let baseline = replay::every_cut(Scenario::ProjectDeleted, SEED, move |cut, world| {
        assert!(world.outcome().crashes.contains(&cut));
        reached.borrow_mut().push(cut);
    })
    .expect("every project deletion cut reaches the checked ending");
    assert_eq!(*cuts.borrow(), (1..=baseline.commits).collect::<Vec<_>>());
    eprintln!("project deletion: {} cuts in {:?}", baseline.commits, started.elapsed());
}
