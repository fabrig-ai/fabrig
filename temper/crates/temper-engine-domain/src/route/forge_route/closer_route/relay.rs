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
    let live = match domain.forge.closer(row.task) {
        Some(previous) => previous.closer.running,
        None => row.closer.running,
    };
    let live = match live {
        Some(running) if !ended(domain, running.change) => Some(running),
        Some(_) | None => None,
    };
    for word in &record.inbox {
        if word.number > offered {
            continue;
        }
        let tasks::Party::Task(from) = word.from else { continue };
        let Some(form) = form(&word.kind) else { continue };
        c::relay::take(&mut row.relay, live, row.goal, from, word.number, &form, &word.words);
    }
    let done = match &row.relay.open {
        Some(open) => ended(domain, open.asker),
        None => false,
    };
    let mut messages = List::with_capacity(env.limits.tasks.batch);
    for message in core::mem::replace(&mut decision.messages, Box::new([])) {
        messages.push(message).expect("step messages fit batch");
    }
    let relayed = c::relay::relay(&row.relay, row.goal, done, messages.room(), env.limits.tasks.message_bytes);
    for message in relayed.messages {
        let kind = match message.form {
            Form::Words => tasks::MessageKind::Words,
            Form::Question => tasks::MessageKind::Question,
            Form::Answer { question } => tasks::MessageKind::Answer { question },
        };
        messages
            .push(StepMessage { target: message.target, kind, words: message.words })
            .expect("relay uses remaining message room");
    }
    row.relay = relayed.relay;
    decision.messages = messages.into_boxed();
    // Reporting waits until the durable question and its last notice have settled.
    if row.done && (row.relay.open.is_some() || !row.relay.replies.is_empty()) {
        row.done = false;
        decision.end = StepEnd::Wait;
    }
    (relayed.asked, relayed.more)
}
fn ended(domain: &Domain, task: u64) -> bool {
    match domain.core.tasks.task(task) {
        Some(row) => match row.phase {
            tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => false,
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
