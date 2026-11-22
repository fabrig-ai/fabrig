//! Whole worker stories: smith's process boundary, the protocol layer's
//! hand-outs played with smith's codecs, the real checkout, fake disk and
//! forge. The world plays jig's worker link as jig's worker worlds do: a
//! channel ready or lost, the grace passed. The referee reads remote trees
//! and records crossing boundaries.
use smith_channel::Record;
use std::collections::VecDeque;
use std::fmt::Write;

use skein_lib::{Duration, Env, Queue, Reader, Time, Token, Wall};
use temper_fake_checkout::{
    Checkout,
    git::{Remote, Tree},
};
use temper_worker_checkout_world::{forge::Forge, translate};
use temper_worker_domain::{
    Domain, Event, Limits, Request, agent, checkout::git::Op, fire, host, resume, step, valid, wire,
};

use crate::protocol;

pub mod close;
pub mod faults;
pub mod jobs;
pub mod merge;
pub mod orders;
pub mod pause;
pub mod prepare;
pub mod restarts;

const RUN: Token = Token::new(81);
const ATTEMPT: Token = Token::new(13);
const WORKSTREAM: u64 = 81;
const PROCESS: Token = Token::new(707);
const CALL: Token = Token::new(44);
/// The scripted channel's `host_reply_text`, small enough to exercise both sides.
pub const REPLY_BYTES: u32 = 32;

const REMOTE: &[u8] = b"org/app";
const PATH: &[u8] = b"code";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Delivered,
    CancelledDelivery,
    ContactWithin,
    ContactPast,
    Drained,
    DrainExpired,
    LinkDrained,
    LinkDrainExpired,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub tree: Tree,
    pub trace: Vec<String>,
    pub notes: Vec<Request>,
    pub cancellations: u32,
    pub drains: u32,
    pub drops: u32,
    pub io_cuts: u32,
    pub saves: u32,
    pub stop_bound: Duration,
    pub pauses: Vec<jig_domain_link::PauseState>,
}

struct World {
    run: Token,
    seed: u64,
    iteration: u64,
    builder: crate::builder::Builder,
    faults: faults::Faults,
    attempt: Token,
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
    owed: VecDeque<Event>,
    forge: Forge,
    disk: Checkout,
    first: u64,
    workspace: Option<Token>,
    process: Option<Token>,
    reading: bool,
    /// The channel to the engine is open: what goes up reaches it.
    up: bool,
    hold_push: bool,
    restart_observer: Option<restarts::Observer>,
    held: Option<(Token, Op)>,
    deliveries: u32,
    delivery_reply: Option<agent::Delivery>,
    call: Option<Box<[u8]>>,
    replies: Vec<smith_channel::Reply>,
    unsent: u32,
    answer: Option<(host::RunAnswer, wire::Work, Option<wire::Preparation>)>,
    /// The workstreams the last hello named.
    workstreams: Box<[u64]>,
    stops: u32,
    root_pauses: Vec<jig_domain_link::PauseState>,
    shutdown_notices: u32,
    busy: Vec<(Token, Token)>,
    report: Report,
}

impl World {
    fn new(seed: u64) -> Self {
        Self::with_slots(seed, 1)
    }

