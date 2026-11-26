//! The TUI's host and its loop (temper/docs/design/client/tui/host.md, 1, and temper/docs/design/client/tui/README.md, 4).
#![forbid(unsafe_code)]

pub mod host;
pub mod journey;
pub mod limits;
pub mod loopback;
pub mod opener;
pub mod run;
pub mod start;

#[doc(hidden)]
pub mod testing {
    pub use crate::start::testing::origin;
    pub mod quit;
}
