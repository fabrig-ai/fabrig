//! A task's immutable identity and retention (domain/tasks/README.md, section 3 and domain/tasks/changes.md).
/// The flags a task is made with; amendments never change them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    /// Turns are entries for its requesting party; only agent tasks have turns.
    pub conversation: bool,
    /// Ended tasks are indexed by project and label.
    pub listed: bool,
    /// How long the ended task is kept.
    pub retention: Retention,
    /// Which authority may end this task, fixed at creation.
    pub ended_by: EndedBy,
}
impl Flags {
    /// Ordinary work: no conversation, not listed, kept.
    pub const PLAIN: Flags =
        Flags { ended_by: EndedBy::Result, conversation: false, listed: false, retention: Retention::Kept };
}
/// How long an ended task is retained (domain/people/README.md, section 5; protocol/store/README.md, section 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    /// Kept to the deployment's retention horizon.
    Kept,
    /// Erased once read by its requesting party and named by no live task.
    ForgottenOnceRead,
}

/// The immutable authority that may end a task (domain/tasks/lifecycle.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndedBy {
    /// Its run's judged result ends it.
    Result,
    /// Only its requester ends it; a run's raw finish parks it (5.8).
    Requester,
    /// A named procedure closes an agent task (jig/docs/design/domain/tasks/lifecycle.md, 11).
    Closer(Closer),
    /// A standing agent task ends only by cancel (jig/docs/design/domain/tasks/standing.md, 2).
    Standing,
}

/// The procedure named to close a task (jig/docs/design/domain/tasks/lifecycle.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Closer {
    pub connector: u16,
    pub code: u32,
}

/// Ending authority fits its executor and conversation (jig/docs/design/domain/tasks/lifecycle.md, 9 and 11).
#[must_use]
pub fn shaped(flags: Flags, executor: crate::Executor) -> bool {
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
