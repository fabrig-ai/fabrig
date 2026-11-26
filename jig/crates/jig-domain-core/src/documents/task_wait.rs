//! A task's next-run wait, separate from its creator preference and its committed account
//! (client/domain/contract/documents.md, 1; domain/accounts-uses.md, 1.3).

use crate::{Core, RunWait};
use jig_domain_accounts as accounts;
pub use jig_domain_core_vocabulary::documents::task_wait::*;
use jig_domain_tasks as tasks;

/// The same next-run wait carried by a Task document, without making the document.
#[must_use]
pub fn waiting(core: &Core, row: &tasks::TaskRecord) -> Option<TaskRunWait> {
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing) => {}
        tasks::Phase::Active(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => return None,
    }
    match row.executor {
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => return None,
        tasks::Executor::Job { kind } => return job_wait(core, row.number, kind),
        tasks::Executor::Agent { .. } => {}
    }
    match core.run_choice(row.number) {
        Ok(_) if !core.credentials_ready(row.number, core.run_now) => Some(TaskRunWait::Credential),
        Ok(_) => None,
        Err(RunWait::InvalidAccount { account, why }) => {
            Some(TaskRunWait::Account { account: Some(account), source: None, why: TaskWaitWhy::Invalid(why) })
        }
        Err(RunWait::Credential) => Some(TaskRunWait::Credential),
        Err(RunWait::Host | RunWait::Worker { .. }) => Some(host_wait(core, row.number)),
        Err(RunWait::Account { account, source, why }) => {
            if why == accounts::WaitWhy::Host
                && let Some(source) = source
                && let Some(wait) = untooled(core, row.number, Some(source))
            {
                return Some(wait);
            }
            Some(TaskRunWait::Account {
                account,
                source,
                why: match why {
                    accounts::WaitWhy::Refreshing => TaskWaitWhy::Refreshing,
                    accounts::WaitWhy::Host => TaskWaitWhy::Host,
                    accounts::WaitWhy::Empty => TaskWaitWhy::Empty,
                    accounts::WaitWhy::Source(state) => TaskWaitWhy::Source(source_state(state)),
                },
            })
        }
    }
}

fn job_wait(core: &Core, task: u64, kind: u32) -> Option<TaskRunWait> {
    let Some(job) = core.job_declaration(kind) else { return Some(TaskRunWait::Host) };
    let run = skein_lib::Token::new(task);
    if core.fleet.job_available(run, job) {
        return None;
    }
    let Some(refusal) = core.fleet.job_refusal(run, job) else { return Some(TaskRunWait::Host) };
    let (why, figures) = match refusal.short {
        Some(short) => (JobWhy::Short(short.why), short.figures),
        None => (JobWhy::Untranslated, None),
    };
    Some(TaskRunWait::Refused(TaskWaitRefused::Job { worker: refusal.worker, kind, why, figures }))
}

fn host_wait(core: &Core, task: u64) -> TaskRunWait {
    if let Some(suspension) = core.tasks.suspension(task)
        && core.run_wall < suspension.until
    {
        return TaskRunWait::Worker { worker: suspension.worker, until: suspension.until };
    }
    let kinds = core.charter_of(task).hosts;
    if let Some((kind, record)) = core.fleet.standing_refusal(kinds) {
        return TaskRunWait::Refused(TaskWaitRefused::Hello {
            kind,
            at: record.at,
            why: record.why,
            figures: record.figures,
        });
    }
    if let Some(version) = core.charter_version(task)
        && let Some(value) = core.fleet.bounds_refusal(version, kinds)
    {
        return TaskRunWait::Refused(TaskWaitRefused::Bounds {
            kind: value.kind,
            why: value.why,
            figures: value.figures,
        });
    }
    untooled(core, task, None).unwrap_or(TaskRunWait::Host)
}

fn untooled(core: &Core, task: u64, waiting_source: Option<u32>) -> Option<TaskRunWait> {
    let fault = core.fleet.untooled(skein_lib::Token::new(task))?;
    let (source, _) = accounts::programs::issued(&core.accounts, fault.source.as_bytes())?;
    let fact = core.accounts.source(source)?;
    if let Some(waiting_source) = waiting_source
        && source != waiting_source
    {
        return None;
    }
    Some(TaskRunWait::Account {
        account: Some(fact.account),
        source: Some(source),
        why: TaskWaitWhy::Untooled {
            worker: fault.worker.checked_add(1).expect("a worker position is its number less one"),
        },
    })
}

fn source_state(state: accounts::State) -> TaskSourceState {
    match state {
        accounts::State::Moving => TaskSourceState::Moving,
        accounts::State::Asking => TaskSourceState::Asking,
        accounts::State::Granted => TaskSourceState::Granted,
        accounts::State::Later { until } => TaskSourceState::Later { until: Some(until.wall) },
        accounts::State::Limited { until } => TaskSourceState::Limited {
            until: match until {
                Some(until) => Some(until.wall),
                None => None,
            },
        },
        accounts::State::Refused => TaskSourceState::Refused,
        accounts::State::NeedsSignIn => TaskSourceState::NeedsSignIn,
        accounts::State::Misconfigured => TaskSourceState::Misconfigured,
        accounts::State::Unloaded { at, why } => TaskSourceState::Unloaded { at, why },
        accounts::State::Cancelled => TaskSourceState::Cancelled,
        accounts::State::Closing => TaskSourceState::Closing,
    }
}
