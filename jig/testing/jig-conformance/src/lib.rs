//! Application conformance from durable records and independent neighbours
//! (`domain/testing.md`, 5–7).
#![forbid(unsafe_code)]
pub mod harness;
pub mod referee;
pub mod store_check;
pub mod trace;
pub use harness::{Application, Clock, Cut, Durable, Harness, Input, Outcome, Output, cut_passed};
pub mod scenarios;

pub mod core;
