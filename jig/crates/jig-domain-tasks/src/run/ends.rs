//! One rule for delegates, dependencies and ending authority (domain/tasks/lifecycle.md, 7–9 and domain/tasks/changes.md).
use crate::domain::{Domain, record};
use crate::{Active, End, Executor, Phase, TaskResult, Was};
use alloc::boxed::Box;

/// Whether an active task may go on, and why not (domain/tasks/lifecycle.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent facts keep their separate claim and finish rules")]
pub(crate) struct Kept {
    /// A dependency is pending.
    pub(crate) dependencies: bool,
    /// Its own unanswered question to its requester, oldest first.
    pub(crate) question: Option<u64>,
    /// Its requester ends it: no run's result may (5.8).
    pub(crate) open: bool,
    /// Its immutable spec requires a requester question it has not sent.
    pub(crate) owes: bool,
    /// An active agent has live delegates, so its ended run cannot hear a refusal (5.6).
    agent_delegates: bool,
}

impl Domain {
    /// A waiting effect reads the task's current kept state (domain/tasks/lifecycle.md, 8).
    #[must_use]
    pub fn waiting_effect_problem(&self, task: u64) -> Option<crate::Problem> {
        let row = record(self, task)?;
        let kept = for_record(row);
        if kept.dependencies {
            Some(crate::Problem::naming(Some(task), crate::Refusal::Kept, &row.waiting_on))
        } else {
            match kept.question {
                Some(question) => Some(crate::Problem::new(Some(task), crate::Refusal::KeptQuestion { question })),
                None => None,
            }
        }
    }
}

pub(crate) fn kept(domain: &Domain, task: u64) -> Kept {
    match record(domain, task) {
        Some(row) => for_record(row),
        None => pending(&[], crate::EndedBy::Result),
    }
}

/// Publication borrows its row mutably; closing keeps cancellation's partial result.
pub(crate) fn for_record(row: &crate::TaskRecord) -> Kept {
    match row.phase {
        Phase::Active(_) | Phase::Held { was: Was::Active(_), .. } => {
            let mut kept = pending(&row.waiting_on, row.flags.ended_by)
                .with_question(question(&row.questions, row.requester))
                .with_owing(row.spec.asks && !row.asked_requester);
            kept.agent_delegates = match row.executor {
                Executor::Agent { .. } => !row.delegates.is_empty(),
                Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => false,
            };
            kept
        }
        Phase::Waiting
        | Phase::Closing(_)
        | Phase::Held { was: Was::Waiting | Was::Closing(_), .. }
        | Phase::Ended(_) => pending(&[], crate::EndedBy::Result),
    }
}

/// The same facts for a compound request's projected dependency list.
pub(crate) fn pending(dependencies: &[u64], ended_by: crate::EndedBy) -> Kept {
    Kept {
        dependencies: !dependencies.is_empty(),
        question: None,
        open: match ended_by {
            crate::EndedBy::Requester | crate::EndedBy::Standing => true,
            crate::EndedBy::Result | crate::EndedBy::Closer(_) => false,
        },
        owes: false,
        agent_delegates: false,
    }
}

pub(crate) fn question(credits: &[crate::QuestionCredit], requester: crate::Party) -> Option<u64> {
    let mut first = None;
    for credit in credits {
        if credit.asked == requester {
            first = Some(match first {
                Some(old) => core::cmp::min(old, credit.number),
                None => credit.number,
            });
        }
    }
    first
}

/// Each caller asks its own question; a new reason to keep a task answers every one.
impl Kept {
    pub(crate) fn with_question(mut self, question: Option<u64>) -> Self {
        self.question = question;
        self
    }
    pub(crate) fn with_owing(mut self, owes: bool) -> Self {
        self.owes = owes;
        self
    }
    /// The task starts no run: an agent is never prepared, claimed or Due (5.7).
    pub(crate) fn blocks_claims(self) -> bool {
        self.dependencies || self.question.is_some()
    }

    /// A done end waits until every dependency is met: a person's answer is refused, a run's parks.
    pub(crate) fn refuses_done(self) -> bool {
        self.blocks_claims() || self.owes
    }

