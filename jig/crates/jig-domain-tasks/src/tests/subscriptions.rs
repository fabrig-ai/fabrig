//! A task's standing interests share one room (domain/tasks/messages.md, 4).
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
    CapacityCause, Closing, Domain, Ending, Event, Key, Limits, MessageKind, NewsClass, Party, Phase, Refusal, Request,
    RestoreBound, RestoreCause, Stage, Stored, Subscription, SubscriptionKind, Word,
};
use skein_lib::{Duration, Queue, Time, Wall};

const HOLDER: u64 = 1;
const TOPIC: u64 = 10;
// Four configured subscriptions and one more (domain/tasks/messages.md, 4).
const LIMIT: u32 = 4;
const ROOM: usize = 5;
const WATCHES: [u64; 4] = [1, 2, 3, 4];
const DUE: u64 = 100;
const REPEAT: Duration = Duration::from_nanos(50);

fn watch(number: u64) -> Subscription {
    Subscription {
        number,
        kind: SubscriptionKind::Task { target: number.checked_add(1).unwrap(), held: true, result: true },
    }
}

fn timer(number: u64, at: u64, period: Option<Duration>) -> Subscription {
    Subscription { number, kind: SubscriptionKind::Timer { at: Wall::from_nanos(at), period } }
}

fn topic(number: u64) -> Subscription {
    Subscription { number, kind: SubscriptionKind::Topic { connector: 1, topic: number } }
}

fn subscribe(h: &mut Harness, subscription: Subscription) -> Vec<Request> {
    let event = match subscription.kind {
        SubscriptionKind::Topic { .. } => Event::SubscribeTopic { reply_to: to(), task: HOLDER, subscription },
        SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } => {
            Event::Subscribe { reply_to: to(), task: HOLDER, subscription, reads_project: false }
        }
    };
    h.send(event)
}

