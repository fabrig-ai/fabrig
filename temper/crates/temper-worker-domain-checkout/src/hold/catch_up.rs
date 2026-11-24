//! A landed dependency's catch-up (domain/worker/checkouts.md): keep the work,
//! move the pin, then merge the landing tip, work first. A delivery checks again;
//! an unchecked save may push the merged tree immediately.
use super::{Asked, Hold, Holding, Push, PushStep, State, To};
use crate::boundary::{Landing, Request, Start};
use crate::git::{Commit, Done, Fault, Into, Op, PushDiagnostic, Want};
use crate::limits::Limits;
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Step {
    Work { head: Commit, merging: Option<Commit> },
    KeepWork,
    Fetch,
    Compare,
    Merge,
    CommitMerge,
    KeepMerge,
}

pub(super) fn waiting(step: &PushStep) -> bool {
    match step {
        PushStep::Dependency { .. } => true,
        PushStep::CatchUp { .. }
        | PushStep::Prefix { .. }
        | PushStep::VerifyPrefix { .. }
        | PushStep::Design { .. }
        | PushStep::CompareLanding { .. }
        | PushStep::Commit { .. }
        | PushStep::Push { .. }
        | PushStep::Verify { .. } => false,
    }
}

pub(super) fn eligible(holding: &Holding, to: &To, repository: u32, parent: Commit) -> bool {
    match to {
        To::Saved => false,
        To::Branches | To::Unchecked => match &super::nth(holding, repository).start {
            Start::Merged { heads, .. } => heads.contains(&parent),
            Start::Base { .. }
            | Start::Branch { .. }
            | Start::Commit { .. }
            | Start::Saved { .. }
            | Start::Merge { .. }
            | Start::Prepare { .. } => false,
        },
    }
}

pub(super) fn refused() -> Landing {
    Landing::Explained {
        fault: Fault::Refused,
        diagnostic: PushDiagnostic::new(b"delivery second parent is not reachable from the branch's remote", 0),
    }
}

#[expect(clippy::too_many_arguments, reason = "the snapshot's parents and the push state start catch-up")]
pub(super) fn begin(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    head: Commit,
    merging: Option<Commit>,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    run(holding, id, push, repository, Step::Work { head, merging }, env, out)
}

