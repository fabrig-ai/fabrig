//! Controls for the cause, not merely the colour, of CI (testing/blame.md, 5).
use skein_fake_forge_domain::api as raw;
use skein_world::{
    faults::{Counter, FaultLedger},
    streams::Streams,
};
use temper_blame::{self as blame, Names, Reaction};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes, fake_config,
    landings::{self, CHANGE, LANDING, Trains},
    translate,
};

pub(crate) fn judged(world: &mut World, seed: u64) -> FaultLedger {
    let observed = world.blame().to_vec();
    let mut ledger = world.take_ledger();
    let findings = blame::judge(&mut ledger, &observed, seed);
    assert!(findings.is_empty(), "{}", findings.join("\n"));
    ledger
}

pub(crate) fn count(ledger: &FaultLedger, kind: skein_world::faults::Kind, counter: Counter) -> u64 {
    ledger.entries().iter().filter(|entry| entry.kind == kind).map(|entry| entry.count(counter)).sum()
}

pub(crate) fn one_change(mut world: World, wrong: bool) -> World {
    world.watch_blame();
    world = landings::turn_in(world);
    world.push_file(b"main", b"file", b"fixed");
    world.external(
        raw::Op::Git(raw::Git::Create { branch: b"temper/51".as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_file(b"temper/51", b"member", b"change");
    if wrong {
        world.push_wrong_tree(b"temper/51", b"file", b"broken");
    }
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    Trains::default().judge(&mut world);
    world
}

pub(crate) fn finish_landing(world: &mut World, clear_lost: bool) {
    for entry in 100..280 {
        let decision = landings::step_landing(world, LANDING, entry);
        if clear_lost && world.ledger().entries().iter().any(|entry| entry.kind == blame::LOST) {
            world.runner_lost(0);
        }
        if decision == landing::Decision::Finish {
            return;
        }
    }
    panic!("landing did not finish: {:?}", world.landing(LANDING));
}

pub(crate) fn ask_repair(world: &mut World) -> change::Decision {
    changes::change_step_task(world, CHANGE, 900, change::Status::Passed, Box::new([]), false, false)
}

pub(crate) fn red_base(world: &mut World) {
    world.adopt();
    world.produce_file(b"temper/51", b"member", b"change");
    world.status(b"temper/51", raw::Check::Passed);
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = changes::change_row();
    row.pull = Some(world.open_pull(b"temper/51", b"main"));
    row.change.last_head = Some(head);
    row.change.state = change::State::Gating { head, asked: Box::new([]) };
    world.event(top::Event::Change { row });
    assert_eq!(ask_repair(world), change::Decision::Ready);
    world.landing_ci(b"main", true);
    world.red_base(b"main");
    assert_eq!(ask_repair(world), change::Decision::QueueRepair);
}

#[test]
fn two_faults_on_one_change_are_each_blamed_on_their_cause() {
    let mut world = one_change(World::batch_turn(1, Some(super::landing_batches::workflow(true))), true);
    world.gate_policy(landing::Project { gate_retries: 1, ..landing::Project::DEFAULT });
    world.runner_lost(1000);
    finish_landing(&mut world, true);
    assert!(matches!(ask_repair(&mut world), change::Decision::Delegate(change::Delegate::Repair(_))));
    let ledger = judged(&mut world, 1);
    assert_eq!(count(&ledger, blame::LOST, Counter::GateRetries), 1);
    assert_eq!(count(&ledger, blame::LOST, Counter::Repairs), 0);
    assert_eq!(count(&ledger, blame::WRONG_TREE, Counter::GateRetries), 0);
    assert_eq!(count(&ledger, blame::WRONG_TREE, Counter::Repairs), 1);
}

#[test]
fn a_red_base_under_a_green_candidate_is_the_bases() {
    let mut world = World::new_in(&Streams::new(1), fake_config());
    world.watch_blame();
    red_base(&mut world);
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert!(world.blame().iter().any(|observed| matches!(observed.reaction,
        Reaction::Hold { names: Names::Base, ref base, .. } if base.is_some())));
    let ledger = judged(&mut world, 1);
    assert_eq!(count(&ledger, blame::BASE_RED, Counter::Repairs), 1);
    assert_eq!(count(&ledger, blame::BASE_RED, Counter::GateRetries), 0);
}
