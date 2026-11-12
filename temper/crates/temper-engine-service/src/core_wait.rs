//! A gate keeping parties' inputs from the root (temper/docs/design/protocol/README.md, 6.2).
use alloc::boxed::Box;
use jig_domain_core::start::Phase;
use skein_lib::{Decimal, Time, bytes};

/// Why the gate keeps a party's input from the root (protocol/README.md, 6.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoreWait {
    Starting { step: Option<Phase> },
    ForgeRecords,
    FirstPeriods,
    Resolving,
    Store { oldest: u64, in_flight: u64 },
    Watermark { backlog: u64, watermark: u32 },
}
impl CoreWait {
    /// The bounded, payload-free reason following the core's not-answering line.
    #[must_use]
    pub fn words(self) -> Box<[u8]> {
        match self {
            Self::Starting { step } => bytes::concat(&[b"starting: ", starting(step)]),
            Self::ForgeRecords => b"loading the forges' records".as_slice().into(),
            Self::FirstPeriods => b"opening the first periods".as_slice().into(),
            Self::Resolving => b"resolving the forges' names".as_slice().into(),
            Self::Store { oldest, in_flight } => bytes::concat(&[
                b"waiting for the store to make commit ",
                Decimal::of(oldest).as_bytes(),
                b" durable, the oldest of ",
                Decimal::of(in_flight).as_bytes(),
                b" in flight",
            ]),
            Self::Watermark { backlog, watermark } => bytes::concat(&[
                b"at its watermark: ",
                Decimal::of(backlog).as_bytes(),
                b" of ",
                Decimal::of(u64::from(watermark)).as_bytes(),
                b" in flight",
            ]),
        }
    }
}
fn starting(step: Option<Phase>) -> &'static [u8] {
    match step {
        None | Some(Phase::LoadCore) => b"loading the core's records",
        Some(Phase::LoadInbox) => b"loading the inbox",
        Some(Phase::RestoreConnector { .. }) => b"loading the forge's records",
        Some(Phase::Link) => b"linking the forge's records",
        Some(Phase::Sweep) => b"sweeping",
        Some(Phase::Switch) => b"switching the store",
        Some(Phase::Wires) => b"committing the wires",
        Some(Phase::AdoptRuns) => b"adopting runs",
        Some(Phase::ReadAfresh { .. }) => b"reading the forge afresh",
        Some(Phase::SettleOutbox { .. }) => b"settling the forge's outbox",
        Some(Phase::Open) => b"opening decisions",
    }
}

/// One reason held; a line when it has kept inputs 1 s, again on change,
/// and once when the gate passes again.
pub(crate) struct Watch {
    since: Option<Time>,
    written: Option<CoreWait>,
}
impl Watch {
    pub(crate) const fn new() -> Self {
        Self { since: None, written: None }
    }
    pub(crate) fn kept(&mut self, now: Time, why: CoreWait) -> Option<crate::Note> {
        let since = *self.since.get_or_insert(now);
        if now.saturating_since(since) < crate::long_pass::LONG_PASS || self.written == Some(why) {
            return None;
        }
        self.written = Some(why);
        Some(crate::Note::CoreWaiting { why })
    }
    pub(crate) fn passed(&mut self) -> Option<crate::Note> {
        self.since = None;
        match self.written.take() {
            Some(_) => Some(crate::Note::CoreAnswers),
            None => None,
        }
    }
}
