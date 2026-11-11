//! Independent VAPID verification (testing/fakes/push.md, 3; RFC 8292).
use skein_json::{Document, Kind, document, tokenizer};
use skein_lib::{
    Wall,
    base64::{self, Alphabet},
};

pub(super) fn check(authorization: &[u8], origin: &[u8], public: &[u8; 65], wall: Wall) -> Result<(), u16> {
    let value = authorization.strip_prefix(b"vapid t=").ok_or(401_u16)?;
    if value.len() > 2048 {
        return Err(401);
    }
    let mut split = None;
    for (index, part) in value.windows(4).enumerate() {
        if part == b", k=" {
            split = Some(index);
            break;
        }
    }
    let split = split.ok_or(401_u16)?;
    let token = value.get(..split).ok_or(401_u16)?;
    let key = value.get(split.checked_add(4).ok_or(401_u16)?..).ok_or(401_u16)?;
    let key = base64::decode(Alphabet::Url, key, 65).or(Err(403_u16))?;
    if key.as_ref() != public {
        return Err(403);
    }
    verify(token, origin, public, wall).ok_or(403)
}
fn verify(token: &[u8], origin: &[u8], public: &[u8; 65], wall: Wall) -> Option<()> {
    let mut dots = [0; 2];
    let mut count = 0_usize;
    for (index, byte) in token.iter().enumerate() {
        if *byte == b'.' {
            *dots.get_mut(count)? = index;
            count = count.checked_add(1)?;
        }
    }
    if count != 2 {
        return None;
    }
    let first = *dots.first()?;
    let second = *dots.get(1)?;
    let header = base64::decode(Alphabet::Url, token.get(..first)?, 64).ok()?;
    if header.as_ref() != b"{\"typ\":\"JWT\",\"alg\":\"ES256\"}" {
        return None;
    }
    let claims = base64::decode(Alphabet::Url, token.get(first.checked_add(1)?..second)?, 1536).ok()?;
    let signature = base64::decode(Alphabet::Url, token.get(second.checked_add(1)?..)?, 64).ok()?;
    if !skein_tls::verify_es256(public, token.get(..second)?, &signature.as_ref().try_into().ok()?) {
        return None;
    }
    let doc = skein_json::parse(
        &claims,
        &tokenizer::Limits { depth: 2, string: 1024, number: 20, chunk: 128, length: 1536 },
        &document::Limits { tokens: 16, text: 1536 },
    )
    .ok()?;
    if text(&doc, b"aud", Kind::String)? != origin {
        return None;
    }
    let exp = text(&doc, b"exp", Kind::Number)?;
    let mut expires = 0_u64;
    if exp.is_empty() {
        return None;
    }
    for byte in exp {
        if !byte.is_ascii_digit() {
            return None;
        }
        expires = expires.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    let now = wall.as_nanos().checked_div(1_000_000_000)?;
    if expires <= now || expires > now.checked_add(86_400)? {
        return None;
    }
    let subject = text(&doc, b"sub", Kind::String)?;
    if subject.is_empty() || subject.len() > 256 || !(subject.starts_with(b"mailto:") || subject.starts_with(b"https:"))
    {
        return None;
    }
    for byte in subject {
        if !(33..=126).contains(byte) || *byte == b'"' || *byte == b'\\' {
            return None;
        }
    }
    Some(())
}
fn text<'a>(doc: &'a Document, name: &[u8], kind: Kind) -> Option<&'a [u8]> {
    let record = doc.token(doc.member(0, name)?)?;
    if record.kind != kind {
        return None;
    }
    doc.text(record)
}
