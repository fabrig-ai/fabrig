//! Structural adapters between neutral forms and the shared client wire.
use crate::{Limits, view as c};
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, Wall, Writer, bytes};
#[must_use]
pub fn restore_revisions_page(value: &w::RevisionsPage) -> c::RevisionsPage {
    let mut rows = List::with_capacity(value.rows.len());
    for item in &value.rows {
        rows.push(settings_restore_revision_row(item)).expect("same capacity");
    }
    c::RevisionsPage { rows, next: value.next, forgotten: value.forgotten }
}

pub(crate) fn settings_restore_revision_row(value: &w::RevisionRow) -> c::RevisionRow {
    let mut items = List::with_capacity(value.items.len());
    for item in &value.items {
        items.push(settings_restore_revision_item_head(item)).expect("same capacity");
    }
    c::RevisionRow { head: settings_restore_revision_head(&value.head), items, more: value.more }
}

pub(crate) fn settings_restore_revision_head(value: &w::RevisionHead) -> c::RevisionHead {
    c::RevisionHead {
        number: value.number,
        against: value.against,
        at: Wall::from_nanos(value.at),
        by: settings_restore_revision_decider(&value.by),
        proposal: optional_proposal(value.proposal.as_ref()),
        moved: value.moved,
    }
}

pub(crate) fn answers_proposal_key(value: &w::ProposalKey) -> c::ProposalKey {
    c::ProposalKey { proposer: answers_proposer(&value.proposer), number: value.number }
}

pub(crate) fn answers_proposer(value: &w::Proposer) -> c::Proposer {
    match value {
        w::Proposer::Task(row) => c::Proposer::Task(row.value),
        w::Proposer::Party(row) => c::Proposer::Party(row.value),
    }
}

pub(crate) fn settings_restore_revision_decider(value: &w::RevisionDecider) -> c::RevisionDecider {
    match value {
        w::RevisionDecider::FirstStart => c::RevisionDecider::FirstStart,
        w::RevisionDecider::Release => c::RevisionDecider::Release,
        w::RevisionDecider::List(item) => c::RevisionDecider::List { provider: item.provider },
        w::RevisionDecider::Party(item) => c::RevisionDecider::Party(settings_restore_revision_party(item)),
    }
}

pub(crate) fn settings_restore_revision_party(value: &w::RevisionParty) -> c::RevisionParty {
    c::RevisionParty { number: value.number }
}

pub(crate) fn settings_restore_revision_item_head(value: &w::RevisionItemHead) -> c::RevisionItemHead {
    c::RevisionItemHead {
        setting: settings_restore_setting(&value.setting),
        edit: settings_restore_revision_item_edit(&value.edit),
    }
}

pub(crate) fn settings_restore_revision_item_edit(value: &w::RevisionItemEdit) -> c::RevisionItemEdit {
    match value {
        w::RevisionItemEdit::Set => c::RevisionItemEdit::Set,
        w::RevisionItemEdit::Removed => c::RevisionItemEdit::Removed,
        w::RevisionItemEdit::Reset => c::RevisionItemEdit::Reset,
    }
}

pub(crate) fn settings_restore_setting(value: &w::Setting) -> c::Setting {
    match value {
        w::Setting::Charter(item) => c::Setting::Charter(settings_restore_setting_number(item)),
        w::Setting::Template(item) => c::Setting::Template(settings_restore_setting_number(item)),
        w::Setting::ProjectPolicy(item) => c::Setting::ProjectPolicy { project: item.project },
        w::Setting::Policy(item) => c::Setting::Policy(settings_restore_setting_number(item)),
        w::Setting::Application(v) => c::Setting::Application { kind: v.kind, scope: v.scope, number: v.number },
        w::Setting::Rules => c::Setting::Rules,
        w::Setting::Limits => c::Setting::Limits,
        w::Setting::Model(item) => c::Setting::Model(settings_restore_setting_number(item)),
        w::Setting::Worker(item) => c::Setting::Worker(settings_restore_setting_number(item)),
        w::Setting::Provider(item) => c::Setting::Provider(settings_restore_setting_number(item)),
    }
}

pub(crate) fn settings_restore_setting_number(value: &w::SettingNumber) -> c::SettingNumber {
    c::SettingNumber { number: value.number }
}

pub fn restore_revision_answer(value: &w::RevisionAnswer, limits: &Limits) -> Result<c::RevisionAnswer, w::Problem> {
    Ok(match value {
        w::RevisionAnswer::Revision(item) => {
            c::RevisionAnswer::Revision(settings_restore_revision_document(item, limits)?)
        }
        w::RevisionAnswer::Forgotten(item) => c::RevisionAnswer::Forgotten(settings_restore_revision_forgotten(item)),
        w::RevisionAnswer::Unknown => c::RevisionAnswer::Unknown,
    })
}

pub(crate) fn settings_restore_revision_forgotten(value: &w::RevisionForgotten) -> c::RevisionForgotten {
    c::RevisionForgotten { oldest: value.oldest }
}

pub(crate) fn settings_restore_revision_document(
    value: &w::RevisionDocument,
    limits: &Limits,
) -> Result<c::RevisionDocument, w::Problem> {
    Ok(c::RevisionDocument {
        head: settings_restore_revision_head(&value.head),
        items: settings_decode_revision_item_pieces(value.items.as_ref(), limits)?,
    })
}

pub(crate) fn settings_decode_revision_item_pieces(
    values: &[u8],
    limits: &Limits,
) -> Result<List<c::RevisionItem>, w::Problem> {
    let count = settings_item_count(values, limits)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    for _ in 0_u32..=1024 {
        let Some(piece) = (match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(root_piece_problem(reason)),
        }) else {
            return Ok(rows);
        };
        let mut input = Reader::new(&piece);
        let row = w::RevisionItem::decode(&limits.settings, &mut input)?;
        if !input.is_empty() {
            return Err(root_too_large());
        }
        if rows.push(settings_restore_revision_item(&row, limits)?).is_err() {
            return Err(root_too_large());
        }
    }
    Err(root_too_large())
}

pub(crate) fn root_too_large() -> w::Problem {
    w::Problem { path: w::paths::OWNED_RECORD, reason: skein_codec::Reason::Bound }
}

pub(crate) fn settings_restore_revision_item(
    value: &w::RevisionItem,
    limits: &Limits,
) -> Result<c::RevisionItem, w::Problem> {
    Ok(c::RevisionItem {
        setting: settings_restore_setting(&value.setting),
        edit: settings_restore_revision_edit(&value.edit, limits)?,
        replaced: settings_restore_revision_replaced(&value.replaced, limits)?,
    })
}

pub(crate) fn settings_restore_revision_replaced(
    value: &w::RevisionReplaced,
    limits: &Limits,
) -> Result<c::RevisionReplaced, w::Problem> {
    Ok(match value {
        w::RevisionReplaced::Value(item) => c::RevisionReplaced::Value(settings_restore_setting_set(item, limits)?),
        w::RevisionReplaced::TooLarge(item) => c::RevisionReplaced::TooLarge(settings_restore_size(item)),
        w::RevisionReplaced::Application(item) => c::RevisionReplaced::Application { size: item.size },
        w::RevisionReplaced::Default => c::RevisionReplaced::Default,
        w::RevisionReplaced::Removed => c::RevisionReplaced::Removed,
        w::RevisionReplaced::NotOffered => c::RevisionReplaced::NotOffered,
    })
}

pub(crate) fn settings_restore_revision_edit(
    value: &w::RevisionEdit,
    limits: &Limits,
) -> Result<c::RevisionEdit, w::Problem> {
    Ok(match value {
        w::RevisionEdit::Set(item) => c::RevisionEdit::Set(settings_restore_setting_set(item, limits)?),
        w::RevisionEdit::TooLarge(item) => c::RevisionEdit::TooLarge(settings_restore_size(item)),
        w::RevisionEdit::Removed => c::RevisionEdit::Removed,
        w::RevisionEdit::Application(v) => c::RevisionEdit::Application(restore_sized_edit(&v.edit)),
        w::RevisionEdit::Reset => c::RevisionEdit::Reset,
    })
}

