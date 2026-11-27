//! Target rows where the engine's policy differs (testing/blame.md, 6).
use super::blame::{ask_repair, finish_landing, judged, one_change};
use skein_fake_forge_domain::api as raw;
use temper_blame::{Names, Reaction};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{LIMITS, World, changes, translate};

const TASK: u64 = 51;
const BRANCH: &[u8] = b"temper/51";
const RERUNS: u32 = LIMITS.change_policy.reruns;
const JOB_FINISH: u64 = 5; // the runner fixture completes a re-run within five seconds

pub(crate) fn checking(world: &mut World, declaration: Option<&[u8]>) {
    world.adopt();
    world.produce_file(BRANCH, b"member", b"change");
    if let Some(declaration) = declaration {
        world.push_file(BRANCH, b"ci", declaration);
    }
    let head = translate::commit(world.branch(BRANCH));
    let mut row = changes::change_row();
    row.pull = Some(world.open_pull(BRANCH, b"main"));
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    world.event(top::Event::Change { row });
}

pub(crate) fn spent_runner(seed: u64, fault: raw::Failing) -> World {
    let mut world = World::runner_failure(seed, 0, fault);
    world.watch_blame();
    world.head_runner_fault(fault);
    checking(&mut world, None);
    run_allowance(&mut world, 100);
    world
}

fn run_allowance(world: &mut World, first_entry: u64) {
    let head = translate::commit(world.branch(BRANCH));
    let before = world.writes();
    let repairs = world.change(TASK).change.repairs;
    for attempt in 0..RERUNS {
        assert_eq!(
            change_step(world, first_entry + u64::from(attempt)),
            change::Decision::Effect(change::Effect::Rerun { head })
        );
        world.run_for(JOB_FINISH);
    }
    let decision = change_step(world, first_entry + u64::from(RERUNS));
    assert_eq!(decision, change::Decision::Hold(change::Cause::Runner));
    assert_eq!(world.change(TASK).change.repairs, repairs);
    assert!(world.change(TASK).repair_tasks.is_empty());
    assert_eq!(world.writes() - before, RERUNS);
}

fn change_step(world: &mut World, entry: u64) -> change::Decision {
    changes::change_step(world, entry, change::Status::Unknown, Box::new([]))
}

fn head_job(world: &World) -> u64 {
    let head = world.branch(BRANCH);
    let run = world.ci_runs().into_iter().find(|run| run.commit == head && run.workflow.is_none()).expect("head CI");
    let raw::Answer::Jobs(jobs) = world.inspect(&raw::Read::Jobs { run: run.id }) else { panic!("head jobs") };
    jobs.first().expect("the build job").id
}

#[test]
fn a_runner_fault_past_its_reruns_holds_without_repair() {
    let mut world = spent_runner(1, raw::Failing::Setup);
    let job = head_job(&world);
    let Some(top::RunnerCi::Held { words }) = &world.change(TASK).runner_ci else { panic!("runner hold account") };
    let words = core::str::from_utf8(words).expect("hold words");
    assert!(words.contains(&format!("job {job} of check build: set-up failed, run again {RERUNS} time")), "{words}");
    assert!(words.contains("the change wasn't judged"), "{words}");
    assert!(world.blame().iter().any(|o| matches!(o.reaction, Reaction::Hold { names: Names::Runner, .. })));
    judged(&mut world, 1);
}

#[test]
fn releasing_a_runner_hold_counts_job_attempts_afresh() {
    const RELEASE_REVISION: u64 = 1; // the first hold on this change
    let mut world = spent_runner(1, raw::Failing::Setup);
    // The party can run the job while the change is held. Count from this
    // attempt at release, not the attempt that brought the hold.
    let job = head_job(&world);
    world.external(raw::Op::Write(raw::Write::Rerun { job }), 2);
    world.run_for(JOB_FINISH);
    let release = top::TaskRelease { revision: RELEASE_REVISION, at: world.wall() };
    world.event(top::Event::ReleaseChange { task: TASK, release: Some(release.clone()) });
    // The accepted release must still use a fresh baseline after restart.
    world.restart();
    run_allowance(&mut world, 200);
    world.event(top::Event::ReleaseChange { task: TASK, release: Some(release) });
    assert!(matches!(
        world.change(TASK).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Runner, .. }, .. }
    ));
}

#[test]
fn a_green_head_releases_the_runner_hold_and_continues() {
    let mut world = spent_runner(1, raw::Failing::Setup);
    world.status(b"main", raw::Check::Passed);
    world.status(BRANCH, raw::Check::Passed);
    assert_eq!(change_step(&mut world, 300), change::Decision::EndHold);
    assert_eq!(change_step(&mut world, 301), change::Decision::Ready);
}

#[test]
fn a_cancelled_check_is_the_runners() {
    let mut world = World::with_check_cue(1, raw::Cue::Declared { path: b"ci".as_slice().into() });
    world.watch_blame();
    checking(&mut world, Some(b"run 60s pass"));
    let head = world.branch(BRANCH);
    let run = world.ci_runs().into_iter().find(|run| run.commit == head && run.workflow.is_none()).expect("head CI");
    world.cancel_ci(run.id);
    let decision = ask_repair(&mut world);
    assert!(
        matches!(decision, change::Decision::Effect(change::Effect::Rerun { .. })),
        "cancelled job not rerun: {decision:?}"
    );
    assert_eq!(world.change(TASK).change.repairs, 0);
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
