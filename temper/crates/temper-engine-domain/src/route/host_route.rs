//! Typed host calls cross the protocol boundary before tool decisions and after
//! their answers (jig's domain/engine/executors.md, 3; jig's domain/hosts/README.md, 2).
use super::{
    Call, CallAnswer, CallKey, Decision, Delivery, Domain, Env, Limits, Payload, Request, Work, emit, relay_payload,
};
use alloc::boxed::Box;
use jig_domain_core::{SettledAnswer, SettledCall};
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use skein_lib::{Decimal, List, ReplyTo, Token, Writer};

/// Root to protocol: a relayed call to decode, or one the fleet dropped.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum HostRequest {
    Decode {
        to: ReplyTo,
        task: u64,
        attempt: u64,
        call: fleet::Call,
    },
    Dropped {
        call: fleet::Call,
    },
    /// Decode a whole authorized settings note, without the model call's payload ceiling.
    SettingsNote {
        to: ReplyTo,
        call: Box<Call>,
        body: Box<[u8]>,
    },
}

/// An adopted repository's canonical resource identity for settings discovery
/// (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryIdentity {
    /// The host in resource names, including any port.
    pub host: Box<[u8]>,
    /// The repository's forge owner.
    pub owner: Box<[u8]>,
    /// The repository's name within that owner.
    pub name: Box<[u8]>,
}

/// One person of the caller's project, for an authorized inventory
/// (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectPerson {
    pub person: u64,
    pub role: u32,
    /// Empty for someone adoption seeded who has not signed in.
    pub login: Box<[u8]>,
    /// Their account on each forge of the project's repositories, each forge once,
    /// in the order the repositories first name it (domain/agent/agents.md, 1).
    pub forges: Box<[PersonForge]>,
    /// The person this chat's task was requested by.
    pub you: bool,
}
/// What the lookup answers for one person on one forge (domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PersonForge {
    /// The forge's host, as the inventory's repository lines name it.
    pub host: Box<[u8]>,
    /// Their user there, unknown as `None`.
    pub user: Option<u64>,
    /// That user is temper's retained writer on this forge, who never counts as a person.
    pub writer: bool,
}
/// One model of the offer, for an authorized inventory (temper/docs/design/domain/agent/agents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OfferedModel {
    pub name: Box<[u8]>,
    pub provider: Box<[u8]>,
    pub efforts: Box<[Box<[u8]>]>,
    pub effort: Option<Box<[u8]>>,
}

/// An owned forge's person, the login made for whoever runs the box.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForgePerson {
    pub host: Box<[u8]>,
    pub login: Box<[u8]>,
}

/// The caller's current asking route, beside its stored judgement
/// (temper/docs/design/domain/agent/tools.md, HOW names the one way its caller can ask).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AskHow {
    Message,
    Person(u64),
    None,
}

/// Root to protocol after the decision commits: render a decided call's
/// answer, or carry a workspace delivery's settled answer to its host, which
/// the hub's link names only as text (jig's `domain/hosts/README.md`, 2).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum HostDelivery {
    /// Settings reads have no durable call part; inventories include project repositories and people.
    Settings {
        to: ReplyTo,
        answer: Box<jig_domain_core::SettingsAnswer>,
        repositories: Box<[RepositoryIdentity]>,
        people: Box<[ProjectPerson]>,
        forge_people: Box<[ForgePerson]>,
        offer: Box<[OfferedModel]>,
        application: Option<temper_engine_domain_forge::settings::SizedValue>,
        from: u32,
        requested: Option<u64>,
    },
    /// A `see` read's answer (domain/agent/ask-temper.md, 1).
    See {
        to: ReplyTo,
        line: Box<[u8]>,
        from: u32,
        answer: Box<super::see::SeeAnswer>,
    },
    Render {
        to: ReplyTo,
        key: CallKey,
        name: Box<[u8]>,
        tool: Box<[u8]>,
        job: Option<skein_forge::api::JobAttempt>,
        page: Option<(u32, u32)>,
        ask: AskHow,
        answer: CallAnswer,
    },
    Answer {
        channel: Token,
        task: u64,
        attempt: u64,
        name: Box<[u8]>,
        call: SettledCall,
    },
}

#[derive(Hash, Debug)]
pub(crate) struct Flight {
    pub task: u64,
    pub attempt: u64,
    pub key: Option<CallKey>,
    pub note_decoding: bool,
    pub render: FlightRender,
    pub name: Box<[u8]>,
    pub tool: Box<[u8]>,
    pub job: Option<skein_forge::api::JobAttempt>,
    pub page: Option<(u32, u32)>,
}

/// Only one tool's rendering context is kept per flight. Boxing the change
/// keeps the slot the same size as the existing amendment context.
#[derive(Hash, Debug)]
pub(crate) enum FlightRender {
    Plain,
    TaskRead,
    ProjectRead,
    ForgeRead,
    Settings {
        see: Option<Box<super::see::SeeFlight>>,
        from: u32,
        requested: Option<u64>,
        asked: Option<(jig_domain_core::settings::Setting, u32)>,
    },
    Amended(crate::Changed),
    Change(Box<ChangeChoice>),
    PersonRoles(Box<[u32]>),
    Document(Box<DocumentFlight>),
    Remember(Box<super::remember::RememberFlight>),
}