    fn with_slots(seed: u64, slots: u32) -> Self {
        let mut limits = crate::limits();
        limits.host.slots = slots;
        limits.checkout.workspaces = slots;
        limits.checkout.repositories = 1;
        limits.agent.agents = slots;
        limits.agent.no_progress = Duration::from_secs(120);
        assert!(valid(&limits));
        let mut forge = Forge::new(seed);
        let first = forge.repository(REMOTE, b"main", [(PATH.to_vec(), b"original\n".to_vec())].into());
        forge.create_branch(REMOTE, b"topic", first).expect("topic");
        forge.create_branch(REMOTE, b"saved", first).expect("saved");
        Self {
            run: RUN,
            seed,
            iteration: 0,
            builder: crate::builder::Builder::new(1),
            faults: faults::Faults::default(),
            attempt: ATTEMPT,
            domain: Domain::new(&limits, Box::new([])),
            out: Queue::uncapped(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            owed: VecDeque::new(),
            forge,
            disk: Checkout::new(),
            first,
            workspace: None,
            process: None,
            reading: false,
            up: false,
            hold_push: false,
            restart_observer: None,
            held: None,
            deliveries: 0,
            delivery_reply: None,
            call: None,
            replies: Vec::new(),
            unsent: 0,
            answer: None,
            workstreams: Box::new([]),
            stops: 0,
            root_pauses: Vec::new(),
            shutdown_notices: 0,
            busy: Vec::new(),
            report: Report {
                tree: Tree::new(),
                trace: Vec::new(),
                notes: Vec::new(),
                cancellations: 0,
                drains: 0,
                drops: 0,
                io_cuts: 0,
                saves: 0,
                stop_bound: Duration::ZERO,
                pauses: Vec::new(),
            },
        }
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        self.report.trace.push(format!("{:?} -> {event:?}", self.env.now));
        step(&mut self.domain, &self.env, event, &mut self.out);
        self.settle()?;
        Ok(())
    }

    fn settle(&mut self) -> Result<(), String> {
        for _ in 0..256 {
            while let Some(request) = self.out.pop() {
                self.report.trace.push(format!("{:?} <- {request:?}", self.env.now));
                self.take(request);
            }
            self.domain.reclaim();
            if self.domain.is_due(self.env.now) {
                fire(&mut self.domain, &self.env, &mut self.out);
                continue;
            }
            if self.domain.is_ready() {
                resume(&mut self.domain, &self.env, &mut self.out);
                continue;
            }
            let Some(event) = self.owed.pop_front() else { return self.check() };
            self.report.trace.push(format!("{:?} -> {event:?}", self.env.now));
            step(&mut self.domain, &self.env, event, &mut self.out);
        }
        Err("bounded script failed to settle".into())
    }

    fn check(&mut self) -> Result<(), String> {
        crate::inspection::check(
            self.seed,
            &self.env,
            &mut self.iteration,
            &self.domain,
            Some(&self.disk),
            &self.builder,
        )
    }

    fn say(&mut self, message: agent::Up) -> Result<(), String> {
        assert!(self.reading, "one terminal for the granted read");
        self.reading = false;
        let owner = self.process.expect("spawned");
        self.event(Event::Process(Box::new(agent::process::Event::Received { owner, message })))?;
        Ok(())
    }

    fn take(&mut self, request: Request) {
        match request {
            Request::Paused { state } => self.root_pauses.push(state),
            Request::RedsAsk { owner, check, ask: wire::Ask::Reds } => {
                self.owed.push_back(Event::RedsAsk { owner, check, bytes: b"reds".as_slice().into() });
            }
            Request::RedsAsk { .. } => panic!("this script only asks for inherited reds"),
            Request::ReadUsage { .. }
            | Request::ReadList { .. }
            | Request::CancelRead { .. }
            | Request::Source { .. }
            | Request::RedsAnswer { .. }
            | Request::SourceFault => {
                panic!("this fixture declares no program sources: {request:?}")
            }
            Request::Hello(hello) => {
                if let Some(observer) = &mut self.restart_observer {
                    observer.hello(self.env.now, hello.hosting.len());
                }
                assert!(hello.hosting.len() <= 1);
                self.report.stop_bound = hello.stop_bound;
                self.workstreams = hello.workstreams;
            }
            Request::Link(up) => self.link(up),
            Request::Answer { run, attempt, answer, work, preparation } => {
                if matches!(answer.ending, host::Ending::Refused(host::Refusal::Busy)) {
                    self.busy.push((run, attempt));
                    return;
                }
                assert_eq!((run, attempt), (self.run, self.attempt));
                assert!(self.held.is_none(), "delivery settles before the final answer");
                if self.up {
                    assert!(self.answer.replace((answer, work, preparation)).is_none());
                }
            }
            Request::Abandon => panic!("unexpected stop bound"),
            Request::Stop => self.stops += 1,
            Request::Process(request) => self.process(*request),
            Request::Decode { agent, run, start } => {
                let start = protocol::decode(run, *start, &self.env.limits.agent);
                self.owed.push_back(Event::Decoded { agent, start, endpoints: Box::new([]) });
            }
            Request::Encode { owner, value } => {
                self.owed.push_back(Event::Encoded { owner, encoded: protocol::encode(value) });
            }
            Request::Fact { owner, body } => {
                self.owed.push_back(Event::Activity { owner, activity: protocol::activity(&body) });
            }
            Request::Io { owner, op, deadline } => self.io(owner, op, deadline),
            Request::CancelIo { owner } => {
                let (held, _) = self.held.take().expect("a cut names the held save push");
                assert_eq!(owner, held);
                self.report.io_cuts += 1;
                self.owed.push_back(Event::Done {
                    owner,
                    done: temper_worker_domain::checkout::git::Done::Failed {
                        fault: temper_worker_domain::checkout::git::Fault::Cancelled,
                    },
                });
            }
            note @ Request::Git { .. } => self.report.notes.push(note),
            Request::ResultsIgnored { .. }
            | Request::AgentSignalled { .. }
            | Request::AgentStopped { .. }
            | Request::Unprepared { .. } => {}
            Request::JobAnswer { .. } | Request::Job { .. } | Request::CancelRelay { .. } => {
                panic!("unscripted request")
            }
        }
    }

    fn link(&mut self, up: host::LinkUp) {
        match up {
            host::LinkUp::Paused { state } => self.report.pauses.push(state),
            host::LinkUp::Draining => self.shutdown_notices += 1,
            host::LinkUp::Turn { run, attempt, turn } => {
                assert_eq!((run, attempt, turn.turn), (self.run, self.attempt, 1));
                if let Some(observer) = &mut self.restart_observer {
                    observer.turned(attempt, &turn);
                }
            }
            host::LinkUp::Activity { run, attempt, .. } => assert_eq!((run, attempt), (self.run, self.attempt)),
            host::LinkUp::Call { run, attempt, call, tool, writes, input, .. } => {
                assert_eq!((run, attempt), (self.run, self.attempt));
                assert_eq!(tool.as_ref(), b"recall");
                assert!(!writes);
                assert_eq!(input.as_ref(), b"{}");
                assert!(self.call.replace(call).is_none());
            }
            // The scripted engine has no inherited reds for these delivery checks.
            host::LinkUp::Ask { run, attempt, number, bytes } => {
                assert_eq!(bytes.as_ref(), b"reds");
                self.owed.push_back(Event::Link(host::LinkDown::Asked {
                    run,
                    attempt,
                    number,
                    answer: host::AskAnswer::Refused,
                }));
            }
            host::LinkUp::UsageRead { .. }
            | host::LinkUp::ListRead { .. }
            | host::LinkUp::Logins { .. }
            | host::LinkUp::SignedOut { .. }
            | host::LinkUp::Piece { .. }
            | host::LinkUp::Unauthorized { .. }
            | host::LinkUp::Usage { .. }
            | host::LinkUp::Limited { .. }
            | host::LinkUp::AccountRefused { .. }
            | host::LinkUp::JobAnswer { .. }
            | host::LinkUp::Answer { .. } => panic!("unscripted link record: {up:?}"),
        }
    }

    fn io(&mut self, owner: Token, op: Op, deadline: Time) {
        assert!(deadline > self.env.now);
        // Includes delivery, save and catch-up, not just prepare (README.md, 4.3).
        let note = match &op {
            Op::Fetch { at, want, into, .. } => {
                Some((at, temper_worker_domain::Git::Fetch { want: want.clone(), into: *into }))
            }
            Op::CheckOut { at, commit, .. } => Some((at, temper_worker_domain::Git::CheckOut { commit: *commit })),
            _ => None,
        };
        if let Some((at, what)) = note {
            assert_eq!(
                self.report.notes.last(),
                Some(&Request::Git {
                    run: self.run,
                    attempt: self.attempt,
                    job: None,
                    repository: at.repository.clone(),
                    what,
                }),
                "the note precedes every fetch and check out"
            );
        }
        if let Op::Make { workspace, .. } = &op {
            self.workspace = Some(*workspace);
        }
        if let Op::Commit { title, body, .. } = &op {
            if title.as_ref() == b"Save unfinished work" {
                self.report.saves += 1;
            } else if title.as_ref() == b"Unchecked: work in progress, saved before a question" {
                assert!(body.is_empty());
            } else {
                assert_eq!(title.as_ref(), b"Delivery");
                assert_eq!(body.as_ref(), b"Worker story");
            }
        }
        if self.hold_push && matches!(op, Op::Push { .. }) {
            assert!(self.held.replace((owner, op)).is_none());
        } else if let Some(done) =
            self.faults.perform(&mut self.forge, &mut self.disk, owner, op, self.env.now, deadline)
        {
            self.owed.push_back(Event::Done { owner, done });
        }
    }

    fn process(&mut self, request: agent::process::Request) {
        match request {
            agent::process::Request::Spawn { owner, deadline, .. } => {
                assert!(deadline > self.env.now);
                assert!(self.process.replace(owner).is_none());
                let spawned = agent::process::Event::Spawned { owner, process: PROCESS };
                self.owed.push_back(Event::Process(Box::new(spawned)));
            }
            agent::process::Request::Send { owner, process, message } => {
                assert_eq!((Some(owner), process), (self.process, PROCESS));
                let sent = self.down(message);
                let event = if sent {
                    agent::process::Event::Sent { owner }
                } else {
                    self.unsent += 1;
                    agent::process::Event::Unsent { owner }
                };
                self.owed.push_back(Event::Process(Box::new(event)));
            }
            agent::process::Request::Read { owner, process } => {
                assert_eq!((Some(owner), process), (self.process, PROCESS));
                assert!(!self.reading);
                self.reading = true;
            }
            agent::process::Request::Wait { owner, process } | agent::process::Request::Reap { owner, process } => {
                assert_eq!((Some(owner), process), (self.process, PROCESS));
            }
            agent::process::Request::CloseChannel { .. } => {}
            agent::process::Request::Signal { owner, signal, .. } => {
                if signal == agent::Signal::Terminate {
                    self.report.drops += 1;
                }
                self.owed.push_back(Event::Process(Box::new(agent::process::Event::Signalled { owner })));
            }
        }
    }

    fn down(&mut self, message: agent::Down) -> bool {
        match message {
            agent::Down::Start { start, .. } => {
                if let Some(observer) = &mut self.restart_observer {
                    observer.started(self.attempt);
                }
                assert_eq!(start.activation, self.attempt.raw());
                assert_eq!(start.directories.len(), 1);
                assert_eq!(start.directories[0].name.as_ref(), b"app");
                assert_eq!(start.workspace, Some(start.directories[0].root));
            }
            agent::Down::Answer { call, reply } => {
                assert_eq!(call, CALL);
                let agent::Reply::Delivery(delivery) = reply else {
                    let Some(reply) = host_reply(reply) else { return false };
                    self.replies.push(reply);
                    return true;
                };
                self.delivery_reply = Some(delivery.clone());
                let agent::Delivery::Delivered(receipts) = delivery else { return true };
                assert_eq!(receipts.receipts().len(), 1);
                assert_eq!(receipts.receipts()[0].directory(), 0);
                let landed = self.forge.branch(REMOTE, b"topic").expect("delivered branch");
                let commit = translate::commit(landed).raw();
                let width = if commit[20..] == [0; 12] { 20 } else { 32 };
                let mut encoded = String::with_capacity(width * 2);
                for byte in &commit[..width] {
                    write!(&mut encoded, "{byte:02x}").expect("String write");
                }
                let expected = if self.forge.message(landed).starts_with(b"Unchecked:") {
                    format!(
                        "saved {encoded} to branch topic, unchecked: no check ran; your checkout stays detached, as expected"
                    )
                } else {
                    format!("pushed {encoded} to branch topic; your checkout stays detached, as expected")
                };
                assert_eq!(
                    receipts.receipts()[0].text(),
                    expected.as_bytes(),
                    "the receipt identifies the pushed head"
                );
                self.deliveries += 1;
            }
            agent::Down::Cancel => self.report.cancellations += 1,
            agent::Down::Drain => self.report.drains += 1,
            agent::Down::Renewals { .. } | agent::Down::Acknowledge { turn: 1 } => {}
            agent::Down::Acknowledge { .. } | agent::Down::Grant { .. } | agent::Down::Message { .. } => {
                panic!("unscripted process record")
            }
        }
        true
    }

    fn connect(&mut self) -> Result<(), String> {
        self.up = true;
        self.event(Event::Ready)?;
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        self.start_from(wire::Start::Base { branch: Box::from(&b"main"[..]) })?;
        Ok(())
    }

    fn start_from(&mut self, start: wire::Start) -> Result<(), String> {
        self.connect()?;
        self.assign(start)?;
        self.say(agent::Up::Admitted)?;
        let place = temper_worker_domain::checkout::git::Place {
            git: None,
            workspace: self.workspace.expect("prepared"),
            repository: Box::from(&b"app"[..]),
        };
        self.disk.write(&[translate::path(&place).as_slice(), b"/code"].concat(), b"edited\n");
        Ok(())
    }

    /// The engine assigns the run, its one repository starting at `start`.
    fn assign(&mut self, start: wire::Start) -> Result<(), String> {
        self.assign_with(start, true)?;
        Ok(())
    }

    fn assign_with(&mut self, start: wire::Start, save: bool) -> Result<(), String> {
        let Event::Assign { mut assignment, items } = assignment(self.first, self.run, self.run.raw(), start) else {
            panic!("workspace assignment")
        };
        assignment.assignment.attempt = self.attempt;
        assignment.assignment.save = save;
        self.event(Event::Assign { assignment, items })?;
        Ok(())
    }

    fn deliver(&mut self) -> Result<(), String> {
        self.request_delivery(protocol::delivery(b"Delivery", b"Worker story"))?;
        if let Some(reply) = &self.delivery_reply {
            assert!(matches!(reply, agent::Delivery::Delivered(_)), "the complete edited tree is delivered: {reply:?}");
        }
        Ok(())
    }

    fn request_delivery(&mut self, ask: agent::Ask) -> Result<(), String> {
        self.delivery_reply = None;
        self.say(agent::Up::Call {
            call: CALL,
            name: agent::CallName { activation: self.attempt.raw(), completion: self.deliveries + 1, position: 0 },
            deadline: self.env.now.saturating_add(Duration::from_secs(60)),
            ask,
        })?;
        Ok(())
    }

    /// A change finish asks the host to deliver before it can answer accepted.
    fn finish_change(&mut self) -> Result<agent::Delivery, String> {
        let ask = protocol::delivery(b"Delivery", b"Worker story");
        let agent::Ask::Deliver { fields } = &ask else { panic!("change delivery") };
        let change = fields.clone().into_value();
        self.request_delivery(ask)?;
        let reply = self.delivery_reply.take().expect("finishing delivery answered");
        match &reply {
            agent::Delivery::Delivered(_) => {
                let declared = smith_domain_run::outcome::Declared::Change(change);
                let outcome = agent::Declared::new(declared, u64::MAX).expect("bounded change result");
                self.say(agent::Up::Answer {
                    answer: agent::Answer {
                        read: None,
                        turns: 0,
                        spent: 0,
                        result: agent::RunResult::Accepted { outcome },
                    },
                })?;
            }
            agent::Delivery::Nothing => {} // smith refuses the finish and keeps the run live.
            reply => panic!("unexpected finishing delivery: {reply:?}"),
        }
        Ok(reply)
    }

    fn park(&mut self) -> Result<(), String> {
        self.say(agent::Up::Answer {
            answer: agent::Answer { read: None, turns: 0, spent: 0, result: agent::RunResult::Parked },
        })?;
        self.gone()?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        let answer = agent::Answer { read: None, turns: 0, spent: 0, result: protocol::report(b"done") };
        self.say(agent::Up::Answer { answer })?;
        Ok(())
    }

    fn gone(&mut self) -> Result<(), String> {
        let owner = self.process.expect("spawned");
        let exited = agent::process::Event::Exited { owner, exit: Some(agent::Exit::Code(0)) };
        self.event(Event::Process(Box::new(exited)))?;
        assert!(self.reading);
        self.reading = false;
        self.event(Event::Process(Box::new(agent::process::Event::Hangup { owner })))?;
        let reaped = agent::process::Event::Reaped { owner, detail: Box::new([]) };
        self.event(Event::Process(Box::new(reaped)))?;
        Ok(())
    }

    fn story(&mut self, case: Case) -> Result<(), String> {
        match case {
            Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired => {
                if matches!(case, Case::LinkDrained | Case::LinkDrainExpired) {
                    self.env.now = self.env.now.saturating_add(Duration::from_secs(7));
                    self.event(Event::Link(host::LinkDown::Pause { mode: jig_domain_link::PauseMode::Drain }))?;
                } else {
                    self.event(Event::Shutdown { drain: true })?;
                }
                assert_eq!(self.report.drains, 1);
                assert_eq!(self.report.cancellations, 0, "the current turn goes on");
                self.env.now = self.env.now.saturating_add(Duration::from_nanos(self.env.limits.drain.as_nanos() - 1));
                self.settle()?;
                assert_eq!(self.report.cancellations, 0, "not before the drain bound");
                if matches!(case, Case::DrainExpired | Case::LinkDrainExpired) {
                    self.env.now = self.env.now.saturating_add(Duration::from_nanos(1));
                    self.settle()?;
                    assert_eq!(self.report.cancellations, 1, "the bound stops a turn still running");
                } else {
                    self.say(agent::Up::Turn { turn: protocol::turn(1, 0, 8) })?;
                }
                let answer = agent::Answer {
                    read: None,
                    turns: u32::from(matches!(case, Case::Drained | Case::LinkDrained)),
                    spent: 0,
                    result: agent::RunResult::Failed { failure: agent::RunFailure::Cancelled },
                };
                self.say(agent::Up::Answer { answer })?;
                self.gone()?;
                assert_eq!(self.stops, 0, "the engine has not acknowledged the answer");
                assert_eq!(self.report.saves, 1);
            }
            Case::Delivered => {
                self.deliver()?;
                assert_eq!(self.deliveries, 1);
                self.finish()?;
                self.gone()?;
            }
            Case::CancelledDelivery => {
                self.hold_push = true;
                self.deliver()?;
                assert!(self.held.is_some());
                assert_eq!(self.forge.branch(REMOTE, b"topic"), Some(self.first));
                self.event(Event::Link(host::LinkDown::Cancel { run: self.run, attempt: self.attempt }))?;
                assert_eq!(self.report.cancellations, 1);
                self.gone()?;
                assert!(self.answer.is_none());
                self.hold_push = false;
                let (owner, op) = self.held.take().expect("push held");
                let done = translate::perform(&mut self.forge, &mut self.disk, op);
                self.event(Event::Done { owner, done })?;
            }
            Case::ContactWithin => {
                self.up = false;
                self.event(Event::Lost)?;
                self.env.now = self.env.now.saturating_add(Duration::from_secs(1));
                assert_eq!(self.report.cancellations, 0);
                self.connect()?;
                self.deliver()?;
                self.finish()?;
                self.gone()?;
            }
            Case::ContactPast => {
                self.up = false;
                self.event(Event::Lost)?;
                self.env.now = self.env.now.saturating_add(Duration::from_secs(60));
                self.event(Event::GracePassed)?;
                assert_eq!(self.report.cancellations, 1);
                self.gone()?;
                assert_eq!(self.report.saves, 1, "save before the cancellation answer reaches the engine");
                assert!(self.answer.is_none());
                self.connect()?;
            }
        }
        Ok(())
    }

    fn run(mut self, case: Case) -> Result<Report, String> {
        self.start()?;
        self.story(case)?;
        self.completed(case)?;
        Ok(self.report)
    }

    fn completed(&mut self, case: Case) -> Result<(), String> {
        assert_eq!(self.domain.agent().agents(), 0);
        assert_eq!(self.domain.workspaces(), 0);
        let (answer, work, preparation) = self.answer.take().expect("a terminal answer");
        assert_eq!(preparation, None);
        let branch = if matches!(
            case,
            Case::ContactPast | Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired
        ) {
            b"saved".as_slice()
        } else {
            b"topic".as_slice()
        };
        let commit = self.forge.branch(REMOTE, branch).expect("worker pushed");
        let expected = match case {
            Case::Delivered | Case::ContactWithin => None,
            Case::CancelledDelivery => Some(host::Reason::Engine),
            Case::ContactPast => Some(host::Reason::Contact),
            Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired => {
                Some(host::Reason::Shutdown)
            }
        };
        let cancelled = match answer.ending {
            host::Ending::Ended { .. } => None,
            host::Ending::Failed { failure: host::Failure::Cancelled(reason), .. } => Some(reason),
            ending @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }) => {
                panic!("ending did not follow the story: {ending:?}")
            }
        };
        assert_eq!(cancelled, expected);
        if matches!(
            case,
            Case::ContactPast | Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired
        ) {
            let saved = work.saved.expect("saved work");
            let [wire::SavedRepository { landing: wire::Landing::Landed { commit: saved }, .. }] = saved.as_ref()
            else {
                panic!("one saved head");
            };
            assert_eq!(*saved, translate::commit(commit).raw());
        } else {
            assert_eq!(
                work.landed.as_ref(),
                [wire::Landed { tag: 1, commit: translate::commit(commit).raw(), checked: true }]
            );
        }
        self.report.tree = self.forge.tree(commit);
        assert_eq!(self.report.tree, [(PATH.to_vec(), b"edited\n".to_vec())].into(), "the entire edited tree landed");
        assert!(self.forge.is_ancestor(self.first, commit));
        self.event(Event::Link(host::LinkDown::Acknowledge { run: self.run, attempt: self.attempt }))?;
        assert_eq!(self.domain.kept_answers(), 0);
        if matches!(case, Case::Drained | Case::LinkDrained) {
            assert_eq!(self.stops, 0, "its last turn must be kept too");
            self.event(Event::Link(host::LinkDown::TurnKept { run: self.run, attempt: self.attempt, turn: 1 }))?;
        }
        if matches!(case, Case::LinkDrained | Case::LinkDrainExpired) {
            assert_eq!(
                self.report.pauses,
                [jig_domain_link::PauseState::Draining, jig_domain_link::PauseState::Drained]
            );
            assert_eq!(self.root_pauses, self.report.pauses, "each change is said both ways");
            assert_eq!(self.stops, 0, "a link drain never stops the worker");
            assert_eq!(self.shutdown_notices, 0, "a link drain is not shutdown");
            assert_eq!(self.domain.abandoned(), 0);
            assert!(self.up, "the worker stays linked");
            self.env.now = self.env.now.saturating_add(self.env.limits.stop_bound);
            self.settle()?;
            assert_eq!(self.stops, 0, "drained stays up past a stop's bound");
        }
        if !matches!(case, Case::Drained | Case::DrainExpired) {
            self.event(Event::Shutdown { drain: true })?;
        }
        assert_eq!(self.stops, 1, "a worker with nothing left stops, once");
        Ok(())
    }
}

