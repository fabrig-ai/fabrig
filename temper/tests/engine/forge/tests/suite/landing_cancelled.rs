//! A person's cancellation redispatches proof without repairing the member (domain/landing.md, 7.4, 15–16).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, CHANGE, LANDING, Trains},
    translate,
};

const RETRIES: u32 = 1;

fn start() -> World {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.latency_min = Duration::from_secs(300);
    workflow.latency_max = workflow.latency_min;
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow)));
    world.gate_policy(landing::Project { gate_retries: RETRIES, ..landing::Project::DEFAULT });
    world.produce(b"temper/51");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    world.push_as(b"main", b"base", b"another hand moved main", 2);
    Trains::default().judge(&mut world);
    world
}
fn step(world: &mut World, entry: u64) -> landing::Decision {
    let decision = landings::step_landing(world, LANDING, entry);
    changes::change_step_task(world, CHANGE, entry + 10000, change::Status::Passed, Box::new([]), false, false);
    decision
}
fn until_dispatch(world: &mut World, first: u64) {
    for entry in first..first + 60 {
        let decision = step(world, entry);
        assert!(!matches!(decision, landing::Decision::Hold(_)), "held before retry bound: {decision:?}");
        if matches!(decision, landing::Decision::Dispatch { .. } | landing::Decision::Retry { .. }) {
            return;
        }
    }
    panic!("no dispatch: {:?}", world.landing(LANDING));
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
fn cancel(world: &mut World) -> raw::Run {
    let run = runs(world)[0].clone();
    assert_eq!(run.state, raw::RunState::Running, "cancel while in flight");
    assert_eq!(world.external(raw::Op::Write(raw::Write::Cancel { run: run.id }), 2), raw::Answer::Done);
    let raw::Answer::Run(cancelled) = world.inspect(&raw::Read::Run { run: run.id }) else { panic!("run") };
    assert_eq!(cancelled.conclusion, Some(raw::Conclusion::Cancelled));
    assert_eq!(cancelled.cancelled_by, Some(2));
    run
}
fn no_receipt_or_repair(world: &World) {
    assert!(world.stored().iter().all(|(_, row)| !matches!(row, top::Stored::Receipt { .. })));
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert!(!matches!(world.change(CHANGE).change.state, change::State::Repairing { .. }));
}
fn dispatches_without_reruns(calls: &[api::Op]) -> Vec<&api::Write> {
    assert!(!calls.iter().any(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))));
    calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(write @ api::Write::Dispatch { .. }) => Some(write),
            _ => None,
        })
        .collect()
}

#[test]
fn a_persons_cancellation_dispatches_afresh_and_restart_finds_that_run() {
    let mut world = start();
    let moved_main = world.branch(b"main");
    let member_head = world.branch(b"temper/51");
    until_dispatch(&mut world, 100);
    let first = cancel(&mut world);
    until_dispatch(&mut world, 200);
    let second = runs(&world)[0].clone();
    assert_ne!(second.id, first.id);
    assert_eq!(second.state, raw::RunState::Running);
    assert_eq!(second.commit, first.commit);
    assert_eq!(second.inputs, first.inputs, "temper_commit and temper_key stay fixed");
    no_receipt_or_repair(&world);
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("candidate");
    assert_eq!(translate::commit(second.commit), candidate);
    let before_restart = world.take_calls();
    let dispatches = dispatches_without_reruns(&before_restart);
    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0], dispatches[1]);

    world.restart();
    world.run_for(1);
    for entry in 300..305 {
        step(&mut world, entry);
    }
    assert_eq!(runs(&world)[0].state, raw::RunState::Running);
    let relearned = world.take_calls();
    assert!(relearned.iter().any(|op| matches!(op, api::Op::Read(api::Read::Run { run }) if *run == second.id)));
    assert_eq!(dispatches_without_reruns(&relearned), Vec::<&api::Write>::new());
    no_receipt_or_repair(&world);
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(dispatches_without_reruns(&world.take_calls()), Vec::<&api::Write>::new());
    assert_eq!(runs(&world).len(), 2);
    assert_eq!(world.branch(b"main"), second.commit);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    let row = world.change(CHANGE);
    assert!(matches!(row.change.state, change::State::Landed { merge, .. } if merge == candidate));
    assert_eq!(row.change.repairs, 0);
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: row.pull.expect("member pull") }) else {
        panic!("member pull")
    };
    assert_eq!(pull.merged, Some(second.commit));
    assert_eq!(pull.commit, member_head);
    assert_eq!(world.branch(b"temper/51"), member_head);
    let raw::Answer::Commits { commits, .. } =
        world.inspect(&raw::Read::Commits { before: moved_main, after: second.commit })
    else {
        panic!("candidate ancestry")
    };
    let merge = commits.iter().find(|commit| commit.id == second.commit).expect("candidate merge");
    assert_eq!(merge.parents.as_ref(), &[moved_main, member_head]);
    let raw::Answer::Statuses(statuses) = world.inspect(&raw::Read::Statuses { commit: second.commit }) else {
        panic!("gate status")
    };
    let gate = statuses.iter().find(|status| status.context.as_ref() == b"temper/gate/gate").expect("gate status");
    assert_eq!(gate.state, raw::Check::Passed);
    let receipts: Vec<_> = world
        .stored()
        .iter()
        .filter_map(|(_, row)| match row {
            top::Stored::Receipt { receipt, .. } => Some(receipt),
            _ => None,
        })
        .collect();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].passed);
    assert_eq!(receipts[0].candidate, candidate);
    assert_eq!(receipts[0].run.id, second.id, "only the replacement run proves the key");
}

#[test]
fn cancellations_past_the_gate_retry_bound_hold_without_repair() {
    let mut world = start();
    until_dispatch(&mut world, 100);
    for retry in 0..=RETRIES {
        cancel(&mut world);
        if retry < RETRIES {
            until_dispatch(&mut world, 200 + u64::from(retry) * 100);
        }
    }
    let mut held = false;
    for entry in 500..560 {
        if matches!(step(&mut world, entry), landing::Decision::Hold(landing::Hold::Faults { member: 0, gate: 0 })) {
            held = true;
            break;
        }
    }
    assert!(held, "cancellations past gate_retries hold the landing");
    assert!(matches!(
        world.landing(LANDING).landing.phase,
        landing::Phase::Held { why: landing::Hold::Faults { member: 0, gate: 0 }, .. }
    ));
    let words = world.landing_hold_words(LANDING).expect("hold words");
    let words = String::from_utf8_lossy(&words);
    assert!(words.contains("gate gate") && words.contains("runner") && words.contains("retries"), "{words}");
    no_receipt_or_repair(&world);
    assert_eq!(runs(&world).len(), usize::try_from(RETRIES + 1).expect("tiny retry bound"));
    assert_eq!(
        dispatches_without_reruns(&world.take_calls()).len(),
        usize::try_from(RETRIES + 1).expect("tiny retry bound")
    );
}
