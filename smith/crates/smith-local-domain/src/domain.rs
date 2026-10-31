//! Local chat transitions and agent composition (domain/host.md, sections 2,
//! 6 and 8–9). The agent's IO requests pass through unchanged. The host saves
//! names before use and holds an answer until every turn has a store terminal.

use crate::agent::{Agent, Output, RunStart, command};
use crate::{AccountFailure, AgentFailure, Checks, Failure, Notice, Undelivered};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
use smith_domain as agent;
use smith_host_domain::{self as host, parent};

use crate::boundary::{
    ChatState, DeliveryRecord, DeliveryState, Event, ExitStatus, GitOp, GitResult, Request, StoreFailure,
};
use crate::chat::{Chat, Gone, Phase};
use crate::credentials::Grants;
#[path = "delivery/machine.rs"]
mod delivery;
#[path = "reconcile.rs"]
mod reconcile;
use crate::person::Lines;
use crate::turns::Turns;
use crate::{AgentLimits, Config, Fact, Invalid, Limits, charter};
use delivery::{InPlace, begin_delivery, deliver_to_agent, delivery_saved, plain_status};
use reconcile::{Reconcile, reconcile_next};

/// One local chat and its inline or spawned agent behind the parent face.
#[derive(Debug)]
pub struct Domain {
    pub(crate) config: Config,
    pub(crate) main_model: usize,
    pub(crate) effort: Option<agent::run::charter::Effort>,
    chat_seed: u64,
    pub(crate) choice: Option<crate::Slash>,
    pub(crate) interactive: bool,
    pub(crate) closing_last: Option<u64>,
    agent: Agent,
    pub(crate) chat: Chat,
    transcript: Option<agent::Transcript>,
    records: Map<agent::run::CallName, DeliveryRecord>,
    delivery: Option<InPlace>,
    saving: Option<DeliveryRecord>,
    delivery_owner: Option<Token>,
    reconciling: Option<Reconcile>,
    heads: Map<u32, Box<[u8]>>,
    own_heads: Map<u32, Box<[u8]>>,
    head_survey: HeadSurvey,
    pub(crate) lines: Lines,
    run_messages: Queue<host::Message>,
    grants: Grants,
    grant_failed: Option<(u32, crate::CredentialFailure)>,
    pub(crate) run: Option<Token>,
    pub(crate) turns: Turns,
    held: Queue<Held>,
    pub(crate) stop_failed: bool,
    pub(crate) end_cause: Option<crate::EndCause>,
    told: crate::end::Told,
    answer_wall: Option<Wall>,
    pub(crate) cancel_requested: Option<Time>,
    agent_failed: Option<AgentFailure>,
    pub(crate) agent_life: AgentLife,
    pub(crate) show_bytes: u32,
    long: crate::long::Long,
    pub(crate) wall_deadline: Option<Time>,
    facts: Queue<Fact>,
    facts_lost: u64,
}

/// One completion terminal held under the local store's acknowledgement pressure.
#[derive(Debug)]
pub(crate) struct Held {
    agent: Token,
    terminal: smith_inline_agent::Below,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AgentLife {
    Starting,
    Gone,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HeadSurvey {
    NotStarted,
    Reading(u32),
    Complete,
}

impl Domain {
    /// Enable the product's initial waiting prompt before receiving input.
    pub fn line_mode(&mut self) {
        self.interactive = true;
    }

    pub(crate) fn new_chat_name(&self) -> Box<[u8]> {
        skein_lib::Decimal::of(self.chat_seed).as_bytes().into()
    }

    fn reading(&self) -> crate::Reading {
        if self.choice.is_some() { crate::Reading::Held } else { self.lines.reading() }
    }

    /// Reserve the initial chat selection before this invocation sends its first Load.
    pub fn initial_chat(&mut self) -> Box<[u8]> {
        assert!(self.chat.phase == Phase::Loading && !self.chat.load_requested, "startup precedes Load");
        self.chat.load_requested = true;
        self.config.chat.clone()
    }
    /// The existing terminal's verdict for one told tool result.
    #[must_use]
    pub fn tool_verdict(result: &smith_domain::session::llm::Returned) -> &'static [u8] {
        crate::person::tool_end(result)
    }

    /// The existing local result renderer for a typed accepted answer.
    #[must_use]
    pub fn result_text(answer: &smith_host_domain::Answer) -> Box<[u8]> {
        crate::agent::answer_text(answer)
    }

    /// Validate local choices and build the agent with the configured endpoints.
    pub fn new(mut config: Config, limits: &Limits, seed: u64) -> Result<Domain, Invalid> {
        config.validate(limits)?;
        let mut models = List::with_capacity(u32::try_from(config.models.len()).expect("validated models count"));
        for (index, model) in config.models.iter().enumerate() {
            let progress = config.progress.get(index).expect("validated per-model progress inventory");
            models
                .push(agent::ConfiguredModel {
                    endpoint: model.endpoint,
                    model: model.model.clone(),
                    efforts: config.efforts.get(index).expect("validated per-model efforts").clone(),
                    prices: model.prices,
                    window: model.window,
                    output: model.output,
                    protocol_allowance: *config
                        .protocol_allowances
                        .get(index)
                        .expect("validated per-model protocol inventory"),
                    head: progress.head,
                    idle: progress.idle,
                })
                .expect("one cell per configured model");
        }
        let agent_config = agent::Config {
            programs: Box::new([]),
            endpoints: Box::from(limits.endpoints.as_ref()),
            models: models.into_boxed(),
        };
        let child_limits = match limits.agent {
            AgentLimits::Inline(mut child) => {
                if child.smith != limits.smith || child.slots != 1 {
                    return Err(Invalid::Limits);
                }
                child.window.turns = child.window.turns.min(limits.unsaved);
                AgentLimits::Inline(child)
            }
            AgentLimits::Spawned(mut child) => {
                child.turns = child.turns.min(limits.unsaved);
                AgentLimits::Spawned(child)
            }
        };
        let mut lines = Lines::new(limits.smith.run.messages);
        match &mut config.mode {
            crate::Mode::Headless { prompt } => lines.receive(mem::replace(prompt, Box::new([]))).expect("one prompt"),
            crate::Mode::Chat { .. } => {}
        }
        let effort = config.models.first().expect("validated main model").effort.clone();
        Ok(Domain {
            main_model: 0,
            effort,
            config,
            chat_seed: seed,
            choice: None,
            interactive: false,
            closing_last: None,
            agent: Agent::new(&child_limits, agent_config, seed)?,
            chat: Chat::new(seed),
            transcript: None,
            records: Map::with_capacity(limits.smith.run.answered_calls),
            delivery: None,
            saving: None,
            delivery_owner: None,
            reconciling: None,
            heads: Map::with_capacity(limits.smith.run.directories),
            own_heads: Map::with_capacity(limits.smith.run.directories),
            head_survey: HeadSurvey::NotStarted,
            lines,
            run_messages: Queue::with_capacity(limits.smith.run.messages.saturating_add(1)),
            grants: Grants::new(limits.smith.accounts),
            grant_failed: None,
            run: None,
            turns: Turns::new(limits.unsaved),
            held: Queue::with_capacity(1),
            stop_failed: false,
            end_cause: None,
            told: crate::end::Told::new(),
            answer_wall: None,
            cancel_requested: None,
            agent_failed: None,
            agent_life: AgentLife::Starting,
            show_bytes: limits.show_bytes,
            long: crate::long::Long::new(),
            wall_deadline: None,
            facts: Queue::with_capacity(limits.facts),
            facts_lost: 0,
        })
    }

    /// First recorded cause of an unanswering end, retained through lower settlement.
    #[must_use]
    pub const fn end_cause(&self) -> Option<crate::EndCause> {
        self.end_cause
    }

    /// The host wall clock when its last answer arrived, before settlement.
    #[must_use]
    pub fn answer_wall(&self) -> Option<Wall> {
        self.answer_wall
    }

    /// Bounded chat name for the shell's final operator line.
    #[must_use]
    pub fn chat_name(&self) -> &[u8] {
        &self.config.chat
    }

    /// Render the sent charter's refused effort using startup's model declaration.
    #[must_use]
    pub fn effort_words(&self) -> Box<[u8]> {
        smith_events::effort_words(
            self.effort.as_deref().unwrap_or_default(),
            self.config.efforts.get(self.main_model).expect("selected startup efforts"),
            self.show_bytes,
            false,
        )
    }

    /// Whether initial loading or the child has deferred work.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        (self.chat.phase == Phase::Loading && !self.chat.load_requested) || self.agent.ready()
    }

