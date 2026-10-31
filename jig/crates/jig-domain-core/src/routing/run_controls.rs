//! Task-origin controls fill the finish message before entering the tasks child.
use crate::{CallKey, CallPart, Core, Event, Request, RoutedCall};
use jig_domain_tasks as tasks;
use skein_lib::{Queue, ReplyTo};

pub(super) fn control(
    core: &mut Core,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    target: u64,
    mut control: tasks::Control,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let tasks::Control::Cancel { grace: Some(_), message, .. } = &mut control {
        *message = crate::fresh(&mut core.counters, crate::Family::Message);
        if message.is_none() {
            super::runs::call_decision(
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
    assert!(
        core.routing_calls.insert(token, RoutedCall::Control { key, target, revised: false }) == Ok(None),
        "one live routed call"
    );
    work.push(Event::Tasks(tasks::Event::Control {
        reply_to: ReplyTo::new(token),
        by: tasks::Party::Task(key.task),
        task: target,
        control,
    }));
    out
}
