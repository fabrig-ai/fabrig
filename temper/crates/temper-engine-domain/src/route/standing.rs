//! Move a leaver's standing roles after a roles save, and retry each period
//! (temper/docs/design/domain/agent/standing.md, 3).
use crate::{Decision, Delivery, Domain, Limits, boundary::Work};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue};

/// The saved answer names the authenticated asker, even if they left the roster
/// in this save. The continuation crosses the roles decision's commit barrier.
pub(crate) fn saved(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, requests: &Queue<core::Request>) {
    for request in requests {
        let core::Request::Write(core::Write::Save(core::Record::People(people::Stored::Roles { project, .. }))) = request
        else {
            continue;
        };
        let mut leaver = false;
        for task in domain.core.tasks.view_tasks() {
            if orphan(domain, task.number, *project) {
                leaver = true;
                break;
            }
        }
        if !leaver {
            continue;
        }
        let mut asker = None;
        for request in requests {
            if let core::Request::Write(core::Write::Save(core::Record::People(people::Stored::Answer {
                key,
                outcome: people::Outcome::RolesSet { project: saved },
                ..
            }))) = request
                && saved == project
            {
                asker = Some(key.person);
                break;
            }
        }
        super::emit(decision, &env.limits, Delivery::MoveStanding { project: *project, owner: owner(domain, *project, asker) });
    }
}

fn owner(domain: &Domain, project: u32, preferred: Option<u64>) -> Option<u64> {
    let mut lowest: Option<u64> = None;
    for holding in domain.core.people.project_members(project)? {
        if holding.role == people::Role::Owner {
            if preferred == Some(holding.person) {
                return preferred;
            }
            lowest = Some(match lowest {
                Some(number) => number.min(holding.person),
                None => holding.person,
            });
        }
    }
    lowest
}

fn orphan(domain: &Domain, task: u64, project: u32) -> bool {
    let Some(row) = domain.core.tasks.task(task) else { return false };
    if row.project != project || row.flags.ended_by != tasks::EndedBy::Standing {
        return false;
    }
    match row.requester {
        tasks::Party::Person(person) => domain.core.people.role(person, project).is_none(),
        tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
    }
}

/// Jig owns admission and the atomic funding and question transfer. A refused
/// move leaves the task for the next roles save or project period.
pub(crate) fn move_roles(domain: &mut Domain, project: u32, preferred: Option<u64>) {
    let Some(person) = owner(domain, project, preferred) else { return };
    let period = domain.core.period(project);
    let (_, pool) = domain.core.person_pool(project, person);
    let Some(funding) = domain.core.tasks.funding(tasks::Funder::Period { project, period }) else { return };
    let period_budget = funding.numbers.budget;
    for task in domain.core.tasks.view_tasks() {
        if orphan(domain, task.number, project) {
            domain.work.push(Work::Tasks(tasks::Event::Move {
                reply_to: super::internal(u64::MAX),
                person,
                period,
                pool_budget: pool.budget,
                period_budget,
                operation: tasks::Move { task: task.number, reason: Box::new([]) },
            }));
        }
    }
}
