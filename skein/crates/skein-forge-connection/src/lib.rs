//! Forge protocol component (forge-connection.md, sections 2, 3, 4, 7 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod app;
pub mod boundary;
pub mod component;
mod connection;
pub mod grow;
pub mod hooks;
mod installations;
pub mod limits;
mod puller;
pub mod settings;
pub mod start;
pub use boundary::{Account, Answered, Change, Event, Fact, Request, StartRead, Stopped, Value};
pub use component::{Component, Diagnostics};
pub use limits::{Limits, Shared, Unusable};
pub use settings::{Access, Settings};
#[cfg(test)]
mod tests;

/// State inspected and seeded by the forge machine world (forge-connection.md, 10).
#[cfg(feature = "seam")]
pub mod seam {
    pub use crate::component::entry::{Base, Dialect, ForgeEntry, GitHubForge};
    pub use crate::component::seam::installation_bounds;
    pub use crate::connection::{Connection, Phase};
    pub use crate::hooks::connection::Hooked;
    pub use crate::installations::{
        Accepted, Clock, Diff, GitHub, HeldToken, Metadata, Read, Reading, Resume, Row, Selected, Tokens, Work,
    };
    pub use crate::puller::{Puller, State};
}
