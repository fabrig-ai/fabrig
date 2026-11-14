//! A supervised interrupt is handled once while the tree remains alive.

use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::Clock;
use skein_world::end_to_end::{Binary, Cleanup, Command, Expectation, Mode};
use skein_world::{Referee, real};

struct Judge {
    sent: bool,
    handled_at: Option<Time>,
    checked: bool,
    ready_to_kill: bool,
    next: Option<Time>,
}

impl Referee<Binary> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Binary]) {
        let child = &mut procs[0];
        if !self.sent && child.stderr() == b"ready\n" {
            child.signal_tree(Signal::Interrupt);
            self.sent = true;
        }
        if self.sent && self.handled_at.is_none() && child.stderr().ends_with(b"interrupted\n") {
            self.handled_at = Some(now);
        }
        if !self.checked && self.ready_to_kill {
            child.signal_tree(Signal::Kill);
            self.checked = true;
        }
    }

    fn watch(&mut self, now: Time, procs: &[Binary]) -> Result<(), String> {
        let child = &procs[0];
        if !self.checked && self.handled_at.is_some_and(|at| now >= at.saturating_add(Duration::from_millis(25))) {
            if child.stderr() != b"ready\ninterrupted\n" {
                return Err("an interrupt never starts a repeated sweep".into());
            }
            if child.exit_status().is_some() {
                return Err("the handler lets the program continue".into());
            }
            if child.counts().is_some() {
                return Err("a live tree retains its keeper".into());
            }
            let status = child.tree_status();
            if status.interrupted_pids.len() != 1 {
                return Err(format!(
                    "assertion `left == right` failed: interrupted pids: {} != 1",
                    status.interrupted_pids.len()
                ));
            }
            if status.cleanup() != Cleanup::None {
                return Err(format!("assertion `left == right` failed: cleanup: {:?} != None", status.cleanup()));
            }
            if !status.terminated_pids.is_empty() {
                return Err(format!(
                    "assertion `left == right` failed: terminated pids: {} != 0",
                    status.terminated_pids.len()
                ));
            }
            if !status.killed_pids.is_empty() {
                return Err(format!(
                    "assertion `left == right` failed: killed pids: {} != 0",
                    status.killed_pids.len()
                ));
            }
            self.ready_to_kill = true;
        }
        self.next = if self.checked {
            None
        } else if let Some(at) = self.handled_at {
            Some(at.saturating_add(Duration::from_millis(25)))
        } else {
            (child.stderr() == b"ready\n" || child.stderr().ends_with(b"interrupted\n")).then_some(now)
        };
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        self.next
    }

    fn passed(&self) -> bool {
        self.checked
    }
}

/// The containments a supervised case runs under: the walk (`Containment::Walk`)
/// is Linux's, its subreaper and pidfds (runtime/shell.md, 8); macOS has the group alone.
const WALKS: &[bool] = if cfg!(target_os = "linux") { &[true, false] } else { &[false] };

#[test]
fn a_tree_interrupt_is_handled_without_starting_cleanup() {
    for &force_walk in WALKS {
        let command = Command {
            program: "/bin/sh".into(),
            arguments: vec![
                "-c".into(),
                "trap 'printf \"interrupted\\n\" >&2' INT; printf 'ready\\n' >&2; while :; do :; done".into(),
            ],
            environment: vec![],
            directory: "/tmp".into(),
        };
        let binary = Binary::start_with_tree(command, Mode::Pipes, 1024, Expectation::Supervise, force_walk)
            .expect("the supervised program starts");
        let mut world =
            real::World::new(Judge { sent: false, handled_at: None, checked: false, ready_to_kill: false, next: None });
        world.spawn_with_fds(binary.descriptors(), || binary);
        let outcome = world.run(&Clock::new(), Duration::from_secs(2));
        let child = &outcome.procs[0];
        assert_eq!(child.exit_status(), Some(Exit::Signal(9)));
        assert_eq!(child.tree_status().cleanup(), Cleanup::Kill);
        assert_eq!(child.tree_status().interrupted_pids, child.tree_status().killed_pids);
    }
}
