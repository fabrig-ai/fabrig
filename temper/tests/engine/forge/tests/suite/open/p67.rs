//! A no-change landing repair regates rather than repairing again (engine-checks.md, 5.1;
//! landing-regate.md, 3).
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    changes,
    landings::{self, CHANGE, LANDING},
};

#[test]
fn a_no_change_landing_repair_regates_once() {
    let mut world = super::reported_gate(31, &[b"suite.failed"], false);
    for entry in 100..180 {
        if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Leave { .. }) {
            break;
        }
    }
    let (key, failed) = world
        .stored()
        .iter()
        .find_map(|(_, row)| match row {
            top::Stored::Receipt { key, receipt } if !receipt.passed => Some((key.clone(), receipt.clone())),
            _ => None,
        })
        .expect("the gate kept its failed receipt");
    let head = world.change(CHANGE).change.last_head.expect("change head");
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("failed candidate");
    let kind = change::Delegate::Repair(change::Repair::LandingAt { gate: 0, candidate, below: None });
    assert_eq!(
        changes::change_step_task(&mut world, CHANGE, 300, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Delegate(kind)
    );
    world.event(top::Event::Delegated { task: CHANGE, child: 91, kind });
    world.event(top::Event::DelegateResult {
        ended_at: None,
        task: CHANGE,
        child: 91,
        status: change::Status::Passed,
        no_change: true,
        flake: false,
        no_test: false,
        failed: false,
        lost: false,
        cancelled: false,
        words: b"Nothing to change".as_slice().into(),
    });
    world.take_calls();
    // Let the old round finish cleanup. The change must proceed without a release.
    for entry in 400..440 {
        if landings::step_landing(&mut world, LANDING, entry) == landing::Decision::Finish {
            break;
        }
    }
    changes::change_step_task(&mut world, CHANGE, 500, change::Status::Passed, Box::new([]), false, false);
    assert!(!matches!(world.change(CHANGE).change.state, change::State::Held { .. }), "no release needed");
    let mut fresh_repair = None;
    for entry in 501..521 {
        let decision =
            changes::change_step_task(&mut world, CHANGE, entry, change::Status::Passed, Box::new([]), false, false);
        if matches!(decision, change::Decision::Delegate(change::Delegate::Repair(_))) {
            fresh_repair = Some(decision);
            break;
        }
        if matches!(world.change(CHANGE).change.state, change::State::Ready { .. }) {
            let mut row = landings::landing_row(world.change(CHANGE));
            row.landing.task = 62;
            row.landing.round = landing::Round::new(b"temper/1/l62".as_slice().into());
            row.landing.members[0].branch.clone_from(&row.landing.round.branch);
            landings::register(&mut world, row);
            for entry in 600..680 {
                if matches!(
                    landings::step_landing(&mut world, 62, entry),
                    landing::Decision::Leave { .. } | landing::Decision::Hold(_) | landing::Decision::Finish
                ) {
                    break;
                }
            }
            break;
        }
    }
    assert_eq!(world.change(CHANGE).change.last_head, Some(head), "no-change keeps the member head");
    assert_eq!(fresh_repair, None, "a no-change landing repair must regate, not ask another repair");
    let dispatches: Vec<_> = world
        .take_calls()
        .into_iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Dispatch { inputs, .. }) => Some(inputs),
            _ => None,
        })
        .collect();
    assert_eq!(dispatches.len(), 1, "one regate after the no-change result");
    // A new prepare can make another candidate commit; the tree's gate key is the identity.
    let inputs = dispatches.first().expect("one dispatch");
    assert!(
        inputs.iter().any(|input| input.name.as_ref() == b"temper_key" && input.value == top::commit_text(&key.key)),
        "regate the failed key: {inputs:?}"
    );
    let replacement = world
        .stored()
        .iter()
        .find_map(|(_, row)| match row {
            top::Stored::Receipt { key: stored_key, receipt } if stored_key == &key => Some(receipt),
            _ => None,
        })
        .expect("receipt at the regated key");
    assert_ne!(replacement.run.id, failed.run.id, "the fresh run replaces the failed receipt");
    let second_run = replacement.run.id;
    measure_again(&mut world, &key, second_run);
}

fn measure_again(world: &mut temper_engine_forge_world::World, key: &top::ReceiptKey, second_run: u64) {
    for entry in 700..740 {
        if landings::step_landing(world, 62, entry) == landing::Decision::Finish {
            break;
        }
    }
    let stopped = changes::change_step_task(world, CHANGE, 800, change::Status::Passed, Box::new([]), false, false);
    assert_eq!(stopped, change::Decision::Hold(change::Cause::RedStands));
    world.take_calls();
    let release = top::TaskRelease { revision: 3, at: world.wall(), measure: true };
    world.event(top::Event::ReleaseChange { task: CHANGE, release: Some(release.clone()) });
    world.restart();
    world.event(top::Event::ReleaseChange { task: CHANGE, release: Some(release) });
    for entry in 801..821 {
        changes::change_step_task(world, CHANGE, entry, change::Status::Passed, Box::new([]), false, false);
        if matches!(world.change(CHANGE).change.state, change::State::Ready { .. }) {
            break;
        }
    }
    assert_eq!(world.change(CHANGE).regate, Some(top::Regate { receipt: key.clone(), run: second_run, taken: None }));
    let mut row = landings::landing_row(world.change(CHANGE));
    row.landing.task = 63;
    row.landing.round = landing::Round::new(b"temper/1/l63".as_slice().into());
    row.landing.members[0].branch.clone_from(&row.landing.round.branch);
    landings::register(world, row);
    for entry in 900..980 {
        if matches!(landings::step_landing(world, 63, entry), landing::Decision::Leave { .. }) {
            break;
        }
    }
    let measured = world
        .change(CHANGE)
        .regate
        .as_ref()
        .expect("measurement identity")
        .taken
        .expect("new dispatch took the measurement");
    assert_ne!(measured, second_run);
    assert_eq!(world.change(CHANGE).change.repairs, 1);
    let reruns =
        world.take_calls().into_iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count();
    assert_eq!(reruns, 1, "the accepted act dispatches at the failed key once");
}
