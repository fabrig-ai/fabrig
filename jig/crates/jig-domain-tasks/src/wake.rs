//! Closed wake rules and bounded batch and pace timers (domain/tasks/messages.md, section 3).
//! Policies and unread messages are durable in task rows. Monotonic deadlines
//! are derived from injected wall and monotonic time after restore. Procedures
//! batch news by their policy and step immediately on other messages.
use crate::domain::{Domain, activate, publish, record, task_mut};
use crate::{
    Active, Hint, Limits, MessageFamily, MessageKind, NewsClass, Party, Phase, Request, ResultsWake, Slot, Spoken,
    WakePolicy, WakeRule, Word,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, Queue, Wall};

/// Whether every batch count, batch age and pace span is positive.
#[must_use]
pub fn valid(policy: &WakePolicy) -> bool {
    for rule in [policy.words, policy.notices, policy.news] {
        match rule {
            WakeRule::Never | WakeRule::Immediate => {}
            WakeRule::Batch { count, age } => {
                if count == 0 || age == Duration::ZERO {
                    return false;
                }
            }
            WakeRule::Paced { span } => {
                if span == Duration::ZERO {
                    return false;
                }
            }
        }
    }
    true
}

pub(crate) fn category(kind: &MessageKind) -> u8 {
    match kind.family() {
        MessageFamily::Spoken(Spoken::Words) => 0,
        MessageFamily::Spoken(Spoken::Question) => 1,
        MessageFamily::Spoken(Spoken::Answer { .. }) => 2,
        MessageFamily::Result => 3,
        MessageFamily::Hint(Hint::Notice { .. }) => 4,
        MessageFamily::Hint(Hint::Timer { .. }) => 5,
        MessageFamily::Hint(Hint::News { .. }) | MessageFamily::Slot(Slot::PersonStalled) => 6,
        MessageFamily::Slot(
            Slot::EffectNotMade
            | Slot::SettingsDecision
            | Slot::ProposalDecision
            | Slot::QuestionAnswered
            | Slot::QuestionOwed
            | Slot::QuestionEnded,
        )
        | MessageFamily::Separate(_)
        | MessageFamily::Virtual => 7,
    }
}

