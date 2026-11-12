//! Web Push requests and answers (skein/docs/design/net/push.md, 2–5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use skein_lib::{
    Decimal, List, Secret, Wall, Writer,
    base64::{self, Alphabet},
};

pub const ENDPOINT_BYTES: usize = 1024;
pub const PAYLOAD_BYTES: usize = 3993;
pub const SUBJECT_BYTES: usize = 256;
pub const VAPID_PEM_BYTES: usize = 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct Subscription {
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    bytes: Box<[u8]>,
    split: usize,
}
impl Endpoint {
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > ENDPOINT_BYTES || bytes.is_empty() {
            return None;
        }
        for byte in bytes {
            if !(33..=126).contains(byte) || *byte == b'#' {
                return None;
            }
        }
        let authority =
            if let Some(rest) = bytes.strip_prefix(b"https://") { rest } else { bytes.strip_prefix(b"http://")? };
        let mut end = None;
        for (index, byte) in authority.iter().enumerate() {
            if *byte == b'/' {
                end = Some(index);
                break;
            }
        }
        let end = end?;
        let authority = authority.get(..end)?;
        let mut colon = authority.len();
        for (index, byte) in authority.iter().enumerate() {
            if *byte == b':' {
                colon = index;
                break;
            }
        }
        let host = authority.get(..colon)?;
        if host.is_empty() || host.len() > 253 {
            return None;
        }
        for byte in host {
            if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'.' {
                return None;
            }
        }
        if colon < authority.len() {
            let port = number(authority.get(colon.checked_add(1)?..)?)?;
            if !(1..=65535).contains(&port) {
                return None;
            }
        }
        let split = (if bytes.starts_with(b"https://") { 8_usize } else { 7 }).checked_add(end)?;
        Some(Self { bytes: Box::from(bytes), split })
    }
    #[must_use]
    pub fn origin(&self) -> &[u8] {
        self.bytes.get(..self.split).expect("validated split")
    }
    #[must_use]
    pub fn target(&self) -> &[u8] {
        self.bytes.get(self.split..).expect("validated split")
    }
}
fn number(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut number = 0_u32;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}

#[derive(Debug)]
pub struct Vapid {
    key: Secret,
    public: [u8; 65],
}
impl Vapid {
    #[must_use]
    pub fn from_pkcs8_pem(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > VAPID_PEM_BYTES {
            return None;
        }
        let body = bytes
            .trim_ascii()
            .strip_prefix(b"-----BEGIN PRIVATE KEY-----")?
            .strip_suffix(b"-----END PRIVATE KEY-----")?;
        let mut compact = List::with_capacity(1024);
        for byte in body {
            if !byte.is_ascii_whitespace() {
                compact.push(*byte).ok()?;
            }
        }
        let key =
            base64::decode(Alphabet::Standard, compact.as_slice(), u32::try_from(skein_tls::ES256_KEY_BYTES).ok()?)
                .ok()?;
        let public = skein_tls::es256_public(&key)?;
        Some(Self { key: Secret::new(key), public })
    }
    #[must_use]
    pub const fn public(&self) -> &[u8; 65] {
        &self.public
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    VeryLow,
    Low,
    Normal,
    High,
}
impl Urgency {
    #[must_use]
    pub const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::VeryLow => b"very-low",
            Self::Low => b"low",
            Self::Normal => b"normal",
            Self::High => b"high",
        }
    }
}
#[derive(Debug)]
pub struct Push<'a> {
    pub payload: &'a [u8],
    pub ttl: u32,
    pub urgency: Urgency,
    pub topic: Option<&'a [u8]>,
}
/// One HTTP field's owned name and value.
pub type Header = (Box<[u8]>, Box<[u8]>);

/// A POST: header names and values in wire order; the owner adds Host and Content-Length.
pub struct Sealed {
    pub origin: Box<[u8]>,
    pub target: Box<[u8]>,
    pub headers: Box<[Header]>,
    pub body: Box<[u8]>,
}
// Subscription authentication and the VAPID authorization value stay out of diagnostics.
impl core::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Subscription")
            .field("endpoint", &self.endpoint)
            .field("p256dh", &self.p256dh)
            .finish_non_exhaustive()
    }
}
impl core::fmt::Debug for Sealed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sealed")
            .field("origin", &self.origin)
            .field("target", &self.target)
            .field("headers", &self.headers.len())
            .field("body", &self.body.len())
            .finish()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealError {
    Payload,
    Topic,
    Ttl,
    Subject,
    Keys,
    Endpoint,
}

