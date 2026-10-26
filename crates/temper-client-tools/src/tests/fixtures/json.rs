//! Lossless checks of script-facing JSON, including keys decoded from escapes.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test diagnostics use String and format to name a repeated key's path"
)]
use alloc::collections::BTreeSet;
use skein_json::{Document, Kind};

/// Check every object before a map decoder can discard repeated members.
pub(crate) fn unique_keys(json: &[u8]) -> Result<(), String> {
    // The renderer's nesting is bounded; the reader also bounds recursive descent.
    let document = match crate::jig::json::document_depth(json, 128) {
        Ok(document) => document,
        Err(error) => return Err(format!("invalid JSON: {error:?}")),
    };
    let end = value(&document, 0, "$")?;
    assert_eq!(end, document.len(), "one whole JSON document");
    Ok(())
}

fn value(document: &Document, at: u32, path: &str) -> Result<u32, String> {
    let token = document.token(at).expect("validated JSON value");
    let mut next = at.checked_add(1).expect("bounded token count");
    match token.kind {
        Kind::ObjectStart => {
            let mut keys = BTreeSet::new();
            for _ in 0..document.len() {
                let key = document.token(next).expect("object key or end");
                next = next.checked_add(1).expect("bounded token count");
                if key.kind == Kind::ObjectEnd {
                    return Ok(next);
                }
                assert_eq!(key.kind, Kind::Key, "validated object member");
                let key = document.text(key).expect("decoded key bytes");
                let key = key_text(key);
                let child = format!("{path}[{key:?}]");
                if !keys.insert(key) {
                    return Err(format!("repeated key {key:?} at {child}"));
                }
                next = value(document, next, &child)?;
            }
            unreachable!("validated object ends");
        }
        Kind::ArrayStart => {
            for index in 0..document.len() {
                if document.token(next).expect("array value or end").kind == Kind::ArrayEnd {
                    return Ok(next.checked_add(1).expect("bounded token count"));
                }
                next = value(document, next, &format!("{path}[{index}]"))?;
            }
            unreachable!("validated array ends");
        }
        Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null => Ok(next),
        Kind::Key | Kind::ObjectEnd | Kind::ArrayEnd | Kind::Long => unreachable!("validated complete value"),
    }
}

#[expect(clippy::disallowed_methods, reason = "the JSON reader validated UTF-8; diagnostics display its decoded keys")]
fn key_text(key: &[u8]) -> &str {
    core::str::from_utf8(key).expect("JSON key is UTF-8")
}
