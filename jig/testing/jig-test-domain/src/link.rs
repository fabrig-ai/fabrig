//! Worker channels at the test root (domain/root.md, section 5).
//! Payloads stay owned here until the fleet asks for or drops them. The
//! protocol routing alone reads result bytes and encodes typed assignments.
use crate::{Delivery, Domain, Limits, Request};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decimal, Env, Queue, ReplyTo, Token, Wall};

pub(super) use jig_domain_hub_link::Ending;

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
/// Route opaque worker payloads through jig's link.
#[must_use]
pub fn up(domain: &mut Domain, channel: Token, up: host::LinkUp) -> Option<core::Event> {
    jig_domain_hub_link::up(&mut domain.hub_link, channel, up)
}
pub(super) fn receive(domain: &mut Domain, channel: Token, link: Event, work: &mut Queue<Work>) {
    let event = match link {
        Event::Hello { worker, hello: value } => {
            // The port is routing state; the fleet owns admission and its refusal.
            domain.hub_link.connect(channel);
            Some(hello(channel, worker, value))
        }
        Event::Up(value) => up(domain, channel, value),
        Event::Lost => {
            domain.hub_link.disconnect(channel);
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
        Delivery::Core(held) => {
            return match jig_domain_hub_link::down(&mut domain.hub_link, held) {
                Ok((channel, down)) => Delivery::Link { channel, down },
                Err(held) => Delivery::Core(held),
            };
        }
        Delivery::Message { channel, task, attempt, word } if domain.hub_link.connected(channel) => (
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
        Delivery::CallBusy { channel, task, attempt, name } if domain.hub_link.connected(channel) => (
            channel,
            host::LinkDown::Relayed {
                run: Token::new(task),
                attempt: Token::new(attempt),
                call: name,
                answer: host::Relayed::Busy,
            },
        ),
        Delivery::CallAnswer { channel, task, attempt, name, call, writes } if domain.hub_link.connected(channel) => {
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
        other @ (Delivery::Start(_)
        | Delivery::AssignedJob { .. }
        | Delivery::Link { .. }
        | Delivery::Message { .. }
        | Delivery::CallBusy { .. }
        | Delivery::CallAnswer { .. }
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

fn delivery(domain: &mut Domain, value: jig_domain_hub_link::Request, writes: bool) -> Delivery {
    use jig_domain_hub_link::Request as Output;
    let value = match value {
        Output::Core(core::Event::Fleet(event)) => Delivery::Fleet(event),
        Output::Assigned { channel, assignment } => Delivery::Assigned { channel, assignment },
        Output::AssignedJob { channel, job } => {
            let items = job.workspace;
            let workspace = if items.is_some() {
                Some(host::Workspace { workstream: job.workstream, items: Token::new(job.task) })
            } else {
                None
            };
            Delivery::AssignedJob {
                channel,
                items,
                job: host::JobAssignment {
                    run: Token::new(job.task),
                    attempt: Token::new(job.attempt),
                    kind: job.kind,
                    spec: job.spec,
                    deadline: job.deadline,
                    workspace,
                    grants: job.grants,
                },
            }
        }
        Output::Down { channel, value } => Delivery::Link { channel, down: value },
        Output::Message { channel, run, attempt, word } => {
            Delivery::Message { channel, task: run.raw(), attempt: attempt.raw(), word }
        }
        Output::Relayed { channel, run, attempt, call: name, answer } => match answer {
            Some(call) => Delivery::CallAnswer { channel, task: run.raw(), attempt: attempt.raw(), name, call, writes },
            None => Delivery::CallBusy { channel, task: run.raw(), attempt: attempt.raw(), name },
        },
        Output::Core(_) | Output::ReadResult { .. } | Output::ConnectorCall { .. } | Output::TurnCopy { .. } => {
            unreachable!("a delivery has no protocol or core hand-out")
        }
    };
    down(domain, value)
}
pub(super) fn held(
    domain: &mut Domain,
    now: skein_lib::Time,
    decision: &mut skein_lib::Decision<crate::Write, Delivery>,
    event: jig_domain_hub_link::Event,
) {
    let mut out = Queue::uncapped();
    jig_domain_hub_link::step(&mut domain.hub_link, now, event, &mut out);
    for _ in 0..out.len() {
        let value = delivery(domain, out.pop().expect("link output count"), true);
        crate::hold(decision, value);
    }
}
pub(super) fn route(
    domain: &mut Domain,
    now: skein_lib::Time,
    event: jig_domain_hub_link::Event,
    work: &mut Queue<Work>,
) {
    let mut out = Queue::uncapped();
    jig_domain_hub_link::step(&mut domain.hub_link, now, event, &mut out);
    for _ in 0..out.len() {
        match out.pop().expect("link output count") {
            jig_domain_hub_link::Request::Core(event) => work.push(Work::Core(event)),
            jig_domain_hub_link::Request::ReadResult { payload, outcome } => {
                domain.now.push(Request::ReadResult { payload, outcome });
            }
            jig_domain_hub_link::Request::ConnectorCall { to, run, attempt, call } => {
                domain.now.push(Request::Now(core::Now::Call { to, run, attempt, call }));
            }
            value @ (jig_domain_hub_link::Request::Message { .. }
            | jig_domain_hub_link::Request::Down { .. }
            | jig_domain_hub_link::Request::Assigned { .. }
            | jig_domain_hub_link::Request::AssignedJob { .. }
            | jig_domain_hub_link::Request::Relayed { .. }) => {
                let value = delivery(domain, value, false);
                domain.journal.now(value).expect("uncapped link delivery door");
            }
            jig_domain_hub_link::Request::TurnCopy { .. } => {
                unreachable!("the test root hosts its agents on worker channels")
            }
        }
    }
}
pub(super) fn answer(domain: &mut Domain, run: Token, attempt: Token, payload: Token, work: &mut Queue<Work>) {
    route(domain, skein_lib::Time::ZERO, jig_domain_hub_link::Event::AnswerPayload { run, attempt, payload }, work);
}
pub(super) fn read(domain: &mut Domain, env: &Env<Limits>, payload: Token, end: tasks::End) {
    if let Some(event) = jig_domain_hub_link::result_event(&domain.hub_link, payload, end) {
        crate::decide(domain, env, Work::Core(event));
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
