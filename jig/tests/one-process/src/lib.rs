//! The actual one-process service, clients and hosted agents (protocol/service-testing.md, 1, and protocol/service.md, 10).
pub mod accounts;
pub mod fixtures;
pub mod referee;
pub mod world;
mod world_signals;
pub use jig_client_protocol_kit::native::Script;
