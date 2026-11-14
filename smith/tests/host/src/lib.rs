//! Host world: scripted parent, agent channel and contained tree, driven by
//! skein's Stage, Schedule, Ledger and Trace (testing-strategy.md, sections 2.2,
//! 6 and 7; domain/host.md, section 10). Observations, never domain cells, prove
//! exact terminals, parent-owned turns and no Gone before tree/EOF/rights.
pub mod inline;

use skein_lib::{Duration, Time, Token};
use skein_world::domain::{Ledger, Schedule, Stage, Trace};
use smith_host_domain::{self as host, Answer, Down, End, Fault, Grant, Input, Limits, Output, Start, Up};
use smith_host_domain::{parent, process};
use std::collections::{BTreeMap, BTreeSet};

/// Distinct lower rights retained even after the process exits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lower {
    /// Process admission terminal.
    Spawn,
    /// Serialized channel write terminal.
    Send,
    /// One demand-driven channel read terminal.
    Read,
    /// Process-exited proof.
    Wait,
    /// Empty-tree proof; distinct from process exit.
    Reap,
    /// First polite tree signal terminal.
    Terminate,
    /// Final tree signal terminal.
    Kill,
}

/// Boundary-only state of one physical agent; no access to domain internals.
#[derive(Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent tree, channel and parent observations are outside the domain state machine"
)]
pub struct Seen {
    /// Parent handle, available only after Started.
    pub agent: Option<Token>,
    /// Lower owner, available to the tree before Started.
    pub owner: Option<Token>,
    /// Agent admission record, independently from Started.
    pub admitted: bool,
    /// One accepted last word.
    pub answer: Option<Answer>,
    /// One failure while the run was untold.
    pub fault: Option<Fault>,
    pub gone: Option<End>,
    /// Bounded operator suffix retained from the Gone terminal.
    pub gone_detail: Option<Box<[u8]>>,
    pub exited: bool,
    pub empty: bool,
    pub eof: bool,
    /// Calls whose operation terminal still belongs to the parent.
    pub calls: BTreeSet<Token>,
    /// Turn payload bytes owned by the parent until its exact ACK.
    pub turns: BTreeMap<u32, host::TurnValue>,
    /// Last read fence observed in a parent Turn or Waiting notice.
    pub read: Option<Token>,
    /// Ingress attempts still pending admission, read or unread.
    pub messages: Vec<Token>,
    /// Names issued to the agent before any covering fence.
    pub sent_messages: BTreeSet<Token>,
    pub read_messages: Vec<Token>,
    /// One unread report, including the empty report.
    pub unread: Option<Vec<Token>>,
    /// Every refused ingress attempt or issued name.
    pub refused_messages: Vec<(Token, host::MessageRefusal)>,
    /// Parent Turn metadata: sequence, scalar prefix and independent
    /// currency/raw overflow attestations. ACK never erases these observations.
    pub turn_metadata: Vec<(u32, u64)>,
    /// Parent notifications after withdrawal, which never consume a call right.
    pub withdrawals: BTreeSet<Token>,
    pub down: Vec<Down>,
    pub account_notices: Vec<host::Up>,
    pub told: u32,
    /// One-way process signals, in order.
    pub signals: Vec<host::Signal>,
}

/// Real host domain and generic harness machinery, with a single scripted tree.
/// Every input goes through the same ordered step/fire path; every lower
/// terminal consumes its independent ledger before the domain sees it.
#[derive(Debug)]
pub struct World {
    /// Deterministic seed is recorded in the trace.
    pub seed: u64,
    pub domain: host::Domain,
    /// Bounded stage, injected clock and handed outputs.
    pub stage: Stage<Limits, Input, Output>,
    pub schedule: Schedule<Input>,
    /// Independent exactly-once lower terminal accounting.
    pub lower: Ledger<Lower, Token>,
    pub seen: Seen,
    pub trace: Trace,
    finding: Option<String>,
}

/// Small valid capacities; every sealed delivery fits the reply cap.
#[must_use]
pub fn limits() -> Limits {
    smith_host_domain_fixtures::small()
}

