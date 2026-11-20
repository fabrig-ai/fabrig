//! The shared listing face's buffer boundary, startup identity, unreaped
//! children (runtime/readiness.md, 5), and blocking flags after Open.

use std::fs;

use skein_io::kernel::{Complete, Done, Entry, Op, OpenHow, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_scratch::Scratch;

use crate::Kernel;
use crate::system::{open_root, status_flags};
fn close(fd: i32) {
    crate::system::close(skein_io::kernel::Fd::new(fd)).expect("close test descriptor");
}
use crate::{Config, Wait};

#[test]
fn the_foreground_is_refused_off_a_terminal() {
    let (reader, writer) = crate::keeper::open_signal_pipe().expect("a pipe, not a terminal");
    let before = crate::posix::terminal::signal_mask();
    let result = crate::take_foreground(reader);
    let after = crate::posix::terminal::signal_mask();
    close(reader.raw());
    close(writer.raw());
    assert_eq!(result, Err(skein_io::kernel::Error::Other(libc::ENOTTY)));
    assert_eq!(after, before, "the calling thread's complete signal mask is restored on refusal");
}

#[test]
fn an_iteration_environment_keeps_the_clocks_and_limits() {
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(29);
    let env = crate::Now { now, wall }.with(7_u32);
    assert_eq!(env.now, now, "the iteration's monotonic time");
    assert_eq!(env.wall, wall, "the iteration's wall time");
    assert_eq!(env.limits, 7, "the step's limits");
}

#[test]
fn a_fixed_clock_starts_at_the_supplied_times() {
    let start = crate::FixedStart { wall: Wall::from_nanos(1_767_690_000_000_000_000), seed: 7 };
    let clock = crate::Clock::fixed(&start);
    assert_eq!(clock.now(), crate::Now { now: crate::FixedStart::NOW, wall: start.wall });
}

#[test]
fn the_host_clock_passes_deadlines_through() {
    let clock = crate::Clock::new();
    let deadline = clock.now().now;
    assert_eq!(clock.until(Some(deadline)), Some(deadline));
    assert_eq!(clock.until(None), None);
}

#[test]
fn startup_host_name_is_the_kernels_bytes_without_terminators() {
    let hostname = crate::host_name().expect("kernel host name");
    assert!(!hostname.contains(&0), "the NUL terminator is not part of the name");
    assert!(!hostname.contains(&b'\n'), "the name has no line feed");
    #[cfg(target_os = "linux")]
    {
        let proc = fs::read("/proc/sys/kernel/hostname").expect("Linux host name");
        assert_eq!(hostname.as_ref(), proc.strip_suffix(b"\n").expect("proc's final line feed"));
    }
}

#[test]
fn startup_process_id_is_this_process() {
    assert_eq!(crate::process_id(), std::process::id());
}

#[test]
#[expect(clippy::disallowed_types, reason = "a test child isolates the unreaped-child observation from io")]
#[expect(unsafe_code, reason = "waitid with WNOWAIT observes this test child's exit without reaping it")]
fn a_child_is_listed_until_its_wait_and_not_after() {
    let mut child = std::process::Command::new("/bin/sh")
        .args(["-c", "exit 0"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn the test child");
    let pid = child.id();
    assert!(crate::unreaped_children().expect("child observation").contains(&pid), "the child is not yet reaped");

    // SAFETY: zero initializes every field of the C siginfo output storage.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: pid names this test's unreaped child and info is writable
        // siginfo storage. WNOWAIT retains the zombie for the observation.
        let result = unsafe { libc::waitid(libc::P_PID, pid, &raw mut info, libc::WEXITED | libc::WNOWAIT) };
        if result == 0 {
            break;
        }
        assert_eq!(crate::system::last_errno(), libc::EINTR, "observing the child's exit can only be interrupted");
    }
    assert!(crate::unreaped_children().expect("zombie observation").contains(&pid), "the zombie awaits its wait");
    assert!(child.wait().expect("reap the test child").success(), "the child exited successfully");
    assert!(!crate::unreaped_children().expect("after wait observation").contains(&pid), "the child was reaped");
}

#[test]
fn a_name_that_fits_in_no_names_is_too_long_and_stays_next() {
    let scratch = Scratch::new("shell");
    fs::write(scratch.path().join("a-name-of-twenty-bytes"), b"").expect("a file laid");
    let dir = open_root(scratch.path()).expect("a scratch directory opens");
    let mut listing = crate::system::Listings::new();
    let mut entries = [Entry::BLANK; 2];
    let mut small = [0_u8; 8];
    assert_eq!(listing.list(dir, &mut entries, &mut small), Err(libc::ENAMETOOLONG));
    let mut names = [0_u8; 255];
    assert_eq!(listing.list(dir, &mut entries, &mut names), Ok(1), "the name is still next");
    let first = entries.first().expect("an entry").name(&names);
    assert_eq!(first, Some(&b"a-name-of-twenty-bytes"[..]));
    assert_eq!(listing.list(dir, &mut entries, &mut names), Ok(0), "then the end");
    listing.forget(dir);
    close(dir.raw());
}

/// Every `Open` goes down without blocking, so that a FIFO opens to be
/// refused; a file it opened is blocking again, so that io_uring sends a
/// read it cannot do at once to its worker (runtime/kernel.md, 6.1).
#[test]
fn a_file_opened_blocks_again() {
    let scratch = Scratch::new("shell");
    fs::write(scratch.path().join("f"), b"f").expect("a file laid");
    let dir = open_root(scratch.path()).expect("a scratch directory opens");
    let mut kernel = Kernel::open(Config { operations: 4 }).expect("the kernel is usable here");
    let mut submissions = Queue::with_capacity(1);
    let path = Box::from(&b"f"[..]);
    submissions.push(Submit { op: Token::new(1), kind: Op::Open { root: dir, path, how: OpenHow::Read } });
    kernel.submit(&mut submissions, Wait::Forever);
    let mut completions = Queue::with_capacity(1);
    kernel.reap(&mut completions);
    let Some(Complete { result: Ok(Done::Fd(file)), .. }) = completions.pop() else { panic!("the file opened") };
    let flags = status_flags(file.raw()).expect("an open descriptor's flags");
    assert_eq!(flags & libc::O_NONBLOCK, 0, "the file blocks");
    close(file.raw());
    close(dir.raw());
}

#[test]
fn make_root_makes_and_opens_a_nested_absent_path() {
    let scratch = Scratch::new("make-root");
    let path = scratch.path().join("state/private");
    let fd = crate::make_root(&path, 0o700).expect("nested root made and opened");
    assert!(path.is_dir(), "the last directory exists");
    assert_eq!(open_root(&path).map(|opened| close(opened.raw())), Ok(()), "opens as a root");
    close(fd.raw());
}

#[test]
fn make_root_refuses_a_file_in_its_path() {
    let scratch = Scratch::new("blocked-root");
    let file = scratch.path().join("file");
    fs::write(&file, b"contents").expect("blocking file");
    for (label, path) in [("final file", file.clone()), ("parent file", file.join("private"))] {
        assert_eq!(crate::make_root(&path, 0o700), Err(libc::ENOTDIR), "{label}");
    }
    assert_eq!(fs::read(file).expect("blocking file remains"), b"contents", "file contents retained");
}

#[test]
fn make_root_preserves_an_existing_directorys_mode() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("existing-root");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o711)).expect("existing mode");
    let fd = crate::make_root(scratch.path(), 0o700).expect("existing root opened");
    close(fd.raw());
    assert_eq!(
        fs::metadata(scratch.path()).expect("existing directory").permissions().mode() & 0o777,
        0o711,
        "existing mode retained"
    );
}
