//! A pending decider job precedes the core's link checks (protocol/store/jobs.md, 2).
use jig_core_world::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_protocol_store::{OwnerSet, OwnerSets};
use jig_test_domain as root;

const BEFORE: OwnerSets = OwnerSets {
    sets: &[
        jig_protocol_store::SKEIN_SET,
        jig_protocol_store::SMITH_SET,
        OwnerSet { jobs: &[], ..jig_protocol_store::JIG_STORE_SET },
    ],
};
const NOW: OwnerSets = OwnerSets {
    sets: &[jig_protocol_store::SKEIN_SET, jig_protocol_store::SMITH_SET, jig_protocol_store::JIG_STORE_SET],
};

#[test]
fn a_stored_ancestor_rejection_is_repaired_before_the_core_starts() {
    const SEED: u64 = 907;
    let (configuration, limits) = jig_core_world::faults::plan_fixture(SEED);
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(configuration),
        limits,
        owner_sets: BEFORE,
        ..Setup::default()
    });
    let parent = world.assignments[0];
    let member = fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"held delegate");
    let child = fixtures::delegate(&mut world, parent, 1, member);
    let mut held = fixtures::live_row(&world, child).clone();
    let tasks::Phase::Active(active) = held.phase.clone() else { panic!("active delegate") };
    held.phase = tasks::Phase::Held { was: tasks::Was::Active(active), why: tasks::Hold::Budget, source: None };
    held.escalation = tasks::Escalation::Rejected {
        revision: held.revisions.hold,
        by: tasks::EscalationDecider::Person(parent.0),
        reason: b"keep held".as_slice().into(),
    };
    // The fixture stands for a store written with the untagged decider.
    world.observer.record();
    world.store.seed(Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(held))))]));
    world.broken();
    world.restart_with_owner_sets(SEED, false, NOW);
    assert!(world.domain.ready(), "{:?}", world.domain.start_failure());
    assert_eq!(world.domain.start_failure(), None);
    let tasks::Escalation::Rejected { by, .. } =
        &world.domain.core().tasks.task(child).expect("restored held task").escalation
    else {
        panic!("rejection retained")
    };
    assert_eq!(*by, tasks::EscalationDecider::Task(parent.0));
    world.restart_with(jig_core_world::faults::plan_fixture(SEED).0);
    assert!(world.domain.ready(), "{:?}", world.domain.start_failure());
}
