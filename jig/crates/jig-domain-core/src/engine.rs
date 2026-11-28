//! The engine passes one decision through to the core
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2).

pub use jig_domain_core_vocabulary::engine::*;

use crate::{Config, Core, Held, Limits, Request, Timer};
use jig_domain_contract::{Connector, Declaration, Released, StartProblem};

mod connectors;
pub use connectors::{bridge_ask, bridge_released};
use skein_lib::{Env, Queue, Time, Wall};

/// The core behind the functions a root calls.
#[derive(Hash, Debug)]
pub struct Engine {
    core: Core,
}

impl Engine {
    #[cfg(test)]
    pub(crate) fn from_core(core: Core) -> Self {
        Self { core }
    }

    /// # Errors
    /// Worker membership exceeds its bounds or the connector declarations do not hold.
    pub fn new(config: Config, connectors: &[Declaration], limits: &Limits) -> Result<Engine, StartProblem> {
        let core = Core::new(config, connectors, limits)?;
        Ok(Self { core })
    }

    /// Shared access to the core while the ports are being introduced.
    #[must_use]
    pub fn core(&self) -> &Core {
        &self.core
    }

    /// Exclusive access to the core while the ports are being introduced.
    pub fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }
}

/// Take one input as one decision, appending the core's requests in order.
pub fn step(engine: &mut Engine, env: &Env<Limits>, input: In, out: &mut Queue<Out>) {
    let requests = match input {
        In::Core(event) => crate::step(&mut engine.core, env, event),
        In::Store(event) => crate::routing::step_store(&mut engine.core, env, event),
        In::Connector(connector, up) => match connectors::input(connector, up) {
            connectors::Input::Event(event) => crate::step(&mut engine.core, env, event),
            connectors::Input::CatchUp(stage) => {
                engine.core.catch_up_done(connector.0, stage);
                Queue::uncapped()
            }
            connectors::Input::Act { key, act, carried } => crate::routing::step_connector_act(
                &mut engine.core,
                env,
                jig_domain_people::ActKey { connector: connector.0, key },
                act,
                carried,
            ),
        },
    };
    append(requests, out);
}

/// Fire one due timer as one decision, appending the core's requests in order.
pub fn fire(engine: &mut Engine, env: &Env<Limits>, timer: Timer, out: &mut Queue<Out>) {
    let requests = crate::fire(&mut engine.core, env, timer);
    append(requests, out);
}

/// The core's first due timer under the root's admission.
#[must_use]
pub fn due(engine: &Engine, now: Time, wall: Wall, admit: Admit) -> Option<Timer> {
    match admit {
        Admit::All => engine.core.due(now, wall),
        Admit::Finishing => engine.core.finishing_due(now, wall),
    }
}

/// The core's earliest deadline under the root's admission.
#[must_use]
pub fn next_deadline(engine: &Engine, now: Time, wall: Wall, admit: Admit) -> Option<Time> {
    match admit {
        Admit::All => engine.core.next_deadline(now, wall),
        Admit::Finishing => engine.core.next_finishing_deadline(now, wall),
    }
}

/// Quiescence of the core alone; the root accounts for its journal separately.
#[must_use]
pub fn idle(engine: &Engine) -> bool {
    engine.core.quiescent(true)
}

/// Append requests from a direct core entrance, keeping every output at its place.
pub fn append(mut requests: Queue<Request>, out: &mut Queue<Out>) {
    for _ in 0..requests.len() {
        let request = requests.pop().expect("core request count");
        let output = match request {
            Request::Ask { connector, ask } => match connectors::ask(ask) {
                Ok(ask) => Out::Ask(Connector(connector), ask),
                Err(ask) => Out::Core(Request::Ask { connector, ask }),
            },
            Request::Held(held) => match *held {
                Held::Store(load) => Out::Held(Peer::Store(load)),
                Held::MakeEffect { connector, entry } => {
                    Out::Held(Peer::Connector(Connector(connector), Released::Make { entry }))
                }
                Held::ConnectorActed { connector, key, outcome } => {
                    Out::Held(Peer::Connector(Connector(connector), Released::Answered { key, outcome }))
                }
                Held::Acted { .. }
                | Held::Drained
                | Held::Restarting { .. }
                | Held::ApplicationDelivery { .. }
                | Held::ApplicationWatchEnded { .. }
                | Held::Read { .. }
                | Held::Applied { .. }
                | Held::CarriedUnused { .. }
                | Held::Delivery { .. }
                | Held::WatchEnded { .. }
                | Held::SettledCall { .. }
                | Held::Assign { .. }
                | Held::ReadUsage { .. }
                | Held::ReadList { .. }
                | Held::Sources { .. }
                | Held::CheckSource { .. }
                | Held::PauseWorker { .. }
                | Held::SignOutSource { .. }
                | Held::SourceSignedOut { .. }
                | Held::Grant { .. }
                | Held::Relayed { .. }
                | Held::Inbound { .. }
                | Held::Relay { .. }
                | Held::PeopleReply { .. }
                | Held::CallAnswer { .. }
                | Held::Result { .. }
                | Held::Acknowledge { .. }
                | Held::TaskTerminalAcknowledged { .. }
                | Held::TaskPartKept { .. }
                | Held::AcknowledgePart { .. }
                | Held::PartBusy { .. }
                | Held::TaskTurnKept { .. }
                | Held::AcknowledgeTurn { .. }
                | Held::Cancel { .. }
                | Held::Refuse { .. }
                | Held::TurnBusy { .. }
                | Held::StopRun { .. }
                | Held::NotesWritten { .. }
                | Held::NotesDeleted { .. }
                | Held::PublicRead { .. }
                | Held::PublicWait { .. }
                | Held::NeedArrived { .. } => Out::Core(Request::Held(held)),
            },
            other @ (Request::ProjectMade { .. } | Request::Write(_) | Request::Now(_)) => Out::Core(other),
        };
        out.push(output);
    }
}
