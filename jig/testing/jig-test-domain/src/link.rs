//! Worker channels at the test root (domain/root.md, section 5).
//! Payloads stay owned here until the fleet asks for or drops them. The
//! protocol routing alone reads result bytes and encodes typed assignments.
use crate::{Delivery, Domain, Limits, Payload, Request};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decimal, Env, List, Queue, ReplyTo, Token, Wall};

/// A scripted terminal is already read; a channel terminal is opaque.
#[derive(Debug)]
pub(super) enum Ending {
    Scripted(tasks::End),
    Link(host::Ending),
}

/// The three inputs of a worker channel, in arrival order.
#[derive(Debug)]
pub enum Event {
    Hello { worker: u32, hello: host::Hello },
    Up(host::LinkUp),
    Lost,
}

/// Translate a worker's complete admission report.
#[must_use]
pub fn hello(channel: Token, worker: u32, hello: host::Hello) -> core::Event {
    core::Event::Fleet(fleet::Event::Hello { channel, worker, hello })
}

/// A channel loss goes through the fleet's grace and fences.
#[must_use]
pub fn lost(channel: Token) -> core::Event {
    core::Event::Fleet(fleet::Event::Lost { channel })
}
/// Retain a worker's bytes and translate only their boundary metadata.
#[must_use]
pub fn up(domain: &mut Domain, channel: Token, up: host::LinkUp) -> Option<core::Event> {
    let event = match up {
        host::LinkUp::Piece { run, attempt, piece } => return live_piece(domain, run, attempt, piece),
        host::LinkUp::Logins { logins } => fleet::Event::Logins { channel, logins },
        host::LinkUp::SignedOut { name, result } => fleet::Event::SignedOut { channel, name, result },
        host::LinkUp::Turn { run, attempt, turn } => {
            let number = turn.turn;
            let body = crate::payload(
                domain,
                Payload::Turn {
                    task: run.raw(),
                    attempt: attempt.raw(),
                    turn: number,
                    cumulative: turn.spent,
                    read: read_name(turn.read),
                    transcript: turn.body,
                    opens_window: turn.opens_window,
                    refused: refusals(turn.refused),
                },
            );
            fleet::Event::Turn { channel, run, attempt, turn: number, body }
        }
        host::LinkUp::Answer { run, attempt, answer } => {
            let kind = match answer.ending {
                host::Ending::Ended { .. } => fleet::Answer::Ended,
                host::Ending::Parked { .. } => fleet::Answer::Parked,
                host::Ending::Failed { failure, .. } => fleet::Answer::Failed { turned: turned(failure) },
                host::Ending::Refused(host::Refusal::Busy) => fleet::Answer::Busy,
                host::Ending::Refused(host::Refusal::Invalid(_)) => fleet::Answer::Invalid,
            };
            let payload = crate::payload(
                domain,
                Payload::Answer {
                    task: run.raw(),
                    attempt: attempt.raw(),
                    cumulative: answer.spent,
                    read: read_name(answer.read),
                    refused: refusals(answer.refused),
                    end: Ending::Link(answer.ending),
                },
            );
            fleet::Event::Answer { channel, run, attempt, answer: kind, payload }
        }
        host::LinkUp::Call { run, attempt, call, tool, writes, input, deadline } => fleet::Event::Relay {
            channel,
            run,
            attempt,
            call: fleet::Call { name: call, tool, writes, input, deadline },
        },
        host::LinkUp::Unauthorized { run, attempt, account, generation } => {
            fleet::Event::Unauthorized { channel, run, attempt, account, generation }
        }
        host::LinkUp::Limited { run, attempt, account, reset } => {
            fleet::Event::Limited { channel, run, attempt, account, reset }
        }
        host::LinkUp::AccountRefused { run, attempt, account, code, reason } => {
            fleet::Event::AccountRefused { channel, run, attempt, account, code, reason }
        }
        host::LinkUp::Activity { run, attempt, activity } => {
            fleet::Event::Observe { channel, run, attempt, report: report(activity) }
        }
    };
    Some(core::Event::Fleet(event))
}
/// The account a failed run turned on, as the fleet carries it.
const fn turned(failure: host::Failure) -> Option<fleet::Turned> {
    match failure {
        host::Failure::Run(host::RunFailure::Limited { account, reset }) => {
            Some(fleet::Turned::Limited { account, reset })
        }
        host::Failure::Run(host::RunFailure::Refused { account }) => Some(fleet::Turned::Refused { account }),
        host::Failure::Run(
            host::RunFailure::Exceeded(_)
            | host::RunFailure::Model
            | host::RunFailure::Budget
            | host::RunFailure::Policy
            | host::RunFailure::Cancelled
            | host::RunFailure::Stale,
        )
        | host::Failure::Unprepared(_)
        | host::Failure::Agent(_)
        | host::Failure::Cancelled(_) => None,
    }
}
fn read_name(name: Option<Token>) -> Option<u64> {
    Some(name?.raw())
}
fn refusals(values: Box<[host::Refused]>) -> Box<[u64]> {
    let mut names = List::with_capacity(u32::try_from(values.len()).expect("bounded refusals"));
    for value in values {
        names.push(value.name.raw()).expect("one slot per refusal");
    }
    names.into_boxed()
}

