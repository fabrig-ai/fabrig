//! The runner's wakes and deadlines in memory (temper/docs/design/testing/qa/README.md, 6).
use temper_qa_tool::{
    events::{self, Events},
    faces::ClockFace,
    testing::{Agent, Clock, World, baseline, land},
};

#[test]
fn a_socket_poke_wakes_and_a_burst_runs_one_tick() {
    let world = World::new();
    let mut run = world.start(baseline(), &[]);
    let mut files = world.files();
    let listener = files.bound_listener(world.home.directory.join("run.sock")).expect("wake socket");
    run.tick();
    run.git.calls.clear();
    for _ in 0..20 {
        events::poke(&world.home.directory, &mut files).expect("poke");
        events::accept(&listener, &run.waker(), &mut files).expect("accept wakes");
    }
    run.cycle();
    assert_eq!(run.clock.time, 0);
    assert_eq!(run.git.calls, ["head refs/heads/main"], "one tick for the burst");
    assert_eq!(run.wait_timeout().expect("idle"), None);
}

#[test]
fn only_fetch_arms_poll_and_other_events_do_not_postpone_it() {
    let world = World::new();
    let mut run = world.start(baseline(), &[]);
    run.tick();
    assert_eq!(run.wait_timeout().expect("no fetch"), None);
    run.home.configuration.git.fetch = Some("upstream".into());
    run.home.configuration.bounds.poll = 20;
    run.tick();
    assert_eq!(run.wait_timeout().expect("fetch due"), Some(20));
    run.clock.time = 7;
    run.waker().poke();
    run.cycle();
    assert_eq!(run.wait_timeout().expect("same fetch deadline"), Some(13));
    assert_eq!(run.git.calls.iter().filter(|call| call.starts_with("fetch ")).count(), 1);
    run.cycle();
    assert_eq!(run.clock.time, 20);
    assert_eq!(run.git.calls.iter().filter(|call| call.starts_with("fetch ")).count(), 2);
    assert_eq!(run.wait_timeout().expect("next fetch"), Some(20));
}

#[test]
fn build_wait_and_failed_step_retry_are_deadlines() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    run.tick();
    assert_eq!(run.wait_timeout().expect("build wait"), Some(90));
    run.git.fail_head = true;
    run.tick();
    assert_eq!(run.wait_timeout().expect("earlier retry"), Some(60));
    run.clock.time = 60;
    run.tick();
    assert_eq!(run.wait_timeout().expect("build before next retry"), Some(30));
    run.clock.time = 90;
    run.git.fail_head = false;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 1);
    assert_eq!(run.wait_timeout().expect("reported waits disarm"), None);
}

#[test]
fn real_clock_receives_an_event_without_a_timeout() {
    let events = Events::default();
    events.waker().poke();
    Clock::default().wait(&events, None);
    assert!(!events.take(), "wait consumed the event");
}

#[test]
fn the_failure_hour_line_can_precede_its_retry() {
    let world = World::new();
    let mut run = world.start(baseline(), &[]);
    run.git.fail_head = true;
    run.tick();
    run.clock.time = 3599;
    run.tick();
    assert_eq!(run.wait_timeout().expect("line before retry"), Some(1));
    run.cycle();
    assert_eq!(run.clock.time, 3600);
    assert_eq!(world.ledger().lines.len(), 1);
    assert_eq!(run.wait_timeout().expect("hour line disarmed"), Some(60));
}
