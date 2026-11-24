//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::{pieces, problem, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Wall, bytes};
#[must_use]
pub fn text(value: &d::Text) -> w::DocumentText {
    mapped::document_text_to_record(value)
}
#[must_use]
pub fn held_role(value: jig_domain_people::Role) -> w::HeldRole {
    mapped::held_role_to_record(&value)
}
#[must_use]
pub fn funder(value: t::Funder) -> w::Funder {
    mapped::funder_to_record(&value)
}
#[must_use]
pub fn numbers(value: t::Numbers) -> w::Numbers {
    mapped::numbers_to_record(&value)
}
#[must_use]
pub fn budget(value: &d::BudgetLine) -> w::Budget {
    mapped::budget_to_record(value)
}
#[must_use]
pub fn ledger(value: &d::Ledger) -> w::PoolLedger {
    mapped::pool_ledger_to_record(value)
}
#[must_use]
pub fn executor(value: t::Executor) -> w::Executor {
    match value {
        t::Executor::Job { kind } => w::Executor::Job(w::ExecutorJob { kind }),
        t::Executor::Agent { charter } => w::Executor::Agent(w::ExecutorAgent { charter }),
        t::Executor::Procedure { connector, code } => w::Executor::Procedure(w::ExecutorProcedure { connector, code }),
        t::Executor::Person(address) => {
            w::Executor::Person(w::ExecutorPerson { value: super::chips::address(address) })
        }
    }
}
#[must_use]
pub fn requester(value: t::Party) -> w::Requester {
    mapped::requester_to_record(&value)
}
#[must_use]
pub fn wake_rule(value: t::WakeRule) -> w::WakeRule {
    mapped::wake_rule_to_record(&value)
}
#[must_use]
pub fn wake(value: t::WakePolicy) -> w::WakePolicy {
    mapped::wake_policy_to_record(&value)
}
fn project(
    value: &d::FrameProjectRole,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::MeProject, w::Problem> {
    let mut defaults = List::with_capacity(value.defaults.len());
    for (template, choice) in &value.defaults {
        let choice = super::llm::choice(Some(choice), models)?.expect("present preview");
        defaults.push(w::MeDefault { template: *template, choice }).expect("same preview count");
    }
    Ok(w::MeProject {
        left_out: value.left_out,
        public: match names {
            Some(names) if value.public => Some(names.project(value.project)),
            Some(_) | None => None,
        },
        project: value.project,
        name: text(&value.name),
        role: held_role(value.role),
        pool: match &value.pool {
            Some(item) => Some(ledger(item)),
            None => None,
        },
        templates: super::me::templates(&value.templates, models, limits)?,
        creating: match &value.creating {
            Some(value) => Some(super::me::creating(models, value, limits)?),
            None => None,
        },
        models: super::llm::models(value.offer.as_slice(), models)?,
        defaults,
    })
}
pub fn frame(
    value: &d::Frame,
    expires: Wall,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::Me, w::Problem> {
    frame_named(value, expires, models, None, limits)
}
/// The component supplies its held public key (jig/docs/design/domain/people/public-reading.md, 8).
pub(crate) fn frame_named(
    value: &d::Frame,
    expires: Wall,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::Me, w::Problem> {
    let mut projects = List::with_capacity(value.projects.len());
    for item in &value.projects {
        projects.push(project(item, models, names, limits)?).expect("supplied projects");
    }
    Ok(w::Me {
        providers: super::catalogue::offers(&value.providers, limits.me_providers, limits)?,
        providers_cut: value.providers_cut,
        key_bytes: value.key_bytes,
        redirect_bytes: value.redirect_bytes,
        words_bytes: value.words_bytes,
        sign_in_expires: expires.as_nanos(),
        party: w::MeParty {
            number: value.party,
            kind: mapped::party_kind_to_record(&value.kind),
            login: text(&value.login),
            name: text(&value.name),
        },
        projects: me_projects_pieces(&projects, limits.me_projects, w::paths::ME_PROJECTS, limits)?,
        needs: mapped::needs_to_record(&value.need_counts),
        needs_total: value.needs,
        updates: super::live::count(value.updates),
        results_seen: value.results_seen,
        cut: value.cut,
        retention: value.retention,
        deployment_role: match value.deployment_role {
            Some(item) => Some(held_role(item)),
            None => None,
        },
    })
}

#[must_use]
pub fn part_kind(value: t::PartKind) -> w::PartKind {
    mapped::part_kind_to_record(&value)
}
#[must_use]
pub fn part(value: &t::Part) -> w::Part {
    mapped::part_to_record(value)
}
#[must_use]
pub fn pattern(segments: &List<Box<[u8]>>, last: &d::PatternEnd) -> w::Pattern {
    let last = match last {
        d::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        d::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    };
    w::Pattern { segments: segments.clone(), last }
}
#[must_use]
pub fn grant(value: &d::GrantLine) -> w::Grant {
    w::Grant { connector: value.connector, kind: value.kind, pattern: pattern(&value.segments, &value.last) }
}
pub fn authority(
    models: &[crate::Model],
    value: &d::AuthorityLine,
    limits: &w::Limits,
) -> Result<w::AuthorityDocument, w::Problem> {
    let mut grants = List::with_capacity(value.grants.len());
    for item in &value.grants {
        grants.push(grant(item)).expect("supplied grants");
    }
    let grants = grants_pieces(
        &grants,
        limits.authority_grants.min(w::CEILINGS.authority_grants),
        w::paths::AUTHORITY_GRANTS,
        limits,
    )?;
    let mut scopes = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        scopes
            .push(w::ResourceScope { connector: item.connector, pattern: pattern(&item.segments, &item.last) })
            .expect("supplied scopes");
    }
    let note_resources = scopes_pieces(
        &scopes,
        limits.authority_note_resources.min(w::CEILINGS.authority_note_resources),
        w::paths::AUTHORITY_NOTE_RESOURCES,
        limits,
    )?;
    let mut kinds = List::with_capacity(value.executors.len());
    for item in &value.executors {
        kinds.push(authority_executor(*item)).expect("supplied executors");
    }
    Ok(w::AuthorityDocument {
        cut: false,
        authority: w::Authority {
            models: super::llm::authority(
                jig_domain_authority::Models::from_parts(value.models.bits(), value.models.is_all()),
                models,
            ),
            tools: crate::translate::convert::tools_to_record(value.tools),
            grants,
            delegation: w::Delegation { kinds, tasks: value.tasks, depth: value.depth },
            spend: value.ceiling.spend,
            deadline: match value.ceiling.deadline {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            notes: crate::translate::convert::scopes_to_record(value.notes),
            note_resources,
        },
    })
}

#[must_use]
pub fn authority_executor(value: t::AuthorityExecutor) -> w::AuthorityExecutor {
    mapped::authority_executor_to_record(&value)
}
pub fn grants_pieces(
    rows: &List<w::Grant>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn scopes_pieces(
    rows: &List<w::ResourceScope>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn specification(value: &d::SpecLine) -> Result<w::SpecificationDocument, w::Problem> {
    let mut parameters = List::with_capacity(value.parameters.capacity());
    for item in &value.parameters {
        parameters.push(parameter(item)?).expect("supplied parameters");
    }
    Ok(w::SpecificationDocument {
        specification: w::DocumentSpecification {
            asks: value.asks,
            words: text(&value.words),
            parameters,
            inputs: value.inputs.clone(),
        },
        cut: value.cut,
    })
}
fn parameter(value: &d::Parameter) -> Result<w::ParameterDocument, w::Problem> {
    Ok(match value {
        d::Parameter::Number { name, value } => {
            w::ParameterDocument::Number(w::ParameterDocumentNumber { name: *name, value: *value })
        }
        d::Parameter::Bytes { name, value } => {
            w::ParameterDocument::Bytes(w::ParameterDocumentBytes { name: *name, value: text(value) })
        }
        d::Parameter::Resource { name, part } => {
            if part.kind != t::PartKind::Resource {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            let resource = match part.key {
                t::PartKey::Number(number) => number,
                t::PartKey::Path(_) => return Err(problem(skein_codec::Reason::Malformed)),
            };
            w::ParameterDocument::Resource(w::ParameterDocumentResource {
                name: *name,
                connector: part.connector,
                resource,
            })
        }
    })
}
#[must_use]
pub fn subscriptions(value: &List<t::Subscription>) -> w::Subscriptions {
    let mut entries = List::with_capacity(value.capacity());
    for item in value {
        entries.push(subscription(*item)).expect("supplied subscriptions");
    }
    w::Subscriptions { entries }
}

#[must_use]
pub fn subscription_kind(value: t::SubscriptionKind) -> w::SubscriptionKind {
    mapped::subscription_kind_to_record(&value)
}
#[must_use]
pub fn subscription(value: t::Subscription) -> w::Subscription {
    mapped::subscription_to_record(&value)
}
pub fn contract(value: &d::ContractLine, limits: &w::Limits) -> Result<w::TaskContractDocument, w::Problem> {
    Ok(match value {
        d::ContractLine::Job { bytes } => w::TaskContractDocument::Job(w::TaskContractDocumentJob { bytes: *bytes }),
        d::ContractLine::Approval { state, part: value } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval {
                state: bytes::copy_of(state),
                part: part(value),
            })
        }
        d::ContractLine::Report { words } => {
            w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: *words })
        }
        d::ContractLine::Change { connector, kind, words } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange {
                connector: *connector,
                kind: *kind,
                words: *words,
            })
        }
        d::ContractLine::Verdict { choices, cut } => {
            let mut rows = List::with_capacity(choices.len());
            for item in choices {
                rows.push(crate::translate::client::verdict_choice(item)?).expect("supplied choices");
            }
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict {
                choices: choices_pieces(
                    &rows,
                    limits
                        .task_contract_document_verdict_choices
                        .min(w::CEILINGS.task_contract_document_verdict_choices),
                    w::paths::TASK_CONTRACT_DOCUMENT_VERDICT_CHOICES,
                    limits,
                )?,
                cut: *cut,
            })
        }
    })
}

pub fn choices_pieces(
    rows: &List<w::VerdictChoice>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}

pub fn me_projects_pieces(
    rows: &List<w::MeProject>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
