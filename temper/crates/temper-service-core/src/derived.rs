//! Carriers computed before document overrides (temper/docs/design/protocol/configuration/derived.md, 2–5).
use crate::agent::Declaration;
use jig_protocol_config::Limits;

/// Jig's stock with temper's counts (configuration/limits.md, one stock).
#[must_use]
pub fn stock() -> Limits {
    let mut limits = jig_protocol_config::stock();
    // Fabrig's own development's 48 calls per response.
    limits.hub.run_calls = 48;
    // Fabrig's own development's workers have at most 12 places; the
    // largest round ceiling whose 48-call engine channel fits u32 is 32.
    limits.core.fleet.slots = 32;
    // Fabrig's own development's agent uses smith's standard profile,
    // whose credential accounts are fixed at four, independent of conversations.
    limits.hub.accounts = 4;
    limits
}

/// The projection of an agent's declaration used by the carriers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sources {
    pub turn: u32,
    pub payload: u32,
    pub inbox: u32,
    pub unacknowledged: u32,
}
/// Smith's standard agent with init's declarations and endpoints, a 63,740-byte
/// tool payload and 48,000-token endpoint outputs (derived.md, 3).
pub const STOCK: Sources = Sources { turn: 18_431_842, payload: 63_740, inbox: 82, unacknowledged: 8 };
/// Fabrig’s own development of 2026-11-16: T + 1,024 bytes of frame margin
/// (configuration/limits.md, the stock covers fabrig’s development).
pub const OUTPUT_FLOOR: u32 = 18_432_866;
/// The stock number of goals in flight (configuration/derived.md, 5.4).
pub const LIVE_GOALS: u32 = 40;
/// Deployment store envelope (configuration/limit-settings.md, 3), from
/// draft/config.md, 5.2's plan for 100 live goals.
pub const STORE_LIVE_GOALS: u32 = 100;
impl Sources {
    /// A process takes the larger of its setting and its agent's payload.
    #[must_use]
    pub fn with_payload(self, tool_payload: u32) -> Self {
        Self { payload: self.payload.max(tool_payload), ..self }
    }
    #[must_use]
    pub fn agent(agent: &Declaration) -> Self {
        Self {
            turn: agent.turn_body,
            payload: agent.tool_payload,
            inbox: agent.bounds.inbox,
            unacknowledged: agent.bounds.unacknowledged,
        }
    }
}
#[must_use]
pub fn resume(turn: u32) -> Option<u32> {
    Some(turn.checked_add(4)?.max(4_194_304))
}
#[must_use]
pub fn transcript(resume: u32) -> Option<u64> {
    let resume = u64::from(resume);
    resume.checked_add(12_u64.checked_mul(resume / 4)?)
}
#[must_use]
pub fn turn_queue(turn: u32, unacknowledged: u32) -> Option<u64> {
    u64::from(turn).checked_mul(u64::from(unacknowledged))
}
#[must_use]
pub fn message(payload: u32) -> Option<u32> {
    payload.checked_sub(2)
}
#[must_use]
pub fn inbox(payload: u32, inbox: u32) -> Option<u32> {
    message(payload)?.checked_mul(inbox)
}
#[must_use]
pub fn loaded(turn: u32, record: u32) -> Option<u32> {
    turn.checked_add(record)
}
#[must_use]
pub fn read(turn: u32, row: u64, record: u64) -> Option<u64> {
    Some(u64::from(turn).checked_mul(2)?.max(u64::from(turn).checked_add(row)?.checked_add(record)?))
}
#[must_use]
pub fn outcome(payload: u32, stock: u64) -> u64 {
    stock.max(u64::from(payload))
}
#[must_use]
pub fn call_answer(payload: u32, stock: u32) -> u32 {
    stock.max(payload)
}

