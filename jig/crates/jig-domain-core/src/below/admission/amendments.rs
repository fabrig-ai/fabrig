//! Task amendment admission outcomes (jig/docs/design/domain/engine/README.md, 4.4).

use jig_domain_authority as authority;
use jig_domain_tasks as tasks;

/// One admission outcome translated by the application into its call answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskAmendDenied {
    /// Invalid task, reference or caller.
    Refused { task: u64, why: tasks::Refusal },
    /// Authority did not allow widening directly.
    Denied(authority::Answer),
}
