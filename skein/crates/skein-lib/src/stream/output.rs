//! Independent output reservations beside a classic read (runtime/lib.md, 7.1).

use alloc::boxed::Box;

use crate::Token;

use super::Fault;

/// Requests to a native independent output face (runtime/lib.md, 7.1).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum OutputDown {
    /// A caller-unique local token and positive bytes within the lower cap.
    /// Only an open writable endpoint with idle output admits this right;
    /// classic room ownership cannot coexist. One `Settled` ends it (runtime/lib.md, 7.1).
    Room {
        /// Generated and never reused by the caller on this stream (runtime/lib.md, 7.1).
        right: Token,
        /// Positive, at most the lower output cap, with one Send slot (runtime/lib.md, 7.1).
        bytes: u32,
    },
    /// Cancel a pending right; an emitted terminal remains its winner (runtime/lib.md, 7.1).
    Cancel {
        /// The pending right; another or retired token is inert (runtime/lib.md, 7.1).
        right: Token,
    },
    /// Move one box within a matching grant, spending its whole reservation.
    /// An empty box also consumes the grant. A stale/no-grant Send is inert;
    /// a matching oversized box is the caller's bug, checked before effects (runtime/lib.md, 7.1).
    Send {
        /// The currently granted right, consumed once (runtime/lib.md, 7.1).
        right: Token,
        /// At most the bytes granted by the matching terminal (runtime/lib.md, 7.1).
        bytes: Box<[u8]>,
    },
    /// Consume an unused matching grant without moving any bytes (runtime/lib.md, 7.1).
    Release {
        /// The currently granted right; another or retired token is inert (runtime/lib.md, 7.1).
        right: Token,
    },
}

/// The native lower face's exactly-once terminal for an admitted output right (runtime/lib.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OutputUp {
    /// Generated only for an admitted right; it neither answers nor withdraws
    /// a classic read. A queued terminal remains the winner across Cancel (runtime/lib.md, 7.1).
    Settled {
        /// The request's exact locally generated identity (runtime/lib.md, 7.1).
        right: Token,
        /// Actual grant, cancellation or failure; never peer receipt (runtime/lib.md, 7.1).
        outcome: OutputOutcome,
    },
}

/// What consumed the output demand (runtime/lib.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OutputOutcome {
    /// Bytes and one Send slot are reserved; one matching Send, Release,
    /// Finish or actual close/failure retires the resulting affine grant (runtime/lib.md, 7.1).
    Granted,
    /// Actual cancellation before grant, including Close/Abort before Closed (runtime/lib.md, 7.1).
    Cancelled,
    /// Actual failure before grant, emitted before classic stream Failed (runtime/lib.md, 7.1).
    Failed(
        /// The actual lower failure, before classic Failed (runtime/lib.md, 7.1).
        Fault,
    ),
}
