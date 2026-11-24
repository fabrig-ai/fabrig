//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::{pieces, problem, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Wall, bytes};
#[must_use]
pub fn text(value: &d::Text) -> w::DocumentText {
    w::DocumentText { bytes: bytes::copy_of(&value.bytes), cut: value.cut }
}
#[must_use]
pub fn held_role(value: jig_domain_people::Role) -> w::HeldRole {
    match value {
        jig_domain_people::Role::Owner => w::HeldRole::Owner,
        jig_domain_people::Role::Maintainer => w::HeldRole::Maintainer,
        jig_domain_people::Role::Member => w::HeldRole::Member,
        jig_domain_people::Role::Observer => w::HeldRole::Observer,
        jig_domain_people::Role::Policy { role } => w::HeldRole::Policy(w::HeldRolePolicy { role }),
    }
}
#[must_use]
pub fn funder(value: t::Funder) -> w::Funder {
    match value {
        t::Funder::Task(number) => w::Funder::Task(w::FunderTask { value: number }),
        t::Funder::Pool { project, person, period } => w::Funder::Pool(w::FunderPool { project, person, period }),
        t::Funder::Recurring { project, task, period } => {
            w::Funder::Recurring(w::FunderRecurring { project, task, period })
        }
        t::Funder::Period { project, period } => w::Funder::Period(w::FunderPeriod { project, period }),
    }
}
#[must_use]
pub fn numbers(value: t::Numbers) -> w::Numbers {
    w::Numbers { budget: value.budget, spent: value.spent, spent_below: value.spent_below, reserved: value.reserved }
}
#[must_use]
pub fn budget(value: &d::BudgetLine) -> w::Budget {
    w::Budget {
        numbers: numbers(value.numbers),
        left: value.left,
        funder: funder(value.funder),
        deadline: match value.deadline {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
    }
}
#[must_use]
pub fn ledger(value: &d::Ledger) -> w::PoolLedger {
    w::PoolLedger {
        source: funder(value.source),
        parent: match value.parent {
            Some(item) => Some(funder(item)),
            None => None,
        },
        numbers: numbers(value.numbers),
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
    }
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
    match value {
        t::Party::Task(number) => w::Requester::Task(w::RequesterTask { value: number }),
        t::Party::Person(number) => w::Requester::Person(w::RequesterPerson { value: number }),
        t::Party::Deployment { project } => w::Requester::Deployment(w::RequesterDeployment { project }),
    }
}
#[must_use]
pub fn wake_rule(value: t::WakeRule) -> w::WakeRule {
    match value {
        t::WakeRule::Never => w::WakeRule::Never,
        t::WakeRule::Immediate => w::WakeRule::Immediate,
        t::WakeRule::Batch { count, age } => w::WakeRule::Batch(w::WakeRuleBatch { count, age }),
        t::WakeRule::Paced { span } => w::WakeRule::Paced(w::WakeRulePaced { span }),
    }
}
#[must_use]
pub fn wake(value: t::WakePolicy) -> w::WakePolicy {
    w::WakePolicy {
        words: wake_rule(value.words),
        notices: wake_rule(value.notices),
        news: wake_rule(value.news),
        results: match value.results {
            t::ResultsWake::Never => w::ResultsWake::Never,
            t::ResultsWake::Each => w::ResultsWake::Each,
            t::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
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
        sign_in_expires: expires.as_nanos(),
        party: w::MeParty {
            number: value.party,
            kind: match value.kind {
                jig_domain_people::Kind::Person => w::PartyKind::Person,
                jig_domain_people::Kind::Service => w::PartyKind::Service,
            },
            login: text(&value.login),
            name: text(&value.name),
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
            Some(item) => Some(held_role(item)),
            None => None,
        },
    })
}

#[must_use]
pub fn part_kind(value: t::PartKind) -> w::PartKind {
    match value {
        t::PartKind::Resource => w::PartKind::Resource,
        t::PartKind::Pool => w::PartKind::Pool,
        t::PartKind::Result => w::PartKind::Result,
        t::PartKind::Procedure => w::PartKind::Procedure,
        t::PartKind::Effect => w::PartKind::Effect,
        t::PartKind::Outbox => w::PartKind::Outbox,
        t::PartKind::Hold => w::PartKind::Hold,
    }
}
#[must_use]
pub fn part(value: &t::Part) -> w::Part {
    let key = match &value.key {
        t::PartKey::Number(number) => w::PartKey::Number(w::PartKeyNumber { value: *number }),
        t::PartKey::Path(value) => w::PartKey::Path(w::PartKeyPath { value: value.clone() }),
    };
    w::Part { connector: value.connector, kind: part_kind(value.kind), key }
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

#[must_use]
pub fn authority_executor(value: t::AuthorityExecutor) -> w::AuthorityExecutor {
    match value {
        t::AuthorityExecutor::Job(value) => w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value }),
        t::AuthorityExecutor::Charter(number) => {
            w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value: number })
        }
        t::AuthorityExecutor::Procedure(number) => {
            w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value: number })
        }
        t::AuthorityExecutor::Role(number) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value: number }),
    }
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
    match value {
        t::SubscriptionKind::Task { target, held, result } => {
            w::SubscriptionKind::Task(w::SubscriptionKindTask { target, held, result })
        }
        t::SubscriptionKind::Timer { at, period } => {
            w::SubscriptionKind::Timer(w::SubscriptionKindTimer { at: at.as_nanos(), period })
        }
        t::SubscriptionKind::Topic { connector, topic } => {
            w::SubscriptionKind::Topic(w::SubscriptionKindTopic { connector, topic })
        }
    }
}
#[must_use]
pub fn subscription(value: t::Subscription) -> w::Subscription {
    w::Subscription { number: value.number, kind: subscription_kind(value.kind) }
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
