//! HTTP addresses of simulated sockets; no DNS or host network is consulted.
use core::net::{IpAddr, SocketAddr};

pub(super) struct Address {
    pub socket: SocketAddr,
    pub authority: Box<[u8]>,
    pub target: Box<[u8]>,
}

pub(super) fn parse(address: &[u8]) -> Option<Address> {
    let text = core::str::from_utf8(address).ok()?;
    if text.bytes().any(|b| b.is_ascii_control() || b == b' ' || b == b'\\') { return None; }
    let rest = text.strip_prefix("http://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = rest.get(..end).expect("authority ends inside address");
    let socket = authority.parse().ok().or_else(|| {
        authority.trim_start_matches('[').trim_end_matches(']').parse::<IpAddr>().ok().map(|ip| SocketAddr::new(ip, 80))
    })?;
    let path = rest.get(end..).expect("path starts inside address").split('#').next()?;
    let target = if path.starts_with('/') { path.to_owned() } else { format!("/{path}") };
    Some(Address { socket, authority: authority.as_bytes().into(), target: target.into_bytes().into_boxed_slice() })
}

pub(super) fn resolve(base: &[u8], location: &[u8]) -> Option<Box<[u8]>> {
    let base = core::str::from_utf8(base).ok()?;
    let location = core::str::from_utf8(location).ok()?;
    let absolute = if location.starts_with("//") {
        format!("http:{location}")
    } else if location.split(['/', '?', '#']).next()?.contains(':') {
        location.to_owned()
    } else {
        let rest = base.strip_prefix("http://")?;
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let origin = format!("http://{}", rest.get(..end).expect("authority ends inside address"));
        let path = rest.get(end..).expect("path starts inside address").split('#').next()?;
        let path = if path.is_empty() { "/" } else { path };
        if location.starts_with('/') { format!("{origin}{location}") }
        else if location.starts_with('?') { format!("{origin}{}{location}", path.split('?').next()?) }
        else if location.starts_with('#') || location.is_empty() { format!("{origin}{path}{location}") }
        else {
            let directory = path.split('?').next()?.rsplit_once('/').map_or("", |(dir, _)| dir);
            format!("{origin}{directory}/{location}")
        }
    };
    let parsed = parse(absolute.as_bytes())?;
    // RFC 3986's dot segments, preserving empty segments and a trailing slash.
    let target = core::str::from_utf8(&parsed.target).ok()?;
    let (path, query) = target.split_once('?').map_or((target, None), |(p, q)| (p, Some(q)));
    let mut segments = Vec::new();
    let mut parts = path.split('/').peekable();
    while let Some(part) = parts.next() {
        match part {
            "." => { if parts.peek().is_none() { segments.push(""); } }
            ".." => { if segments.len() > 1 { segments.pop(); }
                if parts.peek().is_none() { segments.push(""); } }
            part => segments.push(part),
        }
    }
    let path = segments.join("/");
    let query = query.map_or_else(String::new, |q| format!("?{q}"));
    Some(format!("http://{}{path}{query}", core::str::from_utf8(&parsed.authority).ok()?).into_bytes().into_boxed_slice())
}
