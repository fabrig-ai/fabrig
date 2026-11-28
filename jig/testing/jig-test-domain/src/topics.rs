//! Offered topics, completed hub operations, and classified news (domain/connectors-facts.md, 1.3).
use crate::{Domain, Work};
use alloc::boxed::Box;
use jig_domain_contract::effects;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Queue, ReplyTo, Token};

pub(super) fn start(
    domain: &mut Domain,
    to: ReplyTo,
    key: core::CallKey,
    number: u16,
    topic: u16,
    work: &mut Queue<Work>,
) {
    let token = domain.hub_link.fresh();
    domain.hub_link.keep_topic_call(token, (to, key, topic));
    work.push(Work::Connector { number, event: connector::Event::DescribeTopic { token, topic } });
}

pub(super) fn described(
    domain: &mut Domain,
    number: u16,
    token: Token,
    resources: Option<Box<[effects::Effect]>>,
    work: &mut Queue<Work>,
) {
    let Some((to, key, topic)) = domain.hub_link.take_topic_call(token) else { return };
    if !domain.engine.core().current_proof(key.task, key.attempt) {
        return;
    }
    match resources {
        Some(resources) => {
            work.push(Work::Core(core::Event::SubscribeTopic {
                to,
                key,
                connector: number,
                topic: u64::from(topic),
                resources,
            }));
        }
        None => {
            work.push(Work::Core(core::Event::NamedAnswer {
                to,
                key,
                part: core::CallPart::SubscriptionRefused(tasks::Problem {
                    task: Some(key.task),
                    why: tasks::Refusal::Subscription,
                    blocked_by: None,
                    capacity: None,
                }),
            }));
        }
    }
}
