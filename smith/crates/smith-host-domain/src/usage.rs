//! Account allowance observations (protocol/channel.md, 4; protocol/host.md, 2).
//! Provider resets are absolute wall times and are never measured on receipt.

use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

/// One provider allowance window, sent by the agent for a known account.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UsageWindow {
    /// One to 32 printable ASCII bytes, as the provider named it.
    pub label: Box<[u8]>,
    pub length: Option<Duration>,
    /// Basis points, from zero through 10,000.
    pub used: Option<u16>,
    pub resets: Option<Wall>,
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
