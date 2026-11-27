//! The engine passes one decision through to the core
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2).

pub use jig_domain_core_vocabulary::engine::*;

use crate::{Config, Core, Held, Limits, Request, Timer};
use jig_domain_link::Problem;
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
    /// A configured worker count or name exceeds the priced membership bounds.
    pub fn new(config: Config, limits: &Limits) -> Result<Engine, Problem> {
        let core = Core::new(config, limits)?;
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

#[expect(clippy::wildcard_enum_match_arm, reason = "only the store port leaves the core request vocabulary")]
fn append(mut requests: Queue<Request>, out: &mut Queue<Out>) {
    for _ in 0..requests.len() {
        let request = requests.pop().expect("core request count");
        let output = match request {
            Request::Held(held) => {
                if let Held::Store(load) = *held {
                    Out::Held(Peer::Store(load))
                } else {
                    Out::Core(Request::Held(held))
                }
            }
            other => Out::Core(other),
        };
        out.push(output);
    }
}