pub(crate) fn settings_restore_size(value: &w::Size) -> c::Size {
    c::Size { bytes: value.bytes }
}

pub(crate) fn settings_restore_setting_set(
    value: &w::SettingSet,
    limits: &Limits,
) -> Result<c::SettingSet, w::Problem> {
    Ok(c::SettingSet { value: settings_restore_setting_value(&value.value, limits)? })
}

pub(crate) fn settings_restore_setting_value(
    value: &w::SettingValue,
    limits: &Limits,
) -> Result<c::SettingValue, w::Problem> {
    Ok(match value {
        w::SettingValue::Provider(item) => c::SettingValue::Provider(Box::new(providers::decode(item))),
        w::SettingValue::Overlay(item) => c::SettingValue::Overlay(Box::new(restore_overlay(item)?)),
        w::SettingValue::Charter(item) => c::SettingValue::Charter(Box::new(settings_restore_charter(item)?)),
        w::SettingValue::Template(item) => c::SettingValue::Template(Box::new(documents_me_template(item, limits)?)),
        w::SettingValue::Policy(item) => {
            c::SettingValue::Policy(Box::new(settings_restore_setting_policy(item, limits)?))
        }
        w::SettingValue::Application(v) => c::SettingValue::Application(bytes::copy_of(v.bytes.as_ref())),
        w::SettingValue::Worker(item) => c::SettingValue::Worker(Box::new(c::WorkerSetting {
            number: item.number,
            name: item.name.as_ref().into(),
            places: item.places,
            digest: item.digest,
            withdrawn: item.withdrawn,
        })),
        w::SettingValue::Model(item) => c::SettingValue::Model(Box::new(restore_catalogue(item)?)),
        w::SettingValue::Limits(value) => {
            let mut overrides = List::with_capacity(value.overrides.len());
            for item in &value.overrides {
                overrides
                    .push(c::Override {
                        name: item.name.as_ref().into(),
                        value: item.value,
                        reason: item.reason.as_ref().into(),
                    })
                    .expect("override count");
            }
            c::SettingValue::Limits(overrides.into_boxed())
        }
        w::SettingValue::Rules(item) => c::SettingValue::Rules(Box::new(settings_restore_rules(item, limits)?)),
    })
}

pub(crate) fn settings_restore_rules(value: &w::Rules, limits: &Limits) -> Result<c::Rules, w::Problem> {
    let mut implies = List::with_capacity(value.implies.len());
    for item in &value.implies {
        implies
            .push(c::KindImplication { connector: item.connector, kind: item.kind, implies: item.implies })
            .expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(documents_policy_requirement_fields(item)).expect("same capacity");
    }
    Ok(c::Rules {
        ceiling: documents_owner_authority_fields(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements,
    })
}

pub(crate) fn documents_owner_authority_fields(
    value: &w::Authority,
    limits: &Limits,
) -> Result<c::Authority, w::Problem> {
    let opt0 = optional_wall(value.deadline.as_ref());
    Ok(c::Authority {
        models: model_sets_decode(value.models.as_ref())?,
        tools: value.tools,
        grants: documents_owner_grant_piece(value.grants.as_ref(), limits, limits.authority)?,
        delegation: documents_owner_delegation_fields(&value.delegation, limits)?,
        spend: value.spend,
        deadline: opt0,
        notes: value.notes,
        note_resources: documents_owner_resource_scope_piece(value.note_resources.as_ref(), limits, limits.authority)?,
    })
}

pub(crate) fn documents_owner_resource_scope_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ResourceScope>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(root_piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(skein_lib::len_u64(item.len()))
            .ok_or(root_too_large())?
            .checked_add(4)
            .ok_or(root_too_large())?;
        if sum > limits.ask_bytes {
            return Err(root_too_large());
        }
        let row = w::ResourceScope::decode(&limits.settings, &mut Reader::new(&item))?;
        let row = documents_owner_resource_scope_fields(&row);
        if rows.push(row).is_err() {
            return Err(root_too_large());
        }
    }
    Err(root_too_large())
}

pub(crate) fn documents_owner_resource_scope_fields(value: &w::ResourceScope) -> c::ResourceScope {
    c::ResourceScope { connector: value.connector, pattern: answers_owner_pattern(&value.pattern) }
}

pub(crate) fn answers_owner_pattern(value: &w::Pattern) -> c::Pattern {
    let mut list0 = List::with_capacity(value.segments.len());
    for item in &value.segments {
        list0.push(bytes::copy_of(item)).expect("reserved supplied list");
    }
    c::Pattern { segments: list0, last: answers_owner_pattern_end(&value.last) }
}

pub(crate) fn answers_owner_pattern_end(value: &w::PatternEnd) -> c::PatternEnd {
    match value {
        w::PatternEnd::Exact(row) => c::PatternEnd::Exact(bytes::copy_of(row.value.as_ref())),
        w::PatternEnd::Open(row) => c::PatternEnd::Open(bytes::copy_of(row.value.as_ref())),
    }
}

pub(crate) fn root_piece_problem(reason: skein_codec::Reason) -> w::Problem {
    w::Problem { path: w::paths::OWNED_RECORD, reason }
}

pub(crate) fn documents_owner_delegation_fields(
    value: &w::Delegation,
    limits: &Limits,
) -> Result<c::Delegation, w::Problem> {
    if value.kinds.len() > limits.authority {
        return Err(root_too_large());
    }
    let mut list0 = List::with_capacity(value.kinds.len());
    for item in &value.kinds {
        list0.push(documents_owner_authority_executor_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Delegation { kinds: list0, tasks: value.tasks, depth: value.depth })
}

pub(crate) fn documents_owner_authority_executor_fields(value: &w::AuthorityExecutor) -> c::AuthorityExecutor {
    match value {
        w::AuthorityExecutor::Job(row) => c::AuthorityExecutor::Job(row.value),
        w::AuthorityExecutor::Charter(row) => c::AuthorityExecutor::Charter(row.value),
        w::AuthorityExecutor::Procedure(row) => c::AuthorityExecutor::Procedure(row.value),
        w::AuthorityExecutor::Role(row) => c::AuthorityExecutor::Role(row.value),
    }
}

pub(crate) fn documents_owner_grant_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::Grant>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(root_piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(skein_lib::len_u64(item.len()))
            .ok_or(root_too_large())?
            .checked_add(4)
            .ok_or(root_too_large())?;
        if sum > limits.ask_bytes {
            return Err(root_too_large());
        }
        let row = w::Grant::decode(&limits.settings, &mut Reader::new(&item))?;
        let row = documents_owner_grant_fields(&row);
        if rows.push(row).is_err() {
            return Err(root_too_large());
        }
    }
    Err(root_too_large())
}

pub(crate) fn documents_owner_grant_fields(value: &w::Grant) -> c::Grant {
    c::Grant { connector: value.connector, kind: value.kind, pattern: answers_owner_pattern(&value.pattern) }
}

pub(crate) fn model_sets_decode(value: Option<&w::ModelSet>) -> Result<Option<c::AuthorityModels>, w::Problem> {
    Ok(match value {
        None => None,
        Some(w::ModelSet::All) => Some(c::AuthorityModels::All),
        Some(w::ModelSet::Names(value)) => {
            model_sets_validate(&value.names)?;
            Some(c::AuthorityModels::Named(value.names.clone()))
        }
    })
}