    /// Earliest child deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.pressured() { self.wall_deadline } else { self.agent.deadline() }
    }

    /// Whether the child has a deadline due at `now`.
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        }
    }

    /// Reclaim child slots after delivering one iteration's output.
    pub fn reclaim(&mut self) {
        self.agent.reclaim();
    }

    /// Oldest content-free observation, if the caller wants it.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// Observations lost because the caller did not drain them.
    #[must_use]
    pub fn facts_lost(&self) -> u64 {
        self.facts_lost
    }

    /// Entries this domain and its agent child hold for the service to
    /// drain (protocol/limits.md 3.8): the child's `backlog()` and the
    /// child's output queue. The service counts them in its backlog.
    #[must_use]
    pub fn backlog(&self) -> u32 {
        self.agent.backlog()
    }

    /// One fact of the inline run, oldest first; none for a spawned agent.
    pub fn pop_run_fact(&mut self) -> Option<agent::Fact> {
        match &mut self.agent {
            Agent::Inline { domain, .. } => domain.pop_fact(Token::new(1)),
            Agent::Spawned { .. } => None,
        }
    }

    /// The inline parent message, before its run receipt reaches the service.
    pub fn pop_run_message(&mut self) -> Option<host::Message> {
        self.run_messages.pop()
    }

    /// Current run policy for its service's event producer.
    #[must_use]
    pub fn charter(&self) -> (agent::run::Charter, u64) {
        let mut charter = charter(&self.config);
        charter.llm = self.config.models.get(self.main_model).expect("selected startup model").clone();
        charter.llm.effort.clone_from(&self.effort);
        (charter, self.lines.named_count())
    }

    /// Configured model names used only to render an operator notice.
    #[must_use]
    pub fn models(&self) -> &[agent::run::charter::Llm] {
        &self.config.models
    }

    /// Whether one answer ends this invocation.
    #[must_use]
    pub fn headless(&self) -> bool {
        match self.config.mode {
            crate::Mode::Headless { .. } => true,
            crate::Mode::Chat { .. } => false,
        }
    }

    fn pressured(&self) -> bool {
        self.turns.unsaved.room() == 0 && !self.held.is_empty()
    }
}

/// Handle one typed event and its immediate child handoffs.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let reading = domain.reading();
    drain_agent_content(domain);
    match event {
        Event::Line { text } => receive_line(domain, env, text, out),
        Event::Slash(choice) => crate::choices::receive(domain, choice, env, out),
        Event::Interrupt => interrupt(domain, env, out),
        Event::Drop => drop_run(domain, env),
        Event::Closed => closed(domain, out),
        Event::SelectChat { name, create } => select_chat(domain, env, name, create, out),
        Event::ChatSelected { name, result } => chat_selected(domain, name, result, out),
        Event::Loaded { state, transcript, deliveries } => loaded(domain, env, state, transcript, deliveries, out),
        Event::StateSaved => state_saved(domain, env, out),
        Event::TurnSaved { number } => turn_saved(domain, env, number, out),
        Event::DeliverySaved { name } => delivery_saved(domain, env, name, out),
        Event::Git { owner, result } => git_result(domain, env, owner, result, out),
        Event::PlainStatus { owner, changed } => plain_status(domain, env, owner, changed, out),
        Event::StoreFailed { turn, reason } => store_failed(domain, env, turn, reason, out),
        Event::Credential { grant, form } => credential(domain, env, grant, form, out),
        Event::NoCredential { account, reason } => no_credential(domain, env, account, reason, out),
        Event::Below { agent, terminal } => agent_terminal(domain, env, agent, terminal),
        Event::Process(event) => domain.agent.process(env.now, env.wall, event),
    }
    route_agent(domain, env, out);
    release_held(domain, env, out);
    if reading != domain.reading() {
        out.push(Request::Input { reading: domain.reading() });
    }
}

