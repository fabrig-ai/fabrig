//! Cross-record validation, before a restored domain can escape.
use super::{Refusal, codec::require};
use crate::{Config, Domain, actions, store::Repository};
use skein_lib::{Duration, Id, Set, usize_of};

pub(super) fn config(config: &Config) -> Result<(), Refusal> {
    let l = &config.limits;
    if !crate::valid(l)
        || l.repositories == 0
        || l.users == 0
        || l.items == 0
        || l.commits == 0
        || l.files == 0
        || l.branches == 0
        || l.page_size == 0
        || l.calls == 0
    {
        return Err(Refusal::Limits("configuration"));
    }
    require(
        config.latency_min <= config.latency_max
            && config.settle_min <= config.settle_max
            && config.late_min <= config.late_max
            && config.hook_min <= config.hook_max
            && (config.rate_limit == 0 || config.rate_window > Duration::ZERO),
        "configuration ranges",
    )
}
pub(super) fn domain(d: &Domain, limits: &crate::Limits) -> Result<(), Refusal> {
    for (&id, object) in &d.commits {
        require(id != 0 && id <= d.made, "commit counter")?;
        for parent in [object.parent, object.merge_parent].into_iter().flatten() {
            require(parent < id && d.commits.contains_key(&parent), "commit parent")?;
        }
    }
    let mut comments = Set::with_capacity(u32::MAX);
    let mut reviews = Set::with_capacity(u32::MAX);
    let mut runs = Set::with_capacity(u32::MAX);
    let mut jobs = Set::with_capacity(u32::MAX);
    let mut repository_ids = Set::with_capacity(d.repositories.len());
    for (_, r) in &d.repositories {
        require(r.id != 0, "repository id")?;
        if !r.gone {
            require(repository_ids.insert(r.id).expect("one per repository"), "duplicate repository id")?;
        }
        settings(r, limits)?;
        for &commit in &r.has {
            require(d.commits.contains_key(&commit), "repository commit")?;
        }
        for (_, commit) in &r.branches {
            require(r.has.contains(commit), "branch commit")?;
        }
        items(d, r, &mut comments, &mut reviews)?;
        queue(r)?;
        for (key, kept) in &r.runs {
            require(kept.run.workflow.is_none() && *key == kept.run.commit, "CI run store")?;
        }
        for (key, kept) in &r.dispatched {
            require(kept.run.workflow.is_some() && *key == kept.run.id, "dispatch store")?;
        }
        let mut run_numbers = Set::with_capacity(r.runs.len().saturating_add(r.dispatched.len()));
        for (key, kept) in r.runs.iter().chain(r.dispatched.iter()) {
            let run = &kept.run;
            require(run.id != 0 && run.id <= d.runs, "run counter")?;
            require(runs.insert(run.id).or(Err(Refusal::Limits("runs")))?, "duplicate run id")?;
            require(run.number != 0 && run.number <= r.run_number, "run number")?;
            require(run_numbers.insert(run.number).expect("bounded runs"), "duplicate run number")?;
            require(r.has.contains(&run.commit), "run commit")?;
            if let Some(number) = kept.pull {
                let item = r.items.get(&number).ok_or(Refusal::Malformed("run item"))?;
                require(item.pull.is_some(), "run pull")?;
            }
            let names = match &run.workflow {
                Some(file) => {
                    require(*key == run.id && run.event == actions::RunEvent::Dispatch, "dispatch identity")?;
                    let mut found = None;
                    for workflow in &r.workflows {
                        if workflow.file == *file {
                            found = Some(workflow);
                        }
                    }
                    let workflow = found.ok_or(Refusal::Malformed("run workflow"))?;
                    let _failures = crate::reports::failed(d, workflow, run, limits)
                        .or(Err(Refusal::Limits("workflow selection")))?;
                    let _artifact = crate::reports::artifact(d, workflow, run, limits)
                        .or(Err(Refusal::Limits("workflow report")))?;
                    &workflow.jobs
                }
                None => {
                    require(*key == run.commit && run.event != actions::RunEvent::Dispatch, "CI identity")?;
                    require(r.statuses.contains_key(key), "CI statuses")?;
                    &r.checks.contexts
                }
            };
            require(usize_of(kept.jobs.len()) == names.len(), "run jobs")?;
            for (index, job) in kept.jobs.iter().enumerate() {
                require(job.id != 0 && job.id <= d.jobs && job.attempt != 0, "job counter")?;
                require(jobs.insert(job.id).or(Err(Refusal::Limits("jobs")))?, "duplicate job id")?;
                require(
                    job.run == run.id && usize_of(job.index) == index && names.get(index) == Some(&job.name),
                    "job identity",
                )?;
                for (index, step) in job.steps.iter().enumerate() {
                    require(usize_of(step.number) == index, "step number")?;
                }
            }
        }
        for (commit, statuses) in &r.statuses {
            require(r.has.contains(commit), "status commit")?;
            for (_, status) in statuses {
                if let Some(reference) = status.job {
                    let kept = actions::run(r, reference.run).or(Err(Refusal::Malformed("status run")))?;
                    let mut found = false;
                    for job in &kept.jobs {
                        if job.id == reference.job && reference.attempt <= job.attempt && reference.attempt != 0 {
                            found = true;
                        }
                    }
                    require(found && kept.run.commit == *commit, "status job")?;
                }
            }
        }
        for (_, page) in &r.wiki {
            require(page.revision != 0 && page.revision <= r.revisions, "wiki revision")?;
        }
    }
    Ok(())
}
fn items(d: &Domain, r: &Repository, comments: &mut Set<u64>, reviews: &mut Set<u64>) -> Result<(), Refusal> {
    for (&number, item) in &r.items {
        require(!item.title.is_empty(), "empty title")?;
        for label in &item.labels {
            require(r.labels.contains(label), "undefined label")?;
        }
        for dependency in &item.dependencies {
            require(*dependency != number && r.items.contains_key(dependency), "item dependency")?;
        }
        for (&id, _) in &item.comments {
            require(id != 0 && id <= d.comments, "comment counter")?;
            require(comments.insert(id).or(Err(Refusal::Limits("comments")))?, "duplicate comment id")?;
        }
        if let Some(pull) = &item.pull {
            require(r.has.contains(&pull.commit), "pull head commit")?;
            if let Some(merged) = pull.merged {
                require(r.has.contains(&merged), "pull merge commit")?;
            }
            if let Some(group) = pull.group {
                require(r.has.contains(&group.commit), "queue group commit")?;
            }
            for kept in &pull.reviews {
                let review = &kept.review;
                require(review.id != 0 && review.id <= d.reviews, "review counter")?;
                require(reviews.insert(review.id).or(Err(Refusal::Limits("reviews")))?, "duplicate review id")?;
                require(r.has.contains(&review.commit), "review commit")?;
            }
        }
    }
    Ok(())
}
fn queue(r: &Repository) -> Result<(), Refusal> {
    let mut queued = Set::with_capacity(r.queue.len());
    for (index, entry) in r.queue.iter().enumerate() {
        require(entry.position == u32::try_from(index).expect("bounded index").saturating_add(1), "queue position")?;
        require(queued.insert(entry.number).expect("bounded queue"), "duplicate queue entry")?;
        let item = r.items.get(&entry.number).ok_or(Refusal::Malformed("queue item"))?;
        require(item.pull.is_some(), "queue pull")?;
        // An admission keeps the head it took even after a push or close.
        require(r.has.contains(&entry.head), "queue head")?;
    }
    Ok(())
}
fn settings(r: &Repository, limits: &crate::Limits) -> Result<(), Refusal> {
    require(r.checks.latency_min <= r.checks.latency_max, "check latency")?;
    require(
        r.failing.len() == r.checks.contexts.len() && r.opening.len() == r.checks.contexts.len(),
        "context settings",
    )?;
    let mut contexts = Set::with_capacity(u32::try_from(r.checks.contexts.len()).expect("bounded contexts"));
    for name in &r.checks.contexts {
        require(contexts.insert(name).expect("bounded contexts"), "duplicate check context")?;
    }
    let mut cues = Set::with_capacity(u32::try_from(r.checks.cues.len()).expect("bounded cues"));
    for cue in &r.checks.cues {
        require(r.checks.contexts.contains(&cue.context), "cue context")?;
        require(cues.insert(&cue.context).expect("bounded cues"), "duplicate context cue")?;
    }
    let mut files = Set::with_capacity(u32::try_from(r.workflows.len()).expect("bounded workflows"));
    for workflow in &r.workflows {
        require(files.insert(&workflow.file).expect("bounded workflows"), "duplicate workflow")?;
        require(workflow.latency_min <= workflow.latency_max, "workflow latency")?;
        let mut jobs = Set::with_capacity(u32::try_from(workflow.jobs.len()).expect("bounded jobs"));
        for job in &workflow.jobs {
            require(jobs.insert(job).expect("bounded jobs"), "duplicate workflow job")?;
            let size = workflow.file.len().saturating_add(3).saturating_add(job.len());
            if size > usize_of(limits.name_bytes) {
                return Err(Refusal::Limits("workflow context"));
            }
        }
        for ids in [&workflow.fails, &workflow.flaky] {
            let normalized = crate::reports::normalize(ids, limits).or(Err(Refusal::Limits("workflow ids")))?;
            require(normalized == *ids, "workflow ids order")?;
        }
    }
    Ok(())
}
pub(super) fn check_alarm(d: &Domain, id: Id<Repository>, commit: u64, context: u32) -> Result<(), Refusal> {
    let r = d.repositories.get(id).ok_or(Refusal::Malformed("check repository"))?;
    let kept = r.runs.get(&commit).ok_or(Refusal::Malformed("timer CI run"))?;
    require(r.checks.contexts.get(usize_of(context)).is_some() && kept.jobs.get(context).is_some(), "timer CI context")
}
pub(super) fn workflow_alarm(d: &Domain, id: Id<Repository>, run: u64, job: u64) -> Result<(), Refusal> {
    let r = d.repositories.get(id).ok_or(Refusal::Malformed("workflow repository"))?;
    let kept = r.dispatched.get(&run).ok_or(Refusal::Malformed("timer workflow run"))?;
    for entry in &kept.jobs {
        if entry.id == job {
            return Ok(());
        }
    }
    Err(Refusal::Malformed("timer workflow job"))
}
