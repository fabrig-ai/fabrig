//! Shared sign-in URL checks (temper/docs/design/protocol/services.md, 4.2).
use skein_lib::bytes;

/// Whether configured bytes are nonempty visible ASCII.
#[must_use]
pub fn visible(value: &[u8]) -> bool {
    for byte in value {
        if !(b'!'..=b'~').contains(byte) {
            return false;
        }
    }
    !value.is_empty()
}
/// Split a bounded HTTP URL into its origin, hostname and port. No userinfo,
/// fragment, controls or backslashes can change the configured destination.
#[expect(clippy::disallowed_methods, reason = "configuration parses bounded URL authorities")]
#[must_use]
pub fn url(value: &[u8]) -> Option<(&[u8], &[u8], u16, bool)> {
    if !visible(value) || value.contains(&b'\\') || value.contains(&b'#') {
        return None;
    }
    let (start, default, tls) = if value.starts_with(b"https://") {
        (8_usize, 443, true)
    } else if value.starts_with(b"http://") {
        (7, 80, false)
    } else {
        return None;
    };
    let rest = value.get(start..)?;
    let mut end = rest.len();
    for separator in [b"/".as_slice(), b"?"] {
        if let Some(at) = bytes::find(rest, separator) {
            end = end.min(at);
        }
    }
    let authority = rest.get(..end)?;
    if authority.is_empty() || authority.contains(&b'@') {
        return None;
    }
    let (host, port) = if let Some(rest) = authority.strip_prefix(b"[") {
        let at = bytes::find(rest, b"]")?;
        let host = rest.get(..at)?;
        let suffix = rest.get(at.checked_add(1)?..)?;
        let port = if suffix.is_empty() { default } else { port(suffix.strip_prefix(b":")?)? };
        core::str::from_utf8(host).ok()?.parse::<core::net::Ipv6Addr>().ok()?;
        (host, port)
    } else if let Some(at) = bytes::find(authority, b":") {
        (authority.get(..at)?, port(authority.get(at.checked_add(1)?..)?)?)
    } else {
        (authority, default)
    };
    if host.is_empty() || port == 0 {
        return None;
    }
    for byte in host {
        if !byte.is_ascii_alphanumeric() && !b".-:".contains(byte) {
            return None;
        }
    }
    if !tls && !loopback(host) {
        return None;
    }
    Some((value.get(..start.checked_add(end)?)?, host, port, tls))
}
fn port(value: &[u8]) -> Option<u16> {
    if value.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}
/// Whether a configured URL host is loopback, for startup endpoint checks.
#[must_use]
#[expect(clippy::disallowed_methods, reason = "configuration checks numeric loopback hosts")]
pub fn loopback(host: &[u8]) -> bool {
    if host == b"localhost" {
        return true;
    }
    let Ok(host) = core::str::from_utf8(host) else { return false };
    match host.parse::<core::net::IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{loopback, url, visible};

    #[test]
    fn url_origins_require_safe_authorities_and_tls_or_loopback() {
        assert_eq!(
            url(b"https://a.example/x?y"),
            Some((b"https://a.example".as_slice(), b"a.example".as_slice(), 443, true))
        );
        assert_eq!(url(b"http://10.0.0.1:80"), None);
        assert_eq!(
            url(b"http://127.0.0.1:7200/"),
            Some((b"http://127.0.0.1:7200".as_slice(), b"127.0.0.1".as_slice(), 7200, false))
        );
        assert_eq!(url(b"https://u@h"), None);
        assert_eq!(url(b"https://h:0"), None);
        assert!(loopback(b"localhost"));
        assert!(loopback(b"::1"));
    }

    #[test]
    fn visible_requires_nonempty_ascii() {
        assert!(visible(b"!https://a.example/~"));
        assert!(!visible(b""));
        assert!(!visible(b"a b"));
        assert!(!visible(b"a\nb"));
        assert!(!visible(b"\x7f"));
        assert!(!visible(b"\x80"));
    }
}
