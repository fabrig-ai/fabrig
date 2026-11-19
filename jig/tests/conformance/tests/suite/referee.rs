use jig_conformance::{Harness, referee::Promise};
use jig_conformance_world::{
    Config, Testing, actions,
    negative::{Fault, world},
};

#[test]
fn an_observer_finding_reaches_the_harness_with_seed_and_trace() {
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;
    use jig_test_domain as root;

    let mut world = Harness::<Testing>::with_trace(Config::new(31, false, 2, None), 31, true);
    world.drain().expect("faithful startup");
    let line = world
        .store
        .rows
        .values_mut()
        .find_map(|record| match record {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) => Some(line),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable activity");
    line.at = skein_lib::Wall::from_nanos(line.at.as_nanos() + 1);
    world.store.applied += 1;
    let failure = world.drain().expect_err("observer finding must leave the adapter");
    assert_eq!(failure.promise, Promise::Commit);
    assert!(failure.why.contains("seed 31: immutable activity row changed"), "{failure:?}");
    assert!(failure.why.contains("output "), "trace accompanies the finding: {failure:?}");
}

#[test]
fn every_broken_root_is_caught_from_durable_rows_and_outside_deliveries() {
    for (fault, promise) in [
        (Fault::EarlyOutput, Promise::Commit),
        (Fault::SplitDecision, Promise::Spend),
        (Fault::AdmissionAfterMutation, Promise::Spend),
        (Fault::WrongRestart, Promise::Order),
        (Fault::UncheckedEffect, Promise::Spend),
        (Fault::WrongKind, Promise::Authority),
    ] {
        let mut world = world(fault);
        let failure = if matches!(fault, Fault::EarlyOutput | Fault::WrongRestart) {
            world.drain().expect_err("the root corruption must be visible")
        } else {
            world.drain().expect("fault waits for an effect");
            actions::effect(&mut world, 1);
            world.drain().expect_err("the root corruption must be visible")
        };
        assert_eq!(failure.promise, promise, "{fault:?}: {failure:?}");
    }
}

#[test]
fn every_broken_connector_is_caught_from_fake_systems_and_durable_attempts() {
    for fault in [Fault::RepeatedEffect, Fault::StaleVerdict, Fault::ForgottenDeadline, Fault::DifferentDecision] {
        let mut world = world(fault);
        world.drain().expect("faithful startup");
        if fault == Fault::StaleVerdict {
            world.advance(61_000_000_000);
            world.drain().expect("the recorded judge fact ages past its freshness");
        }
        if matches!(fault, Fault::ForgottenDeadline | Fault::DifferentDecision) {
            world.peers.hold_writes = true;
        }
        actions::effect(&mut world, 1);
        let failure = if matches!(fault, Fault::ForgottenDeadline | Fault::DifferentDecision) {
            world.drain().expect("initial decision is faithful and uncertain");
            let (number, pending) = world.peers.pending_writes[0].clone();
            world.crash().expect("cold store retains the initial decision");
            world.drain().expect("reconstructed connector awaits the retained system response");
            let event = world.systems[usize::from(number - 1)].answer(pending, jig_test_system::Fault::Late);
            world.send(jig_test_domain::Event::Connector { number, event: jig_test_connector::Event::System(event) });
            world.drain().expect_err("a second step must preserve decision and deadline")
        } else {
            world.drain().expect_err("the connector corruption must be visible")
        };
        let promise = if fault == Fault::StaleVerdict { Promise::Authority } else { Promise::Once };
        assert_eq!(failure.promise, promise, "{fault:?}: {failure:?}");
    }
}

#[test]
fn the_same_boundaries_accept_a_faithful_root_and_connector_across_restart() {
    let mut world = world(Fault::None);
    world.drain().expect("faithful startup");
    world.peers.hold_writes = true;
    actions::effect(&mut world, 1);
    world.drain().expect("faithful initial decision");
    world.crash().expect("faithful cold store");
    world.drain().expect("deadline and decision survive");
    world.peers.hold_writes = false;
    world.advance(2_000_000_000);
    world.drain().expect("faithful retry");
    assert_eq!(world.systems[0].observed().iter().filter(|effect| effect.applied).count(), 1);
}
