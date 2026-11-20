//! Linux process-boundary tests of the shared watcher over pidfds. A
//! spawning helper bounds std's synchronous
//! exec-error-pipe read. The test thread owns process waits and groups.
//! The hook publishes its pid with a lock-free atomic before starting the
//! watcher; cancellation either kills that group or refuses the hook.

#![expect(
    clippy::disallowed_types,
    reason = "process-boundary tests own std children, a real deadline and a spawn cancellation cell"
)]

use super::{close_bound, parent_error, watch_parent};
use crate::posix::group::{group_live, signal_group};
use skein_io::kernel::Signal;
use std::{
    io::{ErrorKind, Read},
    os::{
        fd::AsRawFd,
        unix::process::{CommandExt, ExitStatusExt},
    },
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const PATIENCE: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(5);

// Linux must reap its adopted watchers: a container's pid 1 need not do so.
// nextest runs each unit test in its own process, so this changes no neighbour.
struct Adoption {
    previous: bool,
}
impl Adoption {
    fn new() -> Self {
        let previous = crate::process::subreaper().expect("read orphan adoption");
        crate::process::set_subreaper(true).expect("adopt the test's orphaned watchers");
        Self { previous }
    }
}
impl Drop for Adoption {
    fn drop(&mut self) {
        crate::process::set_subreaper(self.previous).expect("restore orphan adoption");
    }
}

struct Program {
    child: Child,
    group: u32,
    release: Option<mpsc::Sender<()>>,
}
impl Program {
    fn parent() -> Self {
        let mut command = Command::new("/bin/sleep");
        command.arg("30").process_group(0);
        let child = crate::spawn_dying_with_parent(&mut command, Signal::Kill).expect("stand-in parent starts");
        Self { group: child.id(), child, release: None }
    }

    fn wait(&mut self, what: &str) -> ExitStatus {
        let deadline = Instant::now().checked_add(PATIENCE).expect("patience fits clock");
        loop {
            if let Some(status) = self.child.try_wait().expect("poll owned child") {
                return status;
            }
            if Instant::now() >= deadline {
                let _ended = end_group(self.group);
                panic!("timed out waiting for {what}");
            }
            std::thread::sleep(TICK);
        }
    }

    fn empty(&self) {
        let deadline = Instant::now().checked_add(PATIENCE).expect("patience fits clock");
        loop {
            reap_group(self.group);
            if !group_live(self.group).expect("probe the program's group") {
                return;
            }
            if Instant::now() >= deadline {
                let _ended = end_group(self.group);
                panic!("timed out waiting for the watcher's group to empty");
            }
            std::thread::sleep(TICK);
        }
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        let empty = end_group(self.group);
        assert!(empty || std::thread::panicking(), "timed out cleaning the program and watcher group {}", self.group);
        if let Some(release) = self.release.take() {
            let _released = release.send(());
        }
    }
}

fn end_group(group: u32) -> bool {
    let _killed = signal_group(group, Signal::Kill);
    let deadline = Instant::now().checked_add(PATIENCE).expect("patience fits clock");
    loop {
        reap_group(group);
        if group_live(group) == Ok(false) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(TICK);
    }
}

fn reap_group(group: u32) {
    let group = i32::try_from(group).expect("group fits pid_t").checked_neg().expect("positive group");
    // At most the program and its watcher, with room for interrupted probes.
    for _probe in 0..4 {
        // SAFETY: a negative, specific group selects only children belonging
        // to this fixture. WNOHANG never waits, and no status storage is used.
        let result = unsafe { libc::waitpid(group, std::ptr::null_mut(), libc::WNOHANG) };
        if result > 0 || (result < 0 && crate::system::last_errno() == libc::EINTR) {
            continue;
        }
        break;
    }
}

fn spawn(mut command: Command, parent: u32) -> std::io::Result<Program> {
    let parent = i32::try_from(parent).expect("parent fits pid_t");
    let until = close_bound().expect("descriptor close bound");
    let number = crate::posix::process::signal_number(Signal::Kill);
    let pid = Arc::new(AtomicI32::new(0));
    let published = Arc::clone(&pid);
    let owner = i32::try_from(std::process::id()).expect("owner fits pid_t");
    command.process_group(0);
    // SAFETY: the hook uses only lock-free integer atomics and the watcher's
    // async-signal-safe path, returning an allocation-free errno error. Arc
    // ownership is prepared before fork and never cloned or dropped in it.
    let before_exec = move || {
        // This is the fixture's real owner, independent of stand-in S.
        // Keep the spawning thread alive until teardown, so even a test
        // killed without unwinding leaves no program running.
        // SAFETY: prctl takes only integers and is async-signal-safe.
        if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: getppid is async-signal-safe and takes no arguments.
        if unsafe { libc::getppid() } != owner {
            return Err(std::io::Error::from_raw_os_error(libc::ESRCH));
        }
        // SAFETY: getpid takes no arguments and is async-signal-safe.
        let program = unsafe { libc::getpid() };
        if published.compare_exchange(0, program, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(std::io::Error::from_raw_os_error(libc::ECANCELED));
        }
        // S is deliberately not C's parent: exercise the watch alone,
        // not the caller's getppid re-check.
        watch_parent(parent, number, until).map_err(std::io::Error::from_raw_os_error)
    };
    // SAFETY: the hook follows the allocation-free process-boundary path described above.
    unsafe { command.pre_exec(before_exec) };
    let (sender, receiver) = mpsc::channel();
    let (release, released) = mpsc::channel();
    #[expect(
        clippy::disallowed_methods,
        reason = "bound std's synchronous spawn without letting a broken watcher hang the test"
    )]
    let worker = std::thread::spawn(move || {
        let result = command.spawn().map(|child| Program { group: child.id(), child, release: Some(release) });
        let started = result.is_ok();
        // A cancelled receiver drops the owned fixture, killing its group.
        let _sent = sender.send(result);
        if started {
            // A bounded backstop even if the test abandons its ownership.
            released
                .recv_timeout(Duration::from_secs(30))
                .expect("timed out waiting for fixture teardown to release the spawning thread");
        }
    });
    let result = receiver.recv_timeout(PATIENCE);
    if result.is_err() {
        let program = pid.swap(-1, Ordering::SeqCst);
        if let Ok(group) = u32::try_from(program)
            && group > 1
        {
            assert!(end_group(group), "timed out cleaning a spawn whose exec-error pipe stayed open");
        }
        panic!("timed out waiting for spawn's exec-error pipe to close");
    }
    // Program's teardown releases this worker; don't add an unbounded join.
    drop(worker);
    result.expect("spawn answered before its bound").map_err(|error| error.raw_os_error().map_or(error, parent_error))
}

