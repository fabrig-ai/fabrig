//! Offered topic calls are reads before they enter the task hub (domain/authority-checks.md, 1.5).
use crate::{
    CallHolder, CallKey, CallPart, Core, Event, Family, Held, Limits, Request, RoutedCall, fresh, routing::Work,
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn subscribe(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<Work>,
    to: ReplyTo,
    key: CallKey,
    topic: (u16, u64),
    resources: &[authority::Effect],
) -> Queue<Request> {
    let (connector, topic) = topic;
    let mut out = Queue::uncapped();
    if !core.current_proof(key.task, key.attempt) {
        return out;
    }
    if let Some(record) = core.call_record(key) {
        out.push(Request::Held(Box::new(Held::CallAnswer { to, key, part: record.part })));
        return out;
    }
    if core.counters.deployment().calls == u64::MAX || !core.reserve_call(key, CallHolder::Routed) {
        out.push(Request::Now(Box::new(crate::Now::EffectAnswer { to, key, part: CallPart::Unavailable })));
        return out;
    }
    let mut findings = Queue::uncapped();
    let answer = core.topic_admit(key, resources, &mut findings);
    if answer != authority::Answer::Allow {
        let part = CallPart::ToolDenied {
            answer,
            findings: {
                let mut found = skein_lib::List::with_capacity(findings.len());
                for _ in 0..findings.len() {
                    found.push(findings.pop().expect("finding count")).expect("finding room");
                }
                found.into_boxed()
            },
        };
        crate::routing::run::calls::call_decision(core, &mut out, to, key, part);
    } else if let Some(subscription) = fresh(&mut core.counters, Family::Message) {
        let token = to.into_token();
        let token = core.fly_call(token, RoutedCall::Subscribe { key, subscription });
        work.push(Work::Event(Event::Tasks(tasks::Event::SubscribeTopic {
            reply_to: ReplyTo::new(token),
            task: key.task,
            subscription: tasks::Subscription {
                number: subscription,
                kind: tasks::SubscriptionKind::Topic { connector, topic },
            },
        })));
    } else {
        crate::routing::run::calls::call_decision(core, &mut out, to, key, CallPart::Unavailable);
    }
    out
}
