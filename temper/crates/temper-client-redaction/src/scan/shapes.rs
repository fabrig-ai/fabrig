//! Byte shapes of temper/docs/design/client/web/journeys.md, 3.7.
use super::Shape;

pub(super) fn scan(bytes: &[u8]) -> Option<Shape> {
    for at in 0..bytes.len() {
        let tail = &bytes[at..];
        if key(bytes, at) {
            return Some(Shape::Key);
        }
        if jwt(tail) {
            return Some(Shape::Jwt);
        }
        if bearer(tail) {
            return Some(Shape::Bearer);
        }
        if code(bytes, at) {
            return Some(Shape::Code);
        }
        if refresh(tail) {
            return Some(Shape::RefreshToken);
        }
    }
    None
}
fn url(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"_-".contains(&byte)
}
fn token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"._~%+/-".contains(&byte)
}
fn value(bytes: &[u8], minimum: usize, alphabet: fn(u8) -> bool) -> bool {
    let length = bytes.iter().take_while(|byte| alphabet(**byte)).count();
    length >= minimum && !bytes[..length].iter().all(|byte| *byte == b'x')
}
fn key(bytes: &[u8], at: usize) -> bool {
    if at >= 2 && url(bytes[at - 2]) && url(bytes[at - 1]) {
        return false;
    }
    bytes[at..].strip_prefix(b"sk-").is_some_and(|tail| value(tail, 20, url))
}
fn jwt(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"eyJ") {
        return false;
    }
    let first = bytes.iter().take_while(|byte| url(**byte)).count();
    if first < 8 {
        return false;
    }
    let Some(second) = bytes[first..].strip_prefix(b".") else {
        return false;
    };
    let length = second.iter().take_while(|byte| url(**byte)).count();
    if length < 8 {
        return false;
    }
    let Some(third) = second[length..].strip_prefix(b".") else {
        return false;
    };
    third.iter().take_while(|byte| url(**byte)).count() >= 8
}
fn bearer(bytes: &[u8]) -> bool {
    let Some(prefix) = bytes.get(..7) else {
        return false;
    };
    prefix.eq_ignore_ascii_case(b"Bearer ")
        && value(&bytes[7..], 16, |byte| byte.is_ascii_alphanumeric() || b"._~+/=-".contains(&byte))
}
fn code(bytes: &[u8], at: usize) -> bool {
    if at > 0 && bytes[at - 1].is_ascii_alphanumeric() {
        return false;
    }
    bytes[at..].strip_prefix(b"code=").is_some_and(|tail| value(tail, 8, token))
}
fn refresh(bytes: &[u8]) -> bool {
    let Some(tail) = bytes.strip_prefix(b"refresh_token") else {
        return false;
    };
    let separators = tail.iter().take_while(|byte| b"\"\\:= ".contains(byte)).count();
    separators <= 4 && value(&tail[separators..], 8, token)
}
