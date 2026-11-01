//! Forgejo origins (temper/docs/design/protocol/setup.md, 4).
use crate::record::{self, Address, Transport};
use alloc::boxed::Box;
use core::net::{Ipv6Addr, SocketAddr};

#[expect(
    clippy::disallowed_methods,
    reason = "Connect supplies an origin as bytes; this pure boundary parser validates it"
)]
pub(crate) fn forgejo(address: &[u8]) -> Option<(Address, Transport, Box<[u8]>)> {
    let text = core::str::from_utf8(address).ok()?;
    let origin = text.strip_suffix('/').unwrap_or(text);
    let (api, transport) = if let Some(authority) = origin.strip_prefix("https://") {
        let (host, port) = authority_parts(authority)?;
        (
            Address::Host { name: host.as_bytes().into(), port },
            Transport::Tls { server_name: host.as_bytes().into(), trust: None },
        )
    } else if let Some(authority) = origin.strip_prefix("http://") {
        let socket: SocketAddr = authority.parse().ok()?;
        if !socket.ip().is_loopback() || socket.port() == 0 {
            return None;
        }
        (Address::Socket(socket), Transport::Plaintext)
    } else {
        return None;
    };
    let mut git = skein_lib::Writer::new(origin.len().checked_add(1)?);
    git.put(origin.as_bytes()).ok()?;
    git.put(b"/").ok()?;
    Some((api, transport, git.finish()))
}

#[expect(
    clippy::disallowed_methods,
    reason = "the origin boundary validates numeric ports and IPv6 literals without io"
)]
fn authority_parts(authority: &str) -> Option<(&str, u16)> {
    let (host, suffix) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']')?;
        let _ip: Ipv6Addr = host.parse().ok()?;
        (host, suffix)
    } else {
        let (host, suffix) = match authority.split_once(':') {
            Some((host, _)) => (host, authority.get(host.len()..)?),
            None => (authority, ""),
        };
        for label in host.split('.') {
            if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') {
                return None;
            }
            for byte in label.bytes() {
                if !byte.is_ascii_alphanumeric() && byte != b'-' {
                    return None;
                }
            }
        }
        (host, suffix)
    };
    if host.is_empty() || host.len() > record::HOST_BYTES {
        return None;
    }
    let port = if suffix.is_empty() {
        443
    } else {
        let digits = suffix.strip_prefix(':')?;
        if digits.is_empty() {
            return None;
        }
        for byte in digits.bytes() {
            if !byte.is_ascii_digit() {
                return None;
            }
        }
        digits.parse().ok()?
    };
    if port == 0 {
        return None;
    }
    Some((host, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::disallowed_methods, reason = "the table builds independent expected socket addresses")]
    fn origins_parse_or_refuse() {
        for (origin, host, port) in [
            ("https://forge.example", "forge.example", 443),
            ("https://forge.example/", "forge.example", 443),
            ("https://forge.example:443", "forge.example", 443),
            ("https://forge.example:8443", "forge.example", 8443),
            ("https://127.0.0.1", "127.0.0.1", 443),
            ("https://[::1]:8443", "::1", 8443),
        ] {
            let (api, transport, git) = forgejo(origin.as_bytes()).unwrap();
            assert_eq!(api, Address::Host { name: host.as_bytes().into(), port });
            assert_eq!(transport, Transport::Tls { server_name: host.as_bytes().into(), trust: None });
            assert_eq!(git.strip_suffix(b"/").unwrap(), origin.trim_end_matches('/').as_bytes());
        }
        for origin in ["http://127.0.0.1:3000", "http://127.2.3.4:80/", "http://[::1]:3000"] {
            let (api, transport, git) = forgejo(origin.as_bytes()).unwrap();
            let socket = origin.trim_end_matches('/').strip_prefix("http://").unwrap().parse().unwrap();
            assert_eq!(api, Address::Socket(socket));
            assert_eq!(transport, Transport::Plaintext);
            assert!(git.ends_with(b"/"));
        }
        for origin in [
            "",
            "forge.example",
            "ftp://forge.example",
            "https://",
            "https:///",
            "https://:443",
            "https://forge.example/path",
            "https://forge.example//",
            "https://forge.example?query",
            "https://forge.example/#fragment",
            "https://user@forge.example",
            "https://forge.example:",
            "https://forge.example:0",
            "https://forge.example:65536",
            "https://forge.example:+443",
            "https://forge.example:abc",
            "https://bad host",
            "https://-bad.example",
            "https://bad..example",
            "https://[bad]:443",
            "https://::1",
            "http://localhost:3000",
            "http://192.0.2.1:3000",
            "http://[2001:db8::1]:3000",
            "http://127.0.0.1",
            "http://127.0.0.1:0",
        ] {
            assert_eq!(forgejo(origin.as_bytes()), None, "{origin}");
        }
        assert_eq!(forgejo(b"https://\xff"), None);
    }
}
