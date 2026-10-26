//! Completion obligations at the frozen stop boundary (domain/testing.md, 2.4).
use crate::service::ServiceHost;
use crate::world::Proc;
use jig_domain_tasks::{Active, Phase, Was};
use skein_shell::Host;
use temper_worker_domain::host;

/// One exact, intentional unfinished state; parked tasks need no declaration.
#[derive(Clone, Copy, Debug)]
pub struct Leftover {
    pub task: u64,
    pub state: Left,
    pub why: &'static str,
}

/// States that still owe work when a story's last call has returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Left {
    /// An activation has been requested but not prepared.
    Due,
    /// The engine is gathering the activation's brief and resources.
    Preparing,
    /// The engine committed the attempt, before its start was reported.
    Claimed,
    /// An admitted attempt has not reported its terminal.
    Running,
    /// Task closure still owes a run, delegates, effects or resource release.
    Closing,
    /// A prepared assignment still waits to be handed to a worker.
    AssignmentPending,
    /// The worker is preparing its hosted attempt's workspace.
    HostedPreparing,
    /// The worker has not finished starting the agent.
    HostedStarting,
    /// The worker's agent is at work.
    HostedActive,
    /// A waiting agent still occupies a live worker activation, before parking.
    HostedWaiting,
    /// The worker is stopping or saving an attempt before its terminal.
    HostedEnding,
}