impl FlightRender {
    fn is_core_read(&self) -> bool {
        match self {
            Self::TaskRead | Self::ProjectRead | Self::Settings { .. } => true,
            Self::Plain
            | Self::ForgeRead
            | Self::Amended(_)
            | Self::Change(_)
            | Self::PersonRoles(_)
            | Self::Document(_)
            | Self::Remember(_) => false,
        }
    }
}

/// The call's words join the note family's durable result (domain/agent/ask-temper.md, 3).
#[derive(Hash, Debug)]
pub(crate) struct DocumentFlight {
    pub action: crate::DocumentAction,
    pub heading: Box<[u8]>,
    pub checked: Option<crate::NoteFamily>,
}

impl DocumentFlight {
    pub(crate) fn join(&self, answer: CallAnswer) -> CallAnswer {
        use crate::{DocumentOutcome, DocumentRefusal};
        let outcome = match answer {
            CallAnswer::NoteWritten { name, revision, lines, added, removed, first } => {
                DocumentOutcome::Written { name, revision, lines, added, removed, first }
            }
            CallAnswer::NoteRefused(why) => DocumentOutcome::Refused(match why {
                jig_domain_notes::Refusal::Moved { .. } | jig_domain_notes::Refusal::Missing => DocumentRefusal::Moved,
                jig_domain_notes::Refusal::Busy => DocumentRefusal::Busy,
                jig_domain_notes::Refusal::PastBound { bytes, bound } => DocumentRefusal::PastBound { bytes, bound },
                jig_domain_notes::Refusal::Oversized
                | jig_domain_notes::Refusal::Full
                | jig_domain_notes::Refusal::Exists
                | jig_domain_notes::Refusal::RevisionExhausted
                | jig_domain_notes::Refusal::PastEnd { .. } => DocumentRefusal::Note(why),
            }),
            CallAnswer::ToolDenied { answer, findings } => match self.checked {
                Some(family) => DocumentOutcome::Refused(DocumentRefusal::Denied { family, answer, findings }),
                None => return CallAnswer::ToolDenied { answer, findings },
            },
            answer @ (CallAnswer::Follow(_)
            | CallAnswer::Finding(_)
            | CallAnswer::Decided { .. }
            | CallAnswer::TaskRead(_)
            | CallAnswer::SettingsRefused(_)
            | CallAnswer::HoldOwner { .. }
            | CallAnswer::Again { .. }
            | CallAnswer::PassedUp { .. }
            | CallAnswer::Revised { .. }
            | CallAnswer::Amended { .. }
            | CallAnswer::Moved { .. }
            | CallAnswer::ForgeEffect { .. }
            | CallAnswer::ForgeEffectRefused(_)
            | CallAnswer::ForgeEffectDenied { .. }
            | CallAnswer::ForgeRead(_)
            | CallAnswer::EscalationDecided { .. }
            | CallAnswer::EscalationRefused(_)
            | CallAnswer::Proposed { .. }
            | CallAnswer::ProposalDecided { .. }
            | CallAnswer::ProposalRefused(_)
            | CallAnswer::Controlled
            | CallAnswer::ControlRefused(_)
            | CallAnswer::EffectRefused(_)
            | CallAnswer::ControlDenied { .. }
            | CallAnswer::Sent { .. }
            | CallAnswer::Introduced
            | CallAnswer::MessageRefused(_)
            | CallAnswer::Subscribed { .. }
            | CallAnswer::Unsubscribed
            | CallAnswer::SubscriptionRefused(_)
            | CallAnswer::ForgeSubscriptionRefused { .. }
            | CallAnswer::Delegated(_)
            | CallAnswer::DelegationDenied { .. }
            | CallAnswer::DelegationRefused(_)
            | CallAnswer::ReadRepositoryRefused { .. }
            | CallAnswer::SubscriptionRepositoryRefused { .. }
            | CallAnswer::ChangeRefused { .. }
            | CallAnswer::NoteDeleted { .. }
            | CallAnswer::NoteRecalled { .. }
            | CallAnswer::Document(_)
            | CallAnswer::Remember(_)
            | CallAnswer::Parts(_)
            | CallAnswer::Unavailable) => return answer,
        };
        CallAnswer::Document(Box::new(crate::Documented {
            action: self.action,
            heading: self.heading.clone(),
            outcome,
        }))
    }
}

#[derive(Hash, Debug)]
pub(crate) struct ChangeChoice {
    named: Option<Box<[u8]>>,
    project: u32,
    producers: Box<[Option<u32>]>,
}

pub(crate) fn decode(domain: &mut Domain, to: ReplyTo, task: u64, attempt: u64, call: fleet::Call) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    let flight = Flight {
        task,
        attempt,
        key: None,
        note_decoding: false,
        render: FlightRender::Plain,
        job: None,
        page: None,
        name: call.name.clone(),
        tool: call.tool.clone(),
    };
    assert!(domain.call_renders.insert(right, flight).is_ok(), "typed flights fit fleet calls");
    super::now(
        domain,
        Request::Worker(crate::WorkerRequest::Host(Box::new(HostRequest::Decode { to, task, attempt, call }))),
    );
}

