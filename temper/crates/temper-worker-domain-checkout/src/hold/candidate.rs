//! A one-member prepare, committed before delivery (temper/docs/design/domain/worker.md, 4.4).
use super::{Holding, Next, PrepareStep, Tips, borrowed, failure, nth, place, tips_mut};
use crate::{
    Candidate, CandidateSpec, Start,
    git::{Done, Into, Op, Want},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Step {
    Head,
    Upstream,
    Checkout,
    Merge,
    Brings,
    Commit { squash: bool },
}

pub(super) fn first(repository: u32) -> Next {
    next(repository, Step::Head)
}
fn next(repository: u32, step: Step) -> Next {
    Next::Step(PrepareStep::Candidate { repository, step })
}

pub(super) fn op(holding: &Holding, repository: u32, step: Step) -> Op {
    let spec = nth(holding, repository);
    let Start::Prepare { request } = &spec.start else { unreachable!("candidate steps have a prepare start") };
    let CandidateSpec { base, head, upstream, message, squash_message, .. } = request.as_ref();
    let at = place(holding.workspace.token(), spec);
    match step {
        Step::Head | Step::Upstream => Op::Fetch {
            at,
            store: borrowed(holding, repository).store.token(),
            remote: spec.remote.clone(),
            want: Want::Commit { commit: if step == Step::Head { *head } else { upstream.expect("upstream step") } },
            into: Into::Fetched,
            identity: spec.identity,
            account: spec.account,
        },
        Step::Checkout => Op::CheckOut { at, commit: *base, clean: true },
        Step::Merge => Op::Merge { at, theirs: *head },
        Step::Brings => Op::Brings { at, head: *head, base: *base, upstream: upstream.expect("upstream step") },
        Step::Commit { squash } => {
            let message = if squash { squash_message } else { message };
            Op::Commit {
                at,
                parent: *base,
                merging: if squash { None } else { Some(*head) },
                always: true,
                title: message.title.clone(),
                body: message.body.clone(),
                identity: spec.identity,
                snapshot: None,
            }
        }
    }
}

pub(super) fn after(holding: &mut Holding, repository: u32, step: Step, done: Done) -> Next {
    let Start::Prepare { request } = &nth(holding, repository).start else {
        unreachable!("candidate steps have a prepare start")
    };
    let (base, upstream, squash) = (request.base, request.upstream, request.squash);
    match done {
        Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
        Done::Fetched { .. } => match step {
            Step::Head if upstream.is_some() => next(repository, Step::Upstream),
            Step::Head | Step::Upstream => next(repository, Step::Checkout),
            Step::Checkout | Step::Merge | Step::Brings | Step::Commit { .. } => unreachable!("fetch terminal"),
        },
        Done::Succeeded => {
            assert_eq!(step, Step::Checkout, "checkout terminal");
            holding
                .tips
                .push(Tips {
                    start: base,
                    head: base,
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
        Done::Merged => {
            assert_eq!(step, Step::Merge, "merge terminal");
            next(repository, if squash && upstream.is_some() { Step::Brings } else { Step::Commit { squash } })
        }
        Done::Brings { brings } => {
            assert_eq!(step, Step::Brings, "contribution terminal");
            next(repository, Step::Commit { squash: !brings })
        }
        Done::Committed { commit } => {
            let Step::Commit { squash } = step else { unreachable!("commit terminal") };
            tips_mut(holding, repository).head = commit;
            holding.candidate = Some(Candidate::Combined { squash });
            Next::Ready
        }
        Done::Conflicted { files, cut } => {
            assert_eq!(step, Step::Merge, "merge conflicts precede commit");
            holding.candidate = Some(Candidate::Conflict { files, cut });
            Next::Ready
        }
        Done::Paths { .. }
        | Done::Looked { .. }
        | Done::IsAncestor { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected => unreachable!("candidate steps have their own terminals"),
    }
}
