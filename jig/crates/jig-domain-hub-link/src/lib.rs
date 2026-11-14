//! The engine's hub link (jig/docs/design/domain/hosts/hub.md, 1–8).
//! The root routes core events, hub deliveries and protocol hand-outs.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use jig_domain_core_vocabulary as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::Grant;
use jig_domain_tasks as tasks;
use skein_lib::{Deadlines, Duration, Map, Queue, ReplyTo, Rng, Time, Token};
mod down;
mod grants;
mod leaves;
mod payload;
pub use core::Assignment;
pub use down::{down, now};
pub use leaves::{brief_plan, job_assignment, settled_answer};
pub use payload::{Ending, Payload, up};

/// A job the core has claimed, for the fleet to place (jig/docs/design/domain/engine/executors.md, 5).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct JobStart {
    pub task: u64,
    pub attempt: u64,
    pub workstream: u64,
    pub kind: jig_domain_link::JobKind,
    pub spec: Box<[u8]>,
    pub deadline: Duration,
    /// Its contract's result bound, which the host's bound must cover.
    pub result: u32,
    /// The application's credentials its workspace needs.
    pub grants: Box<[Grant]>,
    /// The application's encoded workspace items, absent when the job needs none.
    pub workspace: Option<Box<[u8]>>,
}

