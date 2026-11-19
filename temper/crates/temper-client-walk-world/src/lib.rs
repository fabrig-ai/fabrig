//! The native journey world (temper/docs/design/client/journey-walks.md, 3.2).

pub mod facts;
pub mod tui;
pub mod walk;
pub mod web;

/// The playground's clients (temper/docs/design/client/journey-walks.md, 3.3).
pub enum Clients {}
impl temper_playground::programs::Harness for Clients {
    type Service = ();
    type Process = tui::Connection;
}

/// The playground composition with the terminal's native HTTP connection.
pub type Playground =
    skein_sim_world::World<temper_playground::programs::Process<Clients>, temper_playground::machine::Machine>;

// The composition's startup step bound.
const STARTUP_STEPS: u64 = 10_000;
