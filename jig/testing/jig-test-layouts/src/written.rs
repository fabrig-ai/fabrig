//! Shared written pass for the store job and wire tests (jig/docs/design/protocol/store/jobs.md, 6;
//! jig/docs/design/protocol/channel/layouts.md, 8).

use crate::layouts::{WRITTEN_NUMBER, WRITTEN_TEXT};
use alloc::boxed::Box;
use skein_codec::layout::{Mode, Refusal, Side};
use skein_codec::read_len;
use skein_lib::{Buffer, Decimal, Reader};

#[cfg(test)]
mod tests;

const NUMBER_TEXT: u32 = 10;
const LABEL: u32 = 16;

/// Converts the fixture's `Retyped` record between its number and decimal text layouts.
/// Read and Keep agree: both carry the number and label or refuse the value.
pub fn convert(_mode: Mode, from: &Side<'_>, to: &Side<'_>, value: &[u8], out: &mut Buffer) -> Result<(), Refusal> {
    out.clear();
    let family = to.text.spelled(to.text.family());
    let declaration = to.text.spelled(to.text.declaration(to.root.decl).name);
    let result = match (family, declaration, to.root.variant) {
        (b"fixture-written", b"Retyped", None) => retyped(from, to, value, out),
        _ => Err(unsupported(from, "unsupported target value type")),
    };
    result.inspect_err(|_| out.clear())
}

fn retyped(from: &Side<'_>, to: &Side<'_>, value: &[u8], out: &mut Buffer) -> Result<(), Refusal> {
    let to_text = match (from.text.text(), to.text.text()) {
        (a, b) if a == WRITTEN_NUMBER.as_bytes() && b == WRITTEN_TEXT.as_bytes() => true,
        (a, b) if a == WRITTEN_TEXT.as_bytes() && b == WRITTEN_NUMBER.as_bytes() => false,
        _ => return Err(unsupported(from, "unsupported layout pair")),
    };
    if Some(from.root) != from.text.root(b"Retyped", None) {
        return Err(unsupported(from, "unsupported layout pair"));
    }
    if u32::try_from(value.len()).is_err() {
        return Err(retyped_refusal("", "value is too long"));
    }
    let mut read = Reader::new(value);
    if read.u16() != Some(from.text.version()) {
        return Err(retyped_refusal("", "missing or wrong source version"));
    }
    put(out, &to.text.version().to_be_bytes(), "")?;
    if to_text {
        let number = read.u32().ok_or_else(|| retyped_refusal("n", "short number"))?;
        put_text(out, Decimal::of(u64::from(number)).as_bytes(), "n")?;
    } else {
        let digits = text(&mut read, NUMBER_TEXT, "n")?;
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(retyped_refusal("n", "expected decimal digits"));
        }
        let number = digits.parse::<u32>().map_err(|_| retyped_refusal("n", "decimal number exceeds u32"))?;
        put(out, &number.to_be_bytes(), "n")?;
    }
    let label = text(&mut read, LABEL, "label")?;
    if !read.is_empty() {
        return Err(retyped_refusal("", "trailing bytes"));
    }
    put_text(out, label.as_bytes(), "label")
}

fn text<'a>(read: &mut Reader<'a>, limit: u32, part: &str) -> Result<&'a str, Refusal> {
    let short_or_large = || retyped_refusal(part, "short or oversized text");
    let length = read_len(read, limit).map_err(|_| short_or_large())?;
    let bytes = read.bytes(length).ok_or_else(short_or_large)?;
    core::str::from_utf8(bytes).map_err(|_| retyped_refusal(part, "text is not UTF-8"))
}

fn put_text(out: &mut Buffer, text: &[u8], part: &str) -> Result<(), Refusal> {
    let length = u32::try_from(text.len()).expect("fixture text is bounded");
    put(out, &length.to_be_bytes(), part)?;
    put(out, text, part)
}

fn put(out: &mut Buffer, bytes: &[u8], part: &str) -> Result<(), Refusal> {
    out.put(bytes).map_err(|_| retyped_refusal(part, "result exceeds output capacity"))
}

fn retyped_refusal(part: &str, why: &str) -> Refusal {
    refuse("Retyped", part, why)
}

fn unsupported(from: &Side<'_>, why: &str) -> Refusal {
    let name = from.text.spelled(from.text.declaration(from.root.decl).name);
    let declaration = core::str::from_utf8(name).expect("schema declaration names are ASCII");
    refuse(declaration, "", why)
}

fn refuse(declaration: &str, part: &str, why: &str) -> Refusal {
    Refusal { declaration: Box::from(declaration), part: Box::from(part), why: Box::from(why) }
}