/// Bounds inherited from the composed children.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub assignments: u32,
    pub payloads: u32,
    pub turns: u32,
}
/// A relay's destination, fenced by its attempt.
#[derive(Clone, Copy, Hash, Debug)]
pub struct HostCall {
    pub channel: Token,
    pub run: Token,
    pub attempt: Token,
}
/// A procedure's continuation in the core's numbered connector vocabulary.
#[derive(Clone, Copy, Hash, Debug)]
pub struct Procedure {
    pub step: u64,
    pub connector: u16,
    pub code: u32,
}
/// An effect's application-owned encoding, handed back to its connector.
#[derive(Hash, Debug)]
pub struct Effect {
    pub connector: u16,
    pub input: Box<[u8]>,
}
/// A connector read's caller and opaque tool envelope.
#[derive(Hash, Debug)]
pub struct ReadCall {
    pub to: Token,
    pub tool: Box<[u8]>,
}
/// The identity kept beside a connector's durable outbox entry.
#[derive(Clone, Copy, Hash, Debug)]
pub struct OutboxOwner {
    pub task: u64,
    pub projection: bool,
}
/// Volatile hub wiring; random jitter is supplied by the root.
#[derive(Hash, Debug)]
pub struct Link {
    assignments: Map<u64, Assignment>,
    jobs: Map<u64, JobStart>,
    briefs: Map<u64, Box<[jig_domain_brief::GatherPlaced]>>,
    payloads: Map<Token, Payload>,
    next_payload: u64,
    turn_backoffs: Deadlines<(Token, Token, u32)>,
    turn_rng: Rng,
    pieces_dropped: u64,
    host_calls: Map<Box<[u8]>, HostCall>,
    channels: Map<Token, ()>,
    read_calls: Map<Token, ReadCall>,
    close_replies: Map<Token, [u8; 16]>,
    topic_calls: Map<Token, (ReplyTo, core::CallKey, u16)>,
    judges: Map<Token, (Token, jig_domain_authority::Judge, [u8; 32])>,
    procedures: Map<u64, Procedure>,
    effects: Map<Token, Effect>,
    effect_procedures: Map<Token, u64>,
    outbox_procedures: Map<u64, u16>,
    outbox_owners: Map<u64, OutboxOwner>,
}
impl Link {
    #[must_use]
    pub fn new(limits: Limits, rng: Rng) -> Self {
        Self {
            assignments: Map::with_capacity(limits.assignments),
            jobs: Map::with_capacity(limits.assignments),
            briefs: Map::with_capacity(limits.assignments),
            payloads: Map::with_capacity(limits.payloads),
            next_payload: 1,
            turn_backoffs: Deadlines::with_capacity(limits.turns),
            turn_rng: rng,
            pieces_dropped: 0,
            host_calls: Map::uncapped(),
            channels: Map::uncapped(),
            read_calls: Map::uncapped(),
            close_replies: Map::uncapped(),
            topic_calls: Map::uncapped(),
            judges: Map::uncapped(),
            procedures: Map::uncapped(),
            effects: Map::uncapped(),
            effect_procedures: Map::uncapped(),
            outbox_procedures: Map::uncapped(),
            outbox_owners: Map::uncapped(),
        }
    }
    /// Register a channel while the fleet judges its hello.
    pub fn connect(&mut self, channel: Token) {
        self.channels.insert(channel, ()).expect("uncapped channels");
    }
    pub fn disconnect(&mut self, channel: Token) {
        self.channels.remove(&channel);
    }
    #[must_use]
    pub fn connected(&self, channel: Token) -> bool {
        self.channels.contains_key(&channel)
    }
    /// Forget the opaque relay when its protocol continuation is dropped.
    pub fn forget_call(&mut self, name: &[u8]) {
        self.host_calls.remove(name);
    }
    pub fn keep_read_call(&mut self, token: Token, call: ReadCall) {
        self.read_calls.insert(token, call).expect("uncapped read calls");
    }
    #[must_use]
    pub fn take_read_call(&mut self, token: Token) -> Option<ReadCall> {
        self.read_calls.remove(&token)
    }
    pub fn keep_close_reply(&mut self, token: Token, key: [u8; 16]) {
        self.close_replies.insert(token, key).expect("uncapped close replies");
    }
    #[must_use]
    pub fn take_close_reply(&mut self, token: Token) -> Option<[u8; 16]> {
        self.close_replies.remove(&token)
    }
    /// Retain the child's opaque topic call continuation.
    pub fn keep_topic_call(&mut self, key: Token, value: (ReplyTo, core::CallKey, u16)) {
        self.topic_calls.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_topic_call(&mut self, key: Token) -> Option<(ReplyTo, core::CallKey, u16)> {
        self.topic_calls.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn topic_call(&self, key: Token) -> Option<&(ReplyTo, core::CallKey, u16)> {
        self.topic_calls.get(&key)
    }
    /// Retain the child's opaque judge continuation.
    pub fn keep_judge(&mut self, key: Token, value: (Token, jig_domain_authority::Judge, [u8; 32])) {
        self.judges.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_judge(&mut self, key: Token) -> Option<(Token, jig_domain_authority::Judge, [u8; 32])> {
        self.judges.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn judge(&self, key: Token) -> Option<&(Token, jig_domain_authority::Judge, [u8; 32])> {
        self.judges.get(&key)
    }
    /// Retain the child's opaque procedure continuation.
    pub fn keep_procedure(&mut self, key: u64, value: Procedure) {
        self.procedures.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_procedure(&mut self, key: u64) -> Option<Procedure> {
        self.procedures.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn procedure(&self, key: u64) -> Option<&Procedure> {
        self.procedures.get(&key)
    }
    /// Retain an effect without naming an application's decoded value.
    pub fn keep_effect(&mut self, owner: Token, effect: Effect) {
        self.effects.insert(owner, effect).expect("uncapped effect inputs");
    }
    /// Hand the opaque input back to the connector selected by the core.
    #[must_use]
    pub fn take_effect(&mut self, owner: Token) -> Option<Effect> {
        self.effects.remove(&owner)
    }
    /// Retain the child's opaque effect procedure continuation.
    pub fn keep_effect_procedure(&mut self, key: Token, value: u64) {
        self.effect_procedures.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_effect_procedure(&mut self, key: Token) -> Option<u64> {
        self.effect_procedures.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn effect_procedure(&self, key: Token) -> Option<&u64> {
        self.effect_procedures.get(&key)
    }
    /// Retain the child's opaque outbox procedure continuation.
    pub fn keep_outbox_procedure(&mut self, key: u64, value: u16) {
        self.outbox_procedures.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_outbox_procedure(&mut self, key: u64) -> Option<u16> {
        self.outbox_procedures.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn outbox_procedure(&self, key: u64) -> Option<&u16> {
        self.outbox_procedures.get(&key)
    }
    /// Retain the child's opaque outbox owner continuation.
    pub fn keep_outbox_owner(&mut self, key: u64, value: OutboxOwner) {
        self.outbox_owners.insert(key, value).expect("uncapped continuation table");
    }
    /// Consume a continuation once its child answers.
    #[must_use]
    pub fn take_outbox_owner(&mut self, key: u64) -> Option<OutboxOwner> {
        self.outbox_owners.remove(&key)
    }
    /// Inspect the continuation without consuming its reply right.
    #[must_use]
    pub fn outbox_owner(&self, key: u64) -> Option<&OutboxOwner> {
        self.outbox_owners.get(&key)
    }
    #[must_use]
    pub fn payload_count(&self) -> u32 {
        self.payloads.len()
    }
    /// Mint one name for a payload or a root's opaque continuation.
    #[must_use]
    pub fn fresh(&mut self) -> Token {
        let token = Token::new(self.next_payload);
        self.next_payload = self.next_payload.checked_add(1).expect("link token room");
        token
    }
    /// Keep an owned payload until its fleet hand-out returns.
    pub fn retain(&mut self, value: Payload) -> Token {
        let token = self.fresh();
        self.payloads.insert(token, value).expect("payload room reserved");
        token
    }
    #[must_use]
    pub fn payload(&self, token: Token) -> Option<&Payload> {
        self.payloads.get(&token)
    }
    #[must_use]
    pub fn take_payload(&mut self, token: Token) -> Option<Payload> {
        self.payloads.remove(&token)
    }
    #[must_use]
    pub fn assignment(&self, task: u64) -> Option<&Assignment> {
        self.assignments.get(&task)
    }
    #[must_use]
    pub fn take_assignment(&mut self, task: u64) -> Option<Assignment> {
        self.jobs.remove(&task);
        self.assignments.remove(&task)
    }
    pub fn keep_assignment(&mut self, assignment: Assignment) {
        self.assignments.insert(assignment.task, assignment).expect("one assignment per task");
    }
    pub fn keep_brief(&mut self, task: u64, brief: Box<[jig_domain_brief::GatherPlaced]>) {
        self.briefs.insert(task, brief).expect("one brief per task");
    }
    #[must_use]
    pub fn take_brief(&mut self, task: u64) -> Option<Box<[jig_domain_brief::GatherPlaced]>> {
        self.briefs.remove(&task)
    }
    #[must_use]
    pub const fn pieces_dropped(&self) -> u64 {
        self.pieces_dropped
    }
}
/// Inputs from the fleet, hub and protocol hand-outs.
#[derive(Debug)]
pub enum Event {
    Asked { channel: Token, run: Token, attempt: Token, number: u64, answer: host::AskAnswer },
    Prepared { assignment: Assignment, writes: Box<[tasks::Name]> },
    StartJob { job: JobStart },
    Start { task: u64, attempt: u64, kinds: fleet::Kinds, version: fleet::Version },
    Hello { channel: Token, worker: u32, hello: jig_domain_link::Hello },
    Up { channel: Token, value: host::LinkUp },
    Lost { channel: Token },
    Receive { channel: Token, payload: Payload },
    ResultRead { payload: Token, end: tasks::End },
    AnswerPayload { run: Token, attempt: Token, payload: Token },
    TurnPayload { run: Token, attempt: Token, turn: u32, body: Token },
    AcceptedTurn { payload: Token, task: u64, attempt: u64, turn: u32, accepted: tasks::Accepted },
    RefusedPayload { request: Token, problem: tasks::Problem },
    DropPayload { payload: Token },
    SettledCall { to: ReplyTo, call: core::SettledCall },
    Busy { to: ReplyTo },
    Relayed { channel: Token, run: Token, attempt: Token, call: Box<[u8]>, answer: Token },
    Assign { channel: Token, run: Token, attempt: Token },
    Down { channel: Token, value: host::LinkDown },
    Inbound { channel: Token, run: Token, attempt: Token, message: fleet::Message },
    RelayBusy { channel: Token, run: Token, attempt: Token, call: Box<[u8]> },
    DropCall { call: fleet::Call },
    Call { to: ReplyTo, run: Token, attempt: Token, call: fleet::Call },
    TurnBusy { run: Token, attempt: Token, turn: u32, retained: bool },
    TurnKept { channel: Token, run: Token, attempt: Token, turn: u32 },
}
/// The root routes each output to one child or its protocol layer.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the route owns complete typed core events")]
pub enum Request {
    Core(core::Event),
    Down { channel: Token, value: host::LinkDown },
    Assigned { channel: Token, assignment: Assignment },
    AssignedJob { channel: Token, job: JobStart },
    ReadResult { payload: Token, outcome: Box<[u8]> },
    Relayed { channel: Token, run: Token, attempt: Token, call: Box<[u8]>, answer: Option<core::SettledCall> },
    Message { channel: Token, run: Token, attempt: Token, word: tasks::Word },
    ConnectorCall { to: ReplyTo, run: Token, attempt: Token, call: fleet::Call },
    TurnCopy { run: Token, attempt: Token, turn: u32 },
}
/// Route one input; the root applies the journal's durability class to outputs.
#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher routes the link boundary")]
pub fn step(link: &mut Link, now: Time, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Prepared { mut assignment, writes } => {
            let task = assignment.task;
            let attempt = assignment.attempt;
            let budget = assignment.run.budget;
            assignment.brief = link.take_brief(task).expect("completed brief");
            link.keep_assignment(assignment);
            out.push(Request::Core(core::Event::ClaimPrepared { task, attempt, budget, writes }));
        }
        Event::StartJob { job } => {
            let (task, attempt) = (job.task, job.attempt);
            let bound = jig_domain_link::JobBound {
                kind: job.kind,
                spec: u32::try_from(job.spec.len()).expect("validated job spec"),
                result: job.result,
                deadline: job.deadline,
            };
            let workstream = job.workstream;
            link.jobs.insert(task, job).expect("one prepared job per claim");
            out.push(Request::Core(core::Event::Fleet(fleet::Event::StartJob {
                reply_to: ReplyTo::new(Token::new(task)),
                run: Token::new(task),
                attempt: Token::new(attempt),
                workstream,
                job: bound,
                adopt: false,
            })));
        }
        Event::Start { task, attempt, kinds, version } => {
            let assignment = link.assignment(task).expect("prepared assignment");
            assert!(assignment.attempt == attempt, "prepared attempt fence");
            let grants = grants::fleet(&assignment.grants);
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Start {
                placement: Some(assignment.run.placement),
                grants,
                reply_to: ReplyTo::new(Token::new(task)),
                run: Token::new(task),
                attempt: Token::new(attempt),
                workstream: task,
                assignment: fleet::Assignment { turns: Token::new(task), answered: Token::new(task) },
                kinds,
                version,
            })));
        }
        Event::Hello { channel, worker, hello } => {
            link.connect(channel);
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Hello { channel, worker, hello })));
        }
        Event::Lost { channel } => {
            link.disconnect(channel);
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Lost { channel })));
        }
        Event::Up { channel, value } => {
            if let Some(event) = up(link, channel, value) {
                out.push(Request::Core(event));
            }
        }
        Event::Receive { channel, payload } => {
            let (run, attempt, turn) = match &payload {
                Payload::Turn { task, attempt, turn, .. } => (Token::new(*task), Token::new(*attempt), Some(*turn)),
                Payload::Answer { task, attempt, .. } => (Token::new(*task), Token::new(*attempt), None),
                Payload::Busy | Payload::SettledCall(_) | Payload::InboxWord(_) => {
                    unreachable!("turn or answer metadata")
                }
            };
            let name = link.retain(payload);
            let event = match turn {
                Some(turn) => fleet::Event::Turn { channel, run, attempt, turn, body: name },
                None => fleet::Event::Answer { channel, run, attempt, answer: fleet::Answer::Ended, payload: name },
            };
            out.push(Request::Core(core::Event::Fleet(event)));
        }
        Event::ResultRead { payload, end } => {
            if let Some(event) = result_event(link, payload, end) {
                out.push(Request::Core(event));
            }
        }
        Event::AnswerPayload { run, attempt, payload } => answer(link, run, attempt, payload, out),
        Event::TurnPayload { run, attempt, turn, body } => {
            let value = link.payload(body).expect("fleet retained the turn body");
            let (cumulative, read, refused) = match value {
                Payload::Turn { cumulative, read, refused, .. } => (*cumulative, *read, refused.clone()),
                Payload::Busy | Payload::InboxWord(_) | Payload::SettledCall(_) | Payload::Answer { .. } => {
                    unreachable!("turn body family")
                }
            };
            out.push(Request::Core(core::Event::TurnPayload { run, attempt, turn, body, cumulative, read, refused }));
        }
        Event::AcceptedTurn { payload, task, attempt, turn, accepted } => {
            let body = link.take_payload(payload).expect("accepted turn body");
            let (cumulative, read, transcript, opens_window) = match body {
                Payload::Turn { cumulative, read, transcript, opens_window, .. } => {
                    (cumulative, read, transcript, opens_window)
                }
                Payload::Busy | Payload::InboxWord(_) | Payload::SettledCall(_) | Payload::Answer { .. } => {
                    unreachable!("accepted turn body family")
                }
            };
            out.push(Request::Core(core::Event::AcceptedTurn {
                task,
                attempt,
                turn,
                accepted,
                opens_window,
                cumulative,
                read,
                transcript,
            }));
        }
        Event::RefusedPayload { request, problem } => {
            let payload = match link.take_payload(request) {
                Some(Payload::Turn { task, attempt, turn, .. }) => {
                    Some(core::PayloadRefusal::Turn { task, attempt, turn })
                }
                Some(Payload::Answer { task, attempt, .. }) => Some(core::PayloadRefusal::Answer { task, attempt }),
                Some(Payload::Busy | Payload::InboxWord(_) | Payload::SettledCall(_)) => {
                    unreachable!("turn or terminal")
                }
                None => None,
            };
            out.push(Request::Core(core::Event::RefusedPayload { request, problem, payload }));
        }
        Event::DropPayload { payload } => {
            drop(link.take_payload(payload));
        }
        Event::SettledCall { to, call } => {
            let answer = link.retain(Payload::SettledCall(call));
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Relayed { to, answer })));
        }
        Event::Busy { to } => {
            let answer = link.retain(Payload::Busy);
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Relayed { to, answer })));
        }
        Event::Asked { channel, run, attempt, number, answer } => {
            if link.connected(channel) {
                out.push(Request::Down { channel, value: host::LinkDown::Asked { run, attempt, number, answer } });
            }
        }
        Event::Relayed { channel, run, attempt, call, answer } => {
            link.host_calls.remove(&call);
            let answer = match link.take_payload(answer).expect("retained relay answer") {
                Payload::Busy => None,
                Payload::SettledCall(call) => Some(call),
                Payload::Turn { .. } | Payload::Answer { .. } | Payload::InboxWord(_) => {
                    unreachable!("relay answer family")
                }
            };
            out.push(Request::Relayed { channel, run, attempt, call, answer });
        }
        Event::Assign { channel, run, attempt } => {
            if let Some(job) = link.jobs.get(&run.raw()) {
                assert_eq!(job.attempt, attempt.raw(), "current job assignment fence");
                out.push(Request::AssignedJob {
                    channel,
                    job: JobStart {
                        task: job.task,
                        attempt: job.attempt,
                        workstream: job.workstream,
                        kind: job.kind,
                        spec: job.spec.clone(),
                        deadline: job.deadline,
                        result: job.result,
                        grants: job.grants.clone(),
                        workspace: job.workspace.clone(),
                    },
                });
                return;
            }
            let assignment = link.take_assignment(run.raw()).expect("prepared claim has assignment");
            assert!(assignment.attempt == attempt.raw(), "current assignment fence");
            out.push(Request::Assigned { channel, assignment });
        }
        Event::Down { channel, value } => out.push(Request::Down { channel, value }),
        Event::Inbound { channel, run, attempt, message } => {
            let Some(Payload::InboxWord(word)) = link.take_payload(message.words) else {
                unreachable!("inbox word payload")
            };
            out.push(Request::Message { channel, run, attempt, word });
        }
        Event::RelayBusy { channel, run, attempt, call } => {
            link.host_calls.remove(&call);
            out.push(Request::Down {
                channel,
                value: host::LinkDown::Relayed { run, attempt, call, answer: host::Relayed::Busy },
            });
        }
        Event::DropCall { call } => {
            if let Some(binding) = link.host_calls.remove(&call.name) {
                out.push(Request::Down {
                    channel: binding.channel,
                    value: host::LinkDown::Relayed {
                        run: binding.run,
                        attempt: binding.attempt,
                        call: call.name,
                        answer: host::Relayed::Answer { error: true, text: skein_lib::bytes::copy_of(b"unavailable") },
                    },
                });
            }
        }
        Event::Call { to, run, attempt, call } => out.push(Request::ConnectorCall { to, run, attempt, call }),
        Event::TurnBusy { run, attempt, turn, retained } => {
            if retained {
                let span = Duration::from_nanos(link.turn_rng.between(50_000_000, 100_000_000));
                link.turn_backoffs
                    .arm((run, attempt, turn), now.saturating_add(span))
                    .expect("hub retained turn bound");
            }
        }
        Event::TurnKept { channel, run, attempt, turn } => {
            link.turn_backoffs.cancel((run, attempt, turn));
            out.push(Request::Down { channel, value: host::LinkDown::TurnKept { run, attempt, turn } });
        }
    }
}
/// Ask for one retained hub turn at its retry deadline.
pub fn fire(link: &mut Link, now: Time, out: &mut Queue<Request>) {
    if let Some((run, attempt, turn)) = link.turn_backoffs.expire(now) {
        out.push(Request::TurnCopy { run, attempt, turn });
    }
}
#[must_use]
pub fn next_deadline(link: &Link) -> Option<Time> {
    link.turn_backoffs.next()
}
fn answer(domain: &mut Link, run: Token, attempt: Token, payload: Token, work: &mut Queue<Request>) {
    let Some(Payload::Answer { task, attempt: retained, end, .. }) = domain.payloads.get(&payload) else {
        unreachable!("fleet retained the answer body");
    };
    assert!(*task == run.raw() && *retained == attempt.raw(), "fenced answer identity");
    let end = match end {
        Ending::Link(host::Ending::Ended { outcome, .. }) => {
            work.push(Request::ReadResult { payload, outcome: outcome.clone() });
            return;
        }
        Ending::Link(
            other @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }),
        ) => payload::ending(other),
        Ending::Scripted(end) => end.clone(),
        Ending::Job(ending) => payload::job_end(ending),
    };
    domain.jobs.remove(&run.raw());
    work.push(Request::Core(result_event(domain, payload, end).expect("retained answer")));
}
#[must_use]
pub fn result_event(domain: &Link, payload: Token, end: tasks::End) -> Option<core::Event> {
    let Some(Payload::Answer { task, attempt, cumulative, read, refused, end: retained }) =
        domain.payloads.get(&payload)
    else {
        return None;
    };
    Some(core::Event::AnswerPayload {
        run: Token::new(*task),
        attempt: Token::new(*attempt),
        payload,
        cumulative: *cumulative,
        end,
        saved: None,
        invalid_saved: false,
        unusable_history: match retained {
            Ending::Link(ending) => payload::unusable_history(ending),
            Ending::Scripted(_) | Ending::Job(_) => false,
        },
        read: *read,
        refused: refused.clone(),
    })
}

#[cfg(test)]
mod tests;