pub(crate) fn model_sets_validate(names: &List<Box<[u8]>>) -> Result<(), w::Problem> {
    let bad = w::Problem { path: w::paths::MODEL_SET_NAMES_NAMES, reason: skein_codec::Reason::Bound };
    if names.len() > c::AUTHORITY_MODELS {
        return Err(bad);
    }
    for (at, name) in names.iter().enumerate() {
        if name.is_empty()
            || name.len() > usize::try_from(c::AUTHORITY_MODEL_BYTES).expect("name bound")
            || !skein_codec::text_is_valid(name)
        {
            return Err(bad);
        }
        for earlier in names.iter().take(at) {
            if earlier == name {
                return Err(bad);
            }
        }
    }
    Ok(())
}

pub(crate) fn documents_policy_requirement_fields(value: &w::PolicyRequirement) -> c::PolicyRequirement {
    c::PolicyRequirement {
        connector: value.connector,
        kind: value.kind,
        pattern: answers_owner_pattern(&value.pattern),
        judge: documents_policy_judge_fields(&value.judge),
        guard: documents_policy_guard_fields(&value.guard),
        must_be_guarded: value.must_be_guarded,
    }
}

pub(crate) fn documents_policy_guard_fields(value: &w::PolicyGuard) -> c::PolicyGuard {
    match value {
        w::PolicyGuard::Guarded => c::PolicyGuard::Guarded,
        w::PolicyGuard::Observed(row) => c::PolicyGuard::Observed { freshness: row.freshness },
    }
}

pub(crate) fn documents_policy_judge_fields(value: &w::PolicyJudge) -> c::PolicyJudge {
    c::PolicyJudge { connector: value.connector, requirement: value.requirement, parameters: value.parameters }
}

pub(crate) fn settings_restore_setting_policy(
    value: &w::SettingPolicy,
    limits: &Limits,
) -> Result<c::SettingPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for item in &value.projections {
        projections.push(documents_owner_grant_fields(item)).expect("same capacity");
    }
    let mut roles = List::with_capacity(value.roles.len());
    for item in &value.roles {
        roles.push(settings_restore_policy_edit_role(item, limits)?).expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(documents_policy_requirement_fields(item)).expect("same capacity");
    }
    let mut permissions = List::with_capacity(value.permissions.len());
    for item in &value.permissions {
        permissions.push(documents_policy_permission_fields(item)).expect("same capacity");
    }
    let mut templates = List::with_capacity(value.templates.len());
    for item in &value.templates {
        templates.push(settings_restore_template_override(item)).expect("same capacity");
    }
    Ok(c::SettingPolicy {
        name: value.name.clone(),
        escalation_role: value.escalation_role,
        ceiling: documents_owner_authority_fields(&value.ceiling, limits)?,
        projections,
        period_spend: value.period_spend,
        roles,
        requirements,
        permissions,
        templates,
    })
}

pub(crate) fn settings_restore_template_override(value: &w::TemplateOverride) -> c::TemplateOverride {
    c::TemplateOverride {
        template: value.template,
        executor: optional_executor(value.executor.as_ref()),
        budget: optional_budget(value.budget.as_ref()),
        deadline: optional_deadline(value.deadline.as_ref()),
        priority: value.priority,
    }
}

pub(crate) fn operations_restore_deadline(value: &w::TemplateDeadline) -> c::TemplateDeadline {
    match value {
        w::TemplateDeadline::None => c::TemplateDeadline::None,
        w::TemplateDeadline::Span(v) => c::TemplateDeadline::Span(v.value),
        w::TemplateDeadline::Given => c::TemplateDeadline::Given,
    }
}

pub(crate) fn operations_restore_budget(value: &w::TemplateBudget) -> c::TemplateBudget {
    match value {
        w::TemplateBudget::Amount(v) => c::TemplateBudget::Amount(v.value),
        w::TemplateBudget::AsAllowed(v) => c::TemplateBudget::AsAllowed { cap: v.cap },
        w::TemplateBudget::Given => c::TemplateBudget::Given,
    }
}

pub(crate) fn operations_restore_executor(value: &w::TemplateExecutor) -> c::TemplateExecutor {
    match value {
        w::TemplateExecutor::Charter(v) => c::TemplateExecutor::Charter(v.value),
        w::TemplateExecutor::Chosen => c::TemplateExecutor::Chosen,
    }
}

pub(crate) fn documents_policy_permission_fields(value: &w::PolicyPermission) -> c::PolicyPermission {
    c::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role }
}

pub(crate) fn settings_restore_policy_edit_role(
    value: &w::PolicyEditRole,
    limits: &Limits,
) -> Result<c::PolicyEditRole, w::Problem> {
    Ok(c::PolicyEditRole {
        number: value.number,
        authority: documents_owner_authority_fields(&value.authority, limits)?,
        period_spend: value.period_spend,
        requests: value.requests,
        decides: value.decides,
    })
}

pub(crate) fn documents_me_template(value: &w::Template, limits: &Limits) -> Result<c::Template, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for resource in &value.resources {
        resources.push(c::Symbol { connector: resource.connector, symbol: resource.symbol }).expect("same capacity");
    }
    Ok(c::Template {
        name: value.name.clone(),
        resources,
        number: value.number,
        shown: facts_document_text(&value.shown, limits)?,
        executor: operations_restore_executor(&value.executor),
        contract: documents_task_contract_document_fields(&value.contract, limits)?,
        shallower: value.shallower,
        budget: operations_restore_budget(&value.budget),
        deadline: operations_restore_deadline(&value.deadline),
        tracked: value.tracked,
        flags: c::TaskFlags {
            ended_by: match &value.flags.ended_by {
                w::EndedBy::Result => c::EndedBy::Result,
                w::EndedBy::Requester => c::EndedBy::Requester,
                w::EndedBy::Standing => c::EndedBy::Standing,
                w::EndedBy::Closer(closer) => {
                    c::EndedBy::Closer(c::Closer { connector: closer.connector, code: closer.code })
                }
            },
            conversation: value.flags.conversation,
            listed: value.flags.listed,
            retention: match &value.flags.retention {
                w::Retention::Kept => c::Retention::Kept,
                w::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
            },
        },
        label: value.label,
        grace: value.grace,
        llm: llm_choice(value.llm.as_ref(), limits)?,
    })
}

pub(crate) fn llm_choice(value: Option<&w::LlmChoice>, limits: &Limits) -> Result<Option<c::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(value.subagents.len());
    for item in &value.subagents {
        subagents.push(llm_model_effort(item, limits)?).expect("same capacity");
    }
    let later = match &value.later {
        Some(later) => {
            let mut subagents = List::with_capacity(later.subagents.len());
            for item in &later.subagents {
                subagents.push(llm_model_effort(item, limits)?).expect("same capacity");
            }
            Some(c::LlmLater { main: llm_model_effort(&later.main, limits)?, subagents })
        }
        None => None,
    };
    Ok(Some(c::LlmChoice { account: value.account, main: llm_model_effort(&value.main, limits)?, subagents, later }))
}

pub(crate) fn llm_model_effort(value: &w::ModelEffort, limits: &Limits) -> Result<c::ModelEffort, w::Problem> {
    Ok(c::ModelEffort {
        model: llm_name(value.model.as_ref(), limits.settings.model_effort_model)?,
        effort: match &value.effort {
            Some(effort) => Some(llm_name(effort, limits.settings.model_effort_effort_some)?),
            None => None,
        },
    })
}

pub(crate) fn llm_name(value: &[u8], limit: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limit) {
        Ok(text) => Ok(text),
        Err(_) => Err(root_too_large()),
    }
}

pub(crate) fn documents_task_contract_document_fields(
    value: &w::TaskContractDocument,
    limits: &Limits,
) -> Result<c::TaskContractDocument, w::Problem> {
    Ok(match value {
        w::TaskContractDocument::Job(row) => c::TaskContractDocument::Job { bytes: row.bytes },
        w::TaskContractDocument::Approval(row) => c::TaskContractDocument::Approval {
            state: documents_raw(row.state.as_ref(), limits)?,
            part: facts_owner_part(&row.part),
        },
        w::TaskContractDocument::Report(row) => c::TaskContractDocument::Report { words: row.words },
        w::TaskContractDocument::Verdict(row) => c::TaskContractDocument::Verdict {
            choices: documents_verdict_choice_piece(row.choices.as_ref(), limits, limits.rows)?,
            cut: row.cut,
        },
        w::TaskContractDocument::Change(row) => {
            c::TaskContractDocument::Change { connector: row.connector, kind: row.kind, words: row.words }
        }
    })
}

