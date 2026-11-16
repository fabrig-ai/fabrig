//! One workflow/commit window across distinct gate inputs (landing-search-by-test.md, 5).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, CHANGE, LANDING, PROJECT},
};

fn start() -> World {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.latency_min = Duration::from_secs(90);
    workflow.latency_max = workflow.latency_min;
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow)));
    let gates = [b"first".as_slice(), b"second".as_slice()].map(|name| landing::Gate {
        name: name.into(),
        workflow: b"gate.yml".as_slice().into(),
        ignored: Box::new([]),
        blocking: true,
        base: false,
        report: None,
    });
    assert!(world.project_landing(
        PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates { repository: REPO, gates: Box::new(gates), tiers: Box::new([]) }]),
    ));
    world.produce(b"temper/51");
    world.push_file(b"main", b"base", b"main moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let mut row = landings::landing_row(world.change(CHANGE));
    row.landing.members[0].gates =
        vec![landing::GateProof { proof: landing::Proof::Unkeyed, posted: landing::Posted::Nothing, faults: 0 }; 2]
            .into_boxed_slice();
    landings::register(&mut world, row);
    world
}
fn step(world: &mut World, entry: u64) -> landing::Decision {
    let decision = landings::step_landing(world, LANDING, entry);
    changes::change_step_task(world, CHANGE, entry + 10000, change::Status::Passed, Box::new([]), false, false);
    decision
}
fn runs(world: &World) -> Box<[raw::Run]> {
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(b"gate.yml".as_slice().into()),
        commit: None,
        number: None,
        page: 1,
        limit: 16,
    }) else {
        panic!("workflow runs")
    };
    runs
}
fn until_dispatch(world: &mut World, first: u64) {
    for entry in first..first + 60 {
        if matches!(step(world, entry), landing::Decision::Dispatch { .. } | landing::Decision::Retry { .. }) {
            return;
        }
    }
    panic!("no dispatch");
}
fn still_waiting(world: &mut World, first: u64) {
    for entry in first..first + 5 {
        assert!(!matches!(step(world, entry), landing::Decision::Dispatch { gate: 1, .. }), "second gate must wait");
        assert!(matches!(
            world.landing(LANDING).landing.members[0].gates[1].proof,
            landing::Proof::Unkeyed | landing::Proof::Keyed { .. }
        ));
    }
}
fn own_receipts(world: &World) {
    let runs = runs(world);
    let receipts: Vec<_> = world
        .stored()
        .iter()
        .filter_map(|(_, row)| match row {
            top::Stored::Receipt { key, receipt } => Some((key, receipt)),
            _ => None,
        })
        .collect();
    assert_eq!(receipts.len(), 2);
    assert_ne!(receipts[0].1.run.id, receipts[1].1.run.id, "each key has its own run");
    for (key, receipt) in receipts {
        let run = runs.iter().find(|run| run.id == receipt.run.id).expect("receipt's dispatched run");
        let input = run.inputs.iter().find(|(name, _)| name.as_ref() == b"temper_key").expect("key input");
        let hex: Vec<_> = key.key.iter().flat_map(|byte| top::hex(*byte)).collect();
        assert_eq!(input.1.as_ref(), hex);
    }
}
#[test]
fn two_gates_wait_across_restart_then_keep_their_own_runs() {
    let mut world = start();
    until_dispatch(&mut world, 100);
    let first = runs(&world)[0].clone();
    assert_eq!(first.state, raw::RunState::Running);
    still_waiting(&mut world, 200);
    assert_eq!(runs(&world).len(), 1);
    world.restart();
    assert_eq!(runs(&world)[0].state, raw::RunState::Running, "restart with the first run in flight");
    still_waiting(&mut world, 300);
    assert_eq!(runs(&world).len(), 1, "restoring a waiter dispatches nothing twice");
    world.take_calls();
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    assert_eq!(runs(&world).len(), 2);
    let calls = world.take_calls();
    let second_at =
        calls.iter().position(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).expect("second dispatch");
    assert!(
        calls[..second_at].iter().any(|op| matches!(op, api::Op::Read(api::Read::Run { run }) if *run == first.id)),
        "the first run is read ended before the next dispatch"
    );
    own_receipts(&world);
}
#[test]
fn retries_keep_the_workflow_reserved_and_the_original_inputs() {
    for cancel in [false, true] {
        let mut world = start();
        if !cancel {
            world.runner_setup_faults(1000);
        }
        until_dispatch(&mut world, 100);
        let first = runs(&world)[0].clone();
        if cancel {
            assert_eq!(world.external(raw::Op::Write(raw::Write::Cancel { run: first.id }), 2), raw::Answer::Done);
        } else {
            world.run_for(100);
        }
        // A fault is read, retained, then retried before the second key can dispatch.
        until_dispatch(&mut world, 200);
        if !cancel {
            world.runner_setup_faults(0);
        }
        let retry = runs(&world)[0].clone();
        assert_eq!(retry.inputs, first.inputs);
        assert_eq!(retry.id == first.id, !cancel, "job faults rerun; cancellation dispatches afresh");
        still_waiting(&mut world, 300);
        assert_eq!(runs(&world).len(), if cancel { 2 } else { 1 });
        landings::drive_landing(&mut world, LANDING);
        assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
        own_receipts(&world);
    }
}
