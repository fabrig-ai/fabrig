//! Durable, exclusive writer slots for held resources (domain/tasks/holds.md, 3).
//! The hub knows only opaque names and fences; connectors own fresh reads.
use crate::domain::{Domain, record, refused};
use crate::{Limits, Name, Party, Refusal, Request, Stored, Writer, WriterSlot};
use skein_lib::{Env, Queue, ReplyTo};

fn covers(domain: &Domain, task: u64, resource: &Name, limit: u32) -> bool {
    let Some(holder) = crate::holds::writer_holder(domain, resource) else { return false };
    below(domain, task, holder, limit)
}

fn below(domain: &Domain, task: u64, ancestor: u64, limit: u32) -> bool {
    let mut current = Some(task);
    for _ in 0..limit {
        let Some(at) = current else { break };
        if at == ancestor {
            return true;
        }
        current = match record(domain, at) {
            Some(row) => match row.requester {
                Party::Task(parent) => Some(parent),
                Party::Person(_) | Party::Deployment { .. } => None,
            },
            None => None,
        };
    }
    false
}

fn room(domain: &Domain, count: u32) -> bool {
    match domain.writers.len().checked_add(count) {
        Some(used) => used <= domain.writers.capacity(),
        None => false,
    }
}

fn take(domain: &mut Domain, resource: Name, writer: Writer, out: &mut Queue<Request>) {
    let current = domain.writers.get(&resource);
    assert!(current.is_none(), "take writer slot for {resource:?}: {writer:?} needs a vacant slot, found {current:?}");
    let Some(number) = domain.next_writer.checked_add(1) else {
        unreachable!("take writer slot for {resource:?}: next slot number must fit for {writer:?}");
    };
    let slot = WriterSlot { number, resource: resource.clone(), writer, lost: false };
    let inserted = domain.writers.insert(resource, slot.clone());
    assert!(inserted == Ok(None), "take writer slot for {:?}: {writer:?} must have admitted room", slot.resource);
    domain.next_writer = number;
    out.push(Request::Save { record: Stored::Writer(slot) });
}

fn held<'a>(domain: &'a mut Domain, resource: &Name, writer: Writer) -> &'a mut WriterSlot {
    let Some(slot) = domain.writers.get_mut(resource) else {
        unreachable!("writer slot for {resource:?}: expected {writer:?}, found vacant");
    };
    assert!(slot.writer == writer, "writer slot for {resource:?}: expected {writer:?}, found {:?}", slot.writer);
    slot
}

fn free(domain: &mut Domain, env: &Env<Limits>, resource: &Name, writer: Writer, out: &mut Queue<Request>) {
    let number = held(domain, resource, writer).number;
    domain.writers.remove(resource);
    out.push(Request::Erase { key: crate::Key::Writer(number) });
    // A hold handed downward can wait for its ancestor's run writer.
    // Ordinary delegate results and effect settlements have their own wake
    // routes, after the connector has received their outcome.
    if let Some(holder) = crate::holds::writer_holder(domain, resource) {
        match writer {
            Writer::Run { task, .. } => {
                if holder != task && below(domain, holder, task, env.limits.depth.saturating_add(1)) {
                    crate::domain::wake_procedure(domain, env, holder, out);
                }
            }
            Writer::Effect { .. } => {}
        }
    }
}

pub(crate) enum ClaimRefusal {
    Invalid,
    Waiting(Name),
    Full(crate::Capacity),
}

pub(crate) fn claim(
    domain: &mut Domain,
    limits: &Limits,
    task: u64,
    attempt: u64,
    writes: &[Name],
    out: &mut Queue<Request>,
) -> Result<(), ClaimRefusal> {
    let Ok(count) = u32::try_from(writes.len()) else { return Err(ClaimRefusal::Invalid) };
    if count > limits.holdings {
        return Err(ClaimRefusal::Invalid);
    }
    let Some(row) = record(domain, task) else { return Err(ClaimRefusal::Invalid) };
    for (index, resource) in writes.iter().enumerate() {
        let mut duplicate = false;
        for prior in writes.iter().take(index) {
            if prior == resource {
                duplicate = true;
                break;
            }
        }
        if !crate::holds::valid_name(limits, resource) {
            return Err(ClaimRefusal::Invalid);
        }
        if crate::holds::available_access(domain, row.project, resource) != crate::ResourceAccess::Writable {
            return Err(ClaimRefusal::Invalid);
        }
        if !covers(domain, task, resource, limits.depth.saturating_add(1)) || duplicate {
            return Err(ClaimRefusal::Invalid);
        }
        if domain.writers.contains_key(resource) {
            return Err(ClaimRefusal::Waiting(resource.clone()));
        }
    }
    if !room(domain, count) {
        return Err(ClaimRefusal::Full(crate::Capacity {
            project: Some(row.project),
            by: Party::Task(task),
            action: crate::CapacityAction::Claim { attempt },
            cause: crate::CapacityCause::Writers { limit: domain.writers.capacity() },
        }));
    }
    let owner = Writer::Run { task, attempt };
    for resource in writes {
        take(domain, resource.clone(), owner, out);
    }
    Ok(())
}

