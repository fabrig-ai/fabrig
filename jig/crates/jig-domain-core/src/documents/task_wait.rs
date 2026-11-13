//! A task's next-run wait, separate from its creator preference and its committed account
//! (client/domain/contract/documents.md, 1; accounts.md, 4.3).

use crate::{Core, RunWait};
use jig_domain_accounts as accounts;
pub use jig_domain_core_vocabulary::documents::task_wait::*;
use jig_domain_tasks as tasks;

pub(super) fn waiting(core: &Core, row: &tasks::TaskRecord) -> Option<TaskRunWait> {
    match row.executor {
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => return None,
        tasks::Executor::Agent { .. } => {}
    }
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing) => {}
        tasks::Phase::Active(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => return None,
    }
    match core.run_choice(row.number) {
        Ok(_) if !core.credentials_ready(row.number, core.run_now) => Some(TaskRunWait::Credential),
        Ok(_) => None,
        Err(RunWait::InvalidAccount { account, why }) => {
            Some(TaskRunWait::Account { account: Some(account), source: None, why: TaskWaitWhy::Invalid(why) })
        }
        Err(RunWait::Credential) => Some(TaskRunWait::Credential),
        Err(RunWait::Host) => Some(match core.fleet.standing_refusal(core.charter_of(row.number).hosts) {
            Some((kind, record)) => TaskRunWait::Refused(TaskWaitRefused::Hello {
                kind,
                at: record.at,
                why: record.why,
                figures: record.figures,
            }),
            None => match core.charter_version(row.number) {
                Some(version) => match core.fleet.bounds_refusal(version, core.charter_of(row.number).hosts) {
                    Some(value) => TaskRunWait::Refused(TaskWaitRefused::Bounds {
                        kind: value.kind,
                        why: value.why,
                        figures: value.figures,
                    }),
                    None => TaskRunWait::Host,
                },
                None => TaskRunWait::Host,
            },
        }),
        Err(RunWait::Account { account, source, why }) => Some(TaskRunWait::Account {
            account,
            source,
            why: match why {
                accounts::WaitWhy::Refreshing => TaskWaitWhy::Refreshing,
                accounts::WaitWhy::Host => TaskWaitWhy::Host,
                accounts::WaitWhy::Empty => TaskWaitWhy::Empty,
                accounts::WaitWhy::Source(state) => TaskWaitWhy::Source(source_state(state)),
            },
        }),
    }
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
