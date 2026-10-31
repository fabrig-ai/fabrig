//! Supervision cells and surviving terminal ledgers (domain/host.md, sections
//! 2–7 and 9.2). Process exit never discards parent calls. A parent's drop
//! releases held turns unacknowledged, retaining submitted send terminals.
//! The kit knows channel send order, not private agent stop
//! decisions; policy and durable decisions remain in the parent.
//! Content-free facts are retained losslessly until the parent drains them.
use crate::{
    Ask, CallName, Down, End, Exit, Fact, Fault, Forced, Grant, Input, Limits, MessageRefusal, Output, Reply,
    RunFailure, RunResult, Signal, Start, Up,
};
use crate::{parent, process};

mod admit;
mod reading;
mod record;
mod supervision;
use admit::{valid_start, within};
use record::{too_large, valid_record};
mod tree;
use supervision::State;
mod word;
use alloc::boxed::Box;
use skein_lib::{Deadlines, Env, Id, Map, Queue, Slab, Time, Token};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Alarm {
    Watch { agent: Id<Agent> },
    Wall { agent: Id<Agent> },
    Grace { agent: Id<Agent> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Host,
    Delivery,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SendState {
    Parent,
    Queued,
    Sending,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Call {
    name: CallName,
    deadline: Time,
    kind: Kind,
    stage: SendState,
    withdrawn: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TurnMeta {
    bytes: u64,
    stage: SendState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Account {
    latest: Grant,
    emitted: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct BusyCall {
    call: Token,
    name: CallName,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Sent {
    Start,
    Message,
    Call(Token),
    Busy { call: Token, name: CallName },
    Ack(u32),
    Grant,
    Cancel,
}

#[derive(Debug)]
pub(crate) struct Agent {
    client: Token,
    logical_run: Token,
    activation: u64,
    supervision: State,
    tree: tree::State,
    word: word::State,
    host_stopped: bool,
    diagnostic: Option<Fault>,
    reading: reading::State,
    sending: Option<Sent>,
    signals: u32,
    forced: Forced,
    outbox: Queue<Down>,
    busy: Queue<BusyCall>,
    calls: Map<Token, Call>,
    turns: Map<u32, TurnMeta>,
    accounts: Map<u32, Account>,
    messages: u32,
    unread: crate::Inbox,
    number: u32,
    spent: u64,
    turn_bytes: u64,
    long: crate::long::Long,
    // Paused only while the run waits with no message still queued or unread.
    waiting: bool,
}

/// Bounded process/channel state, parent terminal rights and lossless content-free facts;
/// no credentials, durable host policy or frame bytes.
#[derive(Debug)]
pub struct Domain {
    agents: Slab<Agent>,
    alarms: Deadlines<Alarm>,
    facts: Queue<Fact>,
}

impl Domain {
    /// Allocate bounded state and uncapped facts; invalid arithmetic/turn/delivery capacity
    /// panics before any process effect.
    #[must_use]
    pub fn new(limits: &Limits) -> Domain {
        assert!(crate::valid(limits), "host limits are representable and every sealed delivery fits");
        assert!(crate::check(limits).is_ok(), "host supervision exceeds the agent's checked bounds");
        Domain {
            agents: Slab::<Agent>::with_capacity(limits.agents),
            alarms: Deadlines::with_capacity(limits.agents.checked_mul(3).expect("valid limits")),
            facts: Queue::uncapped(),
        }
    }

    /// Retired slots count until iteration-boundary reclaim.
    #[must_use]
    pub fn agents(&self) -> u32 {
        self.agents.len()
    }

    /// Pure earliest armed monotonic deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.alarms.next()
    }

    /// Pure scheduling query; expiration itself occurs in fire.
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.alarms.next() {
            Some(at) => at <= now,
            None => false,
        }
    }

    /// Reclaim only after parent routed outputs for this iteration.
    pub fn reclaim(&mut self) {
        self.agents.reclaim();
    }

    /// Drain one losslessly retained observation; decisions never depend on it.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// Entries held for the parent to drain (skein/docs/foundation/programming-model.md, section 7.1): the parent root counts them in its backlog.
    #[must_use]
    pub fn backlog(&self) -> u32 {
        self.facts.len()
    }

    /// Lossless observation retention leaves no dropped facts.
    #[must_use]
    pub const fn facts_lost(&self) -> u64 {
        0
    }
}

/// Consume one parent notice or lower terminal with injected clocks.
/// Caller drains the ordered outputs and returns every operation terminal,
/// even after cancellation/EOF.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Input, out: &mut Queue<Output>) {
    let before = out.len();
    let id = match event {
        Input::Parent(event) => from_parent(domain, env, event, out),
        Input::Process(event) => from_process(domain, env, event, out),
    };
    if let Some(id) = id {
        follow(domain, env, id, out);
        observations(domain, out, before);
    }
}

fn current(domain: &mut Domain, token: Token) -> Option<&mut Agent> {
    let agent = domain.agents.get_mut(Id::from_token(token))?;
    if agent.supervision.closed() { None } else { Some(agent) }
}

fn from_parent(
    domain: &mut Domain,
    env: &Env<Limits>,
    event: parent::Event,
    out: &mut Queue<Output>,
) -> Option<Id<Agent>> {
    let token = match event {
        parent::Event::Spawn { client, start } => {
            spawn(domain, env, client, start, out);
            return None;
        }
        parent::Event::Message { agent: token, finish_requested, name, label, text } => {
            message(current(domain, token)?, name, label, text, finish_requested, env, out);
            token
        }
        parent::Event::Answer { agent: token, call, reply } => {
            answered(current(domain, token)?, call, reply, env);
            token
        }
        parent::Event::Acknowledge { agent: token, turn } => {
            acknowledge_turn(current(domain, token)?, turn);
            token
        }
        parent::Event::Grant { agent: token, grant } => {
            grant_name(current(domain, token)?, grant);
            token
        }
        parent::Event::Drop { agent: token } => {
            drop_run(current(domain, token)?, token, env, out);
            token
        }
        parent::Event::Stop { agent: token } => {
            let agent = current(domain, token)?;
            match agent.supervision {
                State::Watching(_) | State::Cancelled { .. } | State::Draining { .. } => agent.host_stopped = true,
                State::Spawning
                | State::Exiting { .. }
                | State::Terminating { .. }
                | State::Killing
                | State::Closed => {}
            }
            if agent.supervision.fault() == Some(Fault::WallTime) || agent.supervision.fault() == Some(Fault::Exited) {
                agent.supervision.clear_fault();
            }
            polite(agent, env, None);
            token
        }
    };
    Some(Id::from_token(token))
}

fn from_process(
    domain: &mut Domain,
    env: &Env<Limits>,
    event: process::Event,
    out: &mut Queue<Output>,
) -> Option<Id<Agent>> {
    let token = match &event {
        process::Event::Spawned { owner, .. }
        | process::Event::Unspawned { owner, .. }
        | process::Event::Sent { owner }
        | process::Event::Unsent { owner }
        | process::Event::Received { owner, .. }
        | process::Event::Progress { owner }
        | process::Event::Malformed { owner }
        | process::Event::Hangup { owner }
        | process::Event::Signalled { owner }
        | process::Event::Exited { owner, .. }
        | process::Event::Reaped { owner, .. } => *owner,
    };
    let agent = current(domain, token)?;
    match event {
        process::Event::Spawned { process, .. } => tree::spawned(agent, token, process, env, out),
        process::Event::Unspawned { detail, .. } => tree::unspawned(agent, &detail, env, out),
        process::Event::Sent { .. } => send_terminal(agent, false, env),
        process::Event::Unsent { .. } => send_terminal(agent, true, env),
        process::Event::Received { message, .. } => {
            agent.reading.received();
            receive(agent, token, message, env, out);
        }
        process::Event::Progress { .. } => {
            agent.reading.received();
            if agent.word.last_word() {
                fail(agent, token, Fault::Rules, env, out);
            } else if !agent.word.told() && !agent.supervision.forced() {
                if agent.word.admitted() {
                    agent.supervision.progress(env.now);
                } else {
                    fail(agent, token, Fault::Rules, env, out);
                }
            }
        }
        process::Event::Malformed { .. } => {
            agent.reading.end();
            fail(agent, token, Fault::Rules, env, out);
        }
        process::Event::Hangup { .. } => {
            agent.reading.end();
            if !agent.word.last_word() {
                match agent.supervision {
                    State::Cancelled { .. } | State::Draining { .. } => {
                        if let Some(fault) = agent.supervision.take_fault() {
                            report_fault(agent, fault, out);
                        }
                        agent.supervision.hangup();
                    }
                    State::Watching(_) => fail(agent, token, Fault::Exited, env, out),
                    State::Exiting { .. } | State::Terminating { .. } | State::Killing => {}
                    State::Spawning | State::Closed => unreachable!("spawned process reads"),
                }
            }
        }
        process::Event::Signalled { .. } => {
            agent.signals = agent.signals.checked_sub(1).expect("one terminal per signal");
        }
        process::Event::Exited { exit, .. } => tree::exited(agent, exit, env),
        process::Event::Reaped { detail, .. } => tree::reaped(agent, &detail, env),
    }
    Some(Id::from_token(token))
}

fn send_terminal(agent: &mut Agent, unsent: bool, env: &Env<Limits>) {
    let sent = agent.sending.take().expect("send terminal ends one pending send");
    settle_send(agent, sent);
    if unsent && !agent.word.last_word() {
        draining(agent, env);
    }
}

/// Expire one due timer; repeat while due and drain the outputs. Wall never
/// pauses with progress credit.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Output>) {
    let before = out.len();
    let Some(alarm) = domain.alarms.expire(env.now) else {
        return;
    };
    let id = match alarm {
        Alarm::Watch { agent } | Alarm::Wall { agent } | Alarm::Grace { agent } => agent,
    };
    let agent = domain.agents.get_mut(id).expect("armed agent lives");
    match alarm {
        Alarm::Watch { .. } => {
            if pause(agent, env) {
                agent.supervision.pause(true, env.now);
            } else if agent.supervision.watch().expect("watch alarm owns watching state").paused {
                agent.supervision.pause(false, env.now);
            } else {
                fail(agent, id.token(), Fault::NoProgress, env, out);
            }
        }
        Alarm::Wall { .. } => polite(agent, env, Some(Fault::WallTime)),
        Alarm::Grace { .. } => {
            if let Some(fault) = agent.supervision.take_fault() {
                report_fault(agent, fault, out);
            }
            match agent.supervision {
                State::Cancelled { .. } | State::Draining { .. } | State::Exiting { .. } => {
                    terminate(agent, id.token(), env, out);
                }
                State::Terminating { .. } => {
                    signal(agent, id.token(), Signal::Kill, out);
                    agent.supervision.kill();
                }
                State::Spawning | State::Watching(_) | State::Killing | State::Closed => {
                    unreachable!("grace belongs to shutdown")
                }
            }
        }
    }
    follow(domain, env, id, out);
    observations(domain, out, before);
}

fn spawn(domain: &mut Domain, env: &Env<Limits>, client: Token, start: Start, out: &mut Queue<Output>) {
    let refusal = if domain.agents.is_full() {
        Some(End::Busy)
    } else {
        match valid_start(&start, &env.limits) {
            Some(invalid) => Some(End::Invalid(invalid)),
            None => None,
        }
    };
    if let Some(end) = refusal {
        out.push(Output::Parent(parent::Request::Gone { client, end, detail: Box::new([]) }));
        observation(domain, Fact::Gone { client, end });
        return;
    }
    let mut accounts = Map::with_capacity(env.limits.accounts);
    for grant in &start.grants {
        accounts.insert(grant.account, Account { latest: *grant, emitted: 0 }).expect("validated unique grant names");
    }
    let mut unread = crate::Inbox::new(env.limits.messages);
    for message in &start.messages {
        unread
            .admit(message.name, &message.label, &message.text, env.limits.message_bytes)
            .expect("validated start messages");
    }
    let workspace = start.workspace;
    let logical_run = start.logical_run;
    let activation = start.activation;
    let agent = Agent {
        client,
        logical_run,
        activation,
        supervision: State::Spawning,
        tree: tree::State::Spawning { start },
        word: word::State::Open(word::Admission::Pending),
        host_stopped: false,
        diagnostic: None,
        reading: reading::State::Idle,
        sending: None,
        signals: 0,
        forced: Forced::No,
        outbox: Queue::with_capacity(crate::limits::outbox(&env.limits).expect("validated outbox")),
        busy: Queue::with_capacity(2),
        calls: Map::with_capacity(env.limits.calls),
        turns: Map::with_capacity(env.limits.turns),
        accounts,
        messages: 0,
        unread,
        number: 0,
        spent: 0,
        turn_bytes: 0,
        long: crate::long::Long::new(env.now),
        waiting: false,
    };
    let id = domain.agents.insert(agent).expect("slot admitted");
    out.push(Output::Process(process::Request::Spawn {
        owner: id.token(),
        workspace,
        deadline: env.now.saturating_add(env.limits.spawn_timeout),
    }));
}

fn tail(detail: &[u8], limit: u32) -> Box<[u8]> {
    let keep = detail.len().min(usize::try_from(limit).expect("u32 fits usize"));
    let from = detail.len().checked_sub(keep).expect("bounded tail");
    Box::from(detail.get(from..).expect("bounded suffix"))
}

fn accepts(agent: &Agent) -> bool {
    match agent.supervision {
        State::Watching(_) => true,
        State::Spawning
        | State::Cancelled { .. }
        | State::Draining { .. }
        | State::Exiting { .. }
        | State::Terminating { .. }
        | State::Killing
        | State::Closed => false,
    }
}

fn listens(agent: &Agent) -> bool {
    match agent.supervision {
        State::Watching(_) | State::Cancelled { .. } => {
            !agent.reading.ended() && !agent.tree.exited() && !agent.word.last_word()
        }
        State::Spawning
        | State::Draining { .. }
        | State::Exiting { .. }
        | State::Terminating { .. }
        | State::Killing
        | State::Closed => false,
    }
}

fn message(
    agent: &mut Agent,
    name: Token,
    label: Box<[u8]>,
    text: Box<[u8]>,
    finish_requested: bool,
    env: &Env<Limits>,
    out: &mut Queue<Output>,
) {
    let admitted = if accepts(agent) {
        agent.unread.admit(name, &label, &text, env.limits.message_bytes)
    } else {
        Err(MessageRefusal::Ending)
    };
    if let Err(reason) = admitted {
        out.push(Output::Parent(parent::Request::MessageRefused { client: agent.client, name, reason }));
        return;
    }
    agent.messages = agent.messages.checked_add(1).expect("bounded messages");
    agent.waiting = false;
    agent.outbox.push(Down::Message { finish_requested, name, label, text });
}

fn acknowledge_turn(agent: &mut Agent, number: u32) {
    let Some(turn) = agent.turns.get_mut(&number) else {
        assert!(number <= agent.number, "parent ACK names a forwarded turn");
        return;
    };
    match turn.stage {
        SendState::Parent => {
            turn.stage = SendState::Queued;
            agent.turn_bytes = agent.turn_bytes.checked_sub(turn.bytes).expect("outstanding turn bytes");
            if listens(agent) {
                agent.outbox.push(Down::Acknowledge { turn: number });
            } else {
                agent.turns.remove(&number).expect("exact ACK metadata");
            }
        }
        SendState::Queued | SendState::Sending => {}
    }
}

fn grant_name(agent: &mut Agent, grant: Grant) {
    if !listens(agent) {
        return;
    }
    let Some(known) = agent.accounts.get_mut(&grant.account) else {
        return;
    };
    if grant.generation <= known.latest.generation {
        return;
    }
    known.latest = grant;
    let count = agent.outbox.len();
    let mut replaced = false;
    for _ in 0..count {
        let old = agent.outbox.pop().expect("stable bounded outbox");
        let next = match old {
            Down::Grant { grant: old } if old.account == grant.account => {
                replaced = true;
                Down::Grant { grant }
            }
            old @ (Down::Grant { .. }
            | Down::Start { .. }
            | Down::Message { .. }
            | Down::Answer { .. }
            | Down::Acknowledge { .. }
            | Down::Cancel) => old,
        };
        agent.outbox.push(next);
    }
    if !replaced {
        agent.outbox.push(Down::Grant { grant });
    }
}

fn answered(agent: &mut Agent, callback: Token, reply: Reply, env: &Env<Limits>) {
    let call = *agent.calls.get(&callback).expect("parent returns one known operation right");
    assert!(call.stage == SendState::Parent, "one parent answer per call");
    let reply = match reply {
        Reply::Delivery(delivery) => {
            assert!(call.kind == Kind::Delivery, "delivery terminal matches delivery call");
            Reply::Delivery(delivery)
        }
        Reply::Host { error, body } => {
            assert!(call.kind == Kind::Host, "tool terminal matches tool call");
            if within(&body, env.limits.answer_bytes) { Reply::Host { error, body } } else { Reply::TooLarge }
        }
        reply @ (Reply::Busy | Reply::Unavailable | Reply::Withdrawn | Reply::TooLarge) => {
            assert!(call.kind == Kind::Host, "submitted delivery owes its full terminal");
            reply
        }
    };
    if listens(agent) {
        agent.calls.get_mut(&callback).expect("known call").stage = SendState::Queued;
        agent.outbox.push(Down::Answer { call: callback, reply });
    } else {
        agent.calls.remove(&callback).expect("parent right settled");
    }
}

fn receive(agent: &mut Agent, owner: Token, message: Up, env: &Env<Limits>, out: &mut Queue<Output>) {
    if agent.word.last_word() {
        fail(agent, owner, Fault::Rules, env, out);
        return;
    }
    if agent.word.told() || discarded_work(agent, &message) {
        return;
    }
    if too_large(&message, &env.limits) {
        fail(agent, owner, Fault::TooLarge, env, out);
        return;
    }
    if !valid_record(agent, &message, env) {
        fail(agent, owner, Fault::Rules, env, out);
        return;
    }
    agent.supervision.progress(env.now);
    match message {
        Up::Admitted => {
            agent.word.admit();
            out.push(Output::Parent(parent::Request::Admitted { client: agent.client }));
        }
        Up::Call { call, name, deadline, ask } => {
            let kind = match &ask {
                Ask::Host { .. } => Kind::Host,
                Ask::Deliver { .. } => Kind::Delivery,
            };
            agent.waiting = false;
            if agent.calls.len() >= env.limits.calls {
                agent.busy.push(BusyCall { call, name });
            } else {
                agent
                    .calls
                    .insert(call, Call { name, deadline, kind, stage: SendState::Parent, withdrawn: false })
                    .expect("call admitted within capacity");
                out.push(Output::Parent(parent::Request::Called {
                    client: agent.client,
                    logical_run: agent.logical_run,
                    call,
                    name,
                    deadline,
                    ask,
                }));
            }
        }
        Up::Withdraw { call } => match agent.calls.get_mut(&call) {
            Some(entry) if entry.stage == SendState::Parent => {
                entry.withdrawn = true;
                out.push(Output::Parent(parent::Request::Withdrawn { client: agent.client, call }));
            }
            Some(_) | None => {}
        },
        Up::MessageRefused { name, reason } => {
            assert!(agent.unread.refused(name), "validated refusal owns an accepted message");
            out.push(Output::Parent(parent::Request::MessageRefused { client: agent.client, name, reason }));
        }
        Up::Turn { turn } => {
            agent.waiting = false;
            agent.unread.fence(turn.read);
            agent.number = turn.number;
            agent.spent = turn.spent;
            let bytes = turn.body.owned_bytes();
            agent.turn_bytes = agent.turn_bytes.checked_add(bytes).expect("validated byte credit");
            agent
                .turns
                .insert(turn.number, TurnMeta { bytes, stage: SendState::Parent })
                .expect("maximum turn room reserved before read");
            out.push(Output::Parent(parent::Request::Turn { client: agent.client, turn }));
        }
        Up::Waiting { read } => {
            agent.unread.fence(read);
            agent.waiting = agent.messages == 0 && agent.unread.is_empty();
            out.push(Output::Parent(parent::Request::Waiting { client: agent.client, read }));
        }
        Up::Long { operation, span } => {
            agent.waiting = false;
            agent.long.begin(env.now.saturating_add(span));
            out.push(Output::Parent(parent::Request::Long { client: agent.client, operation, span }));
        }
        Up::LongDone => {
            agent.long.end();
            out.push(Output::Parent(parent::Request::LongDone { client: agent.client }));
        }
        Up::Fact { body } => out.push(Output::Parent(parent::Request::Told { client: agent.client, body })),
        Up::Unauthorized { account, generation } => {
            out.push(Output::Parent(parent::Request::Unauthorized { client: agent.client, account, generation }));
        }
        Up::Limited { account, reset } => {
            out.push(Output::Parent(parent::Request::Limited { client: agent.client, account, reset }));
        }
        Up::Refused { account, code, reason } => {
            out.push(Output::Parent(parent::Request::Refused { client: agent.client, account, code, reason }));
        }
        Up::Answer { answer } => heard_answer(agent, answer, env, out),
    }
}

fn discarded_work(agent: &Agent, message: &Up) -> bool {
    match agent.supervision {
        State::Terminating { .. } | State::Killing => match message {
            Up::Answer { .. } => agent.word.dropped(),
            Up::MessageRefused { .. } => false,
            Up::Admitted
            | Up::Call { .. }
            | Up::Withdraw { .. }
            | Up::Turn { .. }
            | Up::Fact { .. }
            | Up::Long { .. }
            | Up::LongDone
            | Up::Waiting { .. }
            | Up::Unauthorized { .. }
            | Up::Refused { .. }
            | Up::Limited { .. } => true,
        },
        State::Spawning
        | State::Watching(_)
        | State::Cancelled { .. }
        | State::Draining { .. }
        | State::Exiting { .. }
        | State::Closed => false,
    }
}

fn heard_answer(agent: &mut Agent, answer: crate::Answer, env: &Env<Limits>, out: &mut Queue<Output>) {
    agent.unread.fence(answer.read);
    agent.spent = answer.spent;
    let wall_cancel = match &answer.result {
        RunResult::Failed { failure: RunFailure::Cancelled } => agent.supervision.fault() == Some(Fault::WallTime),
        RunResult::Refused { .. } | RunResult::Accepted { .. } | RunResult::Parked | RunResult::Failed { .. } => false,
    };
    if wall_cancel {
        report_fault(agent, Fault::WallTime, out);
    } else {
        out.push(Output::Parent(parent::Request::Answered { client: agent.client, answer, detail: Box::default() }));
        report_unread(agent, out);
    }
    agent.word.answer();
    agent.supervision.clear_fault();
    agent.supervision.answer(env.now, env.limits.exit_grace);
}

fn settle_send(agent: &mut Agent, sent: Sent) {
    match sent {
        Sent::Call(callback) => {
            let call = agent.calls.remove(&callback).expect("call transmission still reserved");
            assert!(call.stage == SendState::Sending, "send terminal owns reservation");
        }
        Sent::Ack(number) => {
            let turn = agent.turns.remove(&number).expect("ACK transmission still reserved");
            assert!(turn.stage == SendState::Sending, "ACK send terminal owns metadata");
        }
        Sent::Start | Sent::Message | Sent::Busy { .. } | Sent::Grant | Sent::Cancel => {}
    }
}

fn polite(agent: &mut Agent, env: &Env<Limits>, owed: Option<Fault>) {
    if agent.supervision.cancel(env.now, env.limits.cancel_grace, owed) {
        agent.outbox.push(Down::Cancel);
    }
}

fn draining(agent: &mut Agent, env: &Env<Limits>) {
    agent.supervision.drain(env.now, env.limits.cancel_grace);
}

fn report_unread(agent: &mut Agent, out: &mut Queue<Output>) {
    if let Some(names) = agent.unread.report() {
        out.push(Output::Parent(parent::Request::Unread { client: agent.client, names }));
    }
}

fn report_fault(agent: &mut Agent, fault: Fault, out: &mut Queue<Output>) {
    if !agent.word.told() {
        report_unread(agent, out);
        agent.word.fault();
        out.push(Output::Parent(parent::Request::Faulted { client: agent.client, fault }));
    }
}

fn fail(agent: &mut Agent, owner: Token, fault: Fault, env: &Env<Limits>, out: &mut Queue<Output>) {
    if !agent.host_stopped && !agent.word.told() {
        report_fault(agent, fault, out);
    } else {
        agent.diagnostic = Some(fault);
    }
    agent.supervision.clear_fault();
    match agent.supervision {
        State::Watching(_) | State::Cancelled { .. } | State::Draining { .. } | State::Exiting { .. } => {
            terminate(agent, owner, env, out);
        }
        State::Terminating { .. } | State::Killing => {}
        State::Spawning | State::Closed => unreachable!("only spawned processes fail channel/progress"),
    }
}

fn drop_run(agent: &mut Agent, owner: Token, env: &Env<Limits>, out: &mut Queue<Output>) {
    agent.word.drop_run();
    agent.host_stopped = true;
    agent.supervision.clear_fault();
    let mut released = skein_lib::List::with_capacity(env.limits.turns);
    for (number, turn) in &agent.turns {
        match turn.stage {
            SendState::Parent => released.push(*number).expect("bounded held turns"),
            SendState::Queued | SendState::Sending => {}
        }
    }
    for number in &released {
        let turn = agent.turns.remove(number).expect("held turn released exactly once");
        agent.turn_bytes = agent.turn_bytes.checked_sub(turn.bytes).expect("held turn byte credit");
    }
    match agent.supervision {
        State::Watching(_) | State::Cancelled { .. } | State::Draining { .. } | State::Exiting { .. } => {
            terminate(agent, owner, env, out);
        }
        State::Spawning | State::Terminating { .. } | State::Killing | State::Closed => {}
    }
}

fn terminate(agent: &mut Agent, owner: Token, env: &Env<Limits>, out: &mut Queue<Output>) {
    signal(agent, owner, Signal::Terminate, out);
    agent.supervision.terminate(env.now, env.limits.kill_after);
}

fn signal(agent: &mut Agent, owner: Token, signal: Signal, out: &mut Queue<Output>) {
    agent.forced = match signal {
        Signal::Terminate => Forced::Terminated,
        Signal::Kill => Forced::Killed,
    };
    agent.signals = agent.signals.checked_add(1).expect("two bounded tree signals at most");
    out.push(Output::Process(process::Request::Signal { owner, process: agent.tree.process(), signal }));
}

fn turn_room(agent: &Agent, limits: &Limits) -> bool {
    agent.turns.len() < limits.turns
        && match agent.turn_bytes.checked_add(limits.turn_bytes) {
            Some(bytes) => bytes <= limits.unacknowledged_bytes,
            None => false,
        }
}

fn pause(agent: &Agent, env: &Env<Limits>) -> bool {
    if agent.waiting || !turn_room(agent, &env.limits) || agent.busy.room() == 0 {
        return true;
    }
    for (_, call) in &agent.calls {
        if call.stage == SendState::Parent && call.deadline > env.now {
            return true;
        }
    }
    false
}

fn send_next(agent: &mut Agent, owner: Token, out: &mut Queue<Output>) {
    if agent.sending.is_some() || !listens(agent) {
        return;
    }
    let (sent, message) = match agent.busy.pop() {
        Some(BusyCall { call, name }) => (Sent::Busy { call, name }, Down::Answer { call, reply: Reply::Busy }),
        None => {
            let Some(message) = agent.outbox.pop() else {
                return;
            };
            let sent = match &message {
                Down::Start { .. } => unreachable!("Start sent directly once"),
                Down::Message { name, .. } => {
                    agent.messages = agent.messages.checked_sub(1).expect("queued message count");
                    agent.unread.sent(*name);
                    Sent::Message
                }
                Down::Answer { call, .. } => {
                    let entry = agent.calls.get_mut(call).expect("answer retains callback until Sent");
                    entry.stage = SendState::Sending;
                    Sent::Call(*call)
                }
                Down::Acknowledge { turn } => {
                    agent.turns.get_mut(turn).expect("exact ACK metadata").stage = SendState::Sending;
                    Sent::Ack(*turn)
                }
                Down::Grant { grant } => {
                    agent.accounts.get_mut(&grant.account).expect("known grant").emitted = grant.generation;
                    Sent::Grant
                }
                Down::Cancel => Sent::Cancel,
            };
            (sent, message)
        }
    };
    agent.sending = Some(sent);
    out.push(Output::Process(process::Request::Send { owner, process: agent.tree.process(), message }));
}

fn disconnect(agent: &mut Agent, limits: &Limits, out: &mut Queue<Output>) {
    // Exit can precede the last buffered turn and answer on the read pipe.
    if agent.reading.ended() || agent.word.last_word() {
        report_unread(agent, out);
    }
    let count = agent.outbox.len();
    for _ in 0..count {
        let message = agent.outbox.pop().expect("bounded queue snapshot");
        match message {
            Down::Answer { call, .. } => {
                agent.calls.remove(&call).expect("already answered queued call");
            }
            Down::Acknowledge { turn } => {
                agent.turns.remove(&turn).expect("committed queued turn");
            }
            Down::Message { .. } => {
                agent.messages = agent.messages.checked_sub(1).expect("queued message count");
            }
            Down::Start { .. } => unreachable!("start never queued"),
            Down::Grant { .. } | Down::Cancel => {}
        }
    }
    let busy_count = agent.busy.len();
    for _ in 0..busy_count {
        agent.busy.pop().expect("bounded busy snapshot");
    }
    let mut withdrawn = skein_lib::List::with_capacity(limits.calls);
    for (callback, call) in &agent.calls {
        if call.stage == SendState::Parent && !call.withdrawn {
            withdrawn.push(*callback).expect("one entry per admitted call");
        }
    }
    for callback in withdrawn.into_boxed() {
        agent.calls.get_mut(&callback).expect("retained parent right").withdrawn = true;
        out.push(Output::Parent(parent::Request::Withdrawn { client: agent.client, call: callback }));
    }
}

fn follow(domain: &mut Domain, env: &Env<Limits>, id: Id<Agent>, out: &mut Queue<Output>) {
    let agent = domain.agents.get_mut(id).expect("entry lives until rights settle");
    let owner = id.token();
    let client = agent.client;
    let diagnostic = agent.diagnostic.take();
    if !listens(agent) && !agent.supervision.spawning() {
        disconnect(agent, &env.limits, out);
    }
    send_next(agent, owner, out);
    let may_read = match agent.supervision {
        State::Watching(_) | State::Cancelled { .. } | State::Draining { .. } => {
            turn_room(agent, &env.limits) && agent.busy.room() > 0
        }
        State::Exiting { .. } | State::Terminating { .. } | State::Killing => true,
        State::Spawning | State::Closed => false,
    };
    if may_read && !agent.reading.pending() && !agent.reading.ended() {
        agent.reading.demand();
        out.push(Output::Process(process::Request::Read { owner, process: agent.tree.process() }));
    }
    let working = agent.supervision.watch().is_some();
    let paused = working && pause(agent, env);
    agent.supervision.pause(paused, env.now);
    let (watch, wall) = match agent.supervision.watch() {
        Some(watch) => {
            let deadline = if paused {
                let mut deadline = None;
                for (_, call) in &agent.calls {
                    if call.stage == SendState::Parent && call.deadline > env.now {
                        deadline = Some(match deadline {
                            Some(at) => call.deadline.min(at),
                            None => call.deadline,
                        });
                    }
                }
                deadline
            } else {
                Some(agent.long.stretch(watch.progress).saturating_add(env.limits.no_progress))
            };
            (deadline, Some(watch.wall))
        }
        None => (None, None),
    };
    let grace = agent.supervision.deadline();
    set(&mut domain.alarms, Alarm::Watch { agent: id }, watch);
    set(&mut domain.alarms, Alarm::Wall { agent: id }, wall);
    set(&mut domain.alarms, Alarm::Grace { agent: id }, grace);
    let done = agent.supervision.closed()
        || (agent.tree.reaped()
            && agent.reading.ended()
            && agent.sending.is_none()
            && agent.signals == 0
            && agent.calls.is_empty()
            && agent.turns.is_empty());
    if done {
        if !agent.supervision.closed() {
            if let Some(fault) = agent.supervision.take_fault() {
                report_fault(agent, fault, out);
            }
            tree::finish(agent, out);
        }
        agent.supervision = State::Closed;
        for alarm in [Alarm::Watch { agent: id }, Alarm::Wall { agent: id }, Alarm::Grace { agent: id }] {
            domain.alarms.cancel(alarm);
        }
        domain.agents.retire(id);
    }
    if let Some(fault) = diagnostic {
        observation(domain, Fact::Faulted { client, fault });
    }
}

fn set(alarms: &mut Deadlines<Alarm>, alarm: Alarm, deadline: Option<Time>) {
    if let Some(at) = deadline {
        alarms.arm(alarm, at).expect("three bounded alarms per agent");
    } else {
        alarms.cancel(alarm);
    }
}

fn observation(domain: &mut Domain, fact: Fact) {
    domain.facts.push(fact);
}

fn observations(domain: &mut Domain, out: &Queue<Output>, before: u32) {
    let mut at = 0;
    for request in out {
        if at >= before {
            let fact = match request {
                Output::Parent(parent::Request::Started { client, .. }) => Some(Fact::Started { client: *client }),
                Output::Parent(parent::Request::Admitted { client }) => Some(Fact::Admitted { client: *client }),
                Output::Parent(parent::Request::Answered { client, .. }) => Some(Fact::Answered { client: *client }),
                Output::Parent(parent::Request::Faulted { client, fault }) => {
                    Some(Fact::Faulted { client: *client, fault: *fault })
                }
                Output::Parent(parent::Request::Gone { client, end, .. }) => {
                    Some(Fact::Gone { client: *client, end: *end })
                }
                Output::Parent(
                    parent::Request::Called { .. }
                    | parent::Request::Withdrawn { .. }
                    | parent::Request::Turn { .. }
                    | parent::Request::Waiting { .. }
                    | parent::Request::Long { .. }
                    | parent::Request::LongDone { .. }
                    | parent::Request::Unauthorized { .. }
                    | parent::Request::Refused { .. }
                    | parent::Request::Limited { .. }
                    | parent::Request::Told { .. }
                    | parent::Request::MessageRefused { .. }
                    | parent::Request::Unread { .. },
                )
                | Output::Process(
                    process::Request::Spawn { .. }
                    | process::Request::Send { .. }
                    | process::Request::Read { .. }
                    | process::Request::Signal { .. }
                    | process::Request::Wait { .. }
                    | process::Request::Reap { .. },
                ) => None,
            };
            if let Some(fact) = fact {
                observation(domain, fact);
            }
        }
        at = at.checked_add(1).expect("bounded output queue");
    }
}