pub(crate) fn decoded(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, to: ReplyTo, body: Call) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    if domain.settling.active() {
        drop(domain.call_renders.remove(&right));
        crate::stopping::call(domain, to);
        return;
    }
    let Some(flight) = domain.call_renders.get_mut(&right) else { return };
    if let Some(key) = flight.key {
        if flight.note_decoding && key.completion == body.completion && key.position == body.position {
            flight.note_decoding = false;
            let body = super::checked_call(body, &env.limits);
            relay_payload(domain, env, decision, to, key, body);
        }
        return;
    }
    let body = super::checked_call(body, &env.limits);
    let key =
        CallKey { task: flight.task, attempt: flight.attempt, completion: body.completion, position: body.position };
    let Some(to) = admit(domain, to, key, &body.tool) else { return };
    let mut job = None;
    let mut page = None;
    let render = if let crate::Tool::Finding(crate::FindingCall::List { .. } | crate::FindingCall::Show { .. })
    | crate::Tool::Follow(crate::FollowCall::List | crate::FollowCall::Show { .. }) = &body.tool
    {
        FlightRender::ProjectRead
    } else if let crate::Tool::ReadTask { .. } = &body.tool {
        FlightRender::TaskRead
    } else if let crate::Tool::ReadForge { read, .. } | crate::Tool::ReadNamed { read, .. } = &body.tool {
        job = super::diagnostics::job_attempt(read);
        if let skein_forge::api::Read::Reviews { page: at, skip, .. }
        | skein_forge::api::Read::Remarks { page: at, skip, .. }
        | skein_forge::api::Read::PullFiles { page: at, skip, .. } = read
        {
            page = Some((*at, *skip));
        }
        FlightRender::ForgeRead
    } else if let crate::Tool::Settings { from, read } = &body.tool {
        let requested = match read {
            jig_domain_core::SettingsRead::Revision { number } => Some(u64::from(*number)),
            jig_domain_core::SettingsRead::Inventory
            | jig_domain_core::SettingsRead::Setting { .. }
            | jig_domain_core::SettingsRead::Revisions { .. } => None,
        };
        let asked = match read {
            jig_domain_core::SettingsRead::Setting { setting, revision } => Some((*setting, *revision)),
            jig_domain_core::SettingsRead::Inventory
            | jig_domain_core::SettingsRead::Revision { .. }
            | jig_domain_core::SettingsRead::Revisions { .. } => None,
        };
        FlightRender::Settings { see: None, from: *from, requested, asked }
    } else if let crate::Tool::Document { action, body } = &body.tool {
        FlightRender::Document(Box::new(DocumentFlight {
            action: *action,
            heading: super::document::heading(*action, body),
            checked: None,
        }))
    } else if let crate::Tool::Change { changes } = &body.tool {
        change_choice(domain, key.task, changes)
    } else if let crate::Tool::Delegate { batch } = &body.tool {
        person_roles(domain, key.task, batch)
    } else if let crate::Tool::Amend { amendment, .. } = &body.tool {
        FlightRender::Amended(crate::Changed::amendment(amendment))
    } else {
        FlightRender::Plain
    };
    let flight = domain.call_renders.get_mut(&right).expect("current typed flight");
    flight.render = render;
    flight.job = job;
    flight.page = page;
    flight.key = Some(key);
    relay_payload(domain, env, decision, to, key, body);
}

fn change_choice(domain: &Domain, task: u64, changes: &[crate::ChangeCall]) -> FlightRender {
    let Some(context) = domain.core.tasks.delegation(task) else { return FlightRender::Plain };
    let mut producers = List::with_capacity(u32::try_from(changes.len()).expect("admitted change batch"));
    for change in changes {
        let charter = change.producer.unwrap_or(domain.forge.charters().change);
        let executor = jig_domain_tasks::AuthorityExecutor::Charter(charter);
        let permission = jig_domain_authority::Executor::Charter(charter);
        let mut missing = !context.authority.delegation.kinds.contains(&executor)
            || !domain.core.authority.rules().ceiling.delegation.kinds.contains(&permission);
        if let Some(policy) = domain.core.authority.policy(context.project) {
            missing |= !policy.ceiling.delegation.kinds.contains(&permission);
        }
        let producer = if missing { Some(charter) } else { None };
        producers.push(producer).expect("bounded producer context");
    }
    FlightRender::Change(Box::new(ChangeChoice {
        named: match changes.first() {
            Some(change) => change.repository.clone(),
            None => None,
        },
        project: context.project,
        producers: producers.into_boxed(),
    }))
}

fn person_roles(domain: &Domain, task: u64, batch: &[crate::Delegate]) -> FlightRender {
    let Some(context) = domain.core.tasks.delegation(task) else { return FlightRender::Plain };
    let mut roles = List::with_capacity(u32::try_from(batch.len()).expect("admitted delegate batch"));
    for member in batch {
        if let jig_domain_tasks::Executor::Person(jig_domain_tasks::PersonAddress::Role(role)) = member.executor {
            let permission = jig_domain_authority::Executor::Role(role);
            let mut missing =
                !context.authority.delegation.kinds.contains(&jig_domain_tasks::AuthorityExecutor::Role(role))
                    || !domain.core.authority.rules().ceiling.delegation.kinds.contains(&permission);
            if let Some(policy) = domain.core.authority.policy(context.project) {
                missing |= !policy.ceiling.delegation.kinds.contains(&permission);
            }
            if missing {
                roles.push(role).expect("bounded person roles");
            }
        }
    }
    FlightRender::PersonRoles(roles.into_boxed())
}

