//! Period renewal for standing root tasks (domain/tasks/standing.md, section 4;
//! domain/tasks/README.md, section 5; domain/authority.md, section 7).
//! Old delegates keep their old-period source.
use crate::domain::{Domain, publish, record, task_mut};
use crate::{EndedBy, Executor, Funder, FundingRecord, Hold, Limits, Numbers, Phase, Request, TaskRecord};
use skein_lib::{Env, List, Queue};

pub(crate) fn eligible(task: &TaskRecord) -> bool {
    let standing = match task.executor {
        Executor::Procedure { .. } => !task.subscriptions.is_empty(),
        Executor::Agent { .. } => task.flags.ended_by == EndedBy::Standing,
        Executor::Person(_) | Executor::Job { .. } => false,
    };
    let live = match task.phase {
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } => true,
        Phase::Closing(_) | Phase::Ended(_) => false,
    };
    standing && live && task.root == task.number && task.recurring.is_none()
}

pub(crate) fn after_run(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    let task = record(domain, number).expect("answered task live");
    if task.flags.ended_by != EndedBy::Standing {
        return;
    }
    let mut latest = 0;
    for (source, _) in &domain.funding {
        match *source {
            Funder::Period { project, period } if project == task.project => latest = latest.max(period),
            Funder::Task(_) | Funder::Pool { .. } | Funder::Recurring { .. } | Funder::Period { .. } => {}
        }
    }
    renew(domain, env, number, latest, out);
}

#[expect(
    clippy::too_many_lines,
    reason = "renewal preflights then transfers one period's funding and releases its budget hold atomically"
)]
pub(crate) fn renew(domain: &mut Domain, env: &Env<Limits>, number: u64, period: u64, out: &mut Queue<Request>) {
    if !domain.ready() || period == 0 {
        return;
    }
    let Some(task) = record(domain, number) else { return };
    if !eligible(task) {
        return;
    }
    let project = task.project;
    let source = task.funder;
    let (old_period, pool) = match source {
        Funder::Period { project: owner, period: old } if owner == project && old < period => (old, None),
        Funder::Pool { project: owner, period: old, .. } if owner == project && old < period => (old, Some(source)),
        Funder::Task(_) | Funder::Recurring { .. } | Funder::Period { .. } | Funder::Pool { .. } => return,
    };
    let old = Funder::Period { project, period: old_period };
    let fresh = Funder::Period { project, period };
    let archived = Funder::Recurring { project, task: number, period: old_period };
    let budget = task.numbers.budget;
    let Some(spent) = crate::funders::total(task.numbers) else { return };
    let reserved = task.numbers.reserved;
    let made = task.made;
    if task.run_reserved != 0
        || crate::run::run_attempt(&task.phase).is_some()
        || domain.funding.contains_key(&archived)
        || domain.funding.len() == domain.funding.capacity()
        || match spent.checked_add(reserved) {
            Some(used) => used > budget,
            None => true,
        }
    {
        return;
    }
    if !domain.funding.contains_key(&old) {
        return;
    }
    let Some(new_row) = domain.funding.get(&fresh) else { return };
    let mut new_numbers = new_row.numbers;
    let Some(next_reserved) = new_numbers.reserved.checked_add(budget) else { return };
    new_numbers.reserved = next_reserved;
    if crate::funders::available(new_numbers).is_none() {
        return;
    }
    if let Some(source) = pool {
        let Some(pool) = domain.funding.get(&source) else { return };
        if pool.parent != Some(old)
            || pool.numbers.budget.checked_sub(budget).is_none()
            || pool.numbers.reserved.checked_sub(budget).is_none()
        {
            return;
        }
    }
    let release = match task.phase {
        Phase::Held { why: Hold::Budget, .. } => {
            match crate::control::check_release(
                domain,
                &crate::compound::Staged::new(0),
                env,
                task.requester,
                crate::Release { task: number, again: None, message: None },
            ) {
                Ok(plan) => Some((task.requester, plan)),
                Err(_) => return,
            }
        }
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } | Phase::Closing(_) | Phase::Ended(_) => None,
    };
    let ledger = FundingRecord {
        funder: archived,
        parent: Some(old),
        period: None,
        numbers: Numbers { budget, spent: 0, spent_below: spent, reserved },
        made,
        closed: false,
    };
    assert!(domain.funding.insert(archived, ledger) == Ok(None), "standing ledger preflighted");
    if let Some(source) = pool {
        let pool = domain.funding.get_mut(&source).expect("pool preflighted");
        pool.numbers.budget = pool.numbers.budget.checked_sub(budget).expect("pool budget preflighted");
        pool.numbers.reserved = pool.numbers.reserved.checked_sub(budget).expect("pool reservation preflighted");
        crate::funders::save_funding(domain, source, out);
    }
    domain.funding.get_mut(&fresh).expect("new period preflighted").numbers = new_numbers;
    crate::funders::save_funding(domain, archived, out);
    crate::funders::save_funding(domain, fresh, out);
    let mut children = List::with_capacity(env.limits.tasks);
    for (child, _) in &domain.names {
        if record(domain, *child).expect("live name").funder == Funder::Task(number) {
            children.push(*child).expect("bounded live task set");
        }
    }
    for child in &children {
        task_mut(domain, *child).expect("live funded delegate").record.funder = archived;
        publish(domain, env, *child, out);
    }
    let task = &mut task_mut(domain, number).expect("standing task live").record;
    task.funder = fresh;
    task.numbers = Numbers { budget, spent: 0, spent_below: 0, reserved: 0 };
    task.made = 1;
    match release {
        Some((by, plan)) => crate::control::apply_release(domain, env, by, plan, out),
        None => publish(domain, env, number, out),
    }
    crate::funders::retire(domain, out);
}
