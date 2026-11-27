//! Close request admission and job encoding (temper/docs/design/domain/forge/change/README.md, 1;
//! temper/docs/design/domain/worker/jobs.md; temper/docs/design/protocol/tooling.md, 4.2),
//! and failure retirement (temper/docs/design/domain/forge/change/repairs-releases.md, 1).
use super::{Decision, Domain, Env, Limits, Work, forge, forge_change, tasks};
use alloc::boxed::Box;
use skein_lib::{Writer, bytes::copy_of};
use temper_protocol_worker_wire::v1::{self, Record};

pub(super) fn spec(repository: &[u8], command: &[u8], request: &[u8], bound: u32) -> Option<Box<[u8]>> {
    let length = repository.len().checked_add(1)?.checked_add(command.len())?;
    let mut path = Writer::new(length);
    path.put(repository).ok()?;
    path.put(b"/").ok()?;
    path.put(command).ok()?;
    let spec = v1::JobSpec { command: path.finish(), request: copy_of(request), refusal: true };
    if spec.measure(&v1::CEILINGS).ok()? > bound {
        return None;
    }
    spec.encode(&v1::CEILINGS).ok()
}

pub(super) fn hook(parameters: &[tasks::Parameter]) -> Option<&[u8]> {
    for parameter in parameters {
        if let tasks::Parameter::Bytes { name: 20, value } = parameter {
            return Some(value);
        }
    }
    None
}

pub(super) fn command<'a>(domain: &'a Domain, project: u32, hook: Option<&'a [u8]>) -> Option<&'a [u8]> {
    match hook {
        Some(path) => Some(path),
        None => domain.forge.close_job(project),
    }
}

pub(super) fn refusal(domain: &Domain, context: &tasks::RunContext) -> Option<&'static [u8]> {
    let malformed_hook = b"malformed close hook parameter 20.";
    let mut request = None;
    let mut branch = false;
    let mut catch_up = false;
    let mut repository = None;
    for parameter in &context.spec.parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 17, value } => request = Some(value.as_ref()),
            tasks::Parameter::Bytes { name: 3, .. } | tasks::Parameter::Number { name: 11, value: 1 } => branch = true,
            tasks::Parameter::Bytes { name: 16, .. } => catch_up = true,
            tasks::Parameter::Number { name: 20, .. } | tasks::Parameter::Resource { name: 20, .. } => {
                return Some(malformed_hook);
            }
            tasks::Parameter::Resource { name: 1, connector, resource }
                if *connector == domain.config.forge_connector =>
            {
                repository = domain.forge.repository_tag(context.project, *resource);
            }
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    let hook = hook(&context.spec.parameters);
    if let Some(path) = hook
        && (request.is_none() || !forge::settings::job_path(path))
    {
        return Some(malformed_hook);
    }
    let request = request?;
    if branch || catch_up || !request.starts_with(b"{\"version\":1,\"goal\":") {
        return Some(b"malformed close request parameter 17.");
    }
    let Some(command) = command(domain, context.project, hook) else {
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

// The change, not the failed job, owns the hold.
fn failed_delegate(domain: &Domain, task: u64) -> Option<u64> {
    let row = domain.forge.change(task)?;
    let Some((child, forge_change::Delegate::Close)) = row.delegate else { return None };
    let forge_change::State::Held {
        was,
        hold: forge_change::Hold { cause: forge_change::Cause::Failed | forge_change::Cause::CloseRefused, .. },
    } = &row.change.state
    else {
        return None;
    };
    if **was == (forge_change::State::Producing { requested: true })
        && row.delegate_status == forge_change::Status::Failed
    {
        Some(child)
    } else {
        None
    }
}

pub(super) fn replay_words(at: forge_change::CloseAt, failure: &[u8]) -> Box<[u8]> {
    let mut words = skein_lib::List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    super::append(&mut words, b"not measured again: the goal's branch is at ");
    short_commit(&mut words, &at.branch);
    super::append(&mut words, b" and main at ");
    short_commit(&mut words, &at.landing);
    super::append(&mut words, b", as when its close check refused; it checks again once either moves.\n\n");
    let room = usize::try_from(words.room()).expect("hold bound fits usize");
    super::append(&mut words, &forge::text::fit(failure, room, b""));
    words.into_boxed()
}

fn short_commit(words: &mut skein_lib::List<u8>, commit: &[u8; 32]) {
    for byte in commit.get(..6).expect("six bytes of a commit") {
        super::append(words, &forge::hex(*byte));
    }
}

pub(super) fn retire_on_hold(domain: &mut Domain, task: u64) -> Option<jig_domain_core::StepCancel> {
    let child = failed_delegate(domain, task)?;
    domain.work.push(Work::Forge(forge::Event::RetireFailedClose { task, child }));
    let row = domain.engine.core().tasks.task(child)?;
    if let tasks::Phase::Held { why: tasks::Hold::Failures(_), .. } = row.phase {
        Some(jig_domain_core::StepCancel {
            task: child,
            reason: Box::from(b"its change holds on its failure".as_slice()),
            grace: None,
        })
    } else {
        None
    }
}

pub(super) fn cancel(domain: &mut Domain, task: u64, part: jig_domain_core::StepCancel) {
    domain.work.push(Work::Tasks(tasks::Event::Cancel {
        reply_to: super::super::internal(u64::MAX),
        by: tasks::Party::Task(task),
        operation: tasks::Cancel { task: part.task, reason: part.reason, grace: part.grace, message: None },
    }));
}

// A row saved with the old job still named retires it before the release decides.
pub(super) fn retire_on_release(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, task: u64) {
    let Some(child) = failed_delegate(domain, task) else { return };
    if domain.engine.core().tasks.task(child).is_some() {
        cancel(
            domain,
            task,
            jig_domain_core::StepCancel {
                task: child,
                reason: Box::from(b"its change was released".as_slice()),
                grace: None,
            },
        );
    }
    super::super::forge_event(domain, env, decision, forge::Event::RetireFailedClose { task, child });
}

fn failed_words(phase: &tasks::Phase) -> Option<&[u8]> {
    let tasks::Phase::Held { why, source, .. } = phase else { return None };
    preparation_words(*why, source.as_deref())
}

fn preparation_words(why: tasks::Hold, source: Option<&tasks::HoldSource>) -> Option<&[u8]> {
    let tasks::Hold::Failures(_) = why else { return None };
    let Some(tasks::HoldSource::Preparation { words }) = source else { return None };
    Some(words)
}

pub(super) fn failure_words(domain: &Domain, child: u64) -> Option<&[u8]> {
    failed_words(&domain.engine.core().tasks.task(child)?.phase)
}

// Exhausted job tries hold the producer when its escalation reaches the change
// (temper/docs/design/domain/forge/change/repairs.md).
pub(super) fn failure(domain: &Domain, child: u64, kind: &tasks::MessageKind) -> Option<tasks::Class> {
    let task = domain.engine.core().tasks.task(child)?;
    if *kind != (tasks::MessageKind::Escalation { task: child, revision: task.revisions.hold }) {
        return None;
    }
    match task.phase {
        tasks::Phase::Held { why: tasks::Hold::Failures(class), .. } => Some(class),
        tasks::Phase::Held { .. }
        | tasks::Phase::Active(_)
        | tasks::Phase::Closing(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Ended(_) => None,
    }
}
