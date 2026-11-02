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
use crate::chat::{Chat, HeadSurvey, Live, Starting, State};
use crate::child::Child;
#[path = "chat/machine.rs"]
mod chat_machine;
#[path = "delivery/machine.rs"]
mod delivery;
#[path = "reconcile.rs"]
mod reconcile;
use crate::person::Lines;
use crate::turns::Turns;
use crate::{AgentLimits, Config, Fact, Invalid, Limits, charter};
use chat_machine::{
    chat_selected, credential, drop_run, head_result, interrupt, loaded, no_credential, receive_line, resume_loading,
    send_line, state_saved, store_failed, turn_saved,
};
pub(crate) use chat_machine::{closed, select_chat};
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
    pub(crate) child: Child,
    pub(crate) ending: crate::end::End,
    pub(crate) chat: Chat,
    transcript: Option<agent::Transcript>,
    records: Map<agent::run::CallName, DeliveryRecord>,
    delivery: Option<InPlace>,
    saving: Option<DeliveryRecord>,
    delivery_owner: Option<Token>,
    reconciling: Option<Reconcile>,
    pub(crate) lines: Lines,
    run_messages: Queue<host::Message>,
    pub(crate) turns: Turns,
    held: Queue<Held>,
    told: crate::end::Told,
    answer_wall: Option<Wall>,
    pub(crate) show_bytes: u32,
    long: crate::long::Long,
    facts: Queue<Fact>,
    facts_lost: u64,
}

/// One completion terminal held under the local store's acknowledgement pressure.
#[derive(Debug)]
pub(crate) struct Held {
    agent: Token,
    terminal: smith_inline_agent::Below,
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
        assert!(self.chat.phase.is_loading() && !self.chat.load_requested(), "startup precedes Load");
        self.chat.request_load();
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
            child: Child::Starting,
            ending: crate::end::End::new(),
            transcript: None,
            records: Map::with_capacity(limits.smith.run.answered_calls),
            delivery: None,
            saving: None,
            delivery_owner: None,
            reconciling: None,
            lines,
            run_messages: Queue::with_capacity(limits.smith.run.messages.saturating_add(1)),
            turns: Turns::new(limits.unsaved),
            held: Queue::with_capacity(1),
            told: crate::end::Told::new(),
            answer_wall: None,
            show_bytes: limits.show_bytes,
            long: crate::long::Long::new(),
            facts: Queue::with_capacity(limits.facts),
            facts_lost: 0,
        })
    }

    /// First recorded cause of an unanswering end, retained through lower settlement.
    #[must_use]
    pub const fn end_cause(&self) -> Option<crate::EndCause> {
        self.ending.cause()
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

    /// The sent charter's refused effort and its startup model declaration.
    #[must_use]
    pub fn refused_effort(&self) -> crate::ChoiceNotice {
        crate::ChoiceNotice::Effort {
            effort: self.effort.as_deref().unwrap_or_default().into(),
            choices: self.config.efforts.get(self.main_model).expect("selected startup efforts").clone(),
            dropped: false,
        }
    }

    /// Whether initial loading or the child has deferred work.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        (self.chat.phase.is_loading() && !self.chat.load_requested()) || self.agent.ready()
    }

    /// Earliest child deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.pressured() { self.chat.deadline() } else { self.agent.deadline() }
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
    chat_machine::follow_chat(domain, env, out);
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
        | smith_inline_agent::Below::Aborted { .. }
        | smith_inline_agent::Below::ReadVersion { .. }
        | smith_inline_agent::Below::BuiltinResult { .. } => false,
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
        if let Some(deadline) = domain.chat.deadline()
            && deadline <= env.now
            && domain.chat.phase.is_running()
        {
            interrupt(domain, env, out);
        }
    } else {
        domain.agent.fire(env.now, env.wall);
    }
    route_agent(domain, env, out);
}

/// Ask for the initial transcript or advance one ready child handoff.
pub fn resume(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    drain_agent_content(domain);
    if domain.chat.phase.is_loading() && !domain.chat.load_requested() {
        domain.chat.request_load();
        out.push(Request::Load);
    } else if domain.agent.ready() {
        domain.agent.resume(env.now, env.wall);
        route_agent(domain, env, out);
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
                | smith_inline_agent::Lower::Abort { .. }
                | smith_inline_agent::Lower::Yield { .. }
                | smith_inline_agent::Lower::Close { .. }) => out.push(Request::Lower { agent, request }),
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
    if domain.ending.store_failed() || domain.child.failure().is_some() {
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
            domain.child.started(agent);
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
                    agent: domain.child.run().expect("call belongs to started spawned run"),
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
                    AgentLimits::Inline(_) => domain.ending.requested().is_some(),
                    AgentLimits::Spawned(_) => false,
                },
                host::Fault::Rules | host::Fault::TooLarge | host::Fault::NoProgress | host::Fault::WallTime => false,
            };
            let cause = if dropped { crate::EndCause::Cancelled } else { crate::EndCause::Agent { ended: None } };
            domain.ending.record(cause);
            domain.child.faulted(AgentFailure::Fault(fault));
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
    chat_machine::follow_chat(domain, env, out);
}

