//! CI's family sweep and its cause referee (testing/blame.md, 3–6).
use super::super::{
    blame::judged,
    blame_open::spent_runner,
    landing_batches::workflow,
    open::{check_starved_gate, reported_gate},
};
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_lib::Duration;
use skein_world::{
    streams::{Stream, Streams},
    sweep::{Exclusion, Sweep},
};
use temper_blame::{self as blame};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes, fake_config,
    landings::{self, LANDING},
    translate,
};

fn gate(world: &mut World, file: &[u8], report: bool) {
    assert!(world.project_landing(
        5,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            tiers: Box::new([]),
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: file.into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: report.then(|| landing::GateReport {
                    artifact: b"failed".as_slice().into(),
                    format: landing::Format::Lines,
                    fuzzy: None
                }),
            }]),
        }])
    ));
    world.gate_policy(landing::Project { gate_retries: 1, gate_timeout: Duration::from_secs(5) });
}

fn member(world: &mut World, task: u64, landing_task: u64, base: &[u8]) {
    let branch = format!("temper/ci{task}");
    world.external(raw::Op::Git(raw::Git::Create { branch: branch.as_bytes().into(), commit: world.branch(base) }), 2);
    world.push_file(branch.as_bytes(), b"member", &task.to_be_bytes());
    landings::ready_change_into(world, task, branch.as_bytes(), base);
    let mut row = landings::landing_row(world.change(task));
    row.landing.task = landing_task;
    row.landing.round = landing::Round::new(format!("temper/1/l{landing_task}").into_bytes().into());
    row.landing.members[0].branch.clone_from(&row.landing.round.branch);
    landings::register(world, row);
}

