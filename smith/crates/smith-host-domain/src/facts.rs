//! Content-free, losslessly retained host observations (domain/host.md, section 4;
//! skein/docs/foundation/programming-model.md, section 7.1); the parent drains them.
use crate::{End, Fault};
use skein_lib::Token;
/// Parent-drained diagnostic observation; no payload or credential detail.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A contained process started; independent from run admission.
    Started { client: Token },
    /// Run accepted its start.
    Admitted { client: Token },
    /// One agent last word was accepted.
    Answered { client: Token },
    /// One typed agent failure was reported.
    Faulted { client: Token, fault: Fault },
    /// Spawn containment terminal, after every retained right.
    Gone { client: Token, end: End },
}
