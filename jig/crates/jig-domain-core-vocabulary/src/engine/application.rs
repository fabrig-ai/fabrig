//! The start's application reports (`jig/docs/design/domain/engine/vocabulary.md`, 2.6).

use crate::start::{Failure, Phase};

/// An application report given once the decision is accepted.
#[derive(Hash, Debug)]
pub enum Out {
    /// The journal's numbering restored at a start.
    Numbered { commits: u64 },
    /// Each phase as it begins; none once decisions open.
    Start { phase: Option<Phase> },
    /// The phase the start stopped in and its first failure.
    Refused { phase: Phase, failure: Failure },
}
