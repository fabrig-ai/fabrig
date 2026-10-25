//! The agent process as io and smith's process protocol would run it: smith's
//! agent world, with its fake LLM, behind the worker's process face (smith's
//! `smith_host_domain::process`). The world mounts the tree the worker
//! prepared, as the process protocol attaches a spawned agent's directories,
//! and carries the channel's typed records each way; it chooses no turn, call
//! or ending of smith's. One record answers each read; once smith has said
//! its last word and the worker read it, the process exits and is reaped.

use std::collections::VecDeque;

use jig_domain_host as host;
use skein_fake_checkout::Checkout as Disk;
use skein_fake_llm_domain::api::Script;
use skein_lib::{Duration, ReplyTo, Time, Token};
use smith_agent_world::{Job, Settings, World as Agent};
use smith_domain as smith;
use smith_domain_run as run;
use smith_domain_session::{llm, record};
use temper_engine_domain as engine;
use temper_engine_domain::Record;
use temper_engine_domain_world::store::Store;
use temper_fake_checkout::Checkout;
use temper_worker_checkout_world::translate as git;
use temper_worker_domain::{self as worker, agent, checkout::git::Place};

use super::Observed;

/// The agent process's token, as io names it.
const PROCESS: Token = Token::new(701);

/// One activation's process at a time, and what crossed its channel.
pub(crate) struct Process {
    seed: u64,
    /// The fake LLM's script, extended by each resumed activation.
    script: Script,
    /// The previous activation's concrete turns, which a resume restores.
    history: Option<smith::Transcript>,
    agent: Option<Agent>,
    up: VecDeque<agent::Up>,
    reading: Option<Token>,
    waiting: Option<Token>,
    reaping: Option<Token>,
    admitted: bool,
    settled: bool,
    /// smith's last word: its turns and spend, once it said it.
    pub(crate) answered: Option<(u32, u64)>,
    exited: bool,
    sent_turns: usize,
    sent_waits: usize,
    sent_calls: Vec<Token>,
    owner: Option<Token>,
    producer: Option<Producer>,
}

/// The process service's live producer, played here because smith's domain
/// world emits only whole turns (domain/testing.md, 2.1).
struct Producer {
    count: usize,
    pieces: VecDeque<host::Piece>,
    holding: bool,
}

/// What a process request needs of the world around it.
pub(crate) struct Around<'a> {
    pub(crate) events: &'a mut VecDeque<worker::Event>,
    pub(crate) observed: &'a mut Observed,
    pub(crate) disk: &'a Checkout,
    pub(crate) store: &'a Store,
    pub(crate) assignment: Option<&'a engine::Assignment>,
    pub(crate) repository: bool,
}

impl Process {
    pub(crate) fn notice(&mut self, notice: agent::Up) {
        self.up.push_front(notice);
    }
    pub(crate) fn new(seed: u64, script: Script) -> Process {
        Process {
            seed,
            script,
            history: None,
            agent: None,
            up: VecDeque::new(),
            reading: None,
            waiting: None,
            reaping: None,
            admitted: false,
            settled: false,
            answered: None,
            exited: false,
            sent_turns: 0,
            sent_waits: 0,
            sent_calls: Vec::new(),
            owner: None,
            producer: None,
        }
    }

    pub(crate) fn in_pieces(&mut self, count: usize) {
        assert!(count > 0, "a reply has at least one piece");
        self.producer = Some(Producer { count, pieces: VecDeque::new(), holding: false });
    }

    /// Release the next piece, or the turn and its following channel records
    /// after the last piece has been observed.
    pub(crate) fn release_reply(&mut self, events: &mut VecDeque<worker::Event>) {
        let producer = self.producer.as_mut().expect("a reply in pieces");
        assert!(producer.holding, "a held reply");
        if let Some(piece) = producer.pieces.pop_front() {
            // A live channel record consumes a read just like a whole turn;
            // the worker's Progress terminal grants the following read.
            assert_eq!(self.reading.take(), self.owner, "a granted read for the live piece");
            events.push_back(worker::Event::Live { agent: self.owner.expect("spawned agent"), piece });
        } else {
            producer.holding = false;
        }
    }

