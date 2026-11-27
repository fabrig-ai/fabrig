//! The close job's wait (temper/docs/design/protocol/wires.md, 8 and 11).
use super::super::close_change::finish_job_on;
use super::*;
use jig_domain_core::{documents, reads, settings::WorkerSetting};
use jig_domain_fleet::{Figures, JobRefusal, JobShort, JobShortfall};
use temper_service_core::limits::CLOSE_DEADLINE;

const FIRST_WORKER: u32 = 1;
const COVERING_WORKER: u32 = 2;
const COVERING_POSITION: u32 = 1;
const COVERING_CHANNEL: Token = Token::new(8);
const LANDING: &[u8] = b"main";
const WORK_GOAL: u32 = 4;
const SHORT_DEADLINE: Duration = Duration::from_secs(60);
const UNTRANSLATED: &[u8] = b"the worker's layout cannot keep jig-channel Job.workspace: field";

struct Close {
    world: World,
    goal: u64,
    change: u64,
    job: u64,
}

fn closing(jobs: Box<[JobBound]>) -> Close {
    let workers = [(FIRST_WORKER, b"first-worker".as_slice()), (COVERING_WORKER, b"covering-worker".as_slice())].map(
        |(number, name)| WorkerSetting {
            number,
            name: name.into(),
            places: WORKER_SLOTS,
            digest: Some([7; 32]),
            withdrawn: false,
        },
    );
    let defaults = stock_defaults();
    let mut world = closer_world(LANDING, engine::LANDING_ATTEMPTS, &defaults, Some(Box::new(workers)));
    hello_goal_worker(&mut world, CHANNEL, WORKER_POSITION, defaults.rules.maximum_run_spend, jobs);
    let (mut world, goal, _, _) = start_goal(world, WORK_GOAL, true, None, None, LANDING);
    answer_parked(&mut world, &goal, 0);
    let closer = closer_of(&mut world, goal.task);
    for _ in 0..STEPS {
        if let Some(change) = closer_row(&world, closer).and_then(|row| row.close_changes.first().copied())
            && let Some((job, _)) = world.store.change_hot(change).and_then(|row| row.delegate)
        {
            return Close { world, goal: goal.task, change, job };
        }
        world.tick();
    }
    panic!("closer T{closer} did not make its close job: {:?}", closer_row(&world, closer));
}

fn assert_due(world: &mut World, job: u64, short: Option<JobShortfall>) {
    world.oracle_settle();
    let reads::Answer::Document(document) = world.party_read(reads::Read::Task { task: job }) else {
        panic!("close job document")
    };
    let documents::Document::Task(task) = *document else { panic!("task document") };
    assert_eq!(task.chip.phase, documents::PhaseLine::Active(tasks::Active::Due));
    assert_eq!(
        task.run_wait,
        Some(documents::TaskRunWait::Refused(documents::TaskWaitRefused::Job {
            kind: forge_top::CLOSE_KIND,
            refusal: JobRefusal { worker: Some(FIRST_WORKER), short },
        }))
    );
    assert!(world.jobs.is_empty(), "no assignment while waiting");
}

fn attempts(world: &World, job: u64) -> Vec<u64> {
    world
        .store
        .rows
        .keys()
        .filter_map(|key| match key {
            Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Attempt { task, attempt })) if *task == job => {
                Some(*attempt)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_close_job_waits_for_a_worker_with_enough_time() {
    let short = JobBound { deadline: SHORT_DEADLINE, ..close_bound() };
    let Close { mut world, goal, change, job } = closing(Box::new([short]));
    assert_due(
        &mut world,
        job,
        Some(JobShortfall {
            why: JobShort::Deadline,
            figures: Some(Figures { required: CLOSE_DEADLINE.as_millis(), offered: SHORT_DEADLINE.as_millis() }),
        }),
    );
    assert!(attempts(&world, job).is_empty(), "no close job attempt was committed");
    finish_with_covering_worker(world, goal, change, job);
}

fn finish_with_covering_worker(mut world: World, goal: u64, change: u64, job: u64) {
    hello_goal_worker(
        &mut world,
        COVERING_CHANNEL,
        COVERING_POSITION,
        stock_defaults().rules.maximum_run_spend,
        Box::new([close_bound()]),
    );
    let assigned = close_job(&mut world);
    assert_eq!(assigned.run.raw(), job, "the same close job is placed");
    // Only the channel that received the assignment can finish it.
    finish_job_on(&mut world, COVERING_CHANNEL, &assigned);
    wait_ended(&mut world, goal);
    assert!(matches!(
        ended(&world, change).expect("close change ended").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { .. }))
    ));
    assert!(matches!(
        ended(&world, goal).expect("goal closed").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { .. }))
    ));
    world.close();
}

#[test]
fn a_close_job_waits_when_the_worker_omits_its_kind() {
    let Close { mut world, goal, change, job } = closing(Box::new([]));
    assert_due(&mut world, job, Some(JobShortfall { why: JobShort::Kind, figures: None }));
    assert!(attempts(&world, job).is_empty(), "no close job attempt was committed");
    finish_with_covering_worker(world, goal, change, job);
}

#[test]
fn an_untranslated_close_job_waits_after_its_claim_again_pause() {
    let Close { mut world, goal, change, job } = closing(Box::new([close_bound()]));
    let assigned = close_job(&mut world);
    assert_eq!(assigned.run.raw(), job);
    let refusal = world.protocol.answer_job(
        CHANNEL,
        job,
        assigned.attempt.raw(),
        host::JobEnding::Refused(host::JobRefusal::Untranslated(UNTRANSLATED.into())),
    );
    world.send(refusal);
    let until = (0..STEPS)
        .find_map(|_| {
            if let tasks::Phase::Active(tasks::Active::BackingOff { until }) = live_task(&world, job).phase {
                return Some(until);
            }
            world.tick();
            None
        })
        .unwrap_or_else(|| panic!("close job did not enter its claim-again pause: {:?}", live_task(&world, job).phase));
    world.advance_to(Time::from_nanos(until.as_nanos()).max(world.env.now));
    assert_due(&mut world, job, None);
    assert_eq!(attempts(&world, job), [assigned.attempt.raw()], "the refusal never opens a second attempt");
    assert_eq!(live_task(&world, job).tries, tasks::Tries::NONE);
    finish_with_covering_worker(world, goal, change, job);
}