/// The links' whole frames, the smallest resumed start and the checkout's answer,
/// measured over the base's owners before any document override.
#[must_use]
pub fn output(limits: &Limits) -> Option<u32> {
    use jig_protocol_worker_wire::{CEILINGS, sizes, v17};
    let own = crate::limits::stock();
    let payload = temper_protocol_worker_wire::limits(&checkout_bounds(&own.checkout, &own.peer))?;
    let sizes = worker_sizes(limits, limits.hub.slots, &payload, own.peer.forges)?;
    let bodies = sizes::limits(&sizes)?;
    let mut room = u32::try_from(v17::Credential::encoded_bound(&bodies)?).ok()?.checked_add(22)?.max(270);
    for rule in jig_protocol_worker_wire::KINDS {
        // Assignments, job specs and opening figures are checked against room when sent.
        if rule.body != jig_protocol_worker_wire::Kind::Assignment
            && rule.body != jig_protocol_worker_wire::Kind::Job
            && rule.body != jig_protocol_worker_wire::Kind::Figures
        {
            room = room.max(sizes::largest_body(&sizes, rule.body)?.checked_add(8)?);
        }
    }
    let resumed = sizes::smallest_resumed(
        limits.core.call_records,
        u32::try_from(limits.core.fleet.call_name_bytes).ok()?,
        CEILINGS.call_tool,
        limits.core.call_answer_bytes,
    )?;
    Some(room.max(resumed).max(u32::try_from(checkout_answer(limits, &payload)?).ok()?))
}

