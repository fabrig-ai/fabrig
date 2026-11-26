//! Standing task and timer interests with merged inbox hints (domain/tasks/messages.md,
//! sections 2 and 4). Each subscription stays in its owner's task row.
//! Root assigns numbers and translates state/timer requests into messages in
//! the same decision; connector topics are routed through the application.
use crate::domain::{Domain, entrance, publish, record, refused, task_mut};
use crate::{
    Hint, Limits, MessageFamily, MessageKind, NewsClass, NoticeState, Phase, Refusal, Request, Subscription,
    SubscriptionKind, Word,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Set, Wall};

fn hint_of(kind: &MessageKind) -> Option<u64> {
    match crate::message_family(kind) {
        MessageFamily::Hint(
            Hint::Notice { subscription, .. } | Hint::Timer { subscription } | Hint::News { subscription, .. },
        ) => Some(subscription),
        MessageFamily::Spoken(_)
        | MessageFamily::Slot(_)
        | MessageFamily::Separate(_)
        | MessageFamily::Result
        | MessageFamily::Virtual => None,
    }
}

pub(crate) fn credit(task: &crate::TaskRecord, limits: &Limits) -> Option<(usize, usize)> {
    let mut count = 0_usize;
    let mut bytes = 0_usize;
    // A tracked goal keeps one merged core-news slot, numbered zero because
    // externally assigned subscriptions are nonzero (domain/connectors.md, 8).
    if task.tracked.is_some() {
        let mut waiting = false;
        for word in &task.inbox {
            waiting |= hint_of(&word.kind) == Some(0);
        }
        if !waiting {
            count = count.checked_add(1)?;
            bytes = bytes.checked_add(usize::try_from(limits.message_bytes).ok()?)?;
        }
    }
    for subscription in &task.subscriptions {
        let mut waiting = false;
        for word in &task.inbox {
            if hint_of(&word.kind) == Some(subscription.number) {
                waiting = true;
            }
        }
        if !waiting {
            count = count.checked_add(1)?;
            bytes = bytes.checked_add(usize::try_from(limits.result_bytes.max(limits.message_bytes)).ok()?)?;
        }
    }
    Some((count, bytes))
}

fn find(task: &crate::TaskRecord, number: u64) -> Option<Subscription> {
    for subscription in &task.subscriptions {
        if subscription.number == number {
            return Some(*subscription);
        }
    }
    None
}

fn arm(domain: &mut Domain, env: &Env<Limits>, subscription: Subscription) {
    match subscription.kind {
        SubscriptionKind::Timer { at, .. } => {
            let due = env.now.saturating_add(Duration::from_nanos(at.as_nanos().saturating_sub(env.wall.as_nanos())));
            assert!(domain.timers.arm(subscription.number, due).is_ok(), "timer subscription room admitted");
        }
        SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => {}
    }
}

pub(crate) enum Admission {
    Own,
    Topic,
}

/// The shared admission room for a live task's standing interests.
#[must_use]
pub fn kept_capacity(limits: &Limits) -> Option<u32> {
    limits.subscriptions.checked_add(1)
}