/// Name refused person roles before settlement so retries keep the same feedback.
fn person_refusal(domain: &Domain, render: &FlightRender, key: CallKey, answer: SettledAnswer) -> SettledAnswer {
    let FlightRender::PersonRoles(roles) = render else { return answer };
    if roles.is_empty() {
        return answer;
    }
    let Some(CallAnswer::DelegationDenied { .. }) = super::call_answer(domain, key) else { return answer };
    let SettledAnswer::Host { error: true, body } = &answer else { return answer };
    let mut capacity = body.len();
    for role in roles {
        let digits = Decimal::of(u64::from(*role)).as_bytes().len();
        capacity = capacity
            .saturating_add(b"Person role  requires {\"role\":} in delegation.kinds; ".len())
            .saturating_add(digits.saturating_mul(2));
    }
    if capacity > skein_lib::usize_of(domain.limits.call_answer_bytes) {
        return SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) };
    }
    let mut words = Writer::new(capacity);
    for role in roles {
        let number = Decimal::of(u64::from(*role));
        if words.put(b"Person role ").is_err()
            || words.put(number.as_bytes()).is_err()
            || words.put(b" requires {\"role\":").is_err()
            || words.put(number.as_bytes()).is_err()
            || words.put(b"} in delegation.kinds; ").is_err()
        {
            return SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) };
        }
    }
    finish_refusal(words, body)
}

fn finish_refusal(mut words: Writer, body: &[u8]) -> SettledAnswer {
    if words.put(body).is_err() {
        return SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) };
    }
    SettledAnswer::Host { error: true, body: words.finish() }
}

fn admit(domain: &mut Domain, to: ReplyTo, key: CallKey, tool: &crate::Tool) -> Option<ReplyTo> {
    let refusal = if key.completion == 0 {
        Some(("call-proof", "completion is zero", true))
    } else if !super::current_proof(domain, key.task, key.attempt) {
        Some(("call-proof", "task attempt is not current", true))
    } else if crate::translate::is_core_read(tool) {
        None
    } else if !domain.core.call_room(key) {
        let reason =
            if domain.core.pending_calls.contains_key(&key) { "named call pending" } else { "call record room full" };
        Some(("core-call-room", reason, false))
    } else if super::call_needs_input(tool) && domain.archive_reads.len() >= domain.archive_reads.capacity() {
        Some(("archive-read-room", "archive read room full", false))
    } else {
        None
    };
    let Some((layer, reason, invalid)) = refusal else {
        return Some(to);
    };
    let right = to.into_token();
    let flight = domain.call_renders.remove(&right).expect("current typed flight");
    super::diagnostics::emit(
        domain,
        crate::Fact::CallAdmissionRefused {
            task: key.task,
            attempt: key.attempt,
            tool: flight.tool.clone(),
            layer,
            reason,
        },
    );
    let answer = if invalid {
        Payload::SettledCall(SettledCall {
            serial: 0,
            tool: flight.tool,
            answer: SettledAnswer::Host { error: true, body: Box::from(&b"invalid call"[..]) },
        })
    } else {
        Payload::Busy
    };
    let id = domain.hub_link.retain(answer);
    domain.work.push(Work::Fleet(fleet::Event::Relayed { to: ReplyTo::new(right), answer: id }));
    None
}

pub(crate) fn render(domain: &mut Domain, limits: &Limits, decision: &mut Decision, to: ReplyTo, answer: CallAnswer) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    // A flight dropped with its call has no host left to answer.
    let Some(flight) = domain.call_renders.get(&right) else { return };
    let key = flight.key.expect("decoded typed call");
    if !flight.render.is_core_read()
        && !is_decided(&answer)
        && let Some(call) = domain.core.call_settled.get(&key).cloned()
    {
        drop(domain.call_renders.remove(&right));
        settled(domain, limits, decision, to, call);
        return;
    }
    if let FlightRender::Settings { see: Some(see), from, .. } = &flight.render {
        match see.reading {
            super::see::SeeReading::Settings | super::see::SeeReading::Proposal | super::see::SeeReading::Person(_) => {
                emit(
                    decision,
                    limits,
                    Delivery::Host(Box::new(HostDelivery::See {
                        to,
                        line: see.line.clone(),
                        from: *from,
                        answer: Box::new(super::see::SeeAnswer::Call { tool: b"settings", answer: Box::new(answer) }),
                    })),
                );
                return;
            }
            super::see::SeeReading::Head | super::see::SeeReading::File => {}
        }
    }
    // Fresh and unsettled replay replies carry only the core's part.
    let answer = match &flight.render {
        FlightRender::Document(document) => document.join(answer),
        FlightRender::Remember(remember) => remember.join(answer),
        FlightRender::Amended(changed) => {
            if let CallAnswer::Revised { task, revision } = answer {
                CallAnswer::Amended { task, revision, changed: *changed }
            } else {
                answer
            }
        }
        FlightRender::Change(change) => {
            if let CallAnswer::DelegationRefused(problem) = &answer
                && problem.why == jig_domain_tasks::Refusal::Unknown
            {
                CallAnswer::ChangeRefused {
                    problem: Box::new(problem.clone()),
                    named: change.named.clone(),
                    repositories: super::forge_route::change_repositories(domain, change.project),
                }
            } else {
                answer
            }
        }
        FlightRender::Plain
        | FlightRender::TaskRead
        | FlightRender::ProjectRead
        | FlightRender::ForgeRead
        | FlightRender::Settings { .. }
        | FlightRender::PersonRoles(_) => answer,
    };
    emit(
        decision,
        limits,
        Delivery::Host(Box::new(HostDelivery::Render {
            to,
            key,
            name: flight.name.clone(),
            tool: flight.tool.clone(),
            job: flight.job,
            page: flight.page,
            ask: if is_decided(&answer) { super::decided::how(domain, key.task) } else { AskHow::None },
            answer,
        })),
    );
}

