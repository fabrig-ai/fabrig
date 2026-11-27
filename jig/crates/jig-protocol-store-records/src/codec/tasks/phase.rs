//! Task lifecycle values and bounded creation templates (protocol/store/records.md, 1).
use super::Context;
use super::value;
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::List;

/// Translate `Hold` to its store value.
#[must_use]
pub fn hold(source: &domain::Hold) -> wire::Hold {
    mapped::hold_to_record(source)
}
/// Read `Hold` from its store value.
#[must_use]
pub fn read_hold(source: wire::Hold) -> domain::Hold {
    mapped::hold_to_domain(source)
}

/// Translate `Phase` to its store value.
pub fn phase(source: &domain::Phase, limits: &wire::Limits) -> Result<wire::Phase, wire::Problem> {
    mapped::phase_to_record(source, &Context::new(limits, None))
}
/// Read `Phase` from its store value.
pub fn read_phase(source: wire::Phase, limits: &wire::Limits) -> Result<domain::Phase, wire::Problem> {
    mapped::phase_to_domain(source, &Context::new(limits, None))
}

/// Translate `Grace` to its store record.
#[must_use]
pub fn grace(source: &domain::Grace) -> wire::Grace {
    mapped::grace_to_record(source)
}
/// Read `Grace` from its store record.
#[must_use]
pub fn read_grace(source: wire::Grace) -> domain::Grace {
    mapped::grace_to_domain(source)
}

/// Translate `Revisions` to its store record.
#[must_use]
pub fn revisions(source: &domain::Revisions) -> wire::Revisions {
    mapped::revisions_to_record(source)
}
/// Read `Revisions` from its store record.
#[must_use]
pub fn read_revisions(source: wire::Revisions) -> domain::Revisions {
    mapped::revisions_to_domain(source)
}

/// Translate `RetryNotice` to its store value.
#[must_use]
pub fn retry_notice(source: &domain::RetryNotice) -> wire::RetryNotice {
    mapped::retry_notice_to_record(source)
}
/// Read `RetryNotice` from its store value.
#[must_use]
pub fn read_retry_notice(source: wire::RetryNotice) -> domain::RetryNotice {
    mapped::retry_notice_to_domain(source)
}

