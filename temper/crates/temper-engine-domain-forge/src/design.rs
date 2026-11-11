//! A goal's design writers live beside its slot (domain/forge/branches.md, 3).
use super::{
    Domain, Env, Handoff, Hold, Limits, List, LostReadState, LostWriterKind, Name, PendingLost, Queue, Request, Token,
    Writer, child, emit, goal_branch, lost_failed, save_hold, to_client,
};

pub(super) fn design_claim_permitted(
    d: &Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    names: &[Name],
    holders: &[u64],
    out: &mut Queue<Request>,
) -> bool {
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
        if !goal_branch::is_goal(d, name) || goal_branch::waiting(d, name) || hold.drift.is_some() || !owned {
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

/// The keep remains written until the last slot, design writer or merge ends.
pub(super) fn branch_written(d: &Domain, name: &Name, ending: Option<u64>) -> bool {
    if let Some(hold) = d.holds.get(name)
        && (hold.writer.is_some() || !hold.design_writers.is_empty())
    {
        return true;
    }
    for (number, row) in &d.entries {
        if Some(*number) == ending || row.held || row.handoff != Handoff::Handed {
            continue;
        }
        if let skein_forge_client::Condition::Merge { base } = &row.entry.effect.condition
            && row.entry.repository.forge == name.forge
            && row.entry.repository.repository == name.repository
            && crate::items::branch_matches(name, base)
        {
            return true;
        }
    }
    false
}

/// Nothing pushes a goal's run branch in its goal repository (domain/forge/keeping-up.md, 1).
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

pub(super) fn pending_writer(hold: &Hold, pending: &PendingLost) -> bool {
    match pending.kind {
        LostWriterKind::Slot => hold.writer == Some(Writer::Lost { task: pending.task, attempt: pending.attempt }),
        LostWriterKind::Design => hold.design_writers.contains(&crate::DesignWriter {
            task: pending.task,
            attempt: pending.attempt,
            rereading: true,
        }),
    }
}

pub(super) fn read_designs(d: &mut Domain, env: &Env<Limits>, task: u64, attempt: u64, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if d.forge_ended(name.forge) {
            continue;
        }
        for writer in &hold.design_writers {
            if writer.task == task && writer.attempt == attempt {
                names.push(name.clone()).expect("one design writer per task per hold");
            }
        }
    }
    for name in &names {
        let mut reading = false;
        for (_, pending) in &d.lost {
            if pending.name == *name
                && pending.task == task
                && pending.attempt == attempt
                && pending.kind == LostWriterKind::Design
            {
                reading = true;
            }
        }
        if reading {
            continue;
        }
        let hold = d.holds.get_mut(name).expect("collected design hold");
        for writer in &mut hold.design_writers {
            if writer.task == task && writer.attempt == attempt {
                writer.rereading = true;
            }
        }
        save_hold(out, hold.clone());
        let Some(resource) = to_client(name, &env.limits) else { continue };
        let read = match resource.what {
            skein_forge_client::What::Branch(branch) => skein_forge::api::Read::Branch { branch },
            skein_forge_client::What::Repository
            | skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Issue(_) => {
                unreachable!("a design writer holds a goal branch")
            }
        };
        let Some(sequence) = d.sequence.checked_add(1) else {
            lost_failed(d, task, attempt, name, b"design-writer read sequence exhausted", out);
            return;
        };
        d.sequence = sequence;
        let owner = Token::new(sequence | (1_u64 << 62));
        let baseline = d.branch_head(name);
        if d.lost
            .insert(
                owner,
                PendingLost {
                    task,
                    attempt,
                    name: name.clone(),
                    comparison: None,
                    baseline,
                    state: LostReadState::Asked,
                    kind: LostWriterKind::Design,
                },
            )
            .is_err()
        {
            lost_failed(d, task, attempt, name, b"design-writer read capacity exhausted", out);
            return;
        }
        child(d, env, skein_forge_client::Event::Read { owner, repository: resource.repository, read }, out);
    }
}