/// Opaque source cells, optional workspace and credential names only.
#[must_use]
pub fn start() -> Start {
    Start {
        live: None,
        messages: Box::default(),
        logical_run: Token::new(7),
        identity: None,
        activation: 1,
        workspace: None,
        charter: charter_value(7),
        transcript: Some(transcript_value(512)),
        answered: Box::new([smith_host_domain::AnsweredCall {
            name: smith_host_domain::CallName { activation: 1, completion: 1, position: 0 },
            tool: Box::from(&b"tool"[..]),
            reply: smith_host_domain::SavedReply::Host {
                error: false,
                body: Box::from(&b"post-transcript answers"[..]),
            },
        }]),
        directories: Box::new([]),
        grants: Box::new([Grant {
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
            form: smith_host_domain::Form::Credential(smith_host_domain::Scheme::Bearer),
        }]),
    }
}

impl World {
    /// Construct only the domain and bounded stage; no generic allocator here.
    #[must_use]
    pub fn new(seed: u64, bounds: Limits) -> Self {
        let mut trace = Trace::default();
        trace.log(Time::ZERO, format!("seed {seed}"));
        Self {
            seed,
            domain: host::Domain::new(&bounds),
            stage: Stage::uncapped(bounds),
            schedule: Schedule::new(),
            lower: Ledger::new("host lower operation"),
            seen: Seen::default(),
            trace,
            finding: None,
        }
    }

    /// Deliver one typed input, with lower accounting and ordered outputs.
    pub fn event(&mut self, event: Input) {
        match &event {
            Input::Parent(parent::Event::Spawn { start, .. }) => {
                self.seen.messages.extend(start.messages.iter().map(|message| message.name));
            }
            Input::Parent(parent::Event::Message { agent, name, .. })
                if (self.seen.agent == Some(*agent) || self.seen.owner == Some(*agent)) && self.seen.gone.is_none() =>
            {
                self.seen.messages.push(*name);
            }
            Input::Parent(_) | Input::Process(_) => {}
        }
        self.observe_terminal(&event);
        self.trace.log(self.stage.env.now, format!("up {event:?}"));
        self.stage.push(event);
        while let Some(event) = self.stage.next_event() {
            host::step(&mut self.domain, &self.stage.env, event, &mut self.stage.out);
            self.outputs();
            if self.finding.is_some() {
                break;
            }
        }
        self.watch().unwrap_or_else(|why| panic!("{why}"));
        self.domain.reclaim();
    }

    /// Advance the injected clock; schedule deliveries precede same-time timers.
    pub fn at(&mut self, seconds: u64) {
        let now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        assert!(now >= self.stage.env.now);
        self.stage.tick(now);
        while let Some(event) = self.schedule.next(now) {
            self.event(event);
        }
        while self.domain.is_due(now) {
            host::fire(&mut self.domain, &self.stage.env, &mut self.stage.out);
            self.outputs();
            if self.finding.is_some() {
                break;
            }
        }
        self.watch().unwrap_or_else(|why| panic!("{why}"));
        self.domain.reclaim();
    }

    /// Parent spawns the source run; tree owner exists before parent handle.
    pub fn spawn(&mut self, start: Start) {
        self.event(Input::Parent(parent::Event::Spawn { client: Token::new(1), start }));
    }

    /// Process Started, distinct from agent admission.
    pub fn spawned(&mut self) {
        let owner = self.owner();
        self.event(Input::Process(process::Event::Spawned { owner, process: Token::new(9) }));
    }

    /// Completion of the currently issued channel send.
    pub fn sent(&mut self) {
        self.event(Input::Process(process::Event::Sent { owner: self.owner() }));
    }

    /// Channel message on one demanded read.
    pub fn up(&mut self, message: Up) {
        self.event(Input::Process(process::Event::Received { owner: self.owner(), message }));
    }

    /// Start, finish its Send terminal and admit through the read path.
    pub fn live(&mut self) {
        self.spawn(start());
        self.spawned();
        self.sent();
        self.up(Up::Admitted);
    }

    /// Opaque parent handle only after Started.
    #[must_use]
    pub fn agent(&self) -> Token {
        self.seen.agent.expect("parent learned Started")
    }

    /// Lower owner learned from Spawn request.
    #[must_use]
    pub fn owner(&self) -> Token {
        self.seen.owner.expect("tree learned Spawn")
    }

