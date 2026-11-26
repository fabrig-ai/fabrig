//! Keeper calls against a descendant outside the test's direct wait ownership
//! (skein/docs/design/runtime/readiness.md, 5).
//!
//! The stop wait names the test's own child; it passes no borrowed memory
//! when status is null and cannot reap the child while it is stopping.

use super::{close_keeper_fd, open_child_pidfd, pidfd_exited, process_children, signal_kept_child, wait_readable};
use skein_io::kernel::{Error, Fd, Signal, Target};
use skein_lib::Duration;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

const PATIENCE: Duration = Duration::from_secs(2);
const GONE: Error = Error::Other(libc::ESRCH);

struct Shell(Child);

impl Drop for Shell {
    fn drop(&mut self) {
        // The shell normally waits for its child after CONT. On failure
        // let it do so as well; the sleep is finite even if no kill reached it.
        drop(self.0.stdin.take());
        if let Ok(handle) = open_child_pidfd(self.0.id(), std::process::id()) {
            let _continued = signal_kept_child(handle, Signal::Continue, Target::Child);
            close_keeper_fd(handle);
        }
        let _reaped = self.0.wait();
    }
}

struct Watch(Fd);

impl Drop for Watch {
    fn drop(&mut self) {
        close_keeper_fd(self.0);
    }
}

// Keep the descendant unreaped through the first exit poll. A Linux pidfd
// can report HUP (rather than just readability) after its owner reaps it.
#[expect(unsafe_code, reason = "waitpid observes a stop of this test's child without borrowing memory")]
fn wait_for_stop(pid: u32) {
    let pid = libc::pid_t::try_from(pid).expect("a shell pid fits pid_t");
    // SAFETY: this test alone waits on the shell, which stops itself before waiting for its child.
    let observed = unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WUNTRACED) };
    assert_eq!(observed, pid, "the shell stops itself");
}

#[test]
fn the_keeper_binds_and_signals_only_the_named_child() {
    #[cfg(target_os = "linux")]
    let background = "/usr/bin/setsid"; // dash cannot enable job control without a tty.
    #[cfg(target_os = "macos")]
    let background = "";
    let mut shell = Shell(
        Command::new("/bin/sh")
            .args([
                "-c",
                r#"if [ -z "$1" ]; then set -m; fi
$1 /bin/sleep 10 &
printf '%s\n' "$!"
kill -STOP "$$"
wait "$!" 2>/dev/null
printf 'reaped\n'
read -r end
exit 0"#,
                "keeper-test",
                background,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start shell with a background child"),
    );
    let parent = shell.0.id();
    let mut line = String::new();
    let mut stdout = BufReader::new(shell.0.stdout.take().expect("child pid pipe"));
    stdout.read_line(&mut line).expect("read child pid");
    let child = line.trim().parse().expect("shell reports its child pid");
    wait_for_stop(parent);
    let own = std::process::id();
    let parent_watch = Watch(open_child_pidfd(parent, own).expect("watch the stopped shell"));
    let listed = process_children(parent).expect("list the shell's children");
    let watch = Watch(open_child_pidfd(child, parent).expect("bind the actual child"));
    let exit_before_kill = pidfd_exited(watch.0);
    let wrong_parent = open_child_pidfd(child, own).map(Watch);
    let init_parent = open_child_pidfd(child, 1).map(Watch);
    let init_child = open_child_pidfd(1, parent).map(Watch);
    #[cfg(target_os = "macos")]
    {
        assert!(super::poll_child(watch.0, true).is_err(), "the keeper does not reap a kept process");
        assert_eq!(signal_kept_child(watch.0, Signal::Kill, Target::Group), Err(Error::InvalidArgument));
    }
    assert_eq!(signal_kept_child(watch.0, Signal::Kill, Target::Child), Ok(()));
    assert_eq!(wait_readable(watch.0, PATIENCE), Ok(true), "the descendant's exit is readable before its reap");
    assert_eq!(pidfd_exited(watch.0), Ok(true), "the exit remains posted");
    signal_kept_child(parent_watch.0, Signal::Continue, Target::Child).expect("let the shell reap its child");
    line.clear();
    stdout.read_line(&mut line).expect("read shell's reap report");
    assert_eq!(line, "reaped\n");
    assert_eq!(signal_kept_child(watch.0, Signal::Kill, Target::Child), Err(GONE));
    assert_eq!(open_child_pidfd(child, parent).map(Watch).err(), Some(GONE));
    assert_eq!(listed.as_ref(), &[child]);
    assert_eq!(exit_before_kill, Ok(false));
    assert_eq!(wrong_parent.err(), Some(Error::NotFound));
    assert_eq!(init_parent.err(), Some(Error::InvalidArgument));
    assert_eq!(init_child.err(), Some(Error::InvalidArgument));
}