/// Release duplicate capture content, inline lifecycle observations and unused host facts.
/// The service alone takes native run facts into its event producer.
fn drain_agent_content(domain: &mut Domain) {
    match &mut domain.agent {
        Agent::Inline { domain: child, .. } => {
            let client = Token::new(1);
            // The extra empty pop marks content drained so an ended slot can be reclaimed.
            for _ in 0..=child.pending_content(client) {
                if child.pop_content(client).is_none() {
                    break;
                }
            }
            for _ in 0..child.pending_slot_facts() {
                if child.pop_slot_fact().is_none() {
                    break;
                }
            }
        }
        Agent::Spawned { domain: child, .. } => {
            for _ in 0..child.backlog() {
                if child.pop_fact().is_none() {
                    break;
                }
            }
        }
    }
}

fn agent_terminal(domain: &mut Domain, env: &Env<Limits>, agent: Token, terminal: smith_inline_agent::Below) {
    let completion = match &terminal {
        smith_inline_agent::Below::Completed { .. }
        | smith_inline_agent::Below::Failed { .. }
        | smith_inline_agent::Below::Cancelled { .. } => true,
        smith_inline_agent::Below::Done { .. }
        | smith_inline_agent::Below::Read { .. }
        | smith_inline_agent::Below::Probed { .. }
        | smith_inline_agent::Below::Checked { .. }
        | smith_inline_agent::Below::Aborted { .. } => false,
    };
    if completion && domain.turns.unsaved.room() == 0 {
        domain.held.push(Held { agent, terminal });
    } else {
        drain_agent_content(domain);
        domain.agent.below(env.now, env.wall, agent, terminal);
    }
}

fn release_held(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.turns.unsaved.room() == 0 {
        return;
    }
    if let Some(held) = domain.held.pop() {
        drain_agent_content(domain);
        domain.agent.below(env.now, env.wall, held.agent, held.terminal);
        route_agent(domain, env, out);
    }
}

/// Fire one child deadline and route its output.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    drain_agent_content(domain);
    if domain.pressured() {
        if let Some(deadline) = domain.wall_deadline
            && deadline <= env.now
            && domain.chat.phase == Phase::Running
        {
            interrupt(domain, env, out);
            domain.wall_deadline = None;
        }
    } else {
        domain.agent.fire(env.now, env.wall);
    }
    route_agent(domain, env, out);
}

/// Ask for the initial transcript or advance one ready child handoff.
pub fn resume(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    drain_agent_content(domain);
    if domain.chat.phase == Phase::Loading && !domain.chat.load_requested {
        domain.chat.load_requested = true;
        out.push(Request::Load);
    } else if domain.agent.ready() {
        domain.agent.resume(env.now, env.wall);
        route_agent(domain, env, out);
    }
}

fn receive_line(domain: &mut Domain, env: &Env<Limits>, text: Box<[u8]>, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => return,
        crate::Mode::Chat { .. } => {}
    }
    if text.len() > usize::try_from(env.limits.line_bytes).expect("u32 fits usize") {
        out.push(Request::Notice { notice: Notice::NotDelivered { line: None, why: Undelivered::TooLong } });
        return;
    }
    domain.lines.receive(text).expect("input is held at the inbox plus the completed line");
    dispatch_line(domain, env, out);
}

fn dispatch_line(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.chat.state_pending || domain.lines.is_empty() {
        return;
    }
    match domain.chat.phase {
        Phase::Idle | Phase::Running => {}
        Phase::Loading | Phase::Starting | Phase::Ending | Phase::Answered { .. } | Phase::Done => return,
    }
    if domain.lines.received() {
        let Some(next_message) = domain.chat.state.next_message.checked_add(1) else {
            stop_names(domain, out);
            return;
        };
        domain.chat.state.next_message = next_message;
        domain.lines.name(Token::new(next_message));
    } else if domain.chat.phase == Phase::Running {
        return;
    }
    if domain.chat.phase == Phase::Idle {
        let Some(activation) = domain.chat.state.activation.checked_add(1) else {
            stop_names(domain, out);
            return;
        };
        domain.chat.state.activation = activation;
        domain.chat.phase = Phase::Starting;
        domain.head_survey = HeadSurvey::NotStarted;
        domain.heads = Map::with_capacity(env.limits.smith.run.directories);
        domain.own_heads = Map::with_capacity(env.limits.smith.run.directories);
        domain.chat.start_saved = false;
        domain.grant_failed = None;
        domain.grants.values = List::with_capacity(env.limits.smith.accounts);
        domain.grants.awaiting = u32::try_from(domain.config.accounts.len()).expect("validated account count");
        for account in &domain.config.accounts {
            out.push(Request::Credential { account: *account });
        }
    }
    domain.chat.state_pending = true;
    let fresh = domain.chat.phase == Phase::Starting && !domain.config.resume;
    out.push(Request::SaveState { state: domain.chat.state, fresh });
}

fn stop_names(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.chat.phase = Phase::Done;
    out.push(Request::Notice { notice: Notice::Failure(Failure::NamesExhausted) });
    out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
}

pub(crate) fn select_chat(
    domain: &mut Domain,
    env: &Env<Limits>,
    name: Box<[u8]>,
    create: bool,
    out: &mut Queue<Request>,
) {
    let refusal = if name.len() > usize::try_from(env.limits.chat_bytes).expect("bounded chat name") {
        Some(crate::ChatRefusal::Name)
    } else if domain.chat.phase != Phase::Idle
        || domain.run.is_some()
        || domain.chat.state_pending
        || !domain.turns.unsaved.is_empty()
        || !domain.lines.is_empty()
        || domain.saving.is_some()
    {
        Some(crate::ChatRefusal::Busy)
    } else {
        None
    };
    if let Some(reason) = refusal {
        out.push(Request::Notice { notice: Notice::ChatRefused { name, reason } });
        return;
    }
    if domain.chat_seed == u64::MAX {
        stop_names(domain, out);
        return;
    }
    domain.chat.phase = Phase::Loading;
    domain.chat.load_requested = true;
    out.push(Request::SelectChat { name, create });
}

