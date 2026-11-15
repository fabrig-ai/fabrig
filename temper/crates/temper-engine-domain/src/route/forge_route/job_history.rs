//! Jobs' durable claim and inbox transitions (domain/landing-history.md, 2.2, 3.2).
use super::{Domain, Work, forge, tasks};
use forge::history::{Job, JobFailure, JobOutcome, JobStep};
use jig_domain_host as host;

/// The worker's cause joined to the core's accepted attempt end in one step.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Answer {
    task: u64,
    attempt: u64,
    outcome: JobOutcome,
}

pub(crate) fn answer(domain: &Domain, event: &jig_domain_core::Event) -> Option<Answer> {
    let jig_domain_core::Event::AnswerPayload { run, attempt, payload, .. } = event else { return None };
    let crate::Payload::Answer { end: jig_domain_hub_link::Ending::Job(host::JobEnding::Failed { failure, .. }), .. } =
        domain.hub_link.payload(*payload)? else { return None };
    let outcome = match failure {
        host::JobFailure::Unprepared(_) => JobOutcome::Failed { class: JobFailure::Unprepared },
        host::JobFailure::Failed => JobOutcome::Failed { class: JobFailure::Run },
        host::JobFailure::Deadline => JobOutcome::Timeout,
        host::JobFailure::Result => JobOutcome::Failed { class: JobFailure::Result },
        host::JobFailure::Unstarted => JobOutcome::Failed { class: JobFailure::Unstarted },
        host::JobFailure::Undelivered(_) => JobOutcome::Failed { class: JobFailure::Undelivered },
        host::JobFailure::Cancelled(host::Reason::Engine) => JobOutcome::Cancelled,
        host::JobFailure::Cancelled(host::Reason::Contact | host::Reason::Shutdown) => JobOutcome::Lost,
    };
    Some(Answer { task: run.raw(), attempt: attempt.raw(), outcome })
}

fn answered(answer: Option<Answer>, task: u64, attempt: u64) -> Option<JobOutcome> {
    let answer = answer?;
    (answer.task == task && answer.attempt == attempt).then_some(answer.outcome)
}

pub(crate) fn claimed(domain: &mut Domain, task: u64, attempt: u64, kind: u32) {
    let job = if kind == forge::PREPARE_KIND {
        Job::Prepare
    } else if kind == forge::MERGE_KIND {
        Job::Merge
    } else {
        return;
    };
    let Some(child) = domain.core.tasks.task(task) else { return };
    let tasks::Party::Task(owner) = child.requester else { return };
    if domain.forge.landing(owner).is_none() && domain.forge.change(owner).is_none() {
        return;
    }
    domain.work.push(Work::Forge(forge::Event::JobHistory {
        task: owner,
        job: JobStep {
            task,
            attempt: u32::try_from(attempt).expect("job attempt fits history"),
            job,
            outcome: None,
            head: None,
        },
    }));
}

pub(crate) fn saved(domain: &mut Domain, record: &jig_domain_core::Record, answer: Option<Answer>) {
    if let jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)) = record
        && row.ended.is_some()
    {
        attempt_ended(domain, row, answer);
    }
    let jig_domain_core::Record::Tasks(tasks::Stored::Ended(child)) = record else { return };
    let tasks::Party::Task(owner) = child.requester else { return };
    let job = match child.executor {
        tasks::Executor::Job { kind } if kind == forge::PREPARE_KIND => Job::Prepare,
        tasks::Executor::Job { kind } if kind == forge::MERGE_KIND => Job::Merge,
        tasks::Executor::Agent { .. }
        | tasks::Executor::Procedure { .. }
        | tasks::Executor::Person(_)
        | tasks::Executor::Job { .. } => return,
    };
    let outcome = match &child.phase {
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Job { bytes })) => match job {
            Job::Prepare => {
                let Some(row) = domain.forge.landing(owner) else { return };
                match forge::preparation::outcomes(
                    bytes,
                    u32::try_from(row.landing.members.len()).expect("bounded members"),
                ) {
                    Some(outcomes) => {
                        let mut conflict = false;
                        for outcome in outcomes {
                            conflict |= outcome.outcome != super::landing::Outcome::Combined;
                        }
                        if conflict { JobOutcome::Conflict } else { JobOutcome::Done }
                    }
                    None => JobOutcome::Failed { class: JobFailure::Unknown },
                }
            }
            Job::Merge => {
                if has_left(bytes) {
                    JobOutcome::Conflict
                } else {
                    JobOutcome::Done
                }
            }
        },
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => JobOutcome::Cancelled,
        tasks::Phase::Ended(tasks::Ending::Failed { .. } | tasks::Ending::Done(_)) => {
            answered(answer, child.number, child.attempt).unwrap_or(JobOutcome::Failed { class: JobFailure::Unknown })
        },
        tasks::Phase::Active(_) | tasks::Phase::Waiting | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            return;
        }
    };
    domain.work.push(Work::Forge(forge::Event::JobHistory {
        task: owner,
        job: JobStep {
            task: child.number,
            attempt: u32::try_from(child.attempt).expect("job attempt fits history"),
            job,
            outcome: Some(outcome),
            head: None,
        },
    }));
}
fn attempt_ended(domain: &mut Domain, row: &tasks::AttemptRow, answer: Option<Answer>) {
    let Some(child) = domain.core.tasks.task(row.task) else { return };
    let tasks::Party::Task(owner) = child.requester else { return };
    let job = match child.executor {
        tasks::Executor::Job { kind } if kind == forge::PREPARE_KIND => Job::Prepare,
        tasks::Executor::Job { kind } if kind == forge::MERGE_KIND => Job::Merge,
        tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_)
        | tasks::Executor::Job { .. } => return,
    };
    let outcome = match answered(answer, row.task, row.attempt) {
        Some(outcome) => outcome,
        None => match row.end {
            Some(tasks::EndKind::Finished) => return,
            Some(tasks::EndKind::Cancelled) => JobOutcome::Cancelled,
            Some(tasks::EndKind::Failed { class: tasks::Class::Lost, .. }) => JobOutcome::Lost,
            Some(tasks::EndKind::Failed { .. } | tasks::EndKind::Refused | tasks::EndKind::Parked) => {
                JobOutcome::Failed { class: JobFailure::Unknown }
            }
            None => match child.phase {
                tasks::Phase::Closing(tasks::Closing { ending: tasks::Ending::Cancelled { .. }, .. })
                | tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => JobOutcome::Cancelled,
                tasks::Phase::Active(_) | tasks::Phase::Waiting | tasks::Phase::Closing(_)
                | tasks::Phase::Ended(_) | tasks::Phase::Held { .. } => JobOutcome::Lost,
            },
        },
    };
    domain.work.push(Work::Forge(forge::Event::JobHistory {
        task: owner,
        job: JobStep {
            task: row.task,
            attempt: u32::try_from(row.attempt).expect("job attempt fits history"),
            job,
            outcome: Some(outcome),
            head: None,
        },
    }));
}
fn has_left(bytes: &[u8]) -> bool {
    let Ok(length) = u32::try_from(bytes.len()) else { return false };
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let Ok(document) =
        skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length })
    else {
        return false;
    };
    let Some(left) = document.member(0, b"left") else { return false };
    let Some(end) = document.value_end(left) else { return false };
    end.saturating_sub(left) > 2
}
