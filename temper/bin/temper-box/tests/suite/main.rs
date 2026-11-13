#[path = "../support/listeners.rs"]
pub mod listeners;
use temper_box_kit as support;

mod agent_modes;
mod box_front;
mod box_listeners;
mod configuration;
mod fake;
mod fake_opening;
mod forge;
mod forge_land;
mod inbox_reply;
mod ports;
mod relay;
mod startup;
mod table;

mod admission_edges;
mod producer_policy;
mod provider_admission;
mod support_tests;
mod wiring;
