//! Funding and resource values shared by tasks rows (protocol/store/records.md, 1).
use super::Context;
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{List, Wall};

pub(super) fn wall(value: Option<Wall>) -> Option<u64> {
    match value {
        Some(value) => Some(value.as_nanos()),
        None => None,
    }
}
pub(super) fn read_wall(value: Option<u64>) -> Option<Wall> {
    match value {
        Some(value) => Some(Wall::from_nanos(value)),
        None => None,
    }
}
pub(super) fn name(value: &domain::Name, limits: &wire::Limits) -> Result<wire::Name, wire::Problem> {
    let domain::Name { connector, path } = value;
    let members = super::pieces::segments(path, limits, limits.name_path, wire::paths::NAME_PATH)?;
    let mut bytes = 0_usize;
    for segment in path {
        bytes = bytes.saturating_add(segment.len());
    }
    if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
        return Err(problem(wire::paths::NAME_PATH));
    }
    Ok(wire::Name { connector: *connector, path: members })
}
pub(super) fn read_name(value: wire::Name, limits: &wire::Limits) -> Result<domain::Name, wire::Problem> {
    let wire::Name { connector, path } = value;
    let path = super::pieces::read_segments(path, limits, wire::paths::NAME_PATH)?;
    let mut bytes = 0_usize;
    for segment in &path {
        bytes = bytes.saturating_add(segment.len());
    }
    if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
        return Err(problem(wire::paths::NAME_PATH));
    }
    Ok(domain::Name { connector, path })
}

/// Translate a tasks party.
pub fn party(value: &domain::Party, _limits: &wire::Limits) -> Result<wire::Party, wire::Problem> {
    mapped::party_to_record(value, &Context::new(&super::EMPTY, None))
}
/// Read a tasks party.
#[must_use]
pub fn read_party(value: wire::Party) -> domain::Party {
    mapped::party_to_domain(value, &Context::new(&super::EMPTY, None)).expect("translation has no fallible fields")
}

/// Translate a permitted executor.
pub fn authority_executor(
    value: &domain::AuthorityExecutor,
    _limits: &wire::Limits,
) -> Result<wire::AuthorityExecutor, wire::Problem> {
    mapped::authority_executor_to_record(value, &Context::new(&super::EMPTY, None))
}
/// Read a permitted executor.
#[must_use]
pub fn read_authority_executor(value: wire::AuthorityExecutor) -> domain::AuthorityExecutor {
    mapped::authority_executor_to_domain(value, &Context::new(&super::EMPTY, None))
        .expect("translation has no fallible fields")
}

/// Translate a tasks class.
#[must_use]
pub fn class(value: domain::Class) -> wire::Class {
    mapped::class_to_record(&value, &Context::new(&super::EMPTY, None)).expect("translation has no fallible fields")
}

/// Translate a tasks class.
#[must_use]
pub fn read_class(value: wire::Class) -> domain::Class {
    mapped::class_to_domain(value, &Context::new(&super::EMPTY, None)).expect("translation has no fallible fields")
}

/// Translate the cause of a task activation.
pub fn why(value: domain::Why, limits: &wire::Limits) -> Result<wire::Why, wire::Problem> {
    Ok(match value {
        domain::Why::Made => wire::Why::Made,
        domain::Why::Ready => wire::Why::Ready,
        domain::Why::Message { message } => wire::Why::Message(wire::Number { value: message }),
        domain::Why::Released => wire::Why::Released,
        domain::Why::Retry { tries: value } => {
            wire::Why::Retry(mapped::tries_to_record(&value, &Context::new(limits, None))?)
        }
        domain::Why::Narrowed => wire::Why::Narrowed,
        domain::Why::Changed => wire::Why::Changed,
    })
}
/// Read the cause of a task activation.
#[must_use]
pub fn read_why(value: wire::Why) -> domain::Why {
    match value {
        wire::Why::Made => domain::Why::Made,
        wire::Why::Ready => domain::Why::Ready,
        wire::Why::Message(value) => domain::Why::Message { message: value.value },
        wire::Why::Released => domain::Why::Released,
        wire::Why::Retry(value) => domain::Why::Retry {
            tries: mapped::tries_to_domain(value, &Context::new(&super::EMPTY, None))
                .expect("tries only copies integer fields"),
        },
        wire::Why::Narrowed => domain::Why::Narrowed,
        wire::Why::Changed => domain::Why::Changed,
    }
}
/// Translate the selected executor.
pub fn executor(value: domain::Executor, limits: &wire::Limits) -> Result<wire::Executor, wire::Problem> {
    mapped::executor_to_record(&value, &Context::new(limits, None))
}
/// Read the selected executor.
#[must_use]
pub fn read_executor(value: wire::Executor) -> domain::Executor {
    mapped::executor_to_domain(value, &Context::new(&super::EMPTY, None)).expect("translation has no fallible fields")
}

