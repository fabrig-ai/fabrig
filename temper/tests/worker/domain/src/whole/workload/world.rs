//! The workload's process face: one domain and disk per worker, one shared forge.
use std::collections::VecDeque;

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_world::{
    faults::FaultLedger,
    inspect::{Moment, check_tables, check_waits},
    streams::{Stream, Streams},
};
use temper_fake_checkout::{Checkout, git::Remote};
use temper_faults::bounds;
use temper_worker_checkout_world::forge::Forge;
use temper_worker_domain::{Domain, Event, Limits, Request, agent, fire, host, resume, step, valid, wire};

use super::super::{CALL, Case, PATH, REMOTE, assignment, protocol, translate};
use super::Workload;

struct Worker {
    domain: Domain,
    disk: Checkout,
    out: Queue<Request>,
    owed: VecDeque<Event>,
    stopped: u32,
}

impl Worker {
    fn new(limits: &Limits) -> Self {
        Self {
            domain: Domain::new(limits, Box::new([])),
            disk: Checkout::new(),
            out: Queue::uncapped(),
            owed: VecDeque::new(),
            stopped: 0,
        }
    }
}

pub(super) struct Actor {
    pub run: Token,
    pub attempt: Token,
    worker: usize,
    process: Option<Token>,
    reading: bool,
    workspace: Option<Token>,
    push: Vec<u8>,
    save: Vec<u8>,
    pub edit: Vec<u8>,
    answer: Option<(host::RunAnswer, wire::Work, Option<wire::Preparation>)>,
    reply: Option<agent::Delivery>,
    cancellations: u32,
    drains: u32,
    receipts: u32,
}

pub(super) struct World {
    seed: u64,
    env: Env<Limits>,
    iteration: u64,
    workers: Vec<Worker>,
    forge: Forge,
    builder: crate::builder::Builder,
    holder: Option<usize>,
    hold_push: bool,
    held: Option<(Token, temper_worker_domain::checkout::git::Op)>,
    pub runs: Vec<Actor>,
    pub ledger: FaultLedger,
    pub first: u64,
}

