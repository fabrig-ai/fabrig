#[expect(unsafe_code, reason = "waitid observes this fixture's child without reaping it")]
mod tests {
    use skein_io::kernel::Signal;
    use skein_shell::keeper::{process_live, signal_group as system_signal_group};
    use std::os::unix::process::CommandExt;

    #[test]
    fn a_group_of_one_zombie_accepts_a_kill() {
        #[expect(
            clippy::disallowed_types,
            reason = "the group signal test retains a std child as a zombie independently of shell handles"
        )]
        let mut child = std::process::Command::new("/bin/sh")
            .args(["-c", "exit 0"])
            .process_group(0)
            .spawn()
            .expect("child starts in its own group");
        let pid = child.id();
        // SAFETY: siginfo_t is C output storage; zero is a valid initial representation.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        loop {
            // SAFETY: info is writable for waitid; WNOWAIT retains this owned child.
            let rc =
                unsafe { libc::waitid(libc::P_PID, pid, std::ptr::from_mut(&mut info), libc::WEXITED | libc::WNOWAIT) };
            if rc == 0 {
                break;
            }
            assert_eq!(crate::system::last_errno(), libc::EINTR, "waitid observes the child's exit without reaping");
        }
        let result = system_signal_group(pid, Signal::Kill);
        child.wait().expect("the zombie is reaped after its group signal");
        assert_eq!(result, Ok(()), "a group containing only its zombie leader accepts a kill");
    }

    #[test]
    fn the_running_process_is_live() {
        assert_eq!(process_live(std::process::id()), Ok(true), "the caller exists");
    }

    #[test]
    fn a_reaped_child_is_not_live() {
        #[expect(
            clippy::disallowed_types,
            reason = "the liveness test reaps a std child independently of the shell's process handles"
        )]
        let mut child = std::process::Command::new("/bin/sh").args(["-c", "exit 0"]).spawn().expect("child starts");
        let pid = child.id();
        child.wait().expect("child is reaped");
        assert_eq!(process_live(pid), Ok(false), "a reaped child no longer exists");
    }
}