fn gone(domain: &mut Domain, end: host::End, detail: Box<[u8]>, out: &mut Queue<Request>) {
    let ended = match end {
        host::End::Exited { exit, forced } => {
            match forced {
                host::Forced::No => {}
                host::Forced::Terminated | host::Forced::Killed => {
                    if domain.chat.phase.is_answered() {
                        out.push(Request::Notice {
                            notice: Notice::Warning(crate::Warning::ForcedExit { exit, forced }),
                        });
                    }
                }
            }
            exit
        }
        host::End::Busy | host::End::Invalid(_) | host::End::Unspawned | host::End::Settled => None,
    };
    domain.child.gone();

    if !domain.chat.phase.is_answered() {
        domain.ending.gone(ended);
        out.push(Request::Unanswered {
            cause: domain.ending.cause().expect("Gone records an end cause"),
            turns: domain.told.turns,
            spent: domain.told.spent,
            read: domain.told.read,
        });
    }
    if !domain.chat.phase.is_answered() && !domain.ending.store_failed() {
        crate::person::show_dropped(&mut domain.lines, out);
        let failure = match end {
            host::End::Busy | host::End::Invalid(_) => Failure::Unstarted(end),
            host::End::Unspawned | host::End::Exited { .. } | host::End::Settled => Failure::Agent {
                failure: domain.child.failure().unwrap_or(match end {
                    host::End::Unspawned => AgentFailure::NotStarted,
                    host::End::Exited { .. } | host::End::Settled => AgentFailure::Fault(host::Fault::Exited),
                    host::End::Busy | host::End::Invalid(_) => unreachable!("entrance refusal has no process"),
                }),
                end,
                detail,
            },
        };
        domain.child.faulted(AgentFailure::Fault(host::Fault::Exited));
        match domain.ending.cause() {
            Some(crate::EndCause::Cancelled) => {}
            Some(crate::EndCause::Host { .. } | crate::EndCause::Agent { .. }) | None => {
                out.push(Request::Notice { notice: Notice::Failure(failure) });
            }
        }
    } else {
        match &mut domain.chat.phase {
            State::Answered { detail: held, .. } if held.is_empty() => *held = detail,
            State::Loading(_)
            | State::Idle
            | State::Starting(_)
            | State::Running { .. }
            | State::Ending(_)
            | State::Answered { .. }
            | State::Done => {}
        }
    }
    if domain.turns.unsaved.is_empty() {
        if domain.child.failure().is_some() || domain.ending.store_failed() {
            domain.chat.phase = State::Done;
            out.push(Request::Exit { status: crate::end::exit_status(domain.ending.cause()) });
        } else if domain.chat.phase.is_answered() {
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
            | smith_inline_agent::Below::Aborted { .. }
            | smith_inline_agent::Below::ReadVersion { .. }
            | smith_inline_agent::Below::BuiltinResult { .. } => unreachable!("only completion terminals are held"),
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
    if domain.chat.phase.is_starting() && domain.chat.starting().survey != HeadSurvey::Complete {
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
        parent_request(self, env, crate::fixture::gone(host::End::Settled, Box::default()), out);
        step(self, env, crate::fixture::loaded(), out);
    }

    /// A parent-face harness with a live child whose requests are injected by the test.
    pub(crate) fn starting_parent(config: Config, limits: &Limits, seed: u64) -> Self {
        let mut domain = Self::new(config, limits, seed).expect("bounded parent harness");
        domain.chat.phase = State::Running {
            live: Live {
                heads: Map::with_capacity(limits.smith.run.directories),
                own_heads: Map::with_capacity(limits.smith.run.directories),
            },
            deadline: Time::ZERO,
        };
        domain
    }

    pub(crate) fn running_parent(config: Config, limits: &Limits, seed: u64) -> Self {
        let mut domain = Self::starting_parent(config, limits, seed);
        domain.child.started(Token::new(1));
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
