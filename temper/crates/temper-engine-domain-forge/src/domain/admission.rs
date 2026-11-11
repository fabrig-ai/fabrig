//! Complete watch admission and timed retry (temper/docs/design/domain/forge/keeping-up.md, 1, and temper/docs/design/domain/forge/change/README.md, 3).
use super::{
    ChangeRow, Domain, Env, Limits, Name, PendingStep, Request, StepStage, Stored, What, child, child_env, drain, emit,
    participating, to_client, valid_name,
};
use crate::RefusalCause;
use alloc::boxed::Box;
use skein_forge::api::Error;
use skein_forge_client::{Resource, Watch};
use skein_lib::{Duration, List, Map, Queue, Token};

pub(super) type Failed = (Option<Name>, RefusalCause);

pub(super) fn names(d: &mut Domain, env: &Env<Limits>, task: u64, resources: Box<[Name]>, out: &mut Queue<Request>) {
    let resources = watched_names(d, &resources);
    let watches = match plan(d, &env.limits, task, &resources) {
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

fn watched_names(d: &Domain, resources: &[Name]) -> Box<[Name]> {
    let mut names = List::with_capacity(u32::try_from(resources.len()).expect("bounded task names"));
    for name in resources {
        if !super::goal_run(d, name) {
            names.push(name.clone()).expect("filtered task names");
        }
    }
    names.into_boxed()
}

/// All holds are restored before filtering: store key order need not put them before names.
pub(super) fn restore_names(d: &mut Domain, out: &mut Queue<Request>) {
    let mut changed = List::with_capacity(d.names.len());
    for (task, resources) in &d.names {
        let filtered = watched_names(d, resources);
        if resources != &filtered {
            changed.push((*task, filtered)).expect("restored task names");
        }
    }
    for (task, resources) in changed.into_boxed() {
        d.names.insert(task, resources.clone()).expect("replaces restored names");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Names { task, resources } });
    }
}

fn plan(d: &Domain, limits: &Limits, task: u64, resources: &[Name]) -> Result<Box<[Watch]>, Failed> {
    let resource = resources.first().cloned();
    bound("forge.resources_per_task", limits.resources_per_task, resources.len(), resource.clone())?;
    if !d.names.contains_key(&task) && task != u64::MAX {
        bound("forge.tasks", limits.tasks, skein_lib::usize_of(d.names.len()).saturating_add(1), resource.clone())?;
    }
    let mut watches = Map::uncapped();
    for (owner, names) in &d.names {
        if *owner != task {
            add(d, limits, names, &mut watches)?;
        }
    }
    add(d, limits, resources, &mut watches)?;
    add(d, limits, crate::tiers::watched(d).as_slice(), &mut watches)?;
    finish(watches, limits, resource)
}

fn finish(watches: Map<Resource, Watch>, limits: &Limits, resource: Option<Name>) -> Result<Box<[Watch]>, Failed> {
    bound("forge.client.resources", limits.client.resources, skein_lib::usize_of(watches.len()), resource.clone())?;
    let mut repositories = Map::uncapped();
    for (resource, _) in &watches {
        repositories.insert(resource.repository, ()).expect("internal repository set");
    }
    bound("forge.client.repositories", limits.client.repositories, skein_lib::usize_of(repositories.len()), resource)?;
    let mut list = List::with_capacity(watches.len());
    for (_, watch) in &watches {
        list.push(watch.clone()).expect("counted watches");
    }
    Ok(list.into_boxed())
}

pub(super) struct SubscriptionProblem {
    pub words: &'static [u8],
    pub failure: Option<Failed>,
}

fn subscription_bound(
    words: &'static [u8],
    member: &'static str,
    configured: u32,
    needed: usize,
    resource: Option<Name>,
) -> SubscriptionProblem {
    SubscriptionProblem {
        words,
        failure: Some((
            resource,
            RefusalCause::Limit { member, configured: u64::from(configured), needed: skein_lib::len_u64(needed) },
        )),
    }
}