pub(crate) fn documents_verdict_choice_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::VerdictChoice>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(root_piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(skein_lib::len_u64(item.len()))
            .ok_or(root_too_large())?
            .checked_add(4)
            .ok_or(root_too_large())?;
        if sum > limits.ask_bytes {
            return Err(root_too_large());
        }
        let row = w::VerdictChoice::decode(&limits.settings, &mut Reader::new(&item))?;
        let row = documents_verdict_choice_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(root_too_large());
        }
    }
    Err(root_too_large())
}

pub(crate) fn documents_verdict_choice_fields(
    value: &w::VerdictChoice,
    limits: &Limits,
) -> Result<c::VerdictChoice, w::Problem> {
    if value.fields.len() > limits.fields {
        return Err(root_too_large());
    }
    let mut list0 = List::with_capacity(value.fields.len());
    for item in &value.fields {
        list0.push(documents_field_rule_fields(item, limits)?).expect("reserved supplied list");
    }
    Ok(c::VerdictChoice { code: value.code, words: value.words, followups: value.followups, fields: list0 })
}

pub(crate) fn documents_field_rule_fields(value: &w::FieldRule, limits: &Limits) -> Result<c::FieldRule, w::Problem> {
    Ok(c::FieldRule { name: documents_raw(value.name.as_ref(), limits)?, max: value.max })
}

pub(crate) fn documents_raw(value: &[u8], limits: &Limits) -> Result<Box<[u8]>, w::Problem> {
    if skein_lib::len_u64(value.len()) > u64::from(limits.text) {
        return Err(root_too_large());
    }
    Ok(bytes::copy_of(value))
}

pub(crate) fn facts_owner_part(value: &w::Part) -> c::Part {
    c::Part {
        connector: value.connector,
        kind: facts_owner_part_kind(&value.kind),
        key: facts_owner_part_key(&value.key),
    }
}

pub(crate) fn facts_owner_part_key(value: &w::PartKey) -> c::PartKey {
    match value {
        w::PartKey::Path(row) => {
            let mut list0 = List::with_capacity(row.value.len());
            for item in &row.value {
                list0.push(bytes::copy_of(item)).expect("reserved supplied list");
            }
            c::PartKey::Path(list0)
        }
        w::PartKey::Number(row) => c::PartKey::Number(row.value),
    }
}

pub(crate) fn facts_owner_part_kind(value: &w::PartKind) -> c::PartKind {
    match value {
        w::PartKind::Resource => c::PartKind::Resource,
        w::PartKind::Pool => c::PartKind::Pool,
        w::PartKind::Result => c::PartKind::Result,
        w::PartKind::Procedure => c::PartKind::Procedure,
        w::PartKind::Effect => c::PartKind::Effect,
        w::PartKind::Outbox => c::PartKind::Outbox,
        w::PartKind::Hold => c::PartKind::Hold,
    }
}

pub(crate) fn facts_document_text(value: &w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    Ok(c::DocumentText {
        bytes: match c::Text::new(bytes::copy_of(value.bytes.as_ref()), limits.text) {
            Ok(value) => value,
            Err(_) => return Err(root_too_large()),
        },
        cut: value.cut,
    })
}

pub fn restore_overlay(value: &w::Overlay) -> Result<c::Overlay, w::Problem> {
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(c::CharterPart {
                on: item.on,
                instructions: settings_text(item.instructions.as_ref(), w::CEILINGS.charter_part_instructions)?,
            })
            .expect("same count");
    }
    Ok(c::Overlay {
        charters: List::with_capacity(0),
        parts,
        instructions: settings_text(value.instructions.as_ref(), w::CEILINGS.charter_instructions)?,
        rest: value.rest,
        model: match &value.model {
            Some(v) => Some(settings_restore_setting_model(v)?),
            None => None,
        },
        alternatives: match &value.alternatives {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(settings_restore_setting_model(item)?).expect("same count");
                }
                Some(out)
            }
            None => None,
        },
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        tools: value.tools,
        hosts: match &value.hosts {
            Some(v) => Some(settings_restore_setting_hosts(v)),
            None => None,
        },
        waiting: value.waiting,
        resume: value.resume,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        call_timeout: value.call_timeout,
        spend: value.spend,
        documents: match &value.documents {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(c::CharterDocument {
                        connector: item.connector,
                        name: item.name.clone(),
                        path: item.path.as_ref().into(),
                        heading: item.heading.clone(),
                    })
                    .expect("same count");
                }
                Some(out)
            }
            None => None,
        },
    })
}
#[must_use]
pub fn wire_overlay(value: &c::Overlay) -> w::Overlay {
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            .expect("same count");
    }
    w::Overlay {
        parts,
        instructions: bytes::copy_of(value.instructions.bytes()),
        rest: value.rest,
        model: match &value.model {
            Some(v) => Some(settings_wire_setting_model(v)),
            None => None,
        },
        alternatives: match &value.alternatives {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(settings_wire_setting_model(item)).expect("same count");
                }
                Some(out)
            }
            None => None,
        },
        inspect: value.inspect,
        modify: value.modify,
        shell: value.shell,
        agents: value.agents,
        tools: value.tools,
        hosts: match &value.hosts {
            Some(v) => Some(settings_wire_setting_hosts(v)),
            None => None,
        },
        waiting: value.waiting,
        resume: value.resume,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        call_timeout: value.call_timeout,
        spend: value.spend,
        documents: match &value.documents {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(w::CharterDocument {
                        connector: item.connector,
                        name: item.name.clone(),
                        path: item.path.clone(),
                        heading: item.heading.clone(),
                    })
                    .expect("same count");
                }
                Some(out)
            }
            None => None,
        },
    }
}
pub(crate) fn settings_restore_charter(value: &w::Charter) -> Result<c::Charter, w::Problem> {
    let mut alternatives = List::with_capacity(value.alternatives.len());
    for item in &value.alternatives {
        alternatives.push(settings_restore_setting_model(item)?).expect("same capacity");
    }
    let mut documents = List::with_capacity(value.documents.len());
    for item in &value.documents {
        documents
            .push(c::CharterDocument {
                connector: item.connector,
                name: item.name.clone(),
                path: item.path.as_ref().into(),
                heading: item.heading.clone(),
            })
            .expect("same capacity");
    }
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(c::CharterPart {
                on: item.on,
                instructions: settings_text(item.instructions.as_ref(), w::CEILINGS.charter_part_instructions)?,
            })
            .expect("same capacity");
    }
    Ok(c::Charter {
        name: value.name.clone(),
        parts,
        rest: value.rest,
        documents,
        switches: [value.resume, value.inspect, value.modify, value.shell, value.agents],
        number: value.number,
        instructions: settings_text(value.instructions.as_ref(), 1_114_240)?,
        waiting: value.waiting,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        model: settings_restore_setting_model(&value.model)?,
        alternatives,
        call_timeout: value.call_timeout,
        tools: value.tools,
        hosts: settings_restore_setting_hosts(&value.hosts),
        spend: value.spend,
        wake: settings_restore_wake(&value.wake),
    })
}

pub(crate) fn settings_restore_setting_hosts(value: &w::SettingHosts) -> c::SettingHosts {
    match value {
        w::SettingHosts::Workers => c::SettingHosts::Workers,
        w::SettingHosts::Engine => c::SettingHosts::Engine,
        w::SettingHosts::Both => c::SettingHosts::Both,
    }
}

