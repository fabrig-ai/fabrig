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

// The composition's startup step bound.
const STARTUP_STEPS: u64 = 10_000;
