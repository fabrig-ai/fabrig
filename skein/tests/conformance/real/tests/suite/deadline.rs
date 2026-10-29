//! Must-complete waits on the real clock (testing-strategy.md, 2.10).

use std::net::{Ipv4Addr, TcpListener};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;

use skein_conformance::Backend as _;
use skein_conformance_real::Real;
use skein_io::kernel::{Complete, Done, Fd, Op, Spawn, Submit};
use skein_lib::{Duration, Queue, Token};
use skein_shell::Backend;

/// The task's idle unanswered wait is twenty milliseconds.
const PATIENCE: Duration = Duration::from_millis(20);

fn idle_listener() -> (TcpListener, Real, usize) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a loopback listener");
    let mut real = Real::new(Backend::Readiness);
    let process = real.open();
    (listener, real, process)
}

#[test]
fn an_idle_unanswered_operation_fails_with_its_figures() {
    let (listener, mut real, process) = idle_listener();
    let token = Token::new(1);
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: token, kind: Op::Accept { fd: Fd::new(listener.as_raw_fd()) } });
    real.submit(process, &mut submissions);
    real.wait_begins("the idle conformance accept", PATIENCE);
    // An accept without a peer cannot answer; sleep is idle, not runnable wait.
    real.sleep(PATIENCE);
    real.wait_unanswered();
    let figures = real.wait_figures().expect("an idle unanswered wait has scheduler figures");
    let failure = std::panic::catch_unwind(|| panic!("no completion: {figures}"))
        .expect_err("an idle unanswered operation fails");
    let text = failure.downcast_ref::<String>().expect("failure includes its figures");
    assert!(text.starts_with("no completion: the idle conformance accept: patience 20000000 ns; elapsed "), "{text}");
    assert!(text.contains("thread runnable wait ") && text.contains("thread CPU "), "{text}");
    assert!(text.contains("tree runnable wait "), "{text}");
}

#[test]
fn an_idle_listener_wait_fails_with_its_figures() {
    let (listener, mut real, process) = idle_listener();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        real.wait_for_connection(process, Fd::new(listener.as_raw_fd()), PATIENCE);
    }))
    .expect_err("a listener without a connection fails");
    let text = failure.downcast_ref::<String>().expect("failure includes its figures");
    assert!(text.starts_with("no connection arrived on listener "), "{text}");
    assert!(text.contains("the conformance listener's queued connection: patience 20000000 ns; elapsed "), "{text}");
    assert!(text.contains("thread runnable wait ") && text.contains("thread CPU "), "{text}");
    assert!(text.contains("tree runnable wait "), "{text}");
}

fn completed(real: &mut Real, process: usize, token: Token, op: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: token, kind: op });
    real.submit(process, &mut submissions);
    let mut completions = Queue::with_capacity(1);
    // Spawn completes synchronously; the ring's enter delivers its deferred work.
    real.enter(process);
    real.reap(process, &mut completions);
    completions.pop().expect("the synchronous operation completed")
}

#[test]
fn cleanup_drains_an_accept_and_reaps_its_child() {
    let mut backends = vec![Backend::Readiness];
    #[cfg(target_os = "linux")]
    backends.push(Backend::Ring);
    for backend in backends {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a loopback listener");
        let mut real = Real::new(backend);
        let process = real.open();
        let root = real.root(process, &[]);
        let spawn = Spawn {
            program: Box::from(skein_scratch::paths::binary("process_fixture").as_os_str().as_bytes()),
            args: Box::new([Box::from(&b"never"[..])]),
            env: Box::new([]),
            root,
            dir: Box::from(&b"."[..]),
            pipes: Box::new([]),
        };
        let complete = completed(&mut real, process, Token::new(1), Op::Spawn { spawn: Box::new(spawn) });
        let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("the waiting child spawned") };
        #[cfg(target_os = "linux")]
        let keeper = skein_shell::open_child_pidfd(
            skein_shell::child_id(pidfd).expect("the owned child's PID"),
            std::process::id(),
        )
        .expect("retain the child's identity while checking its reap");
        let mut submissions = Queue::with_capacity(1);
        submissions.push(Submit { op: Token::new(2), kind: Op::Accept { fd: Fd::new(listener.as_raw_fd()) } });
        real.submit(process, &mut submissions);
        drop(real);
        #[cfg(target_os = "linux")]
        {
            let reaped = skein_shell::poll_child(keeper, false);
            skein_shell::close_keeper_fd(keeper);
            assert_eq!(reaped, Err(skein_io::kernel::Error::Other(libc::ECHILD)), "cleanup reaps its owned child");
        }
        assert_eq!(
            skein_shell::poll_child(pidfd, false),
            Err(skein_io::kernel::Error::Other(libc::EBADF)),
            "cleanup reaps the child and closes its identity descriptor"
        );
        assert_eq!(
            skein_shell::wait_readable(root, Duration::ZERO),
            Err(libc::EBADF),
            "cleanup closes its root after draining the kernel"
        );
    }
}
