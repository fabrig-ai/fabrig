//! Concrete client/neutral settings views; no input parsing or text layout lives here.
use alloc::boxed::Box;
use jig_client::contract as c;
use settings_form::view as n;
use skein_lib::{Duration, List, Wall};
pub fn form_authority_models(v: &c::AuthorityModels) -> n::AuthorityModels {
    match v {
        c::AuthorityModels::All => n::AuthorityModels::All,
        c::AuthorityModels::Named(x) => n::AuthorityModels::Named({
            let mut out = List::with_capacity(x.len());
            for item in x {
                out.push(item.clone()).expect("one value per input");
            }
            out
        }),
    }
}
pub fn client_authority_models(v: &n::AuthorityModels) -> c::AuthorityModels {
    match v {
        n::AuthorityModels::All => c::AuthorityModels::All,
        n::AuthorityModels::Named(x) => c::AuthorityModels::Named({
            let mut out = List::with_capacity(x.len());
            for item in x {
                out.push(item.clone()).expect("one value per input");
            }
            out
        }),
    }
}

pub fn form_overlay(v: &c::Overlay) -> n::Overlay {
    n::Overlay {
        charters: List::with_capacity(0),
        rest: v.rest,
        model: match &v.model {
            Some(v) => Some(form_setting_model(v)),
            None => None,
        },
        alternatives: match &v.alternatives {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(form_setting_model(item)).expect("same count");
                }
                Some(out)
            }
            None => None,
        },
        inspect: v.inspect,
        modify: v.modify,
        shell: v.shell,
        agents: v.agents,
        tools: v.tools,
        hosts: match &v.hosts {
            Some(v) => Some(form_setting_hosts(v)),
            None => None,
        },
        waiting: v.waiting,
        resume: v.resume,
        turns: v.turns,
        time: v.time,
        reserve_turns: v.reserve_turns,
        reserve_time: v.reserve_time,
        call_timeout: v.call_timeout,
        spend: v.spend,
        documents: match &v.documents {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(n::CharterDocument {
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
        instructions: form_text(&v.instructions),
        parts: {
            let mut out = List::with_capacity(v.parts.len());
            for p in &v.parts {
                out.push(n::CharterPart { on: p.on, instructions: form_text(&p.instructions) }).expect("same count");
            }
            out
        },
    }
}
pub fn client_overlay(v: &n::Overlay) -> c::Overlay {
    c::Overlay {
        rest: v.rest,
        model: match &v.model {
            Some(v) => Some(client_setting_model(v)),
            None => None,
        },
        alternatives: match &v.alternatives {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(client_setting_model(item)).expect("same count");
                }
                Some(out)
            }
            None => None,
        },
        inspect: v.inspect,
        modify: v.modify,
        shell: v.shell,
        agents: v.agents,
        tools: v.tools,
        hosts: match &v.hosts {
            Some(v) => Some(client_setting_hosts(v)),
            None => None,
        },
        waiting: v.waiting,
        resume: v.resume,
        turns: v.turns,
        time: v.time,
        reserve_turns: v.reserve_turns,
        reserve_time: v.reserve_time,
        call_timeout: v.call_timeout,
        spend: v.spend,
        documents: match &v.documents {
            Some(v) => {
                let mut out = List::with_capacity(v.len());
                for item in v {
                    out.push(c::CharterDocument {
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
        instructions: client_text(&v.instructions),
        parts: {
            let mut out = List::with_capacity(v.parts.len());
            for p in &v.parts {
                out.push(c::CharterPart { on: p.on, instructions: client_text(&p.instructions) }).expect("same count");
            }
            out
        },
    }
}
pub fn form_charter(v: &c::Charter) -> n::Charter {
    n::Charter {
        parts: {
            let mut out = List::with_capacity(v.parts.len());
            for item in &v.parts {
                out.push(n::CharterPart { on: item.on, instructions: form_text(&item.instructions) })
                    .expect("one value per input");
            }
            out
        },
        rest: v.rest,
        documents: {
            let mut out = List::with_capacity(v.documents.len());
            for item in &v.documents {
                out.push(n::CharterDocument {
                    connector: item.connector,
                    name: item.name.clone(),
                    path: item.path.clone(),
                    heading: item.heading.clone(),
                })
                .expect("one value per input");
            }
            out
        },
        number: v.number,
        instructions: form_text(&v.instructions),
        waiting: v.waiting,
        turns: v.turns,
        time: v.time,
        reserve_turns: v.reserve_turns,
        reserve_time: v.reserve_time,
        model: form_setting_model(&v.model),
        alternatives: {
            let mut out = List::with_capacity(v.alternatives.len());
            for item in &v.alternatives {
                out.push(form_setting_model(item)).expect("one value per input");
            }
            out
        },
        call_timeout: v.call_timeout,
        tools: v.tools,
        hosts: form_setting_hosts(&v.hosts),
        spend: v.spend,
        wake: form_wake(v.wake),
        switches: [v.resume, v.inspect, v.modify, v.shell, v.agents],
    }
}
pub fn client_charter(v: &n::Charter) -> c::Charter {
    c::Charter {
        parts: {
            let mut out = List::with_capacity(v.parts.len());
            for item in &v.parts {
                out.push(c::CharterPart { on: item.on, instructions: client_text(&item.instructions) })
                    .expect("one value per input");
            }
            out
        },
        rest: v.rest,
        documents: {
            let mut out = List::with_capacity(v.documents.len());
            for item in &v.documents {
                out.push(c::CharterDocument {
                    connector: item.connector,
                    name: item.name.clone(),
                    path: item.path.clone(),
                    heading: item.heading.clone(),
                })
                .expect("one value per input");
            }
            out
        },
        number: v.number,
        instructions: client_text(&v.instructions),
        waiting: v.waiting,
        resume: v.resume(),
        turns: v.turns,
        time: v.time,
        reserve_turns: v.reserve_turns,
        reserve_time: v.reserve_time,
        model: client_setting_model(&v.model),
        alternatives: {
            let mut out = List::with_capacity(v.alternatives.len());
            for item in &v.alternatives {
                out.push(client_setting_model(item)).expect("one value per input");
            }
            out
        },
        inspect: v.inspect(),
        modify: v.modify(),
        shell: v.shell(),
        agents: v.agents(),
        call_timeout: v.call_timeout,
        tools: v.tools,
        hosts: client_setting_hosts(&v.hosts),
        spend: v.spend,
        wake: client_wake(v.wake),
    }
}
pub fn form_document_text(v: &c::DocumentText) -> n::DocumentText {
    n::DocumentText { bytes: form_text(&v.bytes), cut: v.cut }
}
pub fn client_document_text(v: &n::DocumentText) -> c::DocumentText {
    c::DocumentText { bytes: client_text(&v.bytes), cut: v.cut }
}
pub fn form_ended_by(v: c::EndedBy) -> n::EndedBy {
    match v {
        c::EndedBy::Result => n::EndedBy::Result,
        c::EndedBy::Requester => n::EndedBy::Requester,
        c::EndedBy::Standing => n::EndedBy::Standing,
        c::EndedBy::Closer(closer) => n::EndedBy::Closer(n::Closer { connector: closer.connector, code: closer.code }),
    }
}
pub fn client_ended_by(v: n::EndedBy) -> c::EndedBy {
    match v {
        n::EndedBy::Result => c::EndedBy::Result,
        n::EndedBy::Requester => c::EndedBy::Requester,
        n::EndedBy::Standing => c::EndedBy::Standing,
        n::EndedBy::Closer(closer) => c::EndedBy::Closer(c::Closer { connector: closer.connector, code: closer.code }),
    }
}
pub fn form_field_rule(v: &c::FieldRule) -> n::FieldRule {
    n::FieldRule { name: v.name.clone(), max: v.max }
}
pub fn client_field_rule(v: &n::FieldRule) -> c::FieldRule {
    c::FieldRule { name: v.name.clone(), max: v.max }
}
pub fn form_held_revision(v: &c::HeldRevision) -> n::HeldRevision {
    n::HeldRevision { head: form_revision_head(&v.head), holders: v.holders }
}

pub fn form_llm_choice(v: &c::LlmChoice) -> n::LlmChoice {
    n::LlmChoice {
        later: match &v.later {
            Some(later) => {
                let mut subagents = List::with_capacity(later.subagents.len());
                for item in &later.subagents {
                    subagents.push(form_model_effort(item)).expect("one value per input");
                }
                Some(n::LlmLater { main: form_model_effort(&later.main), subagents })
            }
            None => None,
        },
        account: form_optional_u32(v.account.as_ref()),
        main: form_model_effort(&v.main),
        subagents: {
            let mut out = List::with_capacity(v.subagents.len());
            for item in &v.subagents {
                out.push(form_model_effort(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn client_llm_choice(v: &n::LlmChoice) -> c::LlmChoice {
    c::LlmChoice {
        later: match &v.later {
            Some(later) => {
                let mut subagents = List::with_capacity(later.subagents.len());
                for item in &later.subagents {
                    subagents.push(client_model_effort(item)).expect("one value per input");
                }
                Some(c::LlmLater { main: client_model_effort(&later.main), subagents })
            }
            None => None,
        },
        account: client_optional_u32(v.account.as_ref()),
        main: client_model_effort(&v.main),
        subagents: {
            let mut out = List::with_capacity(v.subagents.len());
            for item in &v.subagents {
                out.push(client_model_effort(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn form_model_effort(v: &c::ModelEffort) -> n::ModelEffort {
    n::ModelEffort { model: form_text(&v.model), effort: form_optional_text(v.effort.as_ref()) }
}
pub fn client_model_effort(v: &n::ModelEffort) -> c::ModelEffort {
    c::ModelEffort { model: client_text(&v.model), effort: client_optional_text(v.effort.as_ref()) }
}
pub fn form_owner_authority(v: &c::Authority) -> n::Authority {
    n::Authority {
        models: form_optional_authority_models(v.models.as_ref()),
        tools: v.tools,
        grants: {
            let mut out = List::with_capacity(v.grants.len());
            for item in &v.grants {
                out.push(form_owner_grant(item)).expect("one value per input");
            }
            out
        },
        delegation: form_owner_delegation(&v.delegation),
        spend: v.spend,
        deadline: form_optional_wall(v.deadline.as_ref()),
        notes: v.notes,
        note_resources: {
            let mut out = List::with_capacity(v.note_resources.len());
            for item in &v.note_resources {
                out.push(form_owner_resource_scope(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn client_owner_authority(v: &n::Authority) -> c::Authority {
    c::Authority {
        models: client_optional_authority_models(v.models.as_ref()),
        tools: v.tools,
        grants: {
            let mut out = List::with_capacity(v.grants.len());
            for item in &v.grants {
                out.push(client_owner_grant(item)).expect("one value per input");
            }
            out
        },
        delegation: client_owner_delegation(&v.delegation),
        spend: v.spend,
        deadline: client_optional_wall(v.deadline.as_ref()),
        notes: v.notes,
        note_resources: {
            let mut out = List::with_capacity(v.note_resources.len());
            for item in &v.note_resources {
                out.push(client_owner_resource_scope(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn form_owner_authority_executor(v: c::AuthorityExecutor) -> n::AuthorityExecutor {
    match v {
        c::AuthorityExecutor::Job(x) => n::AuthorityExecutor::Job(x),
        c::AuthorityExecutor::Charter(x) => n::AuthorityExecutor::Charter(x),
        c::AuthorityExecutor::Procedure(x) => n::AuthorityExecutor::Procedure(x),
        c::AuthorityExecutor::Role(x) => n::AuthorityExecutor::Role(x),
    }
}
pub fn client_owner_authority_executor(v: n::AuthorityExecutor) -> c::AuthorityExecutor {
    match v {
        n::AuthorityExecutor::Job(x) => c::AuthorityExecutor::Job(x),
        n::AuthorityExecutor::Charter(x) => c::AuthorityExecutor::Charter(x),
        n::AuthorityExecutor::Procedure(x) => c::AuthorityExecutor::Procedure(x),
        n::AuthorityExecutor::Role(x) => c::AuthorityExecutor::Role(x),
    }
}
pub fn form_owner_delegation(v: &c::Delegation) -> n::Delegation {
    n::Delegation {
        kinds: {
            let mut out = List::with_capacity(v.kinds.len());
            for item in &v.kinds {
                out.push(form_owner_authority_executor(*item)).expect("one value per input");
            }
            out
        },
        tasks: v.tasks,
        depth: v.depth,
    }
}
pub fn client_owner_delegation(v: &n::Delegation) -> c::Delegation {
    c::Delegation {
        kinds: {
            let mut out = List::with_capacity(v.kinds.len());
            for item in &v.kinds {
                out.push(client_owner_authority_executor(*item)).expect("one value per input");
            }
            out
        },
        tasks: v.tasks,
        depth: v.depth,
    }
}
pub fn form_owner_grant(v: &c::Grant) -> n::Grant {
    n::Grant { connector: v.connector, kind: v.kind, pattern: form_owner_pattern(&v.pattern) }
}
pub fn client_owner_grant(v: &n::Grant) -> c::Grant {
    c::Grant { connector: v.connector, kind: v.kind, pattern: client_owner_pattern(&v.pattern) }
}
pub fn form_owner_kind_implication(v: c::KindImplication) -> n::KindImplication {
    n::KindImplication { connector: v.connector, kind: v.kind, implies: v.implies }
}
pub fn client_owner_kind_implication(v: n::KindImplication) -> c::KindImplication {
    c::KindImplication { connector: v.connector, kind: v.kind, implies: v.implies }
}
pub fn form_owner_part(v: &c::Part) -> n::Part {
    n::Part { connector: v.connector, kind: form_owner_part_kind(v.kind), key: form_owner_part_key(&v.key) }
}
pub fn client_owner_part(v: &n::Part) -> c::Part {
    c::Part { connector: v.connector, kind: client_owner_part_kind(v.kind), key: client_owner_part_key(&v.key) }
}
pub fn form_owner_part_key(v: &c::PartKey) -> n::PartKey {
    match v {
        c::PartKey::Path(x) => n::PartKey::Path({
            let mut out = List::with_capacity(x.len());
            for item in x {
                out.push(item.clone()).expect("one value per input");
            }
            out
        }),
        c::PartKey::Number(x) => n::PartKey::Number(*x),
    }
}
pub fn client_owner_part_key(v: &n::PartKey) -> c::PartKey {
    match v {
        n::PartKey::Path(x) => c::PartKey::Path({
            let mut out = List::with_capacity(x.len());
            for item in x {
                out.push(item.clone()).expect("one value per input");
            }
            out
        }),
        n::PartKey::Number(x) => c::PartKey::Number(*x),
    }
}
pub fn form_owner_part_kind(v: c::PartKind) -> n::PartKind {
    match v {
        c::PartKind::Resource => n::PartKind::Resource,
        c::PartKind::Pool => n::PartKind::Pool,
        c::PartKind::Result => n::PartKind::Result,
        c::PartKind::Procedure => n::PartKind::Procedure,
        c::PartKind::Effect => n::PartKind::Effect,
        c::PartKind::Outbox => n::PartKind::Outbox,
        c::PartKind::Hold => n::PartKind::Hold,
    }
}
pub fn client_owner_part_kind(v: n::PartKind) -> c::PartKind {
    match v {
        n::PartKind::Resource => c::PartKind::Resource,
        n::PartKind::Pool => c::PartKind::Pool,
        n::PartKind::Result => c::PartKind::Result,
        n::PartKind::Procedure => c::PartKind::Procedure,
        n::PartKind::Effect => c::PartKind::Effect,
        n::PartKind::Outbox => c::PartKind::Outbox,
        n::PartKind::Hold => c::PartKind::Hold,
    }
}
pub fn form_owner_pattern(v: &c::Pattern) -> n::Pattern {
    n::Pattern {
        segments: {
            let mut out = List::with_capacity(v.segments.len());
            for item in &v.segments {
                out.push(item.clone()).expect("one value per input");
            }
            out
        },
        last: form_owner_pattern_end(&v.last),
    }
}
pub fn client_owner_pattern(v: &n::Pattern) -> c::Pattern {
    c::Pattern {
        segments: {
            let mut out = List::with_capacity(v.segments.len());
            for item in &v.segments {
                out.push(item.clone()).expect("one value per input");
            }
            out
        },
        last: client_owner_pattern_end(&v.last),
    }
}
pub fn form_owner_pattern_end(v: &c::PatternEnd) -> n::PatternEnd {
    match v {
        c::PatternEnd::Exact(x) => n::PatternEnd::Exact(x.clone()),
        c::PatternEnd::Open(x) => n::PatternEnd::Open(x.clone()),
    }
}
pub fn client_owner_pattern_end(v: &n::PatternEnd) -> c::PatternEnd {
    match v {
        n::PatternEnd::Exact(x) => c::PatternEnd::Exact(x.clone()),
        n::PatternEnd::Open(x) => c::PatternEnd::Open(x.clone()),
    }
}
pub fn form_owner_resource_scope(v: &c::ResourceScope) -> n::ResourceScope {
    n::ResourceScope { connector: v.connector, pattern: form_owner_pattern(&v.pattern) }
}
pub fn client_owner_resource_scope(v: &n::ResourceScope) -> c::ResourceScope {
    c::ResourceScope { connector: v.connector, pattern: client_owner_pattern(&v.pattern) }
}
pub fn form_policy_edit_role(v: &c::PolicyEditRole) -> n::PolicyEditRole {
    n::PolicyEditRole {
        number: v.number,
        authority: form_owner_authority(&v.authority),
        period_spend: v.period_spend,
        requests: v.requests,
        decides: v.decides,
    }
}
pub fn client_policy_edit_role(v: &n::PolicyEditRole) -> c::PolicyEditRole {
    c::PolicyEditRole {
        number: v.number,
        authority: client_owner_authority(&v.authority),
        period_spend: v.period_spend,
        requests: v.requests,
        decides: v.decides,
    }
}
pub fn form_policy_guard(v: &c::PolicyGuard) -> n::PolicyGuard {
    match v {
        c::PolicyGuard::Guarded => n::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => n::PolicyGuard::Observed { freshness: *freshness },
    }
}
pub fn client_policy_guard(v: &n::PolicyGuard) -> c::PolicyGuard {
    match v {
        n::PolicyGuard::Guarded => c::PolicyGuard::Guarded,
        n::PolicyGuard::Observed { freshness } => c::PolicyGuard::Observed { freshness: *freshness },
    }
}
pub fn form_policy_judge(v: c::PolicyJudge) -> n::PolicyJudge {
    n::PolicyJudge { connector: v.connector, requirement: v.requirement, parameters: v.parameters }
}
pub fn client_policy_judge(v: n::PolicyJudge) -> c::PolicyJudge {
    c::PolicyJudge { connector: v.connector, requirement: v.requirement, parameters: v.parameters }
}
pub fn form_policy_permission(v: c::PolicyPermission) -> n::PolicyPermission {
    n::PolicyPermission { connector: v.connector, permission: v.permission, role: v.role }
}
pub fn client_policy_permission(v: n::PolicyPermission) -> c::PolicyPermission {
    c::PolicyPermission { connector: v.connector, permission: v.permission, role: v.role }
}
pub fn form_policy_requirement(v: &c::PolicyRequirement) -> n::PolicyRequirement {
    n::PolicyRequirement {
        connector: v.connector,
        kind: v.kind,
        pattern: form_owner_pattern(&v.pattern),
        judge: form_policy_judge(v.judge),
        guard: form_policy_guard(&v.guard),
        must_be_guarded: v.must_be_guarded,
    }
}
pub fn client_policy_requirement(v: &n::PolicyRequirement) -> c::PolicyRequirement {
    c::PolicyRequirement {
        connector: v.connector,
        kind: v.kind,
        pattern: client_owner_pattern(&v.pattern),
        judge: client_policy_judge(v.judge),
        guard: client_policy_guard(&v.guard),
        must_be_guarded: v.must_be_guarded,
    }
}
pub fn form_proposal_key(v: &c::ProposalKey) -> n::ProposalKey {
    n::ProposalKey { proposer: form_proposer(&v.proposer), number: v.number }
}

pub fn form_proposer(v: &c::Proposer) -> n::Proposer {
    match v {
        c::Proposer::Task(x) => n::Proposer::Task(*x),
        c::Proposer::Party(x) => n::Proposer::Party(*x),
    }
}

pub fn form_retention(v: c::Retention) -> n::Retention {
    match v {
        c::Retention::Kept => n::Retention::Kept,
        c::Retention::ForgottenOnceRead => n::Retention::ForgottenOnceRead,
    }
}
pub fn client_retention(v: n::Retention) -> c::Retention {
    match v {
        n::Retention::Kept => c::Retention::Kept,
        n::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
    }
}
pub fn form_revision_answer(v: &c::RevisionAnswer) -> n::RevisionAnswer {
    match v {
        c::RevisionAnswer::Revision(x) => n::RevisionAnswer::Revision(form_revision_document(x)),
        c::RevisionAnswer::Forgotten(x) => n::RevisionAnswer::Forgotten(form_revision_forgotten(x)),
        c::RevisionAnswer::Unknown => n::RevisionAnswer::Unknown,
    }
}

pub fn form_revision_decider(v: &c::RevisionDecider) -> n::RevisionDecider {
    match v {
        c::RevisionDecider::FirstStart => n::RevisionDecider::FirstStart,
        c::RevisionDecider::Release => n::RevisionDecider::Release,
        c::RevisionDecider::List { provider } => n::RevisionDecider::List { provider: *provider },
        c::RevisionDecider::Party(x) => n::RevisionDecider::Party(form_revision_party(x)),
    }
}

pub fn form_revision_document(v: &c::RevisionDocument) -> n::RevisionDocument {
    n::RevisionDocument {
        head: form_revision_head(&v.head),
        items: {
            let mut out = List::with_capacity(v.items.len());
            for item in &v.items {
                out.push(form_revision_item(item)).expect("one value per input");
            }
            out
        },
    }
}

pub fn form_revision_edit(v: &c::RevisionEdit) -> n::RevisionEdit {
    match v {
        c::RevisionEdit::Application(x) => n::RevisionEdit::Application(form_sized_edit(x)),
        c::RevisionEdit::Set(x) => n::RevisionEdit::Set(form_setting_set(x)),
        c::RevisionEdit::TooLarge(x) => n::RevisionEdit::TooLarge(form_size(x)),
        c::RevisionEdit::Removed => n::RevisionEdit::Removed,
        c::RevisionEdit::Reset => n::RevisionEdit::Reset,
    }
}

pub fn form_revision_forgotten(v: &c::RevisionForgotten) -> n::RevisionForgotten {
    n::RevisionForgotten { oldest: v.oldest }
}

pub fn form_revision_head(v: &c::RevisionHead) -> n::RevisionHead {
    n::RevisionHead {
        number: v.number,
        against: v.against,
        at: v.at,
        by: form_revision_decider(&v.by),
        proposal: form_optional_proposal_key(v.proposal.as_ref()),
        moved: v.moved,
    }
}

pub fn form_revision_item(v: &c::RevisionItem) -> n::RevisionItem {
    n::RevisionItem {
        setting: form_setting(v.setting),
        edit: form_revision_edit(&v.edit),
        replaced: match &v.replaced {
            c::RevisionReplaced::Value(value) => n::RevisionReplaced::Value(form_setting_set(value)),
            c::RevisionReplaced::TooLarge(size) => n::RevisionReplaced::TooLarge(form_size(size)),
            c::RevisionReplaced::Application { size } => n::RevisionReplaced::Application { size: *size },
            c::RevisionReplaced::Default => n::RevisionReplaced::Default,
            c::RevisionReplaced::Removed => n::RevisionReplaced::Removed,
            c::RevisionReplaced::NotOffered => n::RevisionReplaced::NotOffered,
        },
    }
}

pub fn form_revision_item_edit(v: &c::RevisionItemEdit) -> n::RevisionItemEdit {
    match v {
        c::RevisionItemEdit::Set => n::RevisionItemEdit::Set,
        c::RevisionItemEdit::Removed => n::RevisionItemEdit::Removed,
        c::RevisionItemEdit::Reset => n::RevisionItemEdit::Reset,
    }
}

pub fn form_revision_item_head(v: &c::RevisionItemHead) -> n::RevisionItemHead {
    n::RevisionItemHead { setting: form_setting(v.setting), edit: form_revision_item_edit(&v.edit) }
}

pub fn form_revision_party(v: &c::RevisionParty) -> n::RevisionParty {
    n::RevisionParty { number: v.number }
}

pub fn form_revision_row(v: &c::RevisionRow) -> n::RevisionRow {
    n::RevisionRow {
        head: form_revision_head(&v.head),
        items: {
            let mut out = List::with_capacity(v.items.len());
            for item in &v.items {
                out.push(form_revision_item_head(item)).expect("one value per input");
            }
            out
        },
        more: v.more,
    }
}

pub fn form_revisions_page(v: &c::RevisionsPage) -> n::RevisionsPage {
    n::RevisionsPage {
        rows: {
            let mut out = List::with_capacity(v.rows.len());
            for item in &v.rows {
                out.push(form_revision_row(item)).expect("one value per input");
            }
            out
        },
        next: form_optional_u32(v.next.as_ref()),
        forgotten: v.forgotten,
    }
}

pub fn form_rules(v: &c::Rules) -> n::Rules {
    n::Rules {
        ceiling: form_owner_authority(&v.ceiling),
        period_spend: v.period_spend,
        minimum_run_spend: v.minimum_run_spend,
        maximum_run_spend: v.maximum_run_spend,
        implies: {
            let mut out = List::with_capacity(v.implies.len());
            for item in &v.implies {
                out.push(form_owner_kind_implication(*item)).expect("one value per input");
            }
            out
        },
        requirements: {
            let mut out = List::with_capacity(v.requirements.len());
            for item in &v.requirements {
                out.push(form_policy_requirement(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn client_rules(v: &n::Rules) -> c::Rules {
    c::Rules {
        ceiling: client_owner_authority(&v.ceiling),
        period_spend: v.period_spend,
        minimum_run_spend: v.minimum_run_spend,
        maximum_run_spend: v.maximum_run_spend,
        implies: {
            let mut out = List::with_capacity(v.implies.len());
            for item in &v.implies {
                out.push(client_owner_kind_implication(*item)).expect("one value per input");
            }
            out
        },
        requirements: {
            let mut out = List::with_capacity(v.requirements.len());
            for item in &v.requirements {
                out.push(client_policy_requirement(item)).expect("one value per input");
            }
            out
        },
    }
}

pub fn form_setting(v: c::Setting) -> n::Setting {
    match v {
        c::Setting::Charter(x) => n::Setting::Charter(form_setting_number(x)),
        c::Setting::Template(x) => n::Setting::Template(form_setting_number(x)),
        c::Setting::ProjectPolicy { project } => n::Setting::ProjectPolicy { project },
        c::Setting::Policy(x) => n::Setting::Policy(form_setting_number(x)),
        c::Setting::Application { kind, scope, number } => n::Setting::Application { kind, scope, number },
        c::Setting::Model(x) => n::Setting::Model(form_setting_number(x)),
        c::Setting::Rules => n::Setting::Rules,
    }
}
pub fn client_setting(v: n::Setting) -> c::Setting {
    match v {
        n::Setting::Charter(x) => c::Setting::Charter(client_setting_number(x)),
        n::Setting::Template(x) => c::Setting::Template(client_setting_number(x)),
        n::Setting::Policy(x) => c::Setting::Policy(client_setting_number(x)),
        n::Setting::Application { kind, scope, number } => c::Setting::Application { kind, scope, number },
        n::Setting::Model(x) => c::Setting::Model(client_setting_number(x)),
        n::Setting::Rules => c::Setting::Rules,
        n::Setting::ProjectPolicy { project } => c::Setting::ProjectPolicy { project },
    }
}
pub fn form_setting_document(v: &c::SettingDocument) -> n::SettingDocument {
    n::SettingDocument {
        overlay: match &v.overlay {
            Some(v) => Some(Box::new(form_overlay(v))),
            None => None,
        },
        setting: form_setting(v.setting),
        revision: v.revision,
        value: match &v.value {
            Some(v) => Some(form_setting_value(v)),
            None => None,
        },
        application: match &v.application {
            Some(v) => Some(form_application_state(v)),
            None => None,
        },
        proposed: match &v.proposed {
            Some(v) => Some(form_sized_edit(v)),
            None => None,
        },
    }
}

pub fn client_setting_edit(v: &n::SettingEdit) -> c::SettingEdit {
    match v {
        n::SettingEdit::Set(x) => c::SettingEdit::Set(client_setting_set(x)),
        n::SettingEdit::Removed => c::SettingEdit::Removed,
        n::SettingEdit::Reset => c::SettingEdit::Reset,
    }
}
pub fn form_setting_fault(v: c::SettingFault) -> n::SettingFault {
    match v {
        c::SettingFault::Shape => n::SettingFault::Shape,
        c::SettingFault::Count => n::SettingFault::Count,
        c::SettingFault::Reference => n::SettingFault::Reference,
        c::SettingFault::Rules => n::SettingFault::Rules,
        c::SettingFault::Budget => n::SettingFault::Budget,
        c::SettingFault::Grace => n::SettingFault::Grace,
        c::SettingFault::Judged => n::SettingFault::Judged,
        c::SettingFault::Owner => n::SettingFault::Owner,
    }
}

pub fn form_setting_follows(v: &c::SettingFollows) -> n::SettingFollows {
    match v {
        c::SettingFollows::Overlay => n::SettingFollows::Overlay,
        c::SettingFollows::Default => n::SettingFollows::Default,
        c::SettingFollows::Change => n::SettingFollows::Change,
    }
}

pub fn form_setting_hosts(v: &c::SettingHosts) -> n::SettingHosts {
    match v {
        c::SettingHosts::Workers => n::SettingHosts::Workers,
        c::SettingHosts::Engine => n::SettingHosts::Engine,
        c::SettingHosts::Both => n::SettingHosts::Both,
    }
}
pub fn client_setting_hosts(v: &n::SettingHosts) -> c::SettingHosts {
    match v {
        n::SettingHosts::Workers => c::SettingHosts::Workers,
        n::SettingHosts::Engine => c::SettingHosts::Engine,
        n::SettingHosts::Both => c::SettingHosts::Both,
    }
}

pub fn client_setting_item(v: &n::SettingItem) -> c::SettingItem {
    c::SettingItem { setting: client_setting(v.setting), edit: client_setting_edit(&v.edit) }
}
pub fn form_setting_line(v: &c::SettingLine) -> n::SettingLine {
    n::SettingLine {
        setting: form_setting(v.setting),
        name: form_optional_document_text(v.name.as_ref()),
        follows: form_setting_follows(&v.follows),
        last: v.last,
        application: match &v.application {
            Some(v) => Some(form_application_state(v)),
            None => None,
        },
    }
}

pub fn form_setting_model(v: &c::SettingModel) -> n::SettingModel {
    n::SettingModel {
        provider: v.provider,
        dialect: v.dialect,
        endpoint: v.endpoint,
        program: form_optional_text(v.program.as_ref()),
        name: form_text(&v.name),
        window: v.window,
        output: v.output,
        input_price: v.input_price,
        cached_price: v.cached_price,
        cache_write_price: form_optional_u64(v.cache_write_price.as_ref()),
        output_price: v.output_price,
        price_unit: v.price_unit,
        efforts: {
            let mut out = List::with_capacity(v.efforts.len());
            for item in &v.efforts {
                out.push(form_text(item)).expect("one value per input");
            }
            out
        },
        effort: form_optional_text(v.effort.as_ref()),
    }
}
pub fn client_setting_model(v: &n::SettingModel) -> c::SettingModel {
    c::SettingModel {
        provider: v.provider,
        dialect: v.dialect,
        endpoint: v.endpoint,
        program: client_optional_text(v.program.as_ref()),
        name: client_text(&v.name),
        window: v.window,
        output: v.output,
        input_price: v.input_price,
        cached_price: v.cached_price,
        cache_write_price: client_optional_u64(v.cache_write_price.as_ref()),
        output_price: v.output_price,
        price_unit: v.price_unit,
        efforts: {
            let mut out = List::with_capacity(v.efforts.len());
            for item in &v.efforts {
                out.push(client_text(item)).expect("one value per input");
            }
            out
        },
        effort: client_optional_text(v.effort.as_ref()),
    }
}
pub fn form_setting_number(v: c::SettingNumber) -> n::SettingNumber {
    n::SettingNumber { number: v.number }
}
pub fn client_setting_number(v: n::SettingNumber) -> c::SettingNumber {
    c::SettingNumber { number: v.number }
}
pub fn form_setting_policy(v: &c::SettingPolicy) -> n::SettingPolicy {
    n::SettingPolicy {
        escalation_role: form_optional_u32(v.escalation_role.as_ref()),
        ceiling: form_owner_authority(&v.ceiling),
        projections: {
            let mut out = List::with_capacity(v.projections.len());
            for item in &v.projections {
                out.push(form_owner_grant(item)).expect("one value per input");
            }
            out
        },
        period_spend: v.period_spend,
        roles: {
            let mut out = List::with_capacity(v.roles.len());
            for item in &v.roles {
                out.push(form_policy_edit_role(item)).expect("one value per input");
            }
            out
        },
        requirements: {
            let mut out = List::with_capacity(v.requirements.len());
            for item in &v.requirements {
                out.push(form_policy_requirement(item)).expect("one value per input");
            }
            out
        },
        permissions: {
            let mut out = List::with_capacity(v.permissions.len());
            for item in &v.permissions {
                out.push(form_policy_permission(*item)).expect("one value per input");
            }
            out
        },
        templates: {
            let mut out = List::with_capacity(v.templates.len());
            for item in &v.templates {
                out.push(form_template_override(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn client_setting_policy(v: &n::SettingPolicy) -> c::SettingPolicy {
    c::SettingPolicy {
        escalation_role: client_optional_u32(v.escalation_role.as_ref()),
        ceiling: client_owner_authority(&v.ceiling),
        projections: {
            let mut out = List::with_capacity(v.projections.len());
            for item in &v.projections {
                out.push(client_owner_grant(item)).expect("one value per input");
            }
            out
        },
        period_spend: v.period_spend,
        roles: {
            let mut out = List::with_capacity(v.roles.len());
            for item in &v.roles {
                out.push(client_policy_edit_role(item)).expect("one value per input");
            }
            out
        },
        requirements: {
            let mut out = List::with_capacity(v.requirements.len());
            for item in &v.requirements {
                out.push(client_policy_requirement(item)).expect("one value per input");
            }
            out
        },
        permissions: {
            let mut out = List::with_capacity(v.permissions.len());
            for item in &v.permissions {
                out.push(client_policy_permission(*item)).expect("one value per input");
            }
            out
        },
        templates: {
            let mut out = List::with_capacity(v.templates.len());
            for item in &v.templates {
                out.push(client_template_override(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn form_setting_set(v: &c::SettingSet) -> n::SettingSet {
    n::SettingSet { value: form_setting_value(&v.value) }
}
pub fn client_setting_set(v: &n::SettingSet) -> c::SettingSet {
    c::SettingSet { value: client_setting_value(&v.value) }
}
pub fn form_setting_value(v: &c::SettingValue) -> n::SettingValue {
    match v {
        c::SettingValue::Overlay(x) => n::SettingValue::Overlay(Box::new(form_overlay(x))),
        c::SettingValue::Model(x) => n::SettingValue::Model(Box::new(form_catalogue(x))),
        c::SettingValue::Application(x) => n::SettingValue::Application(x.clone()),
        c::SettingValue::Charter(x) => n::SettingValue::Charter(Box::new(form_charter(x))),
        c::SettingValue::Template(x) => n::SettingValue::Template(Box::new(form_template(x))),
        c::SettingValue::Policy(x) => n::SettingValue::Policy(Box::new(form_setting_policy(x))),
        c::SettingValue::Rules(x) => n::SettingValue::Rules(Box::new(form_rules(x))),
    }
}
pub fn client_setting_value(v: &n::SettingValue) -> c::SettingValue {
    match v {
        n::SettingValue::Overlay(x) => c::SettingValue::Overlay(Box::new(client_overlay(x))),
        n::SettingValue::Model(x) => c::SettingValue::Model(Box::new(client_catalogue(x))),
        n::SettingValue::Application(x) => c::SettingValue::Application(x.clone()),
        n::SettingValue::Charter(x) => c::SettingValue::Charter(Box::new(client_charter(x))),
        n::SettingValue::Template(x) => c::SettingValue::Template(Box::new(client_template(x))),
        n::SettingValue::Policy(x) => c::SettingValue::Policy(Box::new(client_setting_policy(x))),
        n::SettingValue::Rules(x) => c::SettingValue::Rules(Box::new(client_rules(x))),
    }
}
pub fn form_settings_document(v: &c::SettingsDocument) -> n::SettingsDocument {
    n::SettingsDocument {
        newest: v.newest,
        settings: {
            let mut out = List::with_capacity(v.settings.len());
            for item in &v.settings {
                out.push(form_setting_line(item)).expect("one value per input");
            }
            out
        },
        held: {
            let mut out = List::with_capacity(v.held.len());
            for item in &v.held {
                out.push(form_held_revision(item)).expect("one value per input");
            }
            out
        },
        pending: {
            let mut out = List::with_capacity(v.pending.len());
            for item in &v.pending {
                out.push(form_settings_proposal(item)).expect("one value per input");
            }
            out
        },
    }
}

pub fn form_settings_proposal(v: &c::SettingsProposal) -> n::SettingsProposal {
    n::SettingsProposal { proposer: form_proposer(&v.proposer), number: v.number }
}

pub fn form_size(v: &c::Size) -> n::Size {
    n::Size { bytes: v.bytes }
}

pub fn form_task_contract_document(v: &c::TaskContractDocument) -> n::TaskContractDocument {
    match v {
        c::TaskContractDocument::Approval { state, part } => {
            n::TaskContractDocument::Approval { state: state.clone(), part: form_owner_part(part) }
        }
        c::TaskContractDocument::Job { bytes } => n::TaskContractDocument::Job { bytes: *bytes },
        c::TaskContractDocument::Report { words } => n::TaskContractDocument::Report { words: *words },
        c::TaskContractDocument::Verdict { choices, cut } => n::TaskContractDocument::Verdict {
            choices: {
                let mut out = List::with_capacity(choices.len());
                for item in choices {
                    out.push(form_verdict_choice(item)).expect("one value per input");
                }
                out
            },
            cut: *cut,
        },
        c::TaskContractDocument::Change { connector, kind, words } => {
            n::TaskContractDocument::Change { connector: *connector, kind: *kind, words: *words }
        }
    }
}
pub fn client_task_contract_document(v: &n::TaskContractDocument) -> c::TaskContractDocument {
    match v {
        n::TaskContractDocument::Approval { state, part } => {
            c::TaskContractDocument::Approval { state: state.clone(), part: client_owner_part(part) }
        }
        n::TaskContractDocument::Job { bytes } => c::TaskContractDocument::Job { bytes: *bytes },
        n::TaskContractDocument::Report { words } => c::TaskContractDocument::Report { words: *words },
        n::TaskContractDocument::Verdict { choices, cut } => c::TaskContractDocument::Verdict {
            choices: {
                let mut out = List::with_capacity(choices.len());
                for item in choices {
                    out.push(client_verdict_choice(item)).expect("one value per input");
                }
                out
            },
            cut: *cut,
        },
        n::TaskContractDocument::Change { connector, kind, words } => {
            c::TaskContractDocument::Change { connector: *connector, kind: *kind, words: *words }
        }
    }
}
pub fn form_task_flags(v: c::TaskFlags) -> n::TaskFlags {
    n::TaskFlags {
        conversation: v.conversation,
        listed: v.listed,
        retention: form_retention(v.retention),
        ended_by: form_ended_by(v.ended_by),
    }
}
pub fn client_task_flags(v: n::TaskFlags) -> c::TaskFlags {
    c::TaskFlags {
        conversation: v.conversation,
        listed: v.listed,
        retention: client_retention(v.retention),
        ended_by: client_ended_by(v.ended_by),
    }
}
pub fn form_template(v: &c::Template) -> n::Template {
    n::Template {
        resources: {
            let mut out = List::with_capacity(v.resources.len());
            for item in &v.resources {
                out.push(n::Symbol { connector: item.connector, symbol: item.symbol }).expect("one value per input");
            }
            out
        },
        number: v.number,
        name: form_document_text(&v.name),
        executor: form_template_executor(v.executor),
        contract: form_task_contract_document(&v.contract),
        shallower: v.shallower,
        budget: form_template_budget(&v.budget),
        deadline: form_template_deadline(&v.deadline),
        tracked: form_optional_u32(v.tracked.as_ref()),
        flags: form_task_flags(v.flags),
        label: v.label,
        grace: form_optional_duration(v.grace.as_ref()),
        llm: form_optional_super_llm_choice(v.llm.as_ref()),
    }
}
pub fn client_template(v: &n::Template) -> c::Template {
    c::Template {
        resources: {
            let mut out = List::with_capacity(v.resources.len());
            for item in &v.resources {
                out.push(c::Symbol { connector: item.connector, symbol: item.symbol }).expect("one value per input");
            }
            out
        },
        number: v.number,
        name: client_document_text(&v.name),
        executor: client_template_executor(v.executor),
        contract: client_task_contract_document(&v.contract),
        shallower: v.shallower,
        budget: client_template_budget(&v.budget),
        deadline: client_template_deadline(&v.deadline),
        tracked: client_optional_u32(v.tracked.as_ref()),
        flags: client_task_flags(v.flags),
        label: v.label,
        grace: client_optional_duration(v.grace.as_ref()),
        llm: client_optional_super_llm_choice(v.llm.as_ref()),
    }
}
pub fn form_template_budget(v: &c::TemplateBudget) -> n::TemplateBudget {
    match v {
        c::TemplateBudget::Amount(x) => n::TemplateBudget::Amount(*x),
        c::TemplateBudget::AsAllowed { cap } => n::TemplateBudget::AsAllowed { cap: form_optional_u64(cap.as_ref()) },
        c::TemplateBudget::Given => n::TemplateBudget::Given,
    }
}
pub fn client_template_budget(v: &n::TemplateBudget) -> c::TemplateBudget {
    match v {
        n::TemplateBudget::Amount(x) => c::TemplateBudget::Amount(*x),
        n::TemplateBudget::AsAllowed { cap } => c::TemplateBudget::AsAllowed { cap: client_optional_u64(cap.as_ref()) },
        n::TemplateBudget::Given => c::TemplateBudget::Given,
    }
}
pub fn form_template_deadline(v: &c::TemplateDeadline) -> n::TemplateDeadline {
    match v {
        c::TemplateDeadline::None => n::TemplateDeadline::None,
        c::TemplateDeadline::Span(x) => n::TemplateDeadline::Span(*x),
        c::TemplateDeadline::Given => n::TemplateDeadline::Given,
    }
}
pub fn client_template_deadline(v: &n::TemplateDeadline) -> c::TemplateDeadline {
    match v {
        n::TemplateDeadline::None => c::TemplateDeadline::None,
        n::TemplateDeadline::Span(x) => c::TemplateDeadline::Span(*x),
        n::TemplateDeadline::Given => c::TemplateDeadline::Given,
    }
}
pub fn form_template_executor(v: c::TemplateExecutor) -> n::TemplateExecutor {
    match v {
        c::TemplateExecutor::Charter(x) => n::TemplateExecutor::Charter(x),
        c::TemplateExecutor::Chosen => n::TemplateExecutor::Chosen,
    }
}
pub fn client_template_executor(v: n::TemplateExecutor) -> c::TemplateExecutor {
    match v {
        n::TemplateExecutor::Charter(x) => c::TemplateExecutor::Charter(x),
        n::TemplateExecutor::Chosen => c::TemplateExecutor::Chosen,
    }
}
pub fn form_template_override(v: &c::TemplateOverride) -> n::TemplateOverride {
    n::TemplateOverride {
        template: v.template,
        executor: form_optional_template_executor(v.executor.as_ref()),
        budget: form_optional_template_budget(v.budget.as_ref()),
        deadline: form_optional_template_deadline(v.deadline.as_ref()),
        priority: form_optional_u32(v.priority.as_ref()),
    }
}
pub fn client_template_override(v: &n::TemplateOverride) -> c::TemplateOverride {
    c::TemplateOverride {
        template: v.template,
        executor: client_optional_template_executor(v.executor.as_ref()),
        budget: client_optional_template_budget(v.budget.as_ref()),
        deadline: client_optional_template_deadline(v.deadline.as_ref()),
        priority: client_optional_u32(v.priority.as_ref()),
    }
}
pub fn form_text(v: &c::Text) -> n::Text {
    n::Text::new(v.bytes().into(), u32::MAX).expect("already validated text")
}
pub fn client_text(v: &n::Text) -> c::Text {
    c::Text::new(v.bytes().into(), u32::MAX).expect("already validated text")
}
pub fn form_verdict_choice(v: &c::VerdictChoice) -> n::VerdictChoice {
    n::VerdictChoice {
        code: v.code,
        words: v.words,
        followups: v.followups,
        fields: {
            let mut out = List::with_capacity(v.fields.len());
            for item in &v.fields {
                out.push(form_field_rule(item)).expect("one value per input");
            }
            out
        },
    }
}
pub fn client_verdict_choice(v: &n::VerdictChoice) -> c::VerdictChoice {
    c::VerdictChoice {
        code: v.code,
        words: v.words,
        followups: v.followups,
        fields: {
            let mut out = List::with_capacity(v.fields.len());
            for item in &v.fields {
                out.push(client_field_rule(item)).expect("one value per input");
            }
            out
        },
    }
}
fn form_optional_u32(v: Option<&u32>) -> Option<u32> {
    let v = v?;
    Some(*v)
}
fn client_optional_u32(v: Option<&u32>) -> Option<u32> {
    let v = v?;
    Some(*v)
}
fn form_optional_text(v: Option<&c::Text>) -> Option<n::Text> {
    let v = v?;
    Some(form_text(v))
}
fn client_optional_text(v: Option<&n::Text>) -> Option<c::Text> {
    let v = v?;
    Some(client_text(v))
}
fn form_optional_authority_models(v: Option<&c::AuthorityModels>) -> Option<n::AuthorityModels> {
    let v = v?;
    Some(form_authority_models(v))
}
fn form_optional_wall(v: Option<&Wall>) -> Option<Wall> {
    let v = v?;
    Some(*v)
}
fn client_optional_authority_models(v: Option<&n::AuthorityModels>) -> Option<c::AuthorityModels> {
    let v = v?;
    Some(client_authority_models(v))
}
fn client_optional_wall(v: Option<&Wall>) -> Option<Wall> {
    let v = v?;
    Some(*v)
}
fn form_optional_proposal_key(v: Option<&c::ProposalKey>) -> Option<n::ProposalKey> {
    let v = v?;
    Some(form_proposal_key(v))
}

fn form_optional_document_text(v: Option<&c::DocumentText>) -> Option<n::DocumentText> {
    let v = v?;
    Some(form_document_text(v))
}

fn form_optional_u64(v: Option<&u64>) -> Option<u64> {
    let v = v?;
    Some(*v)
}
fn client_optional_u64(v: Option<&u64>) -> Option<u64> {
    let v = v?;
    Some(*v)
}
fn form_optional_duration(v: Option<&Duration>) -> Option<Duration> {
    let v = v?;
    Some(*v)
}
fn form_optional_super_llm_choice(v: Option<&c::LlmChoice>) -> Option<n::LlmChoice> {
    let v = v?;
    Some(form_llm_choice(v))
}
fn client_optional_duration(v: Option<&Duration>) -> Option<Duration> {
    let v = v?;
    Some(*v)
}
fn client_optional_super_llm_choice(v: Option<&n::LlmChoice>) -> Option<c::LlmChoice> {
    let v = v?;
    Some(client_llm_choice(v))
}
fn form_optional_template_executor(v: Option<&c::TemplateExecutor>) -> Option<n::TemplateExecutor> {
    let v = v?;
    Some(form_template_executor(*v))
}
fn form_optional_template_budget(v: Option<&c::TemplateBudget>) -> Option<n::TemplateBudget> {
    let v = v?;
    Some(form_template_budget(v))
}
fn form_optional_template_deadline(v: Option<&c::TemplateDeadline>) -> Option<n::TemplateDeadline> {
    let v = v?;
    Some(form_template_deadline(v))
}
fn client_optional_template_executor(v: Option<&n::TemplateExecutor>) -> Option<c::TemplateExecutor> {
    let v = v?;
    Some(client_template_executor(*v))
}
fn client_optional_template_budget(v: Option<&n::TemplateBudget>) -> Option<c::TemplateBudget> {
    let v = v?;
    Some(client_template_budget(v))
}
fn client_optional_template_deadline(v: Option<&n::TemplateDeadline>) -> Option<c::TemplateDeadline> {
    let v = v?;
    Some(client_template_deadline(v))
}

pub fn limits(v: &crate::jig::Limits) -> settings_form::Limits {
    settings_form::Limits {
        settings: v.settings,
        kinds: settings_form::Limits::standard().kinds,
        ask_bytes: v.ask_bytes,
        result_bytes: v.result_bytes,
        text: v.words,
        rows: 32,
        // Eighteen executor places size each receiving authority list.
        authority: 18,
        fields: 16,
    }
}
pub fn form_read_value(v: &c::ReadValue) -> n::ReadValue {
    match v {
        c::ReadValue::Settings(x) => n::ReadValue::Settings(Box::new(form_settings_document(x))),
        c::ReadValue::Setting(x) => n::ReadValue::Setting(Box::new(form_setting_document(x))),
        c::ReadValue::Revision(x) => n::ReadValue::Revision(Box::new(form_revision_answer(x))),
        c::ReadValue::Revisions(x) => n::ReadValue::Revisions(Box::new(form_revisions_page(x))),
        c::ReadValue::TaskText(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Task(_)
        | c::ReadValue::Proposal(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::EndedPage(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_) => unreachable!("selected settings read"),
    }
}

fn form_wake(v: c::WakePolicy) -> n::WakePolicy {
    n::WakePolicy {
        words: form_wake_rule(v.words),
        notices: form_wake_rule(v.notices),
        news: form_wake_rule(v.news),
        results: match v.results {
            c::ResultsWake::Never => n::ResultsWake::Never,
            c::ResultsWake::Each => n::ResultsWake::Each,
            c::ResultsWake::LastOrFailure => n::ResultsWake::LastOrFailure,
        },
        questions: v.questions,
        answers: v.answers,
        timers: v.timers,
    }
}
fn form_wake_rule(v: c::WakeRule) -> n::WakeRule {
    match v {
        c::WakeRule::Never => n::WakeRule::Never,
        c::WakeRule::Immediate => n::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => n::WakeRule::Batch { count, age },
        c::WakeRule::Paced { span } => n::WakeRule::Paced { span },
    }
}

fn client_wake(v: n::WakePolicy) -> c::WakePolicy {
    c::WakePolicy {
        words: client_wake_rule(v.words),
        notices: client_wake_rule(v.notices),
        news: client_wake_rule(v.news),
        results: match v.results {
            n::ResultsWake::Never => c::ResultsWake::Never,
            n::ResultsWake::Each => c::ResultsWake::Each,
            n::ResultsWake::LastOrFailure => c::ResultsWake::LastOrFailure,
        },
        questions: v.questions,
        answers: v.answers,
        timers: v.timers,
    }
}
fn client_wake_rule(v: n::WakeRule) -> c::WakeRule {
    match v {
        n::WakeRule::Never => c::WakeRule::Never,
        n::WakeRule::Immediate => c::WakeRule::Immediate,
        n::WakeRule::Batch { count, age } => c::WakeRule::Batch { count, age },
        n::WakeRule::Paced { span } => c::WakeRule::Paced { span },
    }
}

fn form_sized_edit(v: &c::SizedEdit) -> n::SizedEdit {
    match v {
        c::SizedEdit::Set { size } => n::SizedEdit::Set { size: *size },
        c::SizedEdit::Remove => n::SizedEdit::Remove,
        c::SizedEdit::Reset => n::SizedEdit::Reset,
    }
}
fn form_application_state(v: &c::ApplicationState) -> n::ApplicationState {
    n::ApplicationState {
        follows: match v.follows {
            c::ApplicationFollows::Default => n::ApplicationFollows::Default,
            c::ApplicationFollows::Changed { revision } => n::ApplicationFollows::Changed { revision },
            c::ApplicationFollows::Removed { revision } => n::ApplicationFollows::Removed { revision },
        },
        size: v.size,
        charters: v.charters.clone(),
    }
}

/// Translate the document refusal facts without checking the settings again.
#[must_use]
pub fn form_setting_detail(value: c::SettingDetail) -> n::SettingDetail {
    n::SettingDetail {
        member: match value.member {
            c::SettingMember::Documents => n::SettingMember::Documents,
            c::SettingMember::NameSegments => n::SettingMember::NameSegments,
            c::SettingMember::NameSegmentBytes => n::SettingMember::NameSegmentBytes,
            c::SettingMember::Path => n::SettingMember::Path,
            c::SettingMember::Heading => n::SettingMember::Heading,
            c::SettingMember::Connector => n::SettingMember::Connector,
            c::SettingMember::StoredPolicy => n::SettingMember::StoredPolicy,
        },
        rule: match value.rule {
            c::SettingRule::Maximum => n::SettingRule::Maximum,
            c::SettingRule::Minimum => n::SettingRule::Minimum,
            c::SettingRule::Registered => n::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}

fn form_catalogue(v: &c::CatalogueEntry) -> n::CatalogueEntry {
    n::CatalogueEntry {
        number: v.number,
        model: form_setting_model(&v.model),
        shown: match &v.shown {
            Some(value) => Some(form_text(value)),
            None => None,
        },
        description: match &v.description {
            Some(value) => Some(form_text(value)),
            None => None,
        },
        picker: v.picker,
        withdrawn: v.withdrawn,
        estimated: v.estimated,
        source: match v.source {
            c::ModelSource::Stock => n::ModelSource::Stock,
            c::ModelSource::List => n::ModelSource::List,
            c::ModelSource::Person => n::ModelSource::Person,
        },
    }
}

fn client_catalogue(v: &n::CatalogueEntry) -> c::CatalogueEntry {
    c::CatalogueEntry {
        number: v.number,
        model: client_setting_model(&v.model),
        shown: match &v.shown {
            Some(value) => Some(client_text(value)),
            None => None,
        },
        description: match &v.description {
            Some(value) => Some(client_text(value)),
            None => None,
        },
        picker: v.picker,
        withdrawn: v.withdrawn,
        estimated: v.estimated,
        source: match v.source {
            n::ModelSource::Stock => c::ModelSource::Stock,
            n::ModelSource::List => c::ModelSource::List,
            n::ModelSource::Person => c::ModelSource::Person,
        },
    }
}
