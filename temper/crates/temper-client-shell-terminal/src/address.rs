//! An engine's address or sign-in link (temper/docs/design/client/line.md, 1, and temper/docs/design/client/tui/README.md, 1).
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
};
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Target {
    Link { engine: SocketAddr, link: Secret },
    Address { engine: SocketAddr },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    NotALink(Box<[u8]>),
    NotLoopback,
}

pub fn target(argument: &[u8]) -> Result<Target, Refused> {
    let bad = || Refused::NotALink(argument.into());
    let text = std::str::from_utf8(argument).map_err(|_| bad())?;
    let rest = text.strip_prefix("http://").ok_or_else(bad)?;
    let (address, path) = rest.split_once('/').unwrap_or((rest, ""));
    let engine: SocketAddr = address.parse().map_err(|_| bad())?;
    if !path.is_empty() {
        let secret = path.strip_prefix("jig/v1/sign-in/link?secret=").ok_or_else(bad)?;
        let secret = base64::decode(Alphabet::Url, secret.as_bytes(), 32).map_err(|_| bad())?;
        if secret.len() != 32 {
            return Err(bad());
        }
    }
    if !engine.ip().is_loopback() {
        return Err(Refused::NotLoopback);
    }
    Ok(if path.is_empty() {
        Target::Address { engine }
    } else {
        Target::Link { engine, link: Secret::new(argument.into()) }
    })
}