pub(crate) fn settings_restore_setting_model(value: &w::SettingModel) -> Result<c::SettingModel, w::Problem> {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(settings_text(item, 32)?).expect("same capacity");
    }
    let mut speeds = List::with_capacity(value.speeds.len());
    for speed in &value.speeds {
        let mut also = List::with_capacity(speed.also.len());
        for name in &speed.also {
            also.push(settings_text(name, 32)?).expect("same capacity");
        }
        speeds
            .push(c::SettingSpeed {
                name: settings_text(&speed.name, 32)?,
                also,
                input_price: speed.input_price,
                cached_price: speed.cached_price,
                cache_write_price: speed.cache_write_price,
                output_price: speed.output_price,
            })
            .expect("same capacity");
    }
    Ok(c::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: match &value.program {
            Some(item) => Some(settings_text(item, 32)?),
            None => None,
        },
        name: settings_text(value.name.as_ref(), 4096)?,
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: value.cache_write_price,
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts,
        effort: match &value.effort {
            Some(item) => Some(settings_text(item, 32)?),
            None => None,
        },
        speeds,
        speed: match &value.speed {
            Some(name) => Some(settings_text(name, 32)?),
            None => None,
        },
    })
}

pub(crate) fn settings_text(value: &[u8], bound: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), bound) {
        Ok(value) => Ok(value),
        Err(_) => Err(root_too_large()),
    }
}

pub(crate) fn settings_item_count(values: &[u8], limits: &Limits) -> Result<u32, w::Problem> {
    if skein_lib::len_u64(values.len()) > limits.ask_bytes {
        return Err(root_too_large());
    }
    let mut reader = Reader::new(values);
    for count in 0_u32..=1024 {
        let piece = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(root_piece_problem(reason)),
        };
        if piece.is_none() {
            return Ok(count);
        }
    }
    Err(root_too_large())
}

pub fn restore_setting_document(value: &w::SettingDocument, limits: &Limits) -> Result<c::SettingDocument, w::Problem> {
    Ok(c::SettingDocument {
        overlay: match &value.overlay {
            Some(v) => Some(Box::new(restore_overlay(v)?)),
            None => None,
        },
        setting: settings_restore_setting(&value.setting),
        revision: value.revision,
        value: match &value.value {
            Some(v) => Some(settings_restore_setting_value(v, limits)?),
            None => None,
        },
        application: match &value.application {
            Some(v) => Some(restore_application_state(v)),
            None => None,
        },
        proposed: match &value.proposed {
            Some(v) => Some(restore_sized_edit(v)),
            None => None,
        },
    })
}

pub fn restore_settings_document(
    value: &w::SettingsDocument,
    limits: &Limits,
) -> Result<c::SettingsDocument, w::Problem> {
    let mut settings = List::with_capacity(value.settings.len());
    for item in &value.settings {
        settings.push(settings_restore_setting_line(item, limits)?).expect("same capacity");
    }
    let mut held = List::with_capacity(value.held.len());
    for item in &value.held {
        held.push(settings_restore_held_revision(item)).expect("same capacity");
    }
    let mut pending = List::with_capacity(value.pending.len());
    for item in &value.pending {
        pending.push(settings_restore_settings_proposal(item)).expect("same capacity");
    }
    Ok(c::SettingsDocument { newest: value.newest, settings, held, pending })
}

pub(crate) fn settings_restore_settings_proposal(value: &w::SettingsProposal) -> c::SettingsProposal {
    c::SettingsProposal { proposer: answers_proposer(&value.proposer), number: value.number }
}

pub(crate) fn settings_restore_held_revision(value: &w::HeldRevision) -> c::HeldRevision {
    c::HeldRevision { head: settings_restore_revision_head(&value.head), holders: value.holders }
}

pub(crate) fn settings_restore_setting_line(
    value: &w::SettingLine,
    limits: &Limits,
) -> Result<c::SettingLine, w::Problem> {
    Ok(c::SettingLine {
        name: value.name.clone(),
        setting: settings_restore_setting(&value.setting),
        shown: match &value.shown {
            Some(item) => Some(facts_document_text(item, limits)?),
            None => None,
        },
        follows: settings_restore_setting_follows(&value.follows),
        last: value.last,
        application: match &value.application {
            Some(v) => Some(restore_application_state(v)),
            None => None,
        },
    })
}

pub(crate) fn settings_restore_setting_follows(value: &w::SettingFollows) -> c::SettingFollows {
    match value {
        w::SettingFollows::Overlay => c::SettingFollows::Overlay,
        w::SettingFollows::Default => c::SettingFollows::Default,
        w::SettingFollows::Change => c::SettingFollows::Change,
    }
}

pub fn wire_change_settings(value: &c::ChangeSettings, limits: &w::Limits) -> Result<w::ChangeSettings, w::Problem> {
    Ok(w::ChangeSettings {
        against: value.against,
        items: settings_encode_setting_item_pieces(&value.items, limits)?,
        running: settings_wire_running(&value.running),
    })
}

pub(crate) fn settings_wire_running(value: &c::Running) -> w::Running {
    match value {
        c::Running::Stays => w::Running::Stays,
        c::Running::Moves => w::Running::Moves,
    }
}

