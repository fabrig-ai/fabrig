//! The TUI's engine origin (temper/docs/design/client/tui/README.md, 1).
use alloc::boxed::Box;
use core::net::{IpAddr, SocketAddr};
use skein_lib::{
    Decimal, Intake,
    bytes::{concat, copy_of, decimal},
};

/// An engine on loopback, or a deployment by name over HTTPS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    Http(SocketAddr),
    Https { name: Box<[u8]>, port: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unaddressed {
    NotAnAddress,
    HttpName,
    HttpsAddress,
    OffLoopback,
}

impl Origin {
    /// Whether the engine is a deployment by name, with browser approval alone.
    #[must_use]
    pub const fn at_name(&self) -> bool {
        match self {
            Self::Http(_) => false,
            Self::Https { .. } => true,
        }
    }

    /// Read an origin with no path, query, user or fragment.
    /// # Errors
    /// The address's refusal, as the start names it.
    #[expect(
        clippy::disallowed_methods,
        reason = "Origin is the TUI's text boundary for arguments, kept addresses and recordings"
    )]
    pub fn parse(bytes: &[u8]) -> Result<Origin, Unaddressed> {
        let bytes = bytes.strip_suffix(b"/").unwrap_or(bytes);
        let Ok(text) = core::str::from_utf8(bytes) else { return Err(Unaddressed::NotAnAddress) };
        let (secure, authority) = if let Some(rest) = text.strip_prefix("http://") {
            (false, rest)
        } else if let Some(rest) = text.strip_prefix("https://") {
            (true, rest)
        } else {
            return Err(Unaddressed::NotAnAddress);
        };
        for byte in authority.bytes() {
            if b"/?#@".contains(&byte) {
                return Err(Unaddressed::NotAnAddress);
            }
        }
        if let Ok(socket) = authority.parse::<SocketAddr>() {
            if socket.port() == 0 {
                return Err(Unaddressed::NotAnAddress);
            }
            if secure {
                return Err(Unaddressed::HttpsAddress);
            }
            if !socket.ip().is_loopback() {
                return Err(Unaddressed::OffLoopback);
            }
            return Ok(Origin::Http(socket));
        }
        let unbracketed = match authority.strip_prefix('[') {
            Some(rest) => rest.strip_suffix(']').unwrap_or(authority),
            None => authority,
        };
        if secure && unbracketed.parse::<IpAddr>().is_ok() {
            return Err(Unaddressed::HttpsAddress);
        }
        let (name, port) = match authority.rsplit_once(':') {
            Some((name, port)) => (name, read_port(port.as_bytes())?),
            None if secure => (authority, 443),
            None => return Err(Unaddressed::NotAnAddress),
        };
        if name.parse::<IpAddr>().is_ok() && secure {
            return Err(Unaddressed::HttpsAddress);
        }
        if !valid_name(name.as_bytes()) {
            return Err(Unaddressed::NotAnAddress);
        }
        if !secure {
            return Err(Unaddressed::HttpName);
        }
        let mut name = copy_of(name.as_bytes());
        name.make_ascii_lowercase();
        Ok(Origin::Https { name, port })
    }

    #[must_use]
    pub fn print(&self) -> Box<[u8]> {
        match self {
            Origin::Http(_) => concat(&[b"http://", &self.authority()]),
            Origin::Https { name, port: 443 } => concat(&[b"https://", name]),
            Origin::Https { .. } => concat(&[b"https://", &self.authority()]),
        }
    }

    /// HOST:PORT, including the default port.
    #[must_use]
    pub fn authority(&self) -> Box<[u8]> {
        match self {
            Origin::Https { name, port } => concat(&[name, b":", Decimal::of(u64::from(*port)).as_bytes()]),
            Origin::Http(socket) => {
                let mut out = Intake::with_capacity(64);
                match socket {
                    SocketAddr::V4(socket) => {
                        for (at, octet) in socket.ip().octets().iter().enumerate() {
                            if at != 0 {
                                out.append(b".").expect("IPv4 authority fits");
                            }
                            out.append(Decimal::of(u64::from(*octet)).as_bytes()).expect("IPv4 authority fits");
                        }
                    }
                    SocketAddr::V6(socket) => {
                        assert!(socket.ip().is_loopback(), "HTTP origins are loopback");
                        out.append(b"[::1").expect("IPv6 loopback authority fits");
                        if socket.scope_id() != 0 {
                            out.append(b"%").expect("IPv6 scope fits");
                            out.append(Decimal::of(u64::from(socket.scope_id())).as_bytes()).expect("IPv6 scope fits");
                        }
                        out.append(b"]").expect("IPv6 loopback authority fits");
                    }
                }
                out.append(b":").expect("authority fits");
                out.append(Decimal::of(u64::from(socket.port())).as_bytes()).expect("authority fits");
                out.meet(skein_lib::stream::Read::Fill(out.len())).expect("authority filled")
            }
        }
    }
}

fn read_port(bytes: &[u8]) -> Result<u16, Unaddressed> {
    let value = decimal(bytes).ok_or(Unaddressed::NotAnAddress)?;
    let Ok(value) = u16::try_from(value) else { return Err(Unaddressed::NotAnAddress) };
    if value == 0 {
        return Err(Unaddressed::NotAnAddress);
    }
    Ok(value)
}

fn valid_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 253 {
        return false;
    }
    let mut label = 0_u32;
    let mut last = b'.';
    for byte in name {
        if *byte == b'.' {
            if label == 0 || last == b'-' {
                return false;
            }
            label = 0;
        } else {
            if (!byte.is_ascii_alphanumeric() && *byte != b'-') || (label == 0 && *byte == b'-') {
                return false;
            }
            label = label.saturating_add(1);
            if label > 63 {
                return false;
            }
        }
        last = *byte;
    }
    label != 0 && last != b'-'
}

/// A page on this engine, never an address supplied by a peer.
pub(crate) fn page(origin: &Origin, path: &[u8]) -> Box<[u8]> {
    concat(&[&origin.print(), path])
}
