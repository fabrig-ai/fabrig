//! A single-threaded service: block SIGTERM before spawning its sender.

#[cfg(target_os = "linux")]
#[path = "../../tests/support/deadline.rs"]
#[expect(
    dead_code,
    reason = "the fixture shares deadline and io cleanup support with the test process's polling waits"
)]
mod deadlines;

#[cfg(target_os = "linux")]
#[expect(
    clippy::too_many_lines,
    reason = "the single-threaded signal fixture owns startup and drains io to settlement"
)]
fn main() {
    use skein_io::kernel::{Pipe, ServiceSignal, Spawn, Way};
    use skein_io::{self as io, Event, Request};
    use skein_lib::{Duration, Env, Token};
    use skein_shell::{Backend, Clock, Config, Kernel, Wait};
    use std::os::unix::ffi::OsStrExt;

    // Three entities give io 14 operation slots; 32 records hold a complete
    // batch and the events produced while it lands.
    const RECORDS: u32 = 32;
    skein_fail_stop::install();
    let mut args = std::env::args().skip(1).peekable();
    match args.peek().map(String::as_str) {
        Some("delayed-parent") => {
            use std::io::Write;
            let _mode = args.next();
            let record = args.next().expect("record path");
            let selection = args.next().expect("record selection");
            #[expect(
                clippy::zombie_processes,
                reason = "this recording-only orphan is adopted and waited on by the test's subreaper"
            )]
            let child = std::process::Command::new(std::env::current_exe().expect("fixture path"))
                .args(["record", &std::process::id().to_string(), &record, &selection])
                .stdin(std::process::Stdio::inherit())
                .spawn()
                .expect("spawn recording sender");
            println!("{}", child.id());
            std::io::stdout().flush().expect("report sender");
            return;
        }
        Some("record") => {
            use std::io::BufRead;
            let _mode = args.next();
            let parent: u32 = args.next().expect("intended parent").parse().expect("parent PID");
            let record = args.next().expect("record path");
            let selection = args.next().expect("selection");
            let mut line = String::new();
            if std::io::stdin().lock().read_line(&mut line).expect("release recording sender") == 0 {
                return;
            }
            // A fake target records the adoption hazard; this mode never sends a signal.
            let target = if selection == "legacy" {
                std::os::unix::process::parent_id().to_string()
            } else if let Some(descriptor) = intended_parent(parent) {
                let target = skein_shell::child_id(descriptor).expect("validated parent").to_string();
                skein_shell::close_keeper_fd(descriptor);
                target
            } else {
                "refused".into()
            };
            std::fs::write(record, target).expect("record target");
            return;
        }
        Some("sender") => {
            use skein_io::kernel::{Signal, Target};
            use std::io::BufRead;
            let _mode = args.next();
            let parent: u32 = args.next().expect("intended parent").parse().expect("parent PID");
            if let Some(descriptor) = intended_parent(parent) {
                let sent = skein_shell::signal_kept_child(descriptor, Signal::Terminate, Target::Child);
                skein_shell::close_keeper_fd(descriptor);
                // The intended parent can exit after validation. ESRCH is a safe refusal.
                if sent.is_ok() {
                    let mut line = String::new();
                    std::io::stdin().lock().read_line(&mut line).expect("parent settled its signal");
                }
            }
            return;
        }
        _ => {}
    }
    let backend = match args.next().as_deref() {
        Some("ring") => Backend::Ring,
        Some("readiness") => Backend::Readiness,
        other => panic!("unknown backend {other:?}"),
    };
    let from_child = match args.next().as_deref() {
        Some("child") => true,
        Some("self") => false,
        other => panic!("unknown sender {other:?}"),
    };
    let signals = skein_shell::open_termination_signals().expect("block signals on the only thread");
    let root = skein_shell::open_root(std::path::Path::new(".")).expect("spawn root");
    // One signal source, one child and its input pipe; the pipe keeps the
    // sender live until the parent has consumed its signal (io.md, 7).
    let limits = io::Limits {
        sockets: 3,
        names: 0,
        refusals: 1,
        intake: 8,
        receive: 4,
        output: 8,
        sends: 1,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_secs(1),
        retry: Duration::from_millis(1),
    };
    let mut state = io::Io::new(&limits);
    let source = state.adopt_signals(signals).expect("source entity");
    let mut kernel = Kernel::open_on(Config { operations: io::operations(&limits).expect("operation bound") }, backend)
        .expect("kernel backend");
    let clock = Clock::new();
    let mut patience = std::env::var_os("NEXTEST")
        .map(|_| deadlines::Patience::new("the signal sender service settling", Duration::from_secs(3)));
    let deadline = match &patience {
        Some(patience) => patience.until(),
        None => clock.now().now.saturating_add(Duration::from_secs(3)),
    };
    let (mut submissions, mut completions, mut events) = deadlines::records(RECORDS);
    let now = clock.now();
    let env = Env { now: now.now, wall: now.wall, limits };
    let executable = std::env::current_exe().expect("sender fixture path");
    let parent = std::process::id().to_string();
    let (program, arguments) = if from_child {
        (
            Box::from(executable.as_os_str().as_bytes()),
            vec![Box::from(b"sender".as_slice()), Box::from(parent.as_bytes())].into_boxed_slice(),
        )
    } else {
        (
            Box::from(b"/bin/sh".as_slice()),
            vec![Box::from(b"-c".as_slice()), Box::from(b"read line".as_slice())].into_boxed_slice(),
        )
    };
    io::down(
        &mut state,
        &env,
        Request::Spawn {
            owner: Token::new(100),
            spawn: Spawn {
                root,
                dir: Box::from(b".".as_slice()),
                program,
                args: arguments,
                env: Box::new([]),
                pipes: Box::new([Pipe { child: 0, way: Way::In, parent: None }]),
            },
        },
        &mut submissions,
    );
    // The synchronous spawn completion lands before the first signal read.
    kernel.submit(&mut submissions, Wait::No);
    let mut child = None;
    let mut input = None;
    let mut heard = false;
    state.reclaim();
    loop {
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        while state.is_ready() {
            io::resume(&mut state, &env, &mut events, &mut submissions);
        }
        kernel.reap(&mut completions);
        while let Some(complete) = completions.pop() {
            if let Some(patience) = &mut patience {
                patience.watch_spawn(&complete);
            }
            io::up(&mut state, &env, complete, &mut events, &mut submissions);
        }
        while let Some(event) = events.pop() {
            match event {
                Event::Spawned { child: token, pipes, .. } => {
                    child = Some(token);
                    input = Some(*pipes.first().expect("the child input pipe"));
                    if !from_child {
                        skein_shell::signal_current_thread(ServiceSignal::Terminate).expect("self signal");
                    }
                }
                Event::Shutdown { signal, from, .. } => {
                    assert_eq!(signal, ServiceSignal::Terminate);
                    assert_eq!(from, if from_child { child } else { None });
                    assert!(child.is_some(), "the sender's spawn has completed");
                    assert!(!heard, "one signal");
                    heard = true;
                    io::down(
                        &mut state,
                        &env,
                        Request::Close { entity: child.expect("spawned child") },
                        &mut submissions,
                    );
                    io::down(
                        &mut state,
                        &env,
                        Request::Close { entity: input.expect("the child input pipe") },
                        &mut submissions,
                    );
                    io::down(&mut state, &env, Request::Close { entity: source }, &mut submissions);
                }
                Event::Exited { .. } | Event::Closed { .. } | Event::Stream { .. } => {}
                other @ (Event::Listening { .. }
                | Event::Accepted { .. }
                | Event::Connecting { .. }
                | Event::Connected { .. }
                | Event::Output { .. }
                | Event::Usage { .. }
                | Event::Resized
                | Event::Window { .. }
                | Event::Failed { .. }) => panic!("unexpected signal service event {other:?}"),
            }
        }
        state.reclaim();
        if state.is_empty() {
            assert!(heard, "the service heard its signal");
            assert_eq!(kernel.in_flight(), 0);
            skein_shell::close_keeper_fd(root);
            return;
        }
        if now.now >= deadline {
            if let Some(patience) = &patience {
                match patience.expired() {
                    skein_world::Expired::Starved(starved) => {
                        let mut owned = vec![source];
                        owned.extend(child);
                        owned.extend(input);
                        deadlines::end_io(
                            &mut kernel,
                            &mut state,
                            limits,
                            &mut submissions,
                            &mut completions,
                            &mut events,
                            &owned,
                        );
                        drop(kernel);
                        skein_shell::close_keeper_fd(root);
                        starved.pass();
                    }
                    skein_world::Expired::Failed(figures) => {
                        panic!("signal sender service must settle: child {child:?}, heard {heard}: {figures}");
                    }
                }
            }
            panic!("signal sender service must settle: child {child:?}, heard {heard}");
        }
        let wait = if state.is_ready() || !events.is_empty() { Wait::No } else { Wait::Until(deadline) };
        kernel.submit(&mut submissions, wait);
    }
}

// The starter supplies its own PID. Open before validating the relationship,
// then reject an exited handle so a proc observation cannot validate a replacement.
#[cfg(target_os = "linux")]
fn intended_parent(parent: u32) -> Option<skein_io::kernel::Fd> {
    let descriptor = skein_shell::open_pidfd(parent).ok()?;
    if std::os::unix::process::parent_id() == parent && skein_shell::pidfd_exited(descriptor) == Ok(false) {
        Some(descriptor)
    } else {
        skein_shell::close_keeper_fd(descriptor);
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    skein_fail_stop::install();
}
