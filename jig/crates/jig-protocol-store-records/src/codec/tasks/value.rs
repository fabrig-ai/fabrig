//! Funding and resource values shared by tasks rows (protocol/store/records.md, 1).
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{List, Wall};

pub(super) fn funder(value: domain::Funder, _limits: &wire::Limits) -> wire::Funder {
    match value {
        domain::Funder::Task(task) => wire::Funder::Task(wire::Task { task }),
        domain::Funder::Pool { project, person, period } => wire::Funder::Pool(wire::Pool { project, person, period }),
        domain::Funder::Recurring { project, task, period } => {
            wire::Funder::Recurring(wire::Recurring { project, task, period })
        }
        domain::Funder::Period { project, period } => wire::Funder::Period(wire::Period { project, period }),
    }
}
pub(super) fn read_funder(value: wire::Funder) -> domain::Funder {
    match value {
        wire::Funder::Task(value) => domain::Funder::Task(value.task),
        wire::Funder::Pool(value) => {
            let wire::Pool { project, person, period } = value;
            domain::Funder::Pool { project, person, period }
        }
        wire::Funder::Recurring(value) => {
            let wire::Recurring { project, task, period } = value;
            domain::Funder::Recurring { project, task, period }
        }
        wire::Funder::Period(value) => {
            let wire::Period { project, period } = value;
            domain::Funder::Period { project, period }
        }
    }
}
pub(super) fn funding(value: &domain::FundingRecord, limits: &wire::Limits) -> wire::FundingRecord {
    let domain::FundingRecord { period, funder: source, parent, numbers, made, closed } = value;
    let period = match period {
        Some(domain::PeriodTimes { began, next }) => {
            Some(wire::PeriodTimes { began: began.as_nanos(), next: wall(*next) })
        }
        None => None,
    };
    let parent = match parent {
        Some(value) => Some(funder(*value, limits)),
        None => None,
    };
    let domain::Numbers { budget, spent, spent_below, reserved } = numbers;
    wire::FundingRecord {
        period,
        funder: funder(*source, limits),
        parent,
        numbers: wire::Numbers { budget: *budget, spent: *spent, spent_below: *spent_below, reserved: *reserved },
        made: *made,
        closed: *closed,
    }
}
pub(super) fn read_funding(value: wire::FundingRecord) -> domain::FundingRecord {
    let wire::FundingRecord { period, funder, parent, numbers, made, closed } = value;
    let period = match period {
        Some(value) => {
            let wire::PeriodTimes { began, next } = value;
            Some(domain::PeriodTimes { began: Wall::from_nanos(began), next: read_wall(next) })
        }
        None => None,
    };
    let parent = match parent {
        Some(value) => Some(read_funder(value)),
        None => None,
    };
    let wire::Numbers { budget, spent, spent_below, reserved } = numbers;
    domain::FundingRecord {
        period,
        funder: read_funder(funder),
        parent,
        numbers: domain::Numbers { budget, spent, spent_below, reserved },
        made,
        closed,
    }
}
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
    Ok(match value {
        domain::Party::Task(task) => wire::Party::Task(wire::Number { value: *task }),
        domain::Party::Person(person) => wire::Party::Person(wire::Person { person: *person }),
        domain::Party::Deployment { project } => wire::Party::Deployment(wire::Project { project: *project }),
    })
}
/// Read a tasks party.
#[must_use]
pub fn read_party(value: wire::Party) -> domain::Party {
    match value {
        wire::Party::Task(value) => domain::Party::Task(value.value),
        wire::Party::Person(value) => domain::Party::Person(value.person),
        wire::Party::Deployment(value) => domain::Party::Deployment { project: value.project },
    }
}

/// Translate a person destination.
pub fn person_address(
    value: &domain::PersonAddress,
    _limits: &wire::Limits,
) -> Result<wire::PersonAddress, wire::Problem> {
    Ok(match value {
        domain::PersonAddress::Person(person) => wire::PersonAddress::Person(wire::Person { person: *person }),
        domain::PersonAddress::Role(role) => wire::PersonAddress::Role(wire::Role { role: *role }),
    })
}
/// Read a person destination.
#[must_use]
pub fn read_person_address(value: wire::PersonAddress) -> domain::PersonAddress {
    match value {
        wire::PersonAddress::Person(value) => domain::PersonAddress::Person(value.person),
        wire::PersonAddress::Role(value) => domain::PersonAddress::Role(value.role),
    }
}

