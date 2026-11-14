//! A release and an ended stall exemption read the delegate's durable terminal
//! (temper/docs/design/domain/forge/change/repairs.md, Stalls and A release).
use super::{
    Decision, Delivery, Domain, Env, Limits, ProcedureStep, Record, StepEnd, Token, Work, emit, forge, forge_change,
    tasks,
};
use crate::boundary::Read;
use alloc::boxed::Box;
use jig_domain_core as core;

pub(super) fn is_release(domain: &Domain, task: u64) -> bool {
    let Some(record) = domain.core.tasks.task(task) else { return false };
    let Some(row) = domain.forge.change(task) else { return false };
    if record.due_reason != tasks::Why::Released {
        return false;
    }
    let revision = record.revisions.hold;
    match &row.task_release {
        Some(release) => release.revision != revision,
        None => true,
    }
}

/// Defer this activation only while its historical terminal is being read.
pub(crate) fn start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    context: &tasks::RunContext,
) -> bool {
    let task = context.task;
    let Some((_, _, step)) = domain.core.tasks.procedure_due(task) else { return false };
    let released = is_release(domain, task);
    if released {
        domain
            .forge_delegating
            .insert((task, step), super::ChangeStep { delegate: None, relayed: None, refusal: None, released: true })
            .expect("one release per change step");
        super::super::forge_event(
            domain,
            env,
            decision,
            forge::Event::ReleaseChange {
                task,
                release: Some(forge::TaskRelease {
                    revision: domain.core.tasks.task(task).expect("released task").revisions.hold,
                    at: domain.core.tasks.task(task).expect("released task").phase_since,
                }),
            },
        );
    }
    let Some(row) = domain.forge.change(task) else { return false };
    let Some((child, _)) = row.delegate else { return false };
    let mut stale_question = false;
    for question in &row.questions.open {
        stale_question |= question.asker == child;
    }
    for word in &context.inbox {
        if !released && !stale_question && word.from == tasks::Party::Task(child) && super::change_delegate_status(&word.kind).is_some() {
            return false;
        }
    }
    if domain.core.tasks.task(child).is_some()
        || (!released && row.delegate_status != forge_change::Status::Unknown && !stale_question)
    {
        return false;
    }
    let revision = domain.core.tasks.task(task).expect("due change").revisions.hold;
    let read = Read::ChangeDelegate { task, child, step, revision };
    match domain.archive_reads.insert(Some(read)) {
        Ok(waiter) => emit(
            decision,
            &env.limits,
            Delivery::Load {
                waiter: waiter.token(),
                range: crate::Range::Core(crate::CoreRange::TaskResult { task: child }),
                after: None,
            },
        ),
        Err(_) => retry(domain, env, task, step),
    }
    true
}

fn retry(domain: &mut Domain, env: &Env<Limits>, task: u64, step: u64) {
    if let Some(row) = domain.forge.change(task) {
        let bound = skein_lib::Wall::from_nanos(
            row.change.since.as_nanos().saturating_add(env.limits.forge.change_policy.stall.as_nanos()),
        );
        if env.wall >= bound {
            if let Some(context) = domain.core.tasks.run_context(task)
                && !super::start_change(domain, env, &context)
            {
                super::start_refused(domain, task);
            }
            return;
        }
    }
    let until = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(1_000_000_000));
    domain.forge.wait_change(&env.with(env.limits.forge), task, Some(until));
    domain.work.push(Work::Core(core::Event::ProcedureStep {
        task,
        step,
        connector: domain.config.forge_connector,
        code: super::CHANGE_CODE,
        decision: ProcedureStep::new(StepEnd::Wait),
    }));
}

pub(crate) fn loaded(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    waiter: Token,
    rows: Option<Box<[Record]>>,
) {
    let Some(Read::ChangeDelegate { task, child, step, revision }) = crate::archive::take_read(domain, waiter) else {
        return;
    };
    domain.archive_reads.retire(skein_lib::Id::from_token(waiter));
    let Some(record) = domain.core.tasks.task(task) else { return };
    let Some((_, _, due)) = domain.core.tasks.procedure_due(task) else { return };
    let Some(row) = domain.forge.change(task) else { return };
    let Some((expected, _)) = row.delegate else { return };
    let released_at = match &row.task_release {
        Some(release) => Some(release.at),
        None => None,
    };
    if record.revisions.hold != revision || due != step || expected != child {
        return;
    }
    let Some(rows) = rows else {
        retry(domain, env, task, step);
        return;
    };
    let mut ended = None;
    for row in rows {
        if let Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) = row
            && row.number == child
            && row.requester == tasks::Party::Task(task)
        {
            ended = Some(row);
        }
    }
    let Some(ended) = ended else {
        retry(domain, env, task, step);
        return;
    };
    let tasks::Phase::Ended(ending) = &ended.phase else {
        retry(domain, env, task, step);
        return;
    };
    if let tasks::Ending::Done(tasks::TaskResult::Change { title, words, .. }) = ending {
        super::super::forge_event(
            domain,
            env,
            decision,
            forge::Event::ProducedTitle { task, child, title: title.words.clone(), report: words.clone() },
        );
    }
    let (status, no_change, failed, cancelled, words) = terminal(ending);
    // An end recovered by a release cannot spend that release's fresh allowance.
    let ended_at = match (ended.ended_at, released_at) {
        (Some(at), Some(released)) => Some(at.max(released)),
        (at, None) => at,
        (None, Some(_)) => None,
    };
    super::super::forge_event(
        domain,
        env,
        decision,
        forge::Event::DelegateResult { task, child, ended_at, status, no_change, failed, cancelled, words },
    );
    if let Some(context) = domain.core.tasks.run_context(task)
        && !super::start_recovered_change(domain, env, &context, Some((child, ended_at)))
    {
        super::start_refused(domain, task);
    }
}

fn terminal(ending: &tasks::Ending) -> (forge_change::Status, bool, bool, bool, Box<[u8]>) {
    use forge_change::Status::{Failed, Passed};
    match ending {
        tasks::Ending::Done(result) => match result {
            tasks::TaskResult::Change { words, .. }
            | tasks::TaskResult::Report { words }
            | tasks::TaskResult::Verdict { code: 1, words, .. } => (Passed, false, false, false, words.clone()),
            tasks::TaskResult::Verdict { words, .. } => (Failed, false, false, false, words.clone()),
            tasks::TaskResult::NoChange { words } => (Passed, true, false, false, words.clone()),
            tasks::TaskResult::Failure { reason } => (Failed, false, true, false, reason.clone()),
            tasks::TaskResult::Job { bytes } => (Passed, false, false, false, bytes.clone()),
            tasks::TaskResult::Approved { words, .. } => {
                (Failed, false, false, false, words.clone().unwrap_or_default())
            }
        },
        tasks::Ending::Failed { reason } => (Failed, false, true, false, reason.clone()),
        tasks::Ending::Cancelled { reason, .. } => (Failed, false, false, true, reason.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recovered_negative_verdict_is_still_negative() {
        let ending = tasks::Ending::Done(tasks::TaskResult::Verdict {
            code: 2,
            fields: Box::new([]),
            words: b"Needs a repair".as_slice().into(),
        });
        assert_eq!(
            terminal(&ending),
            (forge_change::Status::Failed, false, false, false, b"Needs a repair".as_slice().into())
        );
    }
}
