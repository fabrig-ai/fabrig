//! Content-free, losslessly retained host observations (domain/host.md, section 4;
//! skein's draft dynamic-memory.md, section 4.1); the parent drains them.
use crate::{End, Fault};
use skein_lib::Token;
/// Parent-drained diagnostic observation; no payload or credential detail.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A contained process started; independent from run admission.
    Started {
        /// Opaque parent owner.
        client: Token,
    },
    /// Run accepted its start.
    Admitted {
        /// Opaque parent owner.
        client: Token,
    },
    /// One agent last word was accepted.
    Answered {
        /// Opaque parent owner.
        client: Token,
    },
    /// One typed agent failure was reported.
    Faulted {
        /// Opaque parent owner.
        client: Token,
        /// Content-free failure class.
        fault: Fault,
    },
    /// Spawn containment terminal, after every retained right.
    Gone {
        /// Opaque parent owner.
        client: Token,
        /// Entrance refusal or completed cleanup.
        end: End,
    },
}
