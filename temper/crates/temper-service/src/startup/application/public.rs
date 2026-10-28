//! The public browser origin (protocol/sign-in.md, 2.2; protocol/configuration.md, 3).
use alloc::boxed::Box;
use skein_lib::Writer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}
#[derive(Clone, Debug)]
pub struct Origin {
    pub scheme: Scheme,
    /// The URL host, including brackets for an IPv6 literal.
    pub host: Box<[u8]>,
    /// The explicitly written port; absent means the scheme's default.
    pub port: Option<u16>,
    /// The authority, byte for byte as configured, for jig's Host guard.
    pub name: Box<[u8]>,
}
impl Origin {
    #[expect(clippy::disallowed_methods, reason = "the configuration protocol parses URL hosts and ports")]
    pub(super) fn parse(input: &[u8]) -> Option<Self> {
        let (scheme, authority) = if let Some(rest) = input.strip_prefix(b"https://") {
            (Scheme::Https, rest)
        } else if let Some(rest) = input.strip_prefix(b"http://") {
            (Scheme::Http, rest)
        } else {
            return None;
        };
        let authority = core::str::from_utf8(authority).ok()?;
        let (host, written_port) = if authority.starts_with('[') {
            let (host, suffix) = authority.split_once(']')?;
            let end = host.len().checked_add(1)?;
            let host = authority.get(..end)?;
            if suffix.is_empty() { (host, None) } else { (host, Some(suffix.strip_prefix(':')?)) }
        } else if let Some((host, port)) = authority.split_once(':') {
            (host, Some(port))
        } else {
            (authority, None)
        };
        let port = if let Some(value) = written_port {
            if value.is_empty() {
                return None;
            }
            for byte in value.bytes() {
                if !byte.is_ascii_digit() {
                    return None;
                }
            }
            let port = value.parse::<u16>().ok()?;
            if port == 0 || port == 443 {
                return None;
            }
            Some(port)
        } else {
            None
        };
        match scheme {
            Scheme::Http => {
                let literal = if let Some(inner) = host.strip_prefix('[') { inner.strip_suffix(']')? } else { host };
                if host != "localhost" && !literal.parse::<core::net::IpAddr>().ok()?.is_loopback() {
                    return None;
                }
                // IPv6 literals require brackets in an origin.
                if literal.contains(':') != host.starts_with('[') {
                    return None;
                }
            }
            Scheme::Https => {
                if host.is_empty() || host.len() > 253 || host.parse::<core::net::IpAddr>().is_ok() {
                    return None;
                }
                for label in host.split('.') {
                    if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') {
                        return None;
                    }
                    for byte in label.bytes() {
                        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'-' {
                            return None;
                        }
                    }
                }
            }
        }
        Some(Self { scheme, host: Box::from(host.as_bytes()), port, name: Box::from(authority.as_bytes()) })
    }
    /// The public origin with the separator for application routes.
    #[must_use]
    pub fn base(&self) -> Box<[u8]> {
        let scheme = match self.scheme {
            Scheme::Http => b"http://".as_slice(),
            Scheme::Https => b"https://".as_slice(),
        };
        let size = scheme.len().checked_add(self.name.len()).expect("origin").checked_add(1).expect("separator");
        let mut out = Writer::new(size);
        out.put(scheme).expect("scheme");
        out.put(&self.name).expect("authority");
        out.put(b"/").expect("separator");
        out.finish()
    }
}