fn failure(figure: &[u8], source: &'static [u8]) -> jig_protocol_config::Problem {
    jig_protocol_config::Problem {
        path: jig_protocol_config::Path::root().member(b"limits").member(b"overrides").member(figure),
        what: jig_protocol_config::What::Because(source),
    }
}
fn both(limits: &mut Limits, sources: Sources) -> Result<(), jig_protocol_config::Problem> {
    limits.hub.event_bytes = u64::from(
        message(sources.payload)
            .ok_or(failure(b"hub.event_bytes", b"cannot derive from the agent's tool payload (P)"))?,
    );

    limits.hub.turn_bytes = u64::from(sources.turn);
    limits.hub.transcript_bytes = transcript(
        resume(sources.turn)
            .ok_or(failure(b"core.resume_bytes", b"cannot derive from the agent's largest turn (T)"))?,
    )
    .ok_or(failure(b"hub.transcript_bytes", b"cannot derive from the agent's largest turn (T)"))?;
    limits.hub.turn_queue_bytes = turn_queue(sources.turn, sources.unacknowledged).ok_or(failure(
        b"hub.turn_queue_bytes",
        b"cannot derive from the agent's unacknowledged turns and largest turn (T)",
    ))?;
    limits.hub.outcome_bytes = outcome(sources.payload, limits.hub.outcome_bytes);
    limits.core.call_answer_bytes = call_answer(sources.payload, limits.core.call_answer_bytes);
    limits.io.output = output(limits)
        .ok_or(failure(
            b"io.output",
            b"cannot derive from the agent's largest turn (T) within the worker wire's ceilings",
        ))?
        .max(OUTPUT_FLOOR);
    Ok(())
}
/// Scale a stock count, rounding up; multiplication of two u32 values fits u64.
fn count(stock: u32, live_goals: u32) -> Result<u32, jig_protocol_config::Problem> {
    let scaled = u64::from(stock)
        .checked_mul(u64::from(live_goals))
        .expect("two u32 values fit u64")
        .div_ceil(u64::from(LIVE_GOALS));
    u32::try_from(scaled).or(Err(failure(b"live_goals", b"scaled count exceeds u32")))
}
/// The forge's work-in-flight sources, independent of the engine's vocabulary.
/// Scaling keeps its stored caps and its landing tie (configuration/derived.md, 5.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ForgeCounts {
    pub tasks: u32,
    pub holds: u32,
    pub subscriptions: u32,
    pub entries: u32,
    pub client_entries: u32,
    pub client_resources: u32,
    pub issues: u32,
    pub landings: u32,
    pub landing_members: u32,
}
impl ForgeCounts {
    fn scale(&mut self, live_goals: u32) -> Result<(), jig_protocol_config::Problem> {
        self.tasks = count(self.tasks, live_goals)?.min(1024);
        self.holds = count(self.holds, live_goals)?.min(256);
        self.subscriptions = count(self.subscriptions, live_goals)?;
        self.entries = count(self.entries, live_goals)?;
        self.client_entries = count(self.client_entries, live_goals)?;
        self.client_resources = count(self.client_resources, live_goals)?;
        self.issues = count(self.issues, live_goals)?;
        self.landing_members = self.landing_members.min(self.holds);
        self.landings = count(self.landings, live_goals)?.min(
            self.holds
                .checked_div(self.landing_members)
                .ok_or(failure(b"self.landing_members", b"requires at least 1"))?,
        );
        Ok(())
    }
}
/// Scale work in flight, keeping the format caps and ties (configuration/derived.md, 5.4).
/// The caller supplies the stock forge; jig's stock stays in jig.
pub fn scaled(jig: &mut Limits, forge: &mut ForgeCounts, live_goals: u32) -> Result<(), jig_protocol_config::Problem> {
    if live_goals == 0 {
        return Err(failure(b"live_goals", b"requires at least 1"));
    }
    let stock = jig_protocol_config::stock().core.tasks;
    let tasks = &mut jig.core.tasks;
    tasks.tasks = count(stock.tasks, live_goals)?;
    tasks.delegates = stock.delegates.min(tasks.tasks.saturating_sub(1));
    tasks.project_tasks = count(stock.project_tasks, live_goals)?;
    tasks.project_goals = count(stock.project_goals, live_goals)?;
    tasks.placed_holds = count(stock.placed_holds, live_goals)?;
    tasks.project_placed_holds = count(stock.project_placed_holds, live_goals)?;
    tasks.party_placed_holds = count(stock.party_placed_holds, live_goals)?;
    tasks.exemptions = count(stock.exemptions, live_goals)?.min(4096).min(tasks.tasks);
    tasks.funders = count(stock.funders, live_goals)?;
    tasks.pools = count(stock.pools, live_goals)?;
    tasks.hold_waiters = count(stock.hold_waiters, live_goals)?;
    jig.core.people.operations = tasks.delegates.saturating_add(1).max(tasks.project_goals).min(tasks.tasks);
    jig.core.people.inbox_tasks =
        tasks.tasks.checked_mul(2).ok_or(failure(b"people.inbox_tasks", b"twice tasks.tasks exceeds u32"))?;
    forge.scale(live_goals)?;
    Ok(())
}
/// Steps 1–3 of configuration/derived.md, 4. Sources have already selected the
/// agent, or the engine's stock turn and the setting's payload alone.
pub fn base(
    live_goals: u32,
    tool_payload: u32,
    sources: Sources,
    forge: &mut ForgeCounts,
) -> Result<Limits, jig_protocol_config::Problem> {
    let mut limits = stock();
    scaled(&mut limits, forge, live_goals)?;
    engine(&mut limits, sources.with_payload(tool_payload))?;
    Ok(limits)
}
/// Read the two source figures before either limits reader derives its base.
/// Invalid values stay with the application reader for its path's diagnostics.
#[must_use]
pub fn figures(document: &skein_json::Document) -> (u32, u32) {
    use crate::startup::member;
    let Some(application) = member(document, 0, b"application") else { return (LIVE_GOALS, STOCK.payload) };
    let goals = figure(document, application, b"live_goals", LIVE_GOALS);
    let payload = figure(document, application, b"tool_payload", STOCK.payload);
    (if goals == 0 { LIVE_GOALS } else { goals }, if payload == 0 { STOCK.payload } else { payload })
}
/// Select the setting alone for an engine with no agent; otherwise keep the
/// agent's payload when it is larger (configuration/derived.md, 2).
#[must_use]
pub fn process_sources(document: &skein_json::Document, sources: Sources) -> Sources {
    use crate::startup::member;
    let payload = figures(document).1;
    if let Some(at) = member(document, 0, b"workers")
        && let Ok(workers) = jig_protocol_config::read::list(document, at, document.len())
        && !workers.is_empty()
    {
        return sources.with_payload(payload);
    }
    Sources { payload, ..STOCK }
}
fn figure(document: &skein_json::Document, application: u32, name: &[u8], stock: u32) -> u32 {
    use crate::startup::member;
    let Some(limits) = member(document, application, b"limits") else { return stock };
    let Some(overrides) = member(document, limits, b"overrides") else { return stock };
    let Some(at) = member(document, overrides, name) else { return stock };
    if let Ok(value) = jig_protocol_config::read::integer(document, at, u64::from(u32::MAX)) {
        return u32::try_from(value).expect("checked u32 figure");
    }
    stock
}
/// Same-set ties follow overrides before jig validates the resulting limits.
pub fn follow_tasks(document: &skein_json::Document, at: u32, limits: &mut Limits) {
    let tasks = figure(document, at, b"tasks.tasks", limits.core.tasks.tasks);
    let delegates = if overridden(document, at, b"tasks.delegates") {
        Some(figure(document, at, b"tasks.delegates", limits.core.tasks.delegates))
    } else {
        None
    };
    follow_counts(
        limits,
        tasks,
        delegates,
        figure(document, at, b"tasks.project_goals", limits.core.tasks.project_goals),
        overridden(document, at, b"people.operations"),
        overridden(document, at, b"people.inbox_tasks"),
        overridden(document, at, b"tasks.exemptions"),
    );
}
/// Settings lists and the worker document keep the same task-count ties.
pub fn follow_task_overrides(limits: &mut Limits, overrides: &[jig_domain_core_vocabulary::settings::Override]) {
    let tasks = limit_figure(overrides, b"tasks.tasks").unwrap_or(limits.core.tasks.tasks);
    let delegates = limit_figure(overrides, b"tasks.delegates");
    let goals = limit_figure(overrides, b"tasks.project_goals").unwrap_or(limits.core.tasks.project_goals);
    follow_counts(
        limits,
        tasks,
        delegates,
        goals,
        limit_figure(overrides, b"people.operations").is_some(),
        limit_figure(overrides, b"people.inbox_tasks").is_some(),
        limit_figure(overrides, b"tasks.exemptions").is_some(),
    );
}
fn limit_figure(overrides: &[jig_domain_core_vocabulary::settings::Override], name: &[u8]) -> Option<u32> {
    for item in overrides {
        if item.name.as_ref() == name {
            return u32::try_from(item.value).ok();
        }
    }
    None
}
fn follow_counts(
    limits: &mut Limits,
    tasks: u32,
    delegates: Option<u32>,
    goals: u32,
    operations: bool,
    inbox: bool,
    exemptions: bool,
) {
    let delegates = match delegates {
        Some(delegates) => delegates,
        None => {
            limits.core.tasks.delegates =
                jig_protocol_config::stock().core.tasks.delegates.min(tasks.saturating_sub(1));
            limits.core.tasks.delegates
        }
    };
    if !operations {
        limits.core.people.operations = delegates.saturating_add(1).max(goals).min(tasks);
    }
    if !inbox {
        limits.core.people.inbox_tasks = tasks.saturating_mul(2);
    }
    if !exemptions {
        limits.core.tasks.exemptions = limits.core.tasks.exemptions.min(tasks);
    }
}
/// Apply the engine's carriers to its base, before jig reads the overrides.
pub fn engine(limits: &mut Limits, sources: Sources) -> Result<(), jig_protocol_config::Problem> {
    limits.core.resume_bytes = resume(sources.turn)
        .ok_or(failure(b"core.resume_bytes", b"cannot derive from the agent's largest turn (T)"))?;
    limits.core.tasks.message_bytes = message(sources.payload)
        .ok_or(failure(b"tasks.message_bytes", b"cannot derive from the agent's tool payload (P)"))?;
    limits.core.tasks.result_bytes = message(sources.payload)
        .ok_or(failure(b"tasks.result_bytes", b"cannot derive from the agent's tool payload (P)"))?;
    limits.core.tasks.inbox_bytes = inbox(sources.payload, sources.inbox)
        .ok_or(failure(b"tasks.inbox_bytes", b"cannot derive from the agent's inbox and tool payload (P)"))?;
    both(limits, sources)
}
/// Apply a worker's carriers to its base, before jig reads the overrides.
pub fn worker(limits: &mut Limits, sources: Sources) -> Result<(), jig_protocol_config::Problem> {
    both(limits, sources)
}