fn cancellation_and_wait(world: &mut World, sweep: &Sweep) {
    gate(world, b"gate.yml", false);
    fake::runners(world.forge_mut(), Some(1));
    world.external(
        raw::Op::Write(raw::Write::Dispatch {
            workflow: b"block.yml".as_slice().into(),
            branch: b"main".as_slice().into(),
            inputs: Box::new([]),
        }),
        1,
    );
    let blocker =
        world.ci_runs().into_iter().find(|run| run.workflow.as_deref() == Some(b"block.yml")).expect("blocker").id;
    member(world, 71, 171, b"main");
    let mut cancelled = false;
    let mut finished = false;
    for entry in 1000..1120 {
        let decision = landings::step_landing(world, 171, entry);
        if !cancelled
            && let Some(run) = world
                .ci_runs()
                .into_iter()
                .find(|run| run.workflow.as_deref() == Some(b"gate.yml") && run.wait == Some(raw::JobWait::Runner))
        {
            assert!(sweep.leaves_out(blame::CANCELLED, &[]).is_none());
            world.cancel_ci(blocker);
            world.cancel_ci(run.id);
            cancelled = true;
        }
        if decision == landing::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(cancelled && finished, "cancelled landing recovers after its runner wait");
    assert_eq!(world.change(71).change.repairs, 0);
    landings::close_landing_at(world, 171, 171_000);
    fake::runners(world.forge_mut(), None);
}

fn held_gate(world: &mut World, file: &[u8], report: bool, task: u64) {
    gate(world, file, report);
    let landing_task = task + 100;
    member(world, task, landing_task, b"main");
    let mut held = false;
    for entry in task * 1000..task * 1000 + 120 {
        if matches!(landings::step_landing(world, landing_task, entry), landing::Decision::Hold(_)) {
            held = true;
            break;
        }
    }
    assert!(held, "runner gate holds without repair: {:?}", world.landing(landing_task));
    assert_eq!(world.change(task).change.repairs, 0);
    for run in world.ci_runs() {
        if run.workflow.as_deref() == Some(file) && run.state == raw::RunState::Running {
            world.external(raw::Op::Write(raw::Write::Cancel { run: run.id }), 1);
        }
    }
    if !report {
        world.event(top::Event::SettleEffects { task, root: task, ending: top::ReleaseEnding::Cancelled });
        let mut heard = super::super::landing_batches::heard(landing::Preparation::Idle, b"");
        heard.released = true;
        landings::step_landing_heard(world, landing_task, task * 1000 + 200, heard);
        for entry in task * 1000 + 201..task * 1000 + 260 {
            if landings::step_landing(world, landing_task, entry) == landing::Decision::Finish {
                break;
            }
        }
        assert!(matches!(
            world.landing(landing_task).landing.phase,
            landing::Phase::Cancelled | landing::Phase::Landed
        ));
    }
}

fn queue(world: &mut World, sweep: &Sweep) {
    assert!(world.project_landing(5, change::Way::Queue, Box::new([])));
    let task = 78;
    let branch = b"temper/ci78";
    world.external(
        raw::Op::Git(raw::Git::Create { branch: branch.as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_file(branch, b"ci", b"run 60s pass");
    let head = translate::commit(world.branch(branch));
    let mut row = changes::change_row();
    row.task = task;
    row.change.task = task;
    row.branch = branch.as_slice().into();
    row.pull = Some(world.open_pull(branch, b"main"));
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    world.event(top::Event::Change { row });
    let run = world.ci_runs().into_iter().find(|run| translate::commit(run.commit) == head).expect("head CI");
    assert_eq!(run.state, raw::RunState::Running);
    assert!(run.workflow.is_none());
    assert!(sweep.leaves_out(blame::CANCELLED, &["a change's CI job"]).is_none());
    world.cancel_ci(run.id);
    assert_eq!(
        changes::change_step_task(world, task, 1400, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Effect(change::Effect::Rerun { head })
    );
    world.run_for(61);
    assert_eq!(
        changes::change_step_task(world, task, 1401, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Ready
    );
    world.landing_ci(b"main", true);
    let tip = translate::commit(world.branch(b"main"));
    let holds_base =
        world.stored().iter().any(|(_, row)| match row {
            top::Stored::Landing(row) => row.landing.members.iter().any(|member| {
                member.base == Some(tip) && member.candidate.is_some() && world.has_branch(&member.branch)
            }),
            _ => false,
        });
    let holding = if holds_base { vec!["a landing candidate holds the red base"] } else { vec![] };
    assert!(sweep.leaves_out(blame::BASE_RED, &holding).is_none(), "queue draw has no live landing candidate");
    world.red_base(b"main");
    assert_eq!(
        changes::change_step_task(world, task, 1402, change::Status::Passed, Box::new([]), false, false),
        change::Decision::QueueRepair
    );
    assert_eq!(world.change(task).change.repairs, 0);
}

#[test]
fn ci_faults_are_blamed_on_their_cause() {
    let mut sweep = Sweep::new("ci_faults_are_blamed_on_their_cause");
    for kind in [
        blame::SET_UP,
        blame::LOST,
        blame::CANCELLED,
        blame::SILENT,
        blame::MISSING_REPORT,
        blame::STARVED,
        blame::RUNNER_WAIT,
        blame::WRONG_TREE,
        blame::BASE_RED,
    ] {
        sweep.configure(kind);
    }
    sweep
        .exclude(Exclusion::new(
            "a_red_inherited_from_the_base_asks_no_repair",
            "a change's reds",
            blame::BASE_RED,
            &["a landing candidate holds the red base"],
        ))
        .expect("named combination");
    for seed in 1..=2 {
        eprintln!("CI seed {seed}");
        let streams = Streams::new(seed);
        let mut silent = workflow(false);
        silent.file = b"silent.yml".as_slice().into();
        silent.silent = 1000;
        let mut missing = workflow(false);
        missing.file = b"missing.yml".as_slice().into();
        missing.passes = 0;
        let mut blocker = workflow(false);
        blocker.file = b"block.yml".as_slice().into();
        blocker.latency_min = Duration::from_secs(600);
        blocker.latency_max = Duration::from_secs(600);
        let mut world =
            World::ci_sweep_in(&streams, Box::new([workflow(true), silent, missing, blocker]), fake_config());
        world.watch_blame();
        world.gate_fault_once(if seed == 1 { blame::LOST } else { blame::SET_UP });
        let workload = streams.seed_of(Stream::Workload, "landing_batches");
        let (mut world, _) = super::landing_batches::run_repairing_in(world, workload, 240);
        let repairs: Vec<_> = world
            .landing(LANDING)
            .landing
            .members
            .iter()
            .filter(|m| matches!(m.left, Some(landing::Leave::Blamed { .. } | landing::Leave::Failed { .. })))
            .map(|m| m.change)
            .collect();
        for task in repairs {
            changes::change_step_task(
                &mut world,
                task,
                1500 + task,
                change::Status::Passed,
                Box::new([]),
                false,
                false,
            );
        }
        cancellation_and_wait(&mut world, &sweep);
        queue(&mut world, &sweep);
        world.status(b"main", raw::Check::Passed);
        held_gate(&mut world, b"silent.yml", false, 72);
        held_gate(&mut world, b"missing.yml", true, 73);
        sweep.ran(&judged(&mut world, seed));

        let mut starved = reported_gate(streams.seed_of(Stream::Workload, "starved_gate"), &[], true);
        check_starved_gate(&mut starved);
        sweep.ran(&judged(&mut starved, seed));

        let fault = if seed == 1 { raw::Failing::NoStep } else { raw::Failing::Setup };
        sweep.ran(&judged(&mut spent_runner(seed, fault), seed));
    }
    sweep.check_fell().expect("each configured CI fault fell");
}
