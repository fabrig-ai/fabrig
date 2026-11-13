//! Serialized, replaceable check files (domain/forge/reds.md, 5.3).
use super::{Asked, Hold, Holding, Message, State, To, after, begin, count, end_push, nth, place, released};
use crate::{Limits, Request, git};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Queue};

#[derive(Debug)]
pub(super) struct Running {
    pub repository: u32,
    pub action: git::Reds,
    pub clear: bool,
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
            Running { repository: 0, action, clear: false, write: None, push: None, asked: Asked::Nothing },
            env,
            out,
        ),
        State::Reds(mut running) => {
            match action {
                git::Reds::Clear => {
                    running.clear = true;
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
        | State::Closed) => other,
    }
}
pub(super) fn done(
    holding: &Holding,
    id: Id<Hold>,
    mut running: Running,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if running.clear {
        running.clear = false;
        running.action = git::Reds::Clear;
        running.repository = 0;
    } else {
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