pub fn run(seed: u64, case: Case) -> Result<Report, String> {
    World::new(seed).run(case)
}

/// A run starting at `start`, whose unfinished edit is saved after contact
/// is lost past the grace: its work, with each repository's tag and start.
pub fn saved(seed: u64, start: wire::Start) -> Result<wire::Work, String> {
    let mut world = World::new(seed);
    // A merge start admits the fixture's one possible conflicting path.
    world.env.limits.checkout.conflicts = 1;
    world.env.limits.checkout.path_bytes = u32::try_from(PATH.len()).expect("fixture path length");
    world.start_from(start)?;
    world.story(Case::ContactPast)?;
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    let (answer, work, preparation) = world.answer.take().expect("a terminal answer");
    assert_eq!(preparation, None);
    assert!(matches!(
        answer.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Contact), .. }
    ));
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 1);
    Ok(work)
}

/// A run whose repository starts from a branch the forge does not have: how
/// its answer ends, and the preparation's detail beside it.
pub fn unprepared(seed: u64) -> Result<(host::Ending, Option<wire::Preparation>), String> {
    let mut world = World::new(seed);
    world.connect()?;
    world.assign(wire::Start::Branch { branch: Box::from(&b"gone"[..]) })?;
    let (answer, _, preparation) = world.answer.take().expect("an answer");
    Ok((answer.ending, preparation))
}

