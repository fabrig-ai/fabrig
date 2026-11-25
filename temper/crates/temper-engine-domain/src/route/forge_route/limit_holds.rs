//! Durable waits for named limits (temper/docs/design/protocol/configuration/limit-settings.md, 6).
use super::{Domain, Env, Limits, Work, forge, holds, tasks};
use crate::{Decision, Key, Record, Write};
use alloc::boxed::Box;
use skein_lib::{List, Map};

/// Only the account is durable; observations and queued releases belong to this process.
#[derive(Hash, Debug)]
pub(crate) struct Waiting {
    row: forge::LimitHold,
    observed: Option<u64>,
    queued: bool,
}

impl Domain {
    /// Kept limit holds, oldest first, for the deployment's Needs-you read.
    #[must_use]
    pub fn forge_limit_holds(&self) -> Box<[forge::LimitHold]> {
        let mut ordered = Map::with_capacity(self.forge_limit_holds.len());
        for (_, waiting) in &self.forge_limit_holds {
            let row = &waiting.row;
            ordered.insert((row.order, row.task), row.task).expect("one limit hold per task");
        }
        let mut rows = List::with_capacity(ordered.len());
        for (_, task) in &ordered {
            rows.push(self.forge_limit_holds.get(task).expect("ordered held task").row.clone()).expect("counted holds");
        }
        rows.into_boxed()
    }

    /// The retained limit account only while its exact task hold still stands.
    /// Inspection uses the same ownership test as release (testing/engine-checks.md, 2.2).
    #[must_use]
    pub fn inspection_limit_hold(&self, task: u64) -> Option<&forge::LimitHold> {
        let waiting = self.forge_limit_holds.get(&task)?;
        if current(self, &waiting.row) { Some(&waiting.row) } else { None }
    }

    /// End every wait on a raised setting; otherwise give freed room oldest first.
    /// Each queued release checks the exact hold revision before going through jig's control path.
    pub fn end_forge_limit_holds(&mut self, member: &[u8], configured: u64) {
        self.changed();
        self.settled = None;
        let mut rooms = Map::with_capacity(self.forge_limit_holds.len());
        for row in self.forge_limit_holds() {
            if row.member.as_ref() != member {
                continue;
            }
            if !current(self, &row) {
                queue(self, &row);
                continue;
            }
            if configured > row.configured {
                queue(self, &row);
                continue;
            }
            let counted = self.forge.limit_usage(&row.set);
            let available = match rooms.get(&row.set) {
                Some(available) => *available,
                None => match counted {
                    Some(used) => configured.saturating_sub(used),
                    None => configured,
                },
            };
            if self.forge_limit_holds.get(&row.task).expect("kept hold").queued {
                if counted.is_some() {
                    rooms.insert(row.set.clone(), available.saturating_sub(row.room)).expect("one set per held task");
                }
                continue;
            }
            if row.room > available {
                continue;
            }
            if counted.is_some() {
                rooms
                    .insert(row.set.clone(), available.checked_sub(row.room).expect("selected hold fits"))
                    .expect("one set per held task");
            }
            queue(self, &row);
        }
    }
}

fn queue(domain: &mut Domain, row: &forge::LimitHold) {
    let waiting = domain.forge_limit_holds.get_mut(&row.task).expect("kept hold");
    if !waiting.queued {
        waiting.queued = true;
        domain.work.push(Work::ForgeLimitRelease { task: row.task, revision: row.revision });
    }
}

pub(super) fn hold(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    cause: forge::RefusalCause,
    set: forge::LimitSet,
    words: Box<[u8]>,
) {
    let forge::RefusalCause::Limit { member, configured, needed } = cause else { return };
    let room = match domain.forge.limit_usage(&set) {
        Some(used) => needed.saturating_sub(used).max(1),
        None => needed,
    };
    domain.work.push(Work::ForgeLimitHold {
        row: forge::LimitHold {
            task,
            member: Box::from(member.as_bytes()),
            configured,
            needed,
            revision: 0,
            at: env.wall,
            order: 0,
            room,
            set,
        },
        words,
    });
}

pub(crate) fn apply_hold(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    mut row: forge::LimitHold,
    words: Box<[u8]>,
) {
    let Some(task) = domain.core.tasks.task(row.task) else { return };
    if task.placed.is_some() {
        return;
    }
    if let tasks::Phase::Held { .. } = task.phase {
        let Some(waiting) = domain.forge_limit_holds.get(&row.task) else { return };
        if waiting.row.revision != task.revisions.hold {
            return;
        }
    }
    let source = holds::source(domain, None, words);
    let requests = super::step_core(
        domain,
        env,
        jig_domain_core::Event::Tasks(tasks::Event::HoldFrom {
            task: row.task,
            why: tasks::Hold::ResourceUnavailable,
            source: Box::new(source.clone()),
        }),
    );
    super::super::route_core_requests(domain, env, decision, requests);
    let Some(task) = domain.core.tasks.task(row.task) else { return };
    let tasks::Phase::Held { source: Some(held), .. } = &task.phase else { return };
    if held.as_ref() != &source {
        return;
    }
    row.revision = task.revisions.hold;
    if let Some(waiting) = domain.forge_limit_holds.get(&row.task)
        && waiting.row.revision == row.revision
    {
        return;
    }
    let mut order = 0;
    for (_, waiting) in &domain.forge_limit_holds {
        order = order.max(waiting.row.order);
    }
    row.order = order.checked_add(1).expect("limit hold order space");
    let observed = domain.forge.limit_usage(&row.set);
    domain
        .forge_limit_holds
        .insert(row.task, Waiting { row: row.clone(), observed, queued: false })
        .expect("one kept limit hold per live jig task");
    super::super::save(
        decision,
        &env.limits,
        Write::Save(Record::Forge { row: Box::new(forge::Stored::LimitHold(row)) }),
    );
}

