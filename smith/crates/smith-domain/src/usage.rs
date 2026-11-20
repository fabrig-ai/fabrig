//! Program allowance notices (protocol/programs.md, 5.6; protocol/channel.md, 4).
//! These are provider observations, not the run's token or spend ledger.

use alloc::boxed::Box;
use skein_lib::Duration;

/// One provider allowance window, reported for the account of its program.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UsageWindow {
    /// One to 32 printable ASCII bytes, as the provider named it.
    pub label: Box<[u8]>,
    pub length: Option<Duration>,
    /// Basis points, from zero through 10,000.
    pub used: Option<u16>,
    /// The provider's Unix seconds, not a remaining duration.
    pub resets: Option<u64>,
    pub state: UsageState,
}

/// The provider's admission state for this allowance window.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UsageState {
    /// The provider allows calls.
    Open,
    /// The provider allows calls but warns of the allowance.
    Warned,
    /// The provider rejects calls at this allowance.
    Limited,
}