/// Translate a permitted executor.
pub fn authority_executor(
    value: &domain::AuthorityExecutor,
    _limits: &wire::Limits,
) -> Result<wire::AuthorityExecutor, wire::Problem> {
    Ok(match value {
        domain::AuthorityExecutor::Job(kind) => wire::AuthorityExecutor::Job(wire::AuthorityJob { kind: *kind }),
        domain::AuthorityExecutor::Charter(charter) => {
            wire::AuthorityExecutor::Charter(wire::Charter { charter: *charter })
        }
        domain::AuthorityExecutor::Procedure(code) => wire::AuthorityExecutor::Procedure(wire::Code { code: *code }),
        domain::AuthorityExecutor::Role(role) => wire::AuthorityExecutor::Role(wire::Role { role: *role }),
    })
}
/// Read a permitted executor.
#[must_use]
pub fn read_authority_executor(value: wire::AuthorityExecutor) -> domain::AuthorityExecutor {
    match value {
        wire::AuthorityExecutor::Job(value) => domain::AuthorityExecutor::Job(value.kind),
        wire::AuthorityExecutor::Charter(value) => domain::AuthorityExecutor::Charter(value.charter),
        wire::AuthorityExecutor::Procedure(value) => domain::AuthorityExecutor::Procedure(value.code),
        wire::AuthorityExecutor::Role(value) => domain::AuthorityExecutor::Role(value.role),
    }
}

/// Translate a tasks class.
#[must_use]
pub fn class(value: domain::Class) -> wire::Class {
    match value {
        domain::Class::Transient => wire::Class::Transient,
        domain::Class::Permanent => wire::Class::Permanent,
        domain::Class::Run => wire::Class::Run,
        domain::Class::Agent => wire::Class::Agent,
        domain::Class::Lost => wire::Class::Lost,
        domain::Class::Invalid => wire::Class::Invalid,
        domain::Class::Infrastructure => wire::Class::Infrastructure,
    }
}

/// Translate a tasks class.
#[must_use]
pub fn read_class(value: wire::Class) -> domain::Class {
    match value {
        wire::Class::Transient => domain::Class::Transient,
        wire::Class::Permanent => domain::Class::Permanent,
        wire::Class::Run => domain::Class::Run,
        wire::Class::Agent => domain::Class::Agent,
        wire::Class::Lost => domain::Class::Lost,
        wire::Class::Invalid => domain::Class::Invalid,
        wire::Class::Infrastructure => domain::Class::Infrastructure,
    }
}

/// Translate a connector part kind.
#[must_use]
pub fn part_kind(value: domain::PartKind) -> wire::PartKind {
    match value {
        domain::PartKind::Resource => wire::PartKind::Resource,
        domain::PartKind::Pool => wire::PartKind::Pool,
        domain::PartKind::Result => wire::PartKind::Result,
        domain::PartKind::Procedure => wire::PartKind::Procedure,
        domain::PartKind::Effect => wire::PartKind::Effect,
        domain::PartKind::Outbox => wire::PartKind::Outbox,
        domain::PartKind::Hold => wire::PartKind::Hold,
    }
}

/// Translate a connector part kind.
#[must_use]
pub fn read_part_kind(value: wire::PartKind) -> domain::PartKind {
    match value {
        wire::PartKind::Resource => domain::PartKind::Resource,
        wire::PartKind::Pool => domain::PartKind::Pool,
        wire::PartKind::Result => domain::PartKind::Result,
        wire::PartKind::Procedure => domain::PartKind::Procedure,
        wire::PartKind::Effect => domain::PartKind::Effect,
        wire::PartKind::Outbox => domain::PartKind::Outbox,
        wire::PartKind::Hold => domain::PartKind::Hold,
    }
}

