//! A task's next-run wait, separate from its creator preference and its committed account
//! (client/domain/contract/documents.md, 1; accounts.md, 4.3).

use jig_domain_accounts as accounts;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskRunWait {
    Account { account: Option<u32>, source: Option<u32>, why: TaskWaitWhy },
    Credential,
    Host,
    Refused(TaskWaitRefused),
}
/// A derived explanation, never a durable task state (domain/engine/fleet.md).
/// Hello keeps the fleet's observed receipt time and cause until admission or restart.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitRefused {
    Hello {
        kind: jig_domain_fleet::HostKind,
        at: Wall,
        why: jig_domain_fleet::HelloRefusal,
        figures: Option<jig_domain_fleet::Figures>,
    },
    Bounds {
        kind: jig_domain_fleet::HostKind,
        why: Option<jig_domain_fleet::AgentShortfall>,
        figures: Option<jig_domain_fleet::Figures>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitWhy {
    Source(TaskSourceState),
    Refreshing,
    Host,
    Empty,
    Invalid(tasks::AccountWhy),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskSourceState {
    Moving,
    Asking,
    Granted,
    Later { until: Option<Wall> },
    Limited { until: Option<Wall> },
    Refused,
    NeedsSignIn,
    Misconfigured,
    Unloaded { at: accounts::Place, why: accounts::Unread },
    Cancelled,
    Closing,
}