pub(crate) fn answered(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    lost: bool,
    out: &mut Queue<Request>,
) {
    let writer = Writer::Run { task, attempt };
    let mut resources = skein_lib::List::with_capacity(domain.writers.capacity());
    // Runs that wrote nothing have no slots to mark or free.
    for (resource, slot) in &domain.writers {
        if slot.writer == writer {
            let added = resources.push(resource.clone());
            assert!(added.is_ok(), "collect writer slot for {resource:?}: {writer:?} must fit in the slot list");
        }
    }
    for resource in &resources {
        if lost {
            let slot = held(domain, resource, writer);
            slot.lost = true;
            out.push(Request::Save { record: Stored::Writer(slot.clone()) });
        } else {
            free(domain, env, resource, writer, out);
        }
    }
}

pub(crate) fn read_afresh(domain: &mut Domain, env: &Env<Limits>, resource: &Name, out: &mut Queue<Request>) {
    let Some(WriterSlot { writer: writer @ Writer::Run { .. }, lost: true, .. }) = domain.writers.get(resource) else {
        return;
    };
    free(domain, env, resource, *writer, out);
}

/// Keep a run writer's task and hold owner until its answer or lost-attempt fresh read.
pub(crate) fn closing_blocked(domain: &Domain, task: u64) -> bool {
    for (resource, slot) in &domain.writers {
        match slot.writer {
            Writer::Run { task: owner, .. } => {
                if owner == task || crate::holds::writer_holder(domain, resource) == Some(task) {
                    return true;
                }
            }
            Writer::Effect { .. } => {}
        }
    }
    false
}

/// Pure whole-set preflight; a repeated procedure entry keeps its own slot.
pub(crate) fn effect_ready(domain: &Domain, limits: &Limits, task: u64, resource: &Name, entry: Option<u64>) -> bool {
    if !crate::holds::valid_name(limits, resource) || !covers(domain, task, resource, limits.depth.saturating_add(1)) {
        return false;
    }
    match domain.writers.get(resource) {
        Some(slot) => match slot.writer {
            Writer::Effect { entry: current } => entry == Some(current),
            Writer::Run { .. } => false,
        },
        None => room(domain, 1),
    }
}

pub(crate) fn effect_in_flight(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    task: u64,
    resource: Name,
    entry: u64,
    out: &mut Queue<Request>,
) {
    let limits = &env.limits;
    if !crate::holds::valid_name(limits, &resource)
        || !covers(domain, task, &resource, limits.depth.saturating_add(1))
        || entry == 0
    {
        refused(to, Some(task), Refusal::Holds, out);
    } else if let Some(slot) = domain.writers.get(&resource) {
        if slot.writer == (Writer::Effect { entry }) {
            out.push(Request::Done { reply_to: to });
        } else {
            out.push(Request::WriterWaiting { reply_to: to, task: Some(task), resource });
        }
    } else if !room(domain, 1) {
        let Some(row) = record(domain, task) else {
            unreachable!("effect {entry} on {resource:?}: the covered task {task} must be live");
        };
        let project = row.project;
        crate::capacity::refuse(
            domain,
            env,
            to,
            Some(task),
            Refusal::Holds,
            crate::Capacity {
                project: Some(project),
                by: Party::Task(task),
                action: crate::CapacityAction::Effect { entry },
                cause: crate::CapacityCause::Writers { limit: domain.writers.capacity() },
            },
            out,
        );
    } else {
        take(domain, resource, Writer::Effect { entry }, out);
        out.push(Request::Done { reply_to: to });
    }
}

pub(crate) fn effect_settled(
    domain: &mut Domain,
    env: &Env<Limits>,
    resource: &Name,
    entry: u64,
    out: &mut Queue<Request>,
) {
    let writer = Writer::Effect { entry };
    // This entry may have no slot here, including after an earlier settlement.
    if let Some(slot) = domain.writers.get(resource)
        && slot.writer == writer
    {
        free(domain, env, resource, writer, out);
    }
}

pub(crate) fn restore(domain: &mut Domain, limits: &Limits, slot: WriterSlot) -> Result<(), crate::RestoreCause> {
    assert!(!domain.writers.contains_key(&slot.resource), "restore Writer({}): duplicate resource index", slot.number);
    crate::stored::fit::name(&slot.resource, limits)?;
    crate::stored::fit::room(crate::RestoreBound::Writers, domain.writers.len(), domain.writers.capacity())?;
    domain.next_writer = domain.next_writer.max(slot.number);
    assert!(domain.writers.insert(slot.resource.clone(), slot) == Ok(None), "restored writer admitted");
    Ok(())
}

pub(crate) fn link(domain: &Domain) -> Result<(), crate::RestoreFailure> {
    for (_, slot) in &domain.writers {
        let key = crate::Key::Writer(slot.number);
        match slot.writer {
            Writer::Run { task, .. } => {
                if record(domain, task).is_none() {
                    return Err(crate::RestoreFailure {
                        key,
                        cause: crate::RestoreCause::Link { target: crate::Key::Live(task) },
                    });
                }
            }
            Writer::Effect { .. } => {}
        }
        if crate::holds::writer_holder(domain, &slot.resource).is_none() {
            return Err(crate::RestoreFailure {
                key,
                cause: crate::RestoreCause::Holder { resource: slot.resource.clone() },
            });
        }
    }
    Ok(())
}
