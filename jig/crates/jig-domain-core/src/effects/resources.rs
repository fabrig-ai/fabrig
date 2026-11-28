//! Connector reports drive named hold admission (domain/connectors.md, 3.3;
//! domain/tasks/holds.md, 2). An unreported batch retains its reply and bounded
//! payload without creating tasks or reserving funding. Current project roles
//! narrow every effect and workspace check (domain/authority.md, 6). Reports retry the
//! complete batch against current state. Uncommitted requests are retried by
//! their sender after restart; this table is never restored from the store.
use crate::{Core, Limits, effects::connector, routing::Work};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Time};

/// An uncommitted whole batch, awaiting connector reports.
#[derive(Hash, Debug)]
pub(crate) struct Batch {
    pub reply_to: ReplyTo,
    pub kind: Kind,
    pub members: Box<[tasks::New]>,
    pub deadline: Time,
}

/// The task admission continuation retained with a pending batch.
#[derive(Hash, Debug)]
pub(crate) enum Kind {
    /// A root, delegate or ordinary accepted proposal.
    Creation { creator: tasks::Party, maker: Option<u64>, places: Box<[tasks::DependencyPlace]> },
    /// An accepted result proposal belonging to a closing task.
    Result { proposer: u64, proposal: u64 },
}

impl Kind {
    fn by(&self) -> tasks::Party {
        match self {
            Kind::Creation { creator, .. } => *creator,
            Kind::Result { proposer, .. } => tasks::Party::Task(*proposer),
        }
    }
    pub(crate) fn event(self, reply_to: ReplyTo, batch: Box<[tasks::New]>) -> tasks::Event {
        match self {
            Kind::Creation { creator, maker, places } => {
                tasks::Event::Make { reads_project: false, reply_to, creator, maker, batch, places }
            }
            Kind::Result { proposer, proposal } => {
                tasks::Event::MakeResultFollowups { reads_project: false, reply_to, proposer, proposal, batch }
            }
        }
    }
}

fn reported(core: &Core, batch: &[tasks::New]) -> bool {
    for member in batch {
        for holding in &member.holdings {
            let name = match holding {
                tasks::Holding::Write { resource, .. } => resource,
                tasks::Holding::Slot { pool, .. } => pool,
            };
            if core.tasks.resource_hold(name).is_none() {
                return false;
            }
        }
    }
    true
}

pub(crate) fn defer(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    kind: Kind,
    batch: Box<[tasks::New]>,
    out: &mut Queue<tasks::Request>,
    deadline: Option<Time>,
) -> Option<tasks::Event> {
    let reported = reported(core, &batch);
    if !tasks::bounded_batch(&env.limits.tasks, &batch) {
        if reported {
            return Some(kind.event(reply_to, batch));
        }
        out.push(tasks::Request::Refused { reply_to, problem: tasks::Problem::new(None, tasks::Refusal::Busy) });
        return None;
    }
    if let Some(problem) = role_problem(core, kind.by(), &batch) {
        if let Some(capacity) = problem.capacity {
            core.tasks.remember_capacity(env.wall, capacity);
        }
        out.push(tasks::Request::Refused { reply_to, problem });
        return None;
    }
    let owner = reply_to.into_token();
    let reply_to = ReplyTo::new(owner);
    if let Err(cause) = core.tasks.pin_reports(owner, &batch) {
        let capacity = tasks::Capacity::batch(kind.by(), &batch, None, cause);
        core.tasks.remember_capacity(env.wall, capacity);
        out.push(tasks::Request::Refused {
            reply_to,
            problem: tasks::Problem::capacity(None, tasks::Refusal::Busy, capacity),
        });
        return None;
    }
    if reported {
        let mut batch = batch;
        normalize(core, &env.limits.tasks, &mut batch);
        return Some(kind.event(reply_to, batch));
    }
    if core.awaiting_resources_flights() == core.flight_tasks_capacity {
        let capacity = tasks::Capacity::batch(
            kind.by(),
            &batch,
            None,
            tasks::CapacityCause::CreationFlights { limit: core.flight_tasks_capacity },
        );
        core.tasks.remember_capacity(env.wall, capacity);
        out.push(tasks::Request::Refused {
            reply_to,
            problem: tasks::Problem::capacity(None, tasks::Refusal::Busy, capacity),
        });
        return None;
    }
    let deadline = deadline.unwrap_or(env.now.saturating_add(env.limits.tasks.hold_wait));
    let batch = Batch { reply_to, kind, members: batch, deadline };
    let _flight = core.fly(crate::routing::flight::Flight::AwaitingResources(batch));
    None
}

/// An application decision cannot park behind an unreported resource.
pub(crate) fn check_make(
    core: &Core,
    limits: &tasks::Limits,
    creator: tasks::Party,
    batch: &mut [tasks::New],
) -> Result<(), tasks::Refusal> {
    let reported = reported(core, batch);
    if !tasks::bounded_batch(limits, batch) {
        return if reported { Ok(()) } else { Err(tasks::Refusal::Busy) };
    }
    if let Some(problem) = role_problem(core, creator, batch) {
        return Err(problem.why);
    }
    if core.tasks.check_report_pins(batch).is_err() {
        return Err(tasks::Refusal::Busy);
    }
    if !reported {
        return Err(tasks::Refusal::Unreported);
    }
    normalize(core, limits, batch);
    Ok(())
}

