//! The task's own procedure where nothing remains to await (jig/docs/design/domain/tasks/lifecycle.md, 11).
use crate::domain::{Domain, publish, record, task_mut};
use crate::{Active, Class, EndedBy, Hold, Limits, Party, Phase, Request, Was};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

/// Fixed words for the task's own closer (jig/docs/design/domain/tasks/lifecycle.md, 11).
pub const CLOSER_WORDS: &[u8] = b"Close your requester: nothing more will come.";

pub(crate) fn claim_numbers(domain: &Domain, task: u64) -> Option<crate::Numbers> {
    let row = record(domain, task)?;
    let mut numbers = row.numbers;
    if crate::run::run_attempt(&row.phase).is_none()
        && let Some(closer) = row.closer
        && let Some(closer) = record(domain, closer)
    {
        numbers.reserved = numbers.reserved.checked_sub(crate::funders::available(closer.numbers)?)?;
    }
    Some(numbers)
}

pub(crate) fn yield_for_run(domain: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, task) else { return };
    let Some(closer) = row.closer else { return };
    let numbers = record(domain, closer).expect("live closer").numbers;
    let floor = crate::funders::total(numbers)
        .expect("valid closer spend")
        .checked_add(numbers.reserved)
        .expect("valid closer reservations");
    crate::funders::resize(domain, env, closer, floor, out);
    publish(domain, env, closer, out);
}

/// Funds returned to an idle closer make it due, even when none remain
/// (jig/docs/design/domain/tasks/lifecycle.md, 11, Its funds follow its task's runs).
pub(crate) fn take_available(domain: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, task) else { return };
    if row.phase != Phase::Active(Active::Idle) && row.phase != Phase::Waiting {
        return;
    }
    let Some(closer) = row.closer else { return };
    let left = crate::funders::available(row.numbers).expect("valid task balance");
    let closing = match record(domain, closer).expect("live closer").phase {
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => true,
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => false,
    };
    if closing {
        return;
    }
    if left > 0 {
        let budget = record(domain, closer)
            .expect("live closer")
            .numbers
            .budget
            .checked_add(left)
            .expect("closer within its task's budget");
        crate::funders::resize(domain, env, closer, budget, out);
        publish(domain, env, closer, out);
    }
    crate::domain::wake_procedure(domain, env, closer, out);
}

pub(crate) fn named(row: &crate::TaskRecord) -> bool {
    match row.flags.ended_by {
        EndedBy::Closer(_) => true,
        EndedBy::Result | EndedBy::Requester | EndedBy::Standing => false,
    }
}

pub(crate) fn ask(domain: &Domain, task: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, task) else { return };
    if named(row) && row.closer.is_none() && crate::run::nothing_awaited(domain, row) {
        out.push(Request::CloserDue { task });
    }
}

