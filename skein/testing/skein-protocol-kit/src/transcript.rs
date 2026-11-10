//! Kept transcripts and their text (skein/docs/design/testing/README.md, 6).

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// A transcript's bytes and optional expectation, before world-specific parsing.
pub struct Kept {
    pub name: String,
    pub bytes: Vec<u8>,
    pub expect: Option<Vec<u8>>,
}

/// Read transcripts of `extension`, sorted by name, with any expectations beside them.
#[must_use]
pub fn kept(directory: &Path, extension: &str) -> Vec<Kept> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory).expect("the transcripts directory") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_some_and(|found| found == extension) {
            names.push(path.file_stem().expect("a name").to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let bytes = fs::read(directory.join(format!("{name}.{extension}"))).expect("the transcript");
            let expect = fs::read(directory.join(format!("{name}.expect"))).ok();
            Kept { name, bytes, expect }
        })
        .collect()
}

/// The first word of `line`, and the rest after the spaces that follow it.
#[must_use]
pub fn word(line: &[u8]) -> (&[u8], &[u8]) {
    match line.iter().position(|&byte| byte == b' ') {
        Some(space) => (&line[..space], line[space + 1..].trim_ascii_start()),
        None => (line, &[]),
    }
}

/// Text in quotes, as an expectation writes it: printable ASCII as it is,
/// and the rest escaped.
#[must_use]
pub fn quote(text: &[u8]) -> String {
    let mut out = String::from("\"");
    for chunk in text.utf8_chunks() {
        for character in chunk.valid().chars() {
            match character {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                _ if character.is_control() => {
                    let mut buffer = [0; 4];
                    for byte in character.encode_utf8(&mut buffer).as_bytes() {
                        write!(out, "\\x{byte:02x}").expect("writing to a String");
                    }
                }
                _ => out.push(character),
            }
        }
        for byte in chunk.invalid() {
            write!(out, "\\x{byte:02x}").expect("writing to a String");
        }
    }
    out.push('"');
    out
}

/// The text of `"..."`, unescaped, which must be all of `quoted`.
#[must_use]
pub fn unquote(quoted: &[u8]) -> Option<Vec<u8>> {
    match self::quoted(quoted)? {
        (text, []) => Some(text),
        _ => None,
    }
}

/// The text of the `"..."` `line` begins with, unescaped, and the rest
/// after it.
#[must_use]
pub fn quoted(line: &[u8]) -> Option<(Vec<u8>, &[u8])> {
    let inner = line.strip_prefix(b"\"")?;
    let mut text = Vec::new();
    let mut at = 0;
    while at < inner.len() {
        let byte = inner[at];
        at += 1;
        match byte {
            b'"' => return Some((text, inner[at..].trim_ascii_start())),
            b'\\' => {
                let escape = *inner.get(at)?;
                at += 1;
                match escape {
                    b'\\' => text.push(b'\\'),
                    b'"' => text.push(b'"'),
                    b'n' => text.push(b'\n'),
                    b'r' => text.push(b'\r'),
                    b't' => text.push(b'\t'),
                    b'x' => {
                        let digits = std::str::from_utf8(inner.get(at..at + 2)?).ok()?;
                        text.push(u8::from_str_radix(digits, 16).ok()?);
                        at += 2;
                    }
                    _ => return None,
                }
            }
            _ => text.push(byte),
        }
    }
    None
}