pub(super) fn receive(domain: &mut Domain, channel: Token, link: Event, work: &mut Queue<Work>) {
    let event = match link {
        Event::Hello { worker, hello: value } => {
            // The port is routing state; the fleet owns admission and its refusal.
            let _inserted = domain.links.insert(channel);
            Some(hello(channel, worker, value))
        }
        Event::Up(value) => up(domain, channel, value),
        Event::Lost => {
            let _removed = domain.links.remove(&channel);
            Some(lost(channel))
        }
    };
    if let Some(event) = event {
        work.push(Work::Core(event));
    }
}

/// Translate complete channel deliveries; typed charters stay handed out.
/// The scripted worker port continues to receive its original vocabulary.
#[must_use]
pub fn down(domain: &mut Domain, delivery: Delivery) -> Delivery {
    let (channel, down) = match delivery {
        Delivery::Core(core::Held::Sources { channel, sources }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::Sources { sources })
        }
        Delivery::Core(core::Held::CheckSource { channel, name }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::Check { name })
        }
        Delivery::Core(core::Held::SignOutSource { channel, name }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::SignOut { name })
        }
        Delivery::Message { channel, task, attempt, word } if domain.links.contains(&channel) => (
            channel,
            host::LinkDown::Message {
                run: Token::new(task),
                attempt: Token::new(attempt),
                name: Token::new(word.number),
                sender: sender(word.from),
                finish_requested: word.finish_requested(),
                words: jig_protocol_tools::words(&word),
            },
        ),
        Delivery::CallBusy { channel, task, attempt, name } if domain.links.contains(&channel) => (
            channel,
            host::LinkDown::Relayed {
                run: Token::new(task),
                attempt: Token::new(attempt),
                call: name,
                answer: host::Relayed::Busy,
            },
        ),
        Delivery::CallAnswer { channel, task, attempt, name, call, writes } if domain.links.contains(&channel) => {
            match call.answer {
                core::SettledAnswer::Host { error, body } => (
                    channel,
                    host::LinkDown::Relayed {
                        run: Token::new(task),
                        attempt: Token::new(attempt),
                        call: name,
                        answer: host::Relayed::Answer { error, text: body },
                    },
                ),
                core::SettledAnswer::Delivery { .. } => {
                    return Delivery::CallAnswer { channel, task, attempt, name, call, writes };
                }
            }
        }
        Delivery::Core(core::Held::Acknowledge { channel, run, attempt }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::Acknowledge { run, attempt })
        }
        Delivery::Core(core::Held::AcknowledgeTurn { channel, run, attempt, turn })
            if domain.links.contains(&channel) =>
        {
            (channel, host::LinkDown::TurnKept { run, attempt, turn })
        }
        Delivery::Core(core::Held::Cancel { channel, run, attempt }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::Cancel { run, attempt })
        }
        Delivery::Core(core::Held::Grant { channel, run, attempt, grant }) if domain.links.contains(&channel) => {
            (channel, host::LinkDown::Grant { run, attempt, grant })
        }
        Delivery::Core(core::Held::Refuse { channel }) => {
            let _removed = domain.links.remove(&channel);
            return Delivery::Core(core::Held::Refuse { channel });
        }
        other @ (Delivery::Start(_)
        | Delivery::Link { .. }
        | Delivery::Message { .. }
        | Delivery::CallBusy { .. }
        | Delivery::CallAnswer { .. }
        | Delivery::Core(_)
        | Delivery::Assigned { .. }
        | Delivery::Fleet(_)
        | Delivery::ApplicationDocument { .. }
        | Delivery::ApplicationRead { .. }
        | Delivery::Application { .. }
        | Delivery::ProjectMade { .. }
        | Delivery::Projection { .. }
        | Delivery::System { .. }
        | Delivery::LostRead { .. }
        | Delivery::Procedure { .. }) => return other,
    };
    Delivery::Link { channel, down }
}

