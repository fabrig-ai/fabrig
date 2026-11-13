//! A round's ordered stack, committed before delivery (temper/docs/design/domain/worker/jobs.md).
use super::{Holding, Next, PrepareStep, Tips, borrowed, failure, nth, place, tips_mut};
use crate::{
    Candidate, CandidateSpec, Combination, Repository, Start,
    git::{Commit, Done, Into, Op, Want},
};
use alloc::boxed::Box;
use skein_lib::List;

#[derive(Debug)]
pub(super) struct Stack {
    outcomes: List<Combination>,
    parent: Commit,
    conflict: Option<Combination>,
}

pub(super) fn new(repositories: &[Repository]) -> Option<Stack> {
    let [repository] = repositories else { return None };
    match &repository.start {
        Start::Prepare { request } => Some(Stack {
            outcomes: List::with_capacity(super::count(request.members.len())),
            parent: request.from,
            conflict: None,
        }),
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merge { .. }
        | Start::Merged { .. } => None,
    }
}
pub(super) fn result(stack: &Stack) -> Candidate {
    Candidate { members: stack.outcomes.clone().into_boxed() }
}
fn stack(holding: &Holding) -> &Stack {
    holding.candidate.as_ref().expect("a prepare stack")
}
fn stack_mut(holding: &mut Holding) -> &mut Stack {
    holding.candidate.as_mut().expect("a prepare stack")
}
fn request(holding: &Holding, repository: u32) -> &CandidateSpec {
    let Start::Prepare { request } = &nth(holding, repository).start else { unreachable!("a prepare stack") };
    request
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Step {
    Head,
    Upstream,
    Base,
    Checkout,
    Merge,
    Brings,
    Commit { squash: bool },
    Advance,
    MainCheckout,
    MainMerge,
    Restore,
}

pub(super) fn first(repository: u32) -> Next {
    next(repository, Step::Head)
}
fn next(repository: u32, step: Step) -> Next {
    Next::Step(PrepareStep::Candidate { repository, step })
}

pub(super) fn op(holding: &Holding, repository: u32, step: Step) -> Op {
    let spec = nth(holding, repository);
    let request = request(holding, repository);
    let stack = stack(holding);
    let at = place(holding.workspace.token(), spec);
    match step {
        Step::Head | Step::Upstream | Step::Base => {
            let commit = match step {
                Step::Head => request.members.get(skein_lib::usize_of(stack.outcomes.len())).expect("next member").head,
                Step::Upstream => request.upstream.expect("upstream step"),
                Step::Base => request.base,
                Step::Checkout
                | Step::Merge
                | Step::Brings
                | Step::Commit { .. }
                | Step::Advance
                | Step::MainCheckout
                | Step::MainMerge
                | Step::Restore => unreachable!("fetch step"),
            };
            Op::Fetch {
                at,
                store: borrowed(holding, repository).token,
                remote: spec.remote.clone(),
                want: Want::Commit { commit },
                into: Into::Fetched,
                identity: spec.identity,
                account: spec.account,
            }
        }
        Step::Checkout | Step::Advance | Step::MainCheckout | Step::Restore => Op::CheckOut {
            at,
            commit: if step == Step::MainCheckout { request.base } else { stack.parent },
            clean: true,
            identity: spec.identity,
            account: spec.account,
        },
        Step::Merge | Step::MainMerge | Step::Brings | Step::Commit { .. } => {
            let member = request.members.get(skein_lib::usize_of(stack.outcomes.len())).expect("current member");
            match step {
                Step::Merge | Step::MainMerge => {
                    Op::Merge { at, theirs: member.head, identity: spec.identity, account: spec.account }
                }
                Step::Brings => Op::Brings {
                    at,
                    head: member.head,
                    base: request.base,
                    upstream: request.upstream.expect("upstream step"),
                },
                Step::Commit { squash } => {
                    let message = if squash { &member.squash_message } else { &member.message };
                    Op::Commit {
                        at,
                        parent: stack.parent,
                        merging: if squash { None } else { Some(member.head) },
                        always: true,
                        title: message.title.clone(),
                        body: message.body.clone(),
                        identity: spec.identity,
                        snapshot: None,
                    }
                }
                Step::Head
                | Step::Upstream
                | Step::Base
                | Step::Checkout
                | Step::Advance
                | Step::MainCheckout
                | Step::Restore => unreachable!("member step"),
            }
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one transition table for a member's merge, conflict probe and prefix checkout"
)]
pub(super) fn after(holding: &mut Holding, repository: u32, step: Step, done: Done) -> Next {
    let request = request(holding, repository);
    let (base, from, upstream, squash, count) =
        (request.base, request.from, request.upstream, request.squash, request.members.len());
    match done {
        Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
        Done::Fetched { .. } => match step {
            Step::Head if stack(holding).outcomes.is_empty() && upstream.is_some() => next(repository, Step::Upstream),
            Step::Head if !stack(holding).outcomes.is_empty() => next(repository, Step::Merge),
            Step::Head | Step::Upstream if from != base => next(repository, Step::Base),
            Step::Head | Step::Upstream | Step::Base => next(repository, Step::Checkout),
            Step::Checkout
            | Step::Merge
            | Step::Brings
            | Step::Commit { .. }
            | Step::Advance
            | Step::MainCheckout
            | Step::MainMerge
            | Step::Restore => unreachable!("fetch terminal"),
        },
        Done::Succeeded => match step {
            Step::Checkout => {
                holding
                    .tips
                    .push(Tips {
                        start: from,
                        head: from,
                        pushed: None,
                        delivered: false,
                        unchecked: false,
                        merging: None,
                        expected: None,
                        landing: None,
                        design_merged: None,
                    })
                    .expect("one candidate repository");
                next(repository, Step::Merge)
            }
            Step::MainCheckout => next(repository, Step::MainMerge),
            Step::Advance | Step::Restore => {
                if skein_lib::usize_of(stack(holding).outcomes.len()) == count {
                    Next::Ready
                } else {
                    next(repository, Step::Head)
                }
            }
            Step::Head
            | Step::Upstream
            | Step::Base
            | Step::Merge
            | Step::Brings
            | Step::Commit { .. }
            | Step::MainMerge => unreachable!("checkout terminal"),
        },
        Done::Merged => match step {
            Step::Merge => {
                next(repository, if squash && upstream.is_some() { Step::Brings } else { Step::Commit { squash } })
            }
            Step::MainMerge => {
                let conflict = stack_mut(holding).conflict.take().expect("stack conflict");
                stack_mut(holding).outcomes.push(conflict).expect("one outcome per member");
                next(repository, Step::Restore)
            }
            Step::Head
            | Step::Upstream
            | Step::Base
            | Step::Checkout
            | Step::Brings
            | Step::Commit { .. }
            | Step::Advance
            | Step::MainCheckout
            | Step::Restore => unreachable!("merge terminal"),
        },
        Done::Brings { brings } => {
            assert_eq!(step, Step::Brings, "contribution terminal");
            next(repository, Step::Commit { squash: !brings })
        }
        Done::Committed { commit } => {
            let Step::Commit { squash } = step else { unreachable!("commit terminal") };
            let member =
                request.members.get(skein_lib::usize_of(stack(holding).outcomes.len())).expect("current member").number;
            let stack = stack_mut(holding);
            stack.outcomes.push(Combination::Combined { member, commit, squash }).expect("one outcome per member");
            stack.parent = commit;
            tips_mut(holding, repository).head = commit;
            next(repository, Step::Advance)
        }
        Done::Conflicted { files, cut } => {
            let member =
                request.members.get(skein_lib::usize_of(stack(holding).outcomes.len())).expect("current member").number;
            let stack = stack_mut(holding);
            match step {
                Step::Merge if stack.parent != base => {
                    stack.conflict = Some(Combination::Conflict { member, branch: false, files, cut });
                    next(repository, Step::MainCheckout)
                }
                Step::Merge | Step::MainMerge => {
                    stack.conflict = None;
                    stack
                        .outcomes
                        .push(Combination::Conflict { member, branch: true, files, cut })
                        .expect("one outcome per member");
                    next(repository, Step::Restore)
                }
                Step::Head
                | Step::Upstream
                | Step::Base
                | Step::Checkout
                | Step::Brings
                | Step::Commit { .. }
                | Step::Advance
                | Step::MainCheckout
                | Step::Restore => unreachable!("merge conflicts precede commit"),
            }
        }
        Done::Paths { .. }
        | Done::Looked { .. }
        | Done::IsAncestor { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected => unreachable!("candidate terminal"),
    }
}

pub(super) fn fits(repository: &Repository, request: &CandidateSpec, limits: &crate::Limits) -> bool {
    if request.members.is_empty() || request.members.len() > 5 {
        return false;
    }
    if repository.prefixes.is_empty() {
        if request.members.len() != 1 || repository.push.is_none() {
            return false;
        }
    } else if repository.push.is_some() || repository.prefixes.len() != request.members.len() {
        return false;
    }
    for branch in &repository.prefixes {
        if !super::named(branch, limits) {
            return false;
        }
    }
    for member in &request.members {
        if !super::message_fits(&member.message, limits) || !super::message_fits(&member.squash_message, limits) {
            return false;
        }
    }
    true
}

fn branch(holding: &Holding, repository: u32, member: u32) -> Box<[u8]> {
    let spec = nth(holding, repository);
    if spec.prefixes.is_empty() {
        spec.push.clone().expect("legacy prepare push")
    } else {
        spec.prefixes.get(skein_lib::usize_of(member)).expect("one branch per member").clone()
    }
}
fn combined(holding: &Holding, member: u32) -> Commit {
    match stack(holding).outcomes.get(member).expect("member in flight") {
        Combination::Combined { commit, .. } => *commit,
        Combination::Conflict { .. } => unreachable!("only combined members are pushed"),
    }
}

pub(super) fn push_from(
    holding: &Holding,
    id: skein_lib::Id<super::Hold>,
    push: super::Push,
    repository: u32,
    from: u32,
    env: &skein_lib::Env<crate::Limits>,
    out: &mut skein_lib::Queue<crate::Request>,
) -> super::State {
    let spec = nth(holding, repository);
    for member in from..stack(holding).outcomes.len() {
        match stack(holding).outcomes.get(member).expect("member") {
            Combination::Combined { commit, .. } => {
                let op = Op::Create {
                    at: place(holding.workspace.token(), spec),
                    remote: spec.remote.clone(),
                    commit: *commit,
                    branch: branch(holding, repository, member),
                    identity: spec.identity,
                    account: spec.account,
                };
                super::io(id, op, env, out);
                return super::State::Pushing {
                    push,
                    step: super::PushStep::Prefix { repository, member },
                    asked: super::Asked::Nothing,
                };
            }
            Combination::Conflict { .. } => {}
        }
    }
    let mut landing = crate::Landing::Unchanged;
    for member in 0..stack(holding).outcomes.len() {
        match stack(holding).outcomes.get(member).expect("member") {
            Combination::Combined { commit, .. } => {
                landing = if spec.prefixes.is_empty() {
                    crate::Landing::Landed { commit: *commit }
                } else {
                    crate::Landing::Prefixes { commit: *commit, branch: branch(holding, repository, member) }
                };
            }
            Combination::Conflict { .. } => {}
        }
    }
    super::complete_push(holding, id, push, repository, landing, super::Asked::Nothing, env, out)
}

#[expect(clippy::too_many_arguments, reason = "prefix delivery carries the in-flight member and terminal")]
pub(super) fn pushed(
    holding: &Holding,
    id: skein_lib::Id<super::Hold>,
    push: super::Push,
    repository: u32,
    member: u32,
    asked: super::Asked,
    done: Done,
    failure: Option<Done>,
    env: &skein_lib::Env<crate::Limits>,
    out: &mut skein_lib::Queue<crate::Request>,
) -> super::State {
    let landing = if let Some(failure) = failure {
        match done {
            Done::Fetched { commit } if commit == combined(holding, member) => crate::Landing::Landed { commit },
            Done::Fetched { .. } | Done::Failed { .. } | Done::FailedWithOutput { .. } => {
                super::failed_landing(failure)
            }
            Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("prefix verification is a fetch"),
        }
    } else {
        match done {
            Done::Succeeded => crate::Landing::Landed { commit: combined(holding, member) },
            Done::Exists | Done::Rejected => crate::Landing::Moved,
            failure @ (Done::Failed {
                fault:
                    crate::git::Fault::TimedOut
                    | crate::git::Fault::Broken
                    | crate::git::Fault::Unreachable
                    | crate::git::Fault::Cancelled,
            }
            | Done::FailedWithOutput {
                fault:
                    crate::git::Fault::TimedOut
                    | crate::git::Fault::Broken
                    | crate::git::Fault::Unreachable
                    | crate::git::Fault::Cancelled,
                ..
            }) => {
                let spec = nth(holding, repository);
                let op = Op::Fetch {
                    at: place(holding.workspace.token(), spec),
                    store: borrowed(holding, repository).token,
                    remote: spec.remote.clone(),
                    want: Want::Branch { branch: branch(holding, repository, member) },
                    into: Into::Fetched,
                    identity: spec.identity,
                    account: spec.account,
                };
                super::io(id, op, env, out);
                return super::State::Pushing {
                    push,
                    step: super::PushStep::VerifyPrefix { repository, member, failure },
                    asked,
                };
            }
            failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. }) => super::failed_landing(failure),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("prefix push terminal"),
        }
    };
    let succeeded = match &landing {
        crate::Landing::Landed { .. } => true,
        crate::Landing::Prefixes { .. }
        | crate::Landing::DesignMerged { .. }
        | crate::Landing::DesignConflict { .. }
        | crate::Landing::DesignCode { .. }
        | crate::Landing::DesignMoving
        | crate::Landing::DesignUnchecked
        | crate::Landing::Conflicted { .. }
        | crate::Landing::Explained { .. }
        | crate::Landing::Moved
        | crate::Landing::Failed
        | crate::Landing::Refused
        | crate::Landing::Unchanged
        | crate::Landing::Aborted => false,
    };
    if succeeded && asked == super::Asked::Nothing {
        push_from(holding, id, push, repository, super::after(member), env, out)
    } else {
        let landing = if succeeded { crate::Landing::Aborted } else { landing };
        super::complete_push(holding, id, push, repository, landing, asked, env, out)
    }
}
