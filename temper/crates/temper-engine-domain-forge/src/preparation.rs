//! The worker's per-member prepare outcomes (domain/landing.md, 5.2).
use alloc::boxed::Box;
use skein_json::{Kind, document::Document};
use skein_lib::List;
use temper_engine_domain_forge_landing::Outcome;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Prepared {
    /// Position in the landing, counted from zero.
    pub member: u16,
    pub outcome: Outcome,
    pub files: Box<[u8]>,
}

#[must_use]
pub fn outcomes(bytes: &[u8], members: u32) -> Option<Box<[Prepared]>> {
    let length = u32::try_from(bytes.len()).ok()?;
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let document =
        skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length }).ok()?;
    let mut result: List<Prepared> = List::with_capacity(members);
    let mut recognized = false;
    for name in [b"combined".as_slice(), b"conflicts"] {
        let Some(array) = document.member(0, name) else { continue };
        recognized = true;
        if document.token(array)?.kind != Kind::ArrayStart {
            return None;
        }
        let end = document.value_end(array)?.checked_sub(1)?;
        let start = array.checked_add(1)?;
        let mut cursor = start;
        for at in start..end {
            if at != cursor {
                continue;
            }
            let token = document.token(document.member(at, b"member")?)?;
            if token.kind != Kind::Number {
                return None;
            }
            let member = skein_lib::bytes::decimal(document.text(token)?)?;
            let member = u16::try_from(member.checked_sub(1)?).ok()?;
            if u32::from(member) >= members {
                return None;
            }
            let mut files = List::with_capacity(length);
            let outcome = if name == b"combined" {
                match text(&document, at, b"by")?.as_ref() {
                    b"merge" | b"squash" => Outcome::Combined,
                    _ => return None,
                }
            } else {
                let array = document.member(at, b"files")?;
                if document.token(array)?.kind != Kind::ArrayStart {
                    return None;
                }
                for at in array.checked_add(1)?..document.value_end(array)?.checked_sub(1)? {
                    let value = document.token(at)?;
                    if value.kind != Kind::String {
                        return None;
                    }
                    if !files.is_empty() {
                        append(&mut files, b", ")?;
                    }
                    append(&mut files, document.text(value)?)?;
                }
                match text(&document, at, b"with")?.as_ref() {
                    b"branch" => Outcome::Conflict,
                    b"member" => Outcome::EarlierConflict,
                    _ => return None,
                }
            };
            for prior in &result {
                if prior.member == member {
                    return None;
                }
            }
            result.push(Prepared { member, outcome, files: files.into_boxed() }).ok()?;
            cursor = document.value_end(at)?;
        }
    }
    if !recognized || result.is_empty() {
        return None;
    }
    Some(result.into_boxed())
}
fn text(document: &Document, object: u32, name: &[u8]) -> Option<Box<[u8]>> {
    let token = document.token(document.member(object, name)?)?;
    if token.kind != Kind::String {
        return None;
    }
    Some(Box::from(document.text(token)?))
}

fn append(list: &mut List<u8>, bytes: &[u8]) -> Option<()> {
    for byte in bytes {
        list.push(*byte).ok()?;
    }
    Some(())
}

/// Keep each leaving member's words beside the current job or turn diagnostic.
pub(crate) fn remember(prior: &[u8], incoming: &[u8], members: u32) -> Box<[u8]> {
    let old = outcomes(prior, members).unwrap_or(Box::new([]));
    let new = outcomes(incoming, members);
    let mut kept = List::with_capacity(members);
    for member in 0..members {
        let mut found = None;
        for outcome in &old {
            if u32::from(outcome.member) == member {
                found = Some(outcome.clone());
            }
        }
        if let Some(new) = &new {
            for outcome in new {
                if u32::from(outcome.member) == member {
                    found = Some(outcome.clone());
                }
            }
        }
        if let Some(found) = found
            && found.outcome != Outcome::Combined
        {
            kept.push(found).expect("one outcome per member");
        }
    }
    if kept.is_empty() {
        return Box::from(incoming);
    }
    let failure = if new.is_some() { Box::from([]) } else { failure(incoming) };
    let limits = skein_json::writer::Limits { depth: 6, length: 8192 };
    for compact in [false, true] {
        let mut measure = skein_json::writer::Encoder::measure(&limits);
        encode(&mut measure, kept.as_slice(), &failure, compact);
        if let Ok(length) = measure.measured() {
            let mut json = skein_json::writer::Encoder::write(length, &limits);
            encode(&mut json, kept.as_slice(), &failure, compact);
            return json.finish();
        }
    }
    unreachable!("compact diagnostics reserve six escaped bytes per input byte")
}
fn encode(json: &mut skein_json::writer::Encoder, kept: &[Prepared], failure: &[u8], compact: bool) {
    let fields = kept.len().checked_add(1).expect("bounded members");
    let room = 7680_usize.checked_div(6).expect("nonzero escape width").checked_div(fields).expect("nonzero fields");
    json.object_start();
    json.key(b"conflicts");
    json.array_start();
    for outcome in kept {
        json.object_start();
        json.key(b"member");
        json.unsigned(u64::from(outcome.member).checked_add(1).expect("member position"));
        json.key(b"with");
        json.string(match outcome.outcome {
            Outcome::Conflict => b"branch",
            Outcome::EarlierConflict => b"member",
            Outcome::Combined => unreachable!("only conflicts retained"),
        });
        json.key(b"files");
        json.array_start();
        let files = if compact { crate::text::fit(&outcome.files, room, b"...") } else { outcome.files.clone() };
        json.string(&files);
        json.array_end();
        json.key(b"cut");
        json.boolean(compact);
        json.object_end();
    }
    json.array_end();
    json.key(b"failure");
    let failure = if compact { crate::text::fit(failure, room, b"...") } else { Box::from(failure) };
    json.string(&failure);
    json.object_end();
}
pub(crate) fn failure(bytes: &[u8]) -> Box<[u8]> {
    let Ok(length) = u32::try_from(bytes.len()) else { return Box::from(bytes) };
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let Ok(document) =
        skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length })
    else {
        return Box::from(bytes);
    };
    if let Some(words) = text(&document, 0, b"failure") {
        return words;
    }
    if document.member(0, b"conflicts").is_some() || document.member(0, b"combined").is_some() {
        return Box::new([]);
    }
    Box::from(bytes)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_later_prepare_or_turn_diagnostic_keeps_each_members_conflict_words() {
        let first = br#"{"combined":[{"member":1,"by":"merge"}],"conflicts":[{"member":2,"with":"member","files":["quoted\"path","comma,path"],"cut":false}]}"#;
        let second = br#"{"combined":[{"member":3,"by":"squash"}]}"#;
        let saved = remember(first, second, 3);
        let saved = remember(&saved, b"push refused", 3);
        assert_eq!(failure(&saved).as_ref(), b"push refused");
        let conflicts = outcomes(&saved, 3).expect("retained outcomes");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].member, 1);
        assert_eq!(conflicts[0].outcome, Outcome::EarlierConflict);
        assert_eq!(conflicts[0].files.as_ref(), b"quoted\"path, comma,path");
        let cleared = remember(&saved, b"", 3);
        assert!(failure(&cleared).is_empty());
        assert_eq!(outcomes(&cleared, 3).expect("kept conflict")[0].files, conflicts[0].files);
        assert!(outcomes(br#"{"conflicts":[]}"#, 3).is_none());
        assert!(outcomes(br#"{"combined":[{"member":4,"by":"merge"}]}"#, 3).is_none());
    }
}
