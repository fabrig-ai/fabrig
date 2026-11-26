//! Workflow failures and runner faults (domain/landing-proof.md, 2.3–2.4).
use crate::Commit;
use alloc::boxed::Box;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RunState {
    Waiting,
    Running,
    Completed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
}
/// The failed step is zero-based, with set-up first and completion last.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Job {
    pub conclusion: Option<Conclusion>,
    pub failed: Option<u32>,
    pub steps: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Run {
    pub commit: Commit,
    pub state: RunState,
    pub conclusion: Option<Conclusion>,
    pub jobs: Box<[Job]>,
    pub cancelled_by_temper: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Verdict {
    Running,
    Passed,
    Failed,
    Fault(Fault),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fault {
    SetUp,
    Completion,
    NoStep,
    Cancelled,
    OtherCommit,
    Lost,
    /// A tier dispatch refused because its workflow is absent at the run's commit.
    /// See temper/docs/design/domain/after-landing-runs.md, 2.
    MissingWorkflow,
    /// A missing report retained in older tier records.
    Unreported,
}

/// What a check showed of its verdict (temper/docs/design/domain/forge/judgement.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Shown {
    /// A passed verdict: a passed run, a settled close.
    Passed,
    /// The change's own failures: a promised record naming them, or a workflow failure where no record is promised.
    Failures,
    /// Nothing the change caused: a record missing, unread or naming no failure, or no verdict from the check.
    Nothing,
}

/// A check's class (temper/docs/design/domain/forge/judgement.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Judgement {
    Passed,
    Refused,
    /// The runner's, naming the first of its facts.
    Runner(Fault),
    Broke,
}

/// `runner` is the first fact of the runner's among all the run's jobs,
/// whatever else they showed. A job or run cancelled by no one of
/// temper's is `Fault::Cancelled`, the run interrupted.
#[must_use]
pub fn judge(shown: Shown, runner: Option<Fault>) -> Judgement {
    match shown {
        Shown::Passed => Judgement::Passed,
        Shown::Failures => Judgement::Refused,
        Shown::Nothing => match runner {
            Some(fault) => Judgement::Runner(fault),
            None => Judgement::Broke,
        },
    }
}

/// Classify only the run and step facts, never log words.
#[must_use]
pub fn verdict(run: &Run, candidate: Commit) -> Verdict {
    if run.commit != candidate {
        return Verdict::Fault(Fault::OtherCommit);
    }
    match run.state {
        RunState::Waiting | RunState::Running => return Verdict::Running,
        RunState::Completed => {}
    }
    match run.conclusion {
        Some(Conclusion::Success) => Verdict::Passed,
        Some(Conclusion::Cancelled) if !run.cancelled_by_temper => Verdict::Fault(Fault::Cancelled),
        Some(Conclusion::Cancelled) => Verdict::Running,
        Some(Conclusion::Failure) => failed_jobs(&run.jobs),
        Some(Conclusion::Skipped) | None => Verdict::Fault(Fault::NoStep),
    }
}

fn failed_jobs(jobs: &[Job]) -> Verdict {
    let mut first = None;
    for job in jobs {
        if job.conclusion != Some(Conclusion::Failure) {
            continue;
        }
        let Some(fault) = job_fault(*job, false) else { return Verdict::Failed };
        if first.is_none() {
            first = Some(fault);
        }
    }
    Verdict::Fault(first.unwrap_or(Fault::NoStep))
}

/// A job's runner fact, independent of what any other job showed.
#[must_use]
pub fn job_fault(job: Job, cancelled_by_temper: bool) -> Option<Fault> {
    match job.conclusion {
        Some(Conclusion::Failure) => match job.failed {
            Some(step) if step > 0 && step < job.steps.saturating_sub(1) => None,
            Some(0) if job.steps > 0 => Some(Fault::SetUp),
            Some(step) if job.steps > 1 && step == job.steps.saturating_sub(1) => Some(Fault::Completion),
            Some(_) | None => Some(Fault::NoStep),
        },
        Some(Conclusion::Cancelled) if !cancelled_by_temper => Some(Fault::Cancelled),
        Some(Conclusion::Cancelled | Conclusion::Success | Conclusion::Skipped) | None => None,
    }
}
