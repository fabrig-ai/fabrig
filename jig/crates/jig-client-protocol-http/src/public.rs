//! Public calls and cache freshness (jig/docs/design/protocol/client/default-http.md, 5).
use crate::{Call, Mapping};
use alloc::boxed::Box;
#[expect(clippy::disallowed_types, reason = "the public HTTP boundary specifies kept headers as String pairs")]
use alloc::string::String;
use skein_lib::{Decimal, List, Writer};

pub const PUBLIC_KEEP: &[&str] = &["cache-control", "age", "retry-after"];
#[derive(Debug, PartialEq, Eq)]
pub struct PublicPath {
    pub owner: u16,
    pub version: u16,
    pub kind: u16,
    pub ids: List<u64>,
}
/// The GET, with no bearer, body or content type.
#[must_use]
pub fn public(mapping: &Mapping, path: &PublicPath) -> Call {
    crate::call::call(mapping, None, skein_http::Method::Get, &public_target(path), Box::new([]))
}
/// The canonical target after the mapping's prefix.
#[must_use]
pub fn public_target(path: &PublicPath) -> Box<[u8]> {
    assert!(path.ids.len() <= 2, "a public read has at most two identifiers");
    let mut size = b"/v1/public".len();
    for number in [path.owner, path.version, path.kind] {
        size = size
            .checked_add(1)
            .expect("separator")
            .checked_add(Decimal::of(u64::from(number)).as_bytes().len())
            .expect("decimal");
    }
    size = size
        .checked_add(skein_lib::length::usize_of(path.ids.len()).checked_mul(17).expect("two identifiers"))
        .expect("public target");
    let mut target = Writer::new(size);
    target.put(b"/v1/public").expect("public prefix");
    for number in [path.owner, path.version, path.kind] {
        target.put(b"/").expect("separator");
        target.put(Decimal::of(u64::from(number)).as_bytes()).expect("decimal");
    }
    for id in &path.ids {
        target.put(b"/").expect("separator");
        for shift in (0..16_u32).rev() {
            let shift = shift.checked_mul(4).expect("hex shift");
            let digit = u8::try_from(id.checked_shr(shift).expect("hex shift") & 15).expect("hex digit");
            let byte = if digit < 10 { b'0'.checked_add(digit) } else { b'a'.checked_add(digit.saturating_sub(10)) };
            target.put(&[byte.expect("hex byte")]).expect("identifier");
        }
    }
    target.finish()
}
#[derive(Debug, PartialEq, Eq)]
pub struct Freshness {
    pub fresh: u32,
    pub retry_after: Option<u32>,
}
/// max-age less Age, saturating at zero; malformed freshness is zero.
#[must_use]
#[expect(clippy::disallowed_types, reason = "kept HTTP headers cross the client boundary as String pairs")]
pub fn freshness(headers: &[(String, String)]) -> Freshness {
    let mut max_age = None;
    let mut age = Some(0);
    let mut retry_after = None;
    let mut invalid = false;
    let mut seen_age = false;
    let mut seen_max = false;
    let mut seen_retry = false;
    for (name, value) in headers {
        match name.as_str() {
            "cache-control" => {
                for directive in value.split(',') {
                    let directive = directive.trim_ascii();
                    if directive.eq_ignore_ascii_case("no-store") {
                        invalid = true;
                    }
                    match directive.split_once('=') {
                        Some((name, value)) if name.trim_ascii().eq_ignore_ascii_case("max-age") => {
                            if seen_max {
                                invalid = true;
                            }
                            seen_max = true;
                            max_age = seconds(value.trim_ascii());
                        }
                        None if directive.eq_ignore_ascii_case("max-age") => invalid = true,
                        Some(_) | None => {}
                    }
                }
            }
            "age" => {
                if seen_age {
                    invalid = true;
                }
                seen_age = true;
                age = seconds(value.trim_ascii());
            }
            "retry-after" => {
                retry_after = if seen_retry { None } else { seconds(value.trim_ascii()) };
                seen_retry = true;
            }
            _ => {}
        }
    }
    let fresh = match (max_age, age) {
        (Some(max_age), Some(age)) if !invalid => max_age.saturating_sub(age),
        _ => 0,
    };
    Freshness { fresh, retry_after }
}
fn seconds(value: &str) -> Option<u32> {
    if value.is_empty() {
        return None;
    }
    let mut seconds = 0_u32;
    for byte in value.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        seconds = seconds.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    Some(seconds)
}
#[derive(Debug, PartialEq, Eq)]
pub enum PublicHeard {
    Document { body: Box<[u8]>, fresh: u32 },
    Unknown { fresh: u32 },
    Busy { retry_after: Option<u32> },
    Lost,
}
#[must_use]
#[expect(clippy::disallowed_types, reason = "kept HTTP headers cross the client boundary as String pairs")]
pub fn public_answered(status: u16, body: Box<[u8]>, headers: &[(String, String)]) -> PublicHeard {
    let freshness = freshness(headers);
    match status {
        200 => PublicHeard::Document { body, fresh: freshness.fresh },
        404 => PublicHeard::Unknown { fresh: freshness.fresh },
        503 => PublicHeard::Busy { retry_after: freshness.retry_after },
        _ => PublicHeard::Lost,
    }
}