    /// No run of it may end it, so a finish grace without a live run cancels it at once.
    pub(crate) fn finishes_alone(self) -> bool {
        !self.refuses_done() && !self.open
    }

    /// Whether a wake may make this executor Due; a person or a procedure is never kept from its step.
    pub(crate) fn may_be_due(self, executor: Executor) -> bool {
        match executor {
            Executor::Agent { .. } | Executor::Job { .. } => !self.blocks_claims(),
            Executor::Person(_) | Executor::Procedure { .. } => true,
        }
    }

    /// An amendment invalidates an agent's queued preparation; a committed claim stays live.
    pub(crate) fn amended(self, executor: Executor, phase: Phase) -> Phase {
        if self.may_be_due(executor) {
            return phase;
        }
        match phase {
            Phase::Active(Active::Due | Active::Preparing) => Phase::Active(Active::Idle),
            Phase::Waiting
            | Phase::Active(
                Active::Idle | Active::Claimed { .. } | Active::Running { .. } | Active::BackingOff { .. },
            )
            | Phase::Held { .. }
            | Phase::Closing(_)
            | Phase::Ended(_) => phase,
        }
    }

    /// Open discards a run's raw finish before judgment; its run parks (domain/tasks/lifecycle.md, 9).
    pub(crate) fn discards(self, end: &End) -> bool {
        self.open
            && match end {
                End::Finished { .. } | End::FinishedWithFollowups { .. } => true,
                End::Exceeded(_)
                | End::Parked
                | End::Failed(_)
                | End::FailedWithReason { .. }
                | End::Refused
                | End::Recheck => false,
            }
    }

    /// A judged done end waits until every dependency is met; a failure still ends (5.7).
    pub(crate) fn parks(self, end: &End) -> bool {
        self.refuses_done()
            && match end {
                End::Finished { result, .. } | End::FinishedWithFollowups { result, .. } => done(result),
                End::Exceeded(_)
                | End::Parked
                | End::Failed(_)
                | End::FailedWithReason { .. }
                | End::Refused
                | End::Recheck => false,
            }
    }

    /// A judged run's finish (5.6, 5.7): live delegates wait, and an own question parks a failure too.
    pub(crate) fn run_parks(self, end: &End) -> bool {
        let delegates = self.agent_delegates
            && match end {
                End::Finished { cancel_delegates, .. } => !cancel_delegates,
                End::FinishedWithFollowups { .. }
                | End::Exceeded(_)
                | End::Parked
                | End::Failed(_)
                | End::FailedWithReason { .. }
                | End::Refused
                | End::Recheck => false,
            };
        self.parks(end) || (self.question.is_some() && finished(end)) || delegates
    }
}

/// The explanation retained when a standing task discards its raw finish.
pub(crate) fn stood(end: &End) -> Option<Box<[u8]>> {
    let result = match end {
        End::Finished { result, .. } | End::FinishedWithFollowups { result, .. } => result,
        End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => return None,
    };
    let words = match result {
        TaskResult::Report { words }
        | TaskResult::Verdict { words, .. }
        | TaskResult::Change { words, .. }
        | TaskResult::NoChange { words } => words.as_ref(),
        TaskResult::Failure { reason } => reason.as_ref(),
        TaskResult::Approved { words, .. } => words.as_deref().unwrap_or(&[]),
        TaskResult::Job { .. } => &[],
    };
    Some(Box::from(skein_lib::bytes::utf8_prefix(words, skein_lib::length::usize_of(crate::STOOD_WORDS))))
}

fn finished(end: &End) -> bool {
    match end {
        End::Finished { .. } | End::FinishedWithFollowups { .. } => true,
        End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => false,
    }
}

fn done(result: &TaskResult) -> bool {
    match result {
        TaskResult::Failure { .. } => false,
        TaskResult::Approved { .. }
        | TaskResult::Report { .. }
        | TaskResult::Verdict { .. }
        | TaskResult::Job { .. }
        | TaskResult::NoChange { .. }
        | TaskResult::Change { .. } => true,
    }
}