fn chat_selected(
    domain: &mut Domain,
    name: Box<[u8]>,
    result: Result<(), crate::ChatRefusal>,
    out: &mut Queue<Request>,
) {
    if domain.stop_failed {
        return;
    }
    assert!(domain.run.is_none() && !domain.chat.state_pending, "selection ends between activations");
    assert!(domain.chat.phase == Phase::Loading && domain.chat.load_requested, "selection has one terminal");
    match result {
        Ok(()) => {
            let closed = domain.chat.closed;
            domain.chat_seed = match domain.chat_seed.checked_add(1) {
                Some(seed) => seed,
                None => {
                    stop_names(domain, out);
                    return;
                }
            };
            domain.chat = Chat::new(domain.chat_seed);
            domain.chat.closed = closed;
            domain.chat.load_requested = true;
            domain.config.chat = name;
            out.push(Request::Load);
        }
        Err(reason) => {
            domain.chat.phase = Phase::Idle;
            domain.chat.load_requested = false;
            out.push(Request::Notice { notice: Notice::ChatRefused { name, reason } });
            if domain.chat.closed {
                closed(domain, out);
            }
        }
    }
}

fn loaded(
    domain: &mut Domain,
    env: &Env<Limits>,
    state: Option<ChatState>,
    transcript: Option<agent::Transcript>,
    deliveries: Box<[DeliveryRecord]>,
    out: &mut Queue<Request>,
) {
    assert!(
        domain.chat.phase == Phase::Loading && domain.chat.load_requested,
        "Load has one terminal: phase {:?}, requested {}",
        domain.chat.phase,
        domain.chat.load_requested
    );
    match state {
        Some(mut saved) => {
            if !domain.config.resume {
                saved.identity = domain.chat.state.identity;
            }
            domain.chat.state = saved;
        }
        None => {
            domain.chat.state =
                ChatState { identity: domain.chat.state.identity, activation: 0, next_message: 0, read: None };
        }
    }
    domain.transcript = transcript;
    domain.records = Map::with_capacity(env.limits.smith.run.answered_calls);
    for record in deliveries {
        match domain.records.insert(record.name, record) {
            Ok(None) => {}
            Ok(Some(_)) | Err(_) => {
                store_failed(domain, env, None, StoreFailure::Read, out);
                return;
            }
        }
    }
    resume_loading(domain, env, out);
}

fn resume_loading(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    for (_, record) in &domain.records {
        match &record.state {
            DeliveryState::Intent(_) => {
                domain.reconciling = Some(Reconcile::new(record.clone()));
                reconcile_next(domain, env, out);
                return;
            }
            DeliveryState::Answer(_) => {}
        }
    }
    domain.chat.phase = Phase::Idle;
    domain.chat.load_requested = false;
    crate::choices::apply(domain, env, out);
    if domain.chat.phase != Phase::Idle {
        return;
    }
    if domain.chat.closed && domain.lines.is_empty() {
        domain.chat.phase = Phase::Done;
        let status = match domain.config.mode {
            crate::Mode::Headless { .. } => crate::end::exit_status(domain.end_cause),
            crate::Mode::Chat { .. } => ExitStatus::Accepted,
        };
        out.push(Request::Exit { status });
    } else {
        dispatch_line(domain, env, out);
        if domain.interactive && domain.chat.phase == Phase::Idle && domain.lines.is_empty() {
            out.push(Request::Notice { notice: Notice::Waiting });
        }
    }
}

fn state_saved(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    assert!(domain.chat.state_pending, "StateSaved answers one SaveState");
    domain.chat.state_pending = false;
    domain.lines.saved();
    match domain.chat.phase {
        Phase::Starting => {
            domain.chat.start_saved = true;
            maybe_start(domain, env, out);
        }
        Phase::Running => {
            send_line(domain, env, out);
        }
        Phase::Answered { .. } => crate::person::finish_answer(domain, out),
        Phase::Ending | Phase::Loading | Phase::Idle | Phase::Done => {}
    }
}

fn credential(domain: &mut Domain, env: &Env<Limits>, grant: agent::Grant, form: host::Form, out: &mut Queue<Request>) {
    let grant =
        host::Grant { account: grant.name.account, generation: grant.name.generation, valid: grant.valid, form };
    match domain.chat.phase {
        Phase::Starting => {
            assert!(domain.config.accounts.contains(&grant.account), "grant names a requested account");
            assert!(domain.grants.awaiting > 0, "one terminal per account request");
            domain.grants.awaiting = domain.grants.awaiting.checked_sub(1).expect("pending account");
            domain.grants.values.push(grant).expect("validated account capacity");
            maybe_start(domain, env, out);
        }
        Phase::Running | Phase::Ending => command(
            &mut domain.agent,
            env.now,
            env.wall,
            parent::Event::Grant { agent: domain.run.expect("grant belongs to the admitted child"), grant },
        ),
        Phase::Answered { .. } => {}
        Phase::Loading | Phase::Idle | Phase::Done => unreachable!("grant answers a live account request"),
    }
}

fn no_credential(
    domain: &mut Domain,
    env: &Env<Limits>,
    account: u32,
    reason: crate::CredentialFailure,
    out: &mut Queue<Request>,
) {
    match domain.chat.phase {
        Phase::Starting => {
            assert!(domain.grants.awaiting > 0, "one terminal per account request");
            domain.grants.awaiting = domain.grants.awaiting.checked_sub(1).expect("pending account");
            domain.grant_failed = Some((account, reason));
            maybe_start(domain, env, out);
        }
        Phase::Running | Phase::Ending => {
            out.push(Request::Notice { notice: Notice::Failure(Failure::Credential { account, reason }) });
        }
        Phase::Answered { .. } => {}
        Phase::Loading | Phase::Idle | Phase::Done => unreachable!("missing grant answers a live account request"),
    }
}

