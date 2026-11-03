//! The one public task projection (jig/docs/design/client/domain/contract.md, 12).
use super::{PublicCounts, PublicExecutor, PublicHold, PublicLine, PublicPhase};
use jig_domain_tasks as tasks;

pub(super) fn make(row: &tasks::TaskRecord, parent: Option<u64>, delegates: PublicCounts) -> PublicLine {
    let first = skein_lib::bytes::find(&row.spec.words, b"\n").unwrap_or(row.spec.words.len());
    let title = crate::documents::bounded_text(row.spec.words.get(..first).expect("first line prefix"), 256);
    PublicLine {
        task: row.number,
        title,
        parent,
        goal: row.tracked.is_some(),
        priority: row.tracked,
        executor: match row.executor {
            tasks::Executor::Agent { .. } => PublicExecutor::Agent,
            tasks::Executor::Procedure { .. } => PublicExecutor::Procedure,
            tasks::Executor::Person(_) => PublicExecutor::Person,
            tasks::Executor::Job { .. } => PublicExecutor::Job,
        },
        phase: phase(row),
        hold: match row.phase {
            tasks::Phase::Held { why, .. } => Some(hold(why)),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => None,
        },
        made: row.created_at.as_secs(),
        changed: row.changed.as_secs(),
        active: match row.run_since {
            Some(at) => Some(at.as_secs()),
            None => None,
        },
        ended: match row.ended_at {
            Some(at) => Some(at.as_secs()),
            None => None,
        },
        spent: row.numbers.spent.checked_add(row.numbers.spent_below).expect("bounded task spend"),
        budget: row.numbers.budget,
        delegates,
    }
}

fn phase(row: &tasks::TaskRecord) -> PublicPhase {
    match row.phase {
        tasks::Phase::Waiting => PublicPhase::Waiting,
        tasks::Phase::Active(active) => match row.executor {
            tasks::Executor::Person(_) => PublicPhase::Active,
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => {
                match active {
                    tasks::Active::Idle => PublicPhase::Idle,
                    tasks::Active::Due => PublicPhase::Due,
                    tasks::Active::Preparing => PublicPhase::Preparing,
                    tasks::Active::Claimed { .. } => PublicPhase::Claimed,
                    tasks::Active::Running { .. } => PublicPhase::Running,
                    tasks::Active::BackingOff { .. } => PublicPhase::BackingOff,
                }
            }
        },
        tasks::Phase::Closing(_) => PublicPhase::Closing,
        tasks::Phase::Held { .. } => PublicPhase::Held,
        tasks::Phase::Ended(_) => PublicPhase::Ended,
    }
}

fn hold(why: tasks::Hold) -> PublicHold {
    match why {
        tasks::Hold::Exceeded(_) => PublicHold::Exceeded,
        tasks::Hold::Failures(_) => PublicHold::Failures,
        tasks::Hold::Stalled => PublicHold::Stalled,
        tasks::Hold::Dependency(_) => PublicHold::Dependency,
        tasks::Hold::Stopped => PublicHold::Stopped,
        tasks::Hold::Placed { .. } => PublicHold::Placed,
        tasks::Hold::Drift => PublicHold::Drift,
        tasks::Hold::Effects => PublicHold::Effects,
        tasks::Hold::EffectFailed => PublicHold::EffectFailed,
        tasks::Hold::Uncertain { .. } => PublicHold::Uncertain,
        tasks::Hold::Budget => PublicHold::Budget,
        tasks::Hold::Deadline => PublicHold::Deadline,
        tasks::Hold::HoldsWaited => PublicHold::HoldsWaited,
        tasks::Hold::Circle => PublicHold::Circle,
        tasks::Hold::ResourceUnavailable => PublicHold::ResourceUnavailable,
        tasks::Hold::ResourceAccess => PublicHold::ResourceAccess,
        tasks::Hold::PoolLost { .. } => PublicHold::PoolLost,
        tasks::Hold::Procedure => PublicHold::Procedure,
    }
}