impl World {
    pub fn new(seed: u64, workload: &Workload) -> Self {
        let limits = crate::limits();
        assert!(valid(&limits));
        let mut forge = Forge::new(Streams::new(seed).seed_of(Stream::Workload, "worker forge"));
        let first = forge.repository(REMOTE, b"main", [(PATH.to_vec(), b"original\n".to_vec())].into());
        let runs = workload
            .runs
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let push = format!("topic-{index}").into_bytes();
                let save = format!("saved-{index}").into_bytes();
                forge.create_branch(REMOTE, &push, first).expect("producer branch");
                forge.create_branch(REMOTE, &save, first).expect("save branch");
                Actor {
                    run: Token::new(81 + u64::try_from(index).expect("three runs")),
                    attempt: Token::new(13),
                    worker: 0,
                    process: None,
                    reading: false,
                    workspace: None,
                    push,
                    save,
                    edit: format!("edited by run {index}\n").into_bytes(),
                    answer: None,
                    reply: None,
                    cancellations: 0,
                    drains: 0,
                    receipts: 0,
                }
            })
            .collect();
        Self {
            seed,
            workers: vec![Worker::new(&limits)],
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            iteration: 0,
            forge,
            builder: crate::builder::Builder::new(1),
            holder: None,
            hold_push: false,
            held: None,
            runs,
            ledger: FaultLedger::new(),
            first,
        }
    }

    pub fn now(&self) -> Time {
        self.env.now
    }

    pub fn tick(&mut self, tick: u64) -> Result<(), String> {
        self.env.now = Time::from_nanos(tick * 1_000_000);
        for worker in 0..self.workers.len() {
            self.settle(worker)?;
        }
        Ok(())
    }

    pub fn event(&mut self, worker: usize, event: Event) -> Result<(), String> {
        let w = &mut self.workers[worker];
        step(&mut w.domain, &self.env, event, &mut w.out);
        self.settle(worker)
    }

    fn settle(&mut self, worker: usize) -> Result<(), String> {
        for _ in 0..512 {
            while let Some(request) = self.workers[worker].out.pop() {
                self.take(worker, request);
            }
            let w = &mut self.workers[worker];
            w.domain.reclaim();
            if w.domain.is_due(self.env.now) {
                fire(&mut w.domain, &self.env, &mut w.out);
            } else if w.domain.is_ready() {
                resume(&mut w.domain, &self.env, &mut w.out);
            } else if let Some(event) = w.owed.pop_front() {
                step(&mut w.domain, &self.env, event, &mut w.out);
            } else {
                return self.inspect();
            }
        }
        Err(format!("seed {}: worker {worker} did not settle", self.seed))
    }

    fn inspect(&mut self) -> Result<(), String> {
        self.iteration += 1;
        let moment = Moment { seed: self.seed, at: self.now(), iteration: self.iteration };
        let mut workers = Vec::new();
        for w in &self.workers {
            let mut found = temper_worker_inspect::inspection();
            temper_worker_inspect::inspect(&w.domain, &self.env.limits, &w.disk, &mut found);
            workers.push(found);
        }
        let builder = self.builder.inspection();
        let mut inspections: Vec<_> = workers.iter().collect();
        inspections.push(&builder);
        check_waits(&inspections, moment)?;
        check_tables(&inspections, moment)?;
        self.ledger.saturation(
            &[
                bounds::WORKSPACES,
                bounds::PREPARE_PERMITS,
                bounds::PER_RUN_SCRATCH,
                bounds::RUN_SLOTS,
                bounds::BUILD_PLACES,
            ],
            &inspections,
            self.now(),
        )
    }

    fn by_run(&self, run: Token, attempt: Token) -> usize {
        self.runs.iter().position(|r| r.run == run && r.attempt == attempt).expect("known run and attempt")
    }

    fn by_process(&self, worker: usize, owner: Token) -> usize {
        self.runs.iter().position(|r| r.worker == worker && r.process == Some(owner)).expect("known process owner")
    }

    fn take(&mut self, worker: usize, request: Request) {
        let event = match request {
            Request::Hello(hello) => {
                assert!(hello.hosting.len() <= 3);
                None
            }
            Request::Answer { run, attempt, answer, work, preparation } => {
                let index = self.by_run(run, attempt);
                assert_eq!(self.runs[index].worker, worker);
                assert!(self.runs[index].answer.replace((answer, work, preparation)).is_none());
                None
            }
            Request::Link(up) => self.link(up),
            Request::Decode { agent, run, start } => Some(Event::Decoded {
                agent,
                start: protocol::decode(run, *start, &self.env.limits.agent),
                endpoints: Box::new([]),
            }),
            Request::Encode { owner, value } => Some(Event::Encoded { owner, encoded: protocol::encode(value) }),
            Request::Fact { owner, body } => Some(Event::Activity { owner, activity: protocol::activity(&body) }),
            Request::RedsAsk { owner, check, ask: wire::Ask::Reds } => {
                Some(Event::RedsAsk { owner, check, bytes: b"reds".as_slice().into() })
            }
            Request::Io { owner, op, deadline } => {
                assert!(deadline > self.now());
                if self.hold_push && matches!(op, temper_worker_domain::checkout::git::Op::Push { .. }) {
                    assert!(self.held.replace((owner, op)).is_none());
                    None
                } else {
                    self.forge.at(self.now());
                    let done = translate::perform(&mut self.forge, &mut self.workers[worker].disk, op);
                    Some(Event::Done { owner, done })
                }
            }
            Request::Process(request) => self.process(worker, *request),
            Request::Stop => {
                self.workers[worker].stopped += 1;
                None
            }
            Request::Git { .. }
            | Request::Paused { .. }
            | Request::ResultsIgnored { .. }
            | Request::AgentSignalled { .. }
            | Request::AgentStopped { .. }
            | Request::Unprepared { .. } => None,
            other => panic!("unscripted workload request: {other:?}"),
        };
        if let Some(event) = event {
            self.workers[worker].owed.push_back(event);
        }
    }

    fn link(&mut self, up: host::LinkUp) -> Option<Event> {
        match up {
            host::LinkUp::Ask { run, attempt, number, bytes } => {
                assert_eq!(bytes.as_ref(), b"reds");
                Some(Event::Link(host::LinkDown::Asked { run, attempt, number, answer: host::AskAnswer::Refused }))
            }
            host::LinkUp::Turn { run, attempt, turn } => {
                self.by_run(run, attempt);
                assert_eq!(turn.turn, 1);
                assert_eq!(turn.spent, 0);
                assert_eq!(turn.body, protocol::transcript_turn(2, 8, false));
                None
            }
            host::LinkUp::Activity { run, attempt, .. } => {
                self.by_run(run, attempt);
                None
            }
            host::LinkUp::Draining | host::LinkUp::Paused { .. } => None,
            other => panic!("unscripted workload link: {other:?}"),
        }
    }

    fn process(&mut self, worker: usize, request: agent::process::Request) -> Option<Event> {
        let event = match request {
            agent::process::Request::Spawn { owner, deadline, .. } => {
                assert!(deadline > self.now());
                // Prepares settle in assignment order; exactly the newly assigned run lacks a process.
                let run = self
                    .runs
                    .iter_mut()
                    .find(|r| r.worker == worker && r.workspace.is_some() && r.process.is_none())
                    .expect("assigned run");
                run.process = Some(owner);
                agent::process::Event::Spawned { owner, process: owner }
            }
            agent::process::Request::Read { owner, process } => {
                assert_eq!(owner, process);
                let index = self.by_process(worker, owner);
                assert!(!self.runs[index].reading);
                self.runs[index].reading = true;
                return None;
            }
            agent::process::Request::Send { owner, process, message } => {
                assert_eq!(owner, process);
                let index = self.by_process(worker, owner);
                self.down(index, message);
                agent::process::Event::Sent { owner }
            }
            agent::process::Request::Wait { .. }
            | agent::process::Request::Reap { .. }
            | agent::process::Request::CloseChannel { .. } => return None,
            agent::process::Request::Signal { owner, .. } => agent::process::Event::Signalled { owner },
        };
        Some(Event::Process(Box::new(event)))
    }

    fn down(&mut self, index: usize, message: agent::Down) {
        match message {
            agent::Down::Start { start, .. } => {
                assert_eq!(start.activation, self.runs[index].attempt.raw());
                assert_eq!(start.directories.len(), 1);
                assert_eq!(start.directories[0].name.as_ref(), b"app");
                self.runs[index].workspace = Some(start.directories[0].root);
            }
            agent::Down::Answer { call, reply: agent::Reply::Delivery(reply) } => {
                assert_eq!(call, CALL);
                let agent::Delivery::Delivered(receipts) = &reply else { panic!("delivery refused: {reply:?}") };
                assert_eq!(receipts.receipts().len(), 1);
                assert_eq!(receipts.receipts()[0].directory(), 0);
                self.runs[index].receipts += 1;
                self.runs[index].reply = Some(reply);
            }
            agent::Down::Cancel => self.runs[index].cancellations += 1,
            agent::Down::Drain { .. } => self.runs[index].drains += 1,
            agent::Down::Renewals { .. } | agent::Down::Acknowledge { turn: 1 } => {}
            other => panic!("unscripted workload process message: {other:?}"),
        }
    }

    pub fn assign(&mut self, index: usize, branch: bool, delivered: Option<u64>) -> Result<(), String> {
        let r = &self.runs[index];
        let start = if branch {
            wire::Start::Branch { branch: r.push.clone().into() }
        } else {
            wire::Start::Base { branch: b"main".as_slice().into() }
        };
        let first = if branch { self.head(index, false) } else { self.first };
        let Event::Assign { mut assignment, items } = assignment(first, r.run, r.run.raw(), start) else {
            unreachable!()
        };
        assignment.assignment.attempt = r.attempt;
        let mut items = items.expect("one repository");
        let repository = &mut items.repositories[0];
        repository.save = Some(r.save.clone().into());
        let wire::Access::Writable { push, delivered: old, .. } = &mut repository.access else { unreachable!() };
        *push = r.push.clone().into();
        *old = delivered.map(|head| translate::commit(head).raw());
        let worker = r.worker;
        // A marker only for routing the Spawn request; Start supplies the real token.
        self.runs[index].workspace = Some(Token::new(0));
        self.runs[index].process = None;
        self.event(worker, Event::Assign { assignment, items: Ok(items) })?;
        self.say(index, agent::Up::Admitted)
    }

    pub fn say(&mut self, index: usize, message: agent::Up) -> Result<(), String> {
        let r = &mut self.runs[index];
        assert!(r.reading, "one terminal for each read credit");
        r.reading = false;
        let owner = r.process.expect("spawned");
        let worker = r.worker;
        self.event(worker, Event::Process(Box::new(agent::process::Event::Received { owner, message })))
    }

    fn path(&self, index: usize) -> Vec<u8> {
        let place = temper_worker_domain::checkout::git::Place {
            git: None,
            workspace: self.runs[index].workspace.expect("prepared"),
            repository: b"app".as_slice().into(),
        };
        [translate::path(&place).as_slice(), b"/code"].concat()
    }

    pub fn edit(&mut self, index: usize) {
        let path = self.path(index);
        self.workers[self.runs[index].worker].disk.write(&path, &self.runs[index].edit);
    }

    pub fn checkout_tree(&self, index: usize) {
        assert_eq!(
            self.workers[self.runs[index].worker].disk.files().get(self.path(index).as_slice()).copied(),
            Some(self.runs[index].edit.as_slice()),
            "the fresh worker fetched the first attempt's saved work"
        );
    }

    pub fn head(&self, index: usize, saved: bool) -> u64 {
        self.forge.branch(REMOTE, if saved { &self.runs[index].save } else { &self.runs[index].push }).expect("branch")
    }

    pub fn tree(&self, index: usize, head: u64) {
        assert_eq!(
            self.forge.tree(head),
            [(PATH.to_vec(), self.runs[index].edit.clone())].into(),
            "entire run's tree, not another run's"
        );
        assert!(self.forge.is_ancestor(self.first, head));
    }

    pub fn call(&mut self, index: usize, ask: agent::Ask) -> Result<(), String> {
        let r = &self.runs[index];
        self.say(
            index,
            agent::Up::Call {
                call: CALL,
                name: agent::CallName { activation: r.attempt.raw(), completion: r.receipts + 1, position: 0 },
                deadline: self.now().saturating_add(Duration::from_secs(60)),
                ask,
            },
        )
    }

    pub fn finish_change(&mut self, index: usize) -> Result<(), String> {
        let ask = protocol::delivery(b"Delivery", b"Worker story");
        let agent::Ask::Deliver { fields } = &ask else { unreachable!() };
        let outcome =
            agent::Declared::new(smith_domain_run::outcome::Declared::Change(fields.clone().into_value()), u64::MAX)
                .expect("bounded change");
        self.call(index, ask)?;
        self.say(
            index,
            agent::Up::Answer {
                answer: agent::Answer {
                    read: None,
                    turns: 0,
                    spent: 0,
                    result: agent::RunResult::Accepted { outcome },
                },
            },
        )
    }

    pub fn finish_report(&mut self, index: usize) -> Result<(), String> {
        self.say(
            index,
            agent::Up::Answer {
                answer: agent::Answer { read: None, turns: 0, spent: 0, result: protocol::report(b"done") },
            },
        )
    }

    pub fn gone(&mut self, index: usize) -> Result<(), String> {
        let r = &self.runs[index];
        let owner = r.process.expect("spawned");
        let worker = r.worker;
        self.event(
            worker,
            Event::Process(Box::new(agent::process::Event::Exited { owner, exit: Some(agent::Exit::Code(0)) })),
        )?;
        assert!(self.runs[index].reading);
        self.runs[index].reading = false;
        self.event(worker, Event::Process(Box::new(agent::process::Event::Hangup { owner })))?;
        self.event(worker, Event::Process(Box::new(agent::process::Event::Reaped { owner, detail: Box::new([]) })))
    }

    pub fn answer(&mut self, index: usize) -> (host::RunAnswer, wire::Work, Option<wire::Preparation>) {
        self.runs[index].answer.take().expect("terminal answer")
    }

    pub fn ack(&mut self, index: usize) -> Result<(), String> {
        let r = &self.runs[index];
        self.event(r.worker, Event::Link(host::LinkDown::Acknowledge { run: r.run, attempt: r.attempt }))
    }

    pub fn move_worker(&mut self, index: usize) -> Result<(), String> {
        let worker = self.workers.len();
        let new = Worker::new(&self.env.limits);
        assert_eq!(new.disk.files().len(), 0, "replacement has its own empty disk");
        self.workers.push(new);
        self.runs[index].worker = worker;
        self.event(worker, Event::Ready)
    }

    pub fn ask_check(&mut self, index: usize) -> Result<(), String> {
        if self.builder.ask(self.runs[index].run, self.now()) {
            self.holder = Some(index);
        }
        self.inspect()
    }

    pub fn checking(&self, index: usize) -> bool {
        self.holder == Some(index)
    }

    pub fn finish_check(&mut self, index: usize) -> Result<(), String> {
        let next = self.builder.finish(self.runs[index].run);
        self.holder = next.map(|run| self.runs.iter().position(|r| r.run == run).expect("queued run"));
        self.say(index, agent::Up::LongDone)
    }

    pub fn story(&mut self, index: usize, story: Case) -> Result<(), String> {
        let worker = self.runs[index].worker;
        match story {
            Case::Delivered | Case::ContactWithin => {
                if story == Case::ContactWithin {
                    self.event(worker, Event::Lost)?;
                    self.event(worker, Event::Ready)?;
                }
                self.call(index, protocol::delivery(b"Delivery", b"Worker story"))?;
                self.finish_report(index)?;
                self.gone(index)?;
            }
            Case::CancelledDelivery => {
                self.hold_push = true;
                self.call(index, protocol::delivery(b"Delivery", b"Worker story"))?;
                assert!(self.held.is_some());
                let r = &self.runs[index];
                self.event(worker, Event::Link(host::LinkDown::Cancel { run: r.run, attempt: r.attempt }))?;
                self.gone(index)?;
                assert!(self.runs[index].answer.is_none());
                self.hold_push = false;
                let (owner, op) = self.held.take().expect("held push");
                let done = translate::perform(&mut self.forge, &mut self.workers[worker].disk, op);
                self.event(worker, Event::Done { owner, done })?;
            }
            other => panic!("worker-wide story {other:?}"),
        }
        self.completed(index, story)
    }

    pub fn cohort(&mut self, pending: &[usize], story: Case) -> Result<(), String> {
        // The current turn is waiting on a provider, not a silent process. Its
        // announced span outlives both the drain and the contact grace.
        for &index in pending {
            self.say(
                index,
                agent::Up::Long { operation: agent::Operation::Completion, span: Duration::from_secs(120) },
            )?;
        }
        let workers: std::collections::BTreeSet<_> = pending.iter().map(|i| self.runs[*i].worker).collect();
        for &worker in &workers {
            let event = match story {
                Case::ContactPast => Event::Lost,
                Case::Drained | Case::DrainExpired => Event::Shutdown { drain: true },
                Case::LinkDrained | Case::LinkDrainExpired => {
                    Event::Link(host::LinkDown::Pause { mode: jig_domain_link::PauseMode::Drain })
                }
                other => panic!("not worker-wide: {other:?}"),
            };
            self.event(worker, event)?;
        }
        if story == Case::ContactPast {
            self.env.now = self.now().saturating_add(Duration::from_secs(60));
            for &worker in &workers {
                self.event(worker, Event::GracePassed)?;
            }
        } else {
            for &index in pending {
                assert_eq!(self.runs[index].drains, 1);
                assert_eq!(self.runs[index].cancellations, 0);
            }
            self.env.now = self.now().saturating_add(Duration::from_nanos(self.env.limits.drain.as_nanos() - 1));
            for &worker in &workers {
                self.settle(worker)?;
            }
            for &index in pending {
                assert_eq!(self.runs[index].cancellations, 0);
            }
            if matches!(story, Case::DrainExpired | Case::LinkDrainExpired) {
                self.env.now = self.now().saturating_add(Duration::from_nanos(1));
                for &worker in &workers {
                    self.settle(worker)?;
                }
            }
        }
        for &index in pending {
            if story == Case::ContactPast {
                assert_eq!(self.runs[index].cancellations, 1);
            } else {
                let turns = u32::from(matches!(story, Case::Drained | Case::LinkDrained));
                if turns == 1 {
                    self.say(index, agent::Up::Turn { turn: protocol::turn(1, 0, 8) })?;
                } else {
                    assert_eq!(self.runs[index].cancellations, 1);
                }
                self.say(
                    index,
                    agent::Up::Answer {
                        answer: agent::Answer {
                            read: None,
                            turns,
                            spent: 0,
                            result: agent::RunResult::Failed { failure: agent::RunFailure::Cancelled },
                        },
                    },
                )?;
            }
            self.gone(index)?;
        }
        if story == Case::ContactPast {
            for &worker in &workers {
                self.event(worker, Event::Ready)?;
            }
        }
        for &index in pending {
            self.completed(index, story)?;
        }
        Ok(())
    }

    fn completed(&mut self, index: usize, story: Case) -> Result<(), String> {
        let (answer, work, preparation) = self.answer(index);
        assert_eq!(preparation, None);
        let reason = match story {
            Case::Delivered | Case::ContactWithin => None,
            Case::CancelledDelivery => Some(host::Reason::Engine),
            Case::ContactPast => Some(host::Reason::Contact),
            Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired => {
                Some(host::Reason::Shutdown)
            }
        };
        match answer.ending {
            host::Ending::Ended { .. } => assert_eq!(reason, None),
            host::Ending::Failed { failure: host::Failure::Cancelled(found), .. } => assert_eq!(Some(found), reason),
            other => panic!("wrong ending for {story:?}: {other:?}"),
        }
        let saved = !matches!(story, Case::Delivered | Case::ContactWithin | Case::CancelledDelivery);
        let head = self.head(index, saved);
        if saved {
            let saved = work.saved.expect("saved work");
            let [wire::SavedRepository { landing: wire::Landing::Landed { commit }, .. }] = saved.as_ref() else {
                panic!("one saved head")
            };
            assert_eq!(*commit, translate::commit(head).raw());
        } else {
            assert_eq!(
                work.landed.as_ref(),
                [wire::Landed { tag: 1, commit: translate::commit(head).raw(), checked: true }]
            );
        }
        self.tree(index, head);
        self.builder.ended(self.runs[index].run);
        self.ack(index)?;
        if matches!(story, Case::Drained | Case::LinkDrained) {
            let r = &self.runs[index];
            self.event(r.worker, Event::Link(host::LinkDown::TurnKept { run: r.run, attempt: r.attempt, turn: 1 }))?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        assert_eq!(self.forge.branch(REMOTE, b"main"), Some(self.first));
        for worker in 0..self.workers.len() {
            let w = &self.workers[worker];
            assert_eq!(w.domain.agent().agents(), 0);
            assert_eq!(w.domain.workspaces(), 0);
            assert_eq!(w.domain.host().hosted(), 0);
            assert_eq!(w.domain.kept_answers(), 0);
            if w.stopped == 0 {
                self.event(worker, Event::Shutdown { drain: true })?;
            }
            assert_eq!(self.workers[worker].stopped, 1);
        }
        self.inspect()
    }
}
