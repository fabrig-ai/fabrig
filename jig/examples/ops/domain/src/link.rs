//! Route the engine's permanent hub link through the journal (domain/root.md, 5, 12).
//! Ops translates its charter and agent content; jig owns the fleet relay.
use crate::domain::{Work, child, now};
use crate::{Domain, Limits, Output, Released, Write};
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_hub_link as link;
use jig_domain_link::Hosting;
use skein_lib::{Decision, Env, Queue, Token};

// The engine's hub connects directly to the fleet at construction and is never lost
// (jig/docs/design/domain/hosts/hub.md, 1), so every hand-off uses this one channel.
const ENGINE: Token = Token::new(0);

fn permanent(channel: Token) {
    assert!(channel == ENGINE, "ops engine's permanent link");
}

pub(crate) fn route(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    event: link::Event,
    held: bool,
    work: &mut Queue<Work>,
) {
    let mut out = Queue::uncapped();
    link::step(&mut domain.hub_link, env.now, event, &mut out);
    outputs(domain, env, decision, &mut out, held, work);
}

fn outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    out: &mut Queue<link::Request>,
    held: bool,
    work: &mut Queue<Work>,
) {
    // A link input emits a finite set of hand-offs, drained before the next entrance.
    for _ in 0..out.len() {
        let released = match out.pop().expect("link output count") {
            link::Request::Core(event) => {
                if held {
                    child(decision, Released::Core(event));
                } else {
                    work.push(Work::Core(event));
                }
                continue;
            }
            link::Request::Down { channel, value } => {
                permanent(channel);
                Released::Host(host::Event::Down(value))
            }
            link::Request::Assigned { channel, assignment } => {
                permanent(channel);
                let assignment = crate::assemble::assignment(domain, env, &assignment).expect("validated charter");
                Released::Host(host::Event::Down(host::LinkDown::Assign { assignment }))
            }
            link::Request::AssignedJob { channel, job } => {
                permanent(channel);
                Released::Host(host::Event::Down(host::LinkDown::AssignJob { job: link::job_assignment(job) }))
            }
            link::Request::ReadResult { payload, outcome } => {
                now(domain, Output::ReadResult { payload, outcome });
                continue;
            }
            link::Request::Relayed { channel, run, attempt, call, answer } => {
                permanent(channel);
                let answer = match answer {
                    Some(call) => link::settled_answer(call.answer).expect("engine charter has no delivery tool"),
                    None => host::Relayed::Busy,
                };
                Released::Host(host::Event::Down(host::LinkDown::Relayed { run, attempt, call, answer }))
            }
            link::Request::Message { channel, run, attempt, word } => {
                permanent(channel);
                Released::Host(host::Event::Down(jig_protocol_charter::message(run, attempt, &word)))
            }
            link::Request::ConnectorCall { to, run, attempt, call } => {
                now(domain, Output::Now(jig_domain_core::Now::Call { to, run, attempt, call }));
                continue;
            }
            link::Request::TurnCopy { run, attempt, turn } => {
                if let Some(turn) = domain.host.turn(run, attempt, turn)
                    && let Some(event) =
                        link::up(&mut domain.hub_link, ENGINE, host::LinkUp::Turn { run, attempt, turn })
                {
                    work.push(Work::Core(event));
                }
                continue;
            }
        };
        if held {
            child(decision, released);
        } else {
            now(domain, Output::ToChild(released));
        }
    }
}

pub(crate) fn fire(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    work: &mut Queue<Work>,
) {
    let mut out = Queue::uncapped();
    link::fire(&mut domain.hub_link, env.now, &mut out);
    outputs(domain, env, decision, &mut out, false, work);
}

pub(crate) fn host_out(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    out: &mut Queue<host::Request>,
    work: &mut Queue<Work>,
) {
    for _ in 0..out.len() {
        match out.pop().expect("host requests") {
            host::Request::Job(job) => work.push(Work::Host(host::Event::Job(job.refused()))),
            host::Request::DeliverJob { owner, .. } => work.push(Work::Host(host::Event::Delivered {
                owner,
                delivery: host::Delivery { outcome: host::DeliveryOutcome::Refused, changed: false, left: owner },
            })),
            host::Request::Agent(request) => crate::domain::held(decision, Output::Agent(request)),
            host::Request::Up(value) => {
                route(domain, env, decision, link::Event::Up { channel: ENGINE, value }, false, work);
            }
            host::Request::CancelRelay { run, attempt, call } => {
                domain.hub_link.forget_call(&call);
                work.push(Work::Host(host::Event::RelayCancelled { run, attempt, call }));
            }
            // This root never asks and configures its engine host directly.
            host::Request::Asked { .. } | host::Request::Hello { .. } => {}
            host::Request::Prepare { owner, .. } => work.push(Work::Host(host::Event::Unprepared {
                owner,
                failure: host::Preparation::Permanent { resource: None },
                detail: Box::new([]),
            })),
            host::Request::ReadList { .. } | host::Request::CancelRead { .. } => {
                unreachable!("the host answers list reads without a machine")
            }
            host::Request::Source { .. } | host::Request::SourceFault => {
                unreachable!("engine hub has no program sources")
            }
            host::Request::DeliverUnchecked { .. }
            | host::Request::Deliver { .. }
            | host::Request::Abort { .. }
            | host::Request::Save { .. }
            | host::Request::Release { .. } => unreachable!("engine hub has no workspace capability"),
        }
    }
    for _ in 0..env.limits.host.told {
        let Some(value) = domain.host.pop_activity() else { break };
        route(domain, env, decision, link::Event::Up { channel: ENGINE, value }, false, work);
    }
}

/// Translate the application's agent stream to jig's live-piece vocabulary.
pub(crate) fn agent_content(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    work: &mut Queue<Work>,
    owner: Token,
    content: crate::AgentContent,
) {
    let Some(hosting) = domain.host.hosting(owner) else { return };
    match content {
        crate::AgentContent::Text { turn, retry, mut offset, text } => {
            // The stream adapter splits at the receiving views' byte bound.
            let bound = skein_lib::length::usize_of(env.limits.core.views.text_bytes);
            if text.is_empty() {
                let event = piece(hosting, turn, retry, offset, text);
                route(domain, env, decision, event, false, work);
            } else {
                for chunk in text.chunks(bound) {
                    let event = piece(hosting, turn, retry, offset, skein_lib::bytes::copy_of(chunk));
                    route(domain, env, decision, event, false, work);
                    offset = offset
                        .checked_add(u32::try_from(chunk.len()).expect("stream piece byte limit"))
                        .expect("supplied text position");
                }
            }
        }
        crate::AgentContent::Call(call) => {
            work.push(Work::Core(jig_domain_core::Event::RunCall { run: hosting.run, attempt: hosting.attempt, call }));
        }
    }
}

fn piece(hosting: Hosting, turn: u32, retry: u32, offset: u32, bytes: Box<[u8]>) -> link::Event {
    link::Event::Up {
        channel: ENGINE,
        value: host::LinkUp::Piece {
            run: hosting.run,
            attempt: hosting.attempt,
            piece: host::Piece { turn, retry, offset: u64::from(offset), kind: host::LiveKind::Text, bytes },
        },
    }
}