fn is_decided(answer: &CallAnswer) -> bool {
    let CallAnswer::Decided { .. } = answer else { return false };
    true
}

pub(crate) fn rendered(domain: &mut Domain, to: ReplyTo, answer: SettledAnswer) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    let Some(flight) = domain.call_renders.remove(&right) else { return };
    let Some(key) = flight.key else {
        answered_at_decode(domain, to, flight, answer);
        return;
    };
    let answer = change_refusal(domain, &flight.render, key, answer);
    let answer = person_refusal(domain, &flight.render, key, answer);
    let mut call = SettledCall { serial: 0, tool: flight.tool, answer };
    let reading = flight.render.is_core_read();
    // A replayed decision is rendered from its stored part with the caller's current HOW.
    let recorded =
        !reading && domain.core.call_parts.contains_key(&key) && !domain.core.call_settled.contains_key(&key);
    let answer_bytes = if reading {
        Some(0)
    } else {
        match super::call_answer(domain, key) {
            Some(answer) => crate::store::call_answer_bytes(&answer),
            None => Some(0),
        }
    };
    let excess = render_excess(domain, &call, answer_bytes);
    let fits = excess.is_none();
    if let Some((reason, measured, limit)) = excess {
        call.answer = match flight.render {
            FlightRender::ForgeRead => {
                super::diagnostics::emit(
                    domain,
                    crate::Fact::ReadRefused {
                        task: key.task,
                        attempt: key.attempt,
                        tool: call.tool.clone(),
                        layer: "host-render",
                        reason,
                        findings: Box::new([]),
                        error: None,
                        job: flight.job,
                    },
                );
                read_size_refusal(domain, &call.tool, reason, measured, limit)
            }
            FlightRender::Plain
            | FlightRender::TaskRead
            | FlightRender::ProjectRead
            | FlightRender::Settings { .. }
            | FlightRender::Amended(_)
            | FlightRender::Change(_)
            | FlightRender::Document(_)
            | FlightRender::Remember(_)
            | FlightRender::PersonRoles(_) => SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) },
        };
    }
    if fits && recorded {
        domain.work.push(Work::Core(jig_domain_core::Event::SettledCall { to, key, name: flight.name, call }));
    } else {
        let id = domain.hub_link.retain(Payload::SettledCall(call));
        domain.work.push(Work::Fleet(fleet::Event::Relayed { to, answer: id }));
    }
}

/// Report the first exceeded bound, counting the tool and any retained typed answer.
fn render_excess(domain: &Domain, call: &SettledCall, answer_bytes: Option<u64>) -> Option<(&'static str, u64, u64)> {
    let bytes = call.owned_bytes().unwrap_or(u64::MAX);
    let limit = u64::from(domain.limits.call_answer_bytes);
    if !call.valid(&super::core_limits(&domain.limits)) {
        return Some(("core.call_answer_bytes exceeded", bytes, limit));
    }
    let total = match answer_bytes {
        Some(answer) => bytes.saturating_add(answer),
        None => u64::MAX,
    };
    let limit = u64::from(domain.limits.journal.transcript_bytes);
    if total > limit { Some(("root.journal.transcript_bytes exceeded", total, limit)) } else { None }
}

/// Keep refusal metadata before fallback advice, within both settled-answer bounds.
fn read_size_refusal(domain: &Domain, tool: &[u8], reason: &str, measured: u64, limit: u64) -> SettledAnswer {
    let measured = Decimal::of(measured);
    let limit = Decimal::of(limit);
    let compare = if tool == b"read_compare" {
        b"For an existing pull, use read_pull for its head, then read_pull_files with that head and page=1; advance page while more follows. ".as_slice()
    } else {
        b"".as_slice()
    };
    let parts = [
        b"Forge read refused: ".as_slice(), reason.as_bytes(), b"; measured ", measured.as_bytes(),
        b" bytes, limit ", limit.as_bytes(),
        b" bytes (including tool name; journal also counts retained typed answer). ", compare,
        b"If a pull-file page is too large, or no pull exists, use read_file to list directories at both endpoints, then read selected files at each endpoint with max_bytes.",
    ];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("fixed refusal and u64 digits");
    }
    let bound = domain.limits.call_answer_bytes.min(domain.limits.journal.transcript_bytes);
    let capacity = skein_lib::usize_of(bound).saturating_sub(tool.len());
    let mut words = Writer::new(length.min(capacity));
    for part in parts {
        let prefix = part.get(..part.len().min(words.room())).expect("bounded refusal prefix");
        words.put(prefix).expect("prefix fits remaining room");
    }
    SettledAnswer::Host { error: true, body: words.finish() }
}