/// The workstreams a worker's hello names once a run delivered and went.
pub fn workstreams(seed: u64) -> Result<Box<[u64]>, String> {
    let mut world = World::new(seed);
    world.start()?;
    world.story(Case::Delivered)?;
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    world.up = false;
    world.event(Event::Lost)?;
    world.connect()?;
    Ok(world.workstreams)
}

/// A host call crossing the engine link, its reply decoded by the agent,
/// followed by a run answer reaching the engine and complete teardown.
pub fn host_answer(seed: u64, bytes: u32, error: bool) -> Result<smith_channel::Reply, String> {
    let mut world = World::new(seed);
    world.start()?;
    world.say(agent::Up::Call {
        call: CALL,
        name: agent::CallName { activation: ATTEMPT.raw(), completion: 1, position: 0 },
        deadline: world.env.now.saturating_add(Duration::from_secs(60)),
        ask: agent::Ask::Host {
            tool: Box::from(&b"recall"[..]),
            effect: agent::Effect::Read,
            body: Box::from(&b"{}"[..]),
        },
    })?;
    let call = world.call.take().expect("call reaches engine");
    world.event(Event::Link(host::LinkDown::Relayed {
        run: RUN,
        attempt: ATTEMPT,
        call,
        answer: host::Relayed::Answer { error, text: vec![b'x'; usize::try_from(bytes).expect("reply length")].into() },
    }))?;
    assert_eq!(world.unsent, 0, "host codec must send the reply instead of draining the run");
    assert_eq!(world.replies.len(), 1);
    finish_host_run(&mut world)?;
    Ok(world.replies.pop().expect("decoded reply"))
}

