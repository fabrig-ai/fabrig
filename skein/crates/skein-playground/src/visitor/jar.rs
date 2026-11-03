//! The visit's bounded, same-origin cookie jar (skein/docs/design/playground.md, 4).
use skein_lib::Wall;

#[derive(Debug)]
struct Cookie {
    name: Box<[u8]>,
    value: Box<[u8]>,
    path: Box<[u8]>,
    expires: Option<Wall>,
}

#[derive(Debug)]
pub(super) struct Jar {
    cookies: Vec<Cookie>,
    count: usize,
    bytes: usize,
}

impl Jar {
    pub(super) fn new(count: u32, bytes: u32) -> Self {
        Self {
            cookies: Vec::new(),
            count: usize::try_from(count).expect("cookie count fits"),
            bytes: usize::try_from(bytes).expect("cookie bytes fit"),
        }
    }

    fn expire(&mut self, wall: Wall) {
        self.cookies.retain(|cookie| cookie.expires.is_none_or(|expires| expires > wall));
    }

    pub(super) fn set(&mut self, field: &[u8], request: &[u8], wall: Wall) {
        self.expire(wall);
        let mut parts = field.split(|byte| *byte == b';');
        let Some((name, value)) = parts.next().and_then(pair) else { return };
        if name.is_empty() || name.len().saturating_add(value.len()) > self.bytes {
            return;
        }
        let mut path = directory(request);
        let mut expires = None;
        let mut clear = false;
        for attribute in parts {
            let Some((name, value)) = pair(attribute) else { continue };
            if name.eq_ignore_ascii_case(b"path") && value.starts_with(b"/") {
                path = value;
            } else if name.eq_ignore_ascii_case(b"max-age") {
                let Ok(text) = core::str::from_utf8(value) else { continue };
                if text
                    .strip_prefix('-')
                    .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                {
                    clear = true;
                } else if let Ok(seconds) = text.parse::<u64>() {
                    clear = seconds == 0;
                    expires =
                        Some(Wall::from_nanos(wall.as_nanos().saturating_add(seconds.saturating_mul(1_000_000_000))));
                }
            }
        }
        let position = self.cookies.iter().position(|cookie| cookie.name.as_ref() == name);
        if clear {
            if let Some(position) = position {
                self.cookies.remove(position);
            }
            return;
        }
        if position.is_none() && self.cookies.len() == self.count {
            return;
        }
        if let Some(position) = position {
            self.cookies.remove(position);
        }
        self.cookies.push(Cookie { name: name.into(), value: value.into(), path: path.into(), expires });
    }

    pub(super) fn header(&mut self, request: &[u8], wall: Wall) -> Option<Box<[u8]>> {
        self.expire(wall);
        let request = path(request);
        let mut bytes = Vec::new();
        for cookie in &self.cookies {
            if request == cookie.path.as_ref()
                || (request.starts_with(&cookie.path)
                    && (cookie.path.ends_with(b"/") || request.get(cookie.path.len()) == Some(&b'/')))
            {
                if !bytes.is_empty() {
                    bytes.extend_from_slice(b"; ");
                }
                bytes.extend_from_slice(&cookie.name);
                bytes.push(b'=');
                bytes.extend_from_slice(&cookie.value);
            }
        }
        (!bytes.is_empty()).then(|| bytes.into_boxed_slice())
    }
}

fn pair(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let (name, value) = bytes.split_at(bytes.iter().position(|byte| *byte == b'=')?);
    Some((name.trim_ascii(), value.strip_prefix(b"=")?.trim_ascii()))
}
fn path(request: &[u8]) -> &[u8] {
    request.split(|byte| *byte == b'?').next().unwrap_or(b"/")
}
fn directory(request: &[u8]) -> &[u8] {
    let path = path(request);
    match path.iter().rposition(|byte| *byte == b'/') {
        Some(at) if at > 0 => path.get(..at).expect("directory within path"),
        Some(_) | None => b"/",
    }
}

#[cfg(test)]
mod tests;
