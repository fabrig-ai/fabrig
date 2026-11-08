//! Shared engine commands (temper/docs/design/protocol/README.md, 4.1).
#[cfg(test)]
extern crate self as temper;
use directories::{Directories, private_root, unread};
use jig_protocol_config::{self as config, read};
use std::{fmt::Write, net::ToSocketAddrs, os::unix::ffi::OsStrExt, path::Path};
use temper_forge_setup_shell::process as forge_exec;
pub use temper_shell::directories;

pub mod forge;
pub mod forge_sign_in;
pub mod setup;
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
fn listeners_refusal(why: skein_shell::ListenersRefusal) -> String {
    format!("LISTEN_FDS: {}", listener_words(why))
}
fn login(user: u32) -> Result<Box<[u8]>, String> {
    if cfg!(target_os = "macos") { forge_exec::login_id() } else { login_passwd(user) }
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
#[must_use]
pub fn listener_words(why: skein_shell::ListenersRefusal) -> String {
    use skein_shell::ListenersRefusal as Why;
    match why {
        Why::BadCount => "expected a decimal listener count".into(),
        Why::TooMany { bound } => format!("listener count exceeds {bound}"),
        Why::NotOpen { fd } => format!("fd {} is not open", fd.raw()),
        Why::NotSocket { fd, .. } => format!("fd {} is not a socket", fd.raw()),
        Why::NotStream { fd, .. } => format!("fd {} is not a stream socket", fd.raw()),
        Why::NotListening { fd } => format!("fd {} is not listening", fd.raw()),
        Why::NotIp { fd } => format!("fd {} is not an IP listener", fd.raw()),
        Why::Check { fd, check, errno } => format!("fd {}: {}: errno {errno}", fd.raw(), check.name()),
        Why::Kernel { fd, errno } => format!("fd {}: close-on-exec: errno {errno}", fd.raw()),
    }
}
/// The effective user's passwd login; forge commands preserve lookup refusals.
pub fn login_passwd(user: u32) -> Result<Box<[u8]>, String> {
    let passwd = skein_shell::read_file(std::path::Path::new("/etc/passwd"), 1 << 20)
        .map_err(|why| format!("user: {}", unread(why)))?;
    for line in passwd.split(|c| *c == b'\n') {
        let fields: Vec<_> = line.split(|c| *c == b':').collect();
        if fields.get(2).and_then(|id| std::str::from_utf8(id).ok()).and_then(|id| id.parse::<u32>().ok()) == Some(user)
        {
            return Ok(fields[0].into());
        }
    }
    Err(format!("user {user}: no login in /etc/passwd"))
}

#[cfg(test)]
#[path = "../../temper/tests/support/configuration.rs"]
pub mod test_configuration;
