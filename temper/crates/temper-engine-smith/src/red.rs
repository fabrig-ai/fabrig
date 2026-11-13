//! A goal's red tool (temper/docs/design/domain/forge/reds.md, 2 and 3).
use crate::{Problem, calls::Spec, json::Value};
use temper_engine_domain::RedCall;
use temper_engine_domain_forge as forge;

pub(crate) const SPEC: Spec = Spec {
    name: b"red",
    description: b"A red is a failure a goal's branch carries that no live change owns. record: key at why; end: key why; list",
    schema: br#"{"type":"object","properties":{"act":{"enum":["record","end","list"]},"key":{"type":"string"},"at":{"type":"string","description":"40 or 64 lowercase hex digits of the check's base."},"why":{"type":"string"}},"required":["act"]}"#,
    effect: smith_domain_run::HostEffect::Write,
    offer: crate::calls::Offer::Delegate,
};

pub(crate) fn decode(value: &Value) -> Result<RedCall, Problem> {
    let act = value.required(b"act")?.text(b"act")?;
    let fields: &[&[u8]] = match act.as_ref() {
        b"record" => &[b"act", b"key", b"at", b"why"],
        b"end" => &[b"act", b"key", b"why"],
        b"list" => &[b"act"],
        _ => return Err(Problem::Range(b"red: act must be record, end or list")),
    };
    let Value::Object(members) = value else { return Err(Problem::Type(b"red")) };
    for (name, _) in members {
        if !fields.contains(&name.as_ref()) {
            return Err(Problem::ActMember { tool: b"red", member: name.clone() });
        }
    }
    if act.as_ref() == b"list" { return Ok(RedCall::List); }
    let key = value.required(b"key")?.text(b"key")?;
    if !forge::reds::key_fits(&key, 512) {
        return Err(Problem::Range(b"red: key must be 1 to forge.red_key_bytes UTF-8 bytes, one line, no control characters or surrounding whitespace"));
    }
    let why = value.required(b"why")?.text(b"why")?;
    if !forge::settings::words_fit(&why, 256) {
        return Err(Problem::Range(b"red: why must be one line of 1 to 256 UTF-8 bytes, no control characters"));
    }
    if act.as_ref() == b"end" { return Ok(RedCall::End { key, why }); }
    let text = value.required(b"at")?.text(b"at")?;
    if text.len() != 40 && text.len() != 64 {
        return Err(Problem::Range(b"red: at must be 40 or 64 lowercase hexadecimal digits"));
    }
    for &byte in &text {
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return Err(Problem::Range(b"red: at must be 40 or 64 lowercase hexadecimal digits"));
        }
    }
    let mut head = skein_lib::Writer::new(text.len().checked_div(2).expect("nonzero divisor"));
    for pair in text.as_chunks::<2>().0 {
        let high = digit(*pair.first().expect("hex pair"));
        let low = digit(*pair.get(1).expect("hex pair"));
        head.put(&[high.saturating_mul(16).saturating_add(low)]).expect("measured head");
    }
    Ok(RedCall::Record { key, at: head.finish(), why })
}
fn digit(byte: u8) -> u8 {
    if byte.is_ascii_digit() { byte.saturating_sub(b'0') } else { byte.saturating_sub(b'a').saturating_add(10) }
}
mod render;
pub(crate) use render::render;