pub(crate) fn make(domain: &mut Domain, env: &Env<Limits>, task: u64, number: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, task) else { return };
    if row.closer.is_some() || !crate::run::nothing_awaited(domain, row) {
        return;
    }
    let closer = match row.flags.ended_by {
        EndedBy::Closer(closer) => closer,
        EndedBy::Result | EndedBy::Requester | EndedBy::Standing => return,
    };
    let Some(spend) = crate::funders::available(row.numbers) else {
        refused(domain, env, task, Hold::Budget, out);
        return;
    };
    if spend == 0 || row.made.saturating_sub(1) >= row.authority.delegation.tasks {
        refused(domain, env, task, Hold::Budget, out);
        return;
    }
    if row.authority.delegation.depth == 0 {
        refused(domain, env, task, Hold::Failures(Class::Permanent), out);
        return;
    }
    let mut authority = row.authority.clone();
    authority.delegation.depth = authority.delegation.depth.saturating_sub(1);
    authority.budget.spend = spend;
    let member = crate::New {
        label: 0,
        flags: crate::Flags::PLAIN,
        number,
        project: row.project,
        executor: crate::Executor::Procedure { connector: closer.connector, code: closer.code },
        llm: None,
        spec: crate::Spec {
            job: None,
            asks: false,
            words: CLOSER_WORDS.into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: row.contract.clone(),
        authority,
        deadline: crate::Deadline::Named,
        numbers: crate::Numbers { budget: spend, spent: 0, spent_below: 0, reserved: 0 },
        funder: crate::Funder::Task(task),
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: crate::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    let batch: Box<[crate::New]> = Box::new([member]);
    if let Err(problem) = crate::batch::check(domain, &env.limits, Party::Task(task), &batch) {
        let why = if problem.why == crate::Refusal::Funding { Hold::Budget } else { Hold::Failures(Class::Permanent) };
        refused(domain, env, task, why, out);
        return;
    }
    task_mut(domain, task).expect("closer's requester live").record.closer = Some(number);
    let _made = crate::domain::make_admitted(domain, env, Party::Task(task), batch, out);
    publish(domain, env, task, out);
}

fn refused(domain: &mut Domain, env: &Env<Limits>, task: u64, why: Hold, out: &mut Queue<Request>) {
    task_mut(domain, task).expect("closer's requester live").record.phase =
        Phase::Held { was: Was::Active(Active::Due), why, source: None };
    publish(domain, env, task, out);
}

pub(crate) fn ended(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    closer: u64,
    ending: &crate::Ending,
    out: &mut Queue<Request>,
) {
    let row = &mut task_mut(domain, task).expect("closer's requester live").record;
    if row.closer != Some(closer) {
        return;
    }
    row.closer = None;
    match row.phase {
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => return,
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => {}
    }
    let attempt = crate::run::run_attempt(&row.phase);
    match ending {
        crate::Ending::Done(result) => {
            row.phase = Phase::Closing(crate::Closing {
                stage: match attempt {
                    Some(attempt) => crate::Stage::Run { attempt },
                    None => crate::Stage::Delegates,
                },
                ending: crate::Ending::Done(result.clone()),
            });
            if let Some(attempt) = attempt {
                out.push(Request::Stop { task, attempt });
            }
        }
        crate::Ending::Failed { .. } => {
            row.tries.add(Class::Run);
            if row.tries.run > env.limits.retries.run.retries {
                row.phase = Phase::Closing(crate::Closing {
                    stage: match attempt {
                        Some(attempt) => crate::Stage::Run { attempt },
                        None => crate::Stage::Delegates,
                    },
                    ending: crate::Ending::Failed { reason: Box::new([]) },
                });
                if let Some(attempt) = attempt {
                    out.push(Request::Stop { task, attempt });
                }
            }
        }
        crate::Ending::Cancelled { .. } => {}
    }
}

/// Project the closer's waits before the batch is admitted, and apply the same projection at making.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::result_large_err, reason = "batch admission retains its exact bounded refusal")
)]
pub(crate) fn with_work(
    domain: &Domain,
    limits: &Limits,
    creator: Party,
    batch: &[crate::New],
    at: skein_lib::Wall,
) -> Result<Option<crate::TaskRecord>, crate::Problem> {
    let parent = match creator {
        Party::Task(task) => task,
        Party::Person(_) | Party::Deployment { .. } => return Ok(None),
    };
    let Some(parent) = record(domain, parent) else { return Ok(None) };
    let Some(closer) = parent.closer else { return Ok(None) };
    let Some(row) = record(domain, closer) else { return Ok(None) };
    match row.phase {
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => {
            return Err(crate::Problem::new(Some(closer), crate::Refusal::State));
        }
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => {}
    }
    let mut dependencies = skein_lib::List::with_capacity(limits.dependencies);
    let mut waiting = skein_lib::List::with_capacity(limits.dependencies);
    let mut added = skein_lib::List::with_capacity(limits.dependencies);
    for dependency in &row.dependencies {
        dependencies.push(*dependency).expect("bounded dependencies");
    }
    for dependency in &row.waiting_on {
        waiting.push(*dependency).expect("bounded pending dependencies");
    }
    for entry in &row.added {
        added.push(entry.clone()).expect("bounded added dependencies");
    }
    let mut changed = false;
    for member in batch {
        if member.number == closer {
            continue;
        }
        if dependencies.as_slice().contains(&member.number) {
            continue;
        }
        if dependencies.push(member.number).is_err() {
            return Err(crate::Problem::new(Some(closer), crate::Refusal::Dependencies));
        }
        waiting.push(member.number).expect("pending dependencies bounded by dependencies");
        added.push(crate::Added { task: member.number, at, by: creator }).expect("added bounded by dependencies");
        changed = true;
    }
    if !changed {
        return Ok(None);
    }
    let Some(revision) = row.revisions.spec.checked_add(1) else {
        return Err(crate::Problem::new(Some(closer), crate::Refusal::State));
    };
    let mut row = row.clone();
    row.dependencies = dependencies.into_boxed();
    row.waiting_on = waiting.into_boxed();
    row.added = added.into_boxed();
    row.revisions.spec = revision;
    Ok(Some(row))
}