/// Translate a specification, checking its aggregate byte bound.
pub fn spec(value: &domain::Spec, limits: &wire::Limits) -> Result<wire::Spec, wire::Problem> {
    let domain::Spec { asks, words, parameters, inputs, job } = value;
    let job = match job {
        Some(job) => Some(mapped::job_spec_to_record(job, &Context::new(limits, None))?),
        None => None,
    };
    let members = super::pieces::parameters(parameters, limits, limits.spec_parameters, wire::paths::SPEC_PARAMETERS)?;
    let mut bytes = words.len();
    for item in parameters {
        match item {
            domain::Parameter::Bytes { value, .. } => bytes = bytes.saturating_add(value.len()),
            domain::Parameter::Number { .. } | domain::Parameter::Resource { .. } => {}
        }
    }
    if bytes > skein_lib::length::usize_of(limits.spec_words) {
        return Err(problem(wire::paths::SPEC_WORDS));
    }
    let mut references = List::with_capacity(limits.spec_inputs);
    for input in inputs {
        let input = wire::Number { value: *input };
        if references.push(input).is_err() {
            return Err(problem(wire::paths::SPEC_INPUTS));
        }
    }
    Ok(wire::Spec { job, asks: *asks, words: words.clone(), parameters: members, inputs: references })
}
/// Read a specification, checking its aggregate byte bound.
pub fn read_spec(value: wire::Spec, limits: &wire::Limits) -> Result<domain::Spec, wire::Problem> {
    let wire::Spec { words, parameters, inputs, asks, job } = value;
    let job = match job {
        Some(job) => {
            let wire::JobSpec { bytes, deadline } = job;
            if bytes.len() > skein_lib::length::usize_of(limits.job_spec_bytes) {
                return Err(problem(wire::paths::JOB_SPEC_BYTES));
            }
            Some(Box::new(mapped::job_spec_to_domain(wire::JobSpec { bytes, deadline }, &Context::new(limits, None))?))
        }
        None => None,
    };
    let members = super::pieces::read_parameters(parameters, limits, wire::paths::SPEC_PARAMETERS)?;
    let mut bytes = words.len();
    for item in &members {
        match item {
            domain::Parameter::Bytes { value, .. } => bytes = bytes.saturating_add(value.len()),
            domain::Parameter::Number { .. } | domain::Parameter::Resource { .. } => {}
        }
    }
    if bytes > skein_lib::length::usize_of(limits.spec_words) {
        return Err(problem(wire::paths::SPEC_WORDS));
    }
    let mut references = List::with_capacity(inputs.len());
    for input in inputs.into_boxed() {
        references.push(input.value).expect("same number of inputs");
    }
    Ok(domain::Spec { job, asks, words, parameters: members, inputs: references.into_boxed() })
}
/// Translate a connector part and its opaque key.
pub fn part(value: &domain::Part, limits: &wire::Limits) -> Result<wire::Part, wire::Problem> {
    mapped::part_to_record(value, &Context::new(limits, None))
}
/// Read a connector part and its opaque key.
pub fn read_part(value: wire::Part, limits: &wire::Limits) -> Result<domain::Part, wire::Problem> {
    mapped::part_to_domain(value, &Context::new(limits, None))
}
fn problem(path: wire::Path) -> wire::Problem {
    wire::Problem { path, reason: skein_codec::Reason::Bound }
}

