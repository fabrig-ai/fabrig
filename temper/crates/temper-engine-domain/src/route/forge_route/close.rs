//! Close request admission and job encoding (temper/docs/design/domain/forge/change/README.md, 1;
//! temper/docs/design/domain/worker/jobs.md; temper/docs/design/protocol/tooling.md, 4.2).
use super::{Domain, List, Work, append, decimal, forge, forge_change, tasks};
use alloc::boxed::Box;
use skein_lib::{Writer, bytes::copy_of};
use temper_protocol_worker_wire::v1;

pub(super) fn spec(repository: &[u8], command: &[u8], request: &[u8], bound: u32) -> Option<Box<[u8]>> {
    let length = repository.len().checked_add(1)?.checked_add(command.len())?;
    let mut path = Writer::new(length);
    path.put(repository).ok()?;
    path.put(b"/").ok()?;
    path.put(command).ok()?;
    let spec =
        v1::JobSpec::new(&v1::CEILINGS, v1::JobSpecParts { command: path.finish(), request: copy_of(request) }).ok()?;
    if spec.measure() > bound {
        return None;
    }
    let mut writer = Writer::new(usize::try_from(spec.measure()).ok()?);
    spec.encode(&mut writer).ok()?;
    Some(writer.finish())
}

pub(super) fn refusal(domain: &Domain, context: &tasks::RunContext) -> Option<&'static [u8]> {
    let mut request = None;
    let mut branch = false;
    let mut catch_up = false;
    let mut repository = None;
    for parameter in &context.spec.parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 17, value } => request = Some(value.as_ref()),
            tasks::Parameter::Bytes { name: 3, .. } | tasks::Parameter::Number { name: 11, value: 1 } => branch = true,
            tasks::Parameter::Bytes { name: 16, .. } => catch_up = true,
            tasks::Parameter::Resource { name: 1, connector, resource }
                if *connector == domain.config.forge_connector =>
            {
                repository = domain.forge.repository_tag(context.project, *resource);
            }
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    let request = request?;
    if branch || catch_up || !request.starts_with(b"{\"version\":1,\"goal\":") {
        return Some(b"malformed close request parameter 17.");
    }
    let Some(command) = domain.forge.close_job(context.project) else {
        return Some(b"change's close: the project names no close_job.");
    };
    let Some(bound) = domain.config.close_kind else {
        return Some(b"change's close: job kind 4 is not declared.");
    };
    if !context.authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Job(forge::CLOSE_KIND)) {
        return Some(b"change's close: its authority does not delegate to job kind 4.");
    }
    if let Some(repository) = repository
        && spec(&repository.name, command, request, bound.spec).is_none()
    {
        return Some(b"malformed close request parameter 17.");
    }
    None
}

// A close job uses no model and reserves none of its change's spend
// (temper/docs/design/domain/forge/change/repairs.md).
pub(super) fn authority(context: &tasks::DelegationContext) -> tasks::Authority {
    let mut authority = context.authority.clone();
    authority.budget.spend = 0;
    authority.delegation.kinds = Box::new([]);
    authority.delegation.tasks = 0;
    authority.delegation.depth = 0;
    authority
}

// Once jig makes a replacement, cancel the held job before the replacement
// starts (temper/docs/design/domain/forge/change/repairs.md).
pub(crate) fn made(domain: &mut Domain, task: u64, fresh: u64) {
    let Some(row) = domain.forge.change(task) else { return };
    let Some((child, forge_change::Delegate::Close)) = row.delegate else { return };
    if child == fresh {
        return;
    }
    let Some(child_row) = domain.core.tasks.task(child) else { return };
    if let tasks::Phase::Held { why: tasks::Hold::Failures(_), .. } = child_row.phase {
        let mut reason = List::with_capacity(96);
        append(&mut reason, b"superseded by its change's fresh close job, task ");
        append(&mut reason, &decimal(fresh));
        domain.work.push(Work::Tasks(tasks::Event::Cancel {
            reply_to: super::super::internal(u64::MAX),
            by: tasks::Party::Task(task),
            operation: tasks::Cancel { task: child, reason: reason.into_boxed(), grace: None, message: None },
        }));
    }
}

fn failed_words(phase: &tasks::Phase) -> Option<&[u8]> {
    let tasks::Phase::Held { why: tasks::Hold::Failures(_), source: Some(source), .. } = phase else { return None };
    let tasks::HoldSource::Preparation { words } = source.as_ref() else { return None };
    Some(words)
}

pub(super) fn failure_words(domain: &Domain, child: u64) -> Option<&[u8]> {
    failed_words(&domain.core.tasks.task(child)?.phase)
}

fn held_change(domain: &Domain, requester: tasks::Party, child: u64) -> Option<u64> {
    let tasks::Party::Task(task) = requester else { return None };
    let row = domain.forge.change(task)?;
    if row.delegate != Some((child, forge_change::Delegate::Close)) || !super::own_hold(domain.core.tasks.task(task)?) {
        return None;
    }
    let forge_change::State::Held { was, why: forge_change::Hold::Failed } = &row.change.state else { return None };
    if **was != (forge_change::State::Producing { requested: true }) {
        return None;
    }
    Some(task)
}

// A held procedure has no due step to read its inbox. Keep its close job's
// terminal, or its next failure hold, as the core commits it (repairs.md).
pub(crate) fn saved(domain: &mut Domain, record: &jig_domain_core::Record) {
    if let jig_domain_core::Record::Tasks(tasks::Stored::Ended(child)) = record {
        let Some(task) = held_change(domain, child.requester, child.number) else { return };
        let Some(context) = domain.core.tasks.run_context(task) else { return };
        super::change_results(domain, &context);
    } else if let jig_domain_core::Record::Tasks(tasks::Stored::Milestone(milestone)) = record {
        let Some(words) = failed_words(&milestone.phase) else { return };
        let Some(child) = domain.core.tasks.task(milestone.task) else { return };
        let Some(task) = held_change(domain, child.requester, child.number) else { return };
        domain.work.push(Work::Forge(forge::Event::DelegateResult {
            task,
            child: child.number,
            status: forge_change::Status::Failed,
            no_change: false,
            failed: true,
            cancelled: false,
            words: words.into(),
        }));
    }
}

// Exhausted job tries hold the producer when its escalation reaches the change
// (temper/docs/design/domain/forge/change/repairs.md).
pub(super) fn failed(domain: &Domain, child: u64, kind: &tasks::MessageKind) -> bool {
    let Some(task) = domain.core.tasks.task(child) else { return false };
    if *kind != (tasks::MessageKind::Escalation { task: child, revision: task.revisions.hold }) {
        return false;
    }
    match task.phase {
        tasks::Phase::Held { why: tasks::Hold::Failures(_), .. } => true,
        tasks::Phase::Held { .. }
        | tasks::Phase::Active(_)
        | tasks::Phase::Closing(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Ended(_) => false,
    }
}
