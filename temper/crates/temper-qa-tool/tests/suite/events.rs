//! Event delivery uses a local fake peer, never a box or network (qa/README.md, 4 and 6).
use super::world::{Agent, World, baseline, land};
use std::{fs, io::Read, os::unix::fs::PermissionsExt};
use temper_qa_tool::{
    Refusal,
    events::{self, Events},
    faces::{ClockFace, RealClock},
};

#[test]
fn a_socket_poke_wakes_and_a_burst_runs_one_tick() {
    let world = World::new();
    let mut run = world.start(baseline(), &[]);
    // A separate listener supplies deterministic accepts to the loop's queue.
    let peer = world.home.directory.join("peer");
    fs::create_dir(&peer).expect("peer home");
    let listener = events::bind(&peer).expect("fake wake socket");
    run.tick();
    run.git.calls.clear();
    for _ in 0..20 {
        events::poke(&peer).expect("poke");
        events::accept(&listener, &run.waker()).expect("accept wakes");
    }
    run.cycle();
    assert_eq!(run.clock.time, 0);
    assert_eq!(run.git.calls, ["head refs/heads/main"], "one tick for the burst");
    assert_eq!(run.wait_timeout().expect("idle"), None);
}

#[test]
fn saved_commands_poke_after_commit_and_refusals_never_poke() {
    let world = World::new();
    let listener = events::bind(&world.home.directory).expect("socket");
    listener.set_nonblocking(true).expect("fake accept");
    world
        .home
        .transact(|ledger| {
            ledger.record_head("committed");
            Ok(())
        })
        .expect("save");
    let (mut stream, _) = listener.accept().expect("save pokes");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("command closes without waiting for a reply");
    assert_eq!(bytes.len(), 0);
    assert_eq!(world.ledger().head.as_deref(), Some("committed"));
    let result = world.home.transact(|ledger| {
        ledger.record_head("refused");
        Err::<(), _>(Refusal("refuse".into()))
    });
    assert!(result.is_err());
    assert_eq!(listener.accept().expect_err("no refused poke").kind(), std::io::ErrorKind::WouldBlock);
    drop(listener);
    world.home.transact(|_| Ok(())).expect("refused connection is no error");
    fs::remove_file(world.home.directory.join("run.sock")).expect("remove stale socket");
    world.home.transact(|_| Ok(())).expect("missing socket is no error");
}

#[test]
fn socket_binding_replaces_stale_paths_and_ignores_the_umask() {
    if super::place::under_umasks(
        "events::socket_binding_replaces_stale_paths_and_ignores_the_umask",
        &["022", "000", "077"],
    ) {
        return;
    }
    let world = World::new();
    let path = world.home.directory.join("run.sock");
    fs::write(&path, "stale").expect("stale path");
    let run = world.start(baseline(), &[]);
    assert_eq!(fs::metadata(&path).expect("socket").permissions().mode() & 0o777, 0o600);
    assert!(
        temper_qa_tool::runner::Run::start(
            world.home.clone(),
            baseline(),
            super::world::Agents::default(),
            super::world::Clock::default(),
            vec![]
        )
        .is_err()
    );
    drop(run);
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
    RealClock::default().wait(&events, None);
    assert!(!events.take(), "wait consumed the event");
}

#[test]
fn a_child_waiter_publishes_its_exit_before_waking_the_loop() {
    use temper_qa_tool::{
        Exit,
        faces::{ProcessCommand, Processes, RealProcesses},
    };
    let world = World::new();
    let events = Events::default();
    let mut processes = RealProcesses::default();
    processes.wake_on_exit(events.waker());
    let input = world.home.directory.join("input");
    fs::write(&input, "").expect("stdin");
    let process = processes
        .start(&ProcessCommand {
            arguments: vec!["/bin/sh".into(), "-c".into(), "exit 7".into()],
            directory: world.home.directory.clone(),
            stdin: input,
            output: world.home.directory.join("output"),
            environment: vec![],
        })
        .expect("fake child");
    assert!(events.wait(Some(2)), "exit sends an event before the timeout");
    assert_eq!(processes.poll(process).expect("published exit"), Some(Exit::Code(7)));
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
