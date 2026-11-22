//! The supplied-state supervisor shared by real stories and replaying controls.

use crate::hosted::Script;
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_world::end_to_end::{Binary, Cleanup};
use skein_world::{Delegate, Host, Referee};

#[derive(Debug)]
pub enum Process {
    Child(Binary),
    Person(Script),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Child(child) => child,
            Self::Person(person) => person,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Child(child) => child,
            Self::Person(person) => person,
        }
    }
}

#[must_use]
pub fn ready_descendant(stderr: &[u8]) -> Option<u32> {
    std::str::from_utf8(stderr)
        .expect("fixture stderr is text")
        .split_inclusive('\n')
        .filter(|line| line.ends_with('\n'))
        .find_map(|line| line.strip_prefix("descendant:").and_then(|pid| pid.trim().parse().ok()))
}

#[derive(Clone, Copy, Debug)]
pub enum SupervisedTrigger {
    // The detached-descendant cases use Linux's tree keeper (runtime/shell.md, 8).
    #[cfg(target_os = "linux")]
    Ready,
    // The detached-descendant case uses Linux's tree keeper (runtime/shell.md, 8).
    #[cfg(target_os = "linux")]
    StderrOverflow,
    // The detached-descendant cases use Linux's tree keeper (runtime/shell.md, 8).
    #[cfg(target_os = "linux")]
    LeaderExit,
    Bound,
    Never,
}

pub struct SupervisedJudge {
    pub trigger: SupervisedTrigger,
    pub grace: Duration,
    pub cleanup: Cleanup,
    pub term_at: Option<Time>,
    pub kill_at: Option<Time>,
    pub next: Option<Time>,
    pub saw_retained_exit: bool,
    pub saw_grace: bool,
    pub failure: Option<&'static str>,
}

impl SupervisedJudge {
    #[must_use]
    pub fn new(trigger: SupervisedTrigger, grace: Duration, cleanup: Cleanup) -> Self {
        Self {
            trigger,
            grace,
            cleanup,
            term_at: None,
            kill_at: None,
            next: None,
            saw_retained_exit: false,
            saw_grace: false,
            failure: None,
        }
    }
}

/// The observer's supplied state, with no process or kernel required
/// (skein/docs/foundation/testing-strategy.md, 2.1).
pub struct SupervisedSnapshot<'a> {
    pub stderr: &'a [u8],
    pub stderr_overflowed: bool,
    pub exit: Option<Exit>,
    pub settled: bool,
    pub tree: &'a skein_world::end_to_end::TreeStatus,
}

impl SupervisedSnapshot<'_> {
    #[must_use]
    pub fn from_binary(child: &Binary) -> SupervisedSnapshot<'_> {
        SupervisedSnapshot {
            stderr: child.stderr(),
            stderr_overflowed: child.stderr_overflowed(),
            exit: child.exit_status(),
            settled: child.counts().is_some(),
            tree: child.tree_status(),
        }
    }
}

impl SupervisedJudge {
    pub fn act_snapshot(&mut self, now: Time, child: &SupervisedSnapshot<'_>) -> Option<Signal> {
        if self.term_at.is_none() {
            let ready = match self.trigger {
                #[cfg(target_os = "linux")]
                SupervisedTrigger::Ready => ready_descendant(child.stderr).is_some(),
                #[cfg(target_os = "linux")]
                SupervisedTrigger::StderrOverflow => child.stderr_overflowed,
                #[cfg(target_os = "linux")]
                SupervisedTrigger::LeaderExit => child.exit.is_some(),
                SupervisedTrigger::Bound => now >= Time::ZERO.saturating_add(Duration::from_millis(10)),
                SupervisedTrigger::Never => false,
            };
            if ready {
                self.term_at = Some(now);
                return Some(Signal::Terminate);
            }
        } else if let Some(term_at) = self.term_at
            && !child.settled
        {
            let grace = term_at.saturating_add(self.grace);
            if now >= grace && self.kill_at.is_none() {
                self.kill_at = Some(now);
                return Some(Signal::Kill);
            }
        }
        None
    }

