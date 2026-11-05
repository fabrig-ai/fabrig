//! Public copy addresses (temper/docs/design/draft/repository-copies.md, 5.4).
use alloc::boxed::Box;
use skein_lib::bytes;

/// A public copy address's maximum byte length.
pub const ADDRESS_BYTES: usize = 512;

/// The checked address's borrowed components, with the optional suffix removed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Source<'a> {
    pub host: &'a [u8],
    pub owner: &'a [u8],
    pub name: &'a [u8],
}

/// Which part of a public source address is refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceRefusal {
    Scheme,
    Segments,
    Host,
    Name,
}

/// Check the public HTTPS source forms, without resolving the host.
pub fn parse(address: &[u8]) -> Result<Source<'_>, SourceRefusal> {
    let path = match address.strip_prefix(b"https://") {
        Some(path) => path,
        None => {
            if bytes::find(address, b"://").is_some() || address.starts_with(b"git@") {
                return Err(SourceRefusal::Scheme);
            }
            address
        }
    };
    let (host, tail) = segment(path).ok_or(SourceRefusal::Segments)?;
    let (owner, name) = segment(tail).ok_or(SourceRefusal::Segments)?;
    if host.is_empty() || owner.is_empty() || name.is_empty() || name.contains(&b'/') {
        return Err(SourceRefusal::Segments);
    }
    if !dns_host(host) {
        return Err(SourceRefusal::Host);
    }
    let name = match name.strip_suffix(b".git") {
        Some(name) => name,
        None => name,
    };
    if !repository(owner) || !repository(name) {
        return Err(SourceRefusal::Name);
    }
    Ok(Source { host, owner, name })
}

fn segment(path: &[u8]) -> Option<(&[u8], &[u8])> {
    let at = bytes::find(path, b"/")?;
    let (head, tail) = path.split_at_checked(at)?;
    Some((head, tail.strip_prefix(b"/")?))
}

fn dns_host(host: &[u8]) -> bool {
    let name = match bytes::find(host, b":") {
        Some(at) => {
            let (name, port) = host.split_at_checked(at).expect("found colon");
            let port = port.strip_prefix(b":").expect("found colon");
            match bytes::decimal(port) {
                Some(port) if port != 0 && u16::try_from(port).is_ok() => {}
                Some(_) | None => return false,
            }
            name
        }
        None => host,
    };
    let name = match name.strip_suffix(b".") {
        Some(name) => name,
        None => name,
    };
    if name.is_empty() || name.len() > 253 {
        return false;
    }
    let mut length = 0_u8;
    let mut last = b'.';
    let mut letter = false;
    for byte in name {
        if *byte == b'.' {
            if length == 0 || last == b'-' {
                return false;
            }
            length = 0;
            letter = false;
        } else {
            if !byte.is_ascii_alphanumeric() && *byte != b'-' || length == 0 && *byte == b'-' {
                return false;
            }
            length = length.saturating_add(1);
            if length > 63 {
                return false;
            }
            letter |= byte.is_ascii_alphabetic();
        }
        last = *byte;
    }
    length != 0 && last != b'-' && letter
}

/// Render the checked address in the form sent to the engine.
#[must_use]
pub fn address(source: &Source<'_>) -> Box<[u8]> {
    bytes::concat(&[b"https://", source.host, b"/", source.owner, b"/", source.name])
}

/// Whether a destination org meets skein's `given::login` grammar.
#[must_use]
pub fn org(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 40 || !name.first().expect("not empty").is_ascii_alphanumeric() {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_alphanumeric() && !b"._-".contains(byte) {
            return false;
        }
    }
    true
}

/// Whether a repository or source owner is a name of 1 to 100 safe bytes.
#[must_use]
pub fn repository(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 100 || name == b"." || name == b".." {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_alphanumeric() && !b"._-".contains(byte) {
            return false;
        }
    }
    true
}