    /// The next activation, with smith's concrete history and more script.
    pub(crate) fn resume(&mut self, script: Script) {
        let agent = self.agent.as_ref().expect("parked activation");
        assert!(matches!(agent.answer(), run::Answer::Parked { .. }));
        let history = crate::world::transcript(agent);
        let mut turns = self.script.turns.to_vec();
        turns.extend(script.turns);
        let script = Script { cue: self.script.cue.clone(), turns: turns.into_boxed_slice() };
        *self = Process::new(self.seed, script);
        self.history = Some(history);
    }

    /// The live or last activation's smith world.
    pub(crate) fn agent(&self) -> &Agent {
        self.agent.as_ref().expect("spawned Smith process")
    }

    /// Whether smith has received a host call.
    pub(crate) fn called(&self) -> bool {
        !self.sent_calls.is_empty()
    }

    /// One of the worker's process requests, ended by the terminal smith's
    /// host domain names for it.
    pub(crate) fn request(&mut self, request: agent::process::Request, around: &mut Around<'_>) {
        let event = match request {
            agent::process::Request::Spawn { owner, workspace, .. } => {
                assert!(self.owner.replace(owner).is_none(), "one spawned agent");
                assert_eq!(workspace.is_some(), around.repository);
                around.observed.spawns += 1;
                agent::process::Event::Spawned { owner, process: PROCESS }
            }
            agent::process::Request::Send { owner, process, message } => {
                assert_eq!(process, PROCESS);
                self.down(message, around);
                agent::process::Event::Sent { owner }
            }
            agent::process::Request::Read { owner, process } => {
                assert_eq!(process, PROCESS);
                assert!(self.reading.replace(owner).is_none(), "one live process read");
                return;
            }
            agent::process::Request::Wait { owner, .. } => {
                self.waiting = Some(owner);
                return;
            }
            agent::process::Request::Reap { owner, .. } => {
                self.reaping = Some(owner);
                return;
            }
            agent::process::Request::Signal { owner, .. } => agent::process::Event::Signalled { owner },
        };
        around.events.push_back(worker::Event::Process(Box::new(event)));
    }

    fn down(&mut self, message: agent::Down, around: &mut Around<'_>) {
        match message {
            agent::Down::Start { start, window } => self.start(start, window, around),
            agent::Down::Acknowledge { turn } => {
                let agent = self.agent.as_mut().expect("live agent");
                agent.acknowledge_at(agent.now(), turn);
            }
            agent::Down::Answer { call, reply } => {
                around.observed.replies += 1;
                let submission = self
                    .agent()
                    .host_submissions()
                    .iter()
                    .find(|entry| entry.relay.owner == call)
                    .expect("pending host callback")
                    .clone();
                let reply = match reply {
                    agent::Reply::Host { error, body } => {
                        assert!(durable(around.store, &submission.name, &body), "durable host-call decision");
                        run::HostReply::Answered(run::HostAnswer::new(body, error).expect("bounded host answer"))
                    }
                    agent::Reply::Withdrawn => run::HostReply::Withdrawn,
                    agent::Reply::Unavailable => run::HostReply::Unanswered(run::Unanswered::Lost),
                    agent::Reply::Busy => run::HostReply::Busy,
                    agent::Reply::TooLarge => run::HostReply::TooLarge,
                    agent::Reply::Delivery(_) => panic!("unscripted delivery reply"),
                };
                let agent = self.agent.as_mut().expect("live agent");
                agent.return_host_reply(submission.relay, reply).expect("one pending relay");
            }
            agent::Down::Message { name, label, text, finish_requested } => {
                assert!(!finish_requested, "finish-request scripts use the shipped process tier");
                around.observed.messages.push((name, text.clone()));
                let agent = self.agent.as_mut().expect("live agent");
                agent.message_at(agent.now(), name, label, text);
            }
            agent::Down::Cancel => {
                around.observed.cancels += 1;
                self.agent.as_mut().expect("live agent").cancel_run();
            }
            agent::Down::Grant { .. } => {}
        }
    }

