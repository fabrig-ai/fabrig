//! Design deliveries (temper/docs/design/domain/worker/checkouts.md): take the goal
//! tip first, admit Markdown alone, and push leased on that tip, never stale.
use super::{Asked, Hold, Holding, Push, PushStep, State, Tips, To};
use crate::boundary::{Landing, Repository, Request};
use crate::git::{Commit, Done, Into, Op, Want};
use crate::limits::Limits;
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Delivery {
    repository: u32,
    attempt: u8,
    tip: Option<Commit>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Step {
    Fetch,
    Compare,
    Merge,
    Commit,
    CheckOut,
    Paths,
    Push,
}

pub(super) fn repository(spec: &Repository) -> bool {
    spec.push.is_some() && spec.push == spec.landing
}

pub(super) fn checked(spec: &Repository, to: &To) -> bool {
    match to {
        To::Branches => repository(spec),
        To::Unchecked | To::Saved => false,
    }
}

pub(super) fn landed(tips: Tips, to: &To) -> Landing {
    match to {
        To::Branches => match tips.design_merged {
            Some(tip) => Landing::DesignMerged { commit: tips.head, tip },
            None => Landing::Landed { commit: tips.head },
        },
        To::Unchecked | To::Saved => Landing::Landed { commit: tips.head },
    }
}

pub(super) fn begin(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    run(holding, id, push, Delivery { repository, attempt: 1, tip: None }, Step::Fetch, Asked::Nothing, env, out)
}

#[expect(clippy::too_many_arguments, reason = "the design transition takes the push state and its terminal")]
#[expect(clippy::too_many_lines, reason = "the design delivery table lists every step and terminal together")]
pub(super) fn pushed(
    holding: &mut Holding,
    id: Id<Hold>,
    push: Push,
    mut delivery: Delivery,
    step: Step,
    asked: Asked,
    done: Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let repository = delivery.repository;
    // A push may have landed even when its answer failed; verification remains
    // the ordinary delivery's, and never turns a design refusal into stale.
    if step == Step::Push {
        match done {
            Done::Rejected if asked == Asked::Nothing && delivery.attempt < 3 => {
                delivery.attempt = delivery.attempt.checked_add(1).expect("three attempts");
                delivery.tip = None;
                return run(holding, id, push, delivery, Step::Fetch, asked, env, out);
            }
            Done::Rejected => {
                return super::complete_push(holding, id, push, repository, Landing::DesignMoving, asked, env, out);
            }
            other @ (Done::Succeeded | Done::Failed { .. } | Done::FailedWithOutput { .. }) => {
                return match super::pushed_to(holding, &push.to, repository, other) {
                    Ok(landing) => super::complete_push(holding, id, push, repository, landing, asked, env, out),
                    Err(failure) => super::verify(holding, id, push, repository, asked, failure, env, out),
                };
            }
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists => unreachable!("a push has its own terminals"),
        }
    }
    let done = match done {
        failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. }) => {
            let landing = super::failed_landing(failure);
            return super::complete_push(holding, id, push, repository, landing, asked, env, out);
        }
        Done::Conflicted { files } => {
            return super::complete_push(
                holding,
                id,
                push,
                repository,
                Landing::DesignConflict { files },
                asked,
                env,
                out,
            );
        }
        other @ (Done::Paths { .. }
        | Done::Looked { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected) => other,
    };
    // Record an explicit merge commit even if the caller asked to stop while
    // it was in flight. Nothing else starts once that operation has settled.
    if step == Step::Commit {
        match done {
            Done::Committed { commit } => {
                let tips = super::tips_mut(holding, repository);
                tips.head = commit;
                tips.merging = None;
            }
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Succeeded
            | Done::Fetched { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("an explicit merge commit records both parents"),
        }
    }
    if asked != Asked::Nothing {
        return super::complete_push(holding, id, push, repository, Landing::Aborted, asked, env, out);
    }
    let next = match step {
        Step::Fetch => match done {
            Done::Fetched { commit } => {
                delivery.tip = Some(commit);
                super::tips_mut(holding, repository).expected = Some(commit);
                Step::Compare
            }
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("a fetch has its own terminals"),
        },
        Step::Compare => match done {
            Done::IsAncestor { contains: true } => Step::Paths,
            Done::IsAncestor { contains: false } => {
                let tips = super::tips_mut(holding, repository);
                tips.merging = delivery.tip;
                tips.design_merged = delivery.tip;
                Step::Merge
            }
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("an ancestry query has its own terminals"),
        },
        Step::Merge => match done {
            Done::Merged => Step::Commit,
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Conflicted { .. }
            | Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("a merge has its own terminals"),
        },
        Step::Commit => Step::CheckOut,
        Step::CheckOut => match done {
            Done::Succeeded => Step::Paths,
            Done::Paths { .. }
            | Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("a checkout has its own terminals"),
        },
        Step::Paths => match done {
            Done::Paths { files } => {
                for path in files {
                    if !path.ends_with(b".md") {
                        return super::complete_push(
                            holding,
                            id,
                            push,
                            repository,
                            Landing::DesignCode { path },
                            asked,
                            env,
                            out,
                        );
                    }
                }
                if super::unpushed(holding, &push.to, repository).is_none() {
                    let landing = super::landed_before(holding, &push.to, repository);
                    return super::complete_push(holding, id, push, repository, landing, asked, env, out);
                }
                Step::Push
            }
            Done::Looked { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Failed { .. }
            | Done::FailedWithOutput { .. } => unreachable!("a changed-path query has its own terminals"),
        },
        Step::Push => unreachable!("push terminals handled before local steps"),
    };
    run(holding, id, push, delivery, next, asked, env, out)
}

#[expect(clippy::too_many_arguments, reason = "a design step carries the push state and its operation")]
fn run(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    delivery: Delivery,
    step: Step,
    asked: Asked,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let spec = super::nth(holding, delivery.repository);
    let at = super::place(holding.workspace.token(), spec);
    let head = super::tips(holding, delivery.repository).head;
    let op = match step {
        Step::Fetch => Op::Fetch {
            store: super::borrowed(holding, delivery.repository).store.token(),
            at,
            remote: spec.remote.clone(),
            want: Want::Branch { branch: super::branch_of(spec, &push.to) },
            into: Into::Fetched,
            identity: spec.identity,
            account: spec.account,
        },
        Step::Compare => Op::IsAncestor { at, ancestor: delivery.tip.expect("fetched tip"), commit: head },
        Step::Merge => Op::Merge { at, theirs: delivery.tip.expect("fetched tip") },
        Step::Commit => {
            let branch = spec.push.as_deref().expect("a design repository is writable");
            let mut title = Writer::new(b"Merge ".len().checked_add(branch.len()).expect("bounded branch"));
            title.put(b"Merge ").expect("measured prefix");
            title.put(branch).expect("measured branch");
            Op::Commit {
                at,
                parent: head,
                merging: delivery.tip,
                always: false,
                snapshot: None,
                title: title.finish(),
                body: Box::new([]),
                identity: spec.identity,
            }
        }
        Step::CheckOut => Op::CheckOut { at, commit: head, clean: true },
        Step::Paths => Op::DiffPaths { at, before: delivery.tip.expect("fetched tip"), after: head },
        Step::Push => Op::Push {
            at,
            remote: spec.remote.clone(),
            commit: head,
            branch: super::branch_of(spec, &push.to),
            expected: delivery.tip,
            identity: spec.identity,
            account: spec.account,
        },
    };
    super::io(id, op, env, out);
    State::Pushing { push, step: PushStep::Design { delivery, step }, asked }
}