fn maybe_start(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.chat.phase != Phase::Starting || !domain.chat.start_saved || domain.grants.awaiting > 0 {
        return;
    }
    if domain.cancel_requested.is_some() {
        crate::person::show_dropped(&mut domain.lines, out);
        domain.chat.phase = Phase::Done;
        out.push(Request::Exit { status: ExitStatus::Interrupted });
        return;
    }
    if domain.chat.closed {
        match domain.config.mode {
            crate::Mode::Headless { .. } => {
                domain.chat.phase = Phase::Done;
                out.push(Request::Exit { status: ExitStatus::Interrupted });
                return;
            }
            crate::Mode::Chat { .. } => {}
        }
    }
    if let Some((account, reason)) = domain.grant_failed.take() {
        domain.chat.phase = Phase::Idle;
        crate::person::show_dropped(&mut domain.lines, out);
        out.push(Request::Notice { notice: Notice::Failure(Failure::Credential { account, reason }) });
        match domain.config.mode {
            crate::Mode::Headless { .. } => {
                domain.chat.phase = Phase::Done;
                out.push(Request::Exit { status: ExitStatus::Credentials { account } });
            }
            crate::Mode::Chat { .. } => {}
        }
        finish_failed_start(domain, out);
        return;
    }
    match domain.head_survey {
        HeadSurvey::NotStarted => {
            domain.head_survey = HeadSurvey::Reading(0);
            survey_heads(domain, env, out);
            return;
        }
        HeadSurvey::Reading(_) => return,
        HeadSurvey::Complete => {}
    }
    let grants = mem::replace(&mut domain.grants.values, List::with_capacity(env.limits.smith.accounts)).into_boxed();
    let history = if domain.config.resume { domain.transcript.take() } else { None };
    let mut answered = List::with_capacity(env.limits.smith.run.answered_calls);
    if domain.config.resume {
        for (name, record) in &domain.records {
            match &record.state {
                DeliveryState::Answer(delivery) => answered
                    .push(agent::AnsweredCall {
                        name: *name,
                        tool: Box::from(&b"deliver"[..]),
                        answer: agent::Answered::Delivery(Box::new(delivery.clone())),
                    })
                    .expect("one turn's calls fit the bound"),
                DeliveryState::Intent(_) => unreachable!("intents reconciled before starting"),
            }
        }
    } else {
        domain.records = Map::with_capacity(env.limits.smith.run.answered_calls);
    }
    domain.wall_deadline = Some(env.now.saturating_add(domain.config.budget.time));
    domain.agent_life = AgentLife::Starting;
    domain.agent_failed = None;
    domain.end_cause = None;
    domain.told = crate::end::Told::new();
    domain.cancel_requested = None;
    let messages = domain.lines.carry();
    let mut initial = List::with_capacity(env.limits.smith.run.messages.saturating_add(1));
    for message in &messages {
        initial.push(message.name).expect("bounded carried lines");
        observe_initial_message(domain, message);
    }
    let source = RunStart {
        live_text: domain.config.live_text,
        messages,
        identity: domain.chat.state.identity,
        activation: domain.chat.state.activation,
        charter: domain.charter().0,
        workspace: domain.config.workspace.clone(),
        transcript: history,
        answered: answered.into_boxed(),
        grants,
    };
    match domain.agent.prepare(source) {
        Ok(start) => {
            command(&mut domain.agent, env.now, env.wall, parent::Event::Spawn { client: Token::new(1), start });
        }
        Err(error) => refused_preparation(domain, error, out),
    }
    domain.chat.phase = Phase::Running;
    observe(domain, Fact::Started { activation: domain.chat.state.activation });
    for name in &initial {
        observe(domain, Fact::Message { name: *name });
    }
}

fn finish_failed_start(domain: &mut Domain, out: &mut Queue<Request>) {
    if domain.chat.closed {
        closed(domain, out);
    }
}

fn refused_preparation(domain: &mut Domain, error: crate::agent::BridgeError, out: &mut Queue<Request>) {
    domain.agent_failed = Some(AgentFailure::NotStarted);
    let invalid = crate::agent::preparation_refusal(error);
    domain.agent_life = AgentLife::Gone;
    crate::person::show_dropped(&mut domain.lines, out);
    out.push(Request::Notice { notice: Notice::Failure(Failure::Unstarted(host::End::Invalid(invalid))) });
    out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
}

fn observe_initial_message(domain: &mut Domain, message: &host::Message) {
    match &domain.agent {
        Agent::Inline { .. } => domain.run_messages.push(message.clone()),
        Agent::Spawned { .. } => {}
    }
}

fn survey_heads(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let next = match domain.head_survey {
        HeadSurvey::Reading(next) => next,
        HeadSurvey::NotStarted | HeadSurvey::Complete => unreachable!("head scan is active"),
    };
    if let Some(workspace) = &domain.config.workspace {
        for (index, directory) in
            workspace.directories.iter().enumerate().skip(usize::try_from(next).expect("bounded position"))
        {
            if directory.writable && directory.git {
                let position = u32::try_from(index).expect("admitted directory count");
                domain.head_survey = HeadSurvey::Reading(position);
                out.push(Request::Git {
                    owner: Token::new(0),
                    directory: position,
                    op: GitOp::Head,
                    deadline: env.now.saturating_add(domain.config.budget.time),
                });
                return;
            }
        }
    }
    domain.head_survey = HeadSurvey::Complete;
    maybe_start(domain, env, out);
}

