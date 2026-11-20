//! One connector topic apart, kept as an ordinary durable subscription (domain/tasks/messages.md, 4).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "unit tests build fixtures and inspect selected child events"
)]
use super::revisions::{Harness, task, to};
use crate::{
    CapacityCause, Closing, Domain, Ending, Event, Key, MessageKind, NewsClass, Party, Phase, Refusal, Request,
    RestoreBound, RestoreCause, Stage, Stored, Subscription, SubscriptionKind, Word,
};
use skein_lib::{Duration, Wall};

const APART: u64 = 10;

fn topic(number: u64) -> Subscription {
    Subscription { number, kind: SubscriptionKind::Topic { connector: 1, topic: number } }
}

fn apart(h: &mut Harness, number: u64) -> Vec<Request> {
    h.send(Event::SubscribeTopicApart { reply_to: to(), task: 1, subscription: topic(number) })
}

fn done(out: &[Request]) {
    assert!(
        out.iter().any(|r| match r {
            Request::Done { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn refused(out: &[Request], why: Refusal, cause: Option<CapacityCause>) {
    let problem = out
        .iter()
        .find_map(|r| match r {
            Request::Refused { problem, .. } => Some(problem),
            _ => None,
        })
        .expect("one refusal");
    assert_eq!(problem.task, Some(1));
    assert_eq!(problem.why, why);
    assert_eq!(problem.capacity.map(|c| c.cause), cause);
}

fn fixture(others: u64) -> Harness {
    let mut h = Harness::new();
    h.env.limits.subscriptions = 4;
    h.env.limits.inbox_messages = 16;
    h.env.limits.inbox_bytes = 1024;
    h.make(vec![task(1, &[])]);
    let out = h.send(Event::Make {
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(1),
        batch: (2..=5).map(|n| task(n, &[])).collect(),
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    for number in 1..=others {
        done(&h.send(Event::Subscribe {
            reply_to: to(),
            task: 1,
            subscription: Subscription {
                number,
                kind: SubscriptionKind::Task { target: number.checked_add(1).unwrap(), held: true, result: true },
            },
        }));
    }
    h
}

#[test]
fn one_apart_topic_fits_past_the_bound_but_not_two() {
    let mut h = fixture(4);
    done(&apart(&mut h, APART));
    let Stored::Live(row) = h.stored.get(&Key::Live(1)).unwrap() else { panic!("live row") };
    assert_eq!(row.subscriptions.len(), 5);
    assert_eq!(row.subscriptions.last(), Some(&topic(APART)));
    refused(&apart(&mut h, 11), Refusal::Busy, Some(CapacityCause::Subscriptions { limit: 4 }));
    assert_eq!(h.domain.task(1).unwrap().subscriptions.len(), 5);
}

#[test]
fn ordinary_subscriptions_count_the_apart_topic_and_reuse_freed_room() {
    for kind in [
        SubscriptionKind::Task { target: 2, held: true, result: true },
        SubscriptionKind::Timer { at: Wall::from_nanos(100), period: None },
        topic(20).kind,
    ] {
        for removed in [1, APART] {
            let mut h = fixture(3);
            done(&apart(&mut h, APART));
            let subscribe = || match kind {
                SubscriptionKind::Topic { .. } => {
                    Event::SubscribeTopic { reply_to: to(), task: 1, subscription: Subscription { number: 20, kind } }
                }
                _ => Event::Subscribe { reply_to: to(), task: 1, subscription: Subscription { number: 20, kind } },
            };
            refused(&h.send(subscribe()), Refusal::Busy, Some(CapacityCause::Subscriptions { limit: 4 }));
            done(&h.send(Event::Unsubscribe { reply_to: to(), task: 1, subscription: removed }));
            done(&h.send(subscribe()));
            assert_eq!(h.domain.task(1).unwrap().subscriptions.len(), 4);
        }
    }
}

fn notice(h: &mut Harness, number: u64, kind: MessageKind) {
    h.send(Event::Notice {
        task: 1,
        word: Word {
            number,
            from: Party::Task(1),
            kind,
            words: Box::new([]),
            at: h.env.wall,
            hits: 1,
            eligible: false,
        },
    });
}

#[test]
fn five_interests_survive_list_rebuilds_and_merged_topic_news() {
    for period in [None, Some(Duration::from_nanos(50))] {
        let mut h = fixture(3);
        done(&h.send(Event::Subscribe {
            reply_to: to(),
            task: 1,
            subscription: Subscription {
                number: 4,
                kind: SubscriptionKind::Timer { at: Wall::from_nanos(100), period },
            },
        }));
        done(&apart(&mut h, APART));
        for number in [1, 2] {
            notice(&mut h, number, MessageKind::News { subscription: APART, class: NewsClass::Wakes });
        }
        let row = h.domain.task(1).unwrap();
        assert_eq!(row.subscriptions.len(), 5);
        assert_eq!(row.inbox.len(), 1);
        assert_eq!(row.inbox.first().unwrap().hits, 2);
        h.env.wall = Wall::from_nanos(100);
        notice(&mut h, 3, MessageKind::Timer { subscription: 4 });
        let row = h.domain.task(1).unwrap();
        if period.is_some() {
            assert_eq!(row.subscriptions.len(), 5);
            assert!(
                row.subscriptions
                    .iter()
                    .any(|s| s.number == 4 && s.kind == SubscriptionKind::Timer { at: Wall::from_nanos(150), period })
            );
        } else {
            assert_eq!(row.subscriptions.len(), 4);
            assert!(!row.subscriptions.iter().any(|s| s.number == 4));
        }
        done(&h.send(Event::Unsubscribe { reply_to: to(), task: 1, subscription: APART }));
        assert_eq!(h.domain.task(1).unwrap().subscriptions.len(), if period.is_some() { 4 } else { 3 });
    }
}

#[test]
fn apart_topics_refuse_closing_and_ended_tasks() {
    let ending = Ending::Failed { reason: Box::new([]) };
    for phase in [Phase::Closing(Closing { stage: Stage::Effects, ending: ending.clone() }), Phase::Ended(ending)] {
        let mut h = fixture(0);
        crate::domain::task_mut(&mut h.domain, 1).unwrap().record.phase = phase;
        refused(&apart(&mut h, APART), Refusal::State, None);
        assert!(h.domain.task(1).unwrap().subscriptions.is_empty());
    }
}

#[test]
fn apart_topics_refuse_zero_and_taken_numbers() {
    let mut h = fixture(1);
    for number in [0, 1] {
        refused(&apart(&mut h, number), Refusal::Subscription, None);
    }
    assert_eq!(h.domain.task(1).unwrap().subscriptions.len(), 1);
}

#[test]
fn apart_topics_still_reserve_an_inbox_slot() {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    let words = h.env.limits.inbox_messages;
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).unwrap().record;
    row.last_message = u64::from(words);
    row.inbox = (1..=words)
        .map(|number| Word {
            number: u64::from(number),
            from: Party::Person(9),
            kind: MessageKind::Words,
            words: Box::new([]),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        })
        .collect();
    let limits = h.env.limits;
    refused(
        &apart(&mut h, APART),
        Refusal::Busy,
        Some(CapacityCause::Inbox { task: 1, messages: limits.inbox_messages, bytes: limits.inbox_bytes }),
    );
    assert!(h.domain.task(1).unwrap().subscriptions.is_empty());
}

#[test]
fn restore_keeps_one_extra_subscription_and_refuses_two() {
    let mut h = fixture(4);
    done(&apart(&mut h, APART));
    let mut cold = Harness::new();
    cold.env.limits = h.env.limits;
    cold.domain = Domain::new(&cold.env.limits, super::KEPT, 91, 1, super::charters(), Box::new([0]));
    for row in h.stored.values().filter(|row| row.key().live()) {
        let out = cold.send(Event::Restore { record: row.clone() });
        assert!(
            !out.iter().any(|r| match r {
                Request::RestoreRefused { .. } => true,
                _ => false,
            }),
            "{out:?}"
        );
    }
    let out = cold.send(Event::Check);
    assert!(cold.domain.checked(), "{out:?}");
    let out = cold.send(Event::Restored);
    assert!(
        !out.iter().any(|r| match r {
            Request::RestoreRefused { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    assert_eq!(cold.domain.task(1).unwrap().subscriptions.len(), 5);

    let mut row = h.domain.task(1).unwrap().clone();
    let mut subscriptions = row.subscriptions.into_vec();
    subscriptions.push(topic(11));
    row.subscriptions = subscriptions.into_boxed_slice();
    let mut cold = Harness::new();
    cold.env.limits = h.env.limits;
    cold.domain = Domain::new(&cold.env.limits, super::KEPT, 91, 1, super::charters(), Box::new([0]));
    let out = cold.send(Event::Restore { record: Stored::Live(Box::new(row)) });
    assert!(
        out.iter().any(|r| match r {
            Request::RestoreRefused { problem } => problem.why == Refusal::Restore,
            _ => false,
        }),
        "{out:?}"
    );
    assert_eq!(
        cold.domain.restore_failure(),
        Some(crate::RestoreFailure {
            key: Key::Live(1),
            cause: RestoreCause::Bound { which: RestoreBound::Subscriptions, held: 6, limit: 5 },
        })
    );
}
