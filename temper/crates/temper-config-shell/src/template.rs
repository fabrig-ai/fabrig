//! Place names in TOML string values (temper/docs/design/testing/qa/deployment.md, 2).
use crate::{parse::table, render::quote};
use toml::de::DeValue;

/// Substitute decoded string values, never keys, comments or other values.
/// Re-quote changed strings as basic strings, keeping source order and untouched text.
pub fn substitute(text: &[u8], mut replace: impl FnMut(&str) -> Result<String, String>) -> Result<Box<[u8]>, String> {
    let (text, table) = table(text).map_err(|why| why.to_string())?;
    let mut edits = Vec::new();
    for (_, value) in table.get_ref() {
        strings(value, &mut replace, &mut edits)?;
    }
    edits.sort_by_key(|(span, _)| span.start);
    let mut out = String::new();
    let mut start = 0;
    for (span, value) in edits {
        out.push_str(&text[start..span.start]);
        out.push_str(&quote(&value));
        start = span.end;
    }
    out.push_str(&text[start..]);
    Ok(out.into_bytes().into())
}

fn strings(
    value: &toml::Spanned<DeValue<'_>>,
    replace: &mut impl FnMut(&str) -> Result<String, String>,
    edits: &mut Vec<(std::ops::Range<usize>, String)>,
) -> Result<(), String> {
    match value.get_ref() {
        DeValue::String(text) => {
            let text = text.as_ref();
            let replaced = replace(text)?;
            if replaced != text {
                edits.push((value.span(), replaced));
            }
        }
        DeValue::Table(table) => {
            for (_, value) in table {
                strings(value, replace, edits)?;
            }
        }
        DeValue::Array(array) => {
            for value in array {
                strings(value, replace, edits)?;
            }
        }
        DeValue::Integer(_) | DeValue::Float(_) | DeValue::Boolean(_) | DeValue::Datetime(_) => {}
    }
    Ok(())
}
