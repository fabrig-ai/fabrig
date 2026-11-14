//! Jobs' durable claim and inbox transitions (domain/landing-history.md, 2.2, 3.2).
use super::{Domain, Work, forge, tasks};
use forge::history::{Job, JobOutcome, JobStep};

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

pub(crate) fn saved(domain: &mut Domain, record: &jig_domain_core::Record) {
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
                    None => JobOutcome::Failed,
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
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => JobOutcome::Lost,
        tasks::Phase::Ended(tasks::Ending::Failed { .. } | tasks::Ending::Done(_)) => JobOutcome::Failed,
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
