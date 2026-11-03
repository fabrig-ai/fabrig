//! Complete watch admission and timed retry (temper/docs/design/domain/forge.md, 5.1).
use super::{
    ChangeRow, Domain, Env, Limits, Name, PendingStep, Request, StepStage, Stored, What, Writer, child, child_env,
    drain, emit, participating, to_client, valid_name,
};
use crate::RefusalCause;
use alloc::boxed::Box;
use skein_forge::api::Error;
use skein_forge_client::{Resource, Watch};
use skein_lib::{Duration, List, Map, Queue, Token};

type Failed = (Option<Name>, RefusalCause);

pub(super) fn names(d: &mut Domain, env: &Env<Limits>, task: u64, resources: Box<[Name]>, out: &mut Queue<Request>) {
    let watches = match plan(d, env, task, &resources) {
        Ok(watches) => watches,
        Err((resource, cause)) => {
            emit(out, Request::ResourceRefused { task, operation: b"watch resources", resource, cause });
            return;
        }
    };
    skein_forge_client::step(
        &mut d.client,
        &child_env(env),
        skein_forge_client::Event::Keep { owner: Token::new(task), watches },
        &mut d.client_out,
    );
    if accepted(&d.client_out) {
        retain(d, task, resources, out);
    }
    drain(d, env, out);
}

fn plan(d: &Domain, env: &Env<Limits>, task: u64, resources: &[Name]) -> Result<Box<[Watch]>, Failed> {
    let resource = resources.first().cloned();
    bound("forge.resources_per_task", env.limits.resources_per_task, resources.len(), resource.clone())?;
    if !d.names.contains_key(&task) && task != u64::MAX {
        bound(
            "forge.tasks",
            env.limits.tasks,
            usize::try_from(d.names.len()).expect("u32 fits usize").saturating_add(1),
            resource.clone(),
        )?;
    }
    let mut watches = Map::uncapped();
    for (owner, names) in &d.names {
        if *owner != task {
            add(d, env, names, &mut watches)?;
        }
    }
    add(d, env, resources, &mut watches)?;
    add(d, env, crate::tiers::watched(d).as_slice(), &mut watches)?;
    bound(
        "forge.client.resources",
        env.limits.client.resources,
        usize::try_from(watches.len()).expect("u32 fits usize"),
        resource.clone(),
    )?;
    let mut repositories = Map::uncapped();
    for (resource, _) in &watches {
        repositories.insert(resource.repository, ()).expect("internal repository set");
    }
    bound(
        "forge.client.repositories",
        env.limits.client.repositories,
        usize::try_from(repositories.len()).expect("u32 fits usize"),
        resource,
    )?;
    let mut list = List::with_capacity(watches.len());
    for (_, watch) in &watches {
        list.push(watch.clone()).expect("counted watches");
    }
    Ok(list.into_boxed())
}

fn add(d: &Domain, env: &Env<Limits>, names: &[Name], watches: &mut Map<Resource, Watch>) -> Result<(), Failed> {
    for name in names {
        validate(d, &env.limits, name)?;
        if d.forge_ended(name.forge) {
            continue;
        }
        let resource = to_client(name, &env.limits).expect("validated name");
        if let skein_forge_client::What::Branch(branch) = &resource.what {
            bound("forge.client.op_bytes", env.limits.client.op_bytes, branch.len(), Some(name.clone()))?;
        }
        let participating = participating(d, &resource);
        watches.insert(resource.clone(), Watch { resource, participating }).expect("internal watch set");
    }
    Ok(())
}

fn validate(d: &Domain, limits: &Limits, name: &Name) -> Result<(), Failed> {
    let repository = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
    if !d.repositories.contains_key(&repository) {
        return Err((Some(name.clone()), RefusalCause::Unadopted));
    }
    if let What::Branch(parts) = &name.what {
        let mut bytes = 0usize;
        for part in parts {
            bytes = bytes.saturating_add(part.len()).saturating_add(1);
        }
        bound("forge.name_bytes", limits.name_bytes, bytes, Some(name.clone()))?;
    }
    if !valid_name(d, limits, name) {
        return Err((Some(name.clone()), RefusalCause::InvalidName));
    }
    Ok(())
}

