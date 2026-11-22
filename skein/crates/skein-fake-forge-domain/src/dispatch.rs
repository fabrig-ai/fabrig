//! Dispatch-only workflows, runner faults and explicit reruns (testing/fakes/forge.md, 2.3).
use crate::fell::Fault;
use crate::{
    Config, Domain, Limits,
    actions::{self, Conclusion, Job, Kept, Run, RunEvent, RunState},
    api::{Answer, Check, Cue, Error, Failing, Permission, What},
    ci,
    domain::{self, Alarm},
    faults,
    store::Repository,
};
use alloc::boxed::Box;
use skein_lib::{
    Env, Id, List,
    bytes::{copy_of, find},
    usize_of,
};

pub(crate) fn fits(
    limits: &Limits,
    workflow: &[u8],
    branch: &[u8],
    inputs: &[crate::api::WorkflowInput],
) -> Result<(), Error> {
    crate::store::fits(workflow, limits.name_bytes)?;
    crate::store::fits(branch, limits.name_bytes)?;
    if inputs.len() > usize_of(limits.contexts) {
        return Err(Error::TooLarge);
    }
    for (key, value) in inputs {
        crate::store::fits(key, limits.name_bytes)?;
        crate::store::fits(value, limits.body_bytes)?;
    }
    Ok(())
}

pub(crate) fn dispatch(
    domain: &mut Domain,
    env: &Env<Config>,
    id: Id<Repository>,
    user: u64,
    file: Box<[u8]>,
    branch: Box<[u8]>,
    inputs: Box<[crate::api::WorkflowInput]>,
) -> Result<Answer, Error> {
    let repository = domain.repositories.get(id).expect("dispatch repository");
    repository.require(user, Permission::Write)?;
    fits(&env.limits.limits, &file, &branch, &inputs)?;
    let workflow = workflow(repository, &file)?.clone();
    let commit = *repository.branches.get(&*branch).ok_or(Error::Missing(What::Branch))?;
    if repository.dispatched.capacity() == 0 {
        return Err(Error::Full);
    }
    let probe = Run {
        id: 0,
        number: 0,
        commit,
        workflow: Some(file.clone()),
        event: RunEvent::Dispatch,
        user,
        cancelled_by: None,
        wait: None,
        state: RunState::Running,
        conclusion: None,
        created: domain::clock(env).max(domain.floor),
        inputs: inputs.clone(),
    };
    crate::reports::failed(domain, &workflow, &probe, &env.limits.limits)?;
    let artifact = crate::reports::artifact(domain, &workflow, &probe, &env.limits.limits)?;
    let repository = domain.repositories.get_mut(id).expect("dispatch repository");
    if repository.dispatched.len() == repository.dispatched.capacity() {
        let (&oldest, _) = repository.dispatched.iter().next().expect("a full nonempty run store");
        let removed = repository.dispatched.remove(&oldest).expect("oldest retained run");
        for job in &removed.jobs {
            domain.timers.cancel(Alarm::Workflow { repository: id, run: oldest, job: job.id });
            domain.runners.forget(job.id);
        }
    }
    domain.runs = domain.runs.checked_add(1).expect("run ids fit");
    repository.run_number = repository.run_number.checked_add(1).expect("run numbers fit");
    let run = Run {
        id: domain.runs,
        number: repository.run_number,
        commit,
        workflow: Some(file),
        event: RunEvent::Dispatch,
        user,
        cancelled_by: None,
        wait: None,
        state: RunState::Running,
        conclusion: None,
        created: domain::clock(env).max(domain.floor),
        inputs,
    };
    let mut jobs = List::with_capacity(u32::try_from(workflow.jobs.len()).expect("bounded jobs"));
    for (index, name) in workflow.jobs.iter().enumerate() {
        domain.jobs = domain.jobs.checked_add(1).expect("job ids fit");
        jobs.push(Job {
            id: domain.jobs,
            run: run.id,
            index: u32::try_from(index).expect("bounded jobs"),
            attempt: 1,
            name: copy_of(name),
            state: RunState::Running,
            conclusion: None,
            steps: Box::new([]),
            labels: Box::new([]),
            needs: Box::new([]),
            wait: Some(actions::JobWait::Runner),
        })
        .expect("one job per definition");
    }
    for job in jobs.iter_mut() {
        for requirement in &workflow.requirements {
            if requirement.job == job.name {
                job.labels.clone_from(&requirement.labels);
                job.needs.clone_from(&requirement.needs);
            }
        }
    }
    let mut kept = Kept { run, pull: None, jobs, artifact: None };
    actions::complete(&mut kept);
    if kept.run.state == RunState::Completed {
        kept.artifact = artifact;
    }
    let run = kept.run.id;
    repository.dispatched.insert(run, kept).expect("run room admitted");
    for index in 0..u32::try_from(workflow.jobs.len()).expect("bounded jobs") {
        let job = domain
            .repositories
            .get(id)
            .expect("repository")
            .dispatched
            .get(&run)
            .expect("run")
            .jobs
            .get(index)
            .expect("job")
            .id;
        pending(domain, env, id, run, job, true);
    }
    wake(domain, env);
    Ok(Answer::Done)
}