#[expect(clippy::too_many_arguments, reason = "a cell handler takes the push state and terminal")]
#[expect(clippy::too_many_lines, reason = "the catch-up table lists each operation and terminal together")]
pub(super) fn pushed(
    holding: &mut Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    step: Step,
    asked: Asked,
    done: Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    match done {
        failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. }) => {
            let landing = super::failed_landing(failure);
            return super::complete_push(holding, id, push, repository, landing, asked, env, out);
        }
        Done::Conflicted { files, .. } => {
            let landing = match step {
                Step::Work { .. } => Landing::Conflicted { files },
                Step::KeepWork | Step::Fetch | Step::Compare | Step::Merge | Step::CommitMerge | Step::KeepMerge => {
                    Landing::CaughtUpConflict { files }
                }
            };
            return super::complete_push(holding, id, push, repository, landing, asked, env, out);
        }
        Done::ParentUnreachable { .. } => unreachable!("catch-up commits have no snapshot"),
        Done::Results { .. }
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::IsAncestor { .. }
        | Done::Brings { .. }
        | Done::Merged
        | Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected => {}
    }
    let next = match step {
        Step::Work { head, merging: _ } => {
            let commit = match done {
                Done::Committed { commit } => commit,
                Done::Unchanged => head,
                Done::Results { .. }
                | Done::Paths { .. }
                | Done::Looked { .. }
                | Done::IsAncestor { .. }
                | Done::Brings { .. }
                | Done::Merged
                | Done::Succeeded
                | Done::Fetched { .. }
                | Done::Exists
                | Done::Rejected
                | Done::Conflicted { .. }
                | Done::Failed { .. }
                | Done::FailedWithOutput { .. }
                | Done::ParentUnreachable { .. } => unreachable!("local commit terminals"),
            };
            let tips = super::tips_mut(holding, repository);
            tips.head = commit;
            tips.merging = None;
            tips.unchecked = false;
            Step::KeepWork
        }
        Step::KeepWork | Step::KeepMerge => {
            match done {
                Done::Committed { .. } => {}
                Done::Results { .. }
                | Done::Paths { .. }
                | Done::Looked { .. }
                | Done::IsAncestor { .. }
                | Done::Brings { .. }
                | Done::Merged
                | Done::Succeeded
                | Done::Fetched { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Conflicted { .. }
                | Done::Failed { .. }
                | Done::FailedWithOutput { .. }
                | Done::ParentUnreachable { .. } => unreachable!("reset terminals"),
            }
            if step == Step::KeepMerge {
                return finished(holding, id, push, repository, asked, env, out);
            }
            Step::Fetch
        }
        Step::Fetch => {
            match done {
                Done::Fetched { commit } => super::tips_mut(holding, repository).landing = Some(commit),
                Done::Results { .. }
                | Done::Paths { .. }
                | Done::Looked { .. }
                | Done::IsAncestor { .. }
                | Done::Brings { .. }
                | Done::Merged
                | Done::Succeeded
                | Done::Committed { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Conflicted { .. }
                | Done::Failed { .. }
                | Done::FailedWithOutput { .. }
                | Done::ParentUnreachable { .. } => unreachable!("fetch terminals"),
            }
            Step::Compare
        }
        Step::Compare => match done {
            Done::IsAncestor { contains: true } => return finished(holding, id, push, repository, asked, env, out),
            Done::IsAncestor { contains: false } => {
                let tips = super::tips_mut(holding, repository);
                tips.merging = tips.landing;
                Step::Merge
            }
            Done::Results { .. }
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::Merged
            | Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Conflicted { .. }
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. }
            | Done::ParentUnreachable { .. } => unreachable!("ancestry terminals"),
        },
        Step::Merge => {
            match done {
                Done::Merged => {}
                Done::Results { .. }
                | Done::Paths { .. }
                | Done::Looked { .. }
                | Done::IsAncestor { .. }
                | Done::Brings { .. }
                | Done::Succeeded
                | Done::Fetched { .. }
                | Done::Committed { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Conflicted { .. }
                | Done::Failed { .. }
                | Done::FailedWithOutput { .. }
                | Done::ParentUnreachable { .. } => unreachable!("merge terminals"),
            }
            Step::CommitMerge
        }
        Step::CommitMerge => {
            match done {
                Done::Committed { commit } => {
                    let tips = super::tips_mut(holding, repository);
                    tips.head = commit;
                    tips.merging = None;
                }
                Done::Results { .. }
                | Done::Paths { .. }
                | Done::Looked { .. }
                | Done::IsAncestor { .. }
                | Done::Brings { .. }
                | Done::Merged
                | Done::Succeeded
                | Done::Fetched { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Conflicted { .. }
                | Done::Failed { .. }
                | Done::FailedWithOutput { .. }
                | Done::ParentUnreachable { .. } => unreachable!("merge commit terminals"),
            }
            Step::KeepMerge
        }
    };
    if asked != Asked::Nothing {
        return super::end_push(holding, push, asked, out);
    }
    run(holding, id, push, repository, next, env, out)
}

fn finished(
    holding: &mut Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    asked: Asked,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if asked != Asked::Nothing {
        return super::end_push(holding, push, asked, out);
    }
    let tip = super::tips(holding, repository).landing.expect("catch-up fetched the landing pin");
    match push.to {
        To::Branches => super::complete_push(holding, id, push, repository, Landing::CaughtUp { tip }, asked, env, out),
        To::Unchecked => {
            super::tips_mut(holding, repository).caught_up = Some(tip);
            super::push_from(holding, id, push, repository, env, out)
        }
        To::Saved => unreachable!("saved work has no snapshot"),
    }
}

fn branch(holding: &Holding, repository: u32) -> Box<[u8]> {
    match &super::nth(holding, repository).start {
        Start::Merged { branch, .. } => branch.clone(),
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merge { .. }
        | Start::Prepare { .. } => unreachable!("only a merged start catches up"),
    }
}

fn run(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    step: Step,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let spec = super::nth(holding, repository);
    let at = super::place(holding.disk, spec);
    let tips = super::tips(holding, repository);
    let op = match step {
        Step::Work { head, merging } => Op::Commit {
            at,
            parent: head,
            merging,
            always: false,
            snapshot: None,
            title: push.message.title.clone(),
            body: push.message.body.clone(),
            identity: spec.identity,
        },
        Step::KeepWork | Step::KeepMerge => Op::Reset { at, commit: tips.head },
        Step::Fetch => Op::Fetch {
            at,
            store: super::borrowed(holding, repository).token,
            remote: spec.remote.clone(),
            want: Want::Branch { branch: branch(holding, repository) },
            into: Into::Landing,
            identity: spec.identity,
            account: spec.account,
        },
        Step::Compare => Op::IsAncestor { at, ancestor: tips.landing.expect("fetched tip"), commit: tips.head },
        Step::Merge => {
            Op::Merge { at, theirs: tips.landing.expect("fetched tip"), identity: spec.identity, account: spec.account }
        }
        Step::CommitMerge => {
            let branch = branch(holding, repository);
            super::commit_merge(
                holding,
                holding.disk,
                repository,
                tips.landing.expect("fetched tip"),
                super::branch_merge_title(&branch),
            )
        }
    };
    super::io(id, op, env, out);
    State::Pushing { push, step: PushStep::CatchUp { repository, step }, asked: Asked::Nothing }
}
