//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use jig_client_protocol_wire::Record as _;
use super::{pieces, problem, w};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn text(value: &d::Text, _limits: &w::Limits) -> Result<w::DocumentText, w::Problem> {
    Ok(w::DocumentText { bytes: bytes::copy_of(&value.bytes), cut: value.cut })
}
pub fn held_role(value: jig_domain_people::Role, _limits: &w::Limits) -> Result<w::HeldRole, w::Problem> {
    Ok(match value {
        jig_domain_people::Role::Owner => w::HeldRole::Owner,
        jig_domain_people::Role::Maintainer => w::HeldRole::Maintainer,
        jig_domain_people::Role::Member => w::HeldRole::Member,
        jig_domain_people::Role::Observer => w::HeldRole::Observer,
        jig_domain_people::Role::Policy { role } => {
            w::HeldRole::Policy(w::HeldRolePolicy { role })
        }
    })
}
pub fn funder(value: t::Funder, _limits: &w::Limits) -> Result<w::Funder, w::Problem> {
    Ok(match value {
        t::Funder::Task(number) => w::Funder::Task(w::FunderTask { value: number }),
        t::Funder::Pool { project, person, period } => {
            w::Funder::Pool(w::FunderPool { project, person, period })
        }
        t::Funder::Recurring { project, task, period } => {
            w::Funder::Recurring(w::FunderRecurring { project, task, period })
        }
        t::Funder::Period { project, period } => {
            w::Funder::Period(w::FunderPeriod { project, period })
        }
    })
}
pub fn numbers(value: t::Numbers, _limits: &w::Limits) -> Result<w::Numbers, w::Problem> {
    Ok(w::Numbers {
            budget: value.budget,
            spent: value.spent,
            spent_below: value.spent_below,
            reserved: value.reserved,
        })
}
pub fn budget(value: &d::BudgetLine, limits: &w::Limits) -> Result<w::Budget, w::Problem> {
    Ok(w::Budget {
            numbers: numbers(value.numbers, limits)?,
            left: value.left,
            funder: funder(value.funder, limits)?,
            deadline: match value.deadline {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        })
}
pub fn ledger(value: &d::Ledger, limits: &w::Limits) -> Result<w::PoolLedger, w::Problem> {
    Ok(w::PoolLedger {
            source: funder(value.source, limits)?,
            parent: match value.parent {
                Some(item) => Some(funder(item, limits)?),
                None => None,
            },
            numbers: numbers(value.numbers, limits)?,
            left: value.left,
            closed: value.closed,
            began: match value.began {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            next: match value.next {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        })
}
pub fn executor(value: t::Executor, limits: &w::Limits) -> Result<w::Executor, w::Problem> {
    Ok(match value {
        t::Executor::Job { kind } => w::Executor::Job(w::ExecutorJob { kind }),
        t::Executor::Agent { charter } => {
            w::Executor::Agent(w::ExecutorAgent { charter })
        }
        t::Executor::Procedure { connector, code } => {
            w::Executor::Procedure(w::ExecutorProcedure { connector, code })
        }
        t::Executor::Person(address) => w::Executor::Person(w::ExecutorPerson { value: super::chips::address(address, limits)? }),
    })
}
pub fn requester(value: t::Party, _limits: &w::Limits) -> Result<w::Requester, w::Problem> {
    Ok(match value {
        t::Party::Task(number) => {
            w::Requester::Task(w::RequesterTask { value: number })
        }
        t::Party::Person(number) => {
            w::Requester::Person(w::RequesterPerson { value: number })
        }
        t::Party::Deployment { project } => {
            w::Requester::Deployment(w::RequesterDeployment { project })
        }
    })
}
pub fn wake_rule(value: t::WakeRule, _limits: &w::Limits) -> Result<w::WakeRule, w::Problem> {
    Ok(match value {
        t::WakeRule::Never => w::WakeRule::Never,
        t::WakeRule::Immediate => w::WakeRule::Immediate,
        t::WakeRule::Batch { count, age } => {
            w::WakeRule::Batch(w::WakeRuleBatch { count, age })
        }
        t::WakeRule::Paced { span } => {
            w::WakeRule::Paced(w::WakeRulePaced { span })
        }
    })
}
pub fn wake(value: t::WakePolicy, limits: &w::Limits) -> Result<w::WakePolicy, w::Problem> {
    Ok(w::WakePolicy {
            words: wake_rule(value.words, limits)?,
            notices: wake_rule(value.notices, limits)?,
            news: wake_rule(value.news, limits)?,
            results: match value.results {
                t::ResultsWake::Never => w::ResultsWake::Never,
                t::ResultsWake::Each => w::ResultsWake::Each,
                t::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
            },
            questions: value.questions,
            answers: value.answers,
            timers: value.timers,
        })
}
fn project(
    value: &d::FrameProjectRole,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::MeProject, w::Problem> {
    let mut defaults = List::with_capacity(value.defaults.len());
    for (template, choice) in &value.defaults {
        let choice = super::llm::choice(Some(choice), models, limits)?.expect("present preview");
        defaults
            .push(w::MeDefault { template: *template, choice })
            .expect("same preview count");
    }
    Ok(w::MeProject {
            left_out: value.left_out,
            public: match names {
                Some(names) if value.public => Some(names.project(value.project)),
                Some(_) | None => None,
            },
            project: value.project,
            name: text(&value.name, limits)?,
            role: held_role(value.role, limits)?,
            pool: match &value.pool {
                Some(item) => Some(ledger(item, limits)?),
                None => None,
            },
            templates: super::me::templates(&value.templates, models, limits)?,
            creating: match &value.creating {
                Some(value) => Some(super::me::creating(models, value, limits)?),
                None => None,
            },
            models: super::llm::models(value.offer.as_slice(), models, limits)?,
            defaults,
        })
}
pub fn frame(value: &d::Frame, models: &[crate::Model], limits: &w::Limits) -> Result<w::Me, w::Problem> {
    frame_named(value, models, None, limits)
}
/// The component supplies its held public key (jig/docs/design/domain/people/public-reading.md, 8).
pub(crate) fn frame_named(
    value: &d::Frame,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::Me, w::Problem> {
    let mut projects = List::with_capacity(value.projects.len());
    for item in &value.projects {
        projects.push(project(item, models, names, limits)?).expect("supplied projects");
    }
    let updates = match value.updates {
        d::Count::Known(value) => w::Count { value, limited: false },
        d::Count::AtLeast(value) => w::Count { value, limited: true },
    };
    Ok(w::Me {
            providers: super::catalogue::offers(&value.providers, limits.me_providers, limits)?,
            providers_cut: value.providers_cut,
            key_bytes: value.key_bytes,
            redirect_bytes: value.redirect_bytes,
            words_bytes: value.words_bytes,
            party: w::MeParty {
                    number: value.party,
                    kind: match value.kind {
                        jig_domain_people::Kind::Person => w::PartyKind::Person,
                        jig_domain_people::Kind::Service => w::PartyKind::Service,
                    },
                    login: text(&value.login, limits)?,
                    name: text(&value.name, limits)?,
                },
            projects: me_projects_pieces(&projects, limits.me_projects, w::paths::ME_PROJECTS, limits)?,
            needs: w::Needs {
                    questions: value.need_counts.questions,
                    proposals: value.need_counts.proposals,
                    escalations: value.need_counts.escalations,
                    person_tasks: value.need_counts.person_tasks,
                },
            needs_total: value.needs,
            updates,
            results_seen: value.results_seen,
            cut: value.cut,
            retention: value.retention,
            deployment_role: match value.deployment_role {
                Some(item) => Some(held_role(item, limits)?),
                None => None,
            },
        })
}

pub fn part_kind(value: t::PartKind) -> Result<w::PartKind, w::Problem> {
    Ok(match value {
        t::PartKind::Resource => w::PartKind::Resource,
        t::PartKind::Pool => w::PartKind::Pool,
        t::PartKind::Result => w::PartKind::Result,
        t::PartKind::Procedure => w::PartKind::Procedure,
        t::PartKind::Effect => w::PartKind::Effect,
        t::PartKind::Outbox => w::PartKind::Outbox,
        t::PartKind::Hold => w::PartKind::Hold,
    })
}
pub fn part(value: &t::Part, _limits: &w::Limits) -> Result<w::Part, w::Problem> {
    let key = match &value.key {
        t::PartKey::Number(number) => {
            w::PartKey::Number(w::PartKeyNumber { value: *number })
        }
        t::PartKey::Path(value) => {
            w::PartKey::Path(w::PartKeyPath { value: value.clone() })
        }
    };
    Ok(w::Part { connector: value.connector, kind: part_kind(value.kind)?, key })
}
pub fn pattern(segments: &List<Box<[u8]>>, last: &d::PatternEnd, _limits: &w::Limits) -> Result<w::Pattern, w::Problem> {
    let last = match last {
        d::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        d::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    };
    Ok(w::Pattern { segments: segments.clone(), last })
}
pub fn grant(value: &d::GrantLine, limits: &w::Limits) -> Result<w::Grant, w::Problem> {
    Ok(w::Grant {
            connector: value.connector,
            kind: value.kind,
            pattern: pattern(&value.segments, &value.last, limits)?,
        })
}
pub fn authority(
    models: &[crate::Model],
    value: &d::AuthorityLine,
    limits: &w::Limits,
) -> Result<w::AuthorityDocument, w::Problem> {
    let mut grants = List::with_capacity(value.grants.len());
    for item in &value.grants {
        grants.push(grant(item, limits)?).expect("supplied grants");
    }
    let grants =
        grants_pieces(&grants, limits.authority_grants.min(w::CEILINGS.authority_grants), w::paths::AUTHORITY_GRANTS, limits)?;
    let mut scopes = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        scopes
            .push(w::ResourceScope {
                    connector: item.connector,
                    pattern: pattern(&item.segments, &item.last, limits)?,
                })
            .expect("supplied scopes");
    }
    let note_resources = scopes_pieces(
        &scopes,
        limits.authority_note_resources.min(w::CEILINGS.authority_note_resources),
        w::paths::AUTHORITY_NOTE_RESOURCES, limits)?;
    let mut kinds = List::with_capacity(value.executors.len());
    for item in &value.executors {
        kinds.push(authority_executor(*item, limits)?).expect("supplied executors");
    }
    Ok(w::AuthorityDocument {
            cut: false,
            authority: w::Authority {
                    models: super::llm::authority(
                        jig_domain_authority::Models::from_parts(value.models.bits(), value.models.is_all()),
                        models,
                        limits,
                    )?,
                    tools: value.tools.0,
                    grants,
                    delegation: w::Delegation { kinds, tasks: value.tasks, depth: value.depth },
                    spend: value.ceiling.spend,
                    deadline: match value.ceiling.deadline {
                        Some(item) => Some(item.as_nanos()),
                        None => None,
                    },
                    notes: value.notes.0,
                    note_resources,
                },
        })
}

pub fn authority_executor(value: t::AuthorityExecutor, _limits: &w::Limits) -> Result<w::AuthorityExecutor, w::Problem> {
    Ok(match value {
        t::AuthorityExecutor::Job(value) => {
            w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value })
        }
        t::AuthorityExecutor::Charter(number) => w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value: number }),
        t::AuthorityExecutor::Procedure(number) => w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value: number }),
        t::AuthorityExecutor::Role(number) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value: number }),
    })
}
pub fn grants_pieces(rows: &List<w::Grant>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
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
pub fn scopes_pieces(rows: &List<w::ResourceScope>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
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
pub fn specification(value: &d::SpecLine, limits: &w::Limits) -> Result<w::SpecificationDocument, w::Problem> {
    let mut parameters = List::with_capacity(value.parameters.capacity());
    for item in &value.parameters {
        parameters.push(parameter(item, limits)?).expect("supplied parameters");
    }
    Ok(w::SpecificationDocument {
            specification: w::DocumentSpecification {
                    asks: value.asks,
                    words: text(&value.words, limits)?,
                    parameters,
                    inputs: value.inputs.clone(),
                },
            cut: value.cut,
        })
}
fn parameter(value: &d::Parameter, limits: &w::Limits) -> Result<w::ParameterDocument, w::Problem> {
    Ok(match value {
        d::Parameter::Number { name, value } => w::ParameterDocument::Number(w::ParameterDocumentNumber { name: *name, value: *value }),
        d::Parameter::Bytes { name, value } => w::ParameterDocument::Bytes(w::ParameterDocumentBytes { name: *name, value: text(value, limits)? }),
        d::Parameter::Resource { name, part } => {
            if part.kind != t::PartKind::Resource {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            let resource = match part.key {
                t::PartKey::Number(number) => number,
                t::PartKey::Path(_) => return Err(problem(skein_codec::Reason::Malformed)),
            };
            w::ParameterDocument::Resource(w::ParameterDocumentResource { name: *name, connector: part.connector, resource })
        }
    })
}
pub fn subscriptions(value: &List<t::Subscription>, limits: &w::Limits) -> Result<w::Subscriptions, w::Problem> {
    let mut entries = List::with_capacity(value.capacity());
    for item in value {
        entries.push(subscription(*item, limits)?).expect("supplied subscriptions");
    }
    Ok(w::Subscriptions { entries })
}

pub fn subscription_kind(value: t::SubscriptionKind, _limits: &w::Limits) -> Result<w::SubscriptionKind, w::Problem> {
    Ok(match value {
        t::SubscriptionKind::Task { target, held, result } => w::SubscriptionKind::Task(w::SubscriptionKindTask { target, held, result }),
        t::SubscriptionKind::Timer { at, period } => w::SubscriptionKind::Timer(w::SubscriptionKindTimer { at: at.as_nanos(), period }),
        t::SubscriptionKind::Topic { connector, topic } => w::SubscriptionKind::Topic(w::SubscriptionKindTopic { connector, topic }),
    })
}
pub fn subscription(value: t::Subscription, limits: &w::Limits) -> Result<w::Subscription, w::Problem> {
    Ok(w::Subscription { number: value.number, kind: subscription_kind(value.kind, limits)? })
}
pub fn contract(value: &d::ContractLine, limits: &w::Limits) -> Result<w::TaskContractDocument, w::Problem> {
    Ok(match value {
        d::ContractLine::Job { bytes } => w::TaskContractDocument::Job(w::TaskContractDocumentJob { bytes: *bytes }),
        d::ContractLine::Approval { state, part: value } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval { state: bytes::copy_of(state), part: part(value, limits)? })
        }
        d::ContractLine::Report { words } => w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: *words }),
        d::ContractLine::Change { connector, kind, words } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange { connector: *connector, kind: *kind, words: *words })
        }
        d::ContractLine::Verdict { choices, cut } => {
            let mut rows = List::with_capacity(choices.len());
            for item in choices {
                let mut fields = List::with_capacity(super::count(item.fields.len())?);
                for field in &item.fields {
                    fields
                        .push(w::FieldRule { name: bytes::copy_of(&field.name), max: field.max })
                        .expect("supplied field rules");
                }
                rows.push(w::VerdictChoice { code: item.code, words: item.words, followups: item.followups, fields })
                .expect("supplied choices");
            }
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict {
                    choices: choices_pieces(
                        &rows,
                        limits
                            .task_contract_document_verdict_choices
                            .min(w::CEILINGS.task_contract_document_verdict_choices),
                        w::paths::TASK_CONTRACT_DOCUMENT_VERDICT_CHOICES, limits)?,
                    cut: *cut,
                })
        }
    })
}

pub fn choices_pieces(rows: &List<w::VerdictChoice>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
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

pub fn me_projects_pieces(rows: &List<w::MeProject>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
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
