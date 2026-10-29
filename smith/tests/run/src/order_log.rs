//! Optional pass observations for scheduling orders (`smith/docs/design/domain/run.md`, 9.1 and 13).
//! This log retains boundary names, pass numbers and phase eligibility; it never
//! knows provider syntax or chooses when an entrance runs.

use skein_lib::{Time, Token};
use smith_domain_run::Event;

/// A relevant input at the run boundary, before the run takes ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// The host starts an unrelated activation.
    Start(Token),
    /// The host explicitly cancels an admitted activation.
    Cancel(Token),
    /// IO settles a check under its call owner.
    Checked(Token),
    /// A session supplies its one close terminal.
    Ended(Token),
    /// Another boundary event contributes to the input backlog.
    Other,
}

impl Input {
    pub(crate) fn of(event: &Event) -> Self {
        match event {
            Event::Start { host_run, .. } => Self::Start(*host_run),
            Event::Cancel { run } => Self::Cancel(*run),
            Event::Checked { owner, .. } => Self::Checked(*owner),
            Event::Ended { conversation, .. } => Self::Ended(*conversation),
            Event::HostReturned { .. }
            | Event::Message { .. }
            | Event::Acknowledge { .. }
            | Event::Turn { .. }
            | Event::JobsCancelled { .. }
            | Event::Started { .. }
            | Event::Yielded { .. }
            | Event::Priced { .. }
            | Event::Used { .. }
            | Event::Read { .. }
            | Event::Probed { .. }
            | Event::Delegated { .. }
            | Event::Withdraw { .. }
            | Event::Aborted { .. }
            | Event::Delivered { .. } => Self::Other,
        }
    }
}

/// An observation made by the world's existing scheduler or boundary dispatcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    /// Input present before the pass, with deferred work already eligible.
    Queued { inputs: Vec<Input>, ready: bool },
    /// An input enters the real run step.
    Taken(Input),
    /// Input left after the input phase, with alarm and resume eligibility.
    Pending { inputs: Vec<Input>, due: bool, ready: bool },
    /// The scheduler enters the real alarm function.
    Alarm,
    /// The scheduler enters the real deferred-work function.
    Resume,
    /// A session close crosses the output boundary, naming its owner and role.
    Close { run: Token, conversation: Token, main: bool },
    /// Main is asked to yield for its final reply.
    Yield { run: Token },
    /// Main starts a final provider request after child settlement.
    Final { run: Token, live_children: u32 },
    /// The world crosses its reclaim point.
    Reclaim,
}

/// A scheduling observation, at the world's injected time and numbered pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub pass: u64,
    pub at: Time,
    pub seen: Seen,
}
