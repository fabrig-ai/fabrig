//! Given listeners survive exec, retain their backlog and remain the parent's.

#[path = "../support/given_listener.rs"]
mod support;

#[path = "../support/deadline.rs"]
pub(super) mod deadlines;

use std::net::{Ipv4Addr, TcpStream};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use skein_io::kernel::{Exit, Fd, Pipe, Spawn, Way};
use skein_io::{self as io, Event, Request};
use skein_lib::{Duration, Env, Token};
use skein_shell::{Backend, Clock, Config, Kernel, Wait};
use skein_world::Expired;

fn spawn_and_exit(
    kernel: &mut Kernel,
    state: &mut io::Io,
    root: Fd,
    held: Token,
    address: std::net::SocketAddr,
    client: TcpStream,
) {
    let clock = Clock::new();
    let limits = support::limits();
    let now = clock.now();
    let env = Env { now: now.now, wall: now.wall, limits };
    let mut deadline = deadlines::Patience::new("the given listener child settling", Duration::from_secs(3));
    let (mut submissions, mut completions, mut events) = deadlines::records(16);
    let spawn = Spawn {
        program: Box::from(skein_scratch::paths::binary("process_fixture").as_os_str().as_bytes()),
        args: Box::new([Box::from(&b"given-listener"[..]), Box::from(address.to_string().as_bytes())]),
        env: Box::new([Box::from(&b"LISTEN_FDS=1"[..])]),
        root,
        dir: Box::from(&b"."[..]),
        pipes: Box::new([Pipe { child: 3, way: Way::Given(held), parent: None }]),
    };
    io::down(state, &env, Request::Spawn { owner: Token::new(100), spawn }, &mut submissions);
    let mut owned = vec![held];
    let mut spawned = false;
    let mut exited = false;
    let mut closed = false;
    loop {
        state.reclaim();
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        while state.is_ready() {
            io::resume(state, &env, &mut events, &mut submissions);
        }
        kernel.reap(&mut completions);
        while let Some(complete) = completions.pop() {
            deadline.watch_spawn(&complete);
            io::up(state, &env, complete, &mut events, &mut submissions);
        }
        while let Some(event) = events.pop() {
            match event {
                Event::Spawned { owner, child, pipes } => {
                    owned.push(child);
                    owned.extend_from_slice(&pipes);
                    assert_eq!(owner, Token::new(100));
                    assert!(!spawned);
                    assert!(pipes.is_empty());
                    spawned = true;
                }
                Event::Exited { owner, exit } => {
                    assert!(spawned && !exited);
                    assert_eq!((owner, exit), (Token::new(100), Exit::Code(0)));
                    exited = true;
                }
                Event::Closed { owner } => {
                    assert!(exited && !closed);
                    assert_eq!(owner, Token::new(100));
                    closed = true;
                }
                other @ (Event::Listening { .. }
                | Event::Accepted { .. }
                | Event::Connecting { .. }
                | Event::Connected { .. }
                | Event::Stream { .. }
                | Event::Output { .. }
                | Event::Usage { .. }
                | Event::Resized
                | Event::Window { .. }
                | Event::Shutdown { .. }
                | Event::Failed { .. }) => panic!("unexpected spawn event: {other:?}"),
            }
        }
        state.reclaim();
        if closed {
            assert_eq!(kernel.in_flight(), 0);
            return;
        }
        if deadline.passed() {
            match deadline.expired() {
                Expired::Starved(starved) => {
                    deadlines::end_io(kernel, state, limits, &mut submissions, &mut completions, &mut events, &owned);
                    skein_shell::close_keeper_fd(root);
                    drop(client);
                    starved.pass();
                }
                Expired::Failed(figures) => panic!("given child must settle: {figures}"),
            }
        }
        let until = now.now.saturating_add(Duration::from_millis(5)).min(deadline.until());
        kernel.submit(&mut submissions, Wait::Until(until));
    }
}

