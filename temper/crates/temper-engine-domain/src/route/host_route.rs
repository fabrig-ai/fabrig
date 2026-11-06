//! Typed host calls cross the protocol boundary before tool decisions and after
//! their answers (jig's domain/engine.md, 7.3; jig's domain/hosts.md, 2).
use super::{
    Call, CallAnswer, CallKey, Decision, Delivery, Domain, Env, Limits, Payload, Request, Work, emit, relay_payload,
};
use alloc::boxed::Box;
use jig_domain_core::{SettledAnswer, SettledCall};
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use skein_lib::{Decimal, List, ReplyTo, Token, Writer};

/// Root to protocol: a relayed call to decode, or one the fleet dropped.
#[derive(PartialEq, Eq, Debug)]
pub enum HostRequest {
    Decode { to: ReplyTo, task: u64, attempt: u64, call: fleet::Call },
    Dropped { call: fleet::Call },
}

/// An adopted repository's canonical resource identity for settings discovery
/// (temper/docs/design/domain/agent.md, 8.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RepositoryIdentity {
    /// The host in resource names, including any port.
    pub host: Box<[u8]>,
    /// The repository's forge owner.
    pub owner: Box<[u8]>,
    /// The repository's name within that owner.
    pub name: Box<[u8]>,
}

/// Root to protocol after the decision commits: render a decided call's
/// answer, or carry a workspace delivery's settled answer to its host, which
/// the hub's link names only as text (jig's `domain/hosts.md`, 2).
#[derive(PartialEq, Eq, Debug)]
pub enum HostDelivery {
    /// Settings reads have no durable call part; inventories include project repositories.
    Settings {
        to: ReplyTo,
        answer: Box<jig_domain_core::SettingsAnswer>,
        repositories: Box<[RepositoryIdentity]>,
        application: Option<temper_engine_domain_forge::settings::SizedValue>,
        from: u32,
        requested: Option<u64>,
    },
    Render {
        to: ReplyTo,
        key: CallKey,
        name: Box<[u8]>,
        tool: Box<[u8]>,
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

#[derive(Debug)]
pub(crate) struct Flight {
    pub task: u64,
    pub attempt: u64,
    pub key: Option<CallKey>,
    pub render: FlightRender,
    pub name: Box<[u8]>,
    pub tool: Box<[u8]>,
}

/// Only one tool's rendering context is kept per flight. Boxing the change
/// keeps the slot the same size as the existing amendment context.
#[derive(Debug)]
pub(crate) enum FlightRender {
    Plain,
    TaskRead,
    ForgeRead,
    Settings { from: u32, requested: Option<u64>, asked: Option<(jig_domain_core::settings::Setting, u32)> },
    Amended(crate::Changed),
    Change(Box<ChangeChoice>),
    PersonRoles(Box<[u32]>),
}

impl FlightRender {
    fn is_core_read(&self) -> bool {
        match self {
            Self::TaskRead | Self::Settings { .. } => true,
            Self::Plain | Self::ForgeRead | Self::Amended(_) | Self::Change(_) | Self::PersonRoles(_) => false,
        }
    }
}

#[derive(Debug)]
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
        render: FlightRender::Plain,
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
    if flight.key.is_some() {
        return;
    }
    let body = super::checked_call(body, &env.limits);
    let key =
        CallKey { task: flight.task, attempt: flight.attempt, completion: body.completion, position: body.position };
    let Some(to) = admit(domain, to, key, &body.tool) else { return };
    let render = if let crate::Tool::ReadTask { .. } = &body.tool {
        FlightRender::TaskRead
    } else if let crate::Tool::ReadForge { .. } | crate::Tool::ReadNamed { .. } = &body.tool {
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
        FlightRender::Settings { from: *from, requested, asked }
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
        && let Some(call) = domain.core.call_settled.get(&key).cloned()
    {
        drop(domain.call_renders.remove(&right));
        settled(domain, limits, decision, to, call);
        return;
    }
    // Fresh and unsettled replay replies carry only the core's part.
    let answer = match &flight.render {
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
            answer,
        })),
    );
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
    let recorded = !reading && domain.core.call_parts.contains_key(&key);
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
                    },
                );
                read_size_refusal(domain, &call.tool, reason, measured, limit)
            }
            FlightRender::Plain
            | FlightRender::TaskRead
            | FlightRender::Settings { .. }
            | FlightRender::Amended(_)
            | FlightRender::Change(_)
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

/// An answer made at decode decides nothing (domain/root.md, 5.2).
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
        answer,
    })));
    assert!(super::now(domain, request), "read render door reserved");
}

/// A settings read crosses the same journal door as other fresh reads.
pub(crate) fn settings(domain: &mut Domain, to: ReplyTo, answer: Box<jig_domain_core::SettingsAnswer>) {
    let right = to.into_token();
    let to = ReplyTo::new(right);
    let flight = domain.call_renders.get(&right).expect("settings read flight");
    let FlightRender::Settings { from, requested, asked } = flight.render else {
        unreachable!("settings rendering context")
    };
    let mut application = super::tools::settings_value(domain, &answer);
    let mut answer = answer;
    let newest = domain.core.current.newest();
    if let jig_domain_core::SettingsAnswer::Unknown = answer.as_ref()
        && let Some((
            setting @ jig_domain_core::settings::Setting::Application { kind: kind @ (6 | 7), scope, number: 1 },
            revision,
        )) = asked
        && (revision == 0 || revision == newest)
        && domain.core.people.project_members(scope).is_some()
    {
        application = temper_engine_domain_forge::settings::project_stock(kind);
        answer = Box::new(jig_domain_core::SettingsAnswer::Setting(jig_domain_core::reads::SettingDocument {
            setting,
            revision: newest,
            value: None,
            application: None,
            proposed: None,
        }));
    }
    let request = crate::boundary::delivery_output(Delivery::Host(Box::new(HostDelivery::Settings {
        to,
        repositories: settings_repositories(domain, flight, &answer),
        application,
        answer,
        from,
        requested,
    })));
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
