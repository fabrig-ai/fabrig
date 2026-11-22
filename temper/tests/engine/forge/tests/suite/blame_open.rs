//! Target rows where the engine's policy differs (testing/blame.md, 6).
use super::blame::{ask_repair, finish_landing, judged, one_change};
use skein_fake_forge_domain::api as raw;
use temper_blame::{Names, Reaction};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes, translate};

pub(crate) fn checking(world: &mut World, declaration: Option<&[u8]>) {
    world.adopt();
    world.produce_file(b"temper/51", b"member", b"change");
    if let Some(declaration) = declaration {
        world.push_file(b"temper/51", b"ci", declaration);
    }
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = changes::change_row();
    row.pull = Some(world.open_pull(b"temper/51", b"main"));
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    world.event(top::Event::Change { row });
}

#[test]
#[ignore = "a change's reds: one way to classify a red, one release rule, a record of temper's own writes"]
fn a_runner_fault_past_its_reruns_holds_without_repair() {
    let mut world = World::runner_failure(1, 0, raw::Failing::Setup);
    world.watch_blame();
    world.head_runner_fault(raw::Failing::Setup);
    checking(&mut world, None);
    assert!(matches!(ask_repair(&mut world), change::Decision::Effect(change::Effect::Rerun { .. })));
    world.run_for(5);
    let decision = ask_repair(&mut world);
    assert_eq!(world.change(51).change.repairs, 0, "runner fault was repaired: {decision:?}");
    assert_eq!(decision, change::Decision::Hold(change::Hold::Stalled));
    assert!(world.blame().iter().any(|o| matches!(o.reaction, Reaction::Hold { names: Names::Runner, .. })));
    judged(&mut world, 1);
}

#[test]
#[ignore = "a change's reds: one way to classify a red, one release rule, a record of temper's own writes"]
fn a_cancelled_check_is_the_runners() {
    let mut world = World::with_check_cue(1, raw::Cue::Declared { path: b"ci".as_slice().into() });
    world.watch_blame();
    checking(&mut world, Some(b"run 60s pass"));
    let head = world.branch(b"temper/51");
    let run = world.ci_runs().into_iter().find(|run| run.commit == head && run.workflow.is_none()).expect("head CI");
    world.cancel_ci(run.id);
    let decision = ask_repair(&mut world);
    assert!(
        matches!(decision, change::Decision::Effect(change::Effect::Rerun { .. })),
        "cancelled job not rerun: {decision:?}"
    );
    assert_eq!(world.change(51).change.repairs, 0);
    judged(&mut world, 1);
}

#[test]
#[ignore = "a change's reds: one way to classify a red, one release rule, a record of temper's own writes"]
fn a_red_inherited_from_the_base_asks_no_repair() {
    let mut world = one_change(World::batch_turn(1, Some(super::landing_batches::workflow(true))), false);
    world.push_file(b"main", b"file", b"broken");
    world.red_base(b"main");
    finish_landing(&mut world, false);
    let decision = ask_repair(&mut world);
    assert_eq!(world.change(51).change.repairs, 0, "base red was repaired on the change: {decision:?}");
    assert!(matches!(decision, change::Decision::Hold(_)));
    assert!(world.blame().iter().any(|o| matches!(o.reaction, Reaction::Hold { names: Names::Base, .. })));
    judged(&mut world, 1);
}