pub(crate) fn subscribe(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    subscription: Subscription,
    (admission, reads_project): (Admission, bool),
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let task = record(domain, number).expect("subscriber live");
    match task.phase {
        Phase::Closing(_) | Phase::Held { was: crate::Was::Closing(_), .. } | Phase::Ended(_) => {
            return refused(to, Some(number), Refusal::State, out);
        }
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } => {}
    }
    if subscription.number == 0 || find(task, subscription.number).is_some() {
        return refused(to, Some(number), Refusal::Subscription, out);
    }
    let count_limit = kept_capacity(&env.limits).expect("validated subscriptions");
    let capacity = crate::Capacity {
        project: Some(task.project),
        by: crate::Party::Task(number),
        action: crate::CapacityAction::Subscription { number: subscription.number },
        cause: crate::CapacityCause::Subscriptions { limit: count_limit },
    };
    let connector_topic = match admission {
        Admission::Own => false,
        Admission::Topic => true,
    };
    if task.subscriptions.len() >= skein_lib::length::usize_of(count_limit) {
        return crate::capacity::refuse(domain, env, to, Some(number), Refusal::Busy, capacity, out);
    }
    match subscription.kind {
        SubscriptionKind::Task { target, held, result } => {
            let Some(watched) = record(domain, target) else {
                return refused(to, Some(number), Refusal::Reference, out);
            };
            let reachable =
                crate::refs::allows(domain, number, target) || reads_project && watched.project == task.project;
            if !reachable || (!held && !result) {
                return refused(to, Some(number), Refusal::Reference, out);
            }
        }
        SubscriptionKind::Timer { period, .. } => {
            if period == Some(Duration::ZERO) {
                return refused(to, Some(number), Refusal::Subscription, out);
            }
        }
        SubscriptionKind::Topic { .. } if !connector_topic => {
            return refused(to, Some(number), Refusal::Subscription, out);
        }
        SubscriptionKind::Topic { .. } => {}
    }
    let timer = match subscription.kind {
        SubscriptionKind::Timer { .. } => true,
        SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => false,
    };
    if timer && domain.timer_subscriptions == domain.timer_capacity {
        return refused(to, Some(number), Refusal::Busy, out);
    }
    let reserve = skein_lib::length::usize_of(env.limits.result_bytes.max(env.limits.message_bytes));
    if !crate::inbox::room(domain, &env.limits, number, 1, reserve) {
        return crate::capacity::refuse(
            domain,
            env,
            to,
            Some(number),
            Refusal::Busy,
            crate::Capacity { cause: crate::capacity::inbox(number, &env.limits), ..capacity },
            out,
        );
    }
    let row = task_mut(domain, number).expect("subscriber live");
    let mut next = List::with_capacity(count_limit);
    for old in &row.record.subscriptions {
        next.push(*old).expect("bounded subscriptions");
    }
    next.push(subscription).expect("subscription room preflighted");
    row.record.subscriptions = next.into_boxed();
    publish(domain, env, number, out);
    if timer {
        domain.timer_subscriptions = domain.timer_subscriptions.checked_add(1).expect("timer room admitted");
    }
    arm(domain, env, subscription);
    match subscription.kind {
        SubscriptionKind::Task { target, held: true, .. } => {
            let watched = record(domain, target).expect("watched target live");
            match watched.phase {
                Phase::Held { .. } => out.push(Request::Notify {
                    task: number,
                    subscription: subscription.number,
                    target,
                    state: NoticeState::Held { revision: watched.revisions.hold },
                    words: Box::new([]),
                }),
                Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => {}
            }
        }
        SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } | SubscriptionKind::Topic { .. } => {}
    }
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn unsubscribe(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    subscription: u64,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let task = record(domain, number).expect("subscriber live");
    let Some(removed) = find(task, subscription) else {
        return refused(to, Some(number), Refusal::Subscription, out);
    };
    let row = task_mut(domain, number).expect("subscriber live");
    forget(&mut row.record, subscription);
    let mut inbox = List::with_capacity(env.limits.inbox_messages);
    for word in &row.record.inbox {
        if hint_of(&word.kind) != Some(subscription) {
            inbox.push(word.clone()).expect("inbox subset bounded");
        }
    }
    row.record.inbox = inbox.into_boxed();
    release_timer(domain, removed);
    publish(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

fn forget(task: &mut crate::TaskRecord, subscription: u64) {
    let mut next = List::with_capacity(u32::try_from(task.subscriptions.len()).expect("bounded subscriptions"));
    for old in &task.subscriptions {
        if old.number != subscription {
            next.push(*old).expect("subscription subset bounded");
        }
    }
    task.subscriptions = next.into_boxed();
}

pub(crate) fn reading_lost(domain: &Domain, project: u32, parties: &[u64], out: &mut Queue<Request>) {
    let mut lost = Set::with_capacity(u32::try_from(parties.len()).expect("bounded previous roster"));
    for party in parties {
        assert!(lost.insert(*party).is_ok(), "one set place per previous holder");
    }
    for (number, _) in &domain.names {
        let task = record(domain, *number).expect("indexed subscriber");
        if task.project != project || task.subscriptions.is_empty() {
            continue;
        }
        let Some(root) = record(domain, task.root) else { continue };
        match root.requester {
            crate::Party::Person(party) if lost.contains(&party) => {}
            crate::Party::Person(_) | crate::Party::Task(_) | crate::Party::Deployment { .. } => continue,
        }
        for subscription in &task.subscriptions {
            match subscription.kind {
                SubscriptionKind::Task { target, .. } if !crate::refs::allows(domain, *number, target) => {
                    out.push(Request::Notify {
                        task: *number,
                        subscription: subscription.number,
                        target,
                        state: NoticeState::Unreadable,
                        words: Box::new([]),
                    });
                }
                SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } | SubscriptionKind::Topic { .. } => {}
            }
        }
    }
}

pub(crate) fn release_timer(domain: &mut Domain, subscription: Subscription) {
    match subscription.kind {
        SubscriptionKind::Timer { .. } => {
            domain.timers.cancel(subscription.number);
            domain.timer_subscriptions = domain.timer_subscriptions.checked_sub(1).expect("live timer held room");
        }
        SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => {}
    }
}

fn matches_kind(kind: &MessageKind, subscription: &SubscriptionKind) -> bool {
    match crate::message_family(kind) {
        MessageFamily::Hint(Hint::Notice { target, .. }) => match subscription {
            SubscriptionKind::Task { target: watched, .. } => target == *watched,
            SubscriptionKind::Timer { .. } | SubscriptionKind::Topic { .. } => false,
        },
        MessageFamily::Hint(Hint::Timer { .. }) => match subscription {
            SubscriptionKind::Timer { .. } => true,
            SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => false,
        },
        MessageFamily::Hint(Hint::News { class, .. }) => match subscription {
            SubscriptionKind::Topic { .. } => class != NewsClass::Dropped,
            SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } => false,
        },
        MessageFamily::Spoken(_)
        | MessageFamily::Slot(_)
        | MessageFamily::Separate(_)
        | MessageFamily::Result
        | MessageFamily::Virtual => false,
    }
}

