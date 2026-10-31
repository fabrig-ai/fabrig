//! The engine's hub link (jig/docs/design/domain/hosts.md, 6.1–6.8).
//! The root routes core events, hub deliveries and protocol hand-outs.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Deadlines, Duration, Map, Queue, ReplyTo, Rng, Time, Token};
mod payload;
pub use payload::{Ending, Payload, up};
/// The root's assembled, worker assignment.
#[derive(Debug)]
pub struct Assignment {
    pub live: core::LiveAsk,
    /// Ordered sections selected by the brief child, typed until protocol routing.
    pub brief: Box<[jig_domain_brief::GatherPlaced]>,
    /// The conversation's first attempt, shared by its resumes.
    pub conversation: u64,
    /// Claimed task and activation fence.
    pub task: u64,
    pub attempt: u64,
    /// Charter and task-specific policy and contract.
    pub charter: u32,
    pub run: Box<core::RunCharter>,
    /// Whole words and prior committed conversation offered to the worker.
    pub messages: Box<[tasks::Word]>,
    pub transcript: Box<[Box<[u8]>]>,
    pub answered: Box<[core::Settled]>,
    /// The selected account and application credential grants.
    pub grants: Box<[jig_domain_accounts::Grant]>,
}

/// Bounds inherited from the composed children.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub assignments: u32,
    pub payloads: u32,
    pub turns: u32,
}
/// Volatile hub wiring; random jitter is supplied by the root.
#[derive(Debug)]
pub struct Link {
    assignments: Map<u64, Assignment>,
    briefs: Map<u64, Box<[jig_domain_brief::GatherPlaced]>>,
    payloads: Map<Token, Payload>,
    next_payload: u64,
    turn_backoffs: Deadlines<(Token, Token, u32)>,
    turn_rng: Rng,
    pieces_dropped: u64,
}
impl Link {
    #[must_use]
    pub fn new(limits: Limits, rng: Rng) -> Self {
        Self {
            assignments: Map::with_capacity(limits.assignments),
            briefs: Map::with_capacity(limits.assignments),
            payloads: Map::with_capacity(limits.payloads),
            next_payload: 1,
            turn_backoffs: Deadlines::with_capacity(limits.turns),
            turn_rng: rng,
            pieces_dropped: 0,
        }
    }
    #[must_use]
    pub fn payload_count(&self) -> u32 {
        self.payloads.len()
    }
    /// Keep an owned payload until its fleet hand-out returns.
    pub fn retain(&mut self, value: Payload) -> Token {
        let token = Token::new(self.next_payload);
        self.next_payload = self.next_payload.checked_add(1).expect("payload token room");
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
#[expect(clippy::large_enum_variant, reason = "the route owns complete typed core continuations")]
pub enum Event {
    Hello { channel: Token, worker: u32, hello: host::Hello },
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
    ReadResult { payload: Token, outcome: Box<[u8]> },
    Relayed { channel: Token, run: Token, attempt: Token, call: Box<[u8]>, answer: Option<core::SettledCall> },
    TurnCopy { run: Token, attempt: Token, turn: u32 },
}
/// Route one input; the root applies the journal's durability class to outputs.
pub fn step(link: &mut Link, now: Time, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Hello { channel, worker, hello } => {
            out.push(Request::Core(core::Event::Fleet(fleet::Event::Hello { channel, worker, hello })))
        }
        Event::Lost { channel } => out.push(Request::Core(core::Event::Fleet(fleet::Event::Lost { channel }))),
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
        Event::Relayed { channel, run, attempt, call, answer } => {
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
            let assignment = link.take_assignment(run.raw()).expect("prepared claim has assignment");
            assert!(assignment.attempt == attempt.raw(), "current assignment fence");
            out.push(Request::Assigned { channel, assignment });
        }
        Event::Down { channel, value } => out.push(Request::Down { channel, value }),
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
    };
    work.push(Request::Core(result_event(domain, payload, end).expect("retained answer")));
}
pub fn result_event(domain: &Link, payload: Token, end: tasks::End) -> Option<core::Event> {
    let Some(Payload::Answer { task, attempt, cumulative, read, refused, .. }) = domain.payloads.get(&payload) else {
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
        read: *read,
        refused: refused.clone(),
    })
}