#[derive(Debug)]
enum Subject {
    Task(u64),
    Child { index: usize, program: &'static str },
}
#[derive(Debug)]
struct Obligation {
    subject: Subject,
    state: Option<Left>,
    source: &'static str,
}
#[derive(Debug, Default)]
pub(crate) struct Obligations {
    live: Vec<Obligation>,
}
impl Obligations {
    /// Called by the referee only, outside every host's heap meter.
    pub(crate) fn read(&mut self, service: &ServiceHost) {
        let Some(service) = &service.service else { return };
        if let Some(engine) = service.engine_domain() {
            let tasks = engine.tasks();
            for row in &tasks.view_tasks() {
                let task = tasks.task(row.number).expect("live task view");
                self.phase(row.number, &task.phase);
            }
            for task in &engine.pending_assignments() {
                self.task(*task, Left::AssignmentPending, "engine");
            }
        }
        if let Some(worker) = service.worker_domain() {
            for hosting in &worker.hosting() {
                let state = match hosting.phase {
                    host::Phase::Preparing => Left::HostedPreparing,
                    host::Phase::Starting => Left::HostedStarting,
                    host::Phase::Active => Left::HostedActive,
                    host::Phase::Waiting => Left::HostedWaiting,
                    host::Phase::Ending => Left::HostedEnding,
                    host::Phase::Answered => continue,
                };
                self.task(hosting.run.raw(), state, "worker");
            }
        }
    }
    pub(crate) fn read_child(&mut self, index: usize, child: &Proc) {
        if child.exit().is_some() {
            return;
        }
        match child {
            Proc::Agent(_) => self.child(index, "agent"),
            Proc::Git(_) | Proc::GitRoute(_) => self.child(index, "git"),
            Proc::Llm(_)
            | Proc::Forge(_)
            | Proc::Issuer(_)
            | Proc::Service(_)
            | Proc::Caller(_)
            | Proc::Ctl(_)
            | Proc::Terminal(_) => {}
        }
    }
    fn phase(&mut self, task: u64, phase: &Phase) {
        match phase {
            Phase::Active(active) | Phase::Held { was: Was::Active(active), .. } => self.active(task, *active),
            Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } => self.task(task, Left::Closing, "engine"),
            Phase::Waiting | Phase::Held { was: Was::Waiting, .. } | Phase::Ended(_) => {}
        }
    }
    fn active(&mut self, task: u64, active: Active) {
        let state = match active {
            Active::Due => Left::Due,
            Active::Preparing => Left::Preparing,
            Active::Claimed { .. } => Left::Claimed,
            Active::Running { .. } => Left::Running,
            Active::Idle | Active::BackingOff { .. } => return,
        };
        self.task(task, state, "engine");
    }
    fn task(&mut self, task: u64, state: Left, source: &'static str) {
        self.live.push(Obligation { subject: Subject::Task(task), state: Some(state), source });
    }
    /// A still-alive hosted child is independent evidence, even if its task vanished.
    pub(crate) fn child(&mut self, index: usize, program: &'static str) {
        self.live.push(Obligation { subject: Subject::Child { index, program }, state: None, source: "worker child" });
    }
    pub(crate) fn check(&self, declared: &[Leftover]) -> Result<(), String> {
        use std::fmt::Write;
        let mut errors = String::new();
        for obligation in &self.live {
            match obligation.subject {
                Subject::Task(task) => {
                    let state = obligation.state.expect("task obligation has a state");
                    if !declared.iter().any(|left| left.task == task && left.state == state) {
                        writeln!(errors, "undeclared {} task {task}: {state:?}", obligation.source).expect("string");
                    }
                }
                Subject::Child { index, program } => {
                    writeln!(errors, "undeclared live {program} child {index}").expect("string");
                }
            }
        }
        for (index, left) in declared.iter().enumerate() {
            if left.why.trim().is_empty() {
                writeln!(errors, "leftover task {} {:?} has no reason", left.task, left.state).expect("string");
            }
            if declared[..index].iter().any(|other| other.task == left.task && other.state == left.state) {
                writeln!(errors, "duplicate leftover task {} {:?}", left.task, left.state).expect("string");
            }
            if !self.live.iter().any(|obligation| {
                matches!(obligation.subject, Subject::Task(task) if task == left.task)
                    && obligation.state == Some(left.state)
            }) {
                writeln!(errors, "absent leftover task {} {:?}: {}", left.task, left.state, left.why).expect("string");
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(format!("story completion obligations:\n{errors}")) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_undeclared_live_attempt_names_its_task() {
        let mut obligations = Obligations::default();
        obligations.active(41, Active::Running { attempt: 7 });
        let error = obligations.check(&[]).expect_err("live attempt");
        assert!(error.contains("task 41: Running"), "{error}");
    }
    #[test]
    fn an_undeclared_closing_task_fails() {
        let mut obligations = Obligations::default();
        obligations.task(42, Left::Closing, "engine");
        let error = obligations.check(&[]).expect_err("closing task");
        assert!(error.contains("task 42: Closing"), "{error}");
    }
    #[test]
    fn an_absent_declared_leftover_fails() {
        let error = Obligations::default()
            .check(&[Leftover { task: 43, state: Left::Due, why: "push-only change" }])
            .expect_err("absent declaration");
        assert!(error.contains("absent leftover task 43 Due"), "{error}");
    }
    #[test]
    fn a_parked_task_passes_without_a_declaration() {
        let mut obligations = Obligations::default();
        obligations.phase(44, &Phase::Active(Active::Idle));
        assert_eq!(obligations.check(&[]), Ok(()));
    }
    #[test]
    fn a_present_declared_leftover_passes() {
        let mut obligations = Obligations::default();
        obligations.active(45, Active::Due);
        assert_eq!(obligations.check(&[Leftover { task: 45, state: Left::Due, why: "ends on the push" }]), Ok(()));
    }
    #[test]
    fn a_wrong_state_reports_both_the_obligation_and_the_absence() {
        let mut obligations = Obligations::default();
        obligations.active(46, Active::Running { attempt: 8 });
        let error = obligations
            .check(&[Leftover { task: 46, state: Left::Preparing, why: "workspace is held" }])
            .expect_err("different state");
        assert!(error.contains("task 46: Running"), "{error}");
        assert!(error.contains("absent leftover task 46 Preparing"), "{error}");
    }
    #[test]
    fn a_live_child_cannot_hide_behind_a_task_declaration() {
        let mut obligations = Obligations::default();
        obligations.child(47, "git");
        let error = obligations.check(&[]).expect_err("live git");
        assert!(error.contains("live git child 47"), "{error}");
    }
}