fn rule(domain: &Domain, number: u64, word: &Word) -> WakeRule {
    let task = record(domain, number).expect("inbox task live");
    let live_run = match task.phase {
        Phase::Active(Active::Claimed { .. } | Active::Running { .. }) => true,
        Phase::Waiting
        | Phase::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. })
        | Phase::Closing(_)
        | Phase::Held { .. }
        | Phase::Ended(_) => false,
    };
    match &word.kind {
        MessageKind::Words => match word.from {
            Party::Person(_) => WakeRule::Immediate,
            Party::Task(_) | Party::Deployment { .. } => task.wake.words,
        },
        MessageKind::EffectNotMade { .. }
        | MessageKind::FinishRequested { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::Met { .. }
        | MessageKind::QuestionAnswered { .. }
        | MessageKind::QuestionOwed
        | MessageKind::NothingAwaited
        | MessageKind::QuestionEnded { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::Escalation { .. } => WakeRule::Immediate,
        MessageKind::PersonStalled { .. } => task.wake.news,
        MessageKind::Question => {
            if task.wake.questions {
                WakeRule::Immediate
            } else {
                WakeRule::Never
            }
        }
        MessageKind::Answer { .. } => {
            if task.wake.answers || word.eligible {
                WakeRule::Immediate
            } else {
                WakeRule::Never
            }
        }
        MessageKind::Result(result) => match task.wake.results {
            ResultsWake::Never => WakeRule::Never,
            ResultsWake::Each => WakeRule::Immediate,
            ResultsWake::LastOrFailure => {
                result_wake(result, task.delegates.is_empty() && (live_run || !crate::closer::named(task)))
            }
        },
        MessageKind::Notice { .. } => task.wake.notices,
        MessageKind::Timer { .. } => {
            if task.wake.timers {
                WakeRule::Immediate
            } else {
                WakeRule::Never
            }
        }
        MessageKind::News { class, .. } => match class {
            NewsClass::Wakes => task.wake.news,
            NewsClass::Kept | NewsClass::Dropped => WakeRule::Never,
        },
    }
}

/// A live run hears results by its wake policy, without choosing another part
/// (jig/docs/design/domain/tasks/messages.md, 6). Refill judges waiting results again.
pub(crate) fn offered(domain: &Domain, number: u64, word: &Word) -> bool {
    let task = record(domain, number).expect("offered task live");
    if crate::offered(word, task.offered) {
        return true;
    }
    match word.kind.family() {
        MessageFamily::Result => rule(domain, number, word) == WakeRule::Immediate,
        MessageFamily::Spoken(_)
        | MessageFamily::Hint(_)
        | MessageFamily::Slot(_)
        | MessageFamily::Separate(_)
        | MessageFamily::Virtual => false,
    }
}

/// The attempt's kept reasons and the kinds that wake this activation
/// (jig/docs/design/domain/tasks/messages.md, 6).
pub(crate) fn reasons(domain: &Domain, number: u64, inbox: &[Word]) -> crate::Reasons {
    let mut reasons = record(domain, number).expect("activation task live").reasons;
    for word in inbox {
        let wakes = match rule(domain, number, word) {
            WakeRule::Immediate => true,
            WakeRule::Batch { .. } | WakeRule::Paced { .. } => word.eligible,
            WakeRule::Never => false,
        };
        if wakes {
            reasons = reasons.union(crate::Reasons::of(&word.kind));
        }
    }
    reasons
}

/// Kinds that can end a session as unoffered. Always-offered messages still
/// choose parts at a fresh start, but cannot end the session for their kind.
pub(crate) fn session_kinds(domain: &Domain, number: u64, inbox: &[Word]) -> crate::Reasons {
    let mut kinds = crate::Reasons::EMPTY;
    for word in inbox {
        let wakes = match rule(domain, number, word) {
            WakeRule::Immediate => true,
            WakeRule::Batch { .. } | WakeRule::Paced { .. } => word.eligible,
            WakeRule::Never => false,
        };
        if wakes && !crate::offered(word, crate::Reasons::EMPTY) {
            kinds = kinds.union(crate::Reasons::of(&word.kind));
        }
    }
    kinds
}

fn batch_state(domain: &Domain, number: u64, word: &Word) -> (u32, Wall, bool) {
    let task = record(domain, number).expect("inbox task live");
    let mut hits = 0_u32;
    let mut oldest = word.at;
    let mut latched = false;
    for item in &task.inbox {
        if category(&item.kind) == category(&word.kind) {
            hits = hits.saturating_add(item.hits);
            oldest = oldest.min(item.at);
            latched |= item.eligible;
        }
    }
    (hits, oldest, latched)
}

/// Admit root's numbered notice only while the kept task can go on (domain/tasks/lifecycle.md, 8).
pub(crate) fn met(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    (dependency, removed): (u64, bool),
    by: Party,
    message: u64,
    out: &mut Queue<Request>,
) {
    let Some(row) = record(domain, number) else { return };
    match row.phase {
        Phase::Active(_) | Phase::Held { was: crate::Was::Active(_), .. } => {}
        Phase::Waiting
        | Phase::Closing(_)
        | Phase::Held { was: crate::Was::Waiting | crate::Was::Closing(_), .. }
        | Phase::Ended(_) => return,
    }
    if crate::run::ends::for_record(row).blocks_claims() || message == 0 || message <= row.last_message {
        return;
    }
    let word = Word {
        number: message,
        from: by,
        kind: MessageKind::Met { dependency, removed },
        words: Box::new([]),
        at: env.wall,
        hits: 1,
        eligible: false,
    };
    let prepared = crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::Merged)
        .expect("kept task has merged notice room");
    let task = task_mut(domain, number).expect("notice task remains live");
    task.record.last_message = message;
    task.record.inbox = prepared.inbox;
    crate::activity::message(&mut task.record, &prepared.activity_word, env.limits.inbox_messages, out);
    publish(domain, env, number, out);
    after_message(domain, env, number, prepared.previous, prepared.word, out);
}

