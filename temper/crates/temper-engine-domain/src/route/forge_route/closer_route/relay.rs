//! Inbox fences and admitted messages for the closer's relay (temper/docs/design/domain/forge/branches.md, 2).
use super::{Box, Domain, Env, Limits, List, ProcedureStep, StepEnd, c, forge, tasks};
use c::relay::Form;
use jig_domain_core::StepMessage;

pub(super) fn apply(
    domain: &Domain,
    env: &Env<Limits>,
    row: &mut forge::CloserRow,
    decision: &mut ProcedureStep,
) -> (bool, bool) {
    let record = domain.core.tasks.task(row.task).expect("live closer");
    let offered = domain.core.tasks.procedure_offered(row.task).unwrap_or(0);
    let closer = match domain.forge.closer(row.task) {
        Some(previous) => &previous.closer,
        None => &row.closer,
    };
    let candidates = c::relay::steps(closer);
    let mut open = List::with_capacity(2);
    for candidate in &candidates {
        match candidate {
            Some(step) if !ended(domain, step.change) => open.push(*step).expect("at most two open steps"),
            Some(_) | None => {}
        }
    }
    let mut took_answer = false;
    for word in &record.inbox {
        if word.number > offered {
            continue;
        }
        let tasks::Party::Task(from) = word.from else { continue };
        let Some(form) = form(&word.kind) else { continue };
        took_answer |=
            c::relay::take(&mut row.relay, open.as_slice(), row.task, row.goal, from, word.number, &form, &word.words);
    }
    let done = match &row.relay.open {
        Some(open) => ended(domain, open.asker),
        None => false,
    };
    let batch = env.limits.tasks.batch;
    let used = u32::try_from(decision.messages.len()).expect("step messages fit batch");
    let room = c::relay::Room { left: batch.checked_sub(used).expect("step messages fit batch"), batch };
    let relayed =
        c::relay::relay(&row.relay, row.goal, done, room, env.limits.tasks.message_bytes, env.wall, took_answer);
    // The held notice spends its question's room, not one of the ordinary slots.
    let mut messages = List::with_capacity(batch.saturating_add(u32::from(relayed.notice.is_some())));
    for message in core::mem::replace(&mut decision.messages, Box::new([])) {
        messages.push(message).expect("step messages fit batch");
    }
    for message in relayed.messages {
        let kind = match message.form {
            Form::Words => tasks::MessageKind::Words,
            Form::Question => tasks::MessageKind::Question,
            Form::Answer { question } => tasks::MessageKind::Answer { question },
        };
        messages
            .push(StepMessage { notice: None, target: message.target, kind, words: message.words })
            .expect("relay uses remaining message room");
    }
    if let Some(notice) = relayed.notice {
        messages
            .push(StepMessage {
                notice: Some(notice.question),
                target: row.goal,
                kind: tasks::MessageKind::Words,
                words: notice.words,
            })
            .expect("question notice has separate room");
    }
    row.relay = relayed.relay;
    decision.messages = messages.into_boxed();
    // Reporting waits until the durable question and its last notice have settled.
    if row.done && (row.relay.open.is_some() || !row.relay.waiting.is_empty() || !row.relay.replies.is_empty()) {
        row.done = false;
        decision.end = StepEnd::Wait;
    }
    (relayed.asked, relayed.more)
}
fn ended(domain: &Domain, task: u64) -> bool {
    match domain.core.tasks.task(task) {
        Some(row) => match row.phase {
            tasks::Phase::Closing(_)
            | tasks::Phase::Held { was: tasks::Was::Closing(_), .. }
            | tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => false,
        },
        None => true,
    }
}
fn form(kind: &tasks::MessageKind) -> Option<Form> {
    match kind {
        tasks::MessageKind::Question => Some(Form::Question),
        tasks::MessageKind::Answer { question } => Some(Form::Answer { question: *question }),
        tasks::MessageKind::Words => Some(Form::Words),
        tasks::MessageKind::Result(_)
        | tasks::MessageKind::Escalation { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::PassedQuestion { .. }
        | tasks::MessageKind::PassedAnswer { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::News { .. } => None,
    }
}
