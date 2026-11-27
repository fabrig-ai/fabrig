//! Whole settings asks into core values (client/domain/contract/requests.md, 2).
use super::super::{Untranslatable, asks, operations, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_authority as a;
use jig_domain_core_vocabulary::{self as c, settings as s};
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{List, Reader};
fn bad() -> Untranslatable {
    Untranslatable { path: w::paths::CHANGE_SETTINGS_ITEMS }
}
pub(crate) fn change(
    value: w::ChangeSettings,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<s::Change, Untranslatable> {
    let v = value;
    let count = asks::count(&v.items, w::paths::CHANGE_SETTINGS_ITEMS)?;
    let mut items = List::with_capacity(count);
    let mut reader = Reader::new(&v.items);
    for _ in 0..count {
        let bytes = asks::item(&mut reader, w::paths::CHANGE_SETTINGS_ITEMS)?;
        let mut piece = Reader::new(&bytes);
        let item = match w::SettingItem::decode(limits, &mut piece) {
            Ok(value) => value,
            Err(problem) => return Err(map_problem(problem)),
        };
        if !piece.is_empty() {
            return Err(bad());
        }
        let item = match mapped::setting_item_to_domain(item, &(models, limits)) {
            Ok(item) => item,
            Err(problem) => return Err(map_problem(problem)),
        };
        items.push(item).expect("counted pieces");
    }
    Ok(s::Change { against: v.against, items: items.into_boxed(), running: mapped::running_to_domain(v.running) })
}
fn map_problem(problem: w::Problem) -> Untranslatable {
    Untranslatable { path: problem.path }
}
pub(crate) fn model_from_record(value: w::SettingModel) -> c::Model {
    let v = value;
    c::Model {
        provider: v.provider,
        dialect: v.dialect,
        endpoint: v.endpoint,
        program: v.program,
        name: v.name,
        cache_life_ms: 0,
        window: v.window,
        output: v.output,
        input_price: v.input_price,
        cached_price: v.cached_price,
        cache_write_price: v.cache_write_price,
        output_price: v.output_price,
        price_unit: v.price_unit,
        efforts: v.efforts.into_boxed(),
        effort: v.effort,
        speeds: Box::default(),
        speed: None,
    }
}
fn wake_rule(value: w::WakeRule) -> t::WakeRule {
    match value {
        w::WakeRule::Never => t::WakeRule::Never,
        w::WakeRule::Immediate => t::WakeRule::Immediate,
        w::WakeRule::Batch(value) => t::WakeRule::Batch { count: value.count, age: value.age },
        w::WakeRule::Paced(value) => t::WakeRule::Paced { span: value.span },
    }
}
fn wake(value: w::WakePolicy) -> t::WakePolicy {
    t::WakePolicy {
        words: wake_rule(value.words),
        notices: wake_rule(value.notices),
        news: wake_rule(value.news),
        results: match value.results {
            w::ResultsWake::Never => t::ResultsWake::Never,
            w::ResultsWake::Each => t::ResultsWake::Each,
            w::ResultsWake::LastOrFailure => t::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}
pub(crate) fn charter_from_record(value: w::Charter) -> c::Charter {
    let rest = value.rest;
    let resume = value.resume;
    let inspect = value.inspect;
    let modify = value.modify;
    let shell = value.shell;
    let agents = value.agents;
    let v = value;
    let mut alternatives = List::with_capacity(v.alternatives.len());
    for value in v.alternatives.into_boxed() {
        alternatives.push(model_from_record(value)).expect("same count");
    }
    let mut documents = List::with_capacity(v.documents.len());
    for document in v.documents.into_boxed() {
        documents.push(charter_document_from_record(document)).expect("same count");
    }
    let mut parts = List::with_capacity(v.parts.len());
    for item in v.parts.into_boxed() {
        parts.push(mapped::charter_part_to_domain(item)).expect("same count");
    }
    c::Charter {
        parts: parts.into_boxed(),
        rest,
        documents: documents.into_boxed(),
        number: v.number,
        hosts: mapped::setting_hosts_to_domain(v.hosts),
        spend: v.spend,
        wake: wake(v.wake),
        policy: c::RunPolicy {
            instructions: v.instructions,
            waiting: v.waiting,
            resume,
            turns: v.turns,
            time: v.time,
            reserve_turns: v.reserve_turns,
            reserve_time: v.reserve_time,
            model: model_from_record(v.model),
            alternatives: alternatives.into_boxed(),
            inspect,
            modify,
            shell,
            agents,
            call_timeout: v.call_timeout,
            tools: crate::translate::convert::tools_to_domain(v.tools),
        },
    }
}
pub(crate) fn charter_document_from_record(value: w::CharterDocument) -> c::Document {
    c::Document {
        connector: value.connector,
        name: a::Name { segments: value.name.into_boxed() },
        path: value.path,
        heading: value.heading,
    }
}
fn authority(value: w::Authority, models: &[crate::Model], limits: &w::Limits) -> Result<a::Authority, Untranslatable> {
    let mut names = List::with_capacity(p::MODELS);
    let mut value = asks::authority(value, limits, &mut names)?;
    if let Some(selected) = value.models
        && !selected.is_all()
    {
        let mut mask = 0_u64;
        for (at, name) in names.as_slice().iter().enumerate() {
            if selected.bits() & (1_u64.checked_shl(u32::try_from(at).expect("bounded model index")).unwrap_or(0)) == 0
            {
                continue;
            }
            let mut found = false;
            for (to, model) in models.iter().enumerate() {
                if model.name == *name {
                    mask |= 1_u64.checked_shl(u32::try_from(to).expect("bounded model index")).unwrap_or(0);
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(bad());
            }
        }
        value.models = Some(p::Models::Named(mask));
    }
    c::people_authority(value).ok_or(bad())
}
fn requirements(values: List<w::PolicyRequirement>) -> Box<[a::Requirement]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(c::requirement_to_authority(asks::requirement(value))).expect("same count");
    }
    result.into_boxed()
}
pub(crate) fn policy_from_record(
    value: w::SettingPolicy,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<c::ConfiguredPolicy, Untranslatable> {
    let v = value;
    let mut roles = List::with_capacity(v.roles.len());
    for value in v.roles.into_boxed() {
        let v = value;
        roles
            .push(a::Role {
                number: v.number,
                authority: authority(v.authority, models, limits)?,
                period_spend: v.period_spend,
                requests: crate::translate::convert::requests_to_domain(v.requests),
                decides: crate::translate::convert::proposals_to_domain(v.decides),
            })
            .expect("same count");
    }
    let mut grants = List::with_capacity(v.projections.len());
    for value in v.projections.into_boxed() {
        grants.push(asks::grant(value)).expect("same count");
    }
    let mut permissions = List::with_capacity(v.permissions.len());
    for value in v.permissions.into_boxed() {
        permissions.push(mapped::policy_permission_to_domain(value)).expect("same count");
    }
    let overrides = operations::overrides(v.templates);
    Ok(c::ConfiguredPolicy {
        policy: a::Policy {
            escalation_role: v.escalation_role,
            ceiling: authority(v.ceiling, models, limits)?,
            projections: c::grants_to_authority(grants.into_boxed()).ok_or(bad())?,
            period_spend: v.period_spend,
            roles: roles.into_boxed(),
            requirements: requirements(v.requirements),
        },
        permissions: permissions.into_boxed(),
        templates: overrides,
    })
}
pub(crate) fn rules_from_record(
    value: w::Rules,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<a::Rules, Untranslatable> {
    let v = value;
    let mut implies = List::with_capacity(v.implies.len());
    for value in v.implies.into_boxed() {
        implies
            .push(a::Implication { connector: value.connector, kind: value.kind, implies: value.implies })
            .expect("same count");
    }
    let count = implies.len();
    Ok(a::Rules {
        ceiling: authority(v.ceiling, models, limits)?,
        period_spend: v.period_spend,
        minimum_run_spend: v.minimum_run_spend,
        maximum_run_spend: v.maximum_run_spend,
        implies: a::Implies::new(implies.into_boxed(), count).ok_or(bad())?,
        requirements: requirements(v.requirements),
    })
}
pub(crate) fn template_from_record(
    value: w::Template,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<c::Template, Untranslatable> {
    let v = value;
    if v.name.cut {
        return Err(bad());
    }
    let flags = v.flags;
    let llm = match super::super::llm::people(v.llm, models, w::paths::TEMPLATE_LLM)? {
        Some(value) => {
            let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("bounded models"));
            for value in value.subagents {
                subagents.push(t::ModelEffort { model: value.model, effort: value.effort }).expect("same count");
            }
            Some(t::LlmChoice {
                later: match value.later {
                    Some(later) => Some(task_later(later)),
                    None => None,
                },
                account: value.account,
                main: t::ModelEffort { model: value.main.model, effort: value.main.effort },
                subagents: subagents.into_boxed(),
            })
        }
        None => None,
    };
    let mut resources = List::with_capacity(v.resources.len());
    for symbol in v.resources.into_boxed() {
        resources
            .push(jig_domain_people::Symbol { connector: symbol.connector, symbol: symbol.symbol })
            .expect("same capacity");
    }
    Ok(c::Template {
        resources: resources.into_boxed(),
        number: v.number,
        name: v.name.bytes,
        executor: operations::executor(v.executor),
        contract: contract(v.contract, limits)?,
        shallower: v.shallower,
        budget: operations::budget(v.budget),
        deadline: operations::deadline(v.deadline),
        tracked: v.tracked,
        flags: t::Flags {
            conversation: flags.conversation,
            listed: flags.listed,
            retention: match flags.retention {
                w::Retention::Kept => t::Retention::Kept,
                w::Retention::ForgottenOnceRead => t::Retention::ForgottenOnceRead,
            },
            ended_by: match flags.ended_by {
                w::EndedBy::Result => t::EndedBy::Result,
                w::EndedBy::Requester => t::EndedBy::Requester,
                w::EndedBy::Standing => t::EndedBy::Standing,
                w::EndedBy::Closer(closer) => {
                    t::EndedBy::Closer(t::Closer { connector: closer.connector, code: closer.code })
                }
            },
        },
        label: v.label,
        grace: match v.grace {
            Some(value) => Some(value),
            None => None,
        },
        llm,
    })
}
fn contract(value: w::TaskContractDocument, limits: &w::Limits) -> Result<t::Contract, Untranslatable> {
    Ok(match value {
        w::TaskContractDocument::Job(_) => return Err(bad()),
        w::TaskContractDocument::Report(value) => t::Contract::Report { words: value.words },
        w::TaskContractDocument::Change(value) => t::Contract::Change {
            connector: value.connector,
            kind: value.kind,
            words: value.words,
            choices: Box::new([]),
        },
        w::TaskContractDocument::Approval(value) => {
            let v = value;
            t::Contract::Approval { state: v.state, part: mapped::part_to_domain(v.part) }
        }
        w::TaskContractDocument::Verdict(value) => {
            if value.cut {
                return Err(bad());
            }
            let path = w::paths::TASK_CONTRACT_DOCUMENT_VERDICT_CHOICES;
            let count = asks::count(value.choices.as_ref(), path)?;
            let mut choices = List::with_capacity(count);
            let mut reader = Reader::new(value.choices.as_ref());
            for _ in 0..count {
                let bytes = asks::item(&mut reader, path)?;
                let mut piece = Reader::new(&bytes);
                let v = match w::VerdictChoice::decode(limits, &mut piece) {
                    Ok(value) => value,
                    Err(problem) => return Err(map_problem(problem)),
                };
                if !piece.is_empty() {
                    return Err(bad());
                }
                let mut fields = List::with_capacity(v.fields.len());
                for field in v.fields.into_boxed() {
                    let f = field;
                    fields.push(t::FieldRule { name: f.name, max: f.max }).expect("same count");
                }
                choices
                    .push(t::Verdict {
                        code: v.code,
                        words: v.words,
                        followups: v.followups,
                        fields: fields.into_boxed(),
                    })
                    .expect("same count");
            }
            t::Contract::Verdict { choices: choices.into_boxed() }
        }
    })
}

pub(crate) fn proposed(
    value: w::Propose,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<s::Proposed, Untranslatable> {
    if value.project != 0 {
        return Err(Untranslatable { path: w::paths::PROPOSE_PROJECT });
    }
    let value_set = match value.action {
        w::ProposeAction::Settings(v) => v,
        w::ProposeAction::Batch(_) | w::ProposeAction::Amendment(_) => return Err(bad()),
    };
    let envelope = w::ChangeSettings { against: value_set.against, items: value_set.items, running: w::Running::Stays };
    if let Err(problem) = envelope.measure(limits) {
        return Err(map_problem(problem));
    }
    let change = change(envelope, models, limits)?;
    Ok(s::Proposed { against: change.against, items: change.items, reason: value.reason })
}

pub(crate) fn catalogue_from_record(value: w::CatalogueEntry) -> s::CatalogueModel {
    s::CatalogueModel {
        first_read: None,
        number: value.number,
        model: model_from_record(value.model),
        shown: value.shown,
        description: value.description,
        picker: value.picker,
        withdrawn: value.withdrawn,
        estimated: value.estimated,
        source: mapped::model_source_to_domain(value.source),
    }
}

fn task_later(value: p::LlmLater) -> t::LlmLater {
    let mut subagents = List::with_capacity(u32::try_from(value.subagents.len()).expect("bounded models"));
    for item in value.subagents {
        subagents.push(t::ModelEffort { model: item.model, effort: item.effort }).expect("same count");
    }
    t::LlmLater {
        main: t::ModelEffort { model: value.main.model, effort: value.main.effort },
        subagents: subagents.into_boxed(),
    }
}