    /// smith's start, its directories mounted from the trees the worker
    /// prepared, at the path the spawned process would see them.
    fn start(&mut self, start: agent::Start, window: agent::Window, around: &Around<'_>) {
        assert!(self.agent.is_none());
        let assignment = around.assignment.expect("committed assignment");
        assert_eq!(assignment.live, host::LiveAsk { text: true, reasoning: false, input: true });
        let asked = assignment.live;
        assert_eq!(
            start.live,
            Some(agent::channel::LiveAsk { text: asked.text, reasoning: asked.reasoning, input: asked.input }),
            "smith receives the assignment's live ask"
        );
        assert_eq!((start.logical_run.raw(), start.activation), (assignment.task, assignment.attempt));
        assert_eq!(start.directories.len(), usize::from(around.repository));
        assert_eq!(start.workspace.is_some(), around.repository);
        assert_eq!(window.turns, 2);
        let transcript = start.transcript.map(agent::Transcript::into_value);
        let restored = transcript.as_ref().map_or(0, |transcript| transcript.turns.len());
        assert_eq!(restored, assignment.transcript.len(), "the committed turns cross whole");
        if let Some(history) = &self.history {
            let restored = transcript.as_ref().expect("a resumed conversation");
            assert_eq!(restored.turns, history.turns, "restored concrete history is the committed prefix");
        }
        let mut settings = Settings::calm(self.seed);
        let largest = smith::max_turn_bytes(&settings.limits).expect("bounded Smith turn");
        let mut mounted = Disk::new();
        let workspace = mount(&start.directories, around.disk, &mut mounted);
        let charter = start.charter.into_value();
        settings.job = Job::Reporting;
        settings.resume = transcript.is_some();
        settings.budget = charter.budget;
        settings.limits.run.budget = charter.budget;
        settings.limits.run.host_tools =
            u32::try_from(temper_engine_smith::tools(Duration::from_secs(10)).len()).expect("tool count");
        settings.limits.run.brief_sections = 16;
        settings.limits.session.spend = charter.budget.spend;
        let event = smith::Event::Start {
            reply_to: ReplyTo::new(Token::new(402)),
            host_run: start.logical_run,
            activation: start.activation,
            identity: start.identity,
            window: smith::Window { turns: window.turns, bytes: largest * u64::from(window.turns) },
            messages: messages(start.messages),
            charter,
            workspace,
            transcript,
            answered: answered(start.answered),
            grants: grants(&start.grants),
        };
        let mut agent = Agent::with_workspace_scripts_start(settings, mounted, Box::new([self.script.clone()]), event);
        agent.enable_parent_host_calls();
        self.agent = Some(agent);
    }

    /// One iteration of smith's world, then what it said, in order, onto the
    /// channel; and one record for a waiting read.
    pub(crate) fn tick(&mut self, now: Time, around: &mut Around<'_>) {
        self.drive(now, around);
        let held_turn = self.producer.as_ref().is_some_and(|producer| producer.holding)
            && matches!(self.up.front(), Some(agent::Up::Turn { .. }));
        if !held_turn && let Some(owner) = self.reading.take() {
            match self.up.pop_front() {
                Some(message) => {
                    let event = agent::process::Event::Received { owner, message };
                    around.events.push_back(worker::Event::Process(Box::new(event)));
                }
                None => self.reading = Some(owner),
            }
        }
        if self.answered.is_some()
            && self.up.is_empty()
            && !self.exited
            && let (Some(read), Some(wait)) = (self.reading, self.waiting)
        {
            self.reading = None;
            self.waiting = None;
            let exited = agent::process::Event::Exited { owner: wait, exit: Some(agent::Exit::Code(0)) };
            around.events.push_back(worker::Event::Process(Box::new(exited)));
            around.events.push_back(worker::Event::Process(Box::new(agent::process::Event::Hangup { owner: read })));
            self.exited = true;
        }
        if self.exited
            && let Some(owner) = self.reaping.take()
        {
            let reaped = agent::process::Event::Reaped { owner, detail: Box::new([]) };
            around.events.push_back(worker::Event::Process(Box::new(reaped)));
            around.observed.gone += 1;
        }
    }

