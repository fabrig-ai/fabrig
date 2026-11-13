//! Fixtures shared by change procedure stories.
use crate::{REPO, World};
use skein_lib::Wall;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;

#[must_use]
pub fn change_row() -> top::ChangeRow {
    top::ChangeRow {
        signed: Box::new([]),
        task: 51,
        repository: REPO,
        branch: Box::from(&b"temper/51"[..]),
        base: Box::from(&b"main"[..]),
        title: Box::from(&b"Small fix"[..]),
        body: Box::from(&b"Fix the file"[..]),
        priority: 1,
        ci: None,
        change: change::Change {
            ended_held: None,
            early: change::Early::None,
            way: temper_engine_domain_forge_change::Way::Queue,
            landing: None,
            task: 51,
            state: change::State::Producing { requested: false },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: Wall::from_nanos(100_000_000_000),
            ready_since: None,
            owns_turn: false,
            last_head: None,
            alone: false,
        },
        pull: None,
        pending: None,
        effect: change::EffectResult::None,
        delegate: None,
        delegate_status: change::Status::Unknown,
        verdicts: Box::new([]),
        goal: None,
        ending: None,
        repair_tasks: Box::new([]),
        resolution_tasks: Box::new([]),
        result: top::ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        producing_llm: None,
        review_llm: None,

        questions: temper_engine_domain_forge_change::Questions::default(),
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        repairs: None,
        follows: None,
        catch_up: None,
        close: None,
        merge_result: None,
        follow_check: None,
    }
}

pub fn change_step(
    world: &mut World,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
) -> change::Decision {
    change_step_with_release(world, entry, heard, gates, false)
}

/// Step the usual change with its merge job available.
pub fn change_step_merge_job(
    world: &mut World,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
) -> change::Decision {
    if world.top.merge_job(5).is_none() {
        assert!(world.project_landing(5, change::Way::Queue, Box::new([])));
    }
    world.top.merge_kind(4096, 4096);
    assert!(world.top.project_merge_job(5, b".temper/merge".as_slice().into()));
    change_step_options(world, 51, entry, heard, gates, false, false, false, true)
}

pub fn change_step_with_release(
    world: &mut World,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
) -> change::Decision {
    change_step_task(world, 51, entry, heard, gates, released, false)
}

pub fn change_step_task(
    world: &mut World,
    task: u64,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
    kept: bool,
) -> change::Decision {
    change_step_task_running(world, task, entry, heard, gates, released, kept, false)
}

#[expect(clippy::too_many_arguments, reason = "the scripted step supplies task standing and forge gates")]
pub fn change_step_task_running(
    world: &mut World,
    task: u64,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
    kept: bool,
    delegate_running: bool,
) -> change::Decision {
    change_step_options(world, task, entry, heard, gates, released, kept, delegate_running, false)
}

#[expect(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "the scripted step supplies task standing, forge gates and a merge job"
)]
fn change_step_options(
    world: &mut World,
    task: u64,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
    kept: bool,
    delegate_running: bool,
    merge_job: bool,
) -> change::Decision {
    world.take_seen();
    world.event(top::Event::StepChange {
        merge_job,
        task,
        entry,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: heard,
            delegate_running,
            delegate_live: false,
            effect: change::EffectResult::None,
            released,
            cancelled: false,
        },
        gates,
        queue_repair_active: false,
        pending: Box::new([]),
        kept,
    });
    world.run_for(1);
    let mut decision = None;
    let mut committed = None;
    for event in world.take_seen() {
        if let top::Request::ChangeDecision { task: decided, decision: current, entry, .. } = event
            && decided == task
        {
            decision = Some(current);
            committed = entry;
        }
    }
    if let Some(entry) = committed {
        world.event(top::Event::Committed { entry });
        world.run_for(1);
    }
    decision.expect("change step made a decision")
}

#[must_use]
pub fn reviewed_change(queued: bool, freshness: change::Freshness) -> (World, skein_forge::api::Commit) {
    let mut world = World::new(17);
    world.adopt();
    world.produce(b"temper/51");
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Passed);
    let pull = world.open_pull(b"temper/51", b"main");
    let head = crate::translate::commit(world.branch(b"temper/51"));
    let mut row = change_row();
    row.change.state = if queued {
        change::State::Queued { head, ready_since: Wall::from_nanos(100_000_000_000) }
    } else {
        change::State::Gating { head, asked: Box::new([]) }
    };
    row.change.last_head = Some(head);
    row.pull = Some(pull);
    row.change.gates =
        Box::new([change::Gate { number: 9, kind: change::GateKind::Agent, blocking: true, freshness, eager: false }]);
    world.event(top::Event::Change { row });
    (world, head)
}
