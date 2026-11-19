//! The native journey world (temper/docs/design/client/journey-walks.md, 3.2).

pub mod box_peer;
pub mod facts;
mod harness;
pub mod walk;
pub mod web;

pub use harness::Journey;

/// The composition and its box pipe peer, with no ctl.
pub type Playground =
    skein_sim_world::World<temper_playground::programs::Process<Journey>, temper_playground::machine::Machine>;

// The composition's startup step bound.
const STARTUP_STEPS: u64 = 10_000;