fn head_result(domain: &mut Domain, env: &Env<Limits>, owner: Token, result: GitResult, out: &mut Queue<Request>) {
    assert_eq!(owner, Token::new(0), "start head inspection uses its own owner");
    let directory = match domain.head_survey {
        HeadSurvey::Reading(directory) => directory,
        HeadSurvey::NotStarted | HeadSurvey::Complete => unreachable!("head inspection is active"),
    };
    match result {
        GitResult::Head { head } if !head.is_empty() && head.len() <= agent::run::Receipt::CAPACITY => {
            domain.heads.insert(directory, head).expect("one head per writable repository");
            domain.head_survey = HeadSurvey::Reading(directory.checked_add(1).expect("admitted position"));
            survey_heads(domain, env, out);
        }
        GitResult::Head { .. } | GitResult::Failed { .. } => {
            domain.chat.phase = Phase::Idle;
            crate::person::show_dropped(&mut domain.lines, out);
            let directory = domain
                .config
                .workspace
                .as_ref()
                .expect("head survey has a workspace")
                .directories
                .get(usize::try_from(directory).expect("directory position fits usize"))
                .expect("head survey names a declared directory")
                .name
                .clone();
            out.push(Request::Notice { notice: Notice::Failure(Failure::HeadUnavailable { directory }) });
            match domain.config.mode {
                crate::Mode::Headless { .. } => {
                    domain.chat.phase = Phase::Done;
                    out.push(Request::Exit { status: ExitStatus::Refused });
                }
                crate::Mode::Chat { .. } => finish_failed_start(domain, out),
            }
        }
        GitResult::Status { .. }
        | GitResult::Inspected { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::NoEffect { .. }
        | GitResult::Uncertain { .. } => unreachable!("start head inspection has one terminal"),
    }
}

fn send_line(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.chat.phase != Phase::Running {
        return;
    }
    let Some(run) = domain.run else { return };
    for message in domain.lines.relay(false) {
        let name = message.name;
        observe_initial_message(domain, &message);
        command(
            &mut domain.agent,
            env.now,
            env.wall,
            parent::Event::Message {
                finish_requested: message.finish_requested,
                agent: run,
                name,
                label: message.label,
                text: message.text,
            },
        );
        observe(domain, Fact::Message { name });
    }
    dispatch_line(domain, env, out);
}

fn turn_saved(domain: &mut Domain, env: &Env<Limits>, number: u32, out: &mut Queue<Request>) {
    if domain.stop_failed {
        return;
    }
    let (pending, sequence) = domain.turns.unsaved.pop().expect("TurnSaved answers a pending SaveTurn");
    assert_eq!(number, pending, "turns become durable in order");
    if let Some(run) = domain.run {
        command(&mut domain.agent, env.now, env.wall, parent::Event::Acknowledge { agent: run, turn: number });
    }
    let mut retired = List::with_capacity(env.limits.smith.run.answered_calls);
    for (name, record) in &domain.records {
        let answered = match record.state {
            DeliveryState::Intent(_) => false,
            DeliveryState::Answer(_) => true,
        };
        if answered && (name.activation != domain.chat.state.activation || name.completion <= sequence) {
            retired.push(*name).expect("bounded saved answers");
        }
    }
    for name in &retired {
        domain.records.remove(name);
    }
    if domain.turns.unsaved.is_empty() {
        if domain.agent_failed.is_some() && domain.agent_life == AgentLife::Gone {
            domain.chat.phase = Phase::Done;
            out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
        } else {
            crate::person::finish_answer(domain, out);
        }
    }
}

fn interrupt(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => record_cause(domain, crate::EndCause::Cancelled),
        crate::Mode::Chat { .. } => {}
    }
    match domain.chat.phase {
        Phase::Idle => {
            domain.chat.phase = Phase::Done;
            out.push(Request::Exit { status: ExitStatus::Accepted });
        }
        Phase::Running => {
            domain.cancel_requested = Some(env.now);
            domain.chat.phase = Phase::Ending;
            domain.wall_deadline = None;
            if let Some(run) = domain.run {
                command(&mut domain.agent, env.now, env.wall, parent::Event::Stop { agent: run });
            }
        }
        Phase::Starting => {
            domain.chat.closed = true;
            domain.cancel_requested = Some(env.now);
        }
        Phase::Loading | Phase::Ending | Phase::Answered { .. } | Phase::Done => domain.chat.closed = true,
    }
}

pub(crate) fn closed(domain: &mut Domain, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => return,
        crate::Mode::Chat { .. } => {}
    }
    if domain.interactive && !domain.chat.closed {
        let extra = if domain.run.is_some() || domain.chat.phase == Phase::Starting { 1 } else { 2 };
        domain.closing_last = Some(domain.chat.state.activation.saturating_add(extra));
    }
    domain.chat.closed = true;
    if domain.chat.phase == Phase::Idle && domain.lines.is_empty() {
        domain.chat.phase = Phase::Done;
        out.push(Request::Exit { status: ExitStatus::Accepted });
    }
}

fn record_cause(domain: &mut Domain, cause: crate::EndCause) {
    if domain.end_cause.is_none() {
        domain.end_cause = Some(cause);
    }
}

fn drop_run(domain: &mut Domain, env: &Env<Limits>) {
    if domain.turns.answer.is_some() {
        return;
    }
    if let Some(run) = domain.run {
        domain.cancel_requested = Some(env.now);
        record_cause(domain, crate::EndCause::Cancelled);
        domain.chat.phase = Phase::Ending;
        domain.wall_deadline = None;
        command(&mut domain.agent, env.now, env.wall, parent::Event::Drop { agent: run });
    }
}

fn store_failed(
    domain: &mut Domain,
    env: &Env<Limits>,
    turn: Option<u32>,
    reason: StoreFailure,
    out: &mut Queue<Request>,
) {
    if domain.stop_failed {
        return;
    }
    record_cause(domain, crate::EndCause::Host { turn, reason });
    domain.stop_failed = true;
    crate::person::show_dropped(&mut domain.lines, out);
    domain.chat.closed = true;
    domain.turns.unsaved = Queue::with_capacity(env.limits.unsaved);
    out.push(Request::Notice { notice: Notice::Failure(Failure::Store { turn, reason }) });
    if let Some(record) = domain.saving.take() {
        let owner = domain.delivery_owner.take();
        match &record.state {
            DeliveryState::Answer(agent::run::Delivery::Delivered(_)) => {
                out.push(Request::Notice { notice: Notice::Failure(Failure::DeliveryUnsaved) });
            }
            DeliveryState::Intent(_)
            | DeliveryState::Answer(
                agent::run::Delivery::Nothing
                | agent::run::Delivery::Refused(_)
                | agent::run::Delivery::Failed(_)
                | agent::run::Delivery::Stale,
            ) => {}
        }
        let delivery = match record.state {
            DeliveryState::Intent(_) => {
                agent::run::Delivery::Failed(agent::run::DeliveryFailure::new(0, agent::run::DeliveryReason::Broken))
            }
            DeliveryState::Answer(delivery) => delivery,
        };
        if let Some(owner) = owner {
            deliver_to_agent(domain, env, owner, delivery);
        }
    }
    if let Some(run) = domain.run {
        command(&mut domain.agent, env.now, env.wall, parent::Event::Drop { agent: run });
        domain.chat.phase = Phase::Ending;
    } else {
        domain.chat.phase = Phase::Done;
        out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
    }
}

