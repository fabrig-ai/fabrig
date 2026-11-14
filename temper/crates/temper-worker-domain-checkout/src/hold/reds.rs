//! Serialized, replaceable check files (domain/forge/reds.md, 5.3).
use super::{Asked, Hold, Holding, Message, State, To, after, begin, count, end_push, nth, place, released};
use crate::{Limits, Request, git};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue};

#[derive(Debug)]
pub(super) struct Running {
    pub repository: u32,
    pub action: git::Reds,
    pub next: Option<git::Reds>,
    pub write: Option<Box<[git::Red]>>,
    pub push: Option<(To, Message)>,
    pub asked: Asked,
}
pub(super) fn update(
    holding: &Holding,
    id: Id<Hold>,
    state: State,
    action: git::Reds,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    match state {
        State::Ready => run(
            holding,
            id,
            Running { repository: 0, action, next: None, write: None, push: None, asked: Asked::Nothing },
            env,
            out,
        ),
        State::Reds(mut running) => {
            match action {
                action @ (git::Reds::Clear | git::Reds::Read { .. }) => {
                    running.next = Some(action);
                    running.write = None;
                    out.push(Request::Cancel { owner: id.token() });
                }
                git::Reds::Write { reds } => running.write = Some(reds),
                git::Reds::Prepare { .. } => unreachable!("prepare is not a check update"),
            }
            State::Reds(running)
        }
        other @ (State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Unprepared
        | State::Pushing { .. }
        | State::Closed) => {
            interrupted(holding, &action, out);
            other
        }
    }
}
pub(super) fn done(
    holding: &Holding,
    id: Id<Hold>,
    mut running: Running,
    done: git::Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if let Some(action) = running.next.take() {
        running.action = action;
        running.repository = 0;
    } else {
        match running.action {
            git::Reds::Read { check } => {
                let bytes = match done {
                    git::Done::Results { bytes } => bytes,
                    git::Done::Paths { .. }
                    | git::Done::Looked { .. }
                    | git::Done::IsAncestor { .. }
                    | git::Done::Brings { .. }
                    | git::Done::Merged
                    | git::Done::Conflicted { .. }
                    | git::Done::Succeeded
                    | git::Done::Fetched { .. }
                    | git::Done::Committed { .. }
                    | git::Done::Unchanged
                    | git::Done::Exists
                    | git::Done::Rejected
                    | git::Done::Failed { .. }
                    | git::Done::FailedWithOutput { .. } => None,
                };
                out.push(Request::Results { client: holding.client, check, repository: running.repository, bytes });
            }
            git::Reds::Clear | git::Reds::Prepare { .. } | git::Reds::Write { .. } => {}
        }
        running.repository = after(running.repository);
    }
    run(holding, id, running, env, out)
}
fn run(holding: &Holding, id: Id<Hold>, mut running: Running, env: &Env<Limits>, out: &mut Queue<Request>) -> State {
    for repository in running.repository..count(holding.repositories.len()) {
        let spec = nth(holding, repository);
        if spec.push.is_none() {
            continue;
        }
        out.push(Request::Io {
            owner: id.token(),
            op: git::Op::Reds { at: place(holding.workspace.token(), spec), action: running.action.clone() },
            deadline: env.now.saturating_add(env.limits.local_timeout),
        });
        running.repository = repository;
        return State::Reds(running);
    }
    match running.action {
        git::Reds::Read { check } => out.push(Request::ResultsEnd { client: holding.client, check }),
        git::Reds::Clear | git::Reds::Prepare { .. } | git::Reds::Write { .. } => {}
    }
    if let Some(reds) = running.write.take() {
        running.repository = 0;
        running.action = git::Reds::Write { reds };
        return run(holding, id, running, env, out);
    }
    if let Some((to, message)) = running.push {
        if running.asked == Asked::Nothing {
            return begin(holding, id, to, message, env, out);
        }
        let push =
            super::Push { to, message, landings: skein_lib::List::with_capacity(count(holding.repositories.len())) };
        return end_push(holding, push, running.asked, out);
    }
    match running.asked {
        Asked::Release => released(holding.client, out),
        Asked::Abort | Asked::Nothing => State::Ready,
    }
}

pub(super) fn stopped(holding: &Holding, running: Running, out: &mut Queue<Request>) -> State {
    if let Some((to, message)) = running.push {
        let push =
            super::Push { to, message, landings: skein_lib::List::with_capacity(count(holding.repositories.len())) };
        return end_push(holding, push, running.asked, out);
    }
    match running.asked {
        Asked::Release => released(holding.client, out),
        Asked::Abort | Asked::Nothing => State::Ready,
    }
}

/// A cancelled read still settles the parent's report collection.
pub(super) fn interrupted(holding: &Holding, action: &git::Reds, out: &mut Queue<Request>) {
    match action {
        git::Reds::Read { check } => out.push(Request::ResultsEnd { client: holding.client, check: *check }),
        git::Reds::Clear | git::Reds::Prepare { .. } | git::Reds::Write { .. } => {}
    }
}