    pub fn observe_snapshot(&mut self, now: Time, child: &SupervisedSnapshot<'_>) -> Result<(), String> {
        // A forced cleanup gives the live TERM-delivered tree its grace before
        // the leader wait completes. The exit may first reach this referee
        // after escalation; its history survives the current membership poll.
        if self.cleanup == Cleanup::Kill {
            let tree = child.tree;
            self.saw_retained_exit |= tree.live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
            if self.term_at.is_some_and(|at| now < at.saturating_add(self.grace))
                && tree.live_pids.iter().any(|pid| tree.terminated_pids.contains(pid))
            {
                if !tree.killed_pids.is_empty() {
                    return Err("the keeper leaves escalation to its referee".into());
                }
                self.saw_grace = true;
            }
        }
        if child.exit.is_some() && !child.tree.live_pids.is_empty() {
            if child.settled {
                return Err("a living tree retains its keeper after leader exit".into());
            }
            self.saw_retained_exit = true;
            if let Some(term_at) = self.term_at
                && now < term_at.saturating_add(self.grace)
            {
                self.saw_grace = true;
                if !child.tree.killed_pids.is_empty() {
                    return Err("the keeper leaves escalation to its referee".into());
                }
            }
        }
        // Once a required window closes or the tree settles, later iterations
        // cannot supply its missing observation. Keep that verdict permanent.
        if self.failure.is_none() {
            self.failure = if self.kill_at.is_some() && self.cleanup != Cleanup::Kill {
                Some("waiting for TERM-only cleanup: the referee unexpectedly escalated to KILL")
            } else if self.cleanup == Cleanup::Kill
                && !self.saw_grace
                && self.term_at.is_some_and(|at| now >= at.saturating_add(self.grace))
            {
                Some(
                    "waiting for a live retained leader exit during TERM grace: the grace window closed before it was observed",
                )
            } else if child.settled && !self.passed() {
                Some(
                    "waiting for a retained leader exit and the expected cleanup: the supervised tree has already settled",
                )
            } else {
                None
            };
        }
        if let Some(why) = self.overdue(now) {
            return Err(format!("supervised referee failed: {why}"));
        }
        self.next = if child.settled {
            None
        } else if let Some(term_at) = self.term_at {
            self.kill_at.is_none().then_some(term_at.saturating_add(self.grace))
        } else {
            match self.trigger {
                #[cfg(target_os = "linux")]
                SupervisedTrigger::Ready => ready_descendant(child.stderr).is_some().then_some(now),
                #[cfg(target_os = "linux")]
                SupervisedTrigger::StderrOverflow => child.stderr_overflowed.then_some(now),
                #[cfg(target_os = "linux")]
                SupervisedTrigger::LeaderExit => child.exit.is_some().then_some(now),
                SupervisedTrigger::Bound => Some(Time::ZERO.saturating_add(Duration::from_millis(10))),
                SupervisedTrigger::Never => None,
            }
        };
        Ok(())
    }
}

impl Referee<Process> for SupervisedJudge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Child(child) = &mut procs[0] else { panic!("binary observer") };
        if let Some(signal) = self.act_snapshot(now, &SupervisedSnapshot::from_binary(child)) {
            child.signal_tree(signal);
        }
    }

    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        let Process::Child(child) = &procs[0] else { return Err("binary observer".into()) };
        self.observe_snapshot(now, &SupervisedSnapshot::from_binary(child))
    }

    fn next_deadline(&self) -> Option<Time> {
        self.next
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        self.failure.map(String::from)
    }
    fn passed(&self) -> bool {
        if self.failure.is_some() {
            return false;
        }
        if self.cleanup == Cleanup::Kill {
            self.kill_at.is_some() && self.saw_grace && self.saw_retained_exit
        } else {
            match self.trigger {
                #[cfg(target_os = "linux")]
                SupervisedTrigger::LeaderExit => self.saw_retained_exit,
                #[cfg(target_os = "linux")]
                SupervisedTrigger::Ready | SupervisedTrigger::StderrOverflow => true,
                SupervisedTrigger::Bound | SupervisedTrigger::Never => true,
            }
        }
    }
}