fn route_agent(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    // Route the current backlog in FIFO order. Replies made while routing
    // remain ready for the next pass, without admitting another external event.
    for _ in 0..domain.agent.backlog() {
        match domain.agent.pop() {
            Some(Output::Parent(request)) => {
                drain_agent_content(domain);
                parent_request(domain, env, request, out);
            }
            Some(Output::Lower { agent, request }) => match request {
                smith_inline_agent::Lower::Cancel { owner } if held_completion(domain, owner) => {}
                request @ (smith_inline_agent::Lower::Complete { .. }
                | smith_inline_agent::Lower::Cancel { .. }
                | smith_inline_agent::Lower::Io { .. }
                | smith_inline_agent::Lower::Job { .. }
                | smith_inline_agent::Lower::CancelIo { .. }
                | smith_inline_agent::Lower::Read { .. }
                | smith_inline_agent::Lower::Probe { .. }
                | smith_inline_agent::Lower::Check { .. }
                | smith_inline_agent::Lower::Abort { .. }) => out.push(Request::Lower { agent, request }),
            },
            Some(Output::Process(request)) => out.push(Request::Process(request)),
            None => break,
        }
    }
    drain_agent_content(domain);
}

fn accept_turn(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u32,
    read: Option<Token>,
    opens_window: bool,
    turn: agent::Turn,
    out: &mut Queue<Request>,
) {
    if domain.stop_failed || domain.agent_failed.is_some() {
        return;
    }
    assert!(domain.turns.unsaved.room() > 0, "store backpressure bounds unsaved turns");
    let shown = crate::person::turn_text(&turn, env.limits.show_bytes);
    domain.turns.unsaved.push((number, turn.sequence));
    domain.lines.fence(read);
    domain.chat.state.read = read;
    observe(domain, Fact::Turn { number });
    out.push(Request::SaveTurn { number, read, opens_window, turn });
    if !domain.config.live_text && !shown.is_empty() {
        out.push(Request::Show { text: shown });
    }
}

fn parent_request(domain: &mut Domain, env: &Env<Limits>, event: parent::Request, out: &mut Queue<Request>) {
    match event {
        parent::Request::Started { agent, .. } => {
            domain.run = Some(agent);
            domain.long = crate::long::Long::new();
        }
        parent::Request::Admitted { .. } => {
            send_line(domain, env, out);
        }
        parent::Request::Turn { turn, .. } => {
            domain.told.turns = turn.number;
            domain.told.spent = turn.spent;
            domain.told.read = turn.read;
            accept_turn(domain, env, turn.number, turn.read, turn.opens_window, turn.body.into_value(), out);
        }
        parent::Request::Answered { answer, detail, .. } => {
            domain.answer_wall = Some(env.wall);
            crate::person::answered(domain, answer, detail, out);
        }
        parent::Request::Waiting { read, .. } => crate::person::waiting(&mut domain.lines, read, out),
        parent::Request::Long { operation, .. } => {
            if domain.long.begin(operation) {
                out.push(Request::Notice { notice: Notice::Checking(Checks::Started) });
            }
        }
        parent::Request::LongDone { .. } => {
            if domain.long.end() {
                out.push(Request::Notice { notice: Notice::Checking(Checks::Ended) });
            }
        }
        parent::Request::Unauthorized { account, .. } => out.push(Request::Credential { account }),
        parent::Request::Limited { account, reset, .. } => {
            let mut reset_at = None;
            if let Some(reset) = reset {
                reset_at = Some(Wall::from_nanos(env.wall.as_nanos().saturating_add(reset.as_nanos())));
            }
            out.push(Request::Notice {
                notice: Notice::Failure(Failure::Account(AccountFailure::Limited { account, reset_at })),
            });
        }
        parent::Request::Refused { account, code, reason, .. } => {
            out.push(Request::Notice {
                notice: Notice::Failure(Failure::Account(AccountFailure::Refused { account, code, reason })),
            });
        }
        parent::Request::Called { call, name, deadline, ask, .. } => match ask {
            host::Ask::Deliver { fields } => begin_delivery(
                domain,
                env,
                agent::run::CallName {
                    activation: name.activation,
                    completion: name.completion,
                    position: name.position,
                },
                call,
                fields.into_value(),
                deadline,
                out,
            ),
            host::Ask::Host { .. } => command(
                &mut domain.agent,
                env.now,
                env.wall,
                parent::Event::Answer {
                    agent: domain.run.expect("call belongs to started spawned run"),
                    call,
                    reply: host::Reply::Unavailable,
                },
            ),
        },
        parent::Request::Withdrawn { .. } | parent::Request::Told { .. } => {}
        parent::Request::Unread { names, .. } => {
            for name in names {
                observe(domain, Fact::Unread { name });
            }
        }
        parent::Request::Faulted { fault, .. } => {
            let dropped = match fault {
                host::Fault::Exited => match env.limits.agent {
                    AgentLimits::Inline(_) => domain.cancel_requested.is_some(),
                    AgentLimits::Spawned(_) => false,
                },
                host::Fault::Rules | host::Fault::TooLarge | host::Fault::NoProgress | host::Fault::WallTime => false,
            };
            let cause = if dropped { crate::EndCause::Cancelled } else { crate::EndCause::Agent { ended: None } };
            record_cause(domain, cause);
            domain.agent_failed = Some(AgentFailure::Fault(fault));
            crate::person::show_dropped(&mut domain.lines, out);
        }
        parent::Request::MessageRefused { name, reason, .. } => {
            domain.lines.refused(name);
            out.push(Request::Notice {
                notice: Notice::NotDelivered { line: Some(name), why: Undelivered::Refused(reason) },
            });
        }
        parent::Request::Gone { end, detail, .. } => gone(domain, end, detail, out),
    }
}

