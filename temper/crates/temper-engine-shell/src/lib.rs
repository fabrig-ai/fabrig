//! Shared engine commands (temper/docs/design/protocol/binaries.md, 1).
#[cfg(test)]
extern crate self as temper_box_shell;
use directories::{Directories, private_root, unread};
use jig_protocol_config::{self as config, read};
use skein_shell::login;
use std::{fmt::Write, net::ToSocketAddrs, os::unix::ffi::OsStrExt, path::Path};
pub use temper_shell::directories;

mod binary;
pub mod commands;
#[cfg(test)]
mod configuration_tests;
pub mod lines;
pub mod served;
pub use binary::{main, versions};
pub use commands::ENGINE;

pub mod checkpoint;
pub mod forge;
pub mod forge_sign_in;
pub mod restore;
pub mod serve;
pub mod setup;
pub mod stopped;
#[doc(hidden)]
pub mod testing;
pub mod upgrade;
const CONFIG_BYTES: u32 = 1 << 20;

fn prefixed(command: &str, why: &str) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{command}: {line}").expect("writing into a string");
    }
    lines
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
fn member(document: &skein_json::Document, at: u32, key: &[u8]) -> u32 {
    read::Object::new(document, at).expect("named checked object").member(document, key).expect("named checked member")
}
pub fn address(configured: &config::Address) -> Result<std::net::SocketAddr, String> {
    match configured {
        config::Address::Addr(address) => address.socket().ok_or_else(|| "requires a socket address".into()),
        config::Address::Host { name, port } => (std::str::from_utf8(name).map_err(|why| why.to_string())?, *port)
            .to_socket_addrs()
            .map_err(|why| why.to_string())?
            .next()
            .ok_or_else(|| "endpoint has no address".into()),
    }
}

#[cfg(test)]
#[path = "../../../testing/temper-box-kit/src/bundle_fixture.rs"]
pub mod test_bundle_fixture;

#[cfg(test)]
#[path = "../../../testing/temper-box-kit/src/configuration.rs"]
pub mod test_configuration;
