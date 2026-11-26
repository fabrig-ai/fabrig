//! Connector effect identities and origins (jig/docs/design/domain/engine/vocabulary.md, 1.3).

pub use jig_domain_contract::effects::{EffectKey, EffectPurpose};
pub mod connector;
pub mod projections;

use crate::CallKey;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Wall};

/// Who requested the connector-owned effect and when a call must answer.
#[derive(Hash, Debug)]
pub enum EffectOrigin {
    /// A connector-owned goal projection, independent of executable task grants.
    Projection {
        goal: u64,
        entry: Option<u64>,
    },
    Call {
        to: ReplyTo,
        key: CallKey,
        deadline: Wall,
    },
    /// An explicit proposal keeps its connector payload without making it.
    Propose {
        to: ReplyTo,
        key: CallKey,
        reason: Box<[u8]>,
        as_holder: bool,
    },
    /// A holder decision already authenticated by the core's proposal route.
    Accept {
        to: ReplyTo,
        key: Option<CallKey>,
        proposal: Box<tasks::Proposal>,
        by: tasks::Party,
    },
    Procedure {
        task: u64,
        step: u64,
        entry: Option<u64>,
    },
}

impl EffectOrigin {
    #[must_use]
    pub fn task(&self) -> u64 {
        match self {
            Self::Projection { goal, .. } => *goal,
            Self::Call { key, .. } | Self::Propose { key, .. } => key.task,
            Self::Accept { proposal, .. } => proposal.proposer,
            Self::Procedure { task, .. } => *task,
        }
    }
}
