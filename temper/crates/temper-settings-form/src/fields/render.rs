use super::{Choice, FIELDS, FORMS, Field, Form, Shape, Value, Width, Word, most, overlay_replaces};
use crate::{Limits, Text};
use alloc::boxed::Box;
use skein_lib::Decimal;

pub const INTRO: &[u8] = b"Read/edit/set whole: `settings show KIND N`. Refusals=shape/count/reference/rules/budget/grace/owner/linked/accounts. Spend=deployment model-price units. uN=unsigned 0..2^N-1; bool=true/false; t[N]/h[N]=text/lowercase hex <=N bytes; ns=nanoseconds; [N] T<=N entries. ?=optional; !=required; |null=nullable. Choices=JSON strings; A|B=alternatives; choice=described alternatives. set=setting.kind; project-policy=policy.";

#[must_use]
pub fn page(limits: &Limits) -> Box<[u8]> {
    page_of(&FORMS, &FIELDS, limits)
}

#[must_use]
pub fn page_of(forms: &[Form], fields: &[Field], limits: &Limits) -> Box<[u8]> {
    let mut measure = Text::measure();
    write(forms, fields, limits, &mut measure);
    let mut out = Text::writing_exact(measure);
    write(forms, fields, limits, &mut out);
    let bytes = out.finish();
    for byte in &bytes {
        assert!(*byte == b'\n' || (b' '..=b'~').contains(byte), "page is printable ASCII and line ends");
    }
    bytes
}

fn write(forms: &[Form], fields: &[Field], limits: &Limits, out: &mut Text) {
    out.put(b"# Settings fields\n\nKind: reference\n\nJSON types, bounds and effects for settings files and `settings show`.\n");
    out.put(b"\n");
    out.put(INTRO);
    out.put(b"\n");
    for form in forms {
        out.put(b"\n## ");
        out.put(form.name);
        out.put(b"\n");
        if !form.line.is_empty() {
            out.put(form.line);
            out.put(b"\n");
        }
        match form.choice {
            Choice::Every | Choice::Number | Choice::Text => {}
            Choice::List { item, bound } => {
                out.put(b"Value: list[");
                number(most(bound, limits), out);
                out.put(b"] ");
                value(&Value::Shape(item), forms, limits, out);
                out.put(b".\n");
            }
            Choice::One { words } => {
                for word in words {
                    out.put(b"- ");
                    quoted(word.word, out);
                    out.put(b":");
                    line(word.line, out);
                    out.put(b"\n");
                }
            }
        }
        if form.shape == Shape::Overlay {
            out.put(b"As `charter`'s fields, replacing the default whole: ");
            let mut first = true;
            for field in fields {
                if overlay_replaces(field) {
                    if !first {
                        out.put(b", ");
                    }
                    first = false;
                    out.put(b"`");
                    out.put(field.name);
                    out.put(b"`");
                }
            }
            out.put(b".\n\n");
        }
        for field in fields {
            if field.shape != form.shape {
                continue;
            }
            out.put(b"- ");
            out.put(field.name);
            out.put(b":");
            if let Some(bound) = field.list {
                out.put(b"[");
                number(most(bound, limits), out);
                out.put(b"] ");
            }
            value(&field.value, forms, limits, out);
            if field.null {
                out.put(b"|null");
            }
            if !field.line.is_empty() {
                out.put(b";");
            }
            line(field.line, out);
            if let Value::Words(words) = field.value {
                for word in words {
                    if !word.line.is_empty() {
                        out.put(b" ");
                        quoted(word.word, out);
                        out.put(b": ");
                        out.put(word.line);
                    }
                }
            }
            out.put(b"\n");
        }
    }
}

fn line(bytes: &[u8], out: &mut Text) {
    if !bytes.is_empty() {
        if let Some(rest) = bytes.strip_prefix(b"Optional; ") {
            out.put(b"? ");
            out.put(rest);
        } else if let Some(rest) = bytes.strip_prefix(b"Required; ") {
            out.put(b"! ");
            out.put(rest);
        } else if let Some(rest) = bytes.strip_prefix(b"Required: ") {
            out.put(b"! ");
            out.put(rest);
        } else if let Some(rest) = bytes.strip_prefix(b"Optional.") {
            out.put(b"?");
            out.put(rest);
        } else if let Some(rest) = bytes.strip_prefix(b"Required ") {
            out.put(b"! ");
            out.put(rest);
        } else if let Some(rest) = bytes.strip_prefix(b"Required.") {
            out.put(b"!");
            out.put(rest);
        } else {
            prose(bytes, out);
        }
    }
}
fn prose(bytes: &[u8], out: &mut Text) {
    for article in [b"The ".as_slice(), b"Its ", b"An ", b"A "] {
        if let Some(rest) = bytes.strip_prefix(article) {
            out.put(rest);
            return;
        }
    }
    out.put(bytes);
}
fn quoted(word: &[u8], out: &mut Text) {
    out.put(b"`");
    if word.is_empty() {
        out.put(b"\"\"");
    } else {
        out.put(word);
    }
    out.put(b"`");
}
fn words(values: &[Word], out: &mut Text) {
    let mut described = !values.is_empty();
    for word in values {
        described &= !word.line.is_empty();
    }
    if described {
        out.put(b"choice");
        return;
    }
    out.put(b"`");
    for (index, word) in values.iter().enumerate() {
        if index > 0 {
            out.put(b"|");
        }
        out.put(if word.word.is_empty() { b"\"\"" } else { word.word });
    }
    out.put(b"`");
}
fn value(value: &Value, forms: &[Form], limits: &Limits, out: &mut Text) {
    match value {
        Value::Integer(width) => out.put(match width {
            Width::U8 => b"u8",
            Width::U16 => b"u16",
            Width::U32 => b"u32",
            Width::U64 => b"u64",
        }),
        Value::CharterOwner => out.put(b"u32|t[32]"),
        Value::Nanoseconds => out.put(b"ns"),
        Value::Time => out.put(b"ns since 1970 UTC"),
        Value::Boolean => out.put(b"bool"),
        Value::Text(bound) | Value::Hex(bound) => {
            out.put(if let Value::Text(_) = value { b"t" } else { b"h" });
            out.put(b"[");
            number(most(*bound, limits), out);
            out.put(b"]");
        }
        Value::Words(values) => words(values, out),
        Value::Shape(shape) => {
            for form in forms {
                if form.shape == *shape {
                    out.put(form.name);
                    return;
                }
            }
            unreachable!("nested shape has a form")
        }
        Value::Setting => out.put(b"the value for `setting.kind`"),
    }
}
fn number(value: u64, out: &mut Text) {
    let decimal = Decimal::of(value);
    out.put(decimal.as_bytes());
}