/// Add the blocked producers before settlement, so replay keeps these words.
fn change_refusal(domain: &Domain, render: &FlightRender, key: CallKey, answer: SettledAnswer) -> SettledAnswer {
    let FlightRender::Change(change) = render else { return answer };
    let Some(CallAnswer::DelegationDenied { .. }) = super::call_answer(domain, key) else { return answer };
    let SettledAnswer::Host { error: true, body } = &answer else { return answer };
    let mut capacity = body.len();
    for (index, producer) in change.producers.iter().enumerate() {
        if let Some(charter) = producer {
            let member = u64::try_from(index).expect("bounded change index").checked_add(1).expect("change member");
            let length = b"Change member : producer charter ; "
                .len()
                .checked_add(Decimal::of(member).as_bytes().len())
                .expect("bounded member number")
                .checked_add(Decimal::of(u64::from(*charter)).as_bytes().len())
                .expect("bounded charter number");
            capacity = capacity.checked_add(length).expect("admitted change context");
        }
    }
    if capacity > usize::try_from(domain.limits.call_answer_bytes).expect("u32 fits") {
        return SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) };
    }
    let mut words = Writer::new(capacity);
    for (index, producer) in change.producers.iter().enumerate() {
        if let Some(charter) = producer {
            let member = u64::try_from(index).expect("bounded change index").checked_add(1).expect("change member");
            if words.put(b"Change member ").is_err()
                || words.put(Decimal::of(member).as_bytes()).is_err()
                || words.put(b": producer charter ").is_err()
                || words.put(Decimal::of(u64::from(*charter)).as_bytes()).is_err()
                || words.put(b"; ").is_err()
            {
                return SettledAnswer::Host { error: true, body: Box::from(b"busy".as_slice()) };
            }
        }
    }
    finish_refusal(words, body)
}

/// An answer made at decode decides nothing (domain/routing.md, 2).
fn answered_at_decode(domain: &mut Domain, to: ReplyTo, flight: Flight, answer: SettledAnswer) {
    let mut call = SettledCall { serial: 0, tool: flight.tool, answer };
    call.answer = if call.valid(&super::core_limits(&domain.limits)) {
        match call.answer {
            answer @ SettledAnswer::Host { .. } => answer,
            SettledAnswer::Delivery { .. } => {
                SettledAnswer::Host { error: true, body: Box::from(&b"invalid call"[..]) }
            }
        }
    } else {
        SettledAnswer::Host { error: true, body: Box::from(&b"invalid call"[..]) }
    };
    let id = domain.hub_link.retain(Payload::SettledCall(call));
    domain.work.push(Work::Fleet(fleet::Event::Relayed { to, answer: id }));
}

pub(crate) fn settled(domain: &mut Domain, limits: &Limits, decision: &mut Decision, to: ReplyTo, call: SettledCall) {
    let id = domain.hub_link.retain(Payload::SettledCall(call));
    emit(decision, limits, Delivery::Fleet(fleet::Event::Relayed { to, answer: id }));
}

/// Core's refusal supplies its own live address; no settled record is kept.
pub(crate) fn refused(domain: &mut Domain, limits: &Limits, decision: &mut Decision, to: ReplyTo, name: Box<[u8]>) {
    let id = domain.hub_link.retain(Payload::Busy);
    domain.refused_calls.insert(id, name).expect("one refusal per payload");
    emit(decision, limits, Delivery::Fleet(fleet::Event::Relayed { to, answer: id }));
}

/// Capacity refusal already carries its validated original address; no flight is needed.
pub(crate) fn busy(channel: Token, run: Token, attempt: Token, name: Box<[u8]>) -> Delivery {
    Delivery::Link { channel, down: host::LinkDown::Relayed { run, attempt, call: name, answer: host::Relayed::Busy } }
}

/// Assemble the answer its owner kept; the core chooses its durability route.
pub(crate) fn relayed(
    domain: &mut Domain,
    channel: Token,
    run: Token,
    attempt: Token,
    name: Box<[u8]>,
    answer: Token,
) -> Delivery {
    domain.hub_link.forget_call(&name);
    if domain.settling.active() {
        drop(super::take_payload(domain, answer));
        return busy(channel, run, attempt, name);
    }
    if let Some(name) = domain.refused_calls.remove(&answer) {
        drop(super::take_payload(domain, answer));
        return Delivery::Link {
            channel,
            down: host::LinkDown::Relayed {
                run,
                attempt,
                call: name,
                answer: host::Relayed::Answer { error: true, text: Box::from(&b"busy"[..]) },
            },
        };
    }
    crate::link::delivery(domain, jig_domain_hub_link::Event::Relayed { channel, run, attempt, call: name, answer })
}

/// A read's owned value crosses the journal door before any durable decision.
pub(crate) fn read(domain: &mut Domain, to: ReplyTo, answer: CallAnswer) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    if let FlightRender::Settings { see: Some(_), .. } = &domain.call_renders.get(&right).expect("decoded read").render
    {
        super::see::forge::answered(domain, to, answer);
        return;
    }
    let key = domain.call_renders.get(&right).expect("a read answers its decoded flight").key.expect("decoded read");
    if let CallAnswer::ReadRepositoryRefused { .. } = &answer {
        super::diagnostics::read(domain, key, "forge-route", "repository name unresolved", None, Box::new([]));
    }
    let flight = domain.call_renders.get(&right).expect("a read answers its decoded flight");
    let request = crate::boundary::delivery_output(Delivery::Host(Box::new(HostDelivery::Render {
        to,
        key: flight.key.expect("decoded read"),
        name: flight.name.clone(),
        tool: flight.tool.clone(),
        job: flight.job,
        page: flight.page,
        ask: AskHow::None,
        answer,
    })));
    assert!(super::now(domain, request), "read render door reserved");
}