#[must_use]
pub fn forge_body(operation: u32) -> Option<u32> {
    skein_forge::marker::length(b"temper", operation)
}
/// Every listing must admit one largest row (skein-forge-connection's validity check).
#[must_use]
pub fn forge_answer(component: &skein_forge_connection::Limits, stock: u32) -> Option<u32> {
    use skein_forge_connection::limits::{Listing, largest_row};
    let mut largest = u64::from(stock);
    for listing in [
        Listing::Items,
        Listing::Reviews,
        Listing::Statuses,
        Listing::Remarks,
        Listing::Branches,
        Listing::Collaborators,
        Listing::PullFiles,
        Listing::Compare,
        Listing::Runs,
        Listing::Tree,
    ] {
        largest = largest.max(largest_row(listing, component)?);
    }
    u32::try_from(largest).ok()
}

pub fn worker_document(
    document: &skein_json::Document,
    at: u32,
    directories: &jig_protocol_config::Directories,
    sources: Sources,
) -> Result<jig_protocol_config::Worker, jig_protocol_config::Problems> {
    let mut base = stock();
    if let Err(found) = worker(&mut base, sources) {
        let mut problems = jig_protocol_config::Problems::empty();
        problems.push(found.path, found.what);
        return Err(problems);
    }
    if crate::startup::member(document, at, b"memory_bytes").is_some() {
        let mut problems = jig_protocol_config::Problems::empty();
        problems.push(jig_protocol_config::Path::root().member(b"memory_bytes"), jig_protocol_config::What::Fixed);
        return Err(problems);
    }
    let mut worker =
        jig_protocol_config::worker_over(document, at, directories, &crate::startup::fixed_worker(), &base)?;
    if worker.limits.is_none() {
        base.hub.slots = worker.slots;
        base.hub.source_interval = worker.check_interval;
        worker.limits = Some(base);
    }
    Ok(worker)
}