fn given_listener(backend: Backend) {
    let (fd, address) = skein_shell::open_listener((Ipv4Addr::LOCALHOST, 0).into(), 2).expect("held listener");
    let root = skein_shell::open_root(Path::new(".")).expect("spawn root");
    let limits = support::limits();
    let mut state = io::Io::new(&limits);
    let held = state.adopt_listener(fd, address).expect("adopt held listener");
    let mut kernel = Kernel::open_on(Config { operations: io::operations(&limits).expect("operation bound") }, backend)
        .expect("parent kernel");
    for _ in 0..2 {
        let client = TcpStream::connect(address).expect("connection queued before spawn");
        spawn_and_exit(&mut kernel, &mut state, root, held, address, client);
        assert_eq!(skein_shell::listener_at(fd), Ok(address), "parent still holds a listening socket after exit");
    }
    let _client = TcpStream::connect(address).expect("parent copy accepts after both children exit");
    support::claim_and_accept(&mut kernel, &mut state, held, address);
    skein_shell::close_keeper_fd(root);
}

#[test]
fn given_listener_survives_exec_and_child_exit_on_readiness() {
    given_listener(Backend::Readiness);
}

#[cfg(target_os = "linux")]
#[test]
fn given_listener_survives_exec_and_child_exit_on_ring() {
    given_listener(Backend::Ring);
}

#[test]
fn cleanup_aborts_a_child_discovered_while_draining() {
    let backends = [
        Backend::Readiness,
        #[cfg(target_os = "linux")]
        Backend::Ring,
    ];
    for backend in backends {
        let root = skein_shell::open_root(Path::new(".")).expect("spawn root");
        let limits = support::limits();
        let mut state = io::Io::new(&limits);
        let mut kernel =
            Kernel::open_on(Config { operations: io::operations(&limits).expect("operation bound") }, backend)
                .expect("parent kernel");
        let clock = Clock::new();
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        let (mut submissions, mut completions, mut events) = deadlines::records(16);
        let spawn = Spawn {
            program: Box::from(skein_scratch::paths::binary("process_fixture").as_os_str().as_bytes()),
            args: Box::new([Box::from(&b"never"[..])]),
            env: Box::new([]),
            root,
            dir: Box::from(&b"."[..]),
            pipes: Box::new([Pipe { child: 0, way: Way::In, parent: None }]),
        };
        io::down(&mut state, &env, Request::Spawn { owner: Token::new(100), spawn }, &mut submissions);
        kernel.submit(&mut submissions, Wait::No);
        kernel.reap(&mut completions);
        #[cfg(target_os = "linux")]
        let keeper = {
            let pidfd = completions
                .iter()
                .find_map(|complete| match complete.result {
                    Ok(skein_io::kernel::Done::Spawned { pidfd, .. }) => Some(pidfd),
                    _ => None,
                })
                .expect("the synchronous spawn completed");
            skein_shell::open_child_pidfd(skein_shell::child_id(pidfd).expect("spawn PID"), std::process::id())
                .expect("retain the child's identity for its reap check")
        };
        deadlines::end_io(&mut kernel, &mut state, limits, &mut submissions, &mut completions, &mut events, &[]);
        skein_shell::close_keeper_fd(root);
        assert!(state.is_empty(), "cleanup aborts the child and its pipe");
        assert_eq!(kernel.in_flight(), 0, "cleanup drains every kernel record");
        #[cfg(target_os = "linux")]
        {
            let reaped = skein_shell::poll_child(keeper, false);
            skein_shell::close_keeper_fd(keeper);
            assert_eq!(reaped, Err(skein_io::kernel::Error::Other(libc::ECHILD)), "cleanup reaps the child");
        }
    }
}

#[test]
fn an_idle_patience_fails_with_its_wait_name() {
    const IDLE: Duration = Duration::from_millis(20);
    let deadline = deadlines::Patience::new("the idle conformance test wait", IDLE);
    std::thread::sleep(std::time::Duration::from_nanos(deadline.remaining().as_nanos()));
    match deadline.expired() {
        Expired::Starved(starved) => starved.pass(),
        Expired::Failed(figures) => {
            assert_eq!(figures.what, "the idle conformance test wait");
            assert_eq!(figures.patience, IDLE);
            assert!(figures.elapsed >= IDLE);
        }
    }
}
