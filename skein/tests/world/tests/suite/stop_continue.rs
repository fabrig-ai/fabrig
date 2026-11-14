//! A supervised tree records stop and continue without starting cleanup.

use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::Clock;
use skein_world::end_to_end::{Binary, Cleanup, Command, Expectation, Mode};
use skein_world::{Referee, real};

struct Judge {
    stage: u8,
    next: Option<Time>,
}

impl Referee<Binary> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Binary]) {
        let child = &mut procs[0];
        let signal = match self.stage {
            0 if child.stderr() == b"ready\n" => Some(Signal::Stop),
            1 if !child.tree_status().stopped_pids.is_empty() => Some(Signal::Continue),
            3 => Some(Signal::Terminate),
            _ => None,
        };
        if let Some(signal) = signal {
            child.signal_tree(signal);
            self.stage += 1;
        }
    }

    fn watch(&mut self, now: Time, procs: &[Binary]) -> Result<(), String> {
        let child = &procs[0];
        let status = child.tree_status();
        if self.stage == 2 && !status.continued_pids.is_empty() {
            if status.stopped_pids != status.continued_pids {
                return Err(format!(
                    "assertion `left == right` failed: stopped and continued pids: {:?} != {:?}",
                    status.stopped_pids, status.continued_pids
                ));
            }
            if status.cleanup() != Cleanup::None {
                return Err(format!("assertion `left == right` failed: cleanup: {:?} != None", status.cleanup()));
            }
            if child.exit_status().is_some() {
                return Err(format!(
                    "assertion `left == right` failed: exit status: {:?} != None",
                    child.exit_status()
                ));
            }
            self.stage = 3;
        }
        self.next = (self.stage < 4).then_some(now);
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        self.next
    }

    fn passed(&self) -> bool {
        self.stage == 4
    }
}

/// The containments a supervised case runs under: the walk (`Containment::Walk`)
/// is Linux's, its subreaper and pidfds (runtime/shell.md, 8); macOS has the group alone.
const WALKS: &[bool] = if cfg!(target_os = "linux") { &[true, false] } else { &[false] };

#[test]
fn a_tree_records_stop_and_continue_without_cleanup() {
    // One shell holds the tree alive until the referee terminates it.
    const STDERR_LIMIT: usize = 1024;
    for &force_walk in WALKS {
        let command = Command {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), "printf 'ready\\n' >&2; while :; do :; done".into()],
            environment: vec![],
            directory: "/tmp".into(),
        };
        let binary = Binary::start_with_tree(command, Mode::Pipes, STDERR_LIMIT, Expectation::Supervise, force_walk)
            .expect("the supervised program starts");
        let mut world = real::World::new(Judge { stage: 0, next: None });
        world.spawn_with_fds(binary.descriptors(), || binary);
        let outcome = world.run(&Clock::new(), Duration::from_secs(2));
        let child = &outcome.procs[0];
        assert_eq!(child.exit_status(), Some(Exit::Signal(15)));
        assert_eq!(child.tree_status().stopped_pids.len(), 1);
        assert_eq!(child.tree_status().continued_pids, child.tree_status().terminated_pids);
    }
}