/// Whether a dotted limit was explicitly supplied, independent of its value.
#[must_use]
pub fn overridden(document: &skein_json::Document, at: u32, name: &[u8]) -> bool {
    use crate::startup::member;
    let Some(limits) = member(document, at, b"limits") else { return false };
    let Some(overrides) = member(document, limits, b"overrides") else { return false };
    member(document, overrides, name).is_some()
}
#[must_use]
pub fn slots(document: &skein_json::Document) -> Option<u32> {
    use crate::startup::member;
    use jig_protocol_config::read;
    let at = member(document, 0, b"workers")?;
    let workers = read::list(document, at, document.len()).ok()?;
    let at = member(document, *workers.get(0)?, b"slots")?;
    u32::try_from(read::integer(document, at, u64::from(u32::MAX)).ok()?).ok()
}
/// A worker's places follow its slots, from the document’s own slot count (derived.md, 5.2).
pub fn places(document: &skein_json::Document, application: u32, slots: u32, limits: &mut crate::limits::Shared) {
    if !overridden(document, application, b"checkout.workspaces") {
        limits.checkout.workspaces = slots;
    }
    if !overridden(document, application, b"git_component.operations") {
        limits.git_component.operations = limits.checkout.workspaces;
    }
}

#[must_use]
pub fn checkout_bounds(
    checkout: &crate::limits::Checkout,
    peer: &crate::limits::Peer,
) -> temper_protocol_worker_wire::Bounds {
    use temper_protocol_worker_wire::v1;
    temper_protocol_worker_wire::Bounds {
        repositories: checkout.repositories,
        name_bytes: checkout.name_bytes,
        remote_bytes: checkout.name_bytes,
        conflicts: checkout.conflicts,
        path_bytes: checkout.path_bytes,
        token_bytes: v1::CEILINGS.token_token,
        entries: peer.installations,
        entry_token_bytes: peer.token_bytes,
        text_bytes: v1::CEILINGS.identity_name,
        reds: v1::CEILINGS.reds_reds,
        red_key_bytes: v1::CEILINGS.red_key,
    }
}
/// The worker link's projection of its owners, shared with carrier sizing.
#[must_use]
pub fn worker_sizes(
    jig: &Limits,
    slots: u32,
    payload: &temper_protocol_worker_wire::v1::Limits,
    forges: u32,
) -> Option<jig_protocol_worker_wire::sizes::Sizes> {
    let core = jig.core;
    let hub = jig.hub;
    let profile = jig.link;
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    let largest = temper_protocol_worker_wire::largest(payload)?;
    Some(jig_protocol_worker_wire::sizes::Sizes {
        application_ask: largest.ask,
        application_answer: largest.answer,
        program_sources: core.fleet.program_sources,
        piece: hub.piece_bytes,
        charter: u32::try_from(hub.charter_bytes).ok()?,
        transcript: u32::try_from(hub.transcript_bytes).ok()?,
        turn: u32::try_from(hub.turn_bytes).ok()?,
        message: core.tasks.message_bytes,
        sender: ceilings.message_sender,
        call_name: u32::try_from(core.fleet.call_name_bytes).ok()?,
        tool_name: ceilings.call_tool,
        call_input: core.call_answer_bytes,
        answer: core.call_answer_bytes,
        result: u32::try_from(hub.outcome_bytes).ok()?,
        detail: hub.detail_bytes,
        grant: crate::values::grant_bytes(crate::values::token_credential_bytes(jig.token), largest.grant)?,
        name: profile.name,
        secret: profile.secret,

        items: largest.items,
        left: largest.left,
        saved: largest.saved,
        evidence: u32::try_from(hub.delivery_evidence_bytes).ok()?,
        resource: largest.resource,
        slots,
        calls: hub.run_calls,
        turns: hub.turns,
        inbox: core.tasks.run_inbox,
        answered: core.call_records,
        models: ceilings.bounds_endpoints,
        workstreams: core.fleet.workstreams,
        accounts: core.accounts.accounts,
        credentials: forges,
        facts: profile.facts,
    })
}