    /// Settle a clean process exit and assert that every terminal was handed on.
    pub fn clean_end(&mut self) {
        self.cleanup(Some(host::Exit::Code(0)));
        self.settled();
    }

    /// Settle physical exit, empty tree and EOF in that order. Pending sends,
    /// signals, calls and turns deliberately remain owed to their owners.
    pub fn cleanup(&mut self, exit: Option<smith_host_domain::Exit>) {
        if self.lower.contains(Lower::Send) {
            self.sent();
        }
        let owner = self.owner();
        if self.lower.contains(Lower::Wait) {
            self.event(Input::Process(process::Event::Exited { owner, exit }));
        }
        if self.lower.contains(Lower::Reap) {
            self.event(Input::Process(process::Event::Reaped {
                owner,
                detail: Box::from(&b"bounded operator tail"[..]),
            }));
        }
        if self.lower.contains(Lower::Read) {
            self.event(Input::Process(process::Event::Hangup { owner }));
        }
        for signal in [Lower::Terminate, Lower::Kill] {
            if self.lower.contains(signal) {
                self.event(Input::Process(process::Event::Signalled { owner }));
            }
        }
    }

    /// Return the first parent referee finding with this world's trace.
    ///
    /// # Errors
    /// Returns the first refused observation, naming the seed and ordered trace.
    pub fn watch(&self) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(format!("{why}\n{}", self.trace.lines().join("\n")));
        }
        Ok(())
    }

    /// Final independent quiescence checks (testing-strategy.md, section 6).
    pub fn settled(&self) {
        self.watch().unwrap_or_else(|why| panic!("{why}"));
        self.lower.assert_settled();
        assert!(self.seen.calls.is_empty(), "every parent call right returned");
        assert!(self.seen.turns.is_empty(), "every parent-owned turn committed");
        assert!(self.seen.gone.is_some());
        assert_eq!(self.domain.agents(), 0);
        assert_eq!(self.domain.next_deadline(), None);
        assert!(self.schedule.is_empty());
    }

    fn observe_terminal(&mut self, event: &Input) {
        let key = match event {
            Input::Process(process::Event::Spawned { .. } | process::Event::Unspawned { .. }) => Some(Lower::Spawn),
            Input::Process(process::Event::Sent { .. } | process::Event::Unsent { .. }) => Some(Lower::Send),
            Input::Process(
                process::Event::Received { .. }
                | process::Event::Progress { .. }
                | process::Event::Malformed { .. }
                | process::Event::Hangup { .. },
            ) => Some(Lower::Read),
            Input::Process(process::Event::Exited { .. }) => {
                self.seen.exited = true;
                Some(Lower::Wait)
            }
            Input::Process(process::Event::Reaped { .. }) => {
                assert!(self.seen.exited);
                self.seen.empty = true;
                Some(Lower::Reap)
            }
            Input::Process(process::Event::Signalled { .. }) => {
                if self.lower.contains(Lower::Terminate) {
                    Some(Lower::Terminate)
                } else {
                    Some(Lower::Kill)
                }
            }
            Input::Parent(parent::Event::Answer { call, .. }) => {
                assert!(self.seen.calls.remove(call), "parent operation right");
                None
            }
            Input::Parent(parent::Event::Acknowledge { turn, .. }) => {
                self.seen.turns.remove(turn);
                None
            }
            Input::Parent(parent::Event::Drop { .. }) => {
                self.seen.turns.clear();
                None
            }
            Input::Parent(
                parent::Event::Spawn { .. }
                | parent::Event::Message { .. }
                | parent::Event::Grant { .. }
                | parent::Event::Drain { .. }
                | parent::Event::Stop { .. },
            ) => None,
        };
        if let Some(key) = key {
            self.lower.end(key);
        }
        match event {
            Input::Process(process::Event::Hangup { .. } | process::Event::Malformed { .. }) => self.seen.eof = true,
            Input::Parent(
                parent::Event::Spawn { .. }
                | parent::Event::Message { .. }
                | parent::Event::Answer { .. }
                | parent::Event::Acknowledge { .. }
                | parent::Event::Grant { .. }
                | parent::Event::Drain { .. }
                | parent::Event::Stop { .. }
                | parent::Event::Drop { .. },
            )
            | Input::Process(
                process::Event::Spawned { .. }
                | process::Event::Unspawned { .. }
                | process::Event::Sent { .. }
                | process::Event::Unsent { .. }
                | process::Event::Received { .. }
                | process::Event::Progress { .. }
                | process::Event::Signalled { .. }
                | process::Event::Exited { .. }
                | process::Event::Reaped { .. },
            ) => {}
        }
    }

    fn outputs(&mut self) {
        if self.finding.is_none() {
            self.finding = self.outputs_checked().err();
        }
    }

    fn outputs_checked(&mut self) -> Result<(), String> {
        while let Some(request) = self.stage.out.pop() {
            self.trace.log(self.stage.env.now, format!("down {request:?}"));
            match request {
                Output::Process(process::Request::Spawn { owner, .. }) => {
                    self.seen.owner = Some(owner);
                    self.lower.open(Lower::Spawn, owner);
                }
                Output::Parent(request) => self.seen.observe_parent(
                    request,
                    self.seen.exited && self.seen.empty && self.seen.eof,
                    self.lower.is_empty(),
                )?,
                Output::Process(process::Request::Send { owner, message, .. }) => {
                    self.lower.open(Lower::Send, owner);
                    match &message {
                        Down::Start { start, .. } => {
                            self.seen.sent_messages.extend(start.messages.iter().map(|message| message.name));
                        }
                        Down::Message { name, .. } => {
                            self.seen.sent_messages.insert(*name);
                        }
                        Down::Answer { .. }
                        | Down::Acknowledge { .. }
                        | Down::Grant { .. }
                        | Down::Cancel
                        | Down::Drain => {}
                    }
                    self.seen.down.push(message);
                }
                Output::Process(process::Request::Read { owner, .. }) => self.lower.open(Lower::Read, owner),
                Output::Process(process::Request::Wait { owner, .. }) => self.lower.open(Lower::Wait, owner),
                Output::Process(process::Request::Reap { owner, .. }) => self.lower.open(Lower::Reap, owner),
                Output::Process(process::Request::Signal { owner, signal, .. }) => {
                    self.seen.signals.push(signal);
                    let key = match signal {
                        host::Signal::Terminate => Lower::Terminate,
                        host::Signal::Kill => Lower::Kill,
                    };
                    self.lower.open(key, owner);
                }
            }
        }
        Ok(())
    }
}

