//! Translate the core's hub outputs (jig/docs/design/domain/root.md, 5).
use crate::{Event, Link};
use jig_domain_core_vocabulary as core;
use jig_domain_host as host;
use skein_lib::Token;
/// Connected channels receive the hub's vocabulary; other outputs stay with the root.
/// # Errors
/// Returns outputs for another root port without consuming them.
#[expect(clippy::result_large_err, reason = "an untranslated output returns to its root without another allocation")]
pub fn down(link: &mut Link, value: core::Held) -> Result<(Token, host::LinkDown), core::Held> {
    let (channel, value) = match value {
        core::Held::ReadList { channel, read, endpoint, grant } if link.connected(channel) => {
            (channel, host::LinkDown::ReadList { read, endpoint, grant })
        }
        core::Held::Sources { channel, sources } if link.connected(channel) => {
            (channel, host::LinkDown::Sources { sources })
        }
        core::Held::CheckSource { channel, name } if link.connected(channel) => {
            (channel, host::LinkDown::Check { name })
        }
        core::Held::SignOutSource { channel, name } if link.connected(channel) => {
            (channel, host::LinkDown::SignOut { name })
        }
        core::Held::Acknowledge { channel, run, attempt } if link.connected(channel) => {
            (channel, host::LinkDown::Acknowledge { run, attempt })
        }
        core::Held::AcknowledgeTurn { channel, run, attempt, turn } if link.connected(channel) => {
            link.turn_backoffs.cancel((run, attempt, turn));
            (channel, host::LinkDown::TurnKept { run, attempt, turn })
        }
        core::Held::Cancel { channel, run, attempt } if link.connected(channel) => {
            (channel, host::LinkDown::Cancel { run, attempt })
        }
        core::Held::Grant { channel, run, attempt, grant } if link.connected(channel) => {
            (channel, host::LinkDown::Grant { run, attempt, grant })
        }
        core::Held::TaskTerminalAcknowledged { task, attempt } => {
            if let Some(job) = link.jobs.get(&task)
                && job.attempt == attempt
            {
                link.jobs.remove(&task);
            }
            return Err(core::Held::TaskTerminalAcknowledged { task, attempt });
        }
        core::Held::Refuse { channel, hello } => {
            link.disconnect(channel);
            return Err(core::Held::Refuse { channel, hello });
        }
        other @ (core::Held::Acted { .. }
        | core::Held::Drained
        | core::Held::Load(..)
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::Read { .. }
        | core::Held::PublicRead { .. }
        | core::Held::PublicWait { .. }
        | core::Held::Applied { .. }
        | core::Held::CarriedUnused { .. }
        | core::Held::Delivery { .. }
        | core::Held::WatchEnded { .. }
        | core::Held::MakeEffect { .. }
        | core::Held::SettledCall { .. }
        | core::Held::Assign { .. }
        | core::Held::SourceSignedOut { .. }
        | core::Held::Relayed { .. }
        | core::Held::Inbound { .. }
        | core::Held::Relay { .. }
        | core::Held::PeopleReply { .. }
        | core::Held::CallAnswer { .. }
        | core::Held::NeedArrived { .. }
        | core::Held::Result { .. }
        | core::Held::TaskTurnKept { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::StopRun { .. }
        | core::Held::NotesLoad { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. }
        | core::Held::ReadList { .. }
        | core::Held::Sources { .. }
        | core::Held::CheckSource { .. }
        | core::Held::SignOutSource { .. }
        | core::Held::Acknowledge { .. }
        | core::Held::AcknowledgeTurn { .. }
        | core::Held::Cancel { .. }
        | core::Held::Grant { .. }) => return Err(other),
    };
    Ok((channel, value))
}

/// The core's outputs for the link (jig/docs/design/domain/root.md, 5).
#[must_use]
pub fn now(value: core::LinkNow) -> Event {
    match value {
        core::LinkNow::Asked { channel, run, attempt, number, answer } => {
            Event::Asked { channel, run, attempt, number, answer }
        }
        core::LinkNow::Relayed { channel, run, attempt, call, answer } => {
            Event::Relayed { channel, run, attempt, call, answer }
        }
        core::LinkNow::DropPayload { payload } => Event::DropPayload { payload },
        core::LinkNow::TurnPayload { run, attempt, turn, body } => Event::TurnPayload { run, attempt, turn, body },
        core::LinkNow::AnswerPayload { run, attempt, payload } => Event::AnswerPayload { run, attempt, payload },
        core::LinkNow::AcceptedTurn { payload, task, attempt, turn, accepted } => {
            Event::AcceptedTurn { payload, task, attempt, turn, accepted }
        }
        core::LinkNow::RefusedPayload { request, problem } => Event::RefusedPayload { request, problem },
    }
}