pub(crate) fn settings_encode_setting_item_pieces(
    values: &List<c::SettingItem>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 1024 {
        return Err(root_too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = settings_wire_setting_item(value, limits)?;
        size = size.checked_add(row.measure(limits)?).ok_or(root_too_large())?.checked_add(4).ok_or(root_too_large())?;
        if size > limits.change_settings_items {
            return Err(root_too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(usize::try_from(size).expect("wire size fits"));
    for row in rows.into_boxed() {
        writer.put(&row.measure(limits)?.to_be_bytes()).expect("measured");
        row.write(limits, &mut writer).expect("measured");
    }
    Ok(writer.finish())
}

pub(crate) fn settings_wire_setting_item(
    value: &c::SettingItem,
    limits: &w::Limits,
) -> Result<w::SettingItem, w::Problem> {
    Ok(w::SettingItem {
        setting: settings_wire_setting(value.setting),
        edit: settings_wire_setting_edit(&value.edit, limits)?,
    })
}

pub(crate) fn settings_wire_setting_edit(
    value: &c::SettingEdit,
    limits: &w::Limits,
) -> Result<w::SettingEdit, w::Problem> {
    Ok(match value {
        c::SettingEdit::Set(item) => w::SettingEdit::Set(settings_wire_setting_set(item, limits)?),
        c::SettingEdit::Removed => w::SettingEdit::Removed,
        c::SettingEdit::Reset => w::SettingEdit::Reset,
    })
}

pub(crate) fn settings_wire_setting_set(
    value: &c::SettingSet,
    limits: &w::Limits,
) -> Result<w::SettingSet, w::Problem> {
    Ok(w::SettingSet { value: settings_wire_setting_value(&value.value, limits)? })
}

pub(crate) fn settings_wire_setting_value(
    value: &c::SettingValue,
    limits: &w::Limits,
) -> Result<w::SettingValue, w::Problem> {
    Ok(match value {
        c::SettingValue::Overlay(item) => w::SettingValue::Overlay(Box::new(wire_overlay(item))),
        c::SettingValue::Charter(item) => w::SettingValue::Charter(Box::new(settings_wire_charter(item))),
        c::SettingValue::Template(item) => w::SettingValue::Template(Box::new(settings_wire_template(item, limits)?)),
        c::SettingValue::Policy(item) => w::SettingValue::Policy(Box::new(settings_wire_setting_policy(item, limits)?)),
        c::SettingValue::Application(v) => w::SettingValue::Application(w::ApplicationValue { bytes: v.clone() }),
        c::SettingValue::Provider(item) => w::SettingValue::Provider(Box::new(providers::encode(item, limits)?)),
        c::SettingValue::Worker(item) => w::SettingValue::Worker(Box::new(w::WorkerSetting {
            number: item.number,
            name: item.name.clone(),
            places: item.places,
            digest: item.digest,
            withdrawn: item.withdrawn,
        })),
        c::SettingValue::Model(item) => w::SettingValue::Model(Box::new(wire_catalogue(item))),
        c::SettingValue::Limits(values) => {
            let mut overrides = List::with_capacity(u32::try_from(values.len()).expect("bounded overrides"));
            for item in values {
                overrides
                    .push(w::Override { name: item.name.clone(), value: item.value, reason: item.reason.clone() })
                    .expect("override count");
            }
            w::SettingValue::Limits(w::LimitsValue { overrides })
        }
        c::SettingValue::Rules(item) => w::SettingValue::Rules(Box::new(settings_wire_rules(item, limits)?)),
    })
}

pub(crate) fn settings_wire_rules(value: &c::Rules, limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    let mut implies = List::with_capacity(value.implies.len());
    for item in &value.implies {
        implies
            .push(w::KindImplication { connector: item.connector, kind: item.kind, implies: item.implies })
            .expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(asks_wire_policy_requirement(item)).expect("same capacity");
    }
    Ok(w::Rules {
        ceiling: asks_wire_owner_authority(&value.ceiling, limits)?,
        period_spend: value.period_spend,
        minimum_run_spend: value.minimum_run_spend,
        maximum_run_spend: value.maximum_run_spend,
        implies,
        requirements,
    })
}

pub(crate) fn asks_wire_owner_authority(value: &c::Authority, limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    let mut list0 = List::with_capacity(value.grants.len());
    for item in &value.grants {
        list0.push(asks_wire_owner_grant(item)).expect("reserved supplied list");
    }
    let opt1 = optional_nanos(value.deadline);
    let mut list2 = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        list2.push(asks_wire_owner_resource_scope(item)).expect("reserved supplied list");
    }
    Ok(w::Authority {
        models: model_sets_encode(value.models.as_ref())?,
        tools: value.tools,
        grants: asks_wire_owner_grant_piece(&list0, limits)?,
        delegation: asks_wire_owner_delegation(&value.delegation),
        spend: value.spend,
        deadline: opt1,
        notes: value.notes,
        note_resources: asks_wire_owner_resource_scope_piece(&list2, limits)?,
    })
}

pub(crate) fn asks_wire_owner_resource_scope_piece(
    rows: &List<w::ResourceScope>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(limits)?).ok_or(root_too_large())?;
        length = length.checked_add(4).ok_or(root_too_large())?;
    }
    if length > limits.owned_record {
        return Err(root_too_large());
    }
    let mut writer = Writer::new(skein_lib::usize_of(length));
    for row in rows {
        let mut item = Writer::new(skein_lib::usize_of(row.measure(limits)?));
        row.write(limits, &mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

pub(crate) fn asks_wire_owner_delegation(value: &c::Delegation) -> w::Delegation {
    let mut list0 = List::with_capacity(value.kinds.len());
    for item in &value.kinds {
        list0.push(asks_wire_owner_authority_executor(*item)).expect("reserved supplied list");
    }
    w::Delegation { kinds: list0, tasks: value.tasks, depth: value.depth }
}

pub(crate) fn asks_wire_owner_authority_executor(value: c::AuthorityExecutor) -> w::AuthorityExecutor {
    match value {
        c::AuthorityExecutor::Job(value) => w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value }),
        c::AuthorityExecutor::Charter(value) => w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value }),
        c::AuthorityExecutor::Procedure(value) => {
            w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value })
        }
        c::AuthorityExecutor::Role(value) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value }),
    }
}

pub(crate) fn asks_wire_owner_grant_piece(rows: &List<w::Grant>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(limits)?).ok_or(root_too_large())?;
        length = length.checked_add(4).ok_or(root_too_large())?;
    }
    if length > limits.owned_record {
        return Err(root_too_large());
    }
    let mut writer = Writer::new(skein_lib::usize_of(length));
    for row in rows {
        let mut item = Writer::new(skein_lib::usize_of(row.measure(limits)?));
        row.write(limits, &mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

pub(crate) fn model_sets_encode(value: Option<&c::AuthorityModels>) -> Result<Option<w::ModelSet>, w::Problem> {
    Ok(match value {
        None => None,
        Some(c::AuthorityModels::All) => Some(w::ModelSet::All),
        Some(c::AuthorityModels::Named(names)) => {
            model_sets_validate(names)?;
            Some(w::ModelSet::Names(w::ModelSetNames { names: names.clone() }))
        }
    })
}

pub(crate) fn asks_wire_owner_resource_scope(value: &c::ResourceScope) -> w::ResourceScope {
    w::ResourceScope { connector: value.connector, pattern: asks_wire_owner_pattern(&value.pattern) }
}

pub(crate) fn asks_wire_owner_pattern(value: &c::Pattern) -> w::Pattern {
    let mut list0 = List::with_capacity(value.segments.len());
    for item in &value.segments {
        list0.push(bytes::copy_of(item)).expect("reserved supplied list");
    }
    w::Pattern { segments: list0, last: asks_wire_owner_pattern_end(&value.last) }
}

pub(crate) fn asks_wire_owner_pattern_end(value: &c::PatternEnd) -> w::PatternEnd {
    match value {
        c::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        c::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    }
}

pub(crate) fn asks_wire_owner_grant(value: &c::Grant) -> w::Grant {
    w::Grant { connector: value.connector, kind: value.kind, pattern: asks_wire_owner_pattern(&value.pattern) }
}

pub(crate) fn asks_wire_policy_requirement(value: &c::PolicyRequirement) -> w::PolicyRequirement {
    w::PolicyRequirement {
        connector: value.connector,
        kind: value.kind,
        pattern: asks_wire_owner_pattern(&value.pattern),
        judge: asks_wire_policy_judge(value.judge),
        guard: asks_wire_policy_guard(&value.guard),
        must_be_guarded: value.must_be_guarded,
    }
}

pub(crate) fn asks_wire_policy_guard(value: &c::PolicyGuard) -> w::PolicyGuard {
    match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => {
            w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness: *freshness })
        }
    }
}

pub(crate) fn asks_wire_policy_judge(value: c::PolicyJudge) -> w::PolicyJudge {
    w::PolicyJudge { connector: value.connector, requirement: value.requirement, parameters: value.parameters }
}

pub(crate) fn settings_wire_setting_policy(
    value: &c::SettingPolicy,
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for item in &value.projections {
        projections.push(asks_wire_owner_grant(item)).expect("same capacity");
    }
    let mut roles = List::with_capacity(value.roles.len());
    for item in &value.roles {
        roles.push(asks_wire_policy_edit_role(item, limits)?).expect("same capacity");
    }
    let mut requirements = List::with_capacity(value.requirements.len());
    for item in &value.requirements {
        requirements.push(asks_wire_policy_requirement(item)).expect("same capacity");
    }
    let mut permissions = List::with_capacity(value.permissions.len());
    for item in &value.permissions {
        permissions.push(asks_wire_policy_permission(*item)).expect("same capacity");
    }
    let mut templates = List::with_capacity(value.templates.len());
    for item in &value.templates {
        templates.push(settings_wire_template_override(item)).expect("same capacity");
    }
    Ok(w::SettingPolicy {
        name: value.name.clone(),
        escalation_role: value.escalation_role,
        ceiling: asks_wire_owner_authority(&value.ceiling, limits)?,
        projections,
        period_spend: value.period_spend,
        roles,
        requirements,
        permissions,
        templates,
    })
}

pub(crate) fn settings_wire_template_override(value: &c::TemplateOverride) -> w::TemplateOverride {
    w::TemplateOverride {
        template: value.template,
        executor: match value.executor {
            Some(value) => Some(operations_executor(value)),
            None => None,
        },
        budget: match value.budget {
            Some(value) => Some(operations_budget(value)),
            None => None,
        },
        deadline: match value.deadline {
            Some(value) => Some(operations_deadline(value)),
            None => None,
        },
        priority: value.priority,
    }
}

