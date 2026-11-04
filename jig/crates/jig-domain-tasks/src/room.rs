//! Durable live-room conditions and refusal order (jig/docs/design/domain/connectors.md, 6.2 and 6.4).
use crate::domain::{publish, record, task_mut, wake_procedure};
use crate::{
    Active, CapacityAction, CapacityCause, Domain, Executor, Limits, Phase, Problem, Refusal, Request, TaskRecord,
};
use skein_lib::{Env, Queue};

/// The live-task count a procedure batch waits for (connectors.md, 6.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LiveRoom {
    Deployment,
    Project(u32),
}

/// A durable live-room condition in global refusal order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RoomWait {
    pub room: LiveRoom,
    pub order: u64,
}

impl Problem {
    /// Only a batch's deployment or project live-task count frees when tasks end.
    #[must_use]
    pub fn freeing_count(&self) -> Option<LiveRoom> {
        let capacity = self.capacity?;
        match capacity.action {
            CapacityAction::Batch { .. } => match capacity.cause {
                CapacityCause::Tasks { .. } => {
                    if self.why == Refusal::Live {
                        Some(LiveRoom::Deployment)
                    } else {
                        None
                    }
                }
                CapacityCause::ProjectTasks { .. } => {
                    if self.why == Refusal::Project {
                        match capacity.project {
                            Some(project) => Some(LiveRoom::Project(project)),
                            None => None,
                        }
                    } else {
                        None
                    }
                }
                CapacityCause::PlacedHolds { .. }
                | CapacityCause::ProjectPlacedHolds { .. }
                | CapacityCause::PartyPlacedHolds { .. }
                | CapacityCause::Exemptions { .. }
                | CapacityCause::CreationFlights { .. }
                | CapacityCause::ResourceReports { .. }
                | CapacityCause::Tree { .. }
                | CapacityCause::Delegates { .. }
                | CapacityCause::Inbox { .. }
                | CapacityCause::Holdings { .. }
                | CapacityCause::HoldWaiters { .. }
                | CapacityCause::Proposals { .. }
                | CapacityCause::References { .. }
                | CapacityCause::Subscriptions { .. }
                | CapacityCause::Funders { .. }
                | CapacityCause::Writers { .. } => None,
            },
            CapacityAction::Proposal { .. }
            | CapacityAction::PersonProposal { .. }
            | CapacityAction::Message { .. }
            | CapacityAction::Source { .. }
            | CapacityAction::Introduce { .. }
            | CapacityAction::Subscription { .. }
            | CapacityAction::Effect { .. }
            | CapacityAction::Claim { .. }
            | CapacityAction::Release
            | CapacityAction::Place { .. }
            | CapacityAction::Move => None,
        }
    }
}

pub(crate) fn wait(domain: &mut Domain, env: &Env<Limits>, number: u64, room: LiveRoom, out: &mut Queue<Request>) {
    let mut last = 0;
    for (other, _) in &domain.names {
        if let Some(wait) = record(domain, *other).expect("live name").room_wait {
            last = last.max(wait.order);
        }
    }
    let order = last.checked_add(1).expect("live room refusal order fits u64");
    let task = task_mut(domain, number).expect("fenced procedure");
    task.record.room_wait = Some(RoomWait { room, order });
    task.record.phase = Phase::Active(Active::Idle);
    task.procedure_offered = None;
    publish(domain, env, number, out);
}

pub(crate) fn freed(domain: &mut Domain, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    let mut oldest: Option<(u64, u64)> = None;
    for (number, _) in &domain.names {
        let Some(wait) = record(domain, *number).expect("live name").room_wait else { continue };
        let serves = match wait.room {
            LiveRoom::Deployment => true,
            LiveRoom::Project(wanted) => wanted == project,
        };
        let earlier = match oldest {
            Some((_, order)) => wait.order < order,
            None => true,
        };
        if serves && earlier {
            oldest = Some((*number, wait.order));
        }
    }
    if let Some((number, _)) = oldest {
        wake_procedure(domain, env, number, out);
    }
}

pub(crate) fn refresh(domain: &mut Domain, number: u64) {
    let row = &mut task_mut(domain, number).expect("published live task").record;
    if row.phase != Phase::Active(Active::Idle) || row.grace.is_some() {
        row.room_wait = None;
    }
}

pub(crate) fn valid(domain: &Domain, task: &TaskRecord) -> bool {
    let Some(wait) = task.room_wait else { return true };
    match task.executor {
        Executor::Procedure { .. } => {}
        Executor::Agent { .. } | Executor::Person(_) | Executor::Job { .. } => return false,
    }
    if wait.order == 0 || task.phase != Phase::Active(Active::Idle) || task.grace.is_some() {
        return false;
    }
    match wait.room {
        LiveRoom::Project(project) if project != task.project => return false,
        LiveRoom::Deployment | LiveRoom::Project(_) => {}
    }
    for (other, _) in &domain.names {
        if *other != task.number
            && let Some(other) = record(domain, *other).expect("live name").room_wait
            && other.order == wait.order
        {
            return false;
        }
    }
    true
}