pub(crate) fn after_message(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    previous: Option<u64>,
    word: Word,
    out: &mut Queue<Request>,
) {
    let decision = match record(domain, number).expect("message task live").executor {
        crate::Executor::Procedure { .. } => match word.kind {
            MessageKind::News { .. } | MessageKind::PersonStalled { .. } => rule(domain, number, &word),
            MessageKind::EffectNotMade { .. }
            | MessageKind::Words
            | MessageKind::FinishRequested { .. }
            | MessageKind::Amendment { .. }
            | MessageKind::Met { .. }
            | MessageKind::QuestionAnswered { .. }
            | MessageKind::QuestionOwed
            | MessageKind::NothingAwaited
            | MessageKind::QuestionEnded { .. }
            | MessageKind::SettingsDecision { .. }
            | MessageKind::ProposalDecision { .. }
            | MessageKind::Proposal { .. }
            | MessageKind::Escalation { .. }
            | MessageKind::Question
            | MessageKind::Answer { .. }
            | MessageKind::Result(_)
            | MessageKind::Notice { .. }
            | MessageKind::Timer { .. } => WakeRule::Immediate,
        },
        crate::Executor::Agent { .. } => rule(domain, number, &word),
        crate::Executor::Person(_) | crate::Executor::Job { .. } => WakeRule::Never,
    };
    let ready = match decision {
        WakeRule::Never => false,
        WakeRule::Immediate => true,
        WakeRule::Batch { count, age } => {
            let (hits, oldest, latched) = batch_state(domain, number, &word);
            let at = oldest.as_nanos().saturating_add(age.as_nanos());
            if latched || hits >= count || env.wall.as_nanos() >= at {
                let task = task_mut(domain, number).expect("batch task live");
                for item in &mut task.record.inbox {
                    if category(&item.kind) == category(&word.kind) {
                        item.eligible = true;
                    }
                }
                publish(domain, env, number, out);
                true
            } else {
                false
            }
        }
        WakeRule::Paced { span } => {
            let row = record(domain, number).expect("paced task live");
            // A procedure has no run start, so its paced news wakes it at once.
            let ready = match row.run_since {
                Some(since) => env.wall.as_nanos() >= since.as_nanos().saturating_add(span.as_nanos()),
                None => true,
            };
            if ready {
                let task = task_mut(domain, number).expect("paced task live");
                for item in &mut task.record.inbox {
                    if category(&item.kind) == category(&word.kind) {
                        item.eligible = true;
                    }
                }
                publish(domain, env, number, out);
            }
            ready
        }
    };
    schedule(domain, env, number);
    if !ready {
        crate::closer::ask(domain, number, out);
        return;
    }
    let row = record(domain, number).expect("batch task live");
    let blocked = !crate::run::ends::kept(domain, number).may_be_due(row.executor);
    let phase = row.phase.clone();
    match phase {
        Phase::Active(Active::Idle) if !blocked => {
            let row = &mut task_mut(domain, number).expect("idle task live").record;
            row.due_reason = crate::Why::Message { message: word.number };
            row.phase = Phase::Active(Active::Due);
            publish(domain, env, number, out);
            activate(domain, number, out);
        }
        Phase::Active(Active::Claimed { attempt } | Active::Running { attempt }) => {
            if !offered(domain, number, &word) {
                return;
            }
            // A run behind its choice hears nothing more: the message starts the next run
            // (domain/tasks/changes.md).
            if row.llm_changed {
                if !row.narrowing {
                    let row = &mut task_mut(domain, number).expect("message task live").record;
                    row.narrowing = true;
                    row.due_reason = crate::Why::Message { message: word.number };
                    publish(domain, env, number, out);
                    out.push(Request::Stop { task: number, attempt });
                }
            } else {
                out.push(Request::Relay { task: number, attempt, previous, word });
            }
        }
        Phase::Waiting
        | Phase::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. })
        | Phase::Closing(_)
        | Phase::Held { .. }
        | Phase::Ended(_) => {}
    }
}

/// News and results arriving after the offered prefix keep their own wake policy.
pub(crate) fn after_step(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    let inbox = record(domain, number).expect("stepped procedure live").inbox.clone();
    schedule(domain, env, number);
    for word in inbox {
        // One activation offers the whole remaining inbox. Stop once it is
        // due so the route's output bound does not grow with its message count.
        if record(domain, number).expect("stepped procedure live").phase != Phase::Active(Active::Idle) {
            return;
        }
        after_message(domain, env, number, None, word, out);
    }
}

