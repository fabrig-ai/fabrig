//! Event delivery uses a local fake peer, never a box or network (qa/README.md, 4 and 6).
use super::world::{World, baseline};
use std::{fs, io::Read, os::unix::fs::PermissionsExt};
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{
    Refusal,
    events::{self, Events},
};

#[test]
fn saved_commands_poke_after_commit_and_refusals_never_poke() {
    let world = World::new();
    let listener = events::bind(&world.home.directory, &mut RealFiles).expect("socket");
    listener.set_nonblocking(true).expect("fake accept");
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.record_head("committed");
            Ok(())
        })
        .expect("save");
    let (mut stream, _) = listener.accept().expect("save pokes");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("command closes without waiting for a reply");
    assert_eq!(bytes.len(), 0);
    assert_eq!(world.ledger().head.as_deref(), Some("committed"));
    let result = world.home.transact(&mut RealFiles, |ledger, _| {
        ledger.record_head("refused");
        Err::<(), _>(Refusal("refuse".into()))
    });
    assert!(result.is_err());
    assert_eq!(listener.accept().expect_err("no refused poke").kind(), std::io::ErrorKind::WouldBlock);
    drop(listener);
    world.home.transact(&mut RealFiles, |_, _| Ok(())).expect("refused connection is no error");
    fs::remove_file(world.home.directory.join("run.sock")).expect("remove stale socket");
    world.home.transact(&mut RealFiles, |_, _| Ok(())).expect("missing socket is no error");
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
            world.agents(),
            super::world::Clock::default(),
            RealFiles,
            vec![]
        )
        .is_err()
    );
    drop(run);
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
