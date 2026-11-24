//! Procedure wait reports (jig/docs/design/domain/engine/waits.md, 6).
use crate::{Ask, Bound, Core, Figures, Limits, ReportRefusal, Reported, ReportedBound, Request, WaitKind, WaitObject};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, List, Queue, Wall};

pub(crate) fn report(
    core: &mut Core,
    env: &Env<Limits>,
    connector: u16,
    task: u64,
    waits: List<Reported>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let Err(why) = admit(core, connector, task, &waits) {
        out.push(Request::Ask { connector, ask: Ask::WaitsRefused { task, why } });
        return out;
    }
    if waits.is_empty() {
        if core.reported_waits.remove(&task).is_none() {
            return out;
        }
    } else {
        if let Some(previous) = core.reported_waits.get(&task)
            && previous.as_ref() == waits.as_slice()
        {
            return out;
        }
        let mut report = waits.into_boxed();
        for wait in &mut report {
            core.set_wait_floor(wait.kind, env.wall);
            if let WaitObject::Procedure { things, .. } = &mut wait.object {
                *things = List::from_boxed(things.to_boxed());
            }
        }
        let _previous = core.reported_waits.insert(task, report).expect("bounded by live tasks");
    }
    core.touch_task_number(&env.limits, task);
    out
}

fn admit(core: &Core, connector: u16, task: u64, waits: &List<Reported>) -> Result<(), ReportRefusal> {
    let Some(row) = core.tasks.task(task) else { return Err(ReportRefusal::NotLive) };
    if let tasks::Phase::Ended(_) = row.phase {
        return Err(ReportRefusal::NotLive);
    }
    let tasks::Executor::Procedure { connector: owner, .. } = row.executor else {
        return Err(ReportRefusal::NotProcedure);
    };
    if owner != connector {
        return Err(ReportRefusal::WrongConnector);
    }
    if waits.len() > 4 {
        return Err(ReportRefusal::TooManyWaits);
    }
    for wait in waits {
        match wait.kind {
            WaitKind::Step | WaitKind::Room | WaitKind::SignOff => {
                let WaitObject::Procedure { things, .. } = &wait.object else {
                    return Err(ReportRefusal::WrongKindOrObject);
                };
                if things.len() > 4 {
                    return Err(ReportRefusal::TooManyThings);
                }
            }
            WaitKind::Turn => {
                let WaitObject::Task(_) = wait.object else { return Err(ReportRefusal::WrongKindOrObject) };
            }
            WaitKind::Party
            | WaitKind::Question
            | WaitKind::Dependency
            | WaitKind::Resource
            | WaitKind::Account
            | WaitKind::Host
            | WaitKind::Delegate
            | WaitKind::News => {
                return Err(ReportRefusal::WrongKindOrObject);
            }
        }
        if let ReportedBound::Figure(_) = wait.bound
            && figure(core.wait_limits.waits, wait.kind).is_none()
        {
            return Err(ReportRefusal::NoFigure);
        }
    }
    Ok(())
}

const fn figure(figures: Figures, kind: WaitKind) -> Option<Duration> {
    match kind {
        WaitKind::Step => Some(figures.step),
        WaitKind::Room => Some(figures.room),
        WaitKind::SignOff => Some(figures.sign_off),
        WaitKind::Party
        | WaitKind::Question
        | WaitKind::Dependency
        | WaitKind::Resource
        | WaitKind::Account
        | WaitKind::Host
        | WaitKind::Delegate
        | WaitKind::News
        | WaitKind::Turn => None,
    }
}

pub(super) fn bound(limits: &Limits, row: &tasks::TaskRecord, wait: &Reported, since: Wall) -> Bound {
    match wait.bound {
        ReportedBound::At { at, then } => Bound::At { at, then },
        ReportedBound::None(why) => Bound::None(why),
        ReportedBound::Figure(then) => {
            let figure = figure(limits.waits, wait.kind).expect("admitted figure kind");
            super::own::clock(since, figure, row.authority.budget.deadline, then)
        }
    }
}