fn remove(h: &mut Harness, subscription: u64) {
    done(&h.send(Event::Unsubscribe { reply_to: to(), task: HOLDER, subscription }));
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

fn refused_at(out: &[Request], task: u64, why: Refusal, cause: Option<CapacityCause>) {
    let problem = out
        .iter()
        .find_map(|r| match r {
            Request::Refused { problem, .. } => Some(problem),
            _ => None,
        })
        .expect("one refusal");
    assert_eq!(problem.task, Some(task));
    assert_eq!(problem.why, why);
    assert_eq!(problem.capacity.map(|c| c.cause), cause);
}

fn refused(out: &[Request], why: Refusal, cause: Option<CapacityCause>) {
    refused_at(out, HOLDER, why, cause);
}

fn full(out: &[Request]) {
    refused(out, Refusal::Busy, Some(CapacityCause::Subscriptions { limit: 5 }));
}

fn fixture(others: usize) -> Harness {
    let mut h = Harness::new();
    h.env.limits.subscriptions = LIMIT;
    h.env.limits.inbox_messages = 16;
    h.env.limits.inbox_bytes = 1024;
    h.make(vec![task(HOLDER, &[])]);
    let out = h.send(Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(HOLDER),
        batch: WATCHES.map(|number| task(number.checked_add(1).unwrap(), &[])).into(),
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    for number in WATCHES.into_iter().take(others) {
        done(&subscribe(&mut h, watch(number)));
    }
    h
}

fn live(h: &Harness) -> &crate::TaskRecord {
    h.domain.task(HOLDER).expect("live subscriber")
}

#[test]
fn task_watches_and_topics_fit_in_either_order() {
    for topic_first in [true, false] {
        let mut h = fixture(0);
        let watches = WATCHES.map(watch);
        let mut interests = watches.to_vec();
        if topic_first {
            interests.insert(0, topic(TOPIC));
        } else {
            interests.push(topic(TOPIC));
        }
        for interest in &interests {
            done(&subscribe(&mut h, *interest));
        }
        let Stored::Live(row) = h.stored.get(&Key::Live(HOLDER)).expect("stored subscriber") else {
            panic!("live row")
        };
        assert_eq!(row.subscriptions.as_ref(), interests);
    }
}

fn any_kind(number: u64) -> [Subscription; 3] {
    [Subscription { number, ..watch(1) }, timer(number, DUE, None), topic(number)]
}

#[test]
fn every_kind_is_refused_at_five_and_reuses_room() {
    let sixth = 11;
    for interest in any_kind(sixth) {
        for removed in [1, TOPIC] {
            let mut h = fixture(3);
            done(&subscribe(&mut h, topic(TOPIC)));
            done(&subscribe(&mut h, timer(4, DUE, None)));
            full(&subscribe(&mut h, interest));
            remove(&mut h, removed);
            done(&subscribe(&mut h, interest));
            assert_eq!(live(&h).subscriptions.len(), ROOM);
            assert_eq!(live(&h).subscriptions.last(), Some(&interest));
        }
    }
}

fn notice(h: &mut Harness, number: u64, kind: MessageKind) {
    h.send(Event::Notice {
        task: HOLDER,
        word: Word {
            number,
            from: Party::Task(HOLDER),
            kind,
            words: Box::new([]),
            at: h.env.wall,
            hits: 1,
            eligible: false,
        },
    });
}

#[test]
fn consecutive_topic_notices_merge_in_the_reserved_slot() {
    let mut h = fixture(3);
    done(&subscribe(&mut h, timer(4, DUE, None)));
    done(&subscribe(&mut h, topic(TOPIC)));
    for number in [1, 2] {
        notice(&mut h, number, MessageKind::News { subscription: TOPIC, class: NewsClass::Wakes });
    }
    assert_eq!(live(&h).subscriptions.len(), ROOM);
    let [word] = live(&h).inbox.as_ref() else { panic!("one merged notice") };
    assert_eq!(word.hits, 2);
}

#[test]
fn topics_refuse_closing_and_ended_tasks() {
    let ending = Ending::Failed { reason: Box::new([]) };
    for phase in [Phase::Closing(Closing { stage: Stage::Effects, ending: ending.clone() }), Phase::Ended(ending)] {
        let mut h = fixture(0);
        crate::domain::task_mut(&mut h.domain, HOLDER).unwrap().record.phase = phase;
        refused(&subscribe(&mut h, topic(TOPIC)), Refusal::State, None);
        assert!(live(&h).subscriptions.is_empty());
    }
}

#[test]
fn topics_refuse_zero_and_taken_numbers() {
    let mut h = fixture(1);
    for number in [0, 1] {
        refused(&subscribe(&mut h, topic(number)), Refusal::Subscription, None);
    }
    assert_eq!(live(&h).subscriptions.as_ref(), [watch(1)]);
}

#[test]
fn topics_still_reserve_an_inbox_slot() {
    let mut h = Harness::new();
    h.make(vec![task(HOLDER, &[])]);
    let words = h.env.limits.inbox_messages;
    let row = &mut crate::domain::task_mut(&mut h.domain, HOLDER).unwrap().record;
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
        &subscribe(&mut h, topic(TOPIC)),
        Refusal::Busy,
        Some(CapacityCause::Inbox { task: HOLDER, messages: limits.inbox_messages, bytes: limits.inbox_bytes }),
    );
    assert!(live(&h).subscriptions.is_empty());
}

fn read_rows(h: &Harness, limits: &Limits) -> Harness {
    let mut cold = Harness::new();
    cold.env.limits = *limits;
    cold.domain = Domain::new(limits, super::KEPT, 91, 1, super::charters(), Box::new([0]));
    for row in h.stored.values().filter(|row| row.key().live()) {
        cold.send(Event::Restore { record: row.clone() });
    }
    cold
}

fn start(cold: &mut Harness) {
    let out = cold.send(Event::Check);
    assert!(cold.domain.checked(), "{out:?}");
    cold.send(Event::Restored);
}

fn fire(h: &mut Harness, at: u64, subscription: u64) {
    h.env.now = Time::from_nanos(at);
    h.env.wall = Wall::from_nanos(at);
    let mut out = Queue::uncapped();
    crate::fire(&mut h.domain, &h.env, &mut out);
    let requests = (0..out.len()).map(|_| out.pop().unwrap()).collect::<Vec<_>>();
    assert!(requests.contains(&Request::Timer { task: HOLDER, subscription }), "{requests:?}");
}

#[test]
fn restore_keeps_seven_interests_and_services_their_timers() {
    let mut h = fixture(WATCHES.len());
    // Four watches, two timers and a topic were admitted before the limit fell.
    h.env.limits.subscriptions = 6;
    let repeating = 5;
    let once = 6;
    for interest in [timer(repeating, DUE, Some(REPEAT)), timer(once, DUE + 1, None), topic(TOPIC)] {
        done(&subscribe(&mut h, interest));
    }
    let mut cold = read_rows(&h, &Limits { subscriptions: LIMIT, ..h.env.limits });
    start(&mut cold);
    assert_eq!(live(&cold).subscriptions, live(&h).subscriptions);
    fire(&mut cold, DUE, repeating);
    notice(&mut cold, 1, MessageKind::Timer { subscription: repeating });
    assert_eq!(live(&cold).subscriptions.len(), 7);
    assert!(live(&cold).subscriptions.contains(&timer(repeating, 150, Some(REPEAT))));
    fire(&mut cold, DUE + 1, once);
    notice(&mut cold, 2, MessageKind::Timer { subscription: once });
    assert_eq!(live(&cold).subscriptions.len(), 6);
    assert!(!live(&cold).subscriptions.iter().any(|interest| interest.number == once));
}

#[test]
fn restored_interests_block_new_ones_until_room_returns() {
    let mut h = fixture(WATCHES.len());
    h.env.limits.subscriptions = 6;
    for number in [TOPIC, TOPIC + 1, TOPIC + 2] {
        done(&subscribe(&mut h, topic(number)));
    }
    let mut cold = read_rows(&h, &Limits { subscriptions: LIMIT, ..h.env.limits });
    start(&mut cold);
    let candidates = any_kind(20);
    for old in [TOPIC, TOPIC + 1, TOPIC + 2] {
        for interest in candidates {
            full(&subscribe(&mut cold, interest));
        }
        remove(&mut cold, old);
    }
    done(&subscribe(&mut cold, candidates[1]));
    assert_eq!(live(&cold).subscriptions.len(), ROOM);
}

#[test]
fn restore_refuses_timers_past_the_shared_deadline_table() {
    let mut h = Harness::new();
    h.make(vec![task(HOLDER, &[])]);
    for number in 1..=3 {
        done(&subscribe(&mut h, timer(number, DUE, None)));
    }
    // One live task times an admission room of two gives the timer table two slots.
    let cold = read_rows(&h, &Limits { tasks: 1, subscriptions: 1, ..h.env.limits });
    assert_eq!(
        cold.domain.restore_failure(),
        Some(crate::RestoreFailure {
            key: Key::Live(HOLDER),
            cause: RestoreCause::Bound { which: RestoreBound::Timers, held: 3, limit: 2 },
        })
    );
}

#[test]
fn restored_timers_reserve_room_through_a_due_notice() {
    const OTHER: u64 = 2;
    let mut h = Harness::new();
    h.make(vec![task(HOLDER, &[]), task(OTHER, &[])]);
    for number in 1..=4 {
        done(&subscribe(&mut h, timer(number, DUE, None)));
    }
    // Both live tasks' timer slots are taken by the first task after the limit falls.
    let mut cold = read_rows(&h, &Limits { tasks: 2, subscriptions: 1, ..h.env.limits });
    start(&mut cold);
    fire(&mut cold, DUE, 1);
    let event = || Event::Subscribe {
        reply_to: to(),
        task: OTHER,
        subscription: timer(5, DUE + 1, None),
        reads_project: false,
    };
    refused_at(&cold.send(event()), OTHER, Refusal::Busy, None);
    notice(&mut cold, 1, MessageKind::Timer { subscription: 1 });
    done(&cold.send(event()));
}