fn finish_host_run(world: &mut World) -> Result<(), String> {
    world.finish()?;
    world.gone()?;
    let (answer, _, preparation) = world.answer.take().expect("run answer reaches engine");
    assert!(matches!(answer.ending, host::Ending::Ended { .. }), "{answer:?}");
    assert_eq!(preparation, None);
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: world.run, attempt: world.attempt }))?;
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 1);
    Ok(())
}

fn channel_limits() -> smith_channel::Limits {
    let mut limits = smith_channel::CEILINGS;
    limits.host_reply_text = REPLY_BYTES;
    limits
}

/// smith's host codec at the process boundary, and the agent's decode.
fn host_reply(reply: agent::Reply) -> Option<smith_channel::Reply> {
    let limits = channel_limits();
    let reply = match reply {
        agent::Reply::Host { error, body } => {
            smith_channel::Reply::Host(smith_channel::HostReply { error, text: body })
        }
        agent::Reply::TooLarge => smith_channel::Reply::TooLarge,
        other @ (agent::Reply::Busy
        | agent::Reply::Unavailable
        | agent::Reply::Withdrawn
        | agent::Reply::Unsent
        | agent::Reply::Delivery(_)) => {
            panic!("unscripted reply: {other:?}")
        }
    };
    let name = smith_channel::CallName { activation: ATTEMPT.raw(), completion: 1, position: 0 };
    let record = smith_channel::HostAnswer { name, reply };
    let bytes = record.encode(&limits).ok()?;
    let mut reader = Reader::new(&bytes);
    let decoded = smith_channel::HostAnswer::decode(&limits, &mut reader).expect("agent decodes");
    assert!(reader.is_empty());
    Some(decoded.reply.clone())
}

