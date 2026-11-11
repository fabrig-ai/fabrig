//! The independent push service (skein/docs/design/testing/fakes/push.md, 2–3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use skein_http::Header;
use skein_lib::{Decimal, List, Map, Wall};
mod token;

pub const RECEIVED: u32 = 256;
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Box<[u8]>,
    pub target: Box<[u8]>,
    pub headers: Box<[Header]>,
    pub body: Box<[u8]>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub target: Box<[u8]>,
    pub topic: Option<Box<[u8]>>,
    pub ttl: u32,
    pub urgency: Box<[u8]>,
    pub length: usize,
    pub at: Wall,
}
#[derive(Debug)]
pub struct Taken {
    pub status: u16,
    pub headers: Box<[Header]>,
    pub received: Option<Received>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Accept,
    Gone,
    Later { seconds: u32 },
    Fail { status: u16 },
}
#[derive(Debug)]
pub struct Service {
    origin: Box<[u8]>,
    key: [u8; 65],
    received: List<Received>,
    scripts: Map<Box<[u8]>, Script>,
}
impl Service {
    #[must_use]
    pub fn new(origin: Box<[u8]>, key: [u8; 65]) -> Self {
        Self { origin, key, received: List::with_capacity(RECEIVED), scripts: Map::uncapped() }
    }
    #[must_use]
    pub fn received(&self) -> &[Received] {
        self.received.as_slice()
    }
    #[must_use]
    pub fn full(&self) -> bool {
        self.received.room() == 0
    }
    pub fn script(&mut self, target: Box<[u8]>, script: Script) {
        let _previous = self.scripts.insert(target, script).expect("the world's script count fits u32");
    }
    #[must_use]
    pub fn take(&mut self, request: Request, wall: Wall) -> Taken {
        match self.check(&request, wall) {
            Err(status) => reply(status),
            Ok(record) => {
                let script = self.scripts.get(request.target.as_ref()).copied().unwrap_or(Script::Accept);
                match script {
                    Script::Gone => reply(410),
                    Script::Fail { status } => reply(status),
                    Script::Later { seconds } => Taken {
                        status: 429,
                        headers: Box::new([Header {
                            name: Box::from(b"Retry-After".as_slice()),
                            value: Box::from(Decimal::of(u64::from(seconds)).as_bytes()),
                        }]),
                        received: None,
                    },
                    Script::Accept => {
                        if self.full() {
                            return reply(507);
                        }
                        self.received.push(record.clone()).expect("checked record room");
                        Taken { status: 201, headers: Box::new([]), received: Some(record) }
                    }
                }
            }
        }
    }
    #[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
    fn check(&self, request: &Request, wall: Wall) -> Result<Received, u16> {
        if !request.target.starts_with(b"/push/") || request.target.len() <= 6 {
            return Err(404);
        }
        if request.method.as_ref() != b"POST" {
            return Err(405);
        }
        let ttl = number(field(&request.headers, b"ttl")?.ok_or(400_u16)?).ok_or(400_u16)?;
        if ttl > 2_419_200 {
            return Err(400);
        }
        let urgency = field(&request.headers, b"urgency")?.unwrap_or(b"normal");
        if urgency != b"very-low" && urgency != b"low" && urgency != b"normal" && urgency != b"high" {
            return Err(400);
        }
        let topic = field(&request.headers, b"topic")?;
        if let Some(topic) = topic {
            if topic.is_empty() || topic.len() > 32 {
                return Err(400);
            }
            for byte in topic {
                if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
                    return Err(400);
                }
            }
        }
        if field(&request.headers, b"content-encoding")? != Some(b"aes128gcm".as_slice()) {
            return Err(400);
        }
        let authorization = field(&request.headers, b"authorization")?.ok_or(401_u16)?;
        token::check(authorization, &self.origin, &self.key, wall)?;
        let body = &request.body;
        if body.len() > 4096 {
            return Err(413);
        }
        if body.len() < 103 {
            return Err(400);
        }
        let rs = u32::from_be_bytes(body.get(16..20).ok_or(400_u16)?.try_into().or(Err(400_u16))?);
        if rs < 18 || body.get(20) != Some(&65) || body.get(21) != Some(&4) || body.len().saturating_sub(86) > 4096 {
            return Err(400);
        }
        Ok(Received {
            target: request.target.clone(),
            topic: match topic {
                Some(topic) => Some(Box::from(topic)),
                None => None,
            },
            ttl,
            urgency: Box::from(urgency),
            length: body.len(),
            at: wall,
        })
    }
}
fn reply(status: u16) -> Taken {
    Taken { status, headers: Box::new([]), received: None }
}
fn field<'a>(headers: &'a [Header], name: &[u8]) -> Result<Option<&'a [u8]>, u16> {
    let mut value = None;
    for header in headers {
        if header.is(name) {
            if value.is_some() {
                return Err(400);
            }
            value = Some(header.value.as_ref());
        }
    }
    Ok(value)
}
fn number(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut value = 0_u32;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
#[cfg(test)]
mod tests;