fn role_problem(core: &Core, creator: tasks::Party, batch: &[tasks::New]) -> Option<tasks::Problem> {
    for member in batch {
        for holding in &member.holdings {
            let name = match holding {
                tasks::Holding::Write { resource, .. } => resource,
                tasks::Holding::Slot { pool, .. } => pool,
            };
            let role = core.resource_roles.get(&Key { project: member.project, name: name.clone() });
            if role == Some(&connector::ResourceRole::Unavailable) {
                return Some(tasks::Problem::new(Some(member.number), tasks::Refusal::ResourceUnavailable));
            }
            if role.is_none() && core.resource_roles_full {
                let capacity = tasks::Capacity::batch(
                    creator,
                    batch,
                    Some(member.number),
                    tasks::CapacityCause::ResourceReports { limit: core.resource_roles.capacity() },
                );
                return Some(tasks::Problem::capacity(
                    Some(member.number),
                    tasks::Refusal::ResourceUnavailable,
                    capacity,
                ));
            }
        }
    }
    None
}

fn normalize(core: &Core, limits: &tasks::Limits, batch: &mut [tasks::New]) {
    for member in batch {
        let mut holdings = List::with_capacity(limits.holdings);
        for holding in core::mem::replace(&mut member.holdings, Box::new([])) {
            let (name, kind) = match holding {
                tasks::Holding::Write { resource, kind } => (resource, kind),
                tasks::Holding::Slot { pool, kind } => (pool, kind),
            };
            match core.tasks.resource_hold(&name).expect("every resource reported") {
                tasks::HoldKind::Shared => {}
                tasks::HoldKind::Exclusive { .. } => {
                    holdings.push(tasks::Holding::Write { resource: name, kind }).expect("bounded holdings");
                }
                tasks::HoldKind::Pooled { .. } => {
                    holdings.push(tasks::Holding::Slot { pool: name, kind }).expect("bounded holdings");
                }
            }
        }
        member.holdings = holdings.into_boxed();
    }
}

pub(crate) fn resume(core: &mut Core, work: &mut Queue<Work>) {
    // Every ready batch is retried, even when an earlier one fails a core check.
    let mut ready = List::with_capacity(core.flight_tasks_capacity);
    for (id, flight) in &core.flights {
        let crate::routing::flight::Flight::AwaitingResources(batch) = flight else { continue };
        if reported(core, &batch.members) {
            ready.push(id.token()).expect("bounded waiting batches");
        }
    }
    for token in ready.into_boxed() {
        let Some(crate::routing::flight::Flight::AwaitingResources(batch)) =
            core.land_kind(token, crate::routing::flight::Kind::AwaitingResources)
        else {
            unreachable!("ready batch retained");
        };
        work.push(Work::Resources(batch));
    }
}

/// Earliest deadline among uncommitted batches (domain/tasks/holds.md, 2).
pub(crate) fn next_deadline(core: &Core) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, flight) in &core.flights {
        let crate::routing::flight::Flight::AwaitingResources(batch) = flight else { continue };
        next = Some(match next {
            Some(next) => next.min(batch.deadline),
            None => batch.deadline,
        });
    }
    next
}

/// Decide every expired batch, naming only members whose reports never came.
pub(crate) fn expire(core: &mut Core, env: &Env<Limits>) -> Queue<tasks::Request> {
    let mut expired = List::with_capacity(core.flight_tasks_capacity);
    for (id, flight) in &core.flights {
        let crate::routing::flight::Flight::AwaitingResources(batch) = flight else { continue };
        if batch.deadline <= env.now {
            expired.push(id.token()).expect("bounded waiting batches");
        }
    }
    let mut out = Queue::uncapped();
    for token in expired.into_boxed() {
        let Some(crate::routing::flight::Flight::AwaitingResources(batch)) =
            core.land_kind(token, crate::routing::flight::Kind::AwaitingResources)
        else {
            unreachable!("expired batch retained")
        };
        let mut missing = List::with_capacity(env.limits.tasks.batch);
        for member in &batch.members {
            if !reported(core, core::slice::from_ref(member)) {
                missing.push(member.number).expect("bounded batch");
            }
        }
        let task = missing.as_slice().first().copied().expect("parked batch lacks a report");
        let problem = tasks::Problem::naming(Some(task), tasks::Refusal::Unreported, missing.as_slice());
        let owner = batch.reply_to.into_token();
        core.tasks.unpin_reports(owner);
        out.push(tasks::Request::Refused { reply_to: ReplyTo::new(owner), problem });
    }
    out
}

/// A role belongs to one project, even when projects name the same object.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Key {
    pub project: u32,
    pub name: tasks::Name,
}