    fn drive(&mut self, now: Time, around: &mut Around<'_>) {
        let Some(agent) = self.agent.as_mut() else { return };
        if self.settled || self.producer.as_ref().is_some_and(|producer| producer.holding) {
            return;
        }
        self.settled = agent.drive(1);
        if !self.admitted
            && agent
                .messages_seen()
                .iter()
                .any(|(_, seen)| matches!(seen, smith_agent_world::messages_referee::Seen::Admitted))
        {
            self.up.push_back(agent::Up::Admitted);
            self.admitted = true;
        }
        for submission in agent.pending_host_calls() {
            if self.sent_calls.contains(&submission.relay.owner) {
                continue;
            }
            self.sent_calls.push(submission.relay.owner);
            let effect = match submission.effect {
                run::HostEffect::Read => agent::Effect::Read,
                run::HostEffect::Write => agent::Effect::Write,
            };
            self.up.push_back(agent::Up::Call {
                call: submission.relay.owner,
                name: agent::CallName {
                    activation: submission.name.activation,
                    completion: submission.name.completion,
                    position: submission.name.position,
                },
                deadline: now.saturating_add(submission.deadline.saturating_since(submission.at)),
                ask: agent::Ask::Host { tool: submission.tool.clone(), effect, body: submission.input.bytes().into() },
            });
        }
        for (number, read, spent) in &agent.turn_metadata()[self.sent_turns..] {
            let index = usize::try_from(*number - 1).expect("positive turn");
            let record = agent.turns()[index].clone();
            if let Some(producer) = &mut self.producer {
                let mut text = Vec::new();
                for message in &record.messages {
                    if message.role == llm::Role::Assistant {
                        for block in &message.content {
                            if let llm::Block::Text { text: bytes, .. } = block {
                                text.extend_from_slice(bytes);
                            }
                        }
                    }
                }
                assert!(text.len() >= producer.count, "one nonempty text slice per piece");
                assert!(!producer.holding, "one held reply turn");
                for part in 0..producer.count {
                    let from = text.len() * part / producer.count;
                    let to = text.len() * (part + 1) / producer.count;
                    producer.pieces.push_back(host::Piece {
                        turn: *number,
                        retry: 0,
                        kind: host::LiveKind::Text,
                        offset: u64::try_from(from).expect("bounded reply offset"),
                        bytes: text[from..to].into(),
                    });
                }
                let piece = producer.pieces.pop_front().expect("the first reply piece");
                assert_eq!(self.reading.take(), self.owner, "a granted read for the live piece");
                around.events.push_back(worker::Event::Live { agent: self.owner.expect("spawned agent"), piece });
                producer.holding = true;
            }
            self.up.push_back(agent::Up::Turn {
                turn: agent::Turn {
                    opens_window: matches!(record.kind, record::Kind::First(_)),
                    number: *number,
                    spent: spent.units,
                    read: *read,
                    body: agent::TurnValue::new(record, u64::MAX).expect("an owned turn"),
                },
            });
        }
        self.sent_turns = agent.turn_metadata().len();
        for (_, read) in &agent.waiting()[self.sent_waits..] {
            self.up.push_back(agent::Up::Waiting { read: *read });
        }
        self.sent_waits = agent.waiting().len();
        if self.settled {
            let mut read = None;
            for (_, turn_read, _) in agent.turn_metadata() {
                if turn_read.is_some() {
                    read = *turn_read;
                }
            }
            let (turns, spent, result) = result(agent.answer());
            self.up.push_back(agent::Up::Answer { answer: agent::Answer { read, turns, spent, result } });
            self.answered = Some((turns, spent));
        }
    }
}