fn workflow<'a>(repository: &'a Repository, file: &[u8]) -> Result<&'a crate::api::Workflow, Error> {
    for workflow in &repository.workflows {
        if workflow.file.as_ref() == file {
            return Ok(workflow);
        }
    }
    Err(Error::Missing(What::File))
}

/// The first recheck is at the declared wait's end. Later rechecks only wait
/// for labels and needs, including after an image restores the timer.
fn pending(domain: &mut Domain, env: &Env<Config>, id: Id<Repository>, run: u64, job: u64, starting: bool) {
    let repository = domain.repositories.get(id).expect("repository");
    let kept = repository.dispatched.get(&run).expect("dispatched run");
    let file = kept.run.workflow.as_ref().expect("workflow run");
    let definition = workflow(repository, file).expect("configured workflow");
    let phases = ci::phases(domain, kept.run.commit, definition.cue.as_ref(), &env.limits.limits);
    let remaining = if starting { phases.wait } else { skein_lib::Duration::ZERO };
    let mut wait = if remaining == skein_lib::Duration::ZERO { None } else { Some(actions::JobWait::Runner) };
    let mut found = None;
    for entry in &kept.jobs {
        if entry.id == job {
            found = Some(entry);
            break;
        }
    }
    let entry = found.expect("retained job");
    if let Some(served) = &env.limits.runner_labels {
        for label in &entry.labels {
            if !served.contains(label) {
                wait = Some(actions::JobWait::Runner);
            }
        }
    }
    for need in &entry.needs {
        for other in &kept.jobs {
            if other.name == *need && other.state != RunState::Completed {
                wait = Some(actions::JobWait::Needs);
            }
        }
    }
    if wait.is_none() && !crate::runners::take(domain, env.now, id, run, job) {
        wait = Some(actions::JobWait::Runner);
    }
    let repository = domain.repositories.get_mut(id).expect("repository");
    let kept = repository.dispatched.get_mut(&run).expect("dispatched run");
    for entry in kept.jobs.iter_mut() {
        if entry.id == job {
            entry.wait = wait;
            if wait.is_none() {
                entry.steps = Box::new([
                    actions::Step {
                        number: 0,
                        name: copy_of(b"Set up job"),
                        status: actions::StepStatus::Success,
                        log: Box::new([]),
                    },
                    actions::Step {
                        number: 1,
                        name: entry.name.clone(),
                        status: actions::StepStatus::Running,
                        log: Box::new([]),
                    },
                ]);
            }
        }
    }
    actions::complete(kept);
    let repository = domain.repositories.get(id).expect("repository");
    let kept = repository.dispatched.get(&run).expect("dispatched run");
    let file = kept.run.workflow.as_ref().expect("workflow run");
    let workflow = workflow(repository, file).expect("configured workflow for retained run");
    let (silent, min, max) = (workflow.silent, workflow.latency_min, workflow.latency_max);
    if wait.is_some() {
        domain
            .timers
            .arm(
                Alarm::Workflow { repository: id, run, job },
                env.now.saturating_add(if remaining == skein_lib::Duration::ZERO {
                    max.max(skein_lib::Duration::from_millis(1))
                } else {
                    remaining
                }),
            )
            .expect("retained job timer");
    } else if domain.rng.chance(silent) {
        domain.tally.silent = domain.tally.silent.saturating_add(1);
        let commit = kept.run.commit;
        domain.ci_fell(env.now, Fault::Silent, id, commit, run);
    } else {
        let span = match phases.run {
            Some(span) => span,
            None => faults::draw(domain, min, max),
        };
        domain
            .timers
            .arm(Alarm::Workflow { repository: id, run, job }, env.now.saturating_add(span))
            .expect("one timer per retained dispatched job");
    }
}