/// Translate a tasks tries.
pub fn tries(value: &domain::Tries, _limits: &wire::Limits) -> Result<wire::Tries, wire::Problem> {
    let domain::Tries { transient, permanent, run, agent, lost, invalid, infrastructure } = value;
    Ok(wire::Tries {
        transient: *transient,
        permanent: *permanent,
        run: *run,
        agent: *agent,
        lost: *lost,
        invalid: *invalid,
        infrastructure: *infrastructure,
    })
}
/// Read a tasks tries.
#[must_use]
pub fn read_tries(value: wire::Tries) -> domain::Tries {
    let wire::Tries { transient, permanent, run, agent, lost, invalid, infrastructure } = value;
    domain::Tries { transient, permanent, run, agent, lost, invalid, infrastructure }
}

/// Translate the cause of a task activation.
pub fn why(value: domain::Why, limits: &wire::Limits) -> Result<wire::Why, wire::Problem> {
    Ok(match value {
        domain::Why::Made => wire::Why::Made,
        domain::Why::Ready => wire::Why::Ready,
        domain::Why::Message { message } => wire::Why::Message(wire::Number { value: message }),
        domain::Why::Released => wire::Why::Released,
        domain::Why::Retry { tries: value } => wire::Why::Retry(tries(&value, limits)?),
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
        wire::Why::Retry(value) => domain::Why::Retry { tries: read_tries(value) },
        wire::Why::Narrowed => domain::Why::Narrowed,
        wire::Why::Changed => domain::Why::Changed,
    }
}
/// Translate the selected executor.
pub fn executor(value: domain::Executor, limits: &wire::Limits) -> Result<wire::Executor, wire::Problem> {
    Ok(match value {
        domain::Executor::Job { kind } => wire::Executor::Job(wire::JobExecutor { kind }),
        domain::Executor::Agent { charter } => wire::Executor::Agent(wire::Agent { charter }),
        domain::Executor::Procedure { connector, code } => {
            wire::Executor::Procedure(wire::Procedure { connector, code })
        }
        domain::Executor::Person(address) => {
            wire::Executor::Person(wire::PersonExecutor { address: person_address(&address, limits)? })
        }
    })
}
/// Read the selected executor.
#[must_use]
pub fn read_executor(value: wire::Executor) -> domain::Executor {
    match value {
        wire::Executor::Job(value) => domain::Executor::Job { kind: value.kind },
        wire::Executor::Agent(value) => domain::Executor::Agent { charter: value.charter },
        wire::Executor::Procedure(value) => {
            let wire::Procedure { connector, code } = value;
            domain::Executor::Procedure { connector, code }
        }
        wire::Executor::Person(value) => domain::Executor::Person(read_person_address(value.address)),
    }
}
/// Translate one specification parameter.
pub fn parameter(value: &domain::Parameter, _limits: &wire::Limits) -> Result<wire::Parameter, wire::Problem> {
    Ok(match value {
        domain::Parameter::Number { name, value } => {
            wire::Parameter::Number(wire::NumericParameter { name: *name, value: *value })
        }
        domain::Parameter::Bytes { name, value } => {
            wire::Parameter::Bytes(wire::BytesParameter { name: *name, value: value.clone() })
        }
        domain::Parameter::Resource { name, connector, resource } => {
            wire::Parameter::Resource(wire::ResourceParameter {
                name: *name,
                connector: *connector,
                resource: *resource,
            })
        }
    })
}
/// Read one specification parameter.
#[must_use]
pub fn read_parameter(value: wire::Parameter) -> domain::Parameter {
    match value {
        wire::Parameter::Number(value) => {
            let wire::NumericParameter { name, value } = value;
            domain::Parameter::Number { name, value }
        }
        wire::Parameter::Bytes(value) => {
            let wire::BytesParameter { name, value } = value;
            domain::Parameter::Bytes { name, value }
        }
        wire::Parameter::Resource(value) => {
            let wire::ResourceParameter { name, connector, resource } = value;
            domain::Parameter::Resource { name, connector, resource }
        }
    }
}
/// Translate a specification, checking its aggregate byte bound.
pub fn spec(value: &domain::Spec, limits: &wire::Limits) -> Result<wire::Spec, wire::Problem> {
    let domain::Spec { asks, words, parameters, inputs, job } = value;
    let job = match job {
        Some(job) => Some(wire::JobSpec { bytes: job.bytes.clone(), deadline: job.deadline }),
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
            Some(Box::new(domain::JobSpec { bytes, deadline }))
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
    let domain::Part { connector, kind, key } = value;
    let key = match key {
        domain::PartKey::Number(value) => wire::PartKey::Number(wire::Number { value: *value }),
        domain::PartKey::Path(path) => {
            let members = super::pieces::part_segments(
                path.as_slice(),
                limits,
                limits.part_path_segments,
                wire::paths::PART_PATH_SEGMENTS,
            )?;
            let mut bytes = 0_usize;
            for segment in path {
                bytes = bytes.saturating_add(segment.len());
            }
            if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
                return Err(problem(wire::paths::PART_PATH_SEGMENTS));
            }
            wire::PartKey::Path(wire::PartPath { segments: members })
        }
    };
    Ok(wire::Part { connector: *connector, kind: part_kind(*kind), key })
}
/// Read a connector part and its opaque key.
pub fn read_part(value: wire::Part, limits: &wire::Limits) -> Result<domain::Part, wire::Problem> {
    let wire::Part { connector, kind, key } = value;
    let key = match key {
        wire::PartKey::Number(value) => domain::PartKey::Number(value.value),
        wire::PartKey::Path(value) => {
            let segments = value.segments;
            let segments = super::pieces::read_part_segments(segments, limits, wire::paths::PART_PATH_SEGMENTS)?;
            let mut members = List::with_capacity(u32::try_from(segments.len()).expect("bounded path"));
            let mut bytes = 0_usize;
            for segment in segments {
                bytes = bytes.saturating_add(segment.len());
                members.push(segment).expect("same number of segments");
            }
            if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
                return Err(problem(wire::paths::PART_PATH_SEGMENTS));
            }
            domain::PartKey::Path(members)
        }
    };
    Ok(domain::Part { connector, kind: read_part_kind(kind), key })
}
fn problem(path: wire::Path) -> wire::Problem {
    wire::Problem { path, reason: skein_codec::Reason::Bound }
}

