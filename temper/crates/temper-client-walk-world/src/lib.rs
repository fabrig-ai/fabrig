//! The native journey world (temper/docs/design/client/journey-walks.md, 3.2).

pub mod facts;
pub mod web;

/// The playground composition without a harness.
pub type Playground =
    skein_sim_world::World<temper_playground::programs::Process<()>, temper_playground::machine::Machine>;

// The composition's startup step bound.
const STARTUP_STEPS: u64 = 10_000;
