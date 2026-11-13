//! Shared client settings and forge origins (protocol/services.md, 4.2).
use crate::startup::application::Origin;
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{List, bytes};

/// The application's clients listener (protocol/configuration/engine.md).
#[derive(Debug)]
pub struct Clients {
    /// The socket address used for the listener, Host guard and printed links.
    pub listener: config::Address,
    /// `application.clients.bundle`: the bundle's directory; `None` serves no browser.
    pub bundle: Option<Box<[u8]>>,
    /// The browser origin and jig public name (protocol/sign-in.md, 2.2).
    pub public: Option<Origin>,
}
/// The forge's browser origin (protocol/openid-provider.md, 1 and 7).
#[must_use]
pub fn forge_origin(git: &[u8], web: Option<&[u8]>, public: Option<&temper_forge_setup::setup::Origin>) -> Box<[u8]> {
    let Some(public) = public else { return bytes::copy_of(web.unwrap_or(git)) };
    let port = skein_lib::Decimal::of(u64::from(public.port));
    let mut parts = List::with_capacity(4);
    parts.push(b"https://".as_slice()).expect("four origin pieces");
    parts.push(public.host.as_ref()).expect("four origin pieces");
    if public.port != 443 {
        parts.push(b":".as_slice()).expect("four origin pieces");
        parts.push(port.as_bytes()).expect("four origin pieces");
    }
    let mut length = 0_usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("bounded origin");
    }
    let mut out = skein_lib::Writer::new(length);
    for part in &parts {
        out.put(part).expect("measured origin");
    }
    out.finish()
}