fn assignment(first: u64, run: Token, workstream: u64, start: wire::Start) -> Event {
    let repository = wire::Repository {
        save: Some(Box::from(&b"saved"[..])),
        git: None,
        tag: 1,
        name: Box::from(&b"app"[..]),
        remote: Box::from(REMOTE),
        identity: 0,
        account: None,
        start,
        access: wire::Access::Writable {
            push: Box::from(&b"topic"[..]),
            expected: Some(translate::commit(first).raw()),
            landing: None,
            delivered: None,
            pin: None,
        },
    };
    let assignment = host::Assignment {
        messages: Box::new([]),
        turns: Box::new([]),
        answered: Box::new([]),
        assignment: host::RunAssignment {
            identity: [9; 16],
            live: host::LiveAsk::NONE,
            run,
            attempt: ATTEMPT,
            workspace: Some(host::Workspace { workstream, items: Token::new(0) }),
            save: true,
            charter: protocol::charter(),
            grants: Box::new([]),
        },
    };
    let items = wire::Workspace { repositories: Box::new([repository]) };
    Event::Assign { assignment, items: Ok(items) }
}

/// An unfinished first attempt writes an untracked file, exits, and a new
/// attempt either restores its saved commit or starts fresh. Dirt left in
/// the cached directory between attempts is never part of the delivered tree.
pub fn retry(seed: u64, resume: bool) -> Result<Report, String> {
    let mut world = World::new(seed);
    world.connect()?;
    world.assign_with(wire::Start::Base { branch: b"main".as_slice().into() }, resume)?;
    world.say(agent::Up::Admitted)?;
    let at = temper_worker_domain::checkout::git::Place {
        git: None,
        workspace: world.workspace.expect("prepared"),
        repository: b"app".as_slice().into(),
    };
    let path = translate::path(&at);
    world.disk.write(&[path.as_slice(), b"/changes/3.md"].concat(), b"repair kept\n");
    world.gone()?;
    let (_, work, _) = world.answer.take().expect("unfinished answer after save");
    let saved = work.saved.is_some();
    assert_eq!(saved, resume);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    world.disk.write(&[path.as_slice(), b"/unrelated"].concat(), b"stale local file");
    world.disk.write(&[path.as_slice(), b"/code"].concat(), b"stale local edit");
    world.attempt = Token::new(world.attempt.raw() + 1);
    world.process = None;
    let start = if saved {
        wire::Start::Saved { branch: b"saved".as_slice().into() }
    } else {
        wire::Start::Base { branch: b"main".as_slice().into() }
    };
    world.assign_with(start, resume)?;
    world.say(agent::Up::Admitted)?;
    let mut restored = world.disk.tree(&path);
    restored.retain(|path, _| !temper_fake_checkout::in_git(path));
    if saved {
        world.deliver()?;
        world.deliver()?; // A finish after delivery still receives the same landing.
        world.finish()?;
    } else {
        // There is no change to deliver. The fresh agent reports its clean tree.
        world.finish()?;
    }
    world.gone()?;
    let (_, work, _) = world.answer.take().expect("retry answer");
    let head = if saved {
        let [wire::Landed { commit, .. }] = work.landed.as_ref() else { panic!("restored file was delivered") };
        translate::fake(temper_worker_domain::checkout::git::Commit::new(*commit))
    } else {
        world.first
    };
    world.report.tree = if saved { world.forge.tree(head) } else { restored };
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.kept_answers(), 0);
    assert_eq!(world.stops, 1);
    Ok(world.report)
}