/// A connector's stored adoption, re-reported at restart (domain/connectors.md, 12 and 14).
#[derive(Clone, Copy, Hash, Debug)]
pub(crate) struct Adoption {
    pub role: connector::AdoptionRole,
    pub home: bool,
}

/// Remember current adoption facts. At capacity, unknown names fail closed;
/// a cached narrowing is never discarded to make room for another name.
pub(crate) fn role(core: &mut Core, limits: &Limits, project: u32, name: tasks::Name, role: connector::ResourceRole) {
    if !tasks::valid_name(&limits.tasks, &name) {
        return;
    }
    let key = Key { project, name };
    if role == connector::ResourceRole::Owned && !core.resource_roles_full {
        core.resource_roles.remove(&key);
        return;
    }
    if core.resource_roles.insert(key, role).is_err() {
        core.resource_roles_full = true;
    }
}

fn access(
    core: &Core,
    connector: u16,
    project: u32,
    name: &authority::Name,
    described: authority::EffectAccess,
) -> authority::EffectAccess {
    let key = Key { project, name: tasks::Name { connector, path: name.segments.clone() } };
    match core.resource_roles.get(&key) {
        Some(connector::ResourceRole::Participant) => match described {
            authority::EffectAccess::Owned | authority::EffectAccess::Participant => {
                authority::EffectAccess::Participant
            }
            authority::EffectAccess::Context | authority::EffectAccess::Unavailable => described,
        },
        Some(connector::ResourceRole::Context) => match described {
            authority::EffectAccess::Unavailable => described,
            authority::EffectAccess::Owned
            | authority::EffectAccess::Participant
            | authority::EffectAccess::Context => authority::EffectAccess::Context,
        },
        Some(connector::ResourceRole::Unavailable) => authority::EffectAccess::Unavailable,
        None if core.resource_roles_full => authority::EffectAccess::Unavailable,
        Some(connector::ResourceRole::Owned) | None => described,
    }
}

/// Apply current adoption without widening the connector's own fresh report.
pub(crate) fn effect(core: &Core, project: u32, mut effect: authority::Effect) -> authority::Effect {
    effect.access = access(core, effect.connector, project, &effect.name, effect.access);
    for resource in &mut effect.additional {
        resource.access = access(core, effect.connector, project, &resource.name, resource.access);
    }
    effect
}

/// Translate adoption's access to the task child's hold and writer checks.
pub(crate) fn task_access(role: connector::ResourceRole) -> tasks::ResourceAccess {
    match role {
        connector::ResourceRole::Owned | connector::ResourceRole::Participant => tasks::ResourceAccess::Writable,
        connector::ResourceRole::Context => tasks::ResourceAccess::Context,
        connector::ResourceRole::Unavailable => tasks::ResourceAccess::Unavailable,
    }
}

pub(crate) fn hold_kind(hold: connector::HoldKind) -> tasks::HoldKind {
    match hold {
        connector::HoldKind::Shared => tasks::HoldKind::Shared,
        connector::HoldKind::Exclusive { wait } => {
            tasks::HoldKind::Exclusive { taken: if wait { tasks::Taken::Waits } else { tasks::Taken::Refuses } }
        }
        connector::HoldKind::Pooled { wait, .. } => {
            tasks::HoldKind::Pooled { taken: if wait { tasks::Taken::Waits } else { tasks::Taken::Refuses } }
        }
    }
}

pub(crate) fn restore_report(
    core: &mut Core,
    env: &Env<Limits>,
    project: u32,
    name: tasks::Name,
    role: connector::ResourceRole,
    hold: connector::HoldKind,
) {
    let _restored = core.restore_resource_role(env, project, name.clone(), role);
    // Cache the hold rule as report state. The core's restored pool rows,
    // not a connector's loading report, supply the durable slot counts.
    let mut out = Queue::uncapped();
    let event = tasks::Event::Resource { name, hold: hold_kind(hold) };
    crate::routing::step::step_tasks(core, env, event, &mut out);
    assert!(out.is_empty(), "restored resource reports decide nothing");
}

impl Core {
    /// Restore a connector's durable adoption before tasks open. This restores
    /// report state only; it cannot decide a hold or stop a live activation.
    #[must_use]
    pub fn restore_resource_role(
        &mut self,
        env: &Env<Limits>,
        project: u32,
        name: tasks::Name,
        role: connector::ResourceRole,
    ) -> bool {
        self.start_input();
        {
            if self.start_phase() != Some(crate::start::Phase::RestoreConnector { connector: name.connector }) {
                return false;
            }
        }
        if !tasks::valid_name(&env.limits.tasks, &name) {
            return false;
        }
        self::role(self, &env.limits, project, name.clone(), role);
        let mut out = Queue::uncapped();
        crate::routing::step::step_tasks(
            self,
            env,
            tasks::Event::ResourceAccess { project, name, access: task_access(role) },
            &mut out,
        );
        out.is_empty() && !self.resource_roles_full
    }
}
