//! The native journey world (temper/docs/design/client/journey-walks.md, 3.2).

pub mod box_peer;
pub mod facts;
mod harness;
pub mod tui;
pub mod walk;
pub mod web;

pub use harness::{Clients, Native};

/// The composition with its box pipe peer and the terminal's native connection.
pub type Playground =
    skein_sim_world::World<temper_playground::programs::Process<Clients>, temper_playground::machine::Machine>;

/// Build a walk's world, with the native box peer available to the service.
#[must_use]
pub fn world(start: temper_playground::story::Start) -> Playground {
    let mut world = temper_playground::story::world_with::<Clients, _>(start, skein_sim_world::Bare);
    world.host(skein_sim_world::HostedProgram {
        program: box_peer::PROGRAM.into(),
        make: box_peer::BoxPeer::program,
        instances: 1,
        operations: 16,
    });
    world
}

// The composition's startup step bound.
const STARTUP_STEPS: u64 = 10_000;
