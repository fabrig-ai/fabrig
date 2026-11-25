mod tests {
    use crate::system;
    #[cfg(target_os = "linux")]
    use skein_io::kernel::Error;
    use skein_io::kernel::{Done, Fd, Op};
    use skein_lib::{Duration, Queue};
    use skein_shell::testing::readiness::{Readiness, submit};
    use skein_shell::{Clock, Config, Wait};

    #[test]
    fn an_inherited_blocking_pipe_waits_before_reading() {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        let (reader, mut writer) = std::io::pipe().expect("a blocking test pipe");
        let fd = Fd::new(reader.as_raw_fd());
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(&mut kernel, 1, Op::PipeRead { fd, buf: vec![0; 8].into_boxed_slice() });
        assert_eq!(kernel.read_waiter(fd).expect("read interest registered"), Some(0));
        assert!(kernel.ready_is_empty(), "an empty blocking pipe is not called at submit");
        assert!(!kernel.is_nonblocking(fd), "an inherited pipe keeps its flags");
        writer.write_all(b"pipe").expect("four bytes fit in an empty pipe");
        kernel.submit(&mut Queue::with_capacity(0), Wait::Forever);
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let complete = out.pop().expect("one pipe read completes");
        assert_eq!(complete.result, Ok(Done::Count(4)));
        let Op::PipeRead { buf, .. } = complete.kind else { unreachable!("the read record comes back") };
        assert_eq!(buf.get(..4), Some(b"pipe".as_slice()));
        assert!(kernel.waiters_are_empty(), "a completed read removes its interest");
    }

    #[test]
    fn an_inherited_blocking_pipe_write_is_capped_at_pipe_buf() {
        use std::io::Read;
        use std::os::fd::AsRawFd;
        let (mut reader, writer) = std::io::pipe().expect("a blocking test pipe");
        let maximum = libc::PIPE_BUF;
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(
            &mut kernel,
            1,
            Op::PipeWrite {
                fd: Fd::new(writer.as_raw_fd()),
                bytes: vec![7; maximum.checked_mul(2).expect("two PIPE_BUFs fit")].into_boxed_slice(),
                from: 0,
            },
        );
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let complete = out.pop().expect("the empty pipe is writable");
        assert_eq!(complete.result, Ok(Done::Count(u32::try_from(maximum).expect("PIPE_BUF fits u32"))));
        let mut buf = vec![0; maximum];
        reader.read_exact(&mut buf).expect("the one write put PIPE_BUF bytes in the pipe");
        assert!(buf.iter().all(|byte| *byte == 7));
        assert!(kernel.waiters_are_empty());
    }

    #[test]
    #[expect(
        clippy::disallowed_types,
        reason = "the inherited descriptor in this test is a std File (runtime/shell.md, 8)"
    )]
    fn a_regular_file_pipe_read_completes_without_interest() {
        use std::io::{Seek, SeekFrom, Write};
        use std::os::fd::AsRawFd;
        let scratch = skein_scratch::Scratch::new("readiness");
        let path = scratch.path().join("file");
        let mut file =
            std::fs::File::options().create_new(true).read(true).write(true).open(&path).expect("a scratch file");
        std::fs::remove_file(path).expect("the open scratch file needs no name");
        file.write_all(b"file").expect("four scratch bytes");
        file.seek(SeekFrom::Start(0)).expect("read from the beginning");
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(&mut kernel, 1, Op::PipeRead { fd: Fd::new(file.as_raw_fd()), buf: vec![0; 8].into_boxed_slice() });
        assert!(kernel.waiters_are_empty(), "the completed file read keeps no interest");
        assert!(kernel.nonblocking_is_empty(), "a file is not a backend stream");
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let complete = out.pop().expect("a file is always ready");
        assert_eq!(complete.result, Ok(Done::Count(4)));
        let Op::PipeRead { buf, .. } = complete.kind else { unreachable!("the read record comes back") };
        assert_eq!(buf.get(..4), Some(b"file".as_slice()));

        submit(&mut kernel, 2, Op::PipeRead { fd: Fd::new(file.as_raw_fd()), buf });
        assert!(kernel.waiters_are_empty(), "a file at EOF keeps no interest either");
        kernel.reap(&mut out);
        let complete = out.pop().expect("EOF completes at the next reap, without waiting");
        assert_eq!(complete.result, Ok(Done::Count(0)), "a file at its end is always ready");
    }

    #[test]
    fn a_signal_source_waits_then_reports_its_signal() {
        use skein_io::kernel::{ServiceSignal, Signalled};
        let fd = system::process::open_signals(crate::Watch::Termination).expect("open the termination signal source");
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(&mut kernel, 1, Op::ReadSignal { fd });
        assert_eq!(kernel.read_waiter(fd).expect("signal source read interest"), Some(0));
        assert!(kernel.ready_is_empty(), "a signal source waits before its first call");
        crate::keeper::signal_current_thread(ServiceSignal::Interrupt).expect("queue SIGINT on this thread");
        kernel.submit(&mut Queue::with_capacity(0), Wait::Forever);
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let sender = if cfg!(target_os = "linux") {
            Some(skein_io::kernel::Sender::new(u64::from(std::process::id())))
        } else {
            None
        };
        assert_eq!(
            out.pop().expect("one signal read").result,
            Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Interrupt, sender }))
        );
        assert!(kernel.waiters_are_empty());
        submit(&mut kernel, 2, Op::Close { fd });
        kernel.reap(&mut out);
        assert_eq!(out.pop().expect("the signal source closes").result, Ok(Done::Nothing));
    }

    #[test]
    fn a_hangup_source_reports_hangup() {
        use skein_io::kernel::{ServiceSignal, Signalled};
        let fd = system::process::open_signals(crate::Watch::TerminationAndHangup).expect("open hangup signals");
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(&mut kernel, 1, Op::ReadSignal { fd });
        assert!(kernel.ready_is_empty(), "no signal before delivery");
        crate::keeper::signal_current_thread(ServiceSignal::Hangup).expect("queue SIGHUP on this thread");
        kernel.submit(&mut Queue::with_capacity(0), Wait::Forever);
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let sender = if cfg!(target_os = "linux") {
            Some(skein_io::kernel::Sender::new(u64::from(std::process::id())))
        } else {
            None
        };
        assert_eq!(
            out.pop().expect("one hangup read").result,
            Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Hangup, sender })),
            "the requested hangup retains its name and sender"
        );
        submit(&mut kernel, 2, Op::Close { fd });
        kernel.reap(&mut out);
        assert_eq!(out.pop().expect("signal source closes").result, Ok(Done::Nothing), "source cleanup");
    }

    #[test]
    #[expect(
        clippy::disallowed_types,
        reason = "the isolated test inherits SIG_IGN from its shell (runtime/shell-startup.md, 1.3)"
    )]
    #[expect(
        clippy::disallowed_methods,
        reason = "skein's round: reads the child marker for the isolated ignored-hangup test"
    )]
    fn a_hangup_ignored_at_startup_is_not_watched() {
        use skein_io::kernel::{ServiceSignal, Signalled};
        // Re-exec this one test with the shell's ignored disposition; no unsafe
        // signal-disposition changes escape the system modules (runtime/readiness.md, 5).
        const CHILD: &str = "SKEIN_TEST_IGNORED_HANGUP";
        if std::env::var_os(CHILD).is_none() {
            let status = std::process::Command::new("sh")
                .args(["-c", r#"trap '' HUP; exec "$0" "$@""#])
                .arg(std::env::current_exe().expect("test executable"))
                .args(["--exact", "readiness::tests::a_hangup_ignored_at_startup_is_not_watched", "--nocapture"])
                .env(CHILD, "1")
                .status()
                .expect("isolated ignored-hangup test");
            assert!(status.success(), "an ignored hangup stays ignored");
            return;
        }
        let fd = system::process::open_signals(crate::Watch::TerminationAndHangup).expect("open hangup signals");
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        submit(&mut kernel, 1, Op::ReadSignal { fd });
        crate::keeper::signal_current_thread(ServiceSignal::Hangup).expect("send ignored SIGHUP");
        kernel.submit(&mut Queue::with_capacity(0), Wait::No);
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        assert!(out.is_empty(), "startup-ignored SIGHUP is absent from the signal source");
        crate::keeper::signal_current_thread(ServiceSignal::Interrupt).expect("send watched SIGINT");
        kernel.submit(&mut Queue::with_capacity(0), Wait::Forever);
        kernel.reap(&mut out);
        assert!(
            matches!(
                out.pop().expect("interrupt read").result,
                Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Interrupt, .. }))
            ),
            "ignoring hangup still watches termination"
        );
        submit(&mut kernel, 2, Op::Close { fd });
        kernel.reap(&mut out);
        assert_eq!(out.pop().expect("signal source closes").result, Ok(Done::Nothing), "source cleanup");
    }

    // EPOLL_CTL_ADD refuses duplicate interest; kevent EV_ADD modifies it.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_registration_error_completes_without_calling_or_adding_a_waiter() {
        use std::os::fd::AsRawFd;
        let (reader, _writer) = std::io::pipe().expect("a blocking test pipe");
        let fd = Fd::new(reader.as_raw_fd());
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        kernel.read_interest(fd, false, true).expect("pre-register the descriptor to provoke EEXIST");
        submit(&mut kernel, 1, Op::PipeRead { fd, buf: vec![0; 8].into_boxed_slice() });
        assert!(kernel.waiters_are_empty(), "failed registration preserves the waiter map");
        let mut out = Queue::with_capacity(1);
        kernel.reap(&mut out);
        let complete = out.pop().expect("the registration error completes the record");
        assert_eq!(complete.result, Err(Error::Other(libc::EEXIST)));
        let Op::PipeRead { buf, .. } = complete.kind else { unreachable!("the record comes back") };
        assert_eq!(&*buf, &[0; 8], "a registration error makes no read");
        kernel.read_interest(fd, true, false).expect("remove the test's injected interest");
    }

    #[test]
    fn an_idle_deadline_does_not_wake_early() {
        let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
        let clock = Clock::new();
        for _ in 0..4 {
            let deadline = clock.now().now.saturating_add(Duration::from_millis(2));
            kernel.submit(&mut Queue::with_capacity(0), Wait::Until(deadline));
            assert!(clock.now().now >= deadline, "the poller must not wake before the deadline");
        }
    }
}