/// An attempt's refusals end with it; unread messages keep their wake policy.
pub(crate) fn after_run(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    let task = task_mut(domain, number).expect("answered task live");
    let refused = core::mem::replace(&mut task.record.refused, Box::new([]));
    let offered = task.record.offered;
    if record(domain, number).expect("answered task live").phase == Phase::Active(Active::Idle) {
        after_step(domain, env, number, out);
    }
    if record(domain, number).expect("answered task live").phase != Phase::Active(Active::Idle) {
        return;
    }
    // Only an idle task judges its entries again: a live run already heard them.
    for entries in [crate::proposals::waiting_for(domain, number), crate::escalation::waiting_for(domain, number)] {
        for word in entries {
            if record(domain, number).expect("decision holder live").phase != Phase::Active(Active::Idle) {
                return;
            }
            if crate::reasons::covered(&word, offered) && !refused.contains(&word.number) {
                continue;
            }
            after_message(domain, env, number, None, word, out);
        }
    }
}

pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(number) = domain.wakes.expire(env.now) else { return };
    let Some(task) = record(domain, number) else { return };
    let mut candidate = None;
    for word in &task.inbox {
        match rule(domain, number, word) {
            WakeRule::Batch { age, .. } => {
                let at = word.at.as_nanos().saturating_add(age.as_nanos());
                if !word.eligible && env.wall.as_nanos() >= at {
                    candidate = Some(word.clone());
                    break;
                }
            }
            WakeRule::Paced { span } => {
                let at = match task.run_since {
                    Some(since) => since.as_nanos().saturating_add(span.as_nanos()),
                    None => env.wall.as_nanos(),
                };
                if !word.eligible && env.wall.as_nanos() >= at {
                    candidate = Some(word.clone());
                    break;
                }
            }
            WakeRule::Never | WakeRule::Immediate => {}
        }
    }
    if let Some(word) = candidate {
        after_message(domain, env, number, None, word, out);
    } else {
        schedule(domain, env, number);
    }
}

fn schedule(domain: &mut Domain, env: &Env<Limits>, number: u64) {
    let task = record(domain, number).expect("batch task live");
    let mut oldest: Option<u64> = None;
    for word in &task.inbox {
        match rule(domain, number, word) {
            WakeRule::Batch { age, .. } if !word.eligible => {
                let at = word.at.as_nanos().saturating_add(age.as_nanos());
                oldest = Some(match oldest {
                    Some(previous) => previous.min(at),
                    None => at,
                });
            }
            WakeRule::Paced { span } if !word.eligible => {
                let at = match task.run_since {
                    Some(since) => since.as_nanos().saturating_add(span.as_nanos()),
                    None => env.wall.as_nanos(),
                };
                oldest = Some(match oldest {
                    Some(previous) => previous.min(at),
                    None => at,
                });
            }
            WakeRule::Never | WakeRule::Immediate | WakeRule::Batch { .. } | WakeRule::Paced { .. } => {}
        }
    }
    domain.wakes.cancel(number);
    if let Some(at) = oldest {
        let due = env.now.saturating_add(Duration::from_nanos(at.saturating_sub(env.wall.as_nanos())));
        assert!(domain.wakes.arm(number, due).is_ok(), "batch alarm room");
    }
}

pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>) {
    let numbers = crate::domain::snapshot(domain, env.limits.tasks);
    for number in numbers.into_boxed() {
        schedule(domain, env, number);
    }
}

fn result_wake(result: &crate::ResultKind, no_delegates: bool) -> WakeRule {
    match result {
        crate::ResultKind::Failed | crate::ResultKind::Cancelled => WakeRule::Immediate,
        crate::ResultKind::Approved { .. }
        | crate::ResultKind::Report
        | crate::ResultKind::Verdict { .. }
        | crate::ResultKind::Job
        | crate::ResultKind::NoChange
        | crate::ResultKind::Change { .. } => {
            if no_delegates {
                WakeRule::Immediate
            } else {
                WakeRule::Never
            }
        }
    }
}
