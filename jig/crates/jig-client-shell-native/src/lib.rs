//! The native client's HTTP transport (protocol/client/sign-in.md and protocol/client/README.md, section 9).
#![forbid(unsafe_code)]
mod connections;
pub mod diagnostics;
mod files;
mod shell;
mod socket;
pub use connections::{Conn, Connections, Limits, valid};
pub use files::store_record;
pub use shell::{Engine, Settings, Shell, Tls, Transport};
pub use skein_tls::client::Limits as TlsLimits;
#[cfg(test)]
mod tests;
