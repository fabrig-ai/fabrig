//! The destination read and lease for a push the engine must admit
//! (temper/docs/design/domain/worker/checkouts-push.md, 1).
use super::{
    Asked, Hold, Holding, Push, PushStep, State, To, borrowed, branch_of, candidate, design, nth, place, tips,
};
use crate::git::{Commit, Done, Fault, Into, Missing, Op, Want};
use crate::{Landing, Limits, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Target {
    Branch { repository: u32 },
    Prefix { repository: u32, member: u32 },
    Design { delivery: design::Delivery },
}

impl Target {
    pub(super) const fn repository(self) -> u32 {
        match self {
            Self::Branch { repository } | Self::Prefix { repository, .. } => repository,
            Self::Design { delivery } => delivery.repository,
        }
    }

    pub(super) fn branch(self, holding: &Holding, to: &To) -> Box<[u8]> {
        match self {
            Self::Branch { repository } | Self::Design { delivery: design::Delivery { repository, .. } } => {
                branch_of(nth(holding, repository), to)
            }
            Self::Prefix { repository, member } => candidate::branch(holding, repository, member),
        }
    }

    pub(super) const fn step(self) -> PushStep {
        match self {
            Self::Branch { repository } => PushStep::Push { repository },
            Self::Prefix { repository, member } => PushStep::Prefix { repository, member },
            Self::Design { delivery } => PushStep::Design { delivery, step: design::Step::Push },
        }
    }
}

pub(super) fn read(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    target: Target,
    commit: Commit,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let repository = target.repository();
    let spec = nth(holding, repository);
    super::io(
        id,
        Op::Fetch {
            at: place(holding.disk, spec),
            store: borrowed(holding, repository).token,
            remote: spec.remote.clone(),
            want: Want::Branch { branch: target.branch(holding, &push.to) },
            into: Into::Fetched,
            identity: spec.identity,
            account: spec.account,
        },
        env,
        out,
    );
    State::Pushing { push, step: PushStep::ReadPush { target, commit }, asked: Asked::Nothing }
}

#[expect(clippy::too_many_arguments, reason = "the destination read carries the pending push and terminal")]
pub(super) fn read_done(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    target: Target,
    commit: Commit,
    asked: Asked,
    done: Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if asked != Asked::Nothing {
        return super::end_push(holding, push, asked, out);
    }
    let repository = target.repository();
    let expected = match done {
        Done::Fetched { commit } => Some(commit),
        Done::Failed { fault: Fault::Missing { missing: Missing::Branch } }
        | Done::FailedWithOutput { fault: Fault::Missing { missing: Missing::Branch }, .. } => None,
        failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. }) => {
            return super::complete_push(
                holding,
                id,
                push,
                repository,
                super::failed_landing(failure),
                asked,
                env,
                out,
            );
        }
        Done::Succeeded
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Message { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => {
            unreachable!("a destination read ends as a fetch")
        }
    };
    match target {
        Target::Prefix { member, .. } if expected == Some(commit) => {
            // This prefix is already on its branch. No write is needed.
            candidate::push_from(holding, id, push, repository, super::after(member), env, out)
        }
        Target::Prefix { .. } if expected.is_some() => {
            super::complete_push(holding, id, push, repository, Landing::Moved, asked, env, out)
        }
        Target::Branch { .. } if guarded(&push.to, holding, repository, expected) => {
            super::complete_push(holding, id, push, repository, Landing::Moved, asked, env, out)
        }
        Target::Branch { .. } | Target::Prefix { .. } => request(holding, id, push, target, commit, expected, out),
        Target::Design { .. } => unreachable!("a design delivery takes the tip it already fetched"),
    }
}

fn guarded(to: &To, holding: &Holding, repository: u32, found: Option<Commit>) -> bool {
    match to {
        To::Branches | To::Unchecked => {
            let expected = tips(holding, repository).expected;
            expected.is_some() && found != expected
        }
        To::Saved => false,
    }
}

pub(super) fn request(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    target: Target,
    commit: Commit,
    expected: Option<Commit>,
    out: &mut Queue<Request>,
) -> State {
    ask(holding, id, target.repository(), target.branch(holding, &push.to), expected, commit, out);
    State::Pushing { push, step: PushStep::AdmitPush { target, commit, expected }, asked: Asked::Nothing }
}

pub(super) fn ask(
    holding: &Holding,
    id: Id<Hold>,
    repository: u32,
    branch: Box<[u8]>,
    expected: Option<Commit>,
    commit: Commit,
    out: &mut Queue<Request>,
) {
    out.push(Request::AdmitPush { client: holding.client, hold: id.token(), repository, branch, expected, commit });
}

/// A branch that was absent must still be absent. The existing create
/// operation has that lease; the ordinary push with no expected head does not.
pub(super) fn leased_op(
    holding: &Holding,
    repository: u32,
    branch: Box<[u8]>,
    commit: Commit,
    expected: Option<Commit>,
) -> Op {
    let spec = nth(holding, repository);
    let at = place(holding.disk, spec);
    let remote = spec.remote.clone();
    let identity = spec.identity;
    let account = spec.account;
    match expected {
        Some(_) => Op::Push { at, remote, commit, branch, expected, identity, account },
        None => Op::Create { at, remote, commit, branch, identity, account },
    }
}

pub(super) fn refusal(branch: &[u8], why: &[u8]) -> Box<[u8]> {
    let parts: [&[u8]; 4] = [b"temper did not admit the push to ", branch, b": ", why];
    let mut len = 0_usize;
    for part in parts {
        len = len.saturating_add(part.len());
    }
    // Keep the beginning so the operation and destination survive a long reason.
    let mut room = len.min(crate::git::PushDiagnostic::CAPACITY);
    let mut text = Writer::new(room);
    for part in parts {
        let take = room.min(part.len());
        text.put(part.get(..take).expect("bounded part")).expect("measured refusal");
        room = room.checked_sub(take).expect("part fits the remaining room");
    }
    text.finish()
}
