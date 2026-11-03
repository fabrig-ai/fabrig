//! Whole values for settings JSON; no display truncation is applied.
use crate::view as c;
use skein_json::writer::Encoder;
fn encode_text(value: &c::Text, out: &mut Encoder) {
    out.string(value.bytes());
}
pub fn encode_document_text(value: &c::DocumentText, out: &mut Encoder) {
    out.object_start();
    out.key(b"bytes");
    out.string(value.bytes());
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}
pub fn encode_charter(value: &c::Charter, out: &mut Encoder) {
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(value.number));
    out.key(b"instructions");
    encode_text(&value.instructions, out);
    out.key(b"waiting");
    out.unsigned(value.waiting.as_nanos());
    out.key(b"resume");
    out.boolean(value.resume());
    out.key(b"turns");
    out.unsigned(u64::from(value.turns));
    out.key(b"time");
    out.unsigned(value.time.as_nanos());
    out.key(b"reserve_turns");
    out.unsigned(u64::from(value.reserve_turns));
    out.key(b"reserve_time");
    out.unsigned(value.reserve_time.as_nanos());
    out.key(b"model");
    encode_setting_model(&value.model, out);
    out.key(b"alternatives");
    out.array_start();
    for value in &value.alternatives {
        encode_setting_model(value, out);
    }
    out.array_end();
    out.key(b"inspect");
    out.boolean(value.inspect());
    out.key(b"modify");
    out.boolean(value.modify());
    out.key(b"shell");
    out.boolean(value.shell());
    out.key(b"agents");
    out.boolean(value.agents());
    out.key(b"call_timeout");
    out.unsigned(value.call_timeout.as_nanos());
    out.key(b"hosts");
    encode_setting_hosts(&value.hosts, out);
    out.key(b"spend");
    out.unsigned(value.spend);
    out.key(b"wake");
    encode_wake(value.wake, out);
    out.key(b"documents");
    out.array_start();
    for document in &value.documents {
        encode_document(document, out);
    }
    out.array_end();
    out.object_end();
}
fn encode_document(value: &c::CharterDocument, out: &mut Encoder) {
    out.object_start();
    if forge_branch(value) {
        out.key(b"forge");
        out.string(value.name.get(1).expect("forge host"));
        out.key(b"repository");
        out.string(&joined_name(&value.name, 2, 4));
        out.key(b"branch");
        out.string(&joined_name(&value.name, 5, value.name.len()));
    } else {
        out.key(b"connector");
        out.unsigned(u64::from(value.connector));
        out.key(b"name");
        out.array_start();
        for segment in &value.name {
            out.string(segment);
        }
        out.array_end();
    }
    out.key(b"path");
    out.string(&value.path);
    out.key(b"heading");
    match &value.heading {
        Some(heading) => out.string(heading),
        None => out.null(),
    }
    out.object_end();
}
fn forge_branch(value: &c::CharterDocument) -> bool {
    if value.connector != 0
        || value.name.len() < 6
        || value.name.get(0).expect("name prefix").as_ref() != b"forge"
        || value.name.get(4).expect("branch prefix").as_ref() != b"branch"
    {
        return false;
    }
    for segment in &value.name {
        if segment.is_empty() || segment.contains(&b'/') {
            return false;
        }
    }
    true
}
fn joined_name(name: &skein_lib::List<alloc::boxed::Box<[u8]>>, start: u32, end: u32) -> alloc::boxed::Box<[u8]> {
    let mut measured = crate::Text::measure();
    write_joined_name(name, start, end, &mut measured);
    let mut out = crate::Text::writing_exact(measured);
    write_joined_name(name, start, end, &mut out);
    out.finish()
}
fn write_joined_name(name: &skein_lib::List<alloc::boxed::Box<[u8]>>, start: u32, end: u32, out: &mut crate::Text) {
    for at in start..end {
        if at != start {
            out.put(b"/");
        }
        out.put(name.get(at).expect("name range"));
    }
}
pub fn encode_template(value: &c::Template, out: &mut Encoder) {
    out.object_start();
    out.key(b"resources");
    out.array_start();
    for symbol in &value.resources {
        out.object_start();
        out.key(b"connector");
        out.unsigned(u64::from(symbol.connector));
        out.key(b"symbol");
        out.unsigned(u64::from(symbol.symbol));
        out.object_end();
    }
    out.array_end();
    out.key(b"number");
    out.unsigned(u64::from(value.number));
    out.key(b"name");
    encode_document_text(&value.name, out);
    out.key(b"executor");
    encode_template_executor(value.executor, out);
    out.key(b"contract");
    encode_task_contract_document(&value.contract, out);
    out.key(b"shallower");
    out.unsigned(u64::from(value.shallower));
    out.key(b"budget");
    encode_template_budget(&value.budget, out);
    out.key(b"deadline");
    encode_template_deadline(&value.deadline, out);
    out.key(b"tracked");
    match &value.tracked {
        Some(value) => {
            out.unsigned(u64::from(*(value)));
        }
        None => out.null(),
    }
    out.key(b"flags");
    encode_task_flags(value.flags, out);
    out.key(b"label");
    out.unsigned(u64::from(value.label));
    out.key(b"grace");
    match &value.grace {
        Some(value) => {
            out.unsigned(value.as_nanos());
        }
        None => out.null(),
    }
    out.key(b"llm");
    match &value.llm {
        Some(value) => {
            encode_llm_choice(value, out);
        }
        None => out.null(),
    }
    out.object_end();
}
pub fn encode_setting_policy(value: &c::SettingPolicy, out: &mut Encoder) {
    out.object_start();
    out.key(b"escalation_role");
    match &value.escalation_role {
        Some(value) => {
            out.unsigned(u64::from(*(value)));
        }
        None => out.null(),
    }
    out.key(b"ceiling");
    encode_owner_authority(&value.ceiling, out);
    out.key(b"projections");
    out.array_start();
    for value in &value.projections {
        encode_owner_grant(value, out);
    }
    out.array_end();
    out.key(b"period_spend");
    out.unsigned(value.period_spend);
    out.key(b"roles");
    out.array_start();
    for value in &value.roles {
        encode_policy_edit_role(value, out);
    }
    out.array_end();
    out.key(b"requirements");
    out.array_start();
    for value in &value.requirements {
        encode_policy_requirement(value, out);
    }
    out.array_end();
    out.key(b"permissions");
    out.array_start();
    for value in &value.permissions {
        encode_policy_permission(*value, out);
    }
    out.array_end();
    out.key(b"templates");
    out.array_start();
    for value in &value.templates {
        encode_template_override(value, out);
    }
    out.array_end();
    out.object_end();
}
pub fn encode_rules(value: &c::Rules, out: &mut Encoder) {
    out.object_start();
    out.key(b"ceiling");
    encode_owner_authority(&value.ceiling, out);
    out.key(b"period_spend");
    out.unsigned(value.period_spend);
    out.key(b"minimum_run_spend");
    out.unsigned(value.minimum_run_spend);
    out.key(b"maximum_run_spend");
    out.unsigned(value.maximum_run_spend);
    out.key(b"implies");
    out.array_start();
    for value in &value.implies {
        encode_owner_kind_implication(*value, out);
    }
    out.array_end();
    out.key(b"requirements");
    out.array_start();
    for value in &value.requirements {
        encode_policy_requirement(value, out);
    }
    out.array_end();
    out.object_end();
}
pub fn encode_setting_model(value: &c::SettingModel, out: &mut Encoder) {
    out.object_start();
    out.key(b"dialect");
    out.unsigned(u64::from(value.dialect));
    out.key(b"endpoint");
    out.unsigned(u64::from(value.endpoint));
    out.key(b"program");
    match &value.program {
        Some(value) => {
            encode_text(value, out);
        }
        None => out.null(),
    }
    out.key(b"name");
    encode_text(&value.name, out);
    out.key(b"window");
    out.unsigned(u64::from(value.window));
    out.key(b"output");
    out.unsigned(u64::from(value.output));
    out.key(b"input_price");
    out.unsigned(value.input_price);
    out.key(b"cached_price");
    out.unsigned(value.cached_price);
    out.key(b"cache_write_price");
    match &value.cache_write_price {
        Some(value) => {
            out.unsigned(*value);
        }
        None => out.null(),
    }
    out.key(b"output_price");
    out.unsigned(value.output_price);
    out.key(b"price_unit");
    out.unsigned(u64::from(value.price_unit));
    out.key(b"efforts");
    out.array_start();
    for value in &value.efforts {
        encode_text(value, out);
    }
    out.array_end();
    out.key(b"effort");
    match &value.effort {
        Some(value) => {
            encode_text(value, out);
        }
        None => out.null(),
    }
    out.object_end();
}
pub fn encode_setting_hosts(value: &c::SettingHosts, out: &mut Encoder) {
    match value {
        c::SettingHosts::Workers => out.string(b"workers"),
        c::SettingHosts::Engine => out.string(b"engine"),
        c::SettingHosts::Both => out.string(b"both"),
    }
}
pub fn encode_template_executor(value: c::TemplateExecutor, out: &mut Encoder) {
    let value = &value;
    match value {
        c::TemplateExecutor::Charter(value) => {
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(*(value)));
            out.object_end();
        }
        c::TemplateExecutor::Chosen => out.string(b"chosen"),
    }
}
pub fn encode_task_contract_document(value: &c::TaskContractDocument, out: &mut Encoder) {
    match value {
        c::TaskContractDocument::Job { bytes } => {
            out.object_start();
            out.key(b"job");
            out.object_start();
            out.key(b"bytes");
            out.unsigned(u64::from(*bytes));
            out.object_end();
            out.object_end();
        }
        c::TaskContractDocument::Approval { state, part } => {
            out.object_start();
            out.key(b"approval");
            out.object_start();
            out.key(b"state");
            crate::hex(state, out);
            out.key(b"part");
            encode_owner_part(part, out);
            out.object_end();
            out.object_end();
        }
        c::TaskContractDocument::Report { words } => {
            out.object_start();
            out.key(b"report");
            out.object_start();
            out.key(b"words");
            out.unsigned(u64::from(*(words)));
            out.object_end();
            out.object_end();
        }
        c::TaskContractDocument::Verdict { choices, cut } => {
            out.object_start();
            out.key(b"verdict");
            out.object_start();
            out.key(b"choices");
            out.array_start();
            for value in choices {
                encode_verdict_choice(value, out);
            }
            out.array_end();
            out.key(b"cut");
            out.boolean(*(cut));
            out.object_end();
            out.object_end();
        }
        c::TaskContractDocument::Change { connector, kind, words } => {
            out.object_start();
            out.key(b"change");
            out.object_start();
            out.key(b"connector");
            out.unsigned(u64::from(*(connector)));
            out.key(b"kind");
            out.unsigned(u64::from(*(kind)));
            out.key(b"words");
            out.unsigned(u64::from(*(words)));
            out.object_end();
            out.object_end();
        }
    }
}
pub fn encode_template_budget(value: &c::TemplateBudget, out: &mut Encoder) {
    match value {
        c::TemplateBudget::Amount(value) => {
            out.object_start();
            out.key(b"amount");
            out.unsigned(*value);
            out.object_end();
        }
        c::TemplateBudget::AsAllowed { cap } => {
            out.object_start();
            out.key(b"as_allowed");
            out.object_start();
            out.key(b"cap");
            match cap {
                Some(value) => {
                    out.unsigned(*value);
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        c::TemplateBudget::Given => out.string(b"given"),
    }
}
pub fn encode_template_deadline(value: &c::TemplateDeadline, out: &mut Encoder) {
    match value {
        c::TemplateDeadline::None => out.string(b"none"),
        c::TemplateDeadline::Span(value) => {
            out.object_start();
            out.key(b"span");
            out.unsigned(value.as_nanos());
            out.object_end();
        }
        c::TemplateDeadline::Given => out.string(b"given"),
    }
}
pub fn encode_task_flags(value: c::TaskFlags, out: &mut Encoder) {
    let value = &value;
    out.object_start();
    out.key(b"conversation");
    out.boolean(value.conversation);
    out.key(b"listed");
    out.boolean(value.listed);
    out.key(b"retention");
    encode_retention(value.retention, out);
    out.key(b"ended_by");
    encode_ended_by(value.ended_by, out);
    out.object_end();
}
pub fn encode_llm_choice(value: &c::LlmChoice, out: &mut Encoder) {
    out.object_start();
    out.key(b"account");
    match &value.account {
        Some(value) => {
            out.unsigned(u64::from(*(value)));
        }
        None => out.null(),
    }
    out.key(b"main");
    encode_model_effort(&value.main, out);
    out.key(b"subagents");
    out.array_start();
    for value in &value.subagents {
        encode_model_effort(value, out);
    }
    out.array_end();
    out.object_end();
}
pub fn encode_owner_authority(value: &c::Authority, out: &mut Encoder) {
    out.object_start();
    out.key(b"models");
    match &value.models {
        Some(value) => {
            encode_authority_models(value, out);
        }
        None => out.null(),
    }
    out.key(b"tools");
    out.unsigned(value.tools);
    out.key(b"grants");
    out.array_start();
    for value in &value.grants {
        encode_owner_grant(value, out);
    }
    out.array_end();
    out.key(b"delegation");
    encode_owner_delegation(&value.delegation, out);
    out.key(b"spend");
    out.unsigned(value.spend);
    out.key(b"deadline");
    match &value.deadline {
        Some(value) => {
            out.unsigned(value.as_nanos());
        }
        None => out.null(),
    }
    out.key(b"notes");
    out.unsigned(u64::from(value.notes));
    out.key(b"note_resources");
    out.array_start();
    for value in &value.note_resources {
        encode_owner_resource_scope(value, out);
    }
    out.array_end();
    out.object_end();
}
pub fn encode_owner_grant(value: &c::Grant, out: &mut Encoder) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"pattern");
    encode_owner_pattern(&value.pattern, out);
    out.object_end();
}
pub fn encode_policy_edit_role(value: &c::PolicyEditRole, out: &mut Encoder) {
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(value.number));
    out.key(b"authority");
    encode_owner_authority(&value.authority, out);
    out.key(b"period_spend");
    out.unsigned(value.period_spend);
    out.key(b"requests");
    out.unsigned(u64::from(value.requests));
    out.key(b"decides");
    out.unsigned(u64::from(value.decides));
    out.object_end();
}
pub fn encode_policy_requirement(value: &c::PolicyRequirement, out: &mut Encoder) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"pattern");
    encode_owner_pattern(&value.pattern, out);
    out.key(b"judge");
    encode_policy_judge(value.judge, out);
    out.key(b"guard");
    encode_policy_guard(&value.guard, out);
    out.key(b"must_be_guarded");
    out.boolean(value.must_be_guarded);
    out.object_end();
}
pub fn encode_policy_permission(value: c::PolicyPermission, out: &mut Encoder) {
    let value = &value;
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"permission");
    out.unsigned(u64::from(value.permission));
    out.key(b"role");
    out.unsigned(u64::from(value.role));
    out.object_end();
}
pub fn encode_template_override(value: &c::TemplateOverride, out: &mut Encoder) {
    out.object_start();
    out.key(b"template");
    out.unsigned(u64::from(value.template));
    out.key(b"executor");
    match &value.executor {
        Some(value) => {
            encode_template_executor(*value, out);
        }
        None => out.null(),
    }
    out.key(b"budget");
    match &value.budget {
        Some(value) => {
            encode_template_budget(value, out);
        }
        None => out.null(),
    }
    out.key(b"deadline");
    match &value.deadline {
        Some(value) => {
            encode_template_deadline(value, out);
        }
        None => out.null(),
    }
    out.key(b"priority");
    match &value.priority {
        Some(value) => {
            out.unsigned(u64::from(*(value)));
        }
        None => out.null(),
    }
    out.object_end();
}
pub fn encode_owner_kind_implication(value: c::KindImplication, out: &mut Encoder) {
    let value = &value;
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"implies");
    out.unsigned(u64::from(value.implies));
    out.object_end();
}
pub fn encode_owner_part(value: &c::Part, out: &mut Encoder) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    encode_owner_part_kind(value.kind, out);
    out.key(b"key");
    encode_owner_part_key(&value.key, out);
    out.object_end();
}
pub fn encode_verdict_choice(value: &c::VerdictChoice, out: &mut Encoder) {
    out.object_start();
    out.key(b"code");
    out.unsigned(u64::from(value.code));
    out.key(b"words");
    out.unsigned(u64::from(value.words));
    out.key(b"followups");
    out.unsigned(u64::from(value.followups));
    out.key(b"fields");
    out.array_start();
    for value in &value.fields {
        encode_field_rule(value, out);
    }
    out.array_end();
    out.object_end();
}
pub fn encode_retention(value: c::Retention, out: &mut Encoder) {
    let value = &value;
    match value {
        c::Retention::Kept => out.string(b"kept"),
        c::Retention::ForgottenOnceRead => out.string(b"forgotten_once_read"),
    }
}
pub fn encode_ended_by(value: c::EndedBy, out: &mut Encoder) {
    let value = &value;
    match value {
        c::EndedBy::Result => out.string(b"result"),
        c::EndedBy::Requester => out.string(b"requester"),
    }
}
pub fn encode_model_effort(value: &c::ModelEffort, out: &mut Encoder) {
    out.object_start();
    out.key(b"model");
    encode_text(&value.model, out);
    out.key(b"effort");
    match &value.effort {
        Some(value) => {
            encode_text(value, out);
        }
        None => out.null(),
    }
    out.object_end();
}
pub fn encode_authority_models(value: &c::AuthorityModels, out: &mut Encoder) {
    match value {
        c::AuthorityModels::All => out.string(b"all"),
        c::AuthorityModels::Named(value) => {
            out.object_start();
            out.key(b"named");
            out.array_start();
            for value in value {
                crate::hex(value, out);
            }
            out.array_end();
            out.object_end();
        }
    }
}
pub fn encode_owner_delegation(value: &c::Delegation, out: &mut Encoder) {
    out.object_start();
    out.key(b"kinds");
    out.array_start();
    for value in &value.kinds {
        encode_owner_authority_executor(*value, out);
    }
    out.array_end();
    out.key(b"tasks");
    out.unsigned(u64::from(value.tasks));
    out.key(b"depth");
    out.unsigned(u64::from(value.depth));
    out.object_end();
}
pub fn encode_owner_resource_scope(value: &c::ResourceScope, out: &mut Encoder) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"pattern");
    encode_owner_pattern(&value.pattern, out);
    out.object_end();
}
pub fn encode_owner_pattern(value: &c::Pattern, out: &mut Encoder) {
    out.object_start();
    out.key(b"segments");
    out.array_start();
    for value in &value.segments {
        crate::hex(value, out);
    }
    out.array_end();
    out.key(b"last");
    encode_owner_pattern_end(&value.last, out);
    out.object_end();
}
pub fn encode_policy_judge(value: c::PolicyJudge, out: &mut Encoder) {
    let value = &value;
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"requirement");
    out.unsigned(u64::from(value.requirement));
    out.key(b"parameters");
    out.unsigned(u64::from(value.parameters));
    out.object_end();
}
pub fn encode_policy_guard(value: &c::PolicyGuard, out: &mut Encoder) {
    match value {
        c::PolicyGuard::Guarded => out.string(b"guarded"),
        c::PolicyGuard::Observed { freshness } => {
            out.object_start();
            out.key(b"observed");
            out.object_start();
            out.key(b"freshness");
            out.unsigned(freshness.as_nanos());
            out.object_end();
            out.object_end();
        }
    }
}
pub fn encode_owner_part_kind(value: c::PartKind, out: &mut Encoder) {
    let value = &value;
    match value {
        c::PartKind::Resource => out.string(b"resource"),
        c::PartKind::Pool => out.string(b"pool"),
        c::PartKind::Result => out.string(b"result"),
        c::PartKind::Procedure => out.string(b"procedure"),
        c::PartKind::Effect => out.string(b"effect"),
        c::PartKind::Outbox => out.string(b"outbox"),
        c::PartKind::Hold => out.string(b"hold"),
    }
}
pub fn encode_owner_part_key(value: &c::PartKey, out: &mut Encoder) {
    match value {
        c::PartKey::Path(value) => {
            out.object_start();
            out.key(b"path");
            out.array_start();
            for value in value {
                crate::hex(value, out);
            }
            out.array_end();
            out.object_end();
        }
        c::PartKey::Number(value) => {
            out.object_start();
            out.key(b"number");
            out.unsigned(*value);
            out.object_end();
        }
    }
}
pub fn encode_field_rule(value: &c::FieldRule, out: &mut Encoder) {
    out.object_start();
    out.key(b"name");
    crate::hex(&value.name, out);
    out.key(b"max");
    out.unsigned(u64::from(value.max));
    out.object_end();
}
pub fn encode_owner_authority_executor(value: c::AuthorityExecutor, out: &mut Encoder) {
    let value = &value;
    match value {
        c::AuthorityExecutor::Job(value) => {
            out.object_start();
            out.key(b"job");
            out.unsigned(u64::from(*value));
            out.object_end();
        }
        c::AuthorityExecutor::Charter(value) => {
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(*(value)));
            out.object_end();
        }
        c::AuthorityExecutor::Procedure(value) => {
            out.object_start();
            out.key(b"procedure");
            out.unsigned(u64::from(*(value)));
            out.object_end();
        }
        c::AuthorityExecutor::Role(value) => {
            out.object_start();
            out.key(b"role");
            out.unsigned(u64::from(*(value)));
            out.object_end();
        }
    }
}
pub fn encode_owner_pattern_end(value: &c::PatternEnd, out: &mut Encoder) {
    match value {
        c::PatternEnd::Exact(value) => {
            out.object_start();
            out.key(b"exact");
            crate::hex(value, out);
            out.object_end();
        }
        c::PatternEnd::Open(value) => {
            out.object_start();
            out.key(b"open");
            crate::hex(value, out);
            out.object_end();
        }
    }
}

