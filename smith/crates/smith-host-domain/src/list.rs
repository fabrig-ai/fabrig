//! List answers returned by the host's half (smith/docs/design/protocol/hosts.md, 2).
//! Keeps only model facts and typed failures; knows no credential or child I/O.

use alloc::boxed::Box;

/// One endpoint model, returned by the half to its host after a list read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedModel {
    pub name: Box<[u8]>,
    pub shown: Option<Box<[u8]>>,
    pub window: Option<u32>,
    pub output: Option<u32>,
    /// Supported efforts, ordered from low to high.
    pub efforts: Box<[Box<[u8]>]>,
}

/// Why the half could not read the endpoint's list, with no provider error body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListFailure {
    /// The provider refused the request with this HTTP status alone.
    Refused { status: u16 },
    /// The agent could not reach the provider.
    Unreachable,
    /// The provider's list could not be read within its bounds.
    Unreadable,
    /// The host's or agent's deadline expired, and the child has settled.
    Deadline,
    /// The agent could not start or return a bounded, readable answer successfully.
    Agent,
}

/// The half's single settled answer to its host's list request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListOutcome {
    /// All rows read from the endpoint, preserving their fields and order.
    Listed(Box<[ListedModel]>),
    /// A list read that failed without returning provider error details.
    Failed(ListFailure),
}