/// The engine assigns one commit twice to one worker, with no saved work
/// from the first attempt (domain/worker/README.md, 4.3).
pub fn reassigned_start(seed: u64) -> Result<(), String> {
    let mut world = World::new(seed);
    world.connect()?;
    for attempt in [1, 2] {
        world.attempt = Token::new(attempt);
        world.process = None;
        let start = wire::Start::Commit { commit: translate::commit(world.first).raw() };
        let Event::Assign { mut assignment, items: Ok(mut items) } = assignment(world.first, RUN, WORKSTREAM, start)
        else {
            panic!("workspace assignment")
        };
        assignment.assignment.attempt = world.attempt;
        assignment.assignment.save = false;
        for repository in &mut items.repositories {
            repository.access = wire::Access::ReadOnly;
        }
        world.event(Event::Assign { assignment, items: Ok(items) })?;
        world.say(agent::Up::Admitted)?;
        world.gone()?;
        let (_, work, _) = world.answer.take().expect("attempt ended");
        assert!(work.saved.is_none());
        world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    }
    let commit = translate::commit(world.first);
    let expected: Vec<_> = [1, 2]
        .into_iter()
        .flat_map(|attempt| {
            [
                Request::Git {
                    run: RUN,
                    attempt: Token::new(attempt),
                    job: None,
                    repository: b"app".as_slice().into(),
                    what: temper_worker_domain::Git::Fetch {
                        want: temper_worker_domain::checkout::git::Want::Commit { commit },
                        into: temper_worker_domain::checkout::git::Into::Fetched,
                    },
                },
                Request::Git {
                    run: RUN,
                    attempt: Token::new(attempt),
                    job: None,
                    repository: b"app".as_slice().into(),
                    what: temper_worker_domain::Git::CheckOut { commit },
                },
            ]
        })
        .collect();
    assert_eq!(world.report.notes, expected, "each attempt notes its fetch and check out");
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.stops, 1);
    Ok(())
}

