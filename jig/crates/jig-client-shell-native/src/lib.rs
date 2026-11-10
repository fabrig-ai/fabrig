//! The native client's HTTP transport (protocol/client/sign-in.md and protocol/client/README.md, section 9).
#![forbid(unsafe_code)]
mod connections;
pub mod diagnostics;
mod files;
mod shell;
pub use connections::{Conn, Connections, Limits, valid};
pub use shell::{Settings, Shell, Transport};
#[cfg(test)]
mod tests;