/// Translate a result contract.
pub fn contract(value: &domain::Contract, limits: &wire::Limits) -> Result<wire::Contract, wire::Problem> {
    check_contract(value, limits)?;
    Ok(match value {
        domain::Contract::Job { bytes } => wire::Contract::Job(wire::JobContract { bytes: *bytes }),
        domain::Contract::Approval { state, part: value } => wire::Contract::Approval(wire::Approval {
            state: state.clone(),
            part: mapped::part_to_record(value, &Context::new(limits, None))?,
        }),
        domain::Contract::Report { words } => wire::Contract::Report(wire::Report { words: *words }),
        domain::Contract::Change { connector, kind, words, choices } => wire::Contract::Change(wire::ChangeContract {
            connector: *connector,
            kind: *kind,
            words: *words,
            choices: contract_choices(
                choices,
                limits.change_contract_choices,
                wire::paths::CHANGE_CONTRACT_CHOICES,
                limits,
            )?,
        }),
        domain::Contract::Verdict { choices } => wire::Contract::Verdict(wire::Choices {
            choices: contract_choices(choices, limits.choices_choices, wire::paths::CHOICES_CHOICES, limits)?,
        }),
    })
}
/// Read a result contract.
pub fn read_contract(value: wire::Contract, limits: &wire::Limits) -> Result<domain::Contract, wire::Problem> {
    let value = match value {
        wire::Contract::Job(value) => domain::Contract::Job { bytes: value.bytes },
        wire::Contract::Approval(value) => {
            let wire::Approval { state, part: value } = value;
            domain::Contract::Approval { state, part: mapped::part_to_domain(value, &Context::new(limits, None))? }
        }
        wire::Contract::Report(value) => domain::Contract::Report { words: value.words },
        wire::Contract::Change(value) => {
            let wire::ChangeContract { connector, kind, words, choices } = value;
            domain::Contract::Change { connector, kind, words, choices: read_contract_choices(choices, limits)? }
        }
        wire::Contract::Verdict(value) => {
            domain::Contract::Verdict { choices: read_contract_choices(value.choices, limits)? }
        }
    };
    check_contract(&value, limits)?;
    Ok(value)
}
fn contract_choices(
    choices: &[domain::Verdict],
    count: u32,
    path: wire::Path,
    limits: &wire::Limits,
) -> Result<List<wire::Verdict>, wire::Problem> {
    let mut members = List::with_capacity(count);
    for choice in choices {
        let domain::Verdict { code, words, followups, fields } = choice;
        let rules = super::pieces::rules(fields, limits)?;
        let choice = wire::Verdict { code: *code, words: *words, followups: *followups, fields: rules };
        if members.push(choice).is_err() {
            return Err(problem(path));
        }
    }
    Ok(members)
}

fn read_contract_choices(
    choices: List<wire::Verdict>,
    limits: &wire::Limits,
) -> Result<Box<[domain::Verdict]>, wire::Problem> {
    let mut members = List::with_capacity(choices.len());
    for choice in choices.into_boxed() {
        let wire::Verdict { code, words, followups, fields } = choice;
        let rules = super::pieces::read_rules(fields, limits)?;
        members.push(domain::Verdict { code, words, followups, fields: rules }).expect("same number of choices");
    }
    Ok(members.into_boxed())
}

