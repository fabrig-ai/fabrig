//! A run's read as its task's person (`jig/docs/design/domain/engine.md`, 7.3).
use super::{Answer, Read, Why};
use crate::{CallKey, Core, Limits, Now, Request, ToolKind};
use alloc::boxed::Box;
use jig_domain_tasks::Party;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn run(core: &mut Core, _env: &Env<Limits>, to: ReplyTo, key: CallKey, _read: Read) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, ToolKind::Ask) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    let task = core.tasks.task(key.task).expect("authorized run's task");
    match task.requester {
        Party::Person(_) => {}
        Party::Task(_) | Party::Deployment { .. } => return unknown(to, key),
    }
    unknown(to, key)
}

fn unknown(to: ReplyTo, key: CallKey) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::PersonAnswer { to, key, answer: Box::new(Answer::None(Why::Unknown)) })));
    out
}
