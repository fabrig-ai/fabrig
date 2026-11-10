//! Settings pages use the host answer allowance (domain/agent/tools.md).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};

/// A settings page cannot start or advance within its allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The offset splits a UTF-8 character.
    Offset,
    /// Framing or the next character exceeds the allowance.
    TooLarge,
}

/// Render a settings page within the default 2,048-byte host allowance.
pub fn page(text: &[u8], from: u32) -> Result<Box<[u8]>, Error> {
    page_bounded(text, from, 2048)
}

/// Page original UTF-8 bytes, pricing JSON escapes and reserving continuation
/// framing. Cuts advance through long lines; offsets count unescaped bytes.
pub fn page_bounded(text: &[u8], from: u32, maximum: u32) -> Result<Box<[u8]>, Error> {
    let start = usize::try_from(from).expect("u32 fits").min(text.len());
    if !boundary(text, start) {
        return Err(Error::Offset);
    }
    let rest = text.get(start..).expect("text suffix");
    let room = usize::try_from(maximum).expect("u32 fits");
    // end plus its newline, and a separator when the source has none.
    let end_marker = if !rest.is_empty() && !rest.ends_with(b"\n") { 7 } else { 5 };
    if escaped_len(rest).saturating_add(end_marker) <= room {
        return render(rest, None);
    }
    // Separator, next label, ten offset digits, newline, all JSON-escaped.
    let room = room.checked_sub(22).ok_or(Error::TooLarge)?;
    let mut used = 0_usize;
    let mut end = start;
    for byte in rest {
        let size = escaped_len(core::slice::from_ref(byte));
        if used.saturating_add(size) > room {
            break;
        }
        used = used.saturating_add(size);
        end = end.checked_add(1).ok_or(Error::TooLarge)?;
    }
    while !boundary(text, end) {
        end = end.checked_sub(1).ok_or(Error::TooLarge)?;
    }
    if end == start {
        return Err(Error::TooLarge);
    }
    render(text.get(start..end).expect("page range"), Some(u32::try_from(end).or(Err(Error::TooLarge))?))
}

fn escaped_len(text: &[u8]) -> usize {
    let mut size = 0_usize;
    for byte in text {
        size = size.saturating_add(match *byte {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' => 2,
            0..=31 => 6,
            32..=255 => 1,
        });
    }
    size
}

fn render(body: &[u8], next: Option<u32>) -> Result<Box<[u8]>, Error> {
    let marker = match next {
        Some(offset) => Decimal::of(u64::from(offset)).as_bytes().len().checked_add(7).ok_or(Error::TooLarge)?,
        None => 4,
    };
    let separator = usize::from(!body.is_empty() && !body.ends_with(b"\n"));
    let size = body.len().checked_add(separator).ok_or(Error::TooLarge)?;
    let mut out = Writer::new(size.checked_add(marker).ok_or(Error::TooLarge)?);
    out.put(body).expect("page room");
    if !body.is_empty() && !body.ends_with(b"\n") {
        out.put(b"\n").expect("separator room");
    }
    match next {
        Some(next) => {
            out.put(b"next: ").expect("marker room");
            out.put(Decimal::of(u64::from(next)).as_bytes()).expect("offset room");
            out.put(b"\n").expect("marker room");
        }
        None => out.put(b"end\n").expect("marker room"),
    }
    Ok(out.finish())
}

fn boundary(text: &[u8], at: usize) -> bool {
    match text.get(at) {
        Some(byte) => byte & 0xc0 != 0x80,
        None => true,
    }
}