pub(crate) fn report(domain: &mut Domain, env: &Env<Config>, id: Id<Repository>, run: u64, job: u64) {
    let repository = domain.repositories.get(id).expect("repository");
    let kept = repository.dispatched.get(&run).expect("timer's retained run");
    for entry in &kept.jobs {
        if entry.id == job && entry.wait.is_some() {
            pending(domain, env, id, run, job, false);
            return;
        }
    }
    let commit = kept.run.commit;
    let lost = domain.rng.chance(env.limits.lost);
    let file = kept.run.workflow.as_ref().expect("dispatched workflow");
    let workflow = workflow(repository, file).expect("configured workflow");
    let failed_ids =
        crate::reports::failed(domain, workflow, &kept.run, &env.limits.limits).expect("report inputs admitted");
    let state = if lost || !failed_ids.is_empty() {
        Check::Failed
    } else {
        match &workflow.cue {
            Some(Cue::Declared { path }) => ci::declared(domain, commit, file, path, None, &env.limits.limits).0,
            Some(Cue::PullFile { no_pull, .. }) => *no_pull,
            Some(Cue::Message { fail }) => {
                if find(&domain.object(commit).expect("run commit").message, fail).is_some() {
                    Check::Failed
                } else {
                    Check::Passed
                }
            }
            Some(Cue::Tests { .. }) => Check::Passed,
            Some(Cue::File { path, green }) => match domain.object(commit).expect("run commit").tree.get(&**path) {
                Some(content) if find(content, green).is_some() => Check::Passed,
                Some(_) | None => Check::Failed,
            },
            None => {
                if domain.rng.chance(workflow.passes) {
                    Check::Passed
                } else {
                    Check::Failed
                }
            }
        }
    };
    let setup = domain.rng.chance(env.limits.setup_faults);
    let state = if setup || lost { Check::Failed } else { state };
    let failing = if lost {
        Failing::NoStep
    } else if setup {
        Failing::Setup
    } else {
        Failing::Workflow
    };
    let artifact =
        crate::reports::artifact(domain, workflow, &kept.run, &env.limits.limits).expect("report inputs admitted");
    let fault = if lost {
        Some(Fault::Lost)
    } else if setup {
        Some(Fault::SetUp)
    } else {
        None
    };
    if let Some(fault) = fault {
        let mut fell = crate::Fell::repository(domain, env.now, fault, id);
        fell.commit = Some(commit);
        fell.run = Some(run);
        fell.runner = domain.runners.runner(job);
        domain.record_fell(fell);
    }
    let repository = domain.repositories.get_mut(id).expect("repository");
    let kept = repository.dispatched.get_mut(&run).expect("run");
    let mut attempt = None;
    for index in 0..kept.jobs.len() {
        let entry = kept.jobs.get_mut(index).expect("retained job");
        if entry.id == job {
            entry.state = RunState::Completed;
            entry.conclusion = Some(if state == Check::Passed { Conclusion::Success } else { Conclusion::Failure });
            entry.steps = actions::outcome_steps(&entry.name, state, failing, None, None, &[]);
            attempt = Some(entry.attempt);
            break;
        }
    }
    actions::complete(kept);
    if crate::reports::reportable(kept) {
        kept.artifact = artifact;
    }
    let attempt = attempt.expect("timer's retained job");
    crate::hooks::notify(
        domain,
        env,
        id,
        crate::hooks::Hook::job(
            crate::api::Ended { job: crate::api::JobRef { run, job, attempt }, commit, state },
            env.limits.ci,
        ),
    );
    domain.tally.verdicts = domain.tally.verdicts.saturating_add(1);
    domain.runners.forget(job);
    wake(domain, env);
}

/// A freed runner starts the oldest ready job at this same instant.
fn wake(domain: &mut Domain, env: &Env<Config>) {
    for _ in 0..domain.runners.waiting_count() {
        if !domain.runners.available() {
            break;
        }
        let Some(next) = domain.runners.next() else { break };
        domain.timers.cancel(Alarm::Workflow { repository: next.repository, run: next.run, job: next.job });
        pending(domain, env, next.repository, next.run, next.job, false);
        if domain.runners.waiting(next.job) {
            break;
        }
    }
}