pub(crate) fn restore(domain: &mut Domain, row: forge::LimitHold) {
    domain
        .forge_limit_holds
        .insert(row.task, Waiting { row, observed: None, queued: false })
        .expect("restored limit holds bounded by live jig tasks");
}

fn current(domain: &Domain, row: &forge::LimitHold) -> bool {
    let Some(task) = domain.core.tasks.task(row.task) else { return false };
    if task.revisions.hold != row.revision {
        return false;
    }
    if let tasks::Phase::Held { why: tasks::Hold::ResourceUnavailable, source: Some(source), .. } = &task.phase
        && let tasks::HoldSource::Account { connector, .. } = source.as_ref()
    {
        return *connector == domain.config.forge_connector;
    }
    false
}

fn forget(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, task: u64) {
    if domain.forge_limit_holds.remove(&task).is_some() {
        super::super::save(decision, &env.limits, Write::Erase(Key::Forge(forge::Key::LimitHold(task))));
    }
}

pub(crate) fn reconcile(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    if domain.start_phase().is_some() || !domain.forge.fresh_reads_in() {
        return;
    }
    for row in domain.forge_limit_holds() {
        if !current(domain, &row) {
            forget(domain, env, decision, row.task);
        }
    }
}

pub(crate) fn restart(domain: &mut Domain) {
    for row in domain.forge_limit_holds() {
        queue(domain, &row);
    }
}

pub(crate) fn release(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, task: u64, revision: u64) {
    let Some(waiting) = domain.forge_limit_holds.get(&task) else { return };
    if waiting.row.revision != revision {
        return;
    }
    let release = current(domain, &waiting.row);
    if release {
        let project = domain.core.tasks.task(task).expect("current held task").project;
        let requests = super::step_core(
            domain,
            env,
            jig_domain_core::Event::Tasks(tasks::Event::Release {
                reply_to: super::super::internal(u64::MAX),
                by: tasks::Party::Deployment { project },
                operation: tasks::Release { task, again: None, message: None },
            }),
        );
        super::super::route_core_requests(domain, env, decision, requests);
        // A placed hold can have arrived since selection. Keep the wait if jig refused it.
        if let Some(waiting) = domain.forge_limit_holds.get_mut(&task) {
            waiting.queued = false;
        }
    } else {
        forget(domain, env, decision, task);
    }
}

fn figure(limits: &forge::Limits, set: &forge::LimitSet) -> Option<u64> {
    use forge::LimitSet;
    Some(u64::from(match set {
        LimitSet::Operation => return None,
        LimitSet::Entries => limits.entries,
        LimitSet::Holds | LimitSet::DesignWriters(_) => limits.holds,
        LimitSet::Tasks => limits.tasks,
        LimitSet::Changes | LimitSet::Steps => limits.changes,
        LimitSet::Subscriptions => limits.subscriptions,
        LimitSet::Watches | LimitSet::Heads => limits.client.resources,
        LimitSet::Repositories => limits.client.repositories,
    }))
}

pub(super) fn set(member: &str, operation: &[u8], resource: Option<&forge::Name>) -> forge::LimitSet {
    use forge::LimitSet;
    if member == "forge.entries" {
        return LimitSet::Entries;
    }
    if member == "forge.holds" {
        if operation == b"claim design"
            && let Some(name) = resource
        {
            return LimitSet::DesignWriters(name.clone());
        }
        return LimitSet::Holds;
    }
    if member == "forge.tasks" {
        return LimitSet::Tasks;
    }
    if member == "forge.changes" {
        return if operation == b"step change" { LimitSet::Steps } else { LimitSet::Changes };
    }
    if member == "forge.subscriptions" {
        return LimitSet::Subscriptions;
    }
    if member == "forge.client.resources" {
        return LimitSet::Watches;
    }
    if member == "forge.client.repositories" {
        return LimitSet::Repositories;
    }
    LimitSet::Operation
}

pub(super) fn room_freed(domain: &mut Domain, limits: &forge::Limits) {
    if domain.start_phase().is_some() || !domain.forge.fresh_reads_in() {
        return;
    }
    let mut freed = Map::with_capacity(domain.forge_limit_holds.len());
    for row in domain.forge_limit_holds() {
        if let Some(used) = domain.forge.limit_usage(&row.set) {
            let waiting = domain.forge_limit_holds.get_mut(&row.task).expect("kept hold");
            if let Some(before) = waiting.observed
                && used < before
            {
                freed
                    .insert(waiting.row.member.clone(), waiting.row.set.clone())
                    .expect("at most one limit per held task");
            }
            waiting.observed = Some(used);
        }
    }
    for (member, set) in &freed {
        if let Some(configured) = figure(limits, set) {
            domain.end_forge_limit_holds(member, configured);
        }
    }
}