/// smith's last word on its channel. Only the endings these stories script
/// cross; any other is a finding.
fn result(answer: &run::Answer) -> (u32, u64, agent::RunResult) {
    match crate::world::copy_answer(answer) {
        run::Answer::Accepted { outcome, spent, turns } => {
            let outcome = agent::Declared::new(outcome, u64::MAX).expect("a bounded result");
            (turns, spent.units, agent::RunResult::Accepted { outcome })
        }
        run::Answer::Parked { spent, turns } => (turns, spent.units, agent::RunResult::Parked),
        run::Answer::Failed { failure: run::Failure::Cancelled, spent, turns } => {
            (turns, spent.units, agent::RunResult::Failed { failure: agent::RunFailure::Cancelled })
        }
        other @ (run::Answer::Refused(_) | run::Answer::Failed { .. }) => panic!("unscripted Smith ending: {other:?}"),
    }
}

/// Whether the engine keeps the decision of the call `name`, settled as `body`.
fn durable(store: &Store, name: &run::CallName, body: &[u8]) -> bool {
    store.rows.values().any(|record| {
        matches!(record,
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record)))
                if record.key.completion == name.completion
                    && record.key.position == name.position
                    && matches!(&record.settled, Some(jig_domain_core::SettledCall {
                        answer: jig_domain_core::SettledAnswer::Host { body: kept, .. }, ..
                    }) if kept.as_ref() == body))
    })
}

/// Mount each prepared directory into smith's disk, under its name, at the
/// absolute path the process would see it.
fn mount(directories: &[agent::Directory], disk: &Checkout, mounted: &mut Disk) -> Option<run::Workspace> {
    if directories.is_empty() {
        return None;
    }
    let mut mounts = Vec::with_capacity(directories.len());
    for directory in directories {
        let place = Place { git: None, workspace: directory.root, repository: directory.name.clone() };
        let prefix = [git::path(&place).as_slice(), b"/"].concat();
        for (file, content) in disk.files() {
            if let Some(relative) = file.strip_prefix(prefix.as_slice()) {
                mounted.write(&[directory.name.as_ref(), b"/", relative].concat(), content);
            }
        }
        mounts.push(run::Directory {
            name: directory.name.clone(),
            host: [b"/".as_slice(), directory.name.as_ref()].concat().into_boxed_slice(),
            root: Token::new(mounted.root(&directory.name)),
            writable: directory.writable,
            git: directory.git,
            conflicts: directory.conflicts.clone(),
        });
    }
    Some(run::Workspace { directories: mounts.into_boxed_slice() })
}

fn messages(messages: Box<[agent::Message]>) -> Box<[run::Message]> {
    let mut translated = Vec::with_capacity(messages.len());
    for message in messages {
        translated.push(run::Message {
            name: message.name,
            label: message.label,
            text: message.text,
            finish_requested: message.finish_requested,
        });
    }
    translated.into_boxed_slice()
}

fn answered(calls: Box<[agent::AnsweredCall]>) -> Box<[smith::AnsweredCall]> {
    let mut translated = Vec::with_capacity(calls.len());
    for call in calls {
        let answer = match call.reply {
            agent::SavedReply::Host { error, body } => {
                smith::Answered::Host(run::HostAnswer::new(body, error).expect("bounded host answer"))
            }
            agent::SavedReply::TooLarge => smith::Answered::TooLarge,
            agent::SavedReply::Delivery(_) => panic!("unscripted delivery replay"),
        };
        let name = run::CallName {
            activation: call.name.activation,
            completion: call.name.completion,
            position: call.name.position,
        };
        translated.push(smith::AnsweredCall { name, tool: call.tool, answer });
    }
    translated.into_boxed_slice()
}

fn grants(grants: &[agent::Grant]) -> Box<[smith::Grant]> {
    let mut translated = Vec::with_capacity(grants.len());
    for grant in grants {
        let name = smith::GrantName { account: grant.account, generation: grant.generation };
        translated.push(smith::Grant {
            name,
            valid: grant.valid,
            renewable: matches!(grant.form, agent::Form::Credential(agent::Scheme::Bearer)),
        });
    }
    translated.into_boxed_slice()
}

#[cfg(test)]
mod notices;