pub(super) fn references(source: &[u64], cap: u32, path: wire::Path) -> Result<List<wire::Number>, wire::Problem> {
    let mut members = List::with_capacity(cap);
    for number in source {
        let number = wire::Number { value: *number };
        if members.push(number).is_err() {
            return Err(problem(path));
        }
    }
    Ok(members)
}
pub(super) fn read_references(source: List<wire::Number>) -> Box<[u64]> {
    let mut members = List::with_capacity(source.len());
    for number in source.into_boxed() {
        members.push(number.value).expect("same number of references");
    }
    members.into_boxed()
}
pub(super) fn member(
    source: &domain::New,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Member, wire::Problem> {
    let context = &Context::new(limits, None);
    let domain::New {
        label,
        flags,
        number,
        project,
        executor,
        llm,
        spec,
        contract,
        authority,
        deadline,
        numbers: allotment,
        funder,
        dependencies,
        holdings,
        wake: policy,
        recurring: _,
        tracked,
    } = source;
    let mut resources = List::with_capacity(limits.member_holdings);
    for resource in holdings {
        let resource = mapped::holding_to_record(resource, context)?;
        if resources.push(resource).is_err() {
            return Err(problem(wire::paths::MEMBER_HOLDINGS));
        }
    }
    Ok(wire::Member {
        label: *label,
        flags: mapped::flags_to_record(flags),
        number: *number,
        project: *project,
        executor: mapped::executor_to_record(executor),
        llm: value::llm(llm.as_ref())?,
        spec: value::spec(spec, limits)?,
        contract: value::contract(contract, limits)?,
        authority: value::authority(authority, limits, moved)?,
        deadline: mapped::deadline_to_record(deadline),
        numbers: mapped::numbers_to_record(allotment),
        funder: mapped::funder_to_record(funder),
        dependencies: references(dependencies, limits.member_dependencies, wire::paths::MEMBER_DEPENDENCIES)?,
        holdings: resources,
        wake: mapped::wake_policy_to_record(policy),
        tracked: *tracked,
    })
}
pub(super) fn read_member(source: wire::Member, limits: &wire::Limits) -> Result<domain::New, wire::Problem> {
    let context = &Context::new(limits, None);
    let wire::Member {
        label,
        flags,
        number,
        project,
        executor,
        llm,
        spec,
        contract,
        authority,
        deadline,
        numbers,
        funder,
        dependencies,
        holdings,
        wake: policy,
        tracked,
    } = source;
    let mut resources = List::with_capacity(holdings.len());
    for resource in holdings.into_boxed() {
        resources.push(mapped::holding_to_domain(resource, context)?).expect("same number of holdings");
    }
    Ok(domain::New {
        deadline: mapped::deadline_to_domain(deadline),
        label,
        flags: mapped::flags_to_domain(flags),
        number,
        project,
        executor: mapped::executor_to_domain(executor),
        llm: value::read_llm(llm)?,
        spec: value::read_spec(spec, limits)?,
        contract: value::read_contract(contract, limits)?,
        authority: value::read_authority(authority, limits)?,
        numbers: mapped::numbers_to_domain(numbers),
        funder: mapped::funder_to_domain(funder),
        dependencies: read_references(dependencies),
        holdings: resources.into_boxed(),
        wake: mapped::wake_policy_to_domain(policy),
        recurring: None,
        tracked,
    })
}
/// Translate one creation, including its optional one-level template.
pub fn new(
    source: &domain::New,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Creation, wire::Problem> {
    let recurring = match &source.recurring {
        Some(value) => Some(template(value, limits, moved)?),
        None => None,
    };
    Ok(wire::Creation { member: member(source, limits, moved)?, recurring })
}
/// Read one creation, including its optional one-level template.
pub fn read_new(source: wire::Creation, limits: &wire::Limits) -> Result<domain::New, wire::Problem> {
    let wire::Creation { member, recurring } = source;
    let mut source = read_member(member, limits)?;
    source.recurring = match recurring {
        Some(value) => Some(Box::new(read_template(value, limits)?)),
        None => None,
    };
    Ok(source)
}
/// Translate a recurring batch, refusing a recurring member.
pub fn template(
    source: &domain::RecurringTemplate,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::RecurringTemplate, wire::Problem> {
    let domain::RecurringTemplate { key, batch, overlap: rule } = source;
    let members = super::pieces::members(
        batch,
        limits,
        limits.recurring_template_batch,
        wire::paths::RECURRING_TEMPLATE_BATCH,
        moved,
    )?;
    Ok(wire::RecurringTemplate { key: *key, batch: members, overlap: mapped::recurring_overlap_to_record(rule) })
}
/// Read a recurring batch whose schema members cannot recur.
pub fn read_template(
    source: wire::RecurringTemplate,
    limits: &wire::Limits,
) -> Result<domain::RecurringTemplate, wire::Problem> {
    let wire::RecurringTemplate { key, batch, overlap } = source;
    let members = super::pieces::read_members(batch, limits, wire::paths::RECURRING_TEMPLATE_BATCH)?;
    Ok(domain::RecurringTemplate { key, batch: members, overlap: mapped::recurring_overlap_to_domain(overlap) })
}
/// Translate a result within its combined byte bound.
pub fn result(source: &domain::TaskResult, limits: &wire::Limits) -> Result<wire::TaskResult, wire::Problem> {
    check_result(source, limits)?;
    Ok(match source {
        domain::TaskResult::Job { bytes } => wire::TaskResult::Job(wire::JobResult { bytes: bytes.clone() }),
        domain::TaskResult::Approved { state, words } => {
            wire::TaskResult::Approved(wire::Approved { state: state.clone(), words: words.clone() })
        }
        domain::TaskResult::NoChange { words } => {
            wire::TaskResult::NoChange(wire::ReportResult { words: words.clone() })
        }
        domain::TaskResult::Report { words } => wire::TaskResult::Report(wire::ReportResult { words: words.clone() }),
        domain::TaskResult::Verdict { code, fields, words } => {
            let members = super::pieces::fields(
                fields,
                limits,
                limits.verdict_result_fields,
                wire::paths::VERDICT_RESULT_FIELDS,
            )?;
            wire::TaskResult::Verdict(wire::VerdictResult { code: *code, fields: members, words: words.clone() })
        }
        domain::TaskResult::Change { connector, kind, resource, title, words } => {
            wire::TaskResult::Change(wire::ChangeResult {
                connector: *connector,
                kind: *kind,
                resource: *resource,
                title: title.words.clone(),
                words: words.clone(),
            })
        }
        domain::TaskResult::Failure { reason } => {
            wire::TaskResult::Failure(wire::FailureResult { reason: reason.clone() })
        }
    })
}
/// Read a result within its combined byte bound.
pub fn read_result(source: wire::TaskResult, limits: &wire::Limits) -> Result<domain::TaskResult, wire::Problem> {
    let source = match source {
        wire::TaskResult::Job(value) => domain::TaskResult::Job { bytes: value.bytes },
        wire::TaskResult::Approved(value) => {
            let wire::Approved { state, words } = value;
            domain::TaskResult::Approved { state, words }
        }
        wire::TaskResult::NoChange(value) => domain::TaskResult::NoChange { words: value.words },
        wire::TaskResult::Report(value) => domain::TaskResult::Report { words: value.words },
        wire::TaskResult::Verdict(value) => {
            let wire::VerdictResult { code, fields, words } = value;
            let members = super::pieces::read_fields(fields, limits, wire::paths::VERDICT_RESULT_FIELDS)?;
            domain::TaskResult::Verdict { code, fields: members, words }
        }
        wire::TaskResult::Change(value) => {
            let wire::ChangeResult { connector, kind, resource, title, words } = value;
            domain::TaskResult::Change { connector, kind, resource, title: domain::ChangeTitle::new(title), words }
        }
        wire::TaskResult::Failure(value) => domain::TaskResult::Failure { reason: value.reason },
    };
    check_result(&source, limits)?;
    Ok(source)
}
fn check_result(source: &domain::TaskResult, limits: &wire::Limits) -> Result<(), wire::Problem> {
    let bytes = match source {
        domain::TaskResult::Job { bytes } => bytes.len(),
        domain::TaskResult::Approved { state, words } => state.len().saturating_add(match words {
            Some(value) => value.len(),
            None => 0,
        }),
        domain::TaskResult::Verdict { words, fields, .. } => {
            let mut bytes = words.len();
            for field in fields {
                bytes = bytes.saturating_add(field.name.len()).saturating_add(field.value.len());
            }
            bytes
        }
        domain::TaskResult::NoChange { words } | domain::TaskResult::Report { words } => words.len(),
        domain::TaskResult::Change { title, words, .. } => title.words.len().max(words.len()),
        domain::TaskResult::Failure { reason } => reason.len(),
    };
    if bytes > skein_lib::length::usize_of(limits.result_field_value) {
        return Err(problem(wire::paths::TASK_RESULT_TAG));
    }
    Ok(())
}
fn problem(path: wire::Path) -> wire::Problem {
    wire::Problem { path, reason: skein_codec::Reason::Bound }
}