/// A settings read crosses the same journal door as other fresh reads.
pub(crate) fn settings(domain: &mut Domain, to: ReplyTo, answer: Box<jig_domain_core::SettingsAnswer>) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    let flight = domain.call_renders.get(&right).expect("settings read flight");
    let FlightRender::Settings { see, from, requested, asked } = &flight.render else {
        unreachable!("settings rendering context")
    };
    let mut application = super::tools::settings_value(domain, &answer);
    let mut answer = answer;
    let newest = domain.core.current.newest();
    if let jig_domain_core::SettingsAnswer::Unknown = answer.as_ref()
        && let Some((
            setting @ jig_domain_core::settings::Setting::Application {
                kind: kind @ (2 | 6 | 7 | 8 | 9 | 10 | 11 | 12),
                scope,
                number,
            },
            revision,
        )) = *asked
        && (revision == 0 || revision == newest)
        && match kind {
            11 => scope == 0 && number > 0 && number <= domain.limits.templates,
            9 => scope == 0 && number > 0 && number <= domain.limits.tasks.charters,
            2 | 6 | 7 | 8 | 10 | 12 => domain.core.people.project_members(scope).is_some(),
            _ => false,
        }
        && let Some(value) = match kind {
            2 => match domain.forge.setting(kind, scope, number) {
                Some(value) => Some(Some(value.clone())),
                None => None,
            },
            9 | 11 => Some(domain.forge.setting(kind, scope, number).cloned()),
            10 if number == 1 => Some(domain.forge.setting(kind, scope, number).cloned()),
            6..=8 if number == 1 => match temper_engine_domain_forge::settings::project_stock(kind) {
                Some(value) => Some(Some(value)),
                None => None,
            },
            _ => None,
        }
    {
        application = value;
        answer = Box::new(jig_domain_core::SettingsAnswer::Setting(jig_domain_core::reads::SettingDocument {
            overlay: None,
            setting,
            revision: newest,
            value: None,
            application: None,
            proposed: None,
        }));
    }
    let repositories = settings_repositories(domain, flight, &answer);
    let people = settings_people(domain, flight, &answer);
    let forge_people = settings_forge_people(domain, flight, &answer);
    let offer = settings_offer(domain, flight, &answer);
    let delivery = match see {
        Some(see) => HostDelivery::See {
            to,
            line: see.line.clone(),
            from: *from,
            answer: Box::new(super::see::SeeAnswer::Settings(Box::new(super::see::SeeSettings {
                answer: *answer,
                repositories,
                people,
                forge_people,
                offer,
                application,
                requested: *requested,
            }))),
        },
        None => HostDelivery::Settings {
            to,
            repositories,
            people,
            forge_people,
            offer,
            application,
            answer,
            from: *from,
            requested: *requested,
        },
    };
    let request = crate::boundary::delivery_output(Delivery::Host(Box::new(delivery)));
    assert!(super::now(domain, request), "settings read render door reserved");
}

/// Enrich only authorized inventories, scoped to the caller's retained project.
fn settings_repositories(
    domain: &Domain,
    flight: &Flight,
    answer: &jig_domain_core::SettingsAnswer,
) -> Box<[RepositoryIdentity]> {
    if let jig_domain_core::SettingsAnswer::Inventory(_) = answer {
        let Some(context) = domain.core.tasks.delegation(flight.task) else { return Box::new([]) };
        let repositories = domain.forge.readable_repositories(context.project);
        let mut identities = List::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
        for repository in &repositories {
            identities
                .push(RepositoryIdentity {
                    host: repository.host.clone(),
                    owner: repository.owner.clone(),
                    name: repository.name.clone(),
                })
                .expect("one identity per readable repository");
        }
        identities.into_boxed()
    } else {
        Box::new([])
    }
}

/// Project roles and forge identities are context of an authorized inventory alone.
fn settings_people(domain: &Domain, flight: &Flight, answer: &jig_domain_core::SettingsAnswer) -> Box<[ProjectPerson]> {
    use jig_domain_core::SettingsAnswer;
    match answer {
        SettingsAnswer::Inventory(_) => {}
        SettingsAnswer::Setting(_)
        | SettingsAnswer::Revision(_)
        | SettingsAnswer::Revisions(_)
        | SettingsAnswer::Unknown => return Box::new([]),
    }
    let Some(context) = domain.core.tasks.delegation(flight.task) else { return Box::new([]) };
    let Some(members) = domain.core.people.project_members(context.project) else { return Box::new([]) };
    let Some(task) = domain.core.tasks.task(flight.task) else { return Box::new([]) };
    let requester = match task.requester {
        jig_domain_tasks::Party::Person(person) => Some(person),
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => None,
    };
    let mut people = List::with_capacity(u32::try_from(members.len()).expect("bounded project members"));
    for member in members {
        match domain.core.people.kind(member.person) {
            Some(jig_domain_people::Kind::Person) => {}
            Some(jig_domain_people::Kind::Service) | None => continue,
        }
        let Some(identity) = domain.core.people.identity(member.person) else { continue };
        people
            .push(ProjectPerson {
                person: member.person,
                role: member.role.number(),
                login: identity.login.clone(),
                forges: super::forge_route::person_forges(domain, context.project, member.person),
                you: requester == Some(member.person),
            })
            .expect("at most one person per holding");
    }
    people.into_boxed()
}

