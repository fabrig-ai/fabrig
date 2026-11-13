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
        job: JobStep { task, attempt: u32::try_from(attempt).expect("job attempt fits history"), job, outcome: None, head: None },
    }));
}

pub(super) fn ended(domain: &Domain, context: &tasks::RunContext, word: &tasks::Word, job: Job) -> Option<JobStep> {
    let tasks::Party::Task(task) = word.from else { return None };
    let child = domain.core.tasks.task(task)?;
    let kind = match job { Job::Prepare => forge::PREPARE_KIND, Job::Merge => forge::MERGE_KIND };
    if child.executor != (tasks::Executor::Job { kind }) {
        return None;
    }
    let outcome = if super::close::failed(domain, task, &word.kind) {
        JobOutcome::Failed
    } else if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Cancelled) {
        JobOutcome::Lost
    } else if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job) {
        match job {
            Job::Prepare => {
                let members = domain.forge.landing(context.task)?.landing.members.len();
                let outcomes = forge::preparation::outcomes(&word.words, u32::try_from(members).ok()?);
                match outcomes {
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
                if has_left(&word.words) { JobOutcome::Conflict } else { JobOutcome::Done }
            }
        }
    } else {
        return None;
    };
    let head = if job == Job::Merge && outcome == JobOutcome::Done {
        let row = domain.forge.change(context.task)?;
        match row.delegate {
            Some((_, super::forge_change::Delegate::Merge { from, .. })) if row.change.last_head != from => row.change.last_head,
            Some(_) | None => None,
        }
    } else { None };
    Some(JobStep { task, attempt: u32::try_from(child.attempt).expect("job attempt fits history"), job, outcome: Some(outcome), head })
}
fn has_left(bytes: &[u8]) -> bool {
    let Ok(length) = u32::try_from(bytes.len()) else { return false };
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let Ok(document) = skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length }) else { return false };
    document.member(0, b"left").is_some()
}