fn bound(member: &'static str, configured: u32, needed: usize, resource: Option<Name>) -> Result<(), Failed> {
    let needed = u64::try_from(needed).expect("bounded working set");
    let configured = u64::from(configured);
    if needed > configured { Err((resource, RefusalCause::Limit { member, configured, needed })) } else { Ok(()) }
}

fn retain(d: &mut Domain, task: u64, resources: Box<[Name]>, out: &mut Queue<Request>) {
    d.names.insert(task, resources.clone()).expect("preflighted task capacity");
    for name in &resources {
        if let Some((role, hold)) = d.resource_facts(name) {
            let repository = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
            emit(
                out,
                Request::Resource {
                    project: d.repository(repository).expect("named adopted resource").project,
                    name: name.clone(),
                    role,
                    hold,
                },
            );
        }
    }
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Names { task, resources } });
}

pub(super) fn accepted(out: &Queue<skein_forge_client::Request>) -> bool {
    for request in out {
        if let skein_forge_client::Request::Kept { result, .. } = request {
            return result.is_ok() || *result == Err(Error::Busy);
        }
    }
    false
}

pub(super) fn kept(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<(), Error>,
    out: &mut Queue<Request>,
) {
    match result {
        Ok(()) => {
            d.watch_due = None;
            writers(d, env);
        }
        Err(Error::Busy) => {
            let until = env.now.saturating_add(Duration::from_secs(1));
            d.watch_due = Some(match d.watch_due {
                Some(when) => when.min(until),
                None => until,
            });
        }
        Err(error) => {
            let resource = match d.names.get(&owner.raw()) {
                Some(names) => names.first().cloned(),
                None => None,
            };
            emit(
                out,
                Request::ResourceRefused {
                    task: owner.raw(),
                    operation: b"watch resources",
                    resource,
                    cause: RefusalCause::Backend(error),
                },
            );
        }
    }
}

pub(super) fn writers(d: &mut Domain, env: &Env<Limits>) {
    for (name, hold) in &d.holds {
        if let Some(Writer::Run { .. } | Writer::Lost { .. }) = hold.writer
            && let Some(resource) = to_client(name, &env.limits)
        {
            skein_forge_client::step(
                &mut d.client,
                &child_env(env),
                skein_forge_client::Event::Writer { resource, taken: true },
                &mut d.client_out,
            );
        }
    }
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match d.watch_due {
        Some(when) if when <= env.now => {}
        Some(_) | None => return,
    }
    d.watch_due = None;
    let watches = plan(d, env, u64::MAX, &[]).expect("retained names fit configured bounds");
    child(d, env, skein_forge_client::Event::Keep { owner: Token::new(u64::MAX), watches }, out);
}

pub(super) fn read_failed(
    row: &ChangeRow,
    pending: &PendingStep,
    result: &Result<skein_forge::api::Answer, Error>,
    out: &mut Queue<Request>,
) -> bool {
    use skein_forge::api::{Answer, Read};
    let (operation, read): (&[u8], _) = match pending.stage {
        StepStage::Branch => (b"read branch", Read::Branch { branch: row.branch.clone() }),
        StepStage::Base => (b"read base", Read::Branch { branch: row.base.clone() }),
        StepStage::Pull => (
            b"read pull",
            match row.pull {
                Some(number) => Read::Pull { number },
                None => Read::PullFor { head: row.branch.clone(), base: row.base.clone() },
            },
        ),
        StepStage::Ci | StepStage::BaseCi | StepStage::Compare | StepStage::Reviews(_) => return false,
    };
    let error = match (pending.stage, result) {
        (StepStage::Branch | StepStage::Base, Ok(Answer::Commit(_)))
        | (StepStage::Pull, Ok(Answer::Pull(_)))
        | (StepStage::Branch | StepStage::Pull, Err(Error::Missing)) => return false,
        (_, Err(error)) => *error,
        (_, Ok(_)) => Error::InvalidAnswer,
    };
    crate::refusal::read(out, pending.task, operation, row.repository, &read, error);
    true
}
