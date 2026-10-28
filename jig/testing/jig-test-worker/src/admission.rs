//! Worker admission and its dependency predicates (skein/docs/foundation/programming-model.md, 7.1–7.2;
//! jig/docs/design/domain/hosts.md, 4).
use crate::{Domain, Event};
use jig_domain_host::LinkDown;
use skein_lib::Token;
use smith_host_domain::{Up, process};

/// How the worker takes an event (skein's programming-model.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Finishes or advances admitted work: taken at any backlog.
    Admitted,
    /// New work: waits in the service's queue while the backlog is at the watermark.
    New,
}

/// Classes an event by its variant and payload alone.
#[must_use]
pub fn class(event: &Event) -> Class {
    match event {
        Event::Link(link) => match link {
            LinkDown::Sources { .. }
            | LinkDown::Check { .. }
            | LinkDown::SignOut { .. }
            | LinkDown::Assign { .. }
            | LinkDown::Message { .. } => Class::New,
            LinkDown::Relayed { .. }
            | LinkDown::TurnKept { .. }
            | LinkDown::Acknowledge { .. }
            | LinkDown::Cancel { .. }
            | LinkDown::Grant { .. } => Class::Admitted,
        },
        Event::Process(event) => match event.as_ref() {
            process::Event::Received { message, .. } => match message {
                Up::Call { .. } => Class::New,
                Up::Admitted
                | Up::MessageRefused { .. }
                | Up::Answer { .. }
                | Up::Turn { .. }
                | Up::Waiting { .. }
                | Up::Long { .. }
                | Up::LongDone
                | Up::Fact { .. }
                | Up::Unauthorized { .. }
                | Up::Limited { .. }
                | Up::Refused { .. }
                | Up::Withdraw { .. } => Class::Admitted,
            },
            process::Event::Spawned { .. }
            | process::Event::Unspawned { .. }
            | process::Event::Sent { .. }
            | process::Event::Unsent { .. }
            | process::Event::Progress { .. }
            | process::Event::Malformed { .. }
            | process::Event::Hangup { .. }
            | process::Event::Signalled { .. }
            | process::Event::Exited { .. }
            | process::Event::Reaped { .. } => Class::Admitted,
        },
        Event::Checks => Class::New,
        Event::Ready
        | Event::Lost
        | Event::Source(_)
        | Event::GracePassed
        | Event::Resend { .. }
        | Event::RelayCancelled { .. }
        | Event::Live { .. }
        | Event::Decoded { .. }
        | Event::Encoded { .. }
        | Event::Activity { .. }
        | Event::Shutdown
        | Event::Timer => Class::Admitted,
    }
}

pub(crate) fn must_follow(domain: &Domain, held: &Event, event: &Event) -> bool {
    match held {
        Event::Link(LinkDown::Assign { assignment }) => match event {
            Event::Link(link) => {
                named_assignment(link) == Some((assignment.assignment.run, assignment.assignment.attempt))
            }
            Event::Ready
            | Event::Lost
            | Event::Source(_)
            | Event::GracePassed
            | Event::Resend { .. }
            | Event::RelayCancelled { .. }
            | Event::Process(_)
            | Event::Live { .. }
            | Event::Decoded { .. }
            | Event::Encoded { .. }
            | Event::Activity { .. }
            | Event::Shutdown
            | Event::Timer
            | Event::Checks => false,
        },
        Event::Process(held) => {
            let agent = match event {
                Event::Process(process) => Some(process_owner(process)),
                Event::Live { agent, .. } => Some(*agent),
                // Payload hand-outs echo the hub's owner, not Smith's process token.
                Event::Decoded { agent: owner, .. } | Event::Encoded { owner, .. } | Event::Activity { owner, .. } => {
                    domain.process_agent(*owner)
                }
                Event::Ready
                | Event::Lost
                | Event::Source(_)
                | Event::Link(_)
                | Event::GracePassed
                | Event::Resend { .. }
                | Event::RelayCancelled { .. }
                | Event::Shutdown
                | Event::Timer
                | Event::Checks => None,
            };
            agent == Some(process_owner(held))
        }
        Event::Link(
            LinkDown::Sources { .. }
            | LinkDown::Check { .. }
            | LinkDown::SignOut { .. }
            | LinkDown::Message { .. }
            | LinkDown::Relayed { .. }
            | LinkDown::TurnKept { .. }
            | LinkDown::Acknowledge { .. }
            | LinkDown::Cancel { .. }
            | LinkDown::Grant { .. },
        )
        | Event::Ready
        | Event::Lost
        | Event::Source(_)
        | Event::GracePassed
        | Event::Resend { .. }
        | Event::RelayCancelled { .. }
        | Event::Live { .. }
        | Event::Decoded { .. }
        | Event::Encoded { .. }
        | Event::Activity { .. }
        | Event::Shutdown
        | Event::Timer
        | Event::Checks => false,
    }
}
fn named_assignment(link: &LinkDown) -> Option<(Token, Token)> {
    match link {
        LinkDown::Cancel { run, attempt }
        | LinkDown::Relayed { run, attempt, .. }
        | LinkDown::TurnKept { run, attempt, .. }
        | LinkDown::Acknowledge { run, attempt }
        | LinkDown::Grant { run, attempt, .. } => Some((*run, *attempt)),
        LinkDown::Sources { .. }
        | LinkDown::Check { .. }
        | LinkDown::SignOut { .. }
        | LinkDown::Assign { .. }
        | LinkDown::Message { .. } => None,
    }
}
fn process_owner(event: &process::Event) -> Token {
    match event {
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
    }
}
