//! Close request admission and job encoding (temper/docs/design/domain/forge/change/README.md, 1;
//! temper/docs/design/domain/worker/jobs.md; temper/docs/design/protocol/tooling.md, 4.2).
use super::{Domain, forge, tasks};
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