pub(super) fn subscription_problem(
    d: &Domain,
    limits: &Limits,
    subscription: &crate::Subscriber,
) -> Option<SubscriptionProblem> {
    use crate::Topic;
    let (repository, resource) = match &subscription.topic {
        Topic::Landings { repository, branch } => {
            let resource = Resource { repository: *repository, what: skein_forge_client::What::Branch(branch.clone()) };
            let name = super::from_client(&resource, limits);
            (*repository, name)
        }
        Topic::Ci { repository, .. } => (
            *repository,
            Some(Name { forge: repository.forge, repository: repository.repository, what: What::Repository }),
        ),
        Topic::Pull { repository, number } | Topic::Participation { repository, number } => (
            *repository,
            Some(Name { forge: repository.forge, repository: repository.repository, what: What::Pull(*number) }),
        ),
    };
    if let Some(tasks) = &subscription.goal_tasks
        && tasks.len() > skein_lib::usize_of(limits.issue_policy.plan_items)
    {
        return Some(subscription_bound(
            b"goal_tasks exceeds issue_policy.plan_items",
            "forge.issue_policy.plan_items",
            limits.issue_policy.plan_items,
            tasks.len(),
            resource,
        ));
    }
    if subscription.number == 0 {
        return Some(SubscriptionProblem { words: b"subscription number must be positive", failure: None });
    }
    if subscription.paths.len() > skein_lib::usize_of(limits.paths_per_subscription) {
        return Some(subscription_bound(
            b"paths exceeds paths_per_subscription",
            "forge.paths_per_subscription",
            limits.paths_per_subscription,
            subscription.paths.len(),
            resource,
        ));
    }
    match d.subscriptions.get(&(subscription.task, subscription.topic.clone())) {
        Some(existing) if existing.number != subscription.number => {
            return Some(SubscriptionProblem {
                words: b"duplicate topic; keep the existing subscription or unsubscribe it first",
                failure: None,
            });
        }
        None if d.subscriptions.len() == limits.subscriptions => {
            return Some(subscription_bound(
                b"forge subscription room exhausted: subscriptions",
                "forge.subscriptions",
                limits.subscriptions,
                skein_lib::usize_of(d.subscriptions.len()).saturating_add(1),
                resource,
            ));
        }
        Some(_) | None => {}
    }
    for path in &subscription.paths {
        if path.len() > skein_lib::usize_of(limits.name_bytes) {
            return Some(subscription_bound(
                b"path exceeds name_bytes",
                "forge.name_bytes",
                limits.name_bytes,
                path.len(),
                resource,
            ));
        }
    }
    if let Topic::Landings { branch, .. } = &subscription.topic
        && branch.len() > skein_lib::usize_of(limits.name_bytes)
    {
        return Some(subscription_bound(
            b"branch exceeds name_bytes",
            "forge.name_bytes",
            limits.name_bytes,
            branch.len(),
            resource,
        ));
    }
    if !d.repositories.contains_key(&repository) {
        return Some(SubscriptionProblem {
            words: b"repository is not adopted; select an adopted owner/name",
            failure: Some((resource, RefusalCause::Unadopted)),
        });
    }
    match &subscription.topic {
        Topic::Landings { branch, .. } => match landing_subscription_problem(d, limits, repository, branch) {
            Ok(()) => None,
            Err(failure) => {
                let words: &[u8] = match failure.1 {
                    RefusalCause::InvalidName => b"malformed branch: use nonempty slash-separated branch segments",
                    RefusalCause::Limit { .. } | RefusalCause::Unadopted | RefusalCause::Backend(_) => {
                        b"landing branch watch does not fit forge resource limits"
                    }
                };
                Some(SubscriptionProblem { words, failure: Some(failure) })
            }
        },
        Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } => None,
    }
}

/// Admit a landing branch against the same union as task names, before retaining its subscription.
pub(super) fn landing_subscription_problem(
    d: &Domain,
    limits: &Limits,
    repository: skein_forge::api::Repository,
    branch: &[u8],
) -> Result<(), Failed> {
    let resource = Resource { repository, what: skein_forge_client::What::Branch(Box::from(branch)) };
    let Some(name) = super::from_client(&resource, limits) else {
        return Err((None, RefusalCause::InvalidName));
    };
    let mut watches = Map::uncapped();
    for (_, names) in &d.names {
        add(d, limits, names, &mut watches)?;
    }
    add(d, limits, crate::tiers::watched(d).as_slice(), &mut watches)?;
    add(d, limits, core::slice::from_ref(&name), &mut watches)?;
    finish(watches, limits, Some(name))?;
    Ok(())
}

fn add(d: &Domain, limits: &Limits, names: &[Name], watches: &mut Map<Resource, Watch>) -> Result<(), Failed> {
    for name in names {
        validate(d, limits, name)?;
        if d.forge_ended(name.forge) || super::goal_run(d, name) {
            continue;
        }
        let resource = to_client(name, limits).expect("validated name");
        if let skein_forge_client::What::Branch(branch) = &resource.what {
            bound("forge.client.op_bytes", limits.client.op_bytes, branch.len(), Some(name.clone()))?;
        }
        let participating = participating(d, &resource);
        watches.insert(resource.clone(), Watch { resource, participating }).expect("internal watch set");
    }
    Ok(())
}

