//! The engine's pass-through vocabulary
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2).

use crate::{Event, Request};
use jig_domain_contract::{Ask, Connector, Released, Up};

/// An input from the root, using the core's vocabulary.
#[derive(Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "the root moves a complete input by value")]
pub enum In {
    Core(Event),
    Store(store::In),
    Connector(Connector, Up),
}

impl In {
    /// Whether this input waits at the root's watermark as new work.
    #[must_use]
    pub fn new_work(&self) -> bool {
        use jig_domain_contract::{
            effects, lifecycle, parts, procedures, projections, requests, resources, settings, topics,
        };
        match self {
            Self::Core(event) => event.new_work(),
            Self::Store(_) => false,
            Self::Connector(_, up) => match up {
                Up::Act { .. } | Up::Topics(topics::Up::News { .. }) => true,
                Up::Resources(up) => match up {
                    resources::Up::Resource { .. } | resources::Up::PoolSlots { .. } | resources::Up::Drift { .. } => {
                        true
                    }
                    resources::Up::Holdings { .. }
                    | resources::Up::DelegateHoldings { .. }
                    | resources::Up::ProcedureHoldings { .. }
                    | resources::Up::ReadAfresh { .. } => false,
                },
                Up::Effects(up) => match up {
                    effects::Up::Start { .. } => true,
                    effects::Up::Described { .. }
                    | effects::Up::Refused { .. }
                    | effects::Up::Busy { .. }
                    | effects::Up::Verdict { .. }
                    | effects::Up::Outbox { .. }
                    | effects::Up::Closed { .. }
                    | effects::Up::Released { .. } => false,
                },
                Up::Lifecycle(
                    lifecycle::Up::Done { .. } | lifecycle::Up::Released { .. } | lifecycle::Up::Adopted { .. },
                )
                | Up::Parts(
                    parts::Up::Made { .. }
                    | parts::Up::Lines { .. }
                    | parts::Up::Public { .. }
                    | parts::Up::Marked { .. },
                )
                | Up::Procedures(
                    procedures::Up::Step { .. } | procedures::Up::Waits { .. } | procedures::Up::Wake { .. },
                )
                | Up::Projections(projections::Up::Settled { .. } | projections::Up::Outbox { .. })
                | Up::Settings(
                    settings::Up::Judged { .. } | settings::Up::Restored { .. } | settings::Up::Defaults { .. },
                )
                | Up::Requests(requests::Up::Decided { .. } | requests::Up::Marked { .. }) => false,
            },
        }
    }
}

/// An output to the root, with the core's request mark intact.
#[derive(Hash, Debug)]
pub enum Out {
    Core(Request),
    Ask(Connector, Ask),
    Held(Peer),
}

/// Which timers the root admits at its watermark.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Admit {
    All,
    Finishing,
}

/// The peer addressed by a held engine output.
#[derive(Hash, Debug)]
pub enum Peer {
    Store(store::Out),
    Connector(Connector, Released),
}

/// The engine's in-process store port.
pub mod store;
