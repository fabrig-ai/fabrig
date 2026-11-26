//! The loop's pass step over commits, recorded agent actions and movable time (qa/passes.md, 9).
use super::world::{Agent, Loop, World, baseline, land};
use std::{fs, fs::File, os::unix::fs::PermissionsExt, path::Path};
use temper_qa_tool::faces::RealFiles;

const SETUPS: &str = "temper/docs/development/qa/setups";

fn recipe(run: &mut Loop, commit: u64, name: &str, readme: &str) {
    for (file, bytes) in [
        ("recipe", b"recipe 1\nweb see Ready\n".as_slice()),
        ("README.md", readme.as_bytes()),
        ("scripts.json", b"{}".as_slice()),
        ("assets/note", b"asset".as_slice()),
    ] {
        run.git.file_at(commit, &format!("{SETUPS}/{name}/{file}"), bytes);
    }
}

fn ready(world: &World, agent: Agent) -> Loop {
    let mut run = world.start(baseline(), &[Agent::Case, agent]);
    run.tick();
    land(&mut run, 2, 7);
    recipe(&mut run, 2, "project", "# Project\n\nA ready project. More detail.\n");
    world.installable_build(2);
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1, "the pass started");
    run
}

fn hold(place: &Path) -> File {
    fs::create_dir_all(place).expect("place");
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(place.join("drive.lock"))
        .expect("lock file");
    file.try_lock().expect("holder takes lock");
    file
}

#[test]
fn ending_a_pass_tells_the_holder_and_releases_its_place_without_the_ledger_watch() {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixStream,
        time::Duration,
    };
    use temper_qa_tool::{
        faces::{RealClock, RealProcesses},
        hold::{Drive, Holder, WATCH},
    };
    let world = World::new();
    let mut run = ready(&world, Agent::Pass);
    let place = world.home.directory.join("places/1");
    fs::write(place.join("setup"), "fresh\n").expect("setup");
    let fake = place.join("bin/temper-client-journeys");
    // Replace the installed link rather than writing through it into the kept build.
    fs::remove_file(&fake).expect("unlink fake");
    fs::write(
        &fake,
        r#"#!/bin/sh
printf 'ready\n'
IFS= read -r line
[ "$line" = end ] || exit 4
printf 'ended\n'
"#,
    )
    .expect("fake drive");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).expect("executable");
    assert_eq!(WATCH, Duration::from_secs(60));
    let session = world.ledger().passes[0].sessions[0];
    let home = world.home.clone();
    let pid = std::process::id();
    let (bound, listening) = std::sync::mpsc::channel();
    let (done, ended) = std::sync::mpsc::channel();
    #[expect(
        clippy::disallowed_methods,
        reason = "the focused socket peer must serve while the loop sends the end (qa/README.md, 6)"
    )]
    let serving = std::thread::spawn(move || {
        let mut files = RealFiles;
        let mut clock = RealClock::default();
        let mut processes = RealProcesses::default();
        let holder = Holder::start(
            Drive::for_session(&home, session, &mut files, &mut clock).expect("drive"),
            &[],
            Duration::from_hours(1),
            pid,
            &mut files,
            &mut processes,
            &mut clock,
        )
        .expect("holder");
        bound.send(()).expect("test waits for socket binding");
        done.send(holder.serve()).expect("test waits for holder");
    });
    listening.recv().expect("holder has bound the socket");
    // Complete the start handshake before ending the agent.
    let mut stream = UnixStream::connect(place.join("drive.sock")).expect("holder socket");
    writeln!(stream, "start {pid}").expect("start request");
    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response).expect("start response");
    run.tick();
    assert!(
        world.ledger().sessions.iter().find(|record| record.number == session).expect("pass session").exit.is_some()
    );
    ended
        .recv_timeout(Duration::from_secs(2))
        .expect("socket end arrives before the hour-long fallback")
        .expect("holder ends from socket request");
    serving.join().expect("holder thread");
    // The loop's own close takes the lock a moment to see it released; the place frees within a bound.
    let lock = File::open(place.join("drive.lock")).expect("lock");
    let mut tries = 200;
    while let Err(why) = lock.try_lock() {
        tries -= 1;
        assert!(tries > 0, "place freed within 2 s: {why:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!place.join("drive.sock").exists());
}

#[test]
fn ending_a_pass_captures_its_holder_before_the_socket_can_be_reused() {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixListener,
        time::Duration,
    };
    let world = World::new();
    let mut run = ready(&world, Agent::Pass);
    let place = world.home.directory.join("places/1");
    let lock = hold(&place);
    let socket = place.join("drive.sock");
    let old = UnixListener::bind(&socket).expect("old holder");
    old.set_nonblocking(true).expect("probe captured connection");
    run.tick();
    fs::remove_file(&socket).expect("old path removed");
    let later = UnixListener::bind(&socket).expect("later holder");
    later.set_nonblocking(true).expect("probe later socket");
    let (mut stream, _) = old.accept().expect("loop captured the old holder before permitting reuse");
    // BSD accept inherits nonblocking mode; reading the end request needs to wait for bytes.
    stream.set_nonblocking(false).expect("blocking old holder stream");
    stream.set_read_timeout(Some(Duration::from_secs(2))).expect("request deadline");
    let mut request = String::new();
    BufReader::new(&mut stream).read_line(&mut request).expect("old holder gets request");
    assert_eq!(request, "end\n");
    assert_eq!(later.accept().expect_err("later holder gets no end").kind(), std::io::ErrorKind::WouldBlock);
    drop(lock);
    stream.write_all(b"0\nended\n").expect("end reply");
}