/// Typed policy with exactly the requested instruction ownership.
#[must_use]
pub fn charter_value(bytes: u64) -> host::Charter {
    use smith_domain::run;
    let value = run::Charter {
        check_timeout: None,
        mode: run::Mode::Finite,
        reserve: run::charter::Reserve { turns: 0, time: skein_lib::Duration::ZERO },
        resume: false,
        waiting: Duration::ZERO,
        instructions: vec![b'x'; usize::try_from(bytes).expect("fixture size")].into_boxed_slice(),
        brief: run::charter::Brief { sections: Box::new([]) },
        conventions: None,
        grants: run::charter::Grants {
            contract: 1,
            wait: false,
            deliver: None,
            save: false,
            tools: run::charter::Tools { inspect: false, modify: false, shell: false },
            agents: false,
            host_tools: Box::new([]),
        },
        outcome: run::outcome::OutcomeSpec { change: None, verdicts: Box::new([]), report: None, failure: None },
        budget: run::Budget { turns: 0, spend: 0, time: Duration::ZERO },
        llm: run::charter::Llm {
            effort: None,
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 0 },
            dialect: 0,
            account: Some(0),
            endpoint: run::charter::Endpoint(0),
            model: Box::new([]),
            window: 0,
            output: 0,
        },
        models: Box::new([]),
    };
    host::Charter::new(value, bytes).expect("bounded charter fixture")
}

/// One host turn with no spend or read fence; the first turn opens its window.
#[must_use]
pub fn turn(number: u32, body: host::TurnValue) -> host::Turn {
    host::Turn { number, opens_window: number == 1, spent: 0, read: None, body }
}

