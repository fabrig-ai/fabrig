//! Translate committed hub records (jig/docs/design/domain/hosts.md, 6.1).
use crate::Link;
use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::Token;
/// Connected channels receive the hub's vocabulary; other outputs stay with the root.
/// # Errors
/// Returns outputs for another root port without consuming them.
#[expect(clippy::result_large_err, reason = "an untranslated output returns to its root without another allocation")]
pub fn down(link: &mut Link, value: core::Held) -> Result<(Token, host::LinkDown), core::Held> {
    let (channel, value) = match value {
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
        core::Held::Refuse { channel } => {
            link.disconnect(channel);
            return Err(core::Held::Refuse { channel });
        }
        other @ (core::Held::Acted { .. }
        | core::Held::Drained
        | core::Held::Load(..)
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::Read { .. }
        | core::Held::PublicRead { .. }
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
        | core::Held::Result { .. }
        | core::Held::TaskTurnKept { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::StopRun { .. }
        | core::Held::NotesLoad { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. }
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
