//! A checking change's pace (domain/forge/keeping-up.md, 2), through HTTP and signed deliveries.
use super::*;
use skein_lib::Duration;
use std::fmt::Write;
use temper_limits_fixtures::small;

// One checking change and its run branch, at the fixture seed used for Forgejo CI.
const SEED: u64 = 71;
const CHANGE: &[u8] = b"temper/51";
// The pace case ends one job every twenty seconds for ten minutes.
const PACE_JOBS: u32 = 30;

fn limits() -> top::Limits {
    let shipped = small().forge;
    let mut limits = temper_engine_forge_world::LIMITS;
    limits.client = shipped.client;
    // Thirty job rows fit one bounded HTTP page; only shape bounds grow, not paces.
    limits.client.rows = 64;
    // Keep this ten-minute story inside the unpushed branch's fallback.
    limits.client.fallback = Duration::from_secs(900);
    limits.change_policy.stall = shipped.change_policy.stall;
    limits.resources_per_task = shipped.resources_per_task;
    limits
}

fn adopted(limits: &top::Limits, hearing: bool, jobs: u32) -> Protocol {
    let mut world = Protocol::forgejo_ci(SEED, limits, hearing, jobs);
    world.adopt();
    let run = branch_name(b"temper/1/r1");
    world.event(top::Event::Names { task: 1, resources: Box::new([run.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: 1, resource: run, from: None });
    world
}

fn pending(hearing: bool) -> (Protocol, Wall, Wall, u64) {
    let mut world = adopted(&limits(), hearing, PACE_JOBS);
    let mut unpushed = change_row();
    unpushed.task = 52;
    unpushed.change.task = 52;
    unpushed.branch = b"temper/unpushed".as_slice().into();
    unpushed.goal = Some(1);
    world.event(top::Event::Change { row: unpushed });
    world.event(top::Event::Names { task: 52, resources: Box::new([branch_name(b"main")]) });
    waiting_step(&mut world, 1);
    world.run_for(1);
    let (pushed, head) = open_change(&mut world);
    let verdict = Wall::from_nanos(pushed.as_nanos() + Duration::from_secs(600).as_nanos());
    (world, pushed, verdict, head)
}

fn open_change(world: &mut Protocol) -> (Wall, u64) {
    let mut row = change_row();
    row.goal = Some(1);
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    let branch = branch_name(CHANGE);
    world.event(top::Event::Names { task: 51, resources: Box::new([branch.clone(), branch_name(b"main")]) });
    world.event(top::Event::Hold { entry: 0, task: 51, resource: branch, from: None });
    world.run_for(0);
    assert_eq!(
        change_step(world, 2, change::Status::Unknown, Box::new([])),
        change::Decision::Delegate(change::Delegate::Produce)
    );
    let pushed = world.wall();
    world.produce(CHANGE);
    let head = world.branch(CHANGE);
    world.take_seen();
    step_change(world, 3, change::Status::Passed, false);
    let mut opened = false;
    for _ in 0..90 {
        for request in world.take_seen() {
            if let top::Request::ChangeDecision {
                task: 51,
                decision: change::Decision::Effect(change::Effect::Open),
                entry: Some(entry),
                ..
            } = request
            {
                world.event(top::Event::Committed { entry });
                opened = true;
            }
        }
        if opened {
            break;
        }
        world.run_for(1);
    }
    assert!(opened, "opening decides after its complete fresh reads");
    (pushed, head)
}

fn waiting_step(world: &mut Protocol, entry: u64) {
    world.event(top::Event::StepChange {
        task: 52,
        entry,
        heard: change::Heard {
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            review_request: None,
            closer_waiting: false,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([change::Pending { task: 40, kind: change::PendingKind::Other }]),
        kept: false,
        merge_job: false,
    });
}

#[expect(
    clippy::too_many_lines,
    reason = "one clock drives both changes, their request budget and each hearing gather"
)]
fn pace(hearing: bool) {
    let (mut world, pushed, verdict, head) = pending(hearing);
    let poll_max = small().forge.client.poll_max;
    let deadline = Wall::from_nanos(verdict.as_nanos() + poll_max.as_nanos());
    let (mut due, mut until, mut entry, mut named) = (true, None, 4, false);
    let (mut job, mut written) = (0, 0);
    let status = status_route(head);
    let mut gathering = false;
    let mut gathers = 0;
    let mut unpushed_due = false;
    let mut next_dependency = 30;
    while world.wall() <= deadline && world.tally().routes.get(&Route::Merge) != Some(&1) {
        let seconds = (world.wall().as_nanos() - pushed.as_nanos()) / 1_000_000_000;
        // Alternating ends have their status written three seconds after delivery.
        if job < PACE_JOBS && seconds >= u64::from(job + 1) * 20 {
            if hearing {
                world.job_end(b"temper/51", job);
            }
            if job % 2 == 1 {
                world.job_status(b"temper/51", job);
                written += 1;
            }
            job += 1;
        }
        if written < job && seconds >= u64::from(job) * 20 + 3 {
            world.job_status(b"temper/51", written);
            written += 1;
        }
        if !named {
            named = ci_poll::name_pull(&mut world);
        }
        for request in world.take_seen() {
            match request {
                top::Request::WakeProcedure { task: 51 } => due = true,
                top::Request::WakeProcedure { task: 52 } => unpushed_due = true,
                top::Request::ChangeDecision { task: 52, decision, .. } => {
                    assert_eq!(decision, change::Decision::Wait { until: None }, "task dependency stays pending");
                }
                top::Request::ChangeDecision { task: 51, decision, entry: committed, .. } => {
                    if hearing && gathering {
                        gathering = false;
                        gathers += 1;
                    }
                    if let Some(entry) = committed {
                        world.event(top::Event::Committed { entry });
                    }
                    match decision {
                        change::Decision::Wait { until: at } => until = at,
                        change::Decision::Hold(why) => panic!("pending CI held: {why:?}"),
                        _ => due = true,
                    }
                }
                _ => {}
            }
        }
        if until.is_some_and(|at| world.wall() >= at) {
            due = true;
            until = None;
        }
        if seconds >= next_dependency || unpushed_due {
            // A dependency can wake a procedure without making its branch due.
            waiting_step(&mut world, entry);
            entry += 1;
            next_dependency = seconds + 30;
            unpushed_due = false;
        }
        if due {
            due = false;
            // The opening overlaps its first keep when its settlement records
            // the number. Measure checking gathers, not that bootstrap.
            let checking = matches!(world.stored().get(&top::Key::ChangeHot(51)),
                Some(top::Stored::ChangeHot(row)) if matches!(row.change.state, change::State::Checking { .. }));
            if hearing && checking {
                gathering = true;
            }
            step_change(&mut world, entry, change::Status::Unknown, false);
            entry += 1;
            world.run_for(0);
        } else {
            world.run_for(1);
        }
    }
    assert_eq!(
        request_routes(&world, Wall::EPOCH, world.wall()).get("GET /api/v1/repos/org/repo/branches/temper%2Funpushed"),
        Some(&1),
        "the absent branch is read once, before the measured minutes",
    );
    let first = pushed.as_nanos() + Duration::from_secs(120).as_nanos();
    let minute = Duration::from_secs(60).as_nanos();
    let mut maxima = [0; 3];
    let mut worst = std::collections::BTreeMap::new();
    for (at, _) in
        world.requests().filter(|(at, _)| at.as_nanos() >= first && at.as_nanos() + minute <= verdict.as_nanos())
    {
        let routes = request_routes(&world, at, Wall::from_nanos(at.as_nanos() + minute));
        for route in [
            "GET /api/v1/repos/org/repo/branches/temper%2Funpushed",
            "GET /api/v1/repos/org/repo/pulls/main/temper%2Funpushed",
            "GET /api/v1/repositories/2",
        ] {
            assert_eq!(routes.get(route).copied().unwrap_or(0), 0, "unpushed route {route}: {routes:?}");
        }
        let total = routes.values().sum();
        let statuses = routes.get(&status).copied().unwrap_or(0);
        let pull = routes.get("GET /api/v1/repos/org/repo/pulls/1").copied().unwrap_or(0);
        if total > maxima[0] {
            worst = routes.clone();
        }
        for (max, count) in maxima.iter_mut().zip([total, statuses, pull]) {
            *max = (*max).max(count);
        }
    }
    eprintln!("Forgejo hearing={hearing}: maxima all/status/pull={maxima:?}; worst={worst:?}; status={status}");
    assert_eq!(world.requests().count() as u64, world.tally().total, "each page counts");
    assert!(
        named,
        "opened pull; state={:?}; all={:?}",
        world.stored().get(&top::Key::ChangeHot(51)),
        request_routes(&world, pushed, deadline)
    );
    assert_eq!(world.tally().routes.get(&Route::Merge), Some(&1), "lands within poll_max of verdict");
    assert!(world.wall() <= deadline);
    assert_eq!(world.delivery_hints() > 0, hearing);
    assert!(!hearing || gathers > 0, "observed complete hearing gathers");
    // Recorded HTTP envelopes include periodic fresh checks and their run reads (stalls.md, 2.1).
    let ceilings = if hearing { [51, 10, 2] } else { [52, 6, 4] };
    for (count, ceiling) in maxima.into_iter().zip(ceilings) {
        assert!(count <= ceiling, "{hearing}: {maxima:?} > {ceilings:?}");
    }
}

fn status_route(head: u64) -> String {
    let mut hex = String::new();
    for byte in skein_fake_forge_forgejo::routes::object(head).bytes.iter().take(20) {
        write!(hex, "{byte:02x}").expect("hex in a String");
    }
    format!("GET /api/v1/repos/org/repo/commits/{hex}/status")
}

// The job ends after the push's settlement window; the test ends before a
// keep poll, slow pass or stall read can see the result (setup/webhooks.md, 1).
const SETTLE_SECONDS: u64 = 3;
const JOB_END_SECONDS: u64 = 10;
const DEADLINE_SECONDS: u64 = 30;
// At most four root turns per second: one clock step and up to three change decisions.
const ROOT_TURNS: u64 = 4 * DEADLINE_SECONDS;
const QUIET_SECONDS: u64 = 60;
const ONLY_JOB: u32 = 0;

fn slow_job(deliver_end: bool) -> (bool, bool) {
    let mut limits = limits();
    let quiet = Duration::from_secs(QUIET_SECONDS);
    limits.client.poll = quiet;
    limits.client.poll_max = quiet;
    limits.client.slow = quiet;
    limits.client.settle = Duration::from_secs(SETTLE_SECONDS);
    limits.change_policy.stall = quiet;
    // Both arms subscribe to hooks; only the job's end is omitted in one.
    let mut world = adopted(&limits, true, ONLY_JOB + 1);
    let (pushed, head) = open_change(&mut world);
    let job_end = Wall::from_nanos(pushed.as_nanos() + Duration::from_secs(JOB_END_SECONDS).as_nanos());
    let deadline = Wall::from_nanos(pushed.as_nanos() + Duration::from_secs(DEADLINE_SECONDS).as_nanos());
    let mut schedule = ci_poll::Schedule::after_open();
    let mut named = false;
    let mut passed = false;
    for _ in 0..ROOT_TURNS {
        if world.wall() >= deadline || world.tally().routes.get(&Route::Merge) == Some(&1) {
            break;
        }
        if !passed && world.wall() >= job_end {
            if deliver_end {
                world.job_end(CHANGE, ONLY_JOB);
            }
            world.job_status(CHANGE, ONLY_JOB);
            passed = true;
        }
        if !named {
            named = ci_poll::name_pull(&mut world);
        }
        ci_poll::hear(&mut world, &mut schedule);
        ci_poll::advance(&mut world, &mut schedule);
    }
    let status = status_route(head);
    let read = world.requests().any(|(at, line)| at >= job_end && line.starts_with(status.as_bytes()));
    let landed = world.wall() < deadline
        && world.observations().iter().any(|observation| {
            matches!(observation, Observation::Merged { head: checked, commit, .. }
            if *checked == head && *commit == world.branch(b"main"))
        });
    assert!(world.wall() >= deadline || landed, "the referee reaches the merge or the cutoff");
    (read, landed)
}

#[test]
fn a_slow_job_lands_its_change_only_by_its_delivered_end() {
    assert_eq!(slow_job(true), (true, true), "the signed job end prompts a CI read and merge");
    assert_eq!(slow_job(false), (false, false), "without the job end no read or merge occurs by the cutoff");
}

#[test]
fn a_forgejo_checking_change_hears_jobs_and_reads_execution() {
    pace(true);
}

#[test]
fn a_forgejo_checking_change_without_deliveries_backs_off_and_lands() {
    pace(false);
}