pub(crate) fn operations_deadline(value: c::TemplateDeadline) -> w::TemplateDeadline {
    match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => w::TemplateDeadline::Span(w::TemplateSpan { value }),
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    }
}

pub(crate) fn operations_budget(value: c::TemplateBudget) -> w::TemplateBudget {
    match value {
        c::TemplateBudget::Amount(value) => w::TemplateBudget::Amount(w::TemplateAmount { value }),
        c::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap }),
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    }
}

pub(crate) fn operations_executor(value: c::TemplateExecutor) -> w::TemplateExecutor {
    match value {
        c::TemplateExecutor::Charter(value) => w::TemplateExecutor::Charter(w::TemplateCharter { value }),
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    }
}

pub(crate) fn asks_wire_policy_permission(value: c::PolicyPermission) -> w::PolicyPermission {
    w::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role }
}

pub(crate) fn asks_wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    Ok(w::PolicyEditRole {
        number: value.number,
        authority: asks_wire_owner_authority(&value.authority, limits)?,
        period_spend: value.period_spend,
        requests: value.requests,
        decides: value.decides,
    })
}

pub(crate) fn settings_wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for resource in &value.resources {
        resources.push(w::Symbol { connector: resource.connector, symbol: resource.symbol }).expect("same capacity");
    }
    Ok(w::Template {
        name: value.name.clone(),
        resources,
        number: value.number,
        shown: settings_wire_document_text(&value.shown),
        executor: operations_executor(value.executor),
        contract: settings_wire_task_contract(&value.contract, limits)?,
        shallower: value.shallower,
        budget: operations_budget(value.budget),
        deadline: operations_deadline(value.deadline),
        tracked: value.tracked,
        flags: w::TaskFlags {
            ended_by: match value.flags.ended_by {
                c::EndedBy::Result => w::EndedBy::Result,
                c::EndedBy::Requester => w::EndedBy::Requester,
                c::EndedBy::Standing => w::EndedBy::Standing,
                c::EndedBy::Closer(closer) => {
                    w::EndedBy::Closer(w::Closer { connector: closer.connector, code: closer.code })
                }
            },
            conversation: value.flags.conversation,
            listed: value.flags.listed,
            retention: match value.flags.retention {
                c::Retention::Kept => w::Retention::Kept,
                c::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
            },
        },
        label: value.label,
        grace: value.grace,
        llm: match &value.llm {
            Some(choice) => Some(llm_wire_choice(choice)),
            None => None,
        },
    })
}

pub(crate) fn llm_wire_choice(value: &c::LlmChoice) -> w::LlmChoice {
    let mut subagents = List::with_capacity(value.subagents.len());
    for item in &value.subagents {
        subagents.push(llm_wire_model_effort(item)).expect("same capacity");
    }
    w::LlmChoice {
        later: match &value.later {
            Some(later) => {
                let mut subagents = List::with_capacity(later.subagents.len());
                for item in &later.subagents {
                    subagents.push(llm_wire_model_effort(item)).expect("same capacity");
                }
                Some(w::LlmLater { main: llm_wire_model_effort(&later.main), subagents })
            }
            None => None,
        },
        account: value.account,
        main: llm_wire_model_effort(&value.main),
        subagents,
    }
}

pub(crate) fn llm_wire_model_effort(value: &c::ModelEffort) -> w::ModelEffort {
    w::ModelEffort { model: bytes::copy_of(value.model.bytes()), effort: optional_text(value.effort.as_ref()) }
}

pub(crate) fn settings_wire_task_contract(
    value: &c::TaskContractDocument,
    limits: &w::Limits,
) -> Result<w::TaskContractDocument, w::Problem> {
    Ok(match value {
        c::TaskContractDocument::Job { bytes } => {
            w::TaskContractDocument::Job(w::TaskContractDocumentJob { bytes: *bytes })
        }
        c::TaskContractDocument::Approval { state, part } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval {
                state: bytes::copy_of(state),
                part: settings_wire_owner_part(part),
            })
        }
        c::TaskContractDocument::Report { words } => {
            w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: *words })
        }
        c::TaskContractDocument::Change { connector, kind, words } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange {
                connector: *connector,
                kind: *kind,
                words: *words,
            })
        }
        c::TaskContractDocument::Verdict { choices, cut } => {
            let mut rows = List::with_capacity(choices.len());
            let mut size = 0_u32;
            for choice in choices {
                let mut fields = List::with_capacity(choice.fields.len());
                for field in &choice.fields {
                    fields
                        .push(w::FieldRule { name: bytes::copy_of(&field.name), max: field.max })
                        .expect("same capacity");
                }
                let row =
                    w::VerdictChoice { code: choice.code, words: choice.words, followups: choice.followups, fields };
                size = size
                    .checked_add(row.measure(limits)?)
                    .ok_or(root_too_large())?
                    .checked_add(4)
                    .ok_or(root_too_large())?;
                rows.push(row).expect("same capacity");
            }
            if size > limits.task_contract_document_verdict_choices {
                return Err(root_too_large());
            }
            let mut writer = Writer::new(usize::try_from(size).expect("wire size fits"));
            for row in &rows {
                writer.put(&row.measure(limits)?.to_be_bytes()).expect("measured");
                row.write(limits, &mut writer).expect("measured");
            }
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict { choices: writer.finish(), cut: *cut })
        }
    })
}

pub(crate) fn settings_wire_owner_part(value: &c::Part) -> w::Part {
    let kind = match value.kind {
        c::PartKind::Resource => w::PartKind::Resource,
        c::PartKind::Pool => w::PartKind::Pool,
        c::PartKind::Result => w::PartKind::Result,
        c::PartKind::Procedure => w::PartKind::Procedure,
        c::PartKind::Effect => w::PartKind::Effect,
        c::PartKind::Outbox => w::PartKind::Outbox,
        c::PartKind::Hold => w::PartKind::Hold,
    };
    let key = match &value.key {
        c::PartKey::Number(value) => w::PartKey::Number(w::PartKeyNumber { value: *value }),
        c::PartKey::Path(values) => {
            let mut rows = List::with_capacity(values.len());
            for value in values {
                rows.push(bytes::copy_of(value)).expect("same capacity");
            }
            w::PartKey::Path(w::PartKeyPath { value: rows })
        }
    };
    w::Part { connector: value.connector, kind, key }
}

pub(crate) fn settings_wire_document_text(value: &c::DocumentText) -> w::DocumentText {
    w::DocumentText { bytes: bytes::copy_of(value.bytes()), cut: value.cut }
}

pub(crate) fn settings_wire_charter(value: &c::Charter) -> w::Charter {
    let mut alternatives = List::with_capacity(value.alternatives.len());
    for item in &value.alternatives {
        alternatives.push(settings_wire_setting_model(item)).expect("same capacity");
    }
    let mut documents = List::with_capacity(value.documents.len());
    for item in &value.documents {
        documents
            .push(w::CharterDocument {
                connector: item.connector,
                name: item.name.clone(),
                path: item.path.clone(),
                heading: item.heading.clone(),
            })
            .expect("same capacity");
    }
    let mut parts = List::with_capacity(value.parts.len());
    for item in &value.parts {
        parts
            .push(w::CharterPart { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })
            .expect("same capacity");
    }
    w::Charter {
        name: value.name.clone(),
        parts,
        documents,
        rest: value.rest,
        resume: value.resume(),
        inspect: value.inspect(),
        modify: value.modify(),
        shell: value.shell(),
        agents: value.agents(),
        number: value.number,
        instructions: bytes::copy_of(value.instructions.bytes()),
        waiting: value.waiting,
        turns: value.turns,
        time: value.time,
        reserve_turns: value.reserve_turns,
        reserve_time: value.reserve_time,
        model: settings_wire_setting_model(&value.model),
        alternatives,
        call_timeout: value.call_timeout,
        tools: value.tools,
        hosts: settings_wire_setting_hosts(&value.hosts),
        spend: value.spend,
        wake: settings_wire_wake(value.wake),
    }
}