/// The task's payer and model authority bound an authorized inventory's offer.
fn settings_offer(domain: &Domain, flight: &Flight, answer: &jig_domain_core::SettingsAnswer) -> Box<[OfferedModel]> {
    use jig_domain_core::SettingsAnswer;
    match answer {
        SettingsAnswer::Inventory(_) => {}
        SettingsAnswer::Setting(_)
        | SettingsAnswer::Revision(_)
        | SettingsAnswer::Revisions(_)
        | SettingsAnswer::Unknown => return Box::new([]),
    }
    let Some(payer) = domain.core.tasks.payer(flight.task) else { return Box::new([]) };
    let Some(context) = domain.core.tasks.delegation(flight.task) else { return Box::new([]) };
    let offered = domain.core.offer(payer, context.project, context.authority.models);
    let mut offer = List::with_capacity(offered.len());
    for row in &offered {
        let model = domain.core.settings.models.get(usize::from(row.model)).expect("offered configured model");
        let provider = domain.core.settings.llm_providers.get(model.provider).expect("offered model provider");
        let effort = match model.default_effort() {
            Some(effort) => Some(skein_lib::bytes::copy_of(effort)),
            None => None,
        };
        offer
            .push(OfferedModel {
                name: model.name.clone(),
                provider: provider.name.clone(),
                efforts: model.efforts.clone(),
                effort,
            })
            .expect("one row per offered model");
    }
    offer.into_boxed()
}

/// Owned forge logins are named once per readable host in the caller's project.
fn settings_forge_people(
    domain: &Domain,
    flight: &Flight,
    answer: &jig_domain_core::SettingsAnswer,
) -> Box<[ForgePerson]> {
    use jig_domain_core::SettingsAnswer;
    match answer {
        SettingsAnswer::Inventory(_) => {}
        SettingsAnswer::Setting(_)
        | SettingsAnswer::Revision(_)
        | SettingsAnswer::Revisions(_)
        | SettingsAnswer::Unknown => return Box::new([]),
    }
    let Some(context) = domain.core.tasks.delegation(flight.task) else { return Box::new([]) };
    let repositories = domain.forge.readable_repositories(context.project);
    let mut people =
        List::<ForgePerson>::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
    for repository in &repositories {
        let Some(login) = domain.forge.owned_person(repository.provider.forge) else { continue };
        let mut named = false;
        for person in &people {
            named |= person.host == repository.host;
        }
        if !named {
            people
                .push(ForgePerson { host: repository.host.clone(), login: login.into() })
                .expect("at most one person per host");
        }
    }
    people.into_boxed()
}

/// Load one settings source without applying it or adding note assembly state.
pub(crate) fn settings_note(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, to: ReplyTo, call: Call) {
    let crate::Tool::ProposeSettingsNote { note, ref reason, .. } = call.tool else { unreachable!("note source") };
    if reason.len() > skein_lib::usize_of(env.limits.tasks.message_bytes) {
        note_refused(domain, to, b"settings reason exceeds the message bound");
        return;
    }
    let right = to.into_token();
    let flight = domain.call_renders.get(&right).expect("settings note flight");
    let key = flight.key.expect("decoded settings note");
    if let Some(part) = domain
        .core
        .authorize_tool(key, jig_domain_core::ToolKind::Recall { by: jig_domain_notes::Recall::Name { name: note } })
    {
        domain.work.push(Work::Core(jig_domain_core::Event::NamedAnswer { to: ReplyTo::new(right), key, part }));
        return;
    }
    let read = crate::boundary::Read::SettingsNote { to: right, call: Box::new(call) };
    match domain.archive_reads.insert(Some(read)) {
        Ok(id) => emit(
            decision,
            &env.limits,
            Delivery::Load {
                waiter: id.token(),
                range: crate::Range::Core(crate::CoreRange::Notes(jig_domain_notes::Range::Entry { name: note })),
                after: None,
            },
        ),
        Err(_) => note_refused(domain, ReplyTo::new(right), b"settings note load busy"),
    }
}

pub(crate) fn note_refused(domain: &mut Domain, to: ReplyTo, words: &[u8]) {
    rendered(domain, to, SettledAnswer::Host { error: true, body: words.into() });
}

pub(crate) fn settings_note_loaded(
    domain: &mut Domain,
    to: ReplyTo,
    call: Call,
    rows: Box<[crate::Record]>,
    cut: bool,
) {
    let right = to.into_token();
    let Some(flight) = domain.call_renders.get(&right) else { return };
    let task = flight.task;
    let crate::Tool::ProposeSettingsNote { note, revision, .. } = call.tool else { unreachable!("note source") };
    if cut || rows.len() != 1 {
        note_refused(domain, ReplyTo::new(right), b"settings note missing or exceeds load bounds");
        return;
    }
    let Some(crate::Record::Core(jig_domain_core::Record::Notes(jig_domain_notes::Record::Entry(entry)))) =
        rows.into_iter().next()
    else {
        note_refused(domain, ReplyTo::new(right), b"settings note missing");
        return;
    };
    if entry.name != note
        || !jig_domain_notes::valid_entry(&entry, &domain.limits.notes)
        || !domain.core.note_readable(task, &entry.scope)
    {
        note_refused(domain, ReplyTo::new(right), b"settings note inaccessible");
        return;
    }
    if entry.revision != revision {
        note_refused(domain, ReplyTo::new(right), b"settings note revision moved; recall and submit its revision");
        return;
    }
    domain.call_renders.get_mut(&right).expect("note flight").note_decoding = true;
    assert!(
        super::now(
            domain,
            Request::Worker(crate::WorkerRequest::Host(Box::new(HostRequest::SettingsNote {
                to: ReplyTo::new(right),
                call: Box::new(call),
                body: entry.body,
            })))
        ),
        "note decode door reserved"
    );
}
