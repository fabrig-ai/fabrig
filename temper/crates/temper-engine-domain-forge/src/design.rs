//! A goal's design writers live beside its slot (domain/forge/branches.md, 3).
use super::{Domain, Env, Limits, List, Name, Queue, Request, emit, goal_branch, save_hold};

pub(super) fn design_claim_permitted(
    d: &Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    names: &[Name],
    holders: &[u64],
    out: &mut Queue<Request>,
) -> bool {
    if !super::repository_reads::names_fresh(d, names) {
        return false;
    }
    if names.len() > skein_lib::usize_of(env.limits.resources_per_task) {
        crate::refusal::resource_limit(
            out,
            task,
            b"claim design",
            names.first().cloned(),
            "forge.resources_per_task",
            env.limits.resources_per_task,
            skein_lib::len_u64(names.len()),
        );
        return false;
    }
    for name in names {
        let Some(hold) = d.holds.get(name) else {
            emit(out, Request::Refused { task });
            return false;
        };
        if hold.task != task && !holders.contains(&hold.task) {
            emit(out, Request::Taken { task, resource: name.clone(), by: hold.task });
            return false;
        }
        let owned = match d.heads.get(name) {
            Some(head) => head.owned,
            None => false,
        };
        if !goal_branch::is_goal(d, name)
            || goal_branch::waiting(d, name)
            || hold.drift.is_some()
            || !owned
            || super::retired_reading(d, name)
        {
            emit(out, Request::Refused { task });
            return false;
        }
        let mut replacing = false;
        for writer in &hold.design_writers {
            if writer.task == task {
                if writer.attempt > attempt {
                    return false;
                }
                replacing = true;
            }
        }
        if !replacing && hold.design_writers.len() >= skein_lib::usize_of(env.limits.holds) {
            crate::refusal::resource_limit(
                out,
                task,
                b"claim design",
                Some(name.clone()),
                "forge.holds",
                env.limits.holds,
                skein_lib::len_u64(hold.design_writers.len()).saturating_add(1),
            );
            return false;
        }
    }
    true
}

pub(super) fn goal_run(d: &Domain, name: &Name) -> bool {
    let repository = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
    let Some(repository) = d.repositories.get(&repository) else { return false };
    let Some(run) = crate::items::run_branch(name, &repository.prefix) else { return false };
    for (branch, hold) in &d.holds {
        if branch.forge == name.forge
            && branch.repository == name.repository
            && hold.task == run.1
            && crate::items::goal_branch(branch, &repository.prefix) == Some(run)
        {
            return true;
        }
    }
    false
}

/// An unpublished c or r holding releases without a forge write (domain/forge/branches.md, 1).
pub(super) fn unpublished_branch(d: &Domain, name: &Name) -> bool {
    if d.branch_head(name).is_some() {
        return false;
    }
    let repository = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
    let Some(repository) = d.repositories.get(&repository) else { return false };
    crate::items::run_branch(name, &repository.prefix).is_some()
        || crate::items::change_branch(name, &repository.prefix).is_some()
}

/// Attempts release their design claims without reading the branch; their
/// admitted pushes settle independently of the claim.
pub(super) fn ended(d: &mut Domain, env: &Env<Limits>, task: u64, attempt: u64, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if hold.design_writers.contains(&crate::DesignWriter { task, attempt }) {
            names.push(name.clone()).expect("one design claim per hold");
        }
    }
    for name in &names {
        let hold = d.holds.get_mut(name).expect("collected design hold");
        let mut writers = List::with_capacity(env.limits.holds);
        for writer in &hold.design_writers {
            if writer.task != task || writer.attempt != attempt {
                writers.push(*writer).expect("bounded design claims");
            }
        }
        hold.design_writers = writers.into_boxed();
        save_hold(out, hold.clone());
        super::lost_release(d, env, name, task, out);
    }
}