/// Translate authority, checking bytes shared by grants and note resources.
pub fn authority(
    value: &domain::Authority,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Authority, wire::Problem> {
    let context = &Context::new(limits, None);
    check_authority(value, limits)?;
    let domain::Authority { tools, models, grants, delegation, budget, notes, note_resources } = value;
    let models = super::models::selector(*models, moved);
    let translated_grants =
        super::pieces::grants(grants, limits, limits.authority_grants, wire::paths::AUTHORITY_GRANTS)?;
    let resources = super::pieces::scopes(
        note_resources,
        limits,
        limits.authority_note_resources,
        wire::paths::AUTHORITY_NOTE_RESOURCES,
    )?;
    Ok(wire::Authority {
        tools: wire::Tools {
            bits: match super::tools_to_record(tools) {
                Ok(bits) => bits,
                Err(reason) => return Err(wire::paths::TOOLS_BITS.problem(reason)),
            },
        },
        models: super::models_to_record(&models, context)?,
        grants: translated_grants,
        delegation: mapped::delegation_to_record(delegation, context)?,
        budget: mapped::budget_to_record(budget, context)?,
        notes: wire::Scopes {
            bits: match super::scopes_to_record(notes) {
                Ok(bits) => bits,
                Err(reason) => return Err(wire::paths::SCOPES_BITS.problem(reason)),
            },
        },
        note_resources: resources,
    })
}
/// Read authority, checking bytes shared by grants and note resources.
pub fn read_authority(value: wire::Authority, limits: &wire::Limits) -> Result<domain::Authority, wire::Problem> {
    let context = &Context::new(limits, None);
    let wire::Authority { tools, models, grants, delegation, budget, notes, note_resources } = value;
    let translated_grants = super::pieces::read_grants(grants, limits, wire::paths::AUTHORITY_GRANTS)?;
    let resources = super::pieces::read_scopes(note_resources, limits, wire::paths::AUTHORITY_NOTE_RESOURCES)?;
    let value = domain::Authority {
        tools: match super::tools_to_domain(tools.bits) {
            Ok(value) => value,
            Err(reason) => return Err(wire::paths::TOOLS_BITS.problem(reason)),
        },
        models: super::models_to_domain(models, context)?,
        grants: translated_grants,
        delegation: mapped::delegation_to_domain(delegation, context)?,
        budget: mapped::budget_to_domain(budget, context)?,
        notes: match super::scopes_to_domain(notes.bits) {
            Ok(value) => value,
            Err(reason) => return Err(wire::paths::SCOPES_BITS.problem(reason)),
        },
        note_resources: resources,
    };
    check_authority(&value, limits)?;
    Ok(value)
}
/// Translate an agent task's LLM choice, if any (domain/tasks/README.md, section 3).
pub(super) fn llm(
    source: Option<&domain::LlmChoice>,
    limits: &wire::Limits,
) -> Result<Option<wire::LlmChoice>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(mapped::llm_choice_to_record(value, &Context::new(limits, None))?)),
        None => Ok(None),
    }
}

/// Read an agent task's LLM choice, if any; its decode bounds its sub-agents.
pub(super) fn read_llm(source: Option<wire::LlmChoice>) -> Result<Option<domain::LlmChoice>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(mapped::llm_choice_to_domain(value, &Context::new(&super::EMPTY, None))?)),
        None => Ok(None),
    }
}

fn check_authority(value: &domain::Authority, limits: &wire::Limits) -> Result<(), wire::Problem> {
    let mut bytes = 0_usize;
    for grant in &value.grants {
        bytes = bytes.saturating_add(pattern_bytes(&grant.pattern));
    }
    for scope in &value.note_resources {
        bytes = bytes.saturating_add(pattern_bytes(&scope.pattern));
    }
    if bytes > skein_lib::length::usize_of(limits.authority_segment_bytes) {
        return Err(problem(wire::paths::AUTHORITY_GRANTS));
    }
    Ok(())
}
fn pattern_bytes(value: &domain::Pattern) -> usize {
    let mut bytes = match &value.last {
        domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
    };
    for segment in &value.segments {
        bytes = bytes.saturating_add(segment.len());
    }
    bytes
}

fn check_contract(value: &domain::Contract, limits: &wire::Limits) -> Result<(), wire::Problem> {
    let bound = skein_lib::length::usize_of(limits.field_rule_name);
    match value {
        domain::Contract::Job { bytes } => {
            if skein_lib::length::usize_of(*bytes) > bound {
                return Err(problem(wire::paths::JOB_CONTRACT_BYTES));
            }
        }
        domain::Contract::Report { words } => {
            if skein_lib::length::usize_of(*words) > bound {
                return Err(problem(wire::paths::REPORT_WORDS));
            }
        }
        domain::Contract::Change { words, choices, .. } => {
            if skein_lib::length::usize_of(*words) > bound {
                return Err(problem(wire::paths::CHANGE_CONTRACT_WORDS));
            }
            check_choices(choices, bound)?;
        }
        domain::Contract::Verdict { choices } => check_choices(choices, bound)?,
        domain::Contract::Approval { .. } => {}
    }
    Ok(())
}

fn check_choices(choices: &[domain::Verdict], bound: usize) -> Result<(), wire::Problem> {
    for choice in choices {
        let mut bytes = skein_lib::length::usize_of(choice.words);
        for field in &choice.fields {
            bytes = bytes.saturating_add(field.name.len()).saturating_add(skein_lib::length::usize_of(field.max));
        }
        if bytes > bound {
            return Err(problem(wire::paths::VERDICT_FIELDS));
        }
    }
    Ok(())
}