/// Translate a result contract.
pub fn contract(value: &domain::Contract, limits: &wire::Limits) -> Result<wire::Contract, wire::Problem> {
    check_contract(value, limits)?;
    Ok(match value {
        domain::Contract::Job { bytes } => wire::Contract::Job(wire::JobContract { bytes: *bytes }),
        domain::Contract::Approval { state, part: value } => {
            wire::Contract::Approval(wire::Approval { state: state.clone(), part: part(value, limits)? })
        }
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
            domain::Contract::Approval { state, part: read_part(value, limits)? }
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

/// Translate a resource authority pattern.
pub fn pattern(value: &domain::Pattern, limits: &wire::Limits) -> Result<wire::Pattern, wire::Problem> {
    let domain::Pattern { segments, last } = value;
    let mut members = List::with_capacity(limits.pattern_segments);
    for segment in segments {
        let segment = wire::AuthoritySegment { bytes: segment.clone() };
        if members.push(segment).is_err() {
            return Err(problem(wire::paths::PATTERN_SEGMENTS));
        }
    }
    let last = match last {
        domain::Last::Exact(bytes) => wire::Last::Exact(wire::AuthoritySegment { bytes: bytes.clone() }),
        domain::Last::Open(bytes) => wire::Last::Open(wire::AuthoritySegment { bytes: bytes.clone() }),
    };
    Ok(wire::Pattern { segments: members, last })
}
/// Read a resource authority pattern.
#[must_use]
pub fn read_pattern(value: wire::Pattern) -> domain::Pattern {
    let wire::Pattern { segments, last } = value;
    let mut members = List::with_capacity(segments.len());
    for segment in segments.into_boxed() {
        members.push(segment.bytes).expect("same number of segments");
    }
    let last = match last {
        wire::Last::Exact(value) => domain::Last::Exact(value.bytes),
        wire::Last::Open(value) => domain::Last::Open(value.bytes),
    };
    domain::Pattern { segments: members.into_boxed(), last }
}
/// Translate authority, checking bytes shared by grants and note resources.
pub fn authority(
    value: &domain::Authority,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Authority, wire::Problem> {
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
    let domain::Delegation { kinds, tasks, depth } = delegation;
    let mut executors = List::with_capacity(limits.delegation_kinds);
    for kind in kinds {
        let kind = authority_executor(kind, limits)?;
        if executors.push(kind).is_err() {
            return Err(problem(wire::paths::DELEGATION_KINDS));
        }
    }
    let domain::Budget { spend, deadline } = budget;
    Ok(wire::Authority {
        tools: wire::Tools { bits: tools.0 },
        models: wire::Models { bits: models.bits(), all: models.is_all() },
        grants: translated_grants,
        delegation: wire::Delegation { kinds: executors, tasks: *tasks, depth: *depth },
        budget: wire::Budget { spend: *spend, deadline: wall(*deadline) },
        notes: wire::Scopes { bits: notes.0 },
        note_resources: resources,
    })
}
/// Read authority, checking bytes shared by grants and note resources.
pub fn read_authority(value: wire::Authority, limits: &wire::Limits) -> Result<domain::Authority, wire::Problem> {
    let wire::Authority { tools, models, grants, delegation, budget, notes, note_resources } = value;
    let translated_grants = super::pieces::read_grants(grants, limits, wire::paths::AUTHORITY_GRANTS)?;
    let resources = super::pieces::read_scopes(note_resources, limits, wire::paths::AUTHORITY_NOTE_RESOURCES)?;
    let wire::Delegation { kinds, tasks, depth } = delegation;
    let mut executors = List::with_capacity(kinds.len());
    for kind in kinds.into_boxed() {
        executors.push(read_authority_executor(kind)).expect("same number of executors");
    }
    let wire::Budget { spend, deadline } = budget;
    let value = domain::Authority {
        tools: domain::Tools(tools.bits),
        models: domain::Models::from_parts(models.bits, models.all),
        grants: translated_grants,
        delegation: domain::Delegation { kinds: executors.into_boxed(), tasks, depth },
        budget: domain::Budget { spend, deadline: read_wall(deadline) },
        notes: domain::Scopes(notes.bits),
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
    let Some(domain::LlmChoice { main, subagents, account, later }) = source else {
        return Ok(None);
    };
    let mut chosen = List::with_capacity(limits.llm_choice_subagents);
    for subagent in subagents {
        if chosen.push(model_effort(*subagent, limits)).is_err() {
            return Err(problem(wire::paths::LLM_CHOICE_SUBAGENTS));
        }
    }
    let later = match later {
        None => None,
        Some(later) => {
            let mut chosen = List::with_capacity(limits.llm_later_subagents);
            for subagent in &later.subagents {
                if chosen.push(model_effort(*subagent, limits)).is_err() {
                    return Err(wire::Problem {
                        path: wire::paths::LLM_LATER_SUBAGENTS,
                        reason: skein_codec::Reason::Bound,
                    });
                }
            }
            Some(wire::LlmLater { main: model_effort(later.main, limits), subagents: chosen })
        }
    };
    Ok(Some(wire::LlmChoice { main: model_effort(*main, limits), subagents: chosen, account: *account, later }))
}
fn model_effort(source: domain::ModelEffort, _limits: &wire::Limits) -> wire::ModelEffort {
    let domain::ModelEffort { model, effort } = source;
    wire::ModelEffort { model, effort }
}
/// Read an agent task's LLM choice, if any; its decode bounds its sub-agents.
pub(super) fn read_llm(source: Option<wire::LlmChoice>) -> Option<domain::LlmChoice> {
    let wire::LlmChoice { main, subagents, account, later } = source?;
    let mut chosen = List::with_capacity(subagents.len());
    for subagent in subagents.into_boxed() {
        chosen.push(read_model_effort(subagent)).expect("same number of sub-agents");
    }
    let later = match later {
        None => None,
        Some(later) => {
            let wire::LlmLater { main, subagents } = later;
            let mut chosen = List::with_capacity(subagents.len());
            for subagent in subagents.into_boxed() {
                chosen.push(read_model_effort(subagent)).expect("decoded later sub-agents bound");
            }
            Some(domain::LlmLater { main: read_model_effort(main), subagents: chosen.into_boxed() })
        }
    };
    Some(domain::LlmChoice { later, main: read_model_effort(main), subagents: chosen.into_boxed(), account })
}
fn read_model_effort(source: wire::ModelEffort) -> domain::ModelEffort {
    let wire::ModelEffort { model, effort } = source;
    domain::ModelEffort { model, effort }
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
