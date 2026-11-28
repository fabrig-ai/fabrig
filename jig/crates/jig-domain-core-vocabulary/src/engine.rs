//! The engine's in-process vocabulary
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2 and 2.6).

use crate::{Event, Request};
use jig_domain_contract::{Ask, Connector, Released, Up};
use skein_lib::Token;

/// An input from the root, addressed to the core or one of the ports.
#[derive(Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "the root moves a complete input by value")]
pub enum In {
    Core(Event),
    Store(store::In),
    Connector(Connector, Up),
    /// A start continuation released by the journal.
    Resume(Resume),
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
            Self::Store(_) | Self::Resume(_) => false,
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
                    | parts::Up::Marked { .. }
                    | parts::Up::Planned { .. }
                    | parts::Up::Sized { .. }
                    | parts::Up::Failed { .. },
                )
                | Up::Procedures(
                    procedures::Up::Step { .. } | procedures::Up::Waits { .. } | procedures::Up::Wake { .. },
                )
                | Up::Projections(projections::Up::Settled { .. } | projections::Up::Outbox { .. })
                | Up::Settings(
                    settings::Up::Judged { .. }
                    | settings::Up::Restored { .. }
                    | settings::Up::Defaults { .. }
                    | settings::Up::Unfit { .. },
                )
                | Up::Requests(requests::Up::Decided { .. } | requests::Up::Marked { .. }) => false,
            },
        }
    }
}

/// An output to the root, marked for its place in the decision.
#[derive(Hash, Debug)]
pub enum Out {
    Core(Request),
    /// A write for the decision's commit.
    Write(store::Write),
    Ask(Connector, Ask),
    Held(Peer),
    /// An output given once the decision is accepted, not held for its commit.
    Now(Peer),
}

/// Which timers the root admits at its watermark.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Admit {
    All,
    Finishing,
}

/// The peer addressed by an engine output.
#[derive(Hash, Debug)]
pub enum Peer {
    Store(store::Out),
    Connector(Connector, Released),
    /// The application's reports of the start.
    Application(application::Out),
    /// The engine itself, once the journal releases the continuation.
    Resume(Resume),
}

/// The engine's in-process store port.
pub mod store;

/// The engine's own continuation; the root does not read it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Resume(pub Token);

/// The application's in-process port.
pub mod application;