pub(super) fn validate(d: &Domain, limits: &Limits, name: &Name) -> Result<(), Failed> {
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
            heads(d, env);
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

/// A branch can acquire its first watch after its push or fresh reread.
/// Seed only an empty keep: an established baseline must still detect drift.
fn heads(d: &mut Domain, env: &Env<Limits>) {
    for (name, head) in &d.heads {
        let Some(hold) = d.holds.get(name) else { continue };
        if hold.drift.is_some() {
            continue;
        }
        let Some(resource) = to_client(name, &env.limits) else { continue };
        let Some(cached) = d.client.cached(&resource) else { continue };
        if cached.tip.is_none() {
            skein_forge_client::step(
                &mut d.client,
                &child_env(env),
                skein_forge_client::Event::Pushed { resource, commit: head.commit },
                &mut d.client_out,
            );
        }
    }
}

pub(super) fn writers(d: &mut Domain, env: &Env<Limits>) {
    for (name, _) in &d.holds {
        if super::branch_written(d, name, None)
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
    let mut entries = List::with_capacity(env.limits.entries);
    for (_, row) in &d.entries {
        if !row.held
            && let skein_forge_client::Condition::Merge { .. } = &row.entry.effect.condition
        {
            entries.push(row.entry.clone()).expect("bounded outbox");
        }
    }
    for entry in &entries {
        merge_writer(d, env, entry, true);
    }
    let mut updates = List::with_capacity(env.limits.entries);
    for (_, row) in &d.entries {
        if !row.held
            && let skein_forge_client::Condition::Update { .. } = row.entry.effect.condition
        {
            updates.push(row.entry.clone()).expect("bounded outbox");
        }
    }
    for entry in &updates {
        update_writer(d, env, entry, true);
    }
    let mut recovering = List::with_capacity(env.limits.changes);
    for (_, row) in &d.changes {
        if row.pending.is_none()
            && row.effect == temper_engine_domain_forge_change::EffectResult::None
            && row.effect_account.is_some()
            && super::matches_updating(&row.change.state)
        {
            recovering.push(row.clone()).expect("bounded changes");
        }
    }
    for row in &recovering {
        change_writer(d, env, row, true);
    }
}

/// An update owns forward moves by temper, not moves by another identity or rewrites.
pub(super) fn update_writer(d: &mut Domain, env: &Env<Limits>, entry: &skein_forge_client::Entry, taken: bool) {
    let skein_forge_client::Condition::Update { .. } = entry.effect.condition else { return };
    let Some(row) = d.changes.get(&entry.task).cloned() else { return };
    if taken && row.pending != Some(entry.number) {
        return;
    }
    let recovering = row.effect == temper_engine_domain_forge_change::EffectResult::None
        && super::matches_updating(&row.change.state);
    change_writer(d, env, &row, taken || recovering);
}

/// A let-go update keeps deciding whose forward head it read until containment settles.
pub(super) fn change_writer(d: &mut Domain, env: &Env<Limits>, row: &ChangeRow, taken: bool) {
    let resource = Resource { repository: row.repository, what: skein_forge_client::What::Branch(row.branch.clone()) };
    let Some(name) = super::from_client(&resource, &env.limits) else { return };
    let taken = taken || super::branch_written(d, &name, None);
    skein_forge_client::step(
        &mut d.client,
        &child_env(env),
        skein_forge_client::Event::Writer { resource, taken },
        &mut d.client_out,
    );
}

/// Keep a merge's held base visible as written until its outcome confirms the head.
pub(super) fn merge_writer(d: &mut Domain, env: &Env<Limits>, entry: &skein_forge_client::Entry, taken: bool) {
    let skein_forge_client::Condition::Merge { base } = &entry.effect.condition else { return };
    let resource = Resource { repository: entry.repository, what: skein_forge_client::What::Branch(base.clone()) };
    let Some(name) = super::from_client(&resource, &env.limits) else { return };
    if !d.holds.contains_key(&name) {
        return;
    }
    let taken = taken || super::branch_written(d, &name, Some(entry.number));
    skein_forge_client::step(
        &mut d.client,
        &child_env(env),
        skein_forge_client::Event::Writer { resource, taken },
        &mut d.client_out,
    );
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match d.watch_due {
        Some(when) if when <= env.now => {}
        Some(_) | None => return,
    }
    d.watch_due = None;
    let watches = plan(d, &env.limits, u64::MAX, &[]).expect("retained names fit configured bounds");
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
                Some(number) => Read::Pull { number, ci: pending.pull_ci },
                None => Read::PullFor { head: row.branch.clone(), base: row.base.clone() },
            },
        ),
        StepStage::Ci
        | StepStage::BaseCi
        | StepStage::Compare
        | StepStage::ContainsHead
        | StepStage::CatchUp
        | StepStage::Reviews(_) => {
            return false;
        }
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
