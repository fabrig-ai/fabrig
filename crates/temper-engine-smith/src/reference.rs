//! The host-tool declarations as their reference page
//! (protocol/documentation.md, 8).

use skein_lib::{Overflow, Writer};
use smith_domain_run as run;

use crate::{
    calls::{SPECS, Spec},
    json::{self, Value},
};

/// The reference page `host-tools`, whole, as docs/help/reference holds it
/// (protocol/documentation.md, 8).
pub fn reference(out: &mut Writer) -> Result<(), Overflow> {
    render(&SPECS, out)
}

/// The page over any declarations, for the step tests' fixtures.
pub(crate) fn render(specs: &[Spec], out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Host tools\n\nKind: reference\n\nEvery tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.\n")?;
    for spec in specs {
        let schema = json::parse(spec.schema).expect("declared tool schema is a JSON object");
        out.put(b"\n## ")?;
        out.put(spec.name)?;
        out.put(b"\n\n")?;
        out.put(spec.description)?;
        out.put(b"\n\n")?;
        out.put(match spec.effect {
            run::HostEffect::Read => b"A read.\n",
            run::HostEffect::Write => b"A write.\n",
        })?;
        let properties = match field(&schema, b"properties") {
            Some(Value::Object(properties)) => properties.as_ref(),
            None => &[],
            Some(Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => {
                unreachable!("declared properties are an object")
            }
        };
        if properties.is_empty() {
            out.put(b"\nIts input is `{}`.\n")?;
            continue;
        }
        out.put(b"\n")?;
        let source = raw_field(&schema, spec.schema, b"properties").expect("declared properties");
        for (index, (name, member)) in properties.iter().enumerate() {
            out.put(b"- `")?;
            out.put(name)?;
            out.put(b"`: ")?;
            member_type(member, raw_at(source, index), out)?;
            out.put(b"; ")?;
            let mut required = false;
            if let Some(Value::Array(names)) = field(&schema, b"required") {
                for item in names {
                    if let Value::String(text) = item {
                        required |= text == name;
                    }
                }
            }
            out.put(if required { b"required." } else { b"optional." })?;
            if let Some(Value::String(description)) = field(member, b"description") {
                out.put(b" ")?;
                out.put(description)?;
            }
            out.put(b"\n")?;
        }
    }
    Ok(())
}

fn field<'a>(value: &'a Value, name: &'static [u8]) -> Option<&'a Value> {
    value.field(name).expect("declared schema member is an object")
}

fn member_type(member: &Value, source: &[u8], out: &mut Writer) -> Result<(), Overflow> {
    if let Some(values) = field(member, b"enum") {
        let values = values.array(b"enum").expect("declared enum is an array");
        let source = raw_field(member, source, b"enum").expect("declared enum");
        out.put(b"one of ")?;
        for (index, value) in values.iter().enumerate() {
            if index != 0 {
                out.put(if index == values.len().saturating_sub(1) { b" or " } else { b", " })?;
            }
            out.put(b"`")?;
            match value {
                Value::String(text) => out.put(text)?,
                Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                    out.put(raw_at(source, index))?;
                }
            }
            out.put(b"`")?;
        }
    } else {
        let kind = match field(member, b"type") {
            Some(Value::String(kind)) => kind.as_ref(),
            None | Some(Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => b"",
        };
        out.put(match kind {
            b"string" => b"text",
            b"integer" => b"an integer",
            b"number" => b"a number",
            b"boolean" => b"true or false",
            b"object" => b"an object",
            b"array" => b"a list",
            _ => b"any value",
        })?;
        if kind == b"integer" || kind == b"number" {
            match (field(member, b"minimum"), field(member, b"maximum")) {
                (Some(minimum), Some(maximum)) => {
                    out.put(b" from ")?;
                    out.put(number_text(minimum))?;
                    out.put(b" to ")?;
                    out.put(number_text(maximum))?;
                }
                (Some(minimum), None) => {
                    out.put(b" of at least ")?;
                    out.put(number_text(minimum))?;
                }
                (None, Some(maximum)) => {
                    out.put(b" of at most ")?;
                    out.put(number_text(maximum))?;
                }
                (None, None) => {}
            }
        }
    }
    if let Some(default) = raw_field(member, source, b"default") {
        out.put(b", ")?;
        out.put(default)?;
        out.put(b" when absent")?;
    }
    Ok(())
}

fn number_text(value: &Value) -> &[u8] {
    match value {
        Value::Number(text) => text,
        Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Boolean(_) | Value::Null => {
            unreachable!("declared numeric bound is a number")
        }
    }
}

// Use the validated tree's member positions to borrow source spellings.
// Defaults retain their JSON bytes, including escapes and internal spaces.
fn raw_field<'a>(value: &Value, source: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    match value {
        Value::Object(fields) => {
            for (index, (key, _)) in fields.iter().enumerate() {
                if key.as_ref() == name {
                    return Some(raw_at(source, index));
                }
            }
            None
        }
        Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
            unreachable!("declared schema member is an object")
        }
    }
}

// The nth value of an already parsed object or array; keys are skipped.
fn raw_at(source: &[u8], index: usize) -> &[u8] {
    let source = source.trim_ascii_start();
    let object = source.first() == Some(&b'{');
    let mut rest = source.get(1..).expect("parsed container").trim_ascii_start();
    for at in 0..=index {
        if object {
            rest = rest.get(value_end(rest)..).expect("parsed key").trim_ascii_start();
            rest = rest.strip_prefix(b":").expect("parsed key separator").trim_ascii_start();
        }
        let end = value_end(rest);
        if at == index {
            return rest.get(..end).expect("parsed value");
        }
        rest = rest.get(end..).expect("parsed value end").trim_ascii_start();
        rest = rest.strip_prefix(b",").expect("parsed value separator").trim_ascii_start();
    }
    unreachable!("parsed member position")
}

// Locate one validated value's end without decoding or normalizing it.
fn value_end(source: &[u8]) -> usize {
    let mut depth = 0_u32;
    let mut string = false;
    let mut escaped = false;
    for (index, byte) in source.iter().enumerate() {
        if string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                string = false;
                if depth == 0 {
                    return index.checked_add(1).expect("parsed string end");
                }
            }
            continue;
        }
        match byte {
            b'"' => string = true,
            b'{' | b'[' => depth = depth.checked_add(1).expect("parsed JSON depth"),
            b'}' | b']' => {
                if depth == 0 {
                    return index;
                }
                depth = depth.checked_sub(1).expect("parsed JSON depth");
                if depth == 0 {
                    return index.checked_add(1).expect("parsed container end");
                }
            }
            b',' | b' ' | b'\t' | b'\r' | b'\n' if depth == 0 => return index,
            _ => {}
        }
    }
    source.len()
}
