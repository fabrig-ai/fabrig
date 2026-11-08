//! Top-level view blocks in committed text (temper/docs/design/client/README.md, 5.16).
use crate::Limits;
use skein_lib::List;

/// One part of a text: Markdown to read, or a view block's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part<'a> {
    Text(&'a str),
    View(&'a [u8]),
}

struct Fence {
    byte: u8,
    length: usize,
    start: usize,
    body: usize,
    view: bool,
    top: bool,
}

/// Split at top-level fenced blocks whose info string is exactly `view`
/// and whose body is one nonempty line (client/README.md, 5.16).
#[must_use]
pub fn parts(text: &str) -> List<Part<'_>> {
    let mut parts = List::with_capacity(Limits::view_parts());
    let mut fence: Option<Fence> = None;
    let mut offset: usize = 0;
    let mut retained: usize = 0;
    for raw in text.split_inclusive('\n') {
        let start = offset;
        offset = offset.checked_add(raw.len()).expect("line ends in text");
        let line = raw.trim_end_matches(['\r', '\n']);
        if let Some(open) = &fence {
            if let Some((byte, length, tail)) = delimiter(if open.top { line } else { container_line(line) })
                && byte == open.byte && length >= open.length && tail.trim().is_empty()
            {
                let body = text.get(open.body..start).expect("fence body in text");
                let body = body.strip_suffix('\n').unwrap_or(body);
                let body = body.strip_suffix('\r').unwrap_or(body);
                let address = body.trim();
                if open.view && !body.contains(['\n', '\r']) && !address.is_empty() && address.len() <= 512 {
                    let needed: u32 = if retained < open.start { 2 } else { 1 };
                    let tail = u32::from(offset < text.len());
                    if parts.room() < needed.saturating_add(tail) {
                        break;
                    }
                    if retained < open.start {
                        parts.push(Part::Text(text.get(retained..open.start).expect("text before fence")))
                            .expect("room for text");
                    }
                    parts.push(Part::View(address.as_bytes())).expect("room for view");
                    retained = offset;
                }
                fence = None;
            }
            continue;
        }
        let nested = container_line(line);
        if let Some((byte, length, info)) = delimiter(nested) {
            fence = Some(Fence { byte, length, start, body: offset, view: nested.len() == line.len() && info.trim() == "view", top: nested.len() == line.len() });
        }
    }
    if retained < text.len() {
        parts.push(Part::Text(text.get(retained..).expect("text remainder"))).expect("room for remainder");
    }
    parts
}

fn delimiter(line: &str) -> Option<(u8, usize, &str)> {
    let byte = *line.as_bytes().first()?;
    if byte != b'`' && byte != b'~' {
        return None;
    }
    let mut length: usize = 0;
    for current in line.bytes() {
        if current != byte { break; }
        length = length.checked_add(1).expect("delimiter in line");
    }
    if length < 3 { return None; }
    Some((byte, length, line.get(length..).expect("ASCII delimiter")))
}

// Fences in indented code, quotes and list items protect their bodies too.
fn container_line(line: &str) -> &str {
    let mut line = line.trim_start();
    for _ in 0..line.len() {
        if let Some(rest) = line.strip_prefix('>') {
            line = rest.trim_start();
        } else if let Some(rest) = list_line(line) {
            line = rest.trim_start();
        } else {
            break;
        }
    }
    line
}
fn list_line(line: &str) -> Option<&str> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return Some(rest);
        }
    }
    let mut digits: usize = 0;
    for byte in line.bytes() {
        if !byte.is_ascii_digit() { break; }
        digits = digits.checked_add(1).expect("digits in line");
    }
    if digits == 0 { return None; }
    line.get(digits..)?.strip_prefix(". ")
}