/// A repair delivers, parks for a question, then finishes on the same branch
/// without edits. Only the next item's evidence makes that clean finish delivered.
pub fn delivered_across_park(seed: u64, names_delivery: bool) -> Result<(), String> {
    let mut world = World::new(seed);
    world.start_from(wire::Start::Branch { branch: b"topic".as_slice().into() })?;
    world.deliver()?;
    assert_eq!(world.deliveries, 1, "the repair received its delivery receipt");
    let head = world.forge.branch(REMOTE, b"topic").expect("repair delivered");
    assert_ne!(head, world.first, "the first run pushed its edit");
    let commit = translate::commit(head).raw();
    let landed = [wire::Landed { tag: 1, commit, checked: true }];
    assert_eq!(world.forge.tree(head), [(PATH.to_vec(), b"edited\n".to_vec())].into());

    // The question's engine wire is outside this story; parking ends its hold.
    world.park()?;
    let (answer, work, preparation) = world.answer.take().expect("parked repair answer");
    assert!(matches!(answer.ending, host::Ending::Parked { .. }));
    assert_eq!(work.landed.as_ref(), landed);
    assert_eq!(preparation, None);
    assert_eq!(world.domain.workspaces(), 0, "the first hold ended");
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    world.attempt = Token::new(world.attempt.raw() + 1);
    world.process = None;

    // The answer assigns the same task afresh, naming its last checked push.
    let Event::Assign { mut assignment, items } =
        assignment(head, RUN, WORKSTREAM, wire::Start::Branch { branch: b"topic".as_slice().into() })
    else {
        panic!("repair assignment")
    };
    assignment.assignment.attempt = world.attempt;
    let mut items = items.expect("repair repository");
    let [repository] = items.repositories.as_mut() else { panic!("one repository") };
    let wire::Access::Writable { delivered, landing, .. } = &mut repository.access else { panic!("writable repair") };
    assert_eq!(*landing, None, "a repair has no landing branch");
    *delivered = names_delivery.then_some(commit);
    world.event(Event::Assign { assignment, items: Ok(items) })?;
    world.say(agent::Up::Admitted)?;

    let reply = world.finish_change()?;
    if names_delivery {
        assert!(matches!(reply, agent::Delivery::Delivered(_)), "the finishing delivery must be delivered: {reply:?}");
        world.gone()?;
    } else {
        assert_eq!(reply, agent::Delivery::Nothing, "a clean tree is not the task's own delivery");
        assert!(world.answer.is_none(), "a nothing refusal leaves the agent running");
        world.park()?;
    }
    let (answer, work, preparation) = world.answer.take().expect("second repair answer");
    if names_delivery {
        assert!(matches!(answer.ending, host::Ending::Ended { .. }), "the change finish ended the run");
        assert_eq!(work.landed.as_ref(), landed, "the finish reports the first run's head landed");
        assert_eq!(world.deliveries, 2, "both runs received the receipt for the same head");
    } else {
        assert!(matches!(answer.ending, host::Ending::Parked { .. }));
        assert!(work.landed.is_empty(), "nothing was delivered by the second run");
        assert_eq!(world.deliveries, 1);
    }
    assert_eq!(preparation, None);
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(head), "no new commit was pushed");
    assert_eq!(world.forge.branch(REMOTE, b"main"), Some(world.first), "the landing branch did not move");
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.kept_answers(), 0);
    assert_eq!(world.stops, 1);
    Ok(())
}

/// A save crosses the worker's host face and is replaced by a checked equal-tree push.
pub fn unchecked_save_then_delivery(seed: u64) -> Result<(), String> {
    let mut world = World::new(seed);
    world.start()?;
    world.say(agent::Up::Call {
        call: CALL,
        name: agent::CallName { activation: ATTEMPT.raw(), completion: 1, position: 0 },
        deadline: world.env.now.saturating_add(Duration::from_secs(60)),
        ask: agent::Ask::Save,
    })?;
    assert_eq!(world.deliveries, 1, "save is answered with its unchecked receipt");
    assert!(world.answer.is_none(), "save is no result");
    let saved = world.forge.branch(REMOTE, b"topic").expect("save moved the producer branch");
    assert_eq!(world.forge.branch(REMOTE, b"main"), Some(world.first), "save leaves the goal branch");
    assert_eq!(world.forge.message(saved), b"Unchecked: work in progress, saved before a question");
    world.deliver()?;
    let checked = world.forge.branch(REMOTE, b"topic").expect("checked push");
    assert_ne!(checked, saved, "an equal tree gets the checked delivery's commit");
    assert_eq!(world.forge.tree(saved), world.forge.tree(checked));
    assert_eq!(world.forge.parent(checked), Some(saved));
    assert_eq!(world.forge.message(checked), b"Delivery\n\nWorker story");
    assert_eq!(world.deliveries, 2);
    world.finish()?;
    world.gone()?;
    assert!(world.answer.is_some());
    assert_eq!(world.domain.workspaces(), 0);
    Ok(())
}

/// The contact bound cuts a hung save, or an agent still stopping. No later
/// end can resume writing, even when a new channel opens before that end.
pub fn contact_bound(seed: u64, saving: bool) -> Result<Report, String> {
    let mut world = World::new(seed);
    world.env.limits.agent.cancel_grace = Duration::from_secs(90);
    world.start()?;
    world.up = false;
    world.event(Event::Lost)?;
    world.event(Event::GracePassed)?;
    assert_eq!(world.report.cancellations, 1);
    world.hold_push = true;
    if saving {
        world.gone()?;
        assert!(world.held.is_some(), "the saved-work push has no terminal yet");
    }
    let at = world.env.limits.after_grace.as_nanos();
    world.env.now = Time::from_nanos(at - 1);
    world.settle()?;
    assert_eq!((world.report.io_cuts, world.report.drops), (0, 0));
    world.env.now = Time::from_nanos(at);
    world.settle()?;
    if saving {
        assert_eq!(world.report.io_cuts, 1, "the bound cuts even a saved-work push");
    } else {
        assert_eq!(world.report.drops, 1, "the root drops an agent still stopping");
        world.connect()?;
        world.gone()?;
        assert_eq!(world.report.saves, 0, "no new commit after the bound");
    }
    if !world.up {
        world.connect()?;
    }
    let (answer, work, _) = world.answer.take().expect("the cut run is answered");
    assert!(matches!(
        answer.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Contact), .. }
    ));
    let saved = work.saved.expect("the save's failed outcome");
    assert!(!matches!(saved[0].landing, wire::Landing::Landed { .. }));
    assert_eq!(world.forge.branch(REMOTE, b"saved"), Some(world.first));
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 1);
    Ok(world.report)
}