/// The generated payload bounds need only the checkout's three capacity values.
#[must_use]
pub fn checkout_payload(checkout: &crate::limits::Checkout) -> temper_protocol_worker_wire::v1::Limits {
    temper_protocol_worker_wire::v1::Limits {
        saved_repositories: checkout.repositories,
        left_landed: checkout.repositories,
        files_files: checkout.conflicts,
        files_files_item: checkout.path_bytes,
        ..temper_protocol_worker_wire::v1::CEILINGS
    }
}

/// Measure the same answer as jig's sizes, with its frame header. Other body
/// fields remain at their ceilings: they do not occur in an Answer.
#[must_use]
pub fn checkout_answer(jig: &Limits, payload: &temper_protocol_worker_wire::v1::Limits) -> Option<u64> {
    use jig_protocol_worker_wire::{CEILINGS, v17};
    let saved = temper_protocol_worker_wire::v1::Saved::encoded_bound(payload)?;
    let ceiling = u64::from(CEILINGS.work_saved_some);
    let limits = v17::Limits {
        work_left_some: u32::try_from(temper_protocol_worker_wire::v1::Left::encoded_bound(payload)?).ok()?,
        work_saved_some: u32::try_from(saved.min(ceiling)).ok()?,
        answer_refused: jig.core.tasks.run_inbox,
        ended_outcome: u32::try_from(jig.hub.outcome_bytes).ok()?,
        failed_detail: jig.hub.detail_bytes,
        invalid_start_why: jig.hub.detail_bytes,
        resource_resource_some: u32::try_from(temper_protocol_worker_wire::v1::Preparation::encoded_bound(payload)?)
            .ok()?,
        ..CEILINGS
    };
    // At the saved ceiling, the largest ending contains Work, not a refusal.
    // Any excess extends that same byte field; measure even a ceiling shortfall
    // so it cannot hide the rooms that must rise with it.
    v17::Answer::encoded_bound(&limits)?.checked_add(saved.saturating_sub(ceiling))?.checked_add(8)
}
