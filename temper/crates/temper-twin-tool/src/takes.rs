//! Conversation values and their places (temper/docs/design/testing/fakes.md, 1).
use crate::Refusal;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueTake {
    pub name: String,
    pub value: String,
    pub before: String,
    pub after: String,
    #[serde(default, deserialize_with = "present")]
    pub only: Option<Box<[Place]>>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    #[serde(default, deserialize_with = "present")]
    pub call: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub argument: Option<String>,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

pub fn read_takes(bytes: &[u8]) -> Result<Box<[ValueTake]>, Refusal> {
    if bytes.len() > temper_fake::DOCUMENT_BYTES as usize {
        return Err(bad("document bytes"));
    }
    let takes: Box<[ValueTake]> = serde_json::from_slice(bytes).map_err(|why| bad(&why.to_string()))?;
    validate(&takes)?;
    Ok(takes)
}
fn bad(why: &str) -> Refusal {
    Refusal::Takes { why: why.into() }
}
pub(crate) fn validate(takes: &[ValueTake]) -> Result<(), Refusal> {
    if takes.len() > 8 {
        return Err(bad("at most 8 takes"));
    }
    for (i, take) in takes.iter().enumerate() {
        if take.name.is_empty()
            || take.name.len() > 32
            || !take.name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(bad(&format!("{}: name must be 1 to 32 ASCII letters, digits or _", take.name)));
        }
        if takes[..i].iter().any(|previous| previous.name == take.name) {
            return Err(bad(&format!("{}: repeated name", take.name)));
        }
        for (name, text) in [("value", &take.value), ("before", &take.before), ("after", &take.after)] {
            if text.is_empty() || text.len() > 256 || (name == "value" && text.contains('\n')) {
                return Err(bad(&format!("{}: invalid {name}", take.name)));
            }
        }
        if let Some(places) = &take.only {
            if places.is_empty() || places.len() > 8 {
                return Err(bad(&format!("{}: only must hold 1 to 8 places", take.name)));
            }
            for place in places {
                if place.prefix.len() > 256 || place.suffix.len() > 256 {
                    return Err(bad(&format!("{}: prefix and suffix must be at most 256 bytes", take.name)));
                }
                if let Some(pointer) = &place.argument
                    && (place.call.is_none() || !valid_pointer(pointer))
                {
                    return Err(bad(&format!("{}: argument needs a call and an RFC 6901 pointer", take.name)));
                }
            }
        }
    }
    Ok(())
}
fn valid_pointer(pointer: &str) -> bool {
    if !pointer.is_empty() && !pointer.starts_with('/') {
        return false;
    }
    let mut bytes = pointer.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
            return false;
        }
    }
    true
}

fn at(place: &Place, call: Option<&str>, pointer: &str) -> bool {
    if place.call.as_deref() != call {
        return false;
    }
    place
        .argument
        .as_ref()
        .is_none_or(|p| pointer == p || (pointer.starts_with(p) && pointer[p.len()..].starts_with('/')))
}
fn places<'a>(take: &'a ValueTake, call: Option<&str>, pointer: &str) -> Vec<&'a Place> {
    take.only.as_ref().map_or_else(Vec::new, |places| places.iter().filter(|p| at(p, call, pointer)).collect())
}
fn add(count: &mut u32) -> Result<(), Refusal> {
    *count = count.checked_add(1).ok_or(Refusal::Bounds { what: "take uses" })?;
    Ok(())
}

pub(crate) fn text(
    input: &str,
    take: &ValueTake,
    call: Option<&str>,
    pointer: &str,
    count: &mut u32,
) -> Result<String, Refusal> {
    let places = places(take, call, pointer);
    let mut out = String::new();
    let mut kept = 0;
    for (at, _) in input.char_indices() {
        if at < kept || !input[at..].starts_with(&take.value) {
            continue;
        }
        let end = at + take.value.len();
        if take.only.is_none()
            || places.iter().any(|p| input[..at].ends_with(&p.prefix) && input[end..].starts_with(&p.suffix))
        {
            push(&mut out, &input[kept..at])?;
            push(&mut out, "{{")?;
            push(&mut out, &take.name)?;
            push(&mut out, "}}")?;
            kept = end;
            add(count)?;
        }
    }
    push(&mut out, &input[kept..])?;
    Ok(out)
}
fn push(out: &mut String, text: &str) -> Result<(), Refusal> {
    if out.len().checked_add(text.len()).is_none_or(|n| n > temper_fake::DOCUMENT_BYTES as usize) {
        return Err(Refusal::Bounds { what: "document bytes" });
    }
    out.push_str(text);
    Ok(())
}

/// Keep raw numeric tokens: 2, 2.0 and 2e0 are different recorded values.
pub(crate) fn arguments(
    input: &RawValue,
    take: &ValueTake,
    call: &str,
    pointer: &str,
    count: &mut u32,
) -> Result<Box<RawValue>, Refusal> {
    let before = *count;
    let raw = input.get();
    let output = match raw.as_bytes().first() {
        Some(b'{') => {
            let mut object: BTreeMap<String, Box<RawValue>> =
                serde_json::from_str(raw).expect("checked argument object");
            for (key, value) in &mut object {
                let path = format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
                *value = arguments(value, take, call, &path, count)?;
            }
            serde_json::to_string(&object).expect("raw object")
        }
        Some(b'[') => {
            let mut array: Vec<Box<RawValue>> = serde_json::from_str(raw).expect("checked argument array");
            for (i, value) in array.iter_mut().enumerate() {
                *value = arguments(value, take, call, &format!("{pointer}/{i}"), count)?;
            }
            serde_json::to_string(&array).expect("raw array")
        }
        Some(b'"') => {
            let input: String = serde_json::from_str(raw).expect("checked argument string");
            serde_json::to_string(&text(&input, take, Some(call), pointer, count)?).expect("string")
        }
        Some(b'-' | b'0'..=b'9')
            if raw == take.value && (take.only.is_none() || !places(take, Some(call), pointer).is_empty()) =>
        {
            add(count)?;
            serde_json::to_string(&format!("{{{{#{}}}}}", take.name)).expect("number placeholder")
        }
        _ => raw.into(),
    };
    if output.len() > temper_fake::DOCUMENT_BYTES as usize {
        return Err(Refusal::Bounds { what: "document bytes" });
    }
    if before == *count {
        return Ok(input.to_owned());
    }
    Ok(RawValue::from_string(output).expect("transformed arguments"))
}
