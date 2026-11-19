//! Typed settings records (protocol/store/README.md, section 4.4 and protocol/store/records.md).
#![expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "typed encoders uniformly borrow the domain values they translate"
)]
use crate::settings_v3::{self as wire, Record};
mod providers;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core_vocabulary::{self as core, settings};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use settings::application;
use skein_lib::{List, Reader, Wall};
const EMPTY: wire::Limits = wire::Limits {
    provider_text_bytes: 0,
    provider_list_keep: 0,
    provider_list_drop: 0,
    provider_tls_server_name: 0,
    provider_tls_trust_some: 0,
    provider_registration_issuer: 0,
    provider_registration_client: 0,
    provider_registration_scopes: 0,
    provider_registration_name_hint_some: 0,
    provider_registration_authorization: 0,
    provider_registration_token: 0,
    provider_registration_revocation_some: 0,
    provider_registration_keys: 0,
    provider_registration_must_grant: 0,
    provider_registration_resource_some: 0,
    provider_registration_redirect_path: 0,
    provider_name: 0,
    provider_endpoint_some: 0,
    provider_direct_some: 0,
    provider_manage_usage_some: 0,
    provider_programs: 0,
    provider_host_name: 0,
    provider_header_name: 0,
    provider_header_value: 0,
    provider_endpoint_name: 0,
    provider_endpoint_provider: 0,
    provider_endpoint_program_some: 0,
    provider_endpoint_headers: 0,
    provider_setting_endpoints: 0,

    worker_setting_name: 0,
    text_bytes: 0,
    override_name: 0,
    override_reason: 0,
    limits_value_overrides: 0,
    model_name_name: 0,
    model_name_efforts: 0,
    model_name_efforts_item: 0,
    catalogue_model_shown_some: 0,
    catalogue_model_description_some: 0,
    model_program_some: 0,
    model_name: 0,
    model_efforts: 0,
    model_effort_some: 0,
    overlay_instructions: 0,
    overlay_parts: 0,
    overlay_documents_some: 0,
    overlay_alternatives_some: 0,
    run_policy_instructions: 0,
    run_policy_alternatives: 0,
    segment_bytes: 0,
    document_name: 0,
    document_path: 0,
    document_heading_some: 0,
    charter_documents: 0,
    charter_parts: 0,
    charter_part_instructions: 0,
    last_exact_bytes: 0,
    last_open_bytes: 0,
    pattern_segments: 0,
    delegation_kinds: 0,
    authority_grants: 0,
    authority_note_resources: 0,
    rules_implies: 0,
    rules_requirements: 0,
    policy_projections: 0,
    policy_roles: 0,
    policy_requirements: 0,
    configured_policy_permissions: 0,
    configured_policy_templates: 0,
    field_rule_name: 0,
    verdict_fields: 0,
    part_path_segments: 0,
    approval_state: 0,
    verdict_choices_choices: 0,
    llm_choice_subagents: 0,
    llm_later_subagents: 0,
    template_name: 0,
    template_resources: 0,
    application_state_charters: 0,
    default_models: 0,
    changed_models: 0,
    revision_models: 0,
    revision_items: 0,
    revision_applications: 0,
    proposal_models: 0,
    proposal_items: 0,
    proposal_applications: 0,
};
fn bound() -> wire::Problem {
    wire::Problem { path: wire::paths::DEFAULT_VALUE, reason: skein_codec::Reason::Bound }
}
fn encode_kinds(value: &fleet::Kinds) -> wire::Kinds {
    match value {
        fleet::Kinds::Workers => wire::Kinds::Workers,
        fleet::Kinds::Engine => wire::Kinds::Engine,
        fleet::Kinds::Both => wire::Kinds::Both,
    }
}
fn decode_kinds(value: wire::Kinds) -> fleet::Kinds {
    match value {
        wire::Kinds::Workers => fleet::Kinds::Workers,
        wire::Kinds::Engine => fleet::Kinds::Engine,
        wire::Kinds::Both => fleet::Kinds::Both,
    }
}
fn encode_setting(value: &settings::Setting, _limits: &wire::Limits) -> wire::Setting {
    match value {
        settings::Setting::ProjectPolicy { project } => {
            wire::Setting::ProjectPolicy(wire::ProjectPolicySetting { project: *project })
        }
        settings::Setting::Charter(entry) => wire::Setting::Charter(wire::CharterNumber { number: *entry }),
        settings::Setting::Template(entry) => wire::Setting::Template(wire::TemplateNumber { number: *entry }),
        settings::Setting::Policy(entry) => wire::Setting::Policy(wire::PolicyNumber { number: *entry }),
        settings::Setting::Rules => wire::Setting::Rules,
        settings::Setting::Limits => wire::Setting::Limits,
        settings::Setting::Model(number) => wire::Setting::Model(wire::ModelNumber { number: *number }),
        settings::Setting::Worker(number) => wire::Setting::Worker(wire::WorkerNumber { number: *number }),
        settings::Setting::Provider(number) => wire::Setting::Provider(wire::ProviderNumber { number: *number }),
        settings::Setting::Application { kind, scope, number } => {
            wire::Setting::Application(wire::ApplicationSetting { kind: *kind, scope: *scope, number: *number })
        }
    }
}
fn decode_setting(value: wire::Setting) -> settings::Setting {
    match value {
        wire::Setting::Charter(entry) => {
            let wire::CharterNumber { number } = entry;
            settings::Setting::Charter(number)
        }
        wire::Setting::Template(entry) => {
            let wire::TemplateNumber { number } = entry;
            settings::Setting::Template(number)
        }
        wire::Setting::Policy(entry) => {
            let wire::PolicyNumber { number } = entry;
            settings::Setting::Policy(number)
        }
        wire::Setting::ProjectPolicy(value) => settings::Setting::ProjectPolicy { project: value.project },
        wire::Setting::Rules => settings::Setting::Rules,
        wire::Setting::Limits => settings::Setting::Limits,
        wire::Setting::Model(number) => settings::Setting::Model(number.number),
        wire::Setting::Worker(number) => settings::Setting::Worker(number.number),
        wire::Setting::Provider(number) => settings::Setting::Provider(number.number),
        wire::Setting::Application(value) => {
            let wire::ApplicationSetting { kind, scope, number } = value;
            settings::Setting::Application { kind, scope, number }
        }
    }
}
fn encode_last(value: &authority::Last, _limits: &wire::Limits) -> wire::Last {
    match value {
        authority::Last::Exact(entry) => wire::Last::Exact(wire::LastExact { bytes: (*entry).clone() }),
        authority::Last::Open(entry) => wire::Last::Open(wire::LastOpen { bytes: (*entry).clone() }),
    }
}
fn decode_last(value: wire::Last) -> authority::Last {
    match value {
        wire::Last::Exact(entry) => {
            let wire::LastExact { bytes } = entry;
            authority::Last::Exact(bytes)
        }
        wire::Last::Open(entry) => {
            let wire::LastOpen { bytes } = entry;
            authority::Last::Open(bytes)
        }
    }
}
fn encode_executor(value: &authority::Executor, _limits: &wire::Limits) -> wire::Executor {
    match value {
        authority::Executor::Job(kind) => wire::Executor::Job(wire::ExecutorJob { kind: *kind }),
        authority::Executor::Charter(entry) => wire::Executor::Charter(wire::ExecutorCharter { number: *entry }),
        authority::Executor::Procedure(entry) => wire::Executor::Procedure(wire::ExecutorProcedure { number: *entry }),
        authority::Executor::Role(entry) => wire::Executor::Role(wire::ExecutorRole { number: *entry }),
    }
}
fn decode_executor(value: wire::Executor) -> authority::Executor {
    match value {
        wire::Executor::Job(entry) => authority::Executor::Job(entry.kind),
        wire::Executor::Charter(entry) => {
            let wire::ExecutorCharter { number } = entry;
            authority::Executor::Charter(number)
        }
        wire::Executor::Procedure(entry) => {
            let wire::ExecutorProcedure { number } = entry;
            authority::Executor::Procedure(number)
        }
        wire::Executor::Role(entry) => {
            let wire::ExecutorRole { number } = entry;
            authority::Executor::Role(number)
        }
    }
}
fn encode_guard(value: &authority::Guard, _limits: &wire::Limits) -> wire::Guard {
    match value {
        authority::Guard::Guarded => wire::Guard::Guarded,
        authority::Guard::Observed { freshness } => wire::Guard::Observed(wire::Observed { freshness: *freshness }),
    }
}
fn decode_guard(value: wire::Guard) -> authority::Guard {
    match value {
        wire::Guard::Guarded => authority::Guard::Guarded,
        wire::Guard::Observed(entry) => {
            let wire::Observed { freshness } = entry;
            authority::Guard::Observed { freshness }
        }
    }
}
fn encode_template_executor(value: &people::TemplateExecutor, _limits: &wire::Limits) -> wire::TemplateExecutor {
    match value {
        people::TemplateExecutor::Charter(entry) => {
            wire::TemplateExecutor::Charter(wire::TemplateCharter { number: *entry })
        }
        people::TemplateExecutor::Chosen => wire::TemplateExecutor::Chosen,
    }
}
fn decode_template_executor(value: wire::TemplateExecutor) -> people::TemplateExecutor {
    match value {
        wire::TemplateExecutor::Charter(entry) => {
            let wire::TemplateCharter { number } = entry;
            people::TemplateExecutor::Charter(number)
        }
        wire::TemplateExecutor::Chosen => people::TemplateExecutor::Chosen,
    }
}
fn encode_template_budget(value: &people::TemplateBudget, _limits: &wire::Limits) -> wire::TemplateBudget {
    match value {
        people::TemplateBudget::Amount(entry) => wire::TemplateBudget::Amount(wire::Amount { amount: *entry }),
        people::TemplateBudget::AsAllowed { cap } => wire::TemplateBudget::AsAllowed(wire::AsAllowed { cap: *cap }),
        people::TemplateBudget::Given => wire::TemplateBudget::Given,
    }
}
fn decode_template_budget(value: wire::TemplateBudget) -> people::TemplateBudget {
    match value {
        wire::TemplateBudget::Amount(entry) => {
            let wire::Amount { amount } = entry;
            people::TemplateBudget::Amount(amount)
        }
        wire::TemplateBudget::AsAllowed(entry) => {
            let wire::AsAllowed { cap } = entry;
            people::TemplateBudget::AsAllowed { cap }
        }
        wire::TemplateBudget::Given => people::TemplateBudget::Given,
    }
}
fn encode_template_deadline(value: &people::TemplateDeadline, _limits: &wire::Limits) -> wire::TemplateDeadline {
    match value {
        people::TemplateDeadline::None => wire::TemplateDeadline::None,
        people::TemplateDeadline::Span(entry) => wire::TemplateDeadline::Span(wire::Span { span: *entry }),
        people::TemplateDeadline::Given => wire::TemplateDeadline::Given,
    }
}
fn decode_template_deadline(value: wire::TemplateDeadline) -> people::TemplateDeadline {
    match value {
        wire::TemplateDeadline::None => people::TemplateDeadline::None,
        wire::TemplateDeadline::Span(entry) => {
            let wire::Span { span } = entry;
            people::TemplateDeadline::Span(span)
        }
        wire::TemplateDeadline::Given => people::TemplateDeadline::Given,
    }
}
fn encode_part_kind(value: &tasks::PartKind) -> wire::PartKind {
    match value {
        tasks::PartKind::Resource => wire::PartKind::Resource,
        tasks::PartKind::Pool => wire::PartKind::Pool,
        tasks::PartKind::Result => wire::PartKind::Result,
        tasks::PartKind::Procedure => wire::PartKind::Procedure,
        tasks::PartKind::Effect => wire::PartKind::Effect,
        tasks::PartKind::Outbox => wire::PartKind::Outbox,
        tasks::PartKind::Hold => wire::PartKind::Hold,
    }
}
fn decode_part_kind(value: wire::PartKind) -> tasks::PartKind {
    match value {
        wire::PartKind::Resource => tasks::PartKind::Resource,
        wire::PartKind::Pool => tasks::PartKind::Pool,
        wire::PartKind::Result => tasks::PartKind::Result,
        wire::PartKind::Procedure => tasks::PartKind::Procedure,
        wire::PartKind::Effect => tasks::PartKind::Effect,
        wire::PartKind::Outbox => tasks::PartKind::Outbox,
        wire::PartKind::Hold => tasks::PartKind::Hold,
    }
}
fn encode_part_key(value: &tasks::PartKey, limits: &wire::Limits) -> Result<wire::PartKey, wire::Problem> {
    Ok(match value {
        tasks::PartKey::Path(entry) => {
            let mut list_1 = List::with_capacity(limits.part_path_segments);
            for entry in entry {
                let element = encode_segment(entry, limits);
                if list_1.push(element).is_err() {
                    return Err(bound());
                }
            }
            wire::PartKey::Path(wire::PartPath { segments: list_1 })
        }
        tasks::PartKey::Number(entry) => wire::PartKey::Number(wire::PartNumber { number: *entry }),
    })
}
fn decode_part_key(value: wire::PartKey) -> tasks::PartKey {
    match value {
        wire::PartKey::Path(entry) => {
            let wire::PartPath { segments } = entry;
            let mut list_2 = List::with_capacity((segments).len());
            for entry in (segments).into_boxed() {
                let element = decode_segment(entry);
                list_2.push(element).expect("decoded list bound");
            }
            tasks::PartKey::Path(list_2)
        }
        wire::PartKey::Number(entry) => {
            let wire::PartNumber { number } = entry;
            tasks::PartKey::Number(number)
        }
    }
}
fn encode_contract(value: &tasks::Contract, limits: &wire::Limits) -> Result<wire::Contract, wire::Problem> {
    Ok(match value {
        tasks::Contract::Job { .. } => return Err(bound()),
        tasks::Contract::Approval { state, part } => {
            wire::Contract::Approval(wire::Approval { state: (*(state)).clone(), part: encode_part(part, limits)? })
        }
        tasks::Contract::Report { words } => wire::Contract::Report(wire::Report { words: *words }),
        tasks::Contract::Verdict { choices } => {
            let mut list_3 = List::with_capacity(limits.verdict_choices_choices);
            for entry in choices {
                let element = encode_verdict(entry, limits)?;
                if list_3.push(element).is_err() {
                    return Err(bound());
                }
            }
            wire::Contract::Verdict(wire::VerdictChoices { choices: list_3 })
        }
        tasks::Contract::Change { connector, kind, words, .. } => {
            wire::Contract::Change(wire::Change { connector: *connector, kind: *kind, words: *words })
        }
    })
}
fn decode_contract(value: wire::Contract, limits: &wire::Limits) -> Result<tasks::Contract, wire::Problem> {
    Ok(match value {
        wire::Contract::Approval(entry) => {
            let wire::Approval { state, part } = entry;
            tasks::Contract::Approval { state, part: decode_part(part) }
        }
        wire::Contract::Report(entry) => {
            let wire::Report { words } = entry;
            tasks::Contract::Report { words }
        }
        wire::Contract::Verdict(entry) => {
            let wire::VerdictChoices { choices } = entry;
            let mut list_4 = List::with_capacity((choices).len());
            for entry in (choices).into_boxed() {
                let element = decode_verdict(entry, limits)?;
                list_4.push(element).expect("decoded list bound");
            }
            tasks::Contract::Verdict { choices: list_4.into_boxed() }
        }
        wire::Contract::Change(entry) => {
            let wire::Change { connector, kind, words } = entry;
            tasks::Contract::Change { connector, kind, words, choices: Box::new([]) }
        }
    })
}
fn encode_retention(value: &tasks::Retention) -> wire::Retention {
    match value {
        tasks::Retention::Kept => wire::Retention::Kept,
        tasks::Retention::ForgottenOnceRead => wire::Retention::ForgottenOnceRead,
    }
}
fn decode_retention(value: wire::Retention) -> tasks::Retention {
    match value {
        wire::Retention::Kept => tasks::Retention::Kept,
        wire::Retention::ForgottenOnceRead => tasks::Retention::ForgottenOnceRead,
    }
}
fn encode_ended_by(value: &tasks::EndedBy, _limits: &wire::Limits) -> wire::EndedBy {
    match value {
        tasks::EndedBy::Result => wire::EndedBy::Result,
        tasks::EndedBy::Requester => wire::EndedBy::Requester,
        tasks::EndedBy::Standing => wire::EndedBy::Standing,
        tasks::EndedBy::Closer(closer) => {
            wire::EndedBy::Closer(wire::Closer { connector: closer.connector, code: closer.code })
        }
    }
}
fn decode_ended_by(value: wire::EndedBy) -> tasks::EndedBy {
    match value {
        wire::EndedBy::Result => tasks::EndedBy::Result,
        wire::EndedBy::Requester => tasks::EndedBy::Requester,
        wire::EndedBy::Standing => tasks::EndedBy::Standing,
        wire::EndedBy::Closer(closer) => {
            tasks::EndedBy::Closer(tasks::Closer { connector: closer.connector, code: closer.code })
        }
    }
}
fn encode_value(value: &settings::Value, limits: &wire::Limits) -> Result<wire::Value, wire::Problem> {
    Ok(match value {
        settings::Value::Charter(entry) => wire::Value::Charter(Box::new(encode_charter(entry, limits)?)),
        settings::Value::Template(entry) => wire::Value::Template(Box::new(encode_template(entry, limits)?)),
        settings::Value::Policy(entry) => wire::Value::Policy(Box::new(encode_configured_policy(entry, limits)?)),
        settings::Value::Rules(entry) => wire::Value::Rules(Box::new(encode_rules(entry, limits)?)),
        settings::Value::Limits(entry) => wire::Value::Limits(encode_limits(entry, limits)),
        settings::Value::Worker(value) => wire::Value::Worker(Box::new(wire::WorkerSetting {
            number: value.number,
            name: value.name.clone(),
            places: value.places,
            digest: value.digest,
            withdrawn: value.withdrawn,
        })),
        settings::Value::Provider(value) => wire::Value::Provider(Box::new(providers::encode(value, limits)?)),
        settings::Value::Model(value) => wire::Value::Model(Box::new(encode_catalogue(value, limits)?)),
        settings::Value::Overlay(item) => wire::Value::Overlay(Box::new(encode_overlay(item, limits)?)),
        settings::Value::Application(_) => return Err(bound()),
    })
}
fn decode_value(value: wire::Value, limits: &wire::Limits) -> Result<settings::Value, wire::Problem> {
    Ok(match value {
        wire::Value::Overlay(item) => settings::Value::Overlay(Box::new(decode_overlay(*item, limits)?)),
        wire::Value::Charter(entry) => settings::Value::Charter(Box::new(decode_charter(*entry, limits)?)),
        wire::Value::Template(entry) => settings::Value::Template(Box::new(decode_template(*entry, limits)?)),
        wire::Value::Policy(entry) => settings::Value::Policy(Box::new(decode_configured_policy(*entry))),
        wire::Value::Rules(entry) => settings::Value::Rules(Box::new(decode_rules(*entry, limits)?)),
        wire::Value::Limits(entry) => settings::Value::Limits(decode_limits(entry)),
        wire::Value::Worker(value) => settings::Value::Worker(Box::new(settings::WorkerSetting {
            number: value.number,
            name: value.name,
            places: value.places,
            digest: value.digest,
            withdrawn: value.withdrawn,
        })),
        wire::Value::Provider(value) => settings::Value::Provider(Box::new(providers::decode(&value))),
        wire::Value::Model(value) => settings::Value::Model(Box::new(decode_catalogue(*value))),
    })
}
fn encode_decider(value: &settings::Decider, _limits: &wire::Limits) -> wire::Decider {
    match value {
        settings::Decider::FirstStart => wire::Decider::FirstStart,
        settings::Decider::Release => wire::Decider::Release,
        settings::Decider::List { provider } => wire::Decider::List(wire::ListDecider { provider: *provider }),
        settings::Decider::Party(entry) => wire::Decider::Party(wire::Party { number: *entry }),
    }
}
fn decode_decider(value: wire::Decider) -> settings::Decider {
    match value {
        wire::Decider::FirstStart => settings::Decider::FirstStart,
        wire::Decider::Release => settings::Decider::Release,
        wire::Decider::List(entry) => settings::Decider::List { provider: entry.provider },
        wire::Decider::Party(entry) => {
            let wire::Party { number } = entry;
            settings::Decider::Party(number)
        }
    }
}
fn encode_proposer(value: &people::Proposer, _limits: &wire::Limits) -> wire::Proposer {
    match value {
        people::Proposer::Task(entry) => wire::Proposer::Task(wire::Party { number: *entry }),
        people::Proposer::Party(entry) => wire::Proposer::Party(wire::Party { number: *entry }),
    }
}
fn decode_proposer(value: wire::Proposer) -> people::Proposer {
    match value {
        wire::Proposer::Task(entry) => {
            let wire::Party { number } = entry;
            people::Proposer::Task(number)
        }
        wire::Proposer::Party(entry) => {
            let wire::Party { number } = entry;
            people::Proposer::Party(number)
        }
    }
}
fn encode_edit(value: &settings::Edit, limits: &wire::Limits) -> Result<wire::Edit, wire::Problem> {
    Ok(match value {
        settings::Edit::Set(entry) => wire::Edit::Set(wire::Set { value: encode_value(entry, limits)? }),
        settings::Edit::Removed => wire::Edit::Removed,
        settings::Edit::Reset => wire::Edit::Reset,
    })
}
fn decode_edit(value: wire::Edit, limits: &wire::Limits) -> Result<settings::Edit, wire::Problem> {
    Ok(match value {
        wire::Edit::Set(entry) => {
            let wire::Set { value } = entry;
            settings::Edit::Set(decode_value(value, limits)?)
        }
        wire::Edit::Removed => settings::Edit::Removed,
        wire::Edit::Reset => settings::Edit::Reset,
    })
}
fn encode_text(value: &[u8], _limits: &wire::Limits) -> wire::Text {
    let field_bytes = Box::from(value);
    wire::Text { bytes: field_bytes }
}
fn decode_text(value: wire::Text) -> Box<[u8]> {
    let wire::Text { bytes } = value;

    bytes
}
fn encode_model(value: &core::Model, limits: &wire::Limits) -> Result<wire::Model, wire::Problem> {
    let field_dialect = value.dialect;
    let field_endpoint = value.endpoint;
    let field_program = match &value.program {
        Some(entry) => Some(entry.clone()),
        None => None,
    };
    let field_name = value.name.clone();
    let field_window = value.window;
    let field_output = value.output;
    let field_input_price = value.input_price;
    let field_cached_price = value.cached_price;
    let field_cache_write_price = value.cache_write_price;
    let field_output_price = value.output_price;
    let field_price_unit = value.price_unit;
    let mut list_5 = List::with_capacity(limits.model_efforts);
    for entry in &value.efforts {
        let element = encode_text(entry, limits);
        if list_5.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_efforts = list_5;
    let field_effort = match &value.effort {
        Some(entry) => Some(entry.clone()),
        None => None,
    };
    Ok(wire::Model {
        provider: value.provider,
        dialect: field_dialect,
        endpoint: field_endpoint,
        program: field_program,
        name: field_name,
        window: field_window,
        cache_life_ms: value.cache_life_ms,
        output: field_output,
        input_price: field_input_price,
        cached_price: field_cached_price,
        cache_write_price: field_cache_write_price,
        output_price: field_output_price,
        price_unit: field_price_unit,
        efforts: field_efforts,
        effort: field_effort,
    })
}
fn decode_model(value: wire::Model) -> core::Model {
    let wire::Model {
        provider,
        dialect,
        endpoint,
        program,
        name,
        window,
        cache_life_ms,
        output,
        input_price,
        cached_price,
        cache_write_price,
        output_price,
        price_unit,
        efforts,
        effort,
    } = value;
    let field_dialect = dialect;
    let field_endpoint = endpoint;
    let field_program = program;
    let field_name = name;
    let field_window = window;
    let field_output = output;
    let field_input_price = input_price;
    let field_cached_price = cached_price;
    let field_cache_write_price = cache_write_price;
    let field_output_price = output_price;
    let field_price_unit = price_unit;
    let mut list_6 = List::with_capacity((efforts).len());
    for entry in (efforts).into_boxed() {
        let element = decode_text(entry);
        list_6.push(element).expect("decoded list bound");
    }
    let field_efforts = list_6.into_boxed();
    let field_effort = effort;
    core::Model {
        provider,
        dialect: field_dialect,
        endpoint: field_endpoint,
        program: field_program,
        name: field_name,
        window: field_window,
        cache_life_ms,
        output: field_output,
        input_price: field_input_price,
        cached_price: field_cached_price,
        cache_write_price: field_cache_write_price,
        output_price: field_output_price,
        price_unit: field_price_unit,
        efforts: field_efforts,
        effort: field_effort,
    }
}
fn encode_run_policy(value: &core::RunPolicy, limits: &wire::Limits) -> Result<wire::RunPolicy, wire::Problem> {
    let field_instructions = value.instructions.clone();
    let field_waiting = value.waiting;
    let field_resume = value.resume;
    let field_turns = value.turns;
    let field_time = value.time;
    let field_reserve_turns = value.reserve_turns;
    let field_reserve_time = value.reserve_time;
    let field_model = encode_model(&value.model, limits)?;
    let mut list_7 = List::with_capacity(limits.run_policy_alternatives);
    for entry in &value.alternatives {
        let element = encode_model(entry, limits)?;
        if list_7.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_alternatives = list_7;
    let field_inspect = value.inspect;
    let field_modify = value.modify;
    let field_shell = value.shell;
    let field_agents = value.agents;
    let field_call_timeout = value.call_timeout;
    let field_tools = value.tools.0;
    Ok(wire::RunPolicy {
        instructions: field_instructions,
        waiting: field_waiting,
        turns: field_turns,
        time: field_time,
        reserve_turns: field_reserve_turns,
        reserve_time: field_reserve_time,
        model: field_model,
        alternatives: field_alternatives,
        call_timeout: field_call_timeout,
        tools: field_tools,
        resume: field_resume,
        inspect: field_inspect,
        modify: field_modify,
        shell: field_shell,
        agents: field_agents,
    })
}
fn decode_run_policy(value: wire::RunPolicy) -> core::RunPolicy {
    let wire::RunPolicy {
        instructions,
        waiting,
        turns,
        time,
        reserve_turns,
        reserve_time,
        model,
        alternatives,
        call_timeout,
        tools,
        resume,
        inspect,
        modify,
        shell,
        agents,
    } = value;
    let field_instructions = instructions;
    let field_waiting = waiting;
    let field_resume = resume;
    let field_turns = turns;
    let field_time = time;
    let field_reserve_turns = reserve_turns;
    let field_reserve_time = reserve_time;
    let field_model = decode_model(model);
    let mut list_8 = List::with_capacity((alternatives).len());
    for entry in (alternatives).into_boxed() {
        let element = decode_model(entry);
        list_8.push(element).expect("decoded list bound");
    }
    let field_alternatives = list_8.into_boxed();
    let field_inspect = inspect;
    let field_modify = modify;
    let field_shell = shell;
    let field_agents = agents;
    let field_call_timeout = call_timeout;
    let field_tools = jig_domain_tasks::Tools(tools);
    core::RunPolicy {
        instructions: field_instructions,
        waiting: field_waiting,
        resume: field_resume,
        turns: field_turns,
        time: field_time,
        reserve_turns: field_reserve_turns,
        reserve_time: field_reserve_time,
        model: field_model,
        alternatives: field_alternatives,
        inspect: field_inspect,
        modify: field_modify,
        shell: field_shell,
        agents: field_agents,
        call_timeout: field_call_timeout,
        tools: field_tools,
    }
}
fn encode_charter(value: &core::Charter, limits: &wire::Limits) -> Result<wire::Charter, wire::Problem> {
    let field_number = value.number;
    let field_policy = encode_run_policy(&value.policy, limits)?;
    let field_hosts = encode_kinds(&value.hosts);
    let field_spend = value.spend;
    let mut documents = List::with_capacity(limits.charter_documents);
    // Hold paths share Segment, but document names follow the authority's byte bound.
    let name_limits = wire::Limits { segment_bytes: limits.last_exact_bytes, ..*limits };
    for document in &value.documents {
        let mut name = List::with_capacity(limits.document_name);
        for segment in &document.name.segments {
            let segment = encode_segment(segment, &name_limits);
            segment.measure(&name_limits)?;
            if name.push(segment).is_err() {
                return Err(bound());
            }
        }
        let document = wire::Document {
            connector: document.connector,
            name,
            path: document.path.clone(),
            heading: document.heading.clone(),
        };
        if documents.push(document).is_err() {
            return Err(bound());
        }
    }
    let mut parts = List::with_capacity(limits.charter_parts);
    for part in &value.parts {
        let part = wire::CharterPart { on: part.on.0, instructions: part.instructions.clone() };
        if parts.push(part).is_err() {
            return Err(bound());
        }
    }
    Ok(wire::Charter {
        rest: value.rest,
        number: field_number,
        policy: field_policy,
        hosts: field_hosts,
        spend: field_spend,
        wake: encode_wake(&value.wake, limits),
        documents,
        parts,
    })
}
fn decode_charter(value: wire::Charter, limits: &wire::Limits) -> Result<core::Charter, wire::Problem> {
    let wire::Charter { rest, number, policy, hosts, spend, wake, documents, parts } = value;
    let mut decoded = List::with_capacity(documents.len());
    for document in documents.into_boxed() {
        let wire::Document { connector, name, path, heading } = document;
        let mut segments = List::with_capacity(name.len());
        for segment in name.into_boxed() {
            let bytes = decode_segment(segment);
            if bytes.len() > skein_lib::length::usize_of(limits.last_exact_bytes) {
                return Err(bound());
            }
            segments.push(bytes).expect("decoded document name bound");
        }
        decoded
            .push(core::Document {
                connector,
                name: authority::Name { segments: segments.into_boxed() },
                path,
                heading,
            })
            .expect("decoded documents bound");
    }
    let mut decoded_parts = List::with_capacity(parts.len());
    for part in parts.into_boxed() {
        let wire::CharterPart { on, instructions } = part;
        let on = tasks::Reasons(on);
        if !on.is_known() || on.is_empty() {
            return Err(bound());
        }
        decoded_parts.push(core::CharterPart { on, instructions }).expect("decoded parts bound");
    }
    Ok(core::Charter {
        parts: decoded_parts.into_boxed(),
        rest,
        number,
        policy: decode_run_policy(policy),
        hosts: decode_kinds(hosts),
        spend,
        wake: decode_wake(wake),
        documents: decoded.into_boxed(),
    })
}
fn encode_segment(value: &[u8], _limits: &wire::Limits) -> wire::Segment {
    let field_bytes = Box::from(value);
    wire::Segment { bytes: field_bytes }
}
fn decode_segment(value: wire::Segment) -> Box<[u8]> {
    let wire::Segment { bytes } = value;

    bytes
}
fn encode_pattern(value: &authority::Pattern, limits: &wire::Limits) -> Result<wire::Pattern, wire::Problem> {
    let mut list_9 = List::with_capacity(limits.pattern_segments);
    for entry in &value.segments {
        let element = encode_segment(entry, limits);
        if list_9.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_segments = list_9;
    let field_last = encode_last(&value.last, limits);
    Ok(wire::Pattern { segments: field_segments, last: field_last })
}
fn decode_pattern(value: wire::Pattern) -> authority::Pattern {
    let wire::Pattern { segments, last } = value;
    let mut list_10 = List::with_capacity((segments).len());
    for entry in (segments).into_boxed() {
        let element = decode_segment(entry);
        list_10.push(element).expect("decoded list bound");
    }
    let field_segments = list_10.into_boxed();
    let field_last = decode_last(last);
    authority::Pattern { segments: field_segments, last: field_last }
}
fn encode_grant(value: &authority::Grant, limits: &wire::Limits) -> Result<wire::Grant, wire::Problem> {
    let field_connector = value.connector;
    let field_kind = value.kind;
    let field_pattern = encode_pattern(&value.pattern, limits)?;
    Ok(wire::Grant { connector: field_connector, kind: field_kind, pattern: field_pattern })
}
fn decode_grant(value: wire::Grant) -> authority::Grant {
    let wire::Grant { connector, kind, pattern } = value;
    let field_connector = connector;
    let field_kind = kind;
    let field_pattern = decode_pattern(pattern);
    authority::Grant { connector: field_connector, kind: field_kind, pattern: field_pattern }
}
fn encode_resource_scope(
    value: &authority::ResourceScope,
    limits: &wire::Limits,
) -> Result<wire::ResourceScope, wire::Problem> {
    let field_connector = value.connector;
    let field_pattern = encode_pattern(&value.pattern, limits)?;
    Ok(wire::ResourceScope { connector: field_connector, pattern: field_pattern })
}
fn decode_resource_scope(value: wire::ResourceScope) -> authority::ResourceScope {
    let wire::ResourceScope { connector, pattern } = value;
    let field_connector = connector;
    let field_pattern = decode_pattern(pattern);
    authority::ResourceScope { connector: field_connector, pattern: field_pattern }
}
fn encode_delegation(value: &authority::Delegation, limits: &wire::Limits) -> Result<wire::Delegation, wire::Problem> {
    let mut list_11 = List::with_capacity(limits.delegation_kinds);
    for entry in &value.kinds {
        let element = encode_executor(entry, limits);
        if list_11.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_kinds = list_11;
    let field_tasks = value.tasks;
    let field_depth = value.depth;
    Ok(wire::Delegation { kinds: field_kinds, tasks: field_tasks, depth: field_depth })
}
fn decode_delegation(value: wire::Delegation) -> authority::Delegation {
    let wire::Delegation { kinds, tasks, depth } = value;
    let mut list_12 = List::with_capacity((kinds).len());
    for entry in (kinds).into_boxed() {
        let element = decode_executor(entry);
        list_12.push(element).expect("decoded list bound");
    }
    let field_kinds = list_12.into_boxed();
    let field_tasks = tasks;
    let field_depth = depth;
    authority::Delegation { kinds: field_kinds, tasks: field_tasks, depth: field_depth }
}
fn encode_models(value: &authority::Models, _limits: &wire::Limits) -> wire::Models {
    let field_bits = value.bits();
    let field_all = value.is_all();
    wire::Models { bits: field_bits, all: field_all }
}
fn decode_models(value: wire::Models) -> authority::Models {
    let wire::Models { bits, all } = value;
    let field_bits = bits;
    let field_all = all;
    authority::Models::from_parts(field_bits, field_all)
}
fn encode_budget(value: &authority::Budget, _limits: &wire::Limits) -> wire::Budget {
    let field_spend = value.spend;
    let field_deadline = match value.deadline {
        Some(deadline) => Some(deadline.as_nanos()),
        None => None,
    };
    wire::Budget { spend: field_spend, deadline: field_deadline }
}
fn decode_budget(value: wire::Budget) -> authority::Budget {
    let wire::Budget { spend, deadline } = value;
    let field_spend = spend;
    let field_deadline = match deadline {
        Some(deadline) => Some(Wall::from_nanos(deadline)),
        None => None,
    };
    authority::Budget { spend: field_spend, deadline: field_deadline }
}
fn encode_authority(value: &authority::Authority, limits: &wire::Limits) -> Result<wire::Authority, wire::Problem> {
    let field_tools = value.tools.0;
    let field_models = encode_models(&value.models, limits);
    let mut list_13 = List::with_capacity(limits.authority_grants);
    for entry in &value.grants {
        let element = encode_grant(entry, limits)?;
        if list_13.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_grants = list_13;
    let field_delegation = encode_delegation(&value.delegation, limits)?;
    let field_budget = encode_budget(&value.budget, limits);
    let field_notes = value.notes.0;
    let mut list_14 = List::with_capacity(limits.authority_note_resources);
    for entry in &value.note_resources {
        let element = encode_resource_scope(entry, limits)?;
        if list_14.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_note_resources = list_14;
    Ok(wire::Authority {
        tools: field_tools,
        models: field_models,
        grants: field_grants,
        delegation: field_delegation,
        budget: field_budget,
        notes: field_notes,
        note_resources: field_note_resources,
    })
}
fn decode_authority(value: wire::Authority) -> authority::Authority {
    let wire::Authority { tools, models, grants, delegation, budget, notes, note_resources } = value;
    let field_tools = authority::Tools(tools);
    let field_models = decode_models(models);
    let mut list_15 = List::with_capacity((grants).len());
    for entry in (grants).into_boxed() {
        let element = decode_grant(entry);
        list_15.push(element).expect("decoded list bound");
    }
    let field_grants = list_15.into_boxed();
    let field_delegation = decode_delegation(delegation);
    let field_budget = decode_budget(budget);
    let field_notes = authority::Scopes(notes);
    let mut list_16 = List::with_capacity((note_resources).len());
    for entry in (note_resources).into_boxed() {
        let element = decode_resource_scope(entry);
        list_16.push(element).expect("decoded list bound");
    }
    let field_note_resources = list_16.into_boxed();
    authority::Authority {
        tools: field_tools,
        models: field_models,
        grants: field_grants,
        delegation: field_delegation,
        budget: field_budget,
        notes: field_notes,
        note_resources: field_note_resources,
    }
}
fn encode_judge(value: &authority::Judge, _limits: &wire::Limits) -> wire::Judge {
    let field_connector = value.connector;
    let field_requirement = value.requirement;
    let field_parameters = value.parameters;
    wire::Judge { connector: field_connector, requirement: field_requirement, parameters: field_parameters }
}
fn decode_judge(value: wire::Judge) -> authority::Judge {
    let wire::Judge { connector, requirement, parameters } = value;
    let field_connector = connector;
    let field_requirement = requirement;
    let field_parameters = parameters;
    authority::Judge { connector: field_connector, requirement: field_requirement, parameters: field_parameters }
}
fn encode_requirement(
    value: &authority::Requirement,
    limits: &wire::Limits,
) -> Result<wire::Requirement, wire::Problem> {
    let field_connector = value.connector;
    let field_kind = value.kind;
    let field_pattern = encode_pattern(&value.pattern, limits)?;
    let field_judge = encode_judge(&value.judge, limits);
    let field_guard = encode_guard(&value.guard, limits);
    let field_must_be_guarded = value.must_be_guarded;
    Ok(wire::Requirement {
        connector: field_connector,
        kind: field_kind,
        pattern: field_pattern,
        judge: field_judge,
        guard: field_guard,
        must_be_guarded: field_must_be_guarded,
    })
}
fn decode_requirement(value: wire::Requirement) -> authority::Requirement {
    let wire::Requirement { connector, kind, pattern, judge, guard, must_be_guarded } = value;
    let field_connector = connector;
    let field_kind = kind;
    let field_pattern = decode_pattern(pattern);
    let field_judge = decode_judge(judge);
    let field_guard = decode_guard(guard);
    let field_must_be_guarded = must_be_guarded;
    authority::Requirement {
        connector: field_connector,
        kind: field_kind,
        pattern: field_pattern,
        judge: field_judge,
        guard: field_guard,
        must_be_guarded: field_must_be_guarded,
    }
}
fn encode_implication(value: &authority::Implication, _limits: &wire::Limits) -> wire::Implication {
    let field_connector = value.connector;
    let field_kind = value.kind;
    let field_implies = value.implies;
    wire::Implication { connector: field_connector, kind: field_kind, implies: field_implies }
}
fn decode_implication(value: wire::Implication) -> authority::Implication {
    let wire::Implication { connector, kind, implies } = value;
    let field_connector = connector;
    let field_kind = kind;
    let field_implies = implies;
    authority::Implication { connector: field_connector, kind: field_kind, implies: field_implies }
}
fn encode_rules(value: &authority::Rules, limits: &wire::Limits) -> Result<wire::Rules, wire::Problem> {
    let field_ceiling = encode_authority(&value.ceiling, limits)?;
    let field_period_spend = value.period_spend;
    let field_minimum_run_spend = value.minimum_run_spend;
    let field_maximum_run_spend = value.maximum_run_spend;
    let mut list_17 = List::with_capacity(limits.rules_implies);
    for entry in value.implies.entries() {
        let element = encode_implication(entry, limits);
        if list_17.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_implies = list_17;
    let mut list_18 = List::with_capacity(limits.rules_requirements);
    for entry in &value.requirements {
        let element = encode_requirement(entry, limits)?;
        if list_18.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_requirements = list_18;
    Ok(wire::Rules {
        ceiling: field_ceiling,
        period_spend: field_period_spend,
        minimum_run_spend: field_minimum_run_spend,
        maximum_run_spend: field_maximum_run_spend,
        implies: field_implies,
        requirements: field_requirements,
    })
}
fn decode_rules(value: wire::Rules, limits: &wire::Limits) -> Result<authority::Rules, wire::Problem> {
    let wire::Rules { ceiling, period_spend, minimum_run_spend, maximum_run_spend, implies, requirements } = value;
    let field_ceiling = decode_authority(ceiling);
    let field_period_spend = period_spend;
    let field_minimum_run_spend = minimum_run_spend;
    let field_maximum_run_spend = maximum_run_spend;
    let mut list_19 = List::with_capacity((implies).len());
    for entry in (implies).into_boxed() {
        let element = decode_implication(entry);
        list_19.push(element).expect("decoded list bound");
    }
    let field_implies = authority::Implies::new(list_19.into_boxed(), limits.rules_implies).ok_or(bound())?;
    let mut list_20 = List::with_capacity((requirements).len());
    for entry in (requirements).into_boxed() {
        let element = decode_requirement(entry);
        list_20.push(element).expect("decoded list bound");
    }
    let field_requirements = list_20.into_boxed();
    Ok(authority::Rules {
        ceiling: field_ceiling,
        period_spend: field_period_spend,
        minimum_run_spend: field_minimum_run_spend,
        maximum_run_spend: field_maximum_run_spend,
        implies: field_implies,
        requirements: field_requirements,
    })
}
fn encode_role(value: &authority::Role, limits: &wire::Limits) -> Result<wire::Role, wire::Problem> {
    let field_number = value.number;
    let field_authority = encode_authority(&value.authority, limits)?;
    let field_period_spend = value.period_spend;
    let field_requests = value.requests.0;
    let field_decides = value.decides.0;
    Ok(wire::Role {
        number: field_number,
        authority: field_authority,
        period_spend: field_period_spend,
        requests: field_requests,
        decides: field_decides,
    })
}
fn decode_role(value: wire::Role) -> authority::Role {
    let wire::Role { number, authority, period_spend, requests, decides } = value;
    let field_number = number;
    let field_authority = decode_authority(authority);
    let field_period_spend = period_spend;
    let field_requests = authority::Requests(requests);
    let field_decides = authority::Proposals(decides);
    authority::Role {
        number: field_number,
        authority: field_authority,
        period_spend: field_period_spend,
        requests: field_requests,
        decides: field_decides,
    }
}
fn encode_policy(value: &authority::Policy, limits: &wire::Limits) -> Result<wire::Policy, wire::Problem> {
    let field_escalation_role = value.escalation_role;
    let field_ceiling = encode_authority(&value.ceiling, limits)?;
    let mut list_21 = List::with_capacity(limits.policy_projections);
    for entry in &value.projections {
        let element = encode_grant(entry, limits)?;
        if list_21.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_projections = list_21;
    let field_period_spend = value.period_spend;
    let mut list_22 = List::with_capacity(limits.policy_roles);
    for entry in &value.roles {
        let element = encode_role(entry, limits)?;
        if list_22.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_roles = list_22;
    let mut list_23 = List::with_capacity(limits.policy_requirements);
    for entry in &value.requirements {
        let element = encode_requirement(entry, limits)?;
        if list_23.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_requirements = list_23;
    Ok(wire::Policy {
        escalation_role: field_escalation_role,
        ceiling: field_ceiling,
        projections: field_projections,
        period_spend: field_period_spend,
        roles: field_roles,
        requirements: field_requirements,
    })
}
fn decode_policy(value: wire::Policy) -> authority::Policy {
    let wire::Policy { escalation_role, ceiling, projections, period_spend, roles, requirements } = value;
    let field_escalation_role = escalation_role;
    let field_ceiling = decode_authority(ceiling);
    let mut list_24 = List::with_capacity((projections).len());
    for entry in (projections).into_boxed() {
        let element = decode_grant(entry);
        list_24.push(element).expect("decoded list bound");
    }
    let field_projections = list_24.into_boxed();
    let field_period_spend = period_spend;
    let mut list_25 = List::with_capacity((roles).len());
    for entry in (roles).into_boxed() {
        let element = decode_role(entry);
        list_25.push(element).expect("decoded list bound");
    }
    let field_roles = list_25.into_boxed();
    let mut list_26 = List::with_capacity((requirements).len());
    for entry in (requirements).into_boxed() {
        let element = decode_requirement(entry);
        list_26.push(element).expect("decoded list bound");
    }
    let field_requirements = list_26.into_boxed();
    authority::Policy {
        escalation_role: field_escalation_role,
        ceiling: field_ceiling,
        projections: field_projections,
        period_spend: field_period_spend,
        roles: field_roles,
        requirements: field_requirements,
    }
}
fn encode_permission_role(value: &people::PermissionRole, _limits: &wire::Limits) -> wire::PermissionRole {
    let field_connector = value.connector;
    let field_permission = value.permission;
    let field_role = value.role;
    wire::PermissionRole { connector: field_connector, permission: field_permission, role: field_role }
}
fn decode_permission_role(value: wire::PermissionRole) -> people::PermissionRole {
    let wire::PermissionRole { connector, permission, role } = value;
    let field_connector = connector;
    let field_permission = permission;
    let field_role = role;
    people::PermissionRole { connector: field_connector, permission: field_permission, role: field_role }
}
fn encode_template_override(value: &people::TemplateOverride, limits: &wire::Limits) -> wire::TemplateOverride {
    let field_template = value.template;
    let field_executor = match &value.executor {
        Some(entry) => Some(encode_template_executor(entry, limits)),
        None => None,
    };
    let field_budget = match &value.budget {
        Some(entry) => Some(encode_template_budget(entry, limits)),
        None => None,
    };
    let field_deadline = match &value.deadline {
        Some(entry) => Some(encode_template_deadline(entry, limits)),
        None => None,
    };
    let field_priority = value.priority;
    wire::TemplateOverride {
        template: field_template,
        executor: field_executor,
        budget: field_budget,
        deadline: field_deadline,
        priority: field_priority,
    }
}
fn decode_template_override(value: wire::TemplateOverride) -> people::TemplateOverride {
    let wire::TemplateOverride { template, executor, budget, deadline, priority } = value;
    let field_template = template;
    let field_executor = match executor {
        Some(entry) => Some(decode_template_executor(entry)),
        None => None,
    };
    let field_budget = match budget {
        Some(entry) => Some(decode_template_budget(entry)),
        None => None,
    };
    let field_deadline = match deadline {
        Some(entry) => Some(decode_template_deadline(entry)),
        None => None,
    };
    let field_priority = priority;
    people::TemplateOverride {
        template: field_template,
        executor: field_executor,
        budget: field_budget,
        deadline: field_deadline,
        priority: field_priority,
    }
}
fn encode_configured_policy(
    value: &core::ConfiguredPolicy,
    limits: &wire::Limits,
) -> Result<wire::ConfiguredPolicy, wire::Problem> {
    let field_policy = encode_policy(&value.policy, limits)?;
    let mut list_27 = List::with_capacity(limits.configured_policy_permissions);
    for entry in &value.permissions {
        let element = encode_permission_role(entry, limits);
        if list_27.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_permissions = list_27;
    let mut list_28 = List::with_capacity(limits.configured_policy_templates);
    for entry in &value.templates {
        let element = encode_template_override(entry, limits);
        if list_28.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_templates = list_28;
    Ok(wire::ConfiguredPolicy { policy: field_policy, permissions: field_permissions, templates: field_templates })
}
fn decode_configured_policy(value: wire::ConfiguredPolicy) -> core::ConfiguredPolicy {
    let wire::ConfiguredPolicy { policy, permissions, templates } = value;
    let field_policy = decode_policy(policy);
    let mut list_29 = List::with_capacity((permissions).len());
    for entry in (permissions).into_boxed() {
        let element = decode_permission_role(entry);
        list_29.push(element).expect("decoded list bound");
    }
    let field_permissions = list_29.into_boxed();
    let mut list_30 = List::with_capacity((templates).len());
    for entry in (templates).into_boxed() {
        let element = decode_template_override(entry);
        list_30.push(element).expect("decoded list bound");
    }
    let field_templates = list_30.into_boxed();
    core::ConfiguredPolicy { policy: field_policy, permissions: field_permissions, templates: field_templates }
}
fn verdict_problem() -> wire::Problem {
    wire::Problem { path: wire::paths::VERDICT_FIELDS, reason: skein_codec::Reason::Bound }
}
fn admit_rule(used: &mut usize, name: &[u8], max: u32, limits: &wire::Limits) -> Result<(), wire::Problem> {
    *used = used.saturating_add(name.len()).saturating_add(skein_lib::length::usize_of(max));
    if name.is_empty() || *used > skein_lib::length::usize_of(limits.field_rule_name) {
        return Err(verdict_problem());
    }
    Ok(())
}
fn encode_verdict(value: &tasks::Verdict, limits: &wire::Limits) -> Result<wire::Verdict, wire::Problem> {
    let Ok(count) = u32::try_from(value.fields.len()) else {
        return Err(verdict_problem());
    };
    if count > tasks::VERDICT_FIELDS {
        return Err(verdict_problem());
    }
    let mut items = List::with_capacity(count);
    let mut used = 0;
    for field in &value.fields {
        let tasks::FieldRule { name, max } = field;
        admit_rule(&mut used, name, *max, limits)?;
        let field = wire::FieldRule { name: name.clone(), max: *max };
        let item = field.encode(limits)?;
        items.push(item).expect("same field count");
    }
    let fields = match super::pieces::join(items, limits.verdict_fields) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(wire::Problem { path: wire::paths::VERDICT_FIELDS, reason }),
    };
    Ok(wire::Verdict { code: value.code, words: value.words, followups: value.followups, fields })
}
fn decode_verdict(value: wire::Verdict, limits: &wire::Limits) -> Result<tasks::Verdict, wire::Problem> {
    let wire::Verdict { code, words, followups, fields } = value;
    let mut reader = Reader::new(&fields);
    let mut values = List::with_capacity(tasks::VERDICT_FIELDS.min(limits.field_rule_name));
    let mut used = 0;
    for _ in 0..=tasks::VERDICT_FIELDS {
        let bytes = match skein_codec::piece::split(&mut reader) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(tasks::Verdict { code, words, followups, fields: values.into_boxed() }),
            Err(reason) => return Err(wire::Problem { path: wire::paths::VERDICT_FIELDS, reason }),
        };
        let wire::FieldRule { name, max } = wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?;
        admit_rule(&mut used, &name, max, limits)?;
        if values.push(tasks::FieldRule { name, max }).is_err() {
            return Err(verdict_problem());
        }
    }
    Err(verdict_problem())
}
fn encode_part(value: &tasks::Part, limits: &wire::Limits) -> Result<wire::Part, wire::Problem> {
    let field_connector = value.connector;
    let field_kind = encode_part_kind(&value.kind);
    let field_key = encode_part_key(&value.key, limits)?;
    Ok(wire::Part { connector: field_connector, kind: field_kind, key: field_key })
}
fn decode_part(value: wire::Part) -> tasks::Part {
    let wire::Part { connector, kind, key } = value;
    let field_connector = connector;
    let field_kind = decode_part_kind(kind);
    let field_key = decode_part_key(key);
    tasks::Part { connector: field_connector, kind: field_kind, key: field_key }
}
fn encode_flags(value: &tasks::Flags, limits: &wire::Limits) -> wire::Flags {
    let field_conversation = value.conversation;
    let field_listed = value.listed;
    let field_retention = encode_retention(&value.retention);
    let field_ended_by = encode_ended_by(&value.ended_by, limits);
    wire::Flags {
        conversation: field_conversation,
        listed: field_listed,
        retention: field_retention,
        ended_by: field_ended_by,
    }
}
fn decode_flags(value: wire::Flags) -> tasks::Flags {
    let wire::Flags { conversation, listed, retention, ended_by } = value;
    let field_conversation = conversation;
    let field_listed = listed;
    let field_retention = decode_retention(retention);
    let field_ended_by = decode_ended_by(ended_by);
    tasks::Flags {
        conversation: field_conversation,
        listed: field_listed,
        retention: field_retention,
        ended_by: field_ended_by,
    }
}
fn encode_model_effort(value: &tasks::ModelEffort, _limits: &wire::Limits) -> wire::ModelEffort {
    let field_model = value.model;
    let field_effort = value.effort;
    wire::ModelEffort { model: field_model, effort: field_effort }
}
fn decode_model_effort(value: wire::ModelEffort) -> tasks::ModelEffort {
    let wire::ModelEffort { model, effort } = value;
    let field_model = model;
    let field_effort = effort;
    tasks::ModelEffort { model: field_model, effort: field_effort }
}
fn encode_llm_choice(value: &tasks::LlmChoice, limits: &wire::Limits) -> Result<wire::LlmChoice, wire::Problem> {
    let field_account = value.account;
    let field_main = encode_model_effort(&value.main, limits);
    let mut list_33 = List::with_capacity(limits.llm_choice_subagents);
    for entry in &value.subagents {
        let element = encode_model_effort(entry, limits);
        if list_33.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_subagents = list_33;
    let later = match &value.later {
        None => None,
        Some(later) => {
            let mut subagents = List::with_capacity(limits.llm_later_subagents);
            for entry in &later.subagents {
                if subagents.push(encode_model_effort(entry, limits)).is_err() {
                    return Err(bound());
                }
            }
            Some(wire::LlmLater { main: encode_model_effort(&later.main, limits), subagents })
        }
    };
    Ok(wire::LlmChoice { account: field_account, main: field_main, subagents: field_subagents, later })
}
fn decode_llm_choice(value: wire::LlmChoice) -> tasks::LlmChoice {
    let wire::LlmChoice { account, main, subagents, later } = value;
    let field_account = account;
    let field_main = decode_model_effort(main);
    let mut list_34 = List::with_capacity((subagents).len());
    for entry in (subagents).into_boxed() {
        let element = decode_model_effort(entry);
        list_34.push(element).expect("decoded list bound");
    }
    let field_subagents = list_34.into_boxed();
    let later = match later {
        None => None,
        Some(later) => {
            let wire::LlmLater { main, subagents } = later;
            let mut chosen = List::with_capacity(subagents.len());
            for entry in subagents.into_boxed() {
                chosen.push(decode_model_effort(entry)).expect("decoded later sub-agents bound");
            }
            Some(tasks::LlmLater { main: decode_model_effort(main), subagents: chosen.into_boxed() })
        }
    };
    tasks::LlmChoice { later, account: field_account, main: field_main, subagents: field_subagents }
}
fn encode_template(value: &core::Template, limits: &wire::Limits) -> Result<wire::Template, wire::Problem> {
    let field_number = value.number;
    let field_name = value.name.clone();
    let field_executor = encode_template_executor(&value.executor, limits);
    let field_contract = encode_contract(&value.contract, limits)?;
    let field_shallower = value.shallower;
    let field_budget = encode_template_budget(&value.budget, limits);
    let field_deadline = encode_template_deadline(&value.deadline, limits);
    let field_tracked = value.tracked;
    let field_flags = encode_flags(&value.flags, limits);
    let field_label = value.label;
    let field_grace = value.grace;
    let field_llm = match &value.llm {
        Some(entry) => Some(encode_llm_choice(entry, limits)?),
        None => None,
    };
    let mut resources = List::with_capacity(limits.template_resources);
    for resource in &value.resources {
        let symbol = wire::Symbol { connector: resource.connector, symbol: resource.symbol };
        if resources.push(symbol).is_err() {
            return Err(wire::Problem { path: wire::paths::TEMPLATE_RESOURCES, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(wire::Template {
        number: field_number,
        name: field_name,
        executor: field_executor,
        contract: field_contract,
        shallower: field_shallower,
        budget: field_budget,
        deadline: field_deadline,
        tracked: field_tracked,
        flags: field_flags,
        label: field_label,
        grace: field_grace,
        llm: field_llm,
        resources,
    })
}
fn decode_template(value: wire::Template, limits: &wire::Limits) -> Result<core::Template, wire::Problem> {
    let wire::Template {
        number,
        name,
        executor,
        contract,
        shallower,
        budget,
        deadline,
        tracked,
        flags,
        label,
        grace,
        llm,
        resources,
    } = value;
    let field_number = number;
    let field_name = name;
    let field_executor = decode_template_executor(executor);
    let field_contract = decode_contract(contract, limits)?;
    let field_shallower = shallower;
    let field_budget = decode_template_budget(budget);
    let field_deadline = decode_template_deadline(deadline);
    let field_tracked = tracked;
    let field_flags = decode_flags(flags);
    let field_label = label;
    let field_grace = grace;
    let field_llm = match llm {
        Some(entry) => Some(decode_llm_choice(entry)),
        None => None,
    };
    let mut symbols = List::with_capacity(resources.len());
    for resource in resources.into_boxed() {
        let wire::Symbol { connector, symbol } = resource;
        symbols.push(people::Symbol { connector, symbol }).expect("decoded resources bound");
    }
    Ok(core::Template {
        resources: symbols.into_boxed(),
        number: field_number,
        name: field_name,
        executor: field_executor,
        contract: field_contract,
        shallower: field_shallower,
        budget: field_budget,
        deadline: field_deadline,
        tracked: field_tracked,
        flags: field_flags,
        label: field_label,
        grace: field_grace,
        llm: field_llm,
    })
}
fn encode_proposal_id(value: &(people::Proposer, u32), limits: &wire::Limits) -> wire::ProposalId {
    let field_proposer = encode_proposer(&value.0, limits);
    let field_number = value.1;
    wire::ProposalId { proposer: field_proposer, number: field_number }
}
fn decode_proposal_id(value: wire::ProposalId) -> (people::Proposer, u32) {
    let wire::ProposalId { proposer, number } = value;
    let field_proposer = decode_proposer(proposer);
    let field_number = number;
    (field_proposer, field_number)
}
fn encode_head(value: &settings::Head, limits: &wire::Limits) -> wire::Head {
    let field_number = value.number;
    let field_against = value.against;
    let field_at = value.at.as_nanos();
    let field_by = encode_decider(&value.by, limits);
    let field_proposal = match &value.proposal {
        Some(entry) => Some(encode_proposal_id(entry, limits)),
        None => None,
    };
    let field_moved = value.moved;
    wire::Head {
        number: field_number,
        against: field_against,
        at: field_at,
        by: field_by,
        proposal: field_proposal,
        moved: field_moved,
    }
}
fn decode_head(value: wire::Head) -> settings::Head {
    let wire::Head { number, against, at, by, proposal, moved } = value;
    let field_number = number;
    let field_against = against;
    let field_at = Wall::from_nanos(at);
    let field_by = decode_decider(by);
    let field_proposal = match proposal {
        Some(entry) => Some(decode_proposal_id(entry)),
        None => None,
    };
    let field_moved = moved;
    settings::Head {
        number: field_number,
        against: field_against,
        at: field_at,
        by: field_by,
        proposal: field_proposal,
        moved: field_moved,
    }
}
fn encode_item(value: &settings::Item, limits: &wire::Limits) -> Result<wire::Item, wire::Problem> {
    let field_setting = encode_setting(&value.setting, limits);
    let field_edit = encode_edit(&value.edit, limits)?;
    Ok(wire::Item { setting: field_setting, edit: field_edit })
}
fn decode_item(value: wire::Item, limits: &wire::Limits) -> Result<settings::Item, wire::Problem> {
    let wire::Item { setting, edit } = value;
    let field_setting = decode_setting(setting);
    let field_edit = decode_edit(edit, limits)?;
    Ok(settings::Item { setting: field_setting, edit: field_edit })
}
fn encode_default(
    value: &settings::Default,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Default, wire::Problem> {
    let field_value = encode_value(&value.value, limits)?;
    let field_revision = value.revision;
    Ok(wire::Default { models: encode_names(names, limits)?, value: field_value, revision: field_revision })
}
fn decode_default(
    value: wire::Default,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Default, wire::Problem> {
    let wire::Default { models, value, revision } = value;
    let field_value = decode_value(value, limits)?;
    let field_revision = revision;
    let mut field_value = field_value;
    let kept = decode_names(models);
    settings::remap(&mut field_value, &settings::moves(&kept, names), &kept, names);
    Ok(settings::Default { value: field_value, revision: field_revision })
}
fn encode_changed(
    value: &settings::Changed,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Changed, wire::Problem> {
    let field_value = match &value.value {
        Some(entry) => Some(encode_value(entry, limits)?),
        None => None,
    };
    let field_revision = value.revision;
    Ok(wire::Changed { models: encode_names(names, limits)?, value: field_value, revision: field_revision })
}
fn decode_changed(
    value: wire::Changed,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Changed, wire::Problem> {
    let wire::Changed { models, value, revision } = value;
    let field_value = match value {
        Some(entry) => Some(decode_value(entry, limits)?),
        None => None,
    };
    let field_revision = revision;
    let mut field_value = field_value;
    let kept = decode_names(models);
    if let Some(value) = &mut field_value {
        settings::remap(value, &settings::moves(&kept, names), &kept, names);
    }
    Ok(settings::Changed { value: field_value, revision: field_revision })
}
fn encode_replaced(value: &settings::Replaced, limits: &wire::Limits) -> Result<wire::Replaced, wire::Problem> {
    Ok(match value {
        settings::Replaced::Value(value) => wire::Replaced::Value(wire::Set { value: encode_value(value, limits)? }),
        settings::Replaced::Default => wire::Replaced::Default,
        settings::Replaced::Removed => wire::Replaced::Removed,
        settings::Replaced::NotOffered => wire::Replaced::NotOffered,
    })
}
fn decode_replaced(value: wire::Replaced, limits: &wire::Limits) -> Result<settings::Replaced, wire::Problem> {
    Ok(match value {
        wire::Replaced::Value(value) => settings::Replaced::Value(decode_value(value.value, limits)?),
        wire::Replaced::Default => settings::Replaced::Default,
        wire::Replaced::Removed => settings::Replaced::Removed,
        wire::Replaced::NotOffered => settings::Replaced::NotOffered,
    })
}
fn encode_revision_item(
    value: &settings::RevisionItem,
    limits: &wire::Limits,
) -> Result<wire::RevisionItem, wire::Problem> {
    Ok(wire::RevisionItem {
        setting: encode_setting(&value.setting, limits),
        edit: encode_edit(&value.edit, limits)?,
        replaced: encode_replaced(&value.replaced, limits)?,
    })
}
fn decode_revision_item(
    value: wire::RevisionItem,
    limits: &wire::Limits,
) -> Result<settings::RevisionItem, wire::Problem> {
    let wire::RevisionItem { setting, edit, replaced } = value;
    Ok(settings::RevisionItem {
        setting: decode_setting(setting),
        edit: decode_edit(edit, limits)?,
        replaced: decode_replaced(replaced, limits)?,
    })
}
fn encode_revision_recorded(value: &application::RevisionRecorded, _limits: &wire::Limits) -> wire::RevisionRecorded {
    let edit = match value.edit {
        application::SizedEdit::Set { size } => wire::SizedEdit::Set(wire::SizedSet { size }),
        application::SizedEdit::Remove => wire::SizedEdit::Remove,
        application::SizedEdit::Reset => wire::SizedEdit::Reset,
    };
    let replaced = match value.replaced {
        application::Replaced::Changed { size } => wire::SizedReplaced::Changed(wire::SizedSet { size }),
        application::Replaced::Default => wire::SizedReplaced::Default,
        application::Replaced::Removed => wire::SizedReplaced::Removed,
        application::Replaced::NotOffered => wire::SizedReplaced::NotOffered,
    };
    wire::RevisionRecorded { kind: value.kind, scope: value.scope, number: value.number, edit, replaced }
}
fn decode_revision_recorded(value: wire::RevisionRecorded) -> application::RevisionRecorded {
    let wire::RevisionRecorded { kind, scope, number, edit, replaced } = value;
    let edit = match edit {
        wire::SizedEdit::Set(value) => application::SizedEdit::Set { size: value.size },
        wire::SizedEdit::Remove => application::SizedEdit::Remove,
        wire::SizedEdit::Reset => application::SizedEdit::Reset,
    };
    let replaced = match replaced {
        wire::SizedReplaced::Changed(value) => application::Replaced::Changed { size: value.size },
        wire::SizedReplaced::Default => application::Replaced::Default,
        wire::SizedReplaced::Removed => application::Replaced::Removed,
        wire::SizedReplaced::NotOffered => application::Replaced::NotOffered,
    };
    application::RevisionRecorded { kind, scope, number, edit, replaced }
}
fn encode_revision(
    value: &settings::Revision,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Revision, wire::Problem> {
    let mut items = List::with_capacity(limits.proposal_items);
    for item in &value.items {
        let item = encode_revision_item(item, limits)?;
        let writer = item.encode(limits)?;
        if items.push(writer).is_err() {
            return Err(bound());
        }
    }
    let items = match super::pieces::join(items, limits.revision_items) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason }),
    };
    let mut applications = List::with_capacity(limits.revision_applications);
    for item in &value.applications {
        let item = encode_revision_recorded(item, limits);
        if applications.push(item).is_err() {
            return Err(bound());
        }
    }
    Ok(wire::Revision {
        models: encode_names(names, limits)?,
        head: encode_head(&value.head, limits),
        items,
        applications,
    })
}
fn decode_revision(
    value: wire::Revision,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Revision, wire::Problem> {
    let wire::Revision { models, head, items, applications } = value;
    let mut reader = Reader::new(&items);
    let mut items = List::with_capacity(limits.proposal_items);
    for _ in 0..limits.proposal_items {
        let bytes = match super::people::pieces::split(&mut reader) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => break,
            Err(problem) => return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason: problem.reason }),
        };
        let mut piece = Reader::new(&bytes);
        let item = wire::RevisionItem::decode(limits, &mut piece)?;
        if !piece.is_empty() {
            return Err(wire::Problem { path: wire::paths::REVISION_ITEMS, reason: skein_codec::Reason::Malformed });
        }
        let item = decode_revision_item(item, limits)?;
        if items.push(item).is_err() {
            return Err(bound());
        }
    }
    if !reader.is_empty() {
        return Err(bound());
    }
    let mut items = items.into_boxed();
    let kept = decode_names(models);
    let moved = settings::moves(&kept, names);
    for item in &mut items {
        match &mut item.edit {
            settings::Edit::Set(value) => settings::remap(value, &moved, &kept, names),
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
        match &mut item.replaced {
            settings::Replaced::Value(value) => settings::remap(value, &moved, &kept, names),
            settings::Replaced::Default | settings::Replaced::Removed | settings::Replaced::NotOffered => {}
        }
    }
    let mut recorded = List::with_capacity(applications.len());
    for item in applications.into_boxed() {
        recorded.push(decode_revision_recorded(item)).expect("decoded application item count");
    }
    Ok(settings::Revision { head: decode_head(head), items, applications: recorded.into_boxed() })
}
fn encode_charter_version(
    value: &settings::CharterVersion,
    limits: &wire::Limits,
) -> Result<wire::CharterVersion, wire::Problem> {
    let field_charter = match &value.charter {
        Some(charter) => Some(encode_charter(charter, limits)?),
        None => None,
    };
    let overlay = match &value.overlay {
        Some(overlay) => Some(encode_overlay(overlay, limits)?),
        None => None,
    };
    Ok(wire::CharterVersion { charter: field_charter, overlay })
}
fn decode_charter_version(
    value: wire::CharterVersion,
    limits: &wire::Limits,
) -> Result<settings::CharterVersion, wire::Problem> {
    let wire::CharterVersion { charter, overlay } = value;
    let field_charter = match charter {
        Some(charter) => Some(Box::new(decode_charter(charter, limits)?)),
        None => None,
    };
    let overlay = match overlay {
        Some(overlay) => Some(Box::new(decode_overlay(overlay, limits)?)),
        None => None,
    };
    Ok(settings::CharterVersion { overlay, charter: field_charter })
}
fn encode_proposal(
    value: &settings::Proposal,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<wire::Proposal, wire::Problem> {
    let field_against = value.against;
    let mut list_37 = List::with_capacity(limits.proposal_items);
    for entry in &value.items {
        let element = encode_item(entry, limits)?;
        if list_37.push(element).is_err() {
            return Err(bound());
        }
    }
    let field_items = list_37;
    Ok(wire::Proposal {
        key: value.key,
        models: encode_names(names, limits)?,
        against: field_against,
        items: field_items,
        applications: encode_recorded(&value.applications, limits, limits.proposal_applications)?,
    })
}
fn decode_proposal(
    value: wire::Proposal,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Proposal, wire::Problem> {
    let wire::Proposal { key, models, against, items, applications } = value;
    let field_against = against;
    let mut list_38 = List::with_capacity((items).len());
    for entry in (items).into_boxed() {
        let element = decode_item(entry, limits)?;
        list_38.push(element).expect("decoded list bound");
    }
    let field_items = list_38.into_boxed();
    let mut field_items = field_items;
    let kept = decode_names(models);
    let moved = settings::moves(&kept, names);
    for item in &mut field_items {
        match &mut item.edit {
            settings::Edit::Set(value) => settings::remap(value, &moved, &kept, names),
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    Ok(settings::Proposal {
        key,
        applications: decode_recorded(applications),
        against: field_against,
        items: field_items,
    })
}

fn encode_application(
    value: &application::State,
    limits: &wire::Limits,
) -> Result<wire::ApplicationState, wire::Problem> {
    let follows = match value.follows {
        application::Follows::Default => wire::Follows::Default,
        application::Follows::Changed { revision } => wire::Follows::Changed(wire::FollowsRevision { revision }),
        application::Follows::Removed { revision } => wire::Follows::Removed(wire::FollowsRevision { revision }),
    };
    let mut charters = List::with_capacity(limits.application_state_charters);
    for &charter in &value.charters {
        if charters.push(charter).is_err() {
            return Err(bound());
        }
    }
    Ok(wire::ApplicationState { follows, size: value.size, charters })
}
fn decode_application(value: wire::ApplicationState) -> application::State {
    let wire::ApplicationState { follows, size, charters } = value;
    let follows = match follows {
        wire::Follows::Default => application::Follows::Default,
        wire::Follows::Changed(value) => application::Follows::Changed { revision: value.revision },
        wire::Follows::Removed(value) => application::Follows::Removed { revision: value.revision },
    };
    application::State { follows, size, charters: charters.into_boxed() }
}
fn encode_recorded(
    values: &[application::Recorded],
    _limits: &wire::Limits,
    capacity: u32,
) -> Result<List<wire::Recorded>, wire::Problem> {
    let mut result = List::with_capacity(capacity);
    for value in values {
        let edit = match value.edit {
            application::SizedEdit::Set { size } => wire::SizedEdit::Set(wire::SizedSet { size }),
            application::SizedEdit::Remove => wire::SizedEdit::Remove,
            application::SizedEdit::Reset => wire::SizedEdit::Reset,
        };
        let value = wire::Recorded { kind: value.kind, scope: value.scope, number: value.number, edit };
        if result.push(value).is_err() {
            return Err(bound());
        }
    }
    Ok(result)
}
fn decode_recorded(values: List<wire::Recorded>) -> Box<[application::Recorded]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let wire::Recorded { kind, scope, number, edit } = value;
        let edit = match edit {
            wire::SizedEdit::Set(value) => application::SizedEdit::Set { size: value.size },
            wire::SizedEdit::Remove => application::SizedEdit::Remove,
            wire::SizedEdit::Reset => application::SizedEdit::Reset,
        };
        result.push(application::Recorded { kind, scope, number, edit }).expect("decoded application item count");
    }
    result.into_boxed()
}

#[must_use]
pub fn limits(domain: &core::Limits) -> Option<wire::Limits> {
    let mut limits = wire::CEILINGS;
    limits.worker_setting_name = domain.fleet.worker_name_bytes;
    limits.text_bytes = core::EFFORT_BYTES;
    limits.catalogue_model_shown_some = core::MODEL_NAME_BYTES;
    limits.catalogue_model_description_some = core::MODEL_DESCRIPTION_BYTES;
    limits.model_name = core::MODEL_NAME_BYTES;
    limits.model_name_name = core::MODEL_NAME_BYTES;
    limits.model_name_efforts = core::EFFORTS;
    limits.model_name_efforts_item = core::EFFORT_BYTES;
    limits.run_policy_instructions = domain.run_bytes;
    limits.overlay_instructions = domain.run_bytes;
    limits.overlay_parts = domain.tasks.charter_parts;
    limits.overlay_documents_some = domain.tasks.charter_documents;
    limits.overlay_alternatives_some = domain.settings.alternatives;
    limits.segment_bytes = domain.authority.segment_bytes.max(domain.tasks.hold_bytes);
    limits.last_exact_bytes = domain.authority.segment_bytes;
    limits.last_open_bytes = domain.authority.segment_bytes;
    limits.pattern_segments = domain.authority.segments;
    limits.charter_documents = domain.tasks.charter_documents;
    limits.charter_parts = domain.tasks.charter_parts;
    limits.charter_part_instructions = domain.run_bytes;
    limits.llm_later_subagents = tasks::MODELS;
    limits.document_name = domain.authority.segments;
    limits.document_path = core::DOCUMENT_PATH_BYTES;
    limits.document_heading_some = core::DOCUMENT_HEADING_BYTES;
    limits.delegation_kinds = domain.authority.executors;
    limits.authority_grants = domain.authority.grants;
    limits.authority_note_resources = domain.authority.grants;
    limits.rules_implies = domain.authority.implications;
    limits.rules_requirements = domain.authority.requirements;
    limits.policy_projections = domain.authority.grants;
    limits.policy_roles = domain.authority.roles;
    limits.policy_requirements = domain.authority.requirements;
    limits.configured_policy_permissions =
        u32::try_from(domain.policy_bytes.checked_div(u64::try_from(size_of::<people::PermissionRole>()).ok()?)?)
            .ok()?;
    limits.configured_policy_templates = domain.templates;
    limits.field_rule_name = domain.tasks.result_bytes;
    limits.verdict_fields = u32::try_from(
        u64::from(tasks::VERDICT_FIELDS.min(domain.tasks.result_bytes))
            .checked_mul(wire::FieldRule::encoded_bound(&EMPTY)?.checked_add(4)?)?
            .checked_add(u64::from(domain.tasks.result_bytes))?,
    )
    .ok()?;
    limits.part_path_segments = domain.tasks.hold_segments;
    limits.approval_state = domain.tasks.result_bytes;
    limits.verdict_choices_choices = domain.tasks.contract_choices;
    limits.template_name = domain.documents.text_bytes;
    limits.revision_items = super::people::pieces::piece_bound(
        u32::try_from(settings::revision_bound(domain)?).ok()?,
        size_of::<settings::RevisionItem>(),
        wire::RevisionItem::encoded_bound(&EMPTY)?,
        true,
    )?;
    limits.proposal_items = settings::capacity(domain)?.checked_mul(2)?.min(tasks::SETTINGS_ITEMS);
    limits.revision_applications = tasks::SETTINGS_ITEMS;
    limits.proposal_applications = tasks::SETTINGS_ITEMS;
    if wire::check_limits(&limits).is_err() {
        return None;
    }
    Some(limits)
}

pub fn encode(
    record: &settings::Record,
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<Box<[u8]>, wire::Problem> {
    match record {
        settings::Record::Application { setting, state } => {
            match setting {
                settings::Setting::Application { .. } => {}
                settings::Setting::Charter(_)
                | settings::Setting::Template(_)
                | settings::Setting::Policy(_)
                | settings::Setting::Provider(_)
                | settings::Setting::Worker(_)
                | settings::Setting::Model(_)
                | settings::Setting::Rules
                | settings::Setting::Limits
                | settings::Setting::ProjectPolicy { .. } => return Err(bound()),
            }
            let value = encode_application(state, limits)?;
            value.encode(limits)
        }
        settings::Record::Default { default, .. } => {
            let value = encode_default(default, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Changed { changed, .. } => {
            let value = encode_changed(changed, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Revision { revision, .. } => {
            let value = encode_revision(revision, limits, names)?;
            value.encode(limits)
        }
        settings::Record::Extent { oldest, keys } => {
            let value = wire::Extent { oldest: *oldest, keys: *keys };
            value.encode(limits)
        }
        settings::Record::Charter { value, .. } => encode_version(value, limits),
        settings::Record::Proposal { value, .. } => {
            let value = encode_proposal(value, limits, names)?;
            value.encode(limits)
        }
    }
}
pub fn decode(
    key: &settings::Key,
    bytes: &[u8],
    limits: &wire::Limits,
    names: &[settings::ModelName],
) -> Result<settings::Record, wire::Problem> {
    let mut reader = Reader::new(bytes);
    Ok(match key {
        settings::Key::Default(setting) => settings::Record::Default {
            setting: *setting,
            default: decode_default(wire::Default::decode(limits, &mut reader)?, limits, names)?,
        },
        settings::Key::Changed(setting) => match setting {
            settings::Setting::Application { .. } => settings::Record::Application {
                setting: *setting,
                state: decode_application(wire::ApplicationState::decode(limits, &mut reader)?),
            },
            settings::Setting::Charter(_)
            | settings::Setting::Template(_)
            | settings::Setting::Policy(_)
            | settings::Setting::Provider(_)
            | settings::Setting::Worker(_)
            | settings::Setting::Model(_)
            | settings::Setting::Rules
            | settings::Setting::Limits
            | settings::Setting::ProjectPolicy { .. } => settings::Record::Changed {
                setting: *setting,
                changed: decode_changed(wire::Changed::decode(limits, &mut reader)?, limits, names)?,
            },
        },
        settings::Key::Held(_) | settings::Key::History(_) => {
            let history = match key {
                settings::Key::History(_) => true,
                settings::Key::Held(_) => false,
                settings::Key::Default(_)
                | settings::Key::Changed(_)
                | settings::Key::Charter { .. }
                | settings::Key::Proposal { .. }
                | settings::Key::Extent => unreachable!("revision key"),
            };
            settings::Record::Revision {
                history,
                revision: decode_revision(wire::Revision::decode(limits, &mut reader)?, limits, names)?,
            }
        }
        settings::Key::Extent => {
            let wire::Extent { oldest, keys } = wire::Extent::decode(limits, &mut reader)?;
            settings::Record::Extent { oldest, keys }
        }
        settings::Key::Charter { charter, revision } => {
            settings::Record::Charter { charter: *charter, revision: *revision, value: decode_version(bytes, limits)? }
        }
        settings::Key::Proposal { proposer, proposal } => settings::Record::Proposal {
            proposer: *proposer,
            proposal: *proposal,
            value: decode_proposal(wire::Proposal::decode(limits, &mut reader)?, limits, names)?,
        },
    })
}

// Like the people family's live policy rows, settings remember the catalogue's
// order. Frozen masks and template choices follow names when that order moves.
fn encode_names(names: &[settings::ModelName], _limits: &wire::Limits) -> Result<List<wire::ModelName>, wire::Problem> {
    let mut result = List::with_capacity(jig_domain_tasks::MODELS);
    for name in names {
        if result.push(wire::ModelName { name: name.name.clone(), efforts: effort_names(&name.efforts)? }).is_err() {
            return Err(bound());
        }
    }
    Ok(result)
}
fn decode_names(names: List<wire::ModelName>) -> Box<[settings::ModelName]> {
    let mut kept = List::with_capacity(names.len());
    for name in names.into_boxed() {
        let wire::ModelName { name, efforts } = name;
        kept.push(settings::ModelName { name, efforts: efforts.into_boxed() }).expect("decoded model count");
    }
    kept.into_boxed()
}
fn effort_names(names: &[Box<[u8]>]) -> Result<List<Box<[u8]>>, wire::Problem> {
    let mut result = List::with_capacity(core::EFFORTS);
    for name in names {
        if result.push(name.clone()).is_err() {
            return Err(bound());
        }
    }
    // ModelName's constructor validates each effort's byte length too.
    Ok(result)
}
fn encode_wake_rule(value: tasks::WakeRule, _limits: &wire::Limits) -> wire::WakeRule {
    match value {
        tasks::WakeRule::Never => wire::WakeRule::Never,
        tasks::WakeRule::Immediate => wire::WakeRule::Immediate,
        tasks::WakeRule::Batch { count, age } => wire::WakeRule::Batch(wire::WakeBatch { count, age }),
        tasks::WakeRule::Paced { span } => wire::WakeRule::Paced(wire::WakePace { span }),
    }
}
fn decode_wake_rule(value: wire::WakeRule) -> tasks::WakeRule {
    match value {
        wire::WakeRule::Never => tasks::WakeRule::Never,
        wire::WakeRule::Immediate => tasks::WakeRule::Immediate,
        wire::WakeRule::Batch(value) => {
            let wire::WakeBatch { count, age } = value;
            tasks::WakeRule::Batch { count, age }
        }
        wire::WakeRule::Paced(value) => tasks::WakeRule::Paced { span: value.span },
    }
}
fn encode_wake(value: &tasks::WakePolicy, limits: &wire::Limits) -> wire::WakePolicy {
    wire::WakePolicy {
        words: encode_wake_rule(value.words, limits),
        notices: encode_wake_rule(value.notices, limits),
        news: encode_wake_rule(value.news, limits),
        results: match value.results {
            tasks::ResultsWake::Never => wire::ResultsWake::Never,
            tasks::ResultsWake::Each => wire::ResultsWake::Each,
            tasks::ResultsWake::LastOrFailure => wire::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}
fn decode_wake(value: wire::WakePolicy) -> tasks::WakePolicy {
    let wire::WakePolicy { words, notices, news, results, questions, answers, timers } = value;
    tasks::WakePolicy {
        words: decode_wake_rule(words),
        notices: decode_wake_rule(notices),
        news: decode_wake_rule(news),
        results: match results {
            wire::ResultsWake::Never => tasks::ResultsWake::Never,
            wire::ResultsWake::Each => tasks::ResultsWake::Each,
            wire::ResultsWake::LastOrFailure => tasks::ResultsWake::LastOrFailure,
        },
        questions,
        answers,
        timers,
    }
}

fn encode_catalogue(
    value: &settings::CatalogueModel,
    limits: &wire::Limits,
) -> Result<wire::CatalogueModel, wire::Problem> {
    Ok(wire::CatalogueModel {
        number: value.number,
        model: encode_model(&value.model, limits)?,
        shown: value.shown.clone(),
        description: value.description.clone(),
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        first_read: match value.first_read {
            Some(at) => Some(at.as_nanos()),
            None => None,
        },
        source: match value.source {
            settings::ModelSource::Stock => wire::ModelSource::Stock,
            settings::ModelSource::List => wire::ModelSource::List,
            settings::ModelSource::Person => wire::ModelSource::Person,
        },
    })
}
fn decode_catalogue(value: wire::CatalogueModel) -> settings::CatalogueModel {
    settings::CatalogueModel {
        number: value.number,
        model: decode_model(value.model),
        shown: value.shown,
        description: value.description,
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        first_read: match value.first_read {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        source: match value.source {
            wire::ModelSource::Stock => settings::ModelSource::Stock,
            wire::ModelSource::List => settings::ModelSource::List,
            wire::ModelSource::Person => settings::ModelSource::Person,
        },
    }
}

fn encode_overlay(value: &settings::Overlay, limits: &wire::Limits) -> Result<wire::Overlay, wire::Problem> {
    let documents = match &value.documents {
        Some(items) => {
            let mut documents = List::with_capacity(limits.overlay_documents_some);
            // Hold paths share Segment, but document names follow the authority's byte bound.
            let name_limits = wire::Limits { segment_bytes: limits.last_exact_bytes, ..*limits };
            for document in items {
                let mut name = List::with_capacity(limits.document_name);
                for segment in &document.name.segments {
                    let segment = encode_segment(segment, &name_limits);
                    segment.measure(&name_limits)?;
                    if name.push(segment).is_err() {
                        return Err(bound());
                    }
                }
                let document = wire::Document {
                    connector: document.connector,
                    name,
                    path: document.path.clone(),
                    heading: document.heading.clone(),
                };
                if documents.push(document).is_err() {
                    return Err(bound());
                }
            }
            Some(documents)
        }
        None => None,
    };
    let mut parts = List::with_capacity(limits.overlay_parts);
    for part in &value.parts {
        let part = wire::CharterPart { on: part.on.0, instructions: part.instructions.clone() };
        if parts.push(part).is_err() {
            return Err(bound());
        }
    }
    let alternatives = match &value.alternatives {
        Some(items) => {
            let mut values = List::with_capacity(limits.overlay_alternatives_some);
            for item in items {
                let item = encode_model(item, limits)?;
                if values.push(item).is_err() {
                    return Err(bound());
                }
            }
            Some(values)
        }
        None => None,
    };
    Ok(wire::Overlay {
        rest: value.rest,
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        waiting: value.waiting,
        resume: value.resume,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        call_timeout: value.call_timeout,
        spend: value.spend,
        model: match &value.model {
            Some(item) => Some(encode_model(item, limits)?),
            None => None,
        },
        alternatives,
        documents,
        parts,
        tools: match value.tools {
            Some(item) => Some(item.0),
            None => None,
        },
        hosts: match &value.hosts {
            Some(item) => Some(encode_kinds(item)),
            None => None,
        },
        instructions: value.instructions.clone(),
    })
}

fn decode_overlay(value: wire::Overlay, limits: &wire::Limits) -> Result<settings::Overlay, wire::Problem> {
    let wire::Overlay {
        rest,
        model,
        alternatives,
        inspect,
        modify,
        shell,
        agents,
        tools,
        hosts,
        waiting,
        resume,
        turns,
        time,
        reserve_turns,
        reserve_time,
        call_timeout,
        spend,
        documents,
        instructions,
        parts,
    } = value;
    let decoded = match documents {
        Some(items) => {
            let mut decoded = List::with_capacity(items.len());
            for document in items.into_boxed() {
                let wire::Document { connector, name, path, heading } = document;
                let mut segments = List::with_capacity(name.len());
                for segment in name.into_boxed() {
                    let bytes = decode_segment(segment);
                    if bytes.len() > skein_lib::length::usize_of(limits.last_exact_bytes) {
                        return Err(bound());
                    }
                    segments.push(bytes).expect("decoded document name bound");
                }
                decoded
                    .push(core::Document {
                        connector,
                        name: authority::Name { segments: segments.into_boxed() },
                        path,
                        heading,
                    })
                    .expect("decoded documents bound");
            }
            Some(decoded.into_boxed())
        }
        None => None,
    };
    let mut decoded_parts = List::with_capacity(parts.len());
    for part in parts.into_boxed() {
        let wire::CharterPart { on, instructions } = part;
        let on = tasks::Reasons(on);
        if !on.is_known() || on.is_empty() {
            return Err(bound());
        }
        decoded_parts.push(core::CharterPart { on, instructions }).expect("decoded parts bound");
    }
    let alternatives = match alternatives {
        Some(items) => {
            let mut values = List::with_capacity(items.len());
            for item in items.into_boxed() {
                values.push(decode_model(item)).expect("decoded alternatives");
            }
            Some(values.into_boxed())
        }
        None => None,
    };
    Ok(settings::Overlay {
        rest,
        inspect,
        modify,
        shell,
        agents,
        waiting,
        resume,
        turns,
        time,
        reserve_turns,
        reserve_time,
        call_timeout,
        spend,
        model: match model {
            Some(item) => Some(decode_model(item)),
            None => None,
        },
        alternatives,
        documents: decoded,
        parts: decoded_parts.into_boxed(),
        tools: match tools {
            Some(item) => Some(tasks::Tools(item)),
            None => None,
        },
        hosts: match hosts {
            Some(item) => Some(decode_kinds(item)),
            None => None,
        },
        instructions,
    })
}

// One root per charter-version range (protocol/store/layouts.md, 2).
fn encode_version(value: &settings::CharterVersion, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = encode_charter_version(value, limits)?;
    value.encode(limits)
}
fn decode_version(bytes: &[u8], limits: &wire::Limits) -> Result<settings::CharterVersion, wire::Problem> {
    decode_charter_version(wire::CharterVersion::decode(limits, &mut Reader::new(bytes))?, limits)
}

fn encode_limits(values: &[settings::Override], _limits: &wire::Limits) -> wire::LimitsValue {
    let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
    for value in values {
        overrides
            .push(wire::Override { name: value.name.clone(), value: value.value, reason: value.reason.clone() })
            .expect("override count");
    }
    wire::LimitsValue { overrides }
}

fn decode_limits(value: wire::LimitsValue) -> Box<[settings::Override]> {
    let values = value.overrides;
    let mut overrides = List::with_capacity(values.len());
    for item in values.into_boxed() {
        overrides
            .push(settings::Override { name: item.name, value: item.value, reason: item.reason })
            .expect("override count");
    }
    overrides.into_boxed()
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    #[test]
    fn a_provider_setting_round_trips_in_the_registered_family() {
        use settings::{Endpoint, EndpointAddress, Provider, ProviderSetting, Transport};
        let provider = ProviderSetting {
            provider: Provider {
                number: 4,
                name: b"local".as_slice().into(),
                endpoint: Some(b"local".as_slice().into()),
                direct: None,
                list: None,
                signed_in_plan: false,
                manage_usage: None,
                sign_in: None,
                keys: false,
                programs: Box::new([]),
            },
            endpoints: Box::new([Endpoint {
                number: 5,
                name: b"local".as_slice().into(),
                provider: b"local".as_slice().into(),
                dialect: 3,
                program: None,
                address: EndpointAddress::Socket(::core::net::SocketAddr::from(([127, 0, 0, 1], 8080))),
                transport: Transport::Plaintext,
                headers: Box::new([]),
                connect: skein_lib::Duration::from_millis(10_000),
                handshake: skein_lib::Duration::from_millis(10_000),
            }]),
            withdrawn: true,
        };
        let key = settings::Key::Changed(settings::Setting::Provider(4));
        let record = settings::Record::Changed {
            setting: settings::Setting::Provider(4),
            changed: settings::Changed { revision: 2, value: Some(settings::Value::Provider(Box::new(provider))) },
        };
        let bytes = encode(&record, &wire::CEILINGS, &[]).unwrap();
        assert_eq!(decode(&key, &bytes, &wire::CEILINGS, &[]).unwrap(), record);
    }
}