pub(crate) fn notice(domain: &mut Domain, env: &Env<Limits>, number: u64, word: Word, out: &mut Queue<Request>) {
    let Some(task) = record(domain, number) else { return };
    let Some(subscription_number) = hint_of(&word.kind) else { return };
    let subscription = match find(task, subscription_number) {
        Some(subscription) => subscription,
        None => {
            if subscription_number != 0
                || task.tracked.is_none()
                || word.kind != (MessageKind::News { subscription: 0, class: NewsClass::Kept })
                || word.words.len() > skein_lib::length::usize_of(env.limits.message_bytes)
            {
                return;
            }
            Subscription { number: 0, kind: SubscriptionKind::Topic { connector: 0, topic: 0 } }
        }
    };
    let matching = matches_kind(&word.kind, &subscription.kind);
    if !matching
        || word.number == 0
        || word.number <= task.last_message
        || word.words.len() > skein_lib::length::usize_of(env.limits.result_bytes.max(env.limits.message_bytes))
    {
        return;
    }
    let prepared = crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::Merged)
        .expect("subscribed hint has a merged inbox slot");
    let row = task_mut(domain, number).expect("subscriber live");
    row.record.last_message = prepared.word.number;
    row.record.inbox = prepared.inbox;
    // The final notice uses the watch's reserved slot before releasing it.
    match crate::message_family(&prepared.word.kind) {
        MessageFamily::Hint(Hint::Notice { state: NoticeState::Unreadable, .. }) => {
            forget(&mut row.record, subscription_number);
        }
        MessageFamily::Hint(_)
        | MessageFamily::Spoken(_)
        | MessageFamily::Slot(_)
        | MessageFamily::Separate(_)
        | MessageFamily::Result
        | MessageFamily::Virtual => {}
    }
    crate::activity::message(&mut row.record, &prepared.activity_word, env.limits.inbox_messages, out);
    publish(domain, env, number, out);
    crate::wake::after_message(domain, env, number, prepared.previous, prepared.word.clone(), out);
    match subscription.kind {
        SubscriptionKind::Timer { period: Some(period), .. } => {
            let at = Wall::from_nanos(env.wall.as_nanos().saturating_add(period.as_nanos()));
            let row = task_mut(domain, number).expect("subscriber live");
            let mut next =
                List::with_capacity(u32::try_from(row.record.subscriptions.len()).expect("bounded subscriptions"));
            for old in &row.record.subscriptions {
                if old.number == subscription_number {
                    next.push(Subscription {
                        number: old.number,
                        kind: SubscriptionKind::Timer { at, period: Some(period) },
                    })
                    .expect("bounded subscriptions");
                } else {
                    next.push(*old).expect("bounded subscriptions");
                }
            }
            row.record.subscriptions = next.into_boxed();
            publish(domain, env, number, out);
            arm(
                domain,
                env,
                Subscription {
                    number: subscription_number,
                    kind: SubscriptionKind::Timer { at, period: Some(period) },
                },
            );
        }
        SubscriptionKind::Timer { period: None, .. } => {
            let row = task_mut(domain, number).expect("subscriber live");
            let mut next =
                List::with_capacity(u32::try_from(row.record.subscriptions.len()).expect("bounded subscriptions"));
            for old in &row.record.subscriptions {
                if old.number != subscription_number {
                    next.push(*old).expect("bounded subscriptions");
                }
            }
            row.record.subscriptions = next.into_boxed();
            release_timer(domain, subscription);
            publish(domain, env, number, out);
        }
        SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => {}
    }
    crate::closer::ask(domain, number, out);
}