pub(crate) fn settings_wire_setting_hosts(value: &c::SettingHosts) -> w::SettingHosts {
    match value {
        c::SettingHosts::Workers => w::SettingHosts::Workers,
        c::SettingHosts::Engine => w::SettingHosts::Engine,
        c::SettingHosts::Both => w::SettingHosts::Both,
    }
}

pub(crate) fn settings_wire_setting_model(value: &c::SettingModel) -> w::SettingModel {
    let mut efforts = List::with_capacity(value.efforts.len());
    for item in &value.efforts {
        efforts.push(bytes::copy_of(item.bytes())).expect("same capacity");
    }
    let mut speeds = List::with_capacity(value.speeds.len());
    for speed in &value.speeds {
        let mut also = List::with_capacity(speed.also.len());
        for name in &speed.also {
            also.push(bytes::copy_of(name.bytes())).expect("same capacity");
        }
        speeds
            .push(w::SettingSpeed {
                name: bytes::copy_of(speed.name.bytes()),
                also,
                input_price: speed.input_price,
                cached_price: speed.cached_price,
                cache_write_price: speed.cache_write_price,
                output_price: speed.output_price,
            })
            .expect("same capacity");
    }
    w::SettingModel {
        provider: value.provider,
        dialect: value.dialect,
        endpoint: value.endpoint,
        program: optional_text(value.program.as_ref()),
        name: bytes::copy_of(value.name.bytes()),
        window: value.window,
        output: value.output,
        input_price: value.input_price,
        cached_price: value.cached_price,
        cache_write_price: value.cache_write_price,
        output_price: value.output_price,
        price_unit: value.price_unit,
        efforts,
        effort: optional_text(value.effort.as_ref()),
        speeds,
        speed: optional_text(value.speed.as_ref()),
    }
}

pub(crate) fn settings_wire_setting(value: c::Setting) -> w::Setting {
    match value {
        c::Setting::Charter(item) => w::Setting::Charter(settings_wire_setting_number(item)),
        c::Setting::Template(item) => w::Setting::Template(settings_wire_setting_number(item)),
        c::Setting::Policy(item) => w::Setting::Policy(settings_wire_setting_number(item)),
        c::Setting::Application { kind, scope, number } => {
            w::Setting::Application(w::ApplicationSetting { kind, scope, number })
        }
        c::Setting::Rules => w::Setting::Rules,
        c::Setting::Limits => w::Setting::Limits,
        c::Setting::Model(item) => w::Setting::Model(settings_wire_setting_number(item)),
        c::Setting::Worker(item) => w::Setting::Worker(settings_wire_setting_number(item)),
        c::Setting::Provider(item) => w::Setting::Provider(settings_wire_setting_number(item)),
        c::Setting::ProjectPolicy { project } => w::Setting::ProjectPolicy(w::ProjectPolicySetting { project }),
    }
}

pub(crate) fn settings_wire_setting_number(value: c::SettingNumber) -> w::SettingNumber {
    w::SettingNumber { number: value.number }
}

fn optional_proposal(value: Option<&w::ProposalKey>) -> Option<c::ProposalKey> {
    let value = value?;
    Some(answers_proposal_key(value))
}
fn optional_wall(value: Option<&u64>) -> Option<Wall> {
    let value = value?;
    Some(Wall::from_nanos(*value))
}
fn optional_nanos(value: Option<Wall>) -> Option<u64> {
    let value = value?;
    Some(value.as_nanos())
}
fn optional_executor(value: Option<&w::TemplateExecutor>) -> Option<c::TemplateExecutor> {
    let value = value?;
    Some(operations_restore_executor(value))
}
fn optional_budget(value: Option<&w::TemplateBudget>) -> Option<c::TemplateBudget> {
    let value = value?;
    Some(operations_restore_budget(value))
}
fn optional_deadline(value: Option<&w::TemplateDeadline>) -> Option<c::TemplateDeadline> {
    let value = value?;
    Some(operations_restore_deadline(value))
}
fn optional_text(value: Option<&c::Text>) -> Option<Box<[u8]>> {
    let value = value?;
    Some(bytes::copy_of(value.bytes()))
}

fn settings_wire_wake(value: c::WakePolicy) -> w::WakePolicy {
    w::WakePolicy {
        words: settings_wire_wake_rule(value.words),
        notices: settings_wire_wake_rule(value.notices),
        news: settings_wire_wake_rule(value.news),
        results: match value.results {
            c::ResultsWake::Never => w::ResultsWake::Never,
            c::ResultsWake::Each => w::ResultsWake::Each,
            c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}
fn settings_wire_wake_rule(value: c::WakeRule) -> w::WakeRule {
    match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => w::WakeRule::Batch(w::WakeRuleBatch { count, age }),
        c::WakeRule::Paced { span } => w::WakeRule::Paced(w::WakeRulePaced { span }),
    }
}

fn settings_restore_wake(value: &w::WakePolicy) -> c::WakePolicy {
    c::WakePolicy {
        words: settings_restore_wake_rule(&value.words),
        notices: settings_restore_wake_rule(&value.notices),
        news: settings_restore_wake_rule(&value.news),
        results: match &value.results {
            w::ResultsWake::Never => c::ResultsWake::Never,
            w::ResultsWake::Each => c::ResultsWake::Each,
            w::ResultsWake::LastOrFailure => c::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}
fn settings_restore_wake_rule(value: &w::WakeRule) -> c::WakeRule {
    match value {
        w::WakeRule::Never => c::WakeRule::Never,
        w::WakeRule::Immediate => c::WakeRule::Immediate,
        w::WakeRule::Batch(value) => c::WakeRule::Batch { count: value.count, age: value.age },
        w::WakeRule::Paced(value) => c::WakeRule::Paced { span: value.span },
    }
}

fn restore_sized_edit(v: &w::SizedEdit) -> c::SizedEdit {
    match v {
        w::SizedEdit::Set(v) => c::SizedEdit::Set { size: v.size },
        w::SizedEdit::Remove => c::SizedEdit::Remove,
        w::SizedEdit::Reset => c::SizedEdit::Reset,
    }
}
fn restore_application_state(v: &w::ApplicationState) -> c::ApplicationState {
    c::ApplicationState {
        follows: match &v.follows {
            w::ApplicationFollows::Default => c::ApplicationFollows::Default,
            w::ApplicationFollows::Changed(v) => c::ApplicationFollows::Changed { revision: v.revision },
            w::ApplicationFollows::Removed(v) => c::ApplicationFollows::Removed { revision: v.revision },
        },
        size: v.size,
        charters: v.charters.clone(),
    }
}
fn wire_catalogue(value: &c::CatalogueEntry) -> w::CatalogueEntry {
    w::CatalogueEntry {
        number: value.number,
        model: settings_wire_setting_model(&value.model),
        shown: match &value.shown {
            Some(value) => Some(bytes::copy_of(value.bytes())),
            None => None,
        },
        description: match &value.description {
            Some(value) => Some(bytes::copy_of(value.bytes())),
            None => None,
        },
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: match value.source {
            c::ModelSource::Stock => w::ModelSource::Stock,
            c::ModelSource::List => w::ModelSource::List,
            c::ModelSource::Person => w::ModelSource::Person,
        },
    }
}
fn restore_catalogue(value: &w::CatalogueEntry) -> Result<c::CatalogueEntry, w::Problem> {
    Ok(c::CatalogueEntry {
        number: value.number,
        model: settings_restore_setting_model(&value.model)?,
        shown: match &value.shown {
            Some(value) => Some(settings_text(value, 4096)?),
            None => None,
        },
        description: match &value.description {
            Some(value) => Some(settings_text(value, 512)?),
            None => None,
        },
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: match &value.source {
            w::ModelSource::Stock => c::ModelSource::Stock,
            w::ModelSource::List => c::ModelSource::List,
            w::ModelSource::Person => c::ModelSource::Person,
        },
    })
}

mod providers;