fn gone(domain: &mut Domain, end: host::End, detail: Box<[u8]>, out: &mut Queue<Request>) {
    let ended = match end {
        host::End::Exited { exit, forced } => {
            match forced {
                host::Forced::No => {}
                host::Forced::Terminated | host::Forced::Killed => {
                    if domain.turns.answer.is_some() {
                        out.push(Request::Notice {
                            notice: Notice::Warning(crate::Warning::ForcedExit { exit, forced }),
                        });
                    }
                }
            }
            match exit {
                Some(host::Exit::Code(code)) => Some(smith_events::AgentExit::Code(i64::from(code))),
                Some(host::Exit::Signal(signal)) => Some(smith_events::AgentExit::Signal(u64::from(signal))),
                None => None,
            }
        }
        host::End::Busy | host::End::Invalid(_) | host::End::Unspawned | host::End::Settled => None,
    };
    domain.agent_life = AgentLife::Gone;
    match domain.chat.phase {
        Phase::Answered { .. } => domain.chat.phase = Phase::Answered { gone: Gone::Heard },
        Phase::Loading | Phase::Idle | Phase::Starting | Phase::Running | Phase::Ending | Phase::Done => {}
    }
    domain.run = None;
    if domain.turns.answer.is_none() {
        match domain.end_cause {
            Some(crate::EndCause::Agent { .. }) | None => domain.end_cause = Some(crate::EndCause::Agent { ended }),
            Some(crate::EndCause::Cancelled | crate::EndCause::Host { .. }) => {}
        }
        out.push(Request::Unanswered {
            cause: domain.end_cause.expect("Gone records an end cause"),
            turns: domain.told.turns,
            spent: domain.told.spent,
            read: domain.told.read,
        });
    }
    if domain.turns.answer.is_none() && !domain.stop_failed {
        crate::person::show_dropped(&mut domain.lines, out);
        let failure = match end {
            host::End::Busy | host::End::Invalid(_) => Failure::Unstarted(end),
            host::End::Unspawned | host::End::Exited { .. } | host::End::Settled => Failure::Agent {
                failure: domain.agent_failed.unwrap_or(match end {
                    host::End::Unspawned => AgentFailure::NotStarted,
                    host::End::Exited { .. } | host::End::Settled => AgentFailure::Fault(host::Fault::Exited),
                    host::End::Busy | host::End::Invalid(_) => unreachable!("entrance refusal has no process"),
                }),
                end,
                detail,
            },
        };
        domain.agent_failed = Some(AgentFailure::Fault(host::Fault::Exited));
        match domain.end_cause {
            Some(crate::EndCause::Cancelled) => {}
            Some(crate::EndCause::Host { .. } | crate::EndCause::Agent { .. }) | None => {
                out.push(Request::Notice { notice: Notice::Failure(failure) });
            }
        }
    } else if domain.turns.detail.is_empty() {
        domain.turns.detail = detail;
    }
    if domain.turns.unsaved.is_empty() {
        if domain.agent_failed.is_some() || domain.stop_failed {
            domain.chat.phase = Phase::Done;
            out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
        } else if domain.turns.answer.is_some() {
            crate::person::finish_answer(domain, out);
        }
    }
}

fn held_completion(domain: &Domain, owner: Token) -> bool {
    for io in &domain.held {
        let pending = match &io.terminal {
            smith_inline_agent::Below::Completed { owner, .. }
            | smith_inline_agent::Below::Failed { owner, .. }
            | smith_inline_agent::Below::Cancelled { owner } => *owner,
            smith_inline_agent::Below::Done { .. }
            | smith_inline_agent::Below::Read { .. }
            | smith_inline_agent::Below::Probed { .. }
            | smith_inline_agent::Below::Checked { .. }
            | smith_inline_agent::Below::Aborted { .. } => unreachable!("only completion terminals are held"),
        };
        if pending == owner {
            return true;
        }
    }
    false
}

pub(crate) fn observe(domain: &mut Domain, fact: Fact) {
    if domain.facts.try_push(fact).is_err() {
        domain.facts_lost = domain.facts_lost.saturating_add(1);
    }
}

fn git_result(domain: &mut Domain, env: &Env<Limits>, owner: Token, result: GitResult, out: &mut Queue<Request>) {
    if domain.chat.phase == Phase::Starting && domain.head_survey != HeadSurvey::Complete {
        head_result(domain, env, owner, result, out);
    } else if domain.reconciling.is_some() {
        reconcile::reconcile_git(domain, env, owner, result, out);
    } else {
        delivery::git_result(domain, env, owner, result, out);
    }
}

#[cfg(test)]
impl Domain {
    pub(crate) fn finish_activation(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        use smith_host_domain::{self as host, parent};
        parent_request(
            self,
            env,
            parent::Request::Answered {
                client: Token::new(1),
                answer: host::Answer { read: None, turns: 0, spent: 0, result: host::RunResult::Parked },
                detail: Box::default(),
            },
            out,
        );
        parent_request(
            self,
            env,
            parent::Request::Gone { client: Token::new(1), end: host::End::Settled, detail: Box::default() },
            out,
        );
        step(self, env, Event::Loaded { state: None, transcript: None, deliveries: Box::default() }, out);
    }

    /// A parent-face harness with a live child whose requests are injected by the test.
    pub(crate) fn running_parent(config: Config, limits: &Limits, seed: u64) -> Self {
        let mut domain = Self::new(config, limits, seed).expect("bounded parent harness");
        domain.chat.phase = Phase::Running;
        domain.run = Some(Token::new(1));
        domain
    }

    /// A parent-face harness retaining one durable delivery owner through withdrawal.
    pub(crate) fn running_delivery(config: Config, limits: &Limits, seed: u64, owner: Token) -> Self {
        let mut domain = Self::running_parent(config, limits, seed);
        domain.delivery_owner = Some(owner);
        domain
    }
}

#[cfg(test)]
#[path = "parent_tests.rs"]
mod parent_tests;

#[cfg(test)]
#[path = "program_check_tests.rs"]
mod program_check_tests;

#[cfg(test)]
#[path = "child_backlog_tests.rs"]
mod child_backlog_tests;
