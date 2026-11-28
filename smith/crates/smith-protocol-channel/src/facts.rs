//! Domain observations become counts or a preparation report
//! (protocol/channel.md, section 8; domain/run.md, section 10).
//!
//! This projection keeps no state, guide text or credential values. The component
//! owns elapsed time, output room and the count of facts it drops.

use smith_channel::FactKind;
use smith_domain::{self as domain, run, session, tools};

pub(crate) enum Projection {
    Count { kind: FactKind, count: u64 },
    Guides(smith_channel::v14::Guides),
}

/// Project an observation without carrying guide text or any other prompt content.
pub(crate) fn project(fact: &domain::Fact) -> Option<Projection> {
    match fact {
        domain::Fact::Run { fact, opening: _ } => match &fact.kind {
            run::facts::FactKind::Admitted { .. } => Some(Projection::Count { kind: FactKind::Admitted, count: 1 }),
            run::facts::FactKind::Answered { .. } => Some(Projection::Count { kind: FactKind::Ended, count: 1 }),
            run::facts::FactKind::Called { .. } => Some(Projection::Count { kind: FactKind::ToolStarted, count: 1 }),
            run::facts::FactKind::Returned { .. } => Some(Projection::Count { kind: FactKind::ToolFinished, count: 1 }),
            run::facts::FactKind::CheckStarted { .. } => {
                Some(Projection::Count { kind: FactKind::CheckStarted, count: 1 })
            }
            run::facts::FactKind::CheckFinished { .. } => {
                Some(Projection::Count { kind: FactKind::CheckFinished, count: 1 })
            }
            run::facts::FactKind::Prepared { guides, .. } => {
                let mut rows = skein_lib::List::with_capacity(u32::try_from(guides.len()).expect("bounded workspace"));
                for guide in guides {
                    rows.push(smith_channel::v14::Guide {
                        directory: guide.directory,
                        path: guide.path.clone(),
                        digest: guide.digest,
                        bytes: guide.bytes,
                        whole: guide.whole,
                    })
                    .expect("one row per guide read");
                }
                Some(Projection::Guides(smith_channel::v14::Guides { guides: rows }))
            }
            run::facts::FactKind::MessageRead { .. }
            | run::facts::FactKind::MessageFence { .. }
            | run::facts::FactKind::MessageUnread { .. }
            | run::facts::FactKind::MessageReceived { .. }
            | run::facts::FactKind::MessageRefused { .. }
            | run::facts::FactKind::Budget { .. }
            | run::facts::FactKind::Opened { .. }
            | run::facts::FactKind::Ended { .. }
            | run::facts::FactKind::Delivered { .. } => None,
        },
        domain::Fact::Session { fact } => match &fact.kind {
            session::FactKind::CompletionStarted { attempt, .. } => {
                Some(Projection::Count { kind: FactKind::LlmStarted, count: u64::from(*attempt) })
            }
            session::FactKind::CompletionRetried { attempt, .. } => {
                Some(Projection::Count { kind: FactKind::LlmRetried, count: u64::from(*attempt) })
            }
            session::FactKind::CompletionAnswered { blocks, .. } => {
                Some(Projection::Count { kind: FactKind::LlmFinished, count: u64::from(*blocks) })
            }
            session::FactKind::CompletionFailed { .. } | session::FactKind::CompletionCancelled { .. } => {
                Some(Projection::Count { kind: FactKind::LlmFinished, count: 0 })
            }
            session::FactKind::Tools { fact, .. } => match fact {
                tools::FactKind::Started { .. } => Some(Projection::Count { kind: FactKind::ToolStarted, count: 1 }),
                tools::FactKind::Answered { .. } => Some(Projection::Count { kind: FactKind::ToolFinished, count: 1 }),
                tools::FactKind::Opened { .. }
                | tools::FactKind::Refused { .. }
                | tools::FactKind::JobEnded { .. }
                | tools::FactKind::Closing { .. }
                | tools::FactKind::Closed { .. } => None,
            },
            session::FactKind::JobEnded { .. }
            | session::FactKind::ReasoningDropped { .. }
            | session::FactKind::CompactionStarted { .. }
            | session::FactKind::CompactionEnded { .. }
            | session::FactKind::Opened { .. }
            | session::FactKind::BuiltinStarted { .. }
            | session::FactKind::BuiltinAnswered { .. }
            | session::FactKind::DelegateStarted { .. }
            | session::FactKind::DelegateAnswered { .. }
            | session::FactKind::DelegateCancelled { .. }
            | session::FactKind::DelegateWithheld { .. }
            | session::FactKind::Yielded { .. }
            | session::FactKind::Used { .. }
            | session::FactKind::Ended { .. } => None,
        },
    }
}