pub(crate) fn notify_state(
    domain: &Domain,
    target: u64,
    state: NoticeState,
    words: &[u8],
    out: &mut Queue<Request>,
) -> bool {
    let mut emitted = false;
    for (number, _) in &domain.names {
        let task = record(domain, *number).expect("indexed subscriber");
        for subscription in &task.subscriptions {
            match subscription.kind {
                SubscriptionKind::Task { target: watched, held, result } if watched == target => {
                    let interested = match state {
                        NoticeState::Held { .. } => held,
                        NoticeState::Done | NoticeState::Failed | NoticeState::Cancelled => result,
                        NoticeState::Unreadable => false,
                    };
                    if interested {
                        out.push(Request::Notify {
                            task: *number,
                            subscription: subscription.number,
                            target,
                            state,
                            words: words.into(),
                        });
                        emitted = true;
                    }
                }
                SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } | SubscriptionKind::Topic { .. } => {}
            }
        }
    }
    emitted
}

pub(crate) fn timer_due(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(subscription) = domain.timers.expire(env.now) else { return };
    for (number, _) in &domain.names {
        let task = record(domain, *number).expect("indexed task");
        if let Some(found) = find(task, subscription) {
            match found.kind {
                SubscriptionKind::Timer { .. } => out.push(Request::Timer { task: *number, subscription }),
                SubscriptionKind::Task { .. } | SubscriptionKind::Topic { .. } => {}
            }
            return;
        }
    }
}

pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>) {
    let numbers = crate::domain::snapshot(domain, env.limits.tasks);
    for number in numbers.into_boxed() {
        let task = record(domain, number).expect("restored task live");
        for subscription in task.subscriptions.clone() {
            arm(domain, env, subscription);
        }
    }
}
