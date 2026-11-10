//! Public paths and fixed answers (jig/docs/design/protocol/client/default-http.md, 5).
use alloc::boxed::Box;
use jig_protocol_engine_clients as kit;
use skein_http::{Header, server as http};
use skein_lib::{Decimal, List, Time, Writer, bytes};

#[must_use]
pub fn public_path(path: &[u8]) -> Option<kit::PublicRead> {
    let path = path.strip_prefix(b"/v1/public/")?;
    let mut read = kit::PublicRead { owner: 0, version: 0, kind: 0, ids: Box::new([]) };
    let mut ids = List::with_capacity(2);
    let mut field = 0_u32;
    let mut start = 0;
    for end in 0..=path.len() {
        if end != path.len() && path.get(end) != Some(&b'/') {
            continue;
        }
        let segment = path.get(start..end)?;
        match field {
            0 => read.owner = decimal(segment)?,
            1 => read.version = decimal(segment)?,
            2 => read.kind = decimal(segment)?,
            _ => {
                if segment.len() != 16 {
                    return None;
                }
                let mut id = 0_u64;
                for byte in segment {
                    let digit = match byte {
                        b'0'..=b'9' => byte.checked_sub(b'0')?,
                        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?,
                        _ => return None,
                    };
                    id = id.checked_mul(16)?.checked_add(u64::from(digit))?;
                }
                ids.push(id).ok()?;
            }
        }
        field = field.checked_add(1)?;
        start = end.saturating_add(1);
    }
    if field < 3 {
        return None;
    }
    read.ids = ids.into_boxed();
    Some(read)
}

fn decimal(segment: &[u8]) -> Option<u16> {
    if segment.is_empty() || (segment.len() > 1 && segment.starts_with(b"0")) {
        return None;
    }
    let mut value = 0_u16;
    for byte in segment {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
pub(crate) fn under(prefix: &[u8], head: &http::Call) -> bool {
    match head.target.strip_prefix(prefix) {
        Some(path) => path.starts_with(b"/v1/public/"),
        None => false,
    }
}
fn header(name: &[u8], value: &[u8]) -> Header {
    Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }
}
pub(crate) fn response(status: u16, length: Option<usize>, age: Option<u32>) -> http::Response {
    let mut headers = List::with_capacity(4);
    if length.is_some() {
        headers.push(header(b"content-type", b"application/octet-stream")).expect("fixed headers");
    }
    let cache = match age {
        Some(age) => {
            let digits = Decimal::of(u64::from(age));
            let size =
                b"public, max-age=, must-revalidate".len().checked_add(digits.as_bytes().len()).expect("cache value");
            let mut value = Writer::new(size);
            value.put(b"public, max-age=").expect("cache prefix");
            value.put(digits.as_bytes()).expect("age");
            value.put(b", must-revalidate").expect("cache suffix");
            header(b"cache-control", &value.finish())
        }
        None => header(b"cache-control", b"no-store"),
    };
    if status != 421 {
        headers.push(cache).expect("fixed headers");
    }
    if status == 503 {
        headers.push(header(b"retry-after", b"1")).expect("fixed headers");
    }
    headers.push(header(b"x-content-type-options", b"nosniff")).expect("fixed headers");
    http::Response {
        status,
        headers: headers.into_boxed(),
        body: http::Body::Length(u64::try_from(length.unwrap_or(0)).expect("bounded public answer")),
        close: false,
    }
}
pub(crate) fn answered(reply: kit::PublicReply, max_age: u32, now: Time) -> (http::Response, Box<[u8]>) {
    match reply {
        kit::PublicReply::Document { bytes, decided } => {
            let seconds = now.saturating_since(decided).as_nanos().div_ceil(1_000_000_000);
            let rest = u64::from(max_age).saturating_sub(seconds);
            (response(200, Some(bytes.len()), Some(u32::try_from(rest).expect("rest within max-age"))), bytes)
        }
        kit::PublicReply::Unknown => (response(404, None, Some(max_age)), Box::new([])),
        kit::PublicReply::Busy => (response(503, None, None), Box::new([])),
        kit::PublicReply::NotServed { record } => (response(200, Some(record.len()), None), record),
    }
}