#[test]
fn a_program_dies_when_the_watched_parent_exits() {
    let _adoption = Adoption::new();
    let mut parent = Program::parent();
    let mut command = Command::new("/bin/sleep");
    command.arg("30");
    let mut program = spawn(command, parent.group).expect("watcher acknowledges both processes");
    parent.child.kill().expect("end stand-in parent");
    parent.wait("stand-in parent to be reaped");
    assert_eq!(program.wait("program's parent-death signal").signal(), Some(9), "watcher sends SIGKILL");
    program.empty();
}

#[test]
fn a_watcher_ends_with_its_program_without_signalling_the_parent() {
    let _adoption = Adoption::new();
    let mut parent = Program::parent();
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "exit 3"]);
    let mut program = spawn(command, parent.group).expect("watcher acknowledges both processes");
    assert_eq!(program.wait("program's own exit").code(), Some(3), "program keeps its own exit status");
    program.empty();
    assert!(parent.child.try_wait().expect("probe stand-in parent").is_none(), "watcher never signals the parent");
}

#[test]
fn a_watcher_holds_neither_the_exec_error_pipe_nor_stdout() {
    let _adoption = Adoption::new();
    let parent = Program::parent();
    // A live program proves spawn returns before the watcher ends.
    let mut command = Command::new("/bin/sleep");
    command.arg("30");
    let mut live = spawn(command, parent.group).expect("spawn returns while the watcher lives");
    assert!(live.child.try_wait().expect("probe live program").is_none(), "spawn did not wait for program exit");

    let mut command = Command::new("/bin/sh");
    command.args(["-c", "echo ready"]).stdout(Stdio::piped());
    let mut program = spawn(command, parent.group).expect("piped program starts");
    let mut output = program.child.stdout.take().expect("piped stdout");
    crate::posix::nonblocking(skein_io::kernel::Fd::new(output.as_raw_fd())).expect("nonblocking output");
    let deadline = Instant::now().checked_add(PATIENCE).expect("patience fits clock");
    let mut bytes = Vec::new();
    loop {
        let mut room = [0; 32];
        match output.read(&mut room) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(room.get(..count).expect("read fits buffer")),
            Err(error) if error.kind() == ErrorKind::WouldBlock || error.kind() == ErrorKind::Interrupted => {}
            Err(error) => panic!("stdout read: {error}"),
        }
        assert!(Instant::now() < deadline, "timed out waiting for piped stdout's end");
        std::thread::sleep(TICK);
    }
    assert_eq!(bytes, b"ready\n", "stdout reaches EOF after the program's output");
    assert_eq!(program.wait("piped program's exit").code(), Some(0), "program succeeds");
    program.empty();
}

#[test]
fn a_parent_already_gone_refuses_before_exec() {
    let _adoption = Adoption::new();
    let mut parent = Program::parent();
    parent.child.kill().expect("end stand-in parent");
    parent.wait("stand-in parent to be reaped before registration");
    let scratch = skein_scratch::Scratch::new("parent-death");
    let marker = scratch.path().join("ran");
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "echo ran > \"$1\"", "marker"]).arg(&marker);
    let error = match spawn(command, parent.group) {
        Ok(_program) => panic!("a missing parent must refuse before exec"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), ErrorKind::NotFound, "a gone parent is NotFound");
    assert!(error.to_string().contains("parent changed before exec"), "refusal names the changed parent: {error}");
    assert!(!marker.exists(), "refused program never ran");
}