pub(super) fn answer(domain: &mut Domain, run: Token, attempt: Token, payload: Token, work: &mut Queue<Work>) {
    let Some(Payload::Answer { task, attempt: retained, end, .. }) = domain.payloads.get(&payload) else {
        unreachable!("fleet retained the answer body");
    };
    assert!(*task == run.raw() && *retained == attempt.raw(), "fenced answer identity");
    let end = match end {
        Ending::Link(host::Ending::Ended { outcome, .. }) => {
            domain.now.push(Request::ReadResult { payload, outcome: outcome.clone() });
            return;
        }
        Ending::Link(
            other @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }),
        ) => ending(other),
        Ending::Scripted(end) => end.clone(),
    };
    work.push(Work::Core(result_event(domain, payload, end).expect("retained answer")));
}
pub(super) fn read(domain: &mut Domain, env: &Env<Limits>, payload: Token, end: tasks::End) {
    if let Some(event) = result_event(domain, payload, end) {
        crate::decide(domain, env, Work::Core(event));
    }
}
pub(super) fn result_event(domain: &Domain, payload: Token, end: tasks::End) -> Option<core::Event> {
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
fn ending(ending: &host::Ending) -> tasks::End {
    match ending {
        host::Ending::Refused(host::Refusal::Busy) => tasks::End::Refused,
        host::Ending::Refused(host::Refusal::Invalid(_)) => tasks::End::Failed(tasks::Class::Invalid),
        host::Ending::Parked { .. } => tasks::End::Parked,
        host::Ending::Failed { failure, .. } => match failure {
            host::Failure::Run(host::RunFailure::Exceeded(value)) => tasks::End::Exceeded(tasks::Exceeded {
                bound: match value.bound {
                    host::Bound::Reasoning => tasks::Bound::Reasoning,
                    host::Bound::Metadata => tasks::Bound::Metadata,
                    host::Bound::Items => tasks::Bound::Items,
                    host::Bound::Answer => tasks::Bound::Answer,
                    host::Bound::Tools => tasks::Bound::Tools,
                    host::Bound::HistoryItems => tasks::Bound::HistoryItems,
                    host::Bound::History => tasks::Bound::History,
                    host::Bound::Window => tasks::Bound::Window,
                    host::Bound::Output => tasks::Bound::Output,
                },
                value: value.value,
            }),
            host::Failure::Unprepared(host::Preparation::Transient) => tasks::End::Failed(tasks::Class::Transient),
            host::Failure::Unprepared(host::Preparation::Permanent { .. }) => {
                tasks::End::Failed(tasks::Class::Permanent)
            }
            host::Failure::Run(
                host::RunFailure::Model
                | host::RunFailure::Budget
                | host::RunFailure::Policy
                | host::RunFailure::Cancelled
                | host::RunFailure::Stale
                | host::RunFailure::Limited { .. }
                | host::RunFailure::Refused { .. },
            ) => tasks::End::Failed(tasks::Class::Run),
            host::Failure::Agent(_) | host::Failure::Cancelled(_) => tasks::End::Failed(tasks::Class::Agent),
        },
        host::Ending::Ended { .. } => unreachable!("ended payloads leave for the routing"),
    }
}

fn report(activity: host::AgentActivity) -> fleet::Report {
    match activity {
        host::AgentActivity::Phase(phase) => fleet::Report::Activity(match phase {
            host::Phase::Preparing => fleet::Activity::Preparing,
            host::Phase::Starting => fleet::Activity::Starting,
            host::Phase::Active => fleet::Activity::Active,
            host::Phase::Waiting => fleet::Activity::Waiting,
            host::Phase::Ending | host::Phase::Answered => fleet::Activity::Ending,
        }),
        host::AgentActivity::Long { operation, span } => {
            fleet::Report::Activity(fleet::Activity::Long { operation, span })
        }
        host::AgentActivity::LongDone => fleet::Report::Activity(fleet::Activity::Active),
        host::AgentActivity::Count { kind, elapsed, count } => fleet::Report::Count { kind, elapsed, count },
    }
}

/// Preserve the task-owned message kind when translating its opaque words.
fn sender(value: tasks::Party) -> Box<[u8]> {
    match value {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
}

#[expect(clippy::large_enum_variant, reason = "the bounded route queue owns complete core events")]
pub(super) enum Work {
    Link {
        channel: Token,
        link: Event,
    },
    WatchApplication {
        watcher: Token,
        sign_in: u64,
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        subject: connector::ClientSubject,
        window: core::documents::Window,
    },
    ReadApplication {
        reply: ReplyTo,
        sign_in: u64,
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        subject: connector::ClientSubject,
        window: core::documents::Window,
    },
    Application {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        scope: jig_domain_people::ApplicationScope,
        number: u16,
        digest: [u8; 32],
        value: connector::ApplicationRequest,
        carried: Option<core::Carried>,
    },
    ProjectionEffect {
        goal: u64,
        number: u16,
        effect: connector::Effect,
    },
    ConnectorFire {
        number: u16,
    },
    ConnectorResume {
        number: u16,
    },
    Start(crate::StartInput),
    ConnectorRestartDone {
        number: u16,
    },
    CallAnswered {
        to: ReplyTo,
        call: core::SettledCall,
    },
    WriterRead {
        connector: u16,
        resource: tasks::Name,
        event: connector::SystemEvent,
    },
    Timer(core::Timer),
    Core(core::Event),
    ResumeFleet,
    Finish,
    Connector {
        number: u16,
        event: connector::Event,
    },
    TopicCall {
        to: ReplyTo,
        key: core::CallKey,
        number: u16,
        topic: u16,
    },
    EffectCall {
        to: ReplyTo,
        key: core::CallKey,
        number: u16,
        effect: connector::Effect,
        deadline: Wall,
        proposal: Option<Box<[u8]>>,
    },
    Turn {
        channel: Token,
        task: u64,
        attempt: u64,
        turn: u32,
        cumulative: u64,
        read: Option<u64>,
        opens_window: bool,
        refused: Box<[u64]>,
        transcript: Box<[u8]>,
    },
    Answer {
        channel: Token,
        task: u64,
        attempt: u64,
        cumulative: u64,
        end: tasks::End,
        read: Option<u64>,
        refused: Box<[u64]>,
    },
}

fn live_piece(domain: &mut Domain, run: Token, attempt: Token, piece: host::Piece) -> Option<core::Event> {
    if let Ok(offset) = u32::try_from(piece.offset) {
        match piece.kind {
            host::LiveKind::Text => {
                return Some(core::Event::RunText {
                    run,
                    attempt,
                    turn: piece.turn,
                    retry: piece.retry,
                    offset,
                    text: piece.bytes,
                    call: None,
                });
            }
            host::LiveKind::Input { call, tool } => {
                return Some(core::Event::RunInput {
                    run,
                    attempt,
                    turn: piece.turn,
                    retry: piece.retry,
                    offset,
                    id: call,
                    tool,
                    input: piece.bytes,
                });
            }
            host::LiveKind::Reasoning => {}
        }
    }
    domain.pieces_dropped = domain.pieces_dropped.saturating_add(1);
    None
}
