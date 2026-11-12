//! Joint workers channel on the simulated kernel (protocol/channel/README.md, 11).
pub mod fixtures;
mod processes;
pub mod referee;
mod relay;
pub mod stories;
pub mod world;

mod account_setup;
mod failed_starts;
mod live_input;
mod live_text;
pub mod program_sources;

pub mod both_ways;
pub mod program_runs;

pub mod worker_orders;

pub mod list_read;
