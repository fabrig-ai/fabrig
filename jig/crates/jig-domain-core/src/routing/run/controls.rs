//! Task-origin controls fill the finish message before entering the tasks child.
use crate::{CallKey, CallPart, Core, Event, Request, RoutedCall, routing::Work};
use jig_domain_tasks as tasks;
use skein_lib::{Queue, ReplyTo};

pub(crate) fn control(
    core: &mut Core,
    work: &mut Queue<Work>,
    to: ReplyTo,
    key: CallKey,
    mut cancel: tasks::Cancel,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let target = cancel.task;
    if cancel.grace.is_some() {
        cancel.message = crate::fresh(&mut core.counters, crate::Family::Message);
        if cancel.message.is_none() {
            super::calls::call_decision(
                core,
                &mut out,
                to,
                key,
                CallPart::ControlRefused(tasks::Problem {
                    task: Some(target),
                    why: tasks::Refusal::Busy,
                    blocked_by: None,
                    capacity: None,
                }),
            );
            return out;
        }
    }
    let token = to.into_token();
    assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
    let token = core.fly_call(token, RoutedCall::Control { key, target, revised: false });
    work.push(Work::Event(Event::Tasks(tasks::Event::Cancel {
        reply_to: ReplyTo::new(token),
        by: tasks::Party::Task(key.task),
        operation: cancel,
    })));
    out
}