/// Concrete turn padded to the requested ownership when one message fits.
#[must_use]
pub fn turn_value(bytes: u64) -> host::TurnValue {
    use smith_domain::session::{llm, record};
    let opener = u64::try_from(core::mem::size_of::<record::Opener>()).expect("opener footprint");
    let kind = if bytes >= opener {
        record::Kind::First(Box::new(record::Opener {
            system: None,
            affinity: llm::Affinity { key: [0; 16], thread: 0 },
            account: None,
            backend: record::Backend::Direct,
        }))
    } else {
        record::Kind::Ordinary
    };
    let overhead = u64::try_from(core::mem::size_of::<llm::Message>() + core::mem::size_of::<llm::Block>())
        .expect("fixture footprint")
        + opener;
    let messages = if bytes >= overhead {
        Box::from([llm::Message {
            role: llm::Role::Assistant,
            content: Box::from([llm::Block::Text {
                text: vec![b't'; usize::try_from(bytes - overhead).expect("fixture size")].into_boxed_slice(),
                replay: None,
            }]),
        }])
    } else {
        Box::<[llm::Message]>::default()
    };
    host::TurnValue::new(smith_domain_session_fixtures::turn(Box::new([]), kind, messages), bytes)
        .expect("bounded turn fixture")
}

/// One saved concrete turn with exactly the requested retained ownership.
#[must_use]
pub fn transcript_value(bytes: u64) -> host::Transcript {
    use smith_domain::session::{llm, record};
    let turn_bytes = u64::try_from(core::mem::size_of::<record::Turn>()).expect("turn footprint");
    let turns = if bytes >= turn_bytes {
        Box::from([turn_value(bytes - turn_bytes).into_value()])
    } else {
        Box::<[record::Turn]>::default()
    };
    host::Transcript::new(
        record::Transcript { version: record::VERSION, endpoint: Some(llm::Endpoint(0)), dialect: 0, turns },
        bytes,
    )
    .expect("bounded history fixture")
}

/// Generic delivery fields padded to the requested checked ownership.
#[must_use]
pub fn fields_value(bytes: u64) -> host::Fields {
    use smith_domain::run::outcome::{Change, Field};
    let overhead = u64::try_from(core::mem::size_of::<Field>()).expect("field footprint");
    let fields = if bytes >= overhead {
        Box::from([Field {
            name: Box::new([]),
            value: vec![b'f'; usize::try_from(bytes - overhead).expect("fixture size")].into_boxed_slice(),
        }])
    } else {
        Box::<[Field]>::default()
    };
    host::Fields::new(Change { fields }, bytes).expect("bounded fields fixture")
}

/// An accepted report owning exactly the requested text bytes.
#[must_use]
pub fn declared_value(bytes: u64) -> host::Declared {
    use smith_domain::run::outcome::{Declared, Report};
    host::Declared::new(
        Declared::Report(Report {
            text: vec![b'r'; usize::try_from(bytes).expect("fixture size")].into_boxed_slice(),
            fields: Box::new([]),
        }),
        bytes,
    )
    .expect("bounded outcome fixture")
}

impl Seen {
    fn fence(&mut self, read: Option<Token>) -> Result<(), String> {
        if read == self.read {
            return Ok(());
        }
        let name = read.ok_or("fences never go backwards")?;
        let at = self.messages.iter().position(|old| *old == name).ok_or("fence owns an accepted message")?;
        if self.messages.iter().take(at + 1).any(|name| !self.sent_messages.contains(name)) {
            return Err("a covering fence follows message issuance".into());
        }
        for name in self.messages.drain(..=at) {
            self.sent_messages.remove(&name);
            self.read_messages.push(name);
        }
        self.read = read;
        Ok(())
    }

    /// Observe the parent's answer to one relayed inline call.
    ///
    /// # Errors
    /// Returns a finding without consuming a right when the call is not pending.
    pub fn observe_answer(&mut self, call: Token) -> Result<(), String> {
        if !self.calls.contains(&call) {
            return Err("one parent terminal".into());
        }
        self.calls.remove(&call);
        Ok(())
    }