/// A pool queue is not stored: its jobs resume as ready work after decoding.
pub(crate) fn restore_ready(domain: &mut Domain, env: &Env<Config>) {
    let mut ready = List::with_capacity(domain.runners.capacity());
    for (id, repository) in &domain.repositories {
        for (&run, kept) in &repository.dispatched {
            for job in &kept.jobs {
                if job.state == RunState::Running && job.wait.is_none() && job.steps.is_empty() {
                    ready.push(Alarm::Workflow { repository: id, run, job: job.id }).expect("bounded jobs");
                }
            }
        }
    }
    for alarm in &ready {
        domain.timers.cancel(*alarm);
        match *alarm {
            Alarm::Workflow { repository, run, job } => pending(domain, env, repository, run, job, false),
            Alarm::Call(_) | Alarm::Check { .. } | Alarm::Rerun { .. } | Alarm::Hook(_) | Alarm::Queue(_) => {
                unreachable!("ready jobs")
            }
        }
    }
}

/// A person's cancellation ends dispatch-only work without blaming a workflow step.
pub(crate) fn cancel(
    domain: &mut Domain,
    env: &Env<Config>,
    id: Id<Repository>,
    user: u64,
    run: u64,
) -> Result<Answer, Error> {
    let repository = domain.repositories.get_mut(id).expect("cancel repository");
    repository.require(user, Permission::Write)?;
    let mut commit = None;
    for (&candidate, kept) in &repository.runs {
        if kept.run.id == run {
            commit = Some(candidate);
            break;
        }
    }
    if let Some(commit) = commit {
        return ci::cancel(domain, env, id, user, commit);
    }
    let kept = repository.dispatched.get_mut(&run).ok_or(Error::Missing(What::Job))?;
    if kept.run.state == RunState::Completed {
        return Err(Error::Refused);
    }
    let mut waits = List::with_capacity(kept.jobs.len());
    for job in kept.jobs.iter_mut() {
        if job.state == RunState::Completed {
            continue;
        }
        domain.timers.cancel(Alarm::Workflow { repository: id, run, job: job.id });
        if let Some(wait) = domain.runners.release(job.id) {
            waits.push(wait).expect("at most one wait per cancelled job");
        }
        job.state = RunState::Completed;
        job.conclusion = Some(Conclusion::Cancelled);
        job.wait = None;
        job.steps = actions::outcome_steps(&job.name, Check::Failed, Failing::NoStep, None, None, &[]);
    }
    kept.run.cancelled_by = Some(user);
    actions::complete(kept);
    kept.artifact = None;
    for wait in &waits {
        crate::runners::end_wait(domain, *wait, env.now, None);
    }
    wake(domain, env);
    Ok(Answer::Done)
}

pub(crate) fn rerun(
    domain: &mut Domain,
    env: &Env<Config>,
    id: Id<Repository>,
    user: u64,
    job: u64,
) -> Result<Answer, Error> {
    let repository = domain.repositories.get(id).expect("repository");
    repository.require(user, Permission::Write)?;
    let (commit, entry) = actions::latest(repository, job)?;
    let kept = actions::run(repository, entry.run)?;
    if kept.run.state != RunState::Completed {
        return Err(Error::Refused);
    }
    if kept.run.workflow.is_none() {
        domain.timers.cancel(Alarm::Rerun { repository: id, commit, context: entry.index });
        ci::rerun(domain, env, id, commit, entry.index);
        return Ok(Answer::Done);
    }
    let repository = domain.repositories.get_mut(id).expect("repository");
    let kept = repository.dispatched.get_mut(&entry.run).expect("dispatched run");
    kept.artifact = None;
    kept.run.cancelled_by = None;
    for index in 0..kept.jobs.len() {
        let entry = kept.jobs.get_mut(index).expect("retained job");
        if entry.id == job {
            entry.attempt = entry.attempt.checked_add(1).expect("attempts fit");
            entry.state = RunState::Running;
            entry.conclusion = None;
            entry.steps = Box::new([]);
            break;
        }
    }
    actions::complete(kept);
    pending(domain, env, id, entry.run, job, true);
    Ok(Answer::Done)
}