fn encode_wake(value: c::WakePolicy, out: &mut Encoder) {
    out.object_start();
    out.key(b"words");
    encode_wake_rule(value.words, out);
    out.key(b"notices");
    encode_wake_rule(value.notices, out);
    out.key(b"news");
    encode_wake_rule(value.news, out);
    out.key(b"results");
    out.string(match value.results {
        c::ResultsWake::Never => b"never",
        c::ResultsWake::Each => b"each",
        c::ResultsWake::LastOrFailure => b"last_or_failure",
    });
    out.key(b"questions");
    out.boolean(value.questions);
    out.key(b"answers");
    out.boolean(value.answers);
    out.key(b"timers");
    out.boolean(value.timers);
    out.object_end();
}
fn encode_wake_rule(value: c::WakeRule, out: &mut Encoder) {
    match value {
        c::WakeRule::Never => out.string(b"never"),
        c::WakeRule::Immediate => out.string(b"immediate"),
        c::WakeRule::Batch { count, age } => {
            out.object_start();
            out.key(b"batch");
            out.object_start();
            out.key(b"count");
            out.unsigned(u64::from(count));
            out.key(b"age");
            out.unsigned(age.as_nanos());
            out.object_end();
            out.object_end();
        }
        c::WakeRule::Paced { span } => {
            out.object_start();
            out.key(b"paced");
            out.unsigned(span.as_nanos());
            out.object_end();
        }
    }
}
