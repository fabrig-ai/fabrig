//! A task's immutable identity and retention (domain/tasks/README.md, section 3 and domain/tasks/changes.md).

pub use jig_domain_contract::ops::{Closer, EndedBy, Flags, Retention};
/// Ending authority fits its executor and conversation (jig/docs/design/domain/tasks/lifecycle-endings.md, 2 and 3).
#[must_use]
pub fn shaped(flags: Flags, executor: crate::Executor) -> bool {
    if crate::gathering::executor(executor) {
        return flags.ended_by == EndedBy::Requester && !flags.conversation;
    }
    if crate::batch::matches_job(executor) {
        return flags.ended_by == EndedBy::Result && !flags.conversation;
    }
    if flags.ended_by == EndedBy::Result {
        return true;
    }
    let agent = match executor {
        crate::Executor::Agent { .. } => true,
        crate::Executor::Person(_) | crate::Executor::Procedure { .. } | crate::Executor::Job { .. } => false,
    };
    match flags.ended_by {
        EndedBy::Result => true,
        EndedBy::Requester => flags.conversation && agent,
        EndedBy::Closer(_) | EndedBy::Standing => !flags.conversation && agent,
    }
}

/// Requester authority uses the existing requester, never a new cancel right.
pub(crate) fn valid(flags: Flags, executor: crate::Executor, requester: crate::Party) -> bool {
    if !shaped(flags, executor) {
        return false;
    }
    match flags.ended_by {
        EndedBy::Result | EndedBy::Closer(_) => return true,
        EndedBy::Requester | EndedBy::Standing => {}
    }
    match requester {
        crate::Party::Person(number) => number != 0,
        crate::Party::Task(_) | crate::Party::Deployment { .. } => false,
    }
}