pub fn seal(
    vapid: &Vapid,
    subject: &[u8],
    subscription: &Subscription,
    push: &Push<'_>,
    wall: Wall,
) -> Result<Sealed, SealError> {
    if !(1..=PAYLOAD_BYTES).contains(&push.payload.len()) {
        return Err(SealError::Payload);
    }
    if push.ttl > 2_419_200 {
        return Err(SealError::Ttl);
    }
    if let Some(topic) = push.topic {
        if topic.is_empty() || topic.len() > 32 {
            return Err(SealError::Topic);
        }
        for byte in topic {
            if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
                return Err(SealError::Topic);
            }
        }
    }
    if !subject_valid(subject) {
        return Err(SealError::Subject);
    }
    let endpoint = Endpoint::parse(&subscription.endpoint).ok_or(SealError::Endpoint)?;
    let body =
        skein_tls::seal_aes128gcm(&subscription.p256dh, &subscription.auth, push.payload).ok_or(SealError::Keys)?;
    let expires = Decimal::of(
        wall.as_nanos()
            .checked_div(1_000_000_000)
            .expect("nonzero divisor")
            .checked_add(43_200)
            .expect("wall seconds plus twelve hours fits u64"),
    );
    // Origin and subject have already excluded every byte that needs JSON escaping.
    let claims =
        join(&[b"{\"aud\":\"", endpoint.origin(), b"\",\"exp\":", expires.as_bytes(), b",\"sub\":\"", subject, b"\"}"]);
    let header = encoded(b"{\"typ\":\"JWT\",\"alg\":\"ES256\"}");
    let claims = encoded(&claims);
    let input = join(&[&header, b".", &claims]);
    let signature = skein_tls::sign_es256(vapid.key.expose(), &input).ok_or(SealError::Keys)?;
    let signature = encoded(&signature);
    let public = encoded(vapid.public());
    let authorization = join(&[b"vapid t=", &input, b".", &signature, b", k=", &public]);
    let mut headers = List::with_capacity(6);
    headers
        .push((Box::from(b"TTL".as_slice()), Box::from(Decimal::of(u64::from(push.ttl)).as_bytes())))
        .expect("six headers");
    headers.push((Box::from(b"Urgency".as_slice()), Box::from(push.urgency.as_bytes()))).expect("six headers");
    if let Some(topic) = push.topic {
        headers.push((Box::from(b"Topic".as_slice()), Box::from(topic))).expect("six headers");
    }
    headers.push((Box::from(b"Content-Encoding".as_slice()), Box::from(b"aes128gcm".as_slice()))).expect("six headers");
    headers
        .push((Box::from(b"Content-Type".as_slice()), Box::from(b"application/octet-stream".as_slice())))
        .expect("six headers");
    headers.push((Box::from(b"Authorization".as_slice()), authorization)).expect("six headers");
    Ok(Sealed {
        origin: Box::from(endpoint.origin()),
        target: Box::from(endpoint.target()),
        headers: headers.into_boxed(),
        body,
    })
}
fn subject_valid(subject: &[u8]) -> bool {
    if subject.is_empty()
        || subject.len() > SUBJECT_BYTES
        || !(subject.starts_with(b"mailto:") || subject.starts_with(b"https:"))
    {
        return false;
    }
    for byte in subject {
        if !(33..=126).contains(byte) || *byte == b'"' || *byte == b'\\' {
            return false;
        }
    }
    true
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded request parts");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured parts");
    }
    writer.finish()
}
fn encoded(bytes: &[u8]) -> Box<[u8]> {
    let len = base64::encoded_len(Alphabet::Url, u32::try_from(bytes.len()).expect("bounded token"))
        .expect("bounded encoding");
    let mut writer = Writer::new(skein_lib::usize_of(len));
    base64::encode(Alphabet::Url, bytes, &mut writer).expect("measured encoding");
    writer.finish()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Made,
    Gone,
    TooLarge,
    Later(Option<u32>),
    Refused(u16),
    Failed(u16),
}
#[must_use]
pub fn answer(status: u16, retry_after: Option<u32>) -> Answer {
    match status {
        200..=299 => Answer::Made,
        404 | 410 => Answer::Gone,
        413 => Answer::TooLarge,
        429 | 500..=599 => Answer::Later(retry_after),
        400 | 401 | 403 => Answer::Refused(status),
        0..=199 | 300..=399 | 402 | 405..=409 | 411..=412 | 414..=428 | 430..=499 | 600..=u16::MAX => {
            Answer::Failed(status)
        }
    }
}
#[cfg(test)]
mod tests;
