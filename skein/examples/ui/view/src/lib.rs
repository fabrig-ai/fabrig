//! The notes example view (examples.md, 9.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

pub use skein_ui_example_view_codec::records;
pub mod draw;
pub mod limits;
pub mod places;
mod state;
mod view;
pub use places::Place;
pub use state::{Duties, Exchange, Record, Report, State, StreamDuty};

#[cfg(test)]
mod tests;