    /// Common parent referee; the kind supplies lower containment proof.
    ///
    /// # Errors
    /// Returns the first broken contract without recording the refused observation.
    #[expect(clippy::too_many_lines, reason = "the common parent referee checks each observation before recording it")]
    pub fn observe_parent(
        &mut self,
        request: parent::Request,
        contained: bool,
        lower_settled: bool,
    ) -> Result<(), String> {
        match request {
            parent::Request::Started { agent, .. } => {
                if self.agent.is_some() {
                    return Err("one parent handle".into());
                }
                self.agent = Some(agent);
            }
            parent::Request::Admitted { .. } => {
                if self.admitted {
                    return Err("one admission".into());
                }
                self.admitted = true;
            }
            parent::Request::Called { call, .. } => {
                if self.calls.contains(&call) {
                    return Err("one parent call".into());
                }
                self.calls.insert(call);
            }
            parent::Request::Withdrawn { call, .. } => {
                if !self.calls.contains(&call) {
                    return Err("withdrawal retains its parent call".into());
                }
                if self.withdrawals.contains(&call) {
                    return Err("one parent withdrawal".into());
                }
                self.withdrawals.insert(call);
            }
            parent::Request::Turn { turn, .. } => {
                if self.turns.contains_key(&turn.number) {
                    return Err("one parent turn".into());
                }
                self.fence(turn.read)?;
                self.turn_metadata.push((turn.number, turn.spent));
                self.turns.insert(turn.number, turn.body);
            }
            parent::Request::Answered { answer, .. } => {
                if self.fault.is_some() {
                    return Err("answer has no failure".into());
                }
                if self.answer.is_some() {
                    return Err("one run answer".into());
                }
                self.fence(answer.read)?;
                self.answer = Some(answer);
            }
            parent::Request::Faulted { fault, .. } => {
                if self.answer.is_some() {
                    return Err("failure has no answer".into());
                }
                if self.unread.is_none() {
                    return Err("unread is reported before a failure".into());
                }
                if self.fault.is_some() {
                    return Err("one failure".into());
                }
                self.fault = Some(fault);
            }
            parent::Request::Gone { end, detail, .. } => {
                if matches!(end, End::Exited { .. } | End::Settled) {
                    if !contained {
                        return Err("Gone needs the kind's containment proof".into());
                    }
                    if !lower_settled {
                        return Err("Gone retains every lower terminal".into());
                    }
                    if !self.calls.is_empty() {
                        return Err("Gone retains every parent call".into());
                    }
                    if !self.turns.is_empty() {
                        return Err("Gone retains every told turn".into());
                    }
                }
                if self.agent.is_some() || self.owner.is_some() {
                    if self.unread.is_none() {
                        return Err("the unread report precedes Gone".into());
                    }
                    if !self.messages.is_empty() {
                        return Err("every accepted message ended before Gone".into());
                    }
                }
                if self.gone.is_some() || self.gone_detail.is_some() {
                    return Err("one Gone terminal".into());
                }
                if self.agent.is_none() && !matches!(end, End::Exited { .. } | End::Settled) {
                    self.messages.clear(); // Start refusal refuses its carried messages with it.
                }
                self.gone = Some(end);
                self.gone_detail = Some(detail);
            }
            parent::Request::Unread { names, .. } => {
                if self.fault.is_some() || self.gone.is_some() {
                    return Err("unread precedes an unanswered terminal".into());
                }
                if self.unread.is_some() {
                    return Err("one unread report".into());
                }
                if names.as_ref() != self.messages.as_slice() {
                    return Err(format!(
                        "unread names every remaining accepted message in order\nleft: {:?}\nright: {:?}",
                        names, self.messages
                    ));
                }
                self.messages.clear();
                self.sent_messages.clear();
                self.unread = Some(names.into_vec());
            }
            parent::Request::MessageRefused { name, reason, .. } => {
                let at = self.messages.iter().rposition(|old| *old == name).ok_or("one message terminal")?;
                self.messages.remove(at);
                if !self.messages.contains(&name) {
                    self.sent_messages.remove(&name);
                }
                self.refused_messages.push((name, reason));
            }
            parent::Request::Unauthorized { account, generation, .. } => {
                self.account_notices.push(host::Up::Unauthorized { account, generation });
            }
            parent::Request::Told { .. } => self.told += 1,
            parent::Request::Waiting { read, .. } => self.fence(read)?,
            parent::Request::Limited { account, reset, .. } => {
                self.account_notices.push(host::Up::Limited { account, reset });
            }
            parent::Request::Refused { account, code, reason, .. } => {
                self.account_notices.push(host::Up::Refused { account, code, reason });
            }
            parent::Request::Long { .. } | parent::Request::LongDone { .. } => {}
        }
        Ok(())
    }
}
