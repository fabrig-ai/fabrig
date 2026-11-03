//! A question's read-only delegate and its brief (temper/docs/design/domain/forge.md, 8.11).
use super::{Box, Domain, Env, Limits, List, append, forge_change as change, tasks};

pub(super) fn is_task(questions: &change::Questions, task: u64) -> bool {
    for question in &questions.open {
        if question.stands == (change::Stands::Foreman { task: Some(task) }) {
            return true;
        }
    }
    false
}

pub(super) fn delegate(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    question: &change::Open,
) -> Option<super::super::Delegate> {
    let row = domain.forge.change(task)?;
    let context = domain.core.tasks.delegation(task)?;
    let mut authority = super::child_authority(&context)?;
    super::gate_authority(domain, env, &mut authority, change::GateKind::Agent)?;
    // Keep the question within the core's task section, including its cut marker.
    let brief = env.limits.brief;
    let bound = env
        .limits
        .tasks
        .spec_bytes
        .min(brief.brief_bytes.saturating_sub(32))
        .min(brief.budgets.task.saturating_sub(32))
        .min(brief.read_bytes.saturating_sub(32));
    let words = words(row, &domain.core.tasks.task(task)?.spec.words, question, bound)?;
    let cap = env.limits.tasks.message_bytes.saturating_sub(64).min(env.limits.tasks.result_bytes);
    let mut choices = List::with_capacity(3);
    for code in [1, 2, 3] {
        choices.push(tasks::Verdict { code, words: cap, followups: 0, fields: Box::new([]) }).ok()?;
    }
    Some(super::super::Delegate {
        deadline: tasks::Deadline::Named,
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: super::people::Budget::Named,
        executor: tasks::Executor::Agent { charter: row.foreman_charter },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words,
            parameters: Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: domain.config.forge_connector,
                    resource: super::forge::tag(row.repository.forge, row.repository.repository),
                },
                tasks::Parameter::Bytes { name: 2, value: row.base.clone() },
            ]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Verdict { choices: choices.into_boxed() },
        authority,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
        llm: None,
    })
}

fn words(row: &super::forge::ChangeRow, change: &[u8], question: &change::Open, bound: u32) -> Option<Box<[u8]>> {
    let mut prefix = List::uncapped();
    append(&mut prefix, b"You are the foreman of change task ");
    append(&mut prefix, skein_lib::Decimal::of(row.task).as_bytes());
    append(&mut prefix, b", a question from task ");
    append(&mut prefix, skein_lib::Decimal::of(question.asker).as_bytes());
    append(&mut prefix, b" (its question ");
    append(&mut prefix, skein_lib::Decimal::of(question.number).as_bytes());
    append(&mut prefix, b").\n\nThe change's words:\n");
    let mut suffix = List::uncapped();
    append(&mut suffix, b"\nThe question:\n");
    append(&mut suffix, &question.words);
    // The branch read knows a producer's push before its producing task ends.
    match row.result.head.or(row.change.last_head) {
        Some(head) => {
            append(&mut suffix, b"\nThe change's branch: ");
            append(&mut suffix, &row.branch);
            append(&mut suffix, b", pushed at ");
            append(&mut suffix, &super::commit_text(&head));
            append(&mut suffix, b".");
        }
        None => append(&mut suffix, b"\nThe change has pushed nothing yet."),
    }
    append(&mut suffix, b"\nAnswer with a verdict: answer, finish or escalate.");
    // Parameters count toward the spec; only the change's words may be shortened.
    let room = usize::try_from(bound).ok()?.checked_sub(row.base.len())?;
    let left = room.checked_sub(prefix.as_slice().len())?.checked_sub(suffix.as_slice().len())?;
    if !change.is_empty() && left < "…".len() {
        return None;
    }
    let change = super::forge::text::fit(change, left, "…".as_bytes());
    let mut words = List::with_capacity(u32::try_from(room).ok()?);
    append(&mut words, prefix.as_slice());
    append(&mut words, &change);
    append(&mut words, suffix.as_slice());
    Some(words.into_boxed())
}

#[expect(clippy::wildcard_enum_match_arm, reason = "only the foreman contract terminals settle a question")]
pub(super) fn settled<'a>(
    row: &'a tasks::TaskRecord,
    offered: u64,
    questions: &change::Questions,
) -> Option<(u64, change::Settled<'a>)> {
    for word in &row.inbox {
        let tasks::Party::Task(task) = word.from else { continue };
        if word.number > offered || !is_task(questions, task) {
            continue;
        }
        let settled = match word.kind {
            tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 1, .. }) => {
                change::Settled::Answer(&word.words)
            }
            tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, .. }) => {
                change::Settled::Finish(&word.words)
            }
            tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 3, .. }) => {
                change::Settled::Escalate(&word.words)
            }
            tasks::MessageKind::Result(tasks::ResultKind::Failed) => change::Settled::Failed(&word.words),
            tasks::MessageKind::Result(tasks::ResultKind::Cancelled) => {
                change::Settled::Failed(b"the foreman was cancelled")
            }
            _ => continue,
        };
        return Some((task, settled));
    }
    None
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "delegation authority refusals get specific words; other step refusals pass up as admission failures"
)]
pub(super) fn refusal_words(why: tasks::Refusal) -> &'static [u8] {
    match why {
        tasks::Refusal::Tree => b"no remaining task authority",
        tasks::Refusal::Depth => b"no remaining delegation depth",
        tasks::Refusal::Funding => b"insufficient spend for the foreman",
        tasks::Refusal::Executor => b"foreman charter unavailable",
        tasks::Refusal::Spec => b"foreman brief exceeds its configured bounds",
        tasks::Refusal::Contract => b"foreman verdict exceeds its configured bounds",
        _ => b"foreman admission exceeds the task's current capacity or authority",
    }
}
