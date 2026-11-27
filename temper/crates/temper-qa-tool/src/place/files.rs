//! A recipe keeps the files named by forge commit, forge push, deployment patch,
//! ctl settings apply and ctl settings propose (temper/docs/design/testing/qa/places.md, 6;
//! temper/docs/design/client/drive.md, 3.4 and 3.5).
//! Read fields to find the files to copy, using the journeys' quoting rules;
//! the place's drive tool validates the whole recipe.
use crate::faces::{DirectoryMode, FilesFace, PathKind};
use crate::{Refusal, refused};
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::Path};

pub(super) fn copy(
    lines: &[String],
    source: &Path,
    destination: &Path,
    files: &mut impl FilesFace,
) -> Result<(), Refusal> {
    for line in lines {
        if let Some(tail) = line.strip_prefix("ctl ") {
            if let Some(paths) = settings_paths(tail) {
                for path in paths {
                    copy_path(path, source, destination, files)?;
                }
            }
            continue;
        }
        let (tail, skip) = if let Some(tail) = line.strip_prefix("forge commit forgejo ") {
            (tail, 2)
        } else if let Some(tail) = line.strip_prefix("forge push forgejo ") {
            (tail, 3)
        } else if let Some(tail) = line.strip_prefix("deployment patch ") {
            (tail, 1)
        } else {
            continue;
        };
        let mut fields = Fields { rest: tail.as_bytes() };
        for _ in 0..skip {
            fields.next().map_err(field_refusal)?;
        }
        while !fields.rest.is_empty() {
            let path = fields.next().map_err(field_refusal)?;
            copy_path(path, source, destination, files)?;
        }
    }
    Ok(())
}

// Read through the flags too, so a malformed settings line is left to recipe hash.
fn settings_paths(tail: &str) -> Option<Vec<Vec<u8>>> {
    let mut fields = Fields { rest: tail.as_bytes() };
    if fields.next().ok()? != b"settings" || !matches!(fields.next().ok()?.as_slice(), b"apply" | b"propose") {
        return None;
    }
    let mut paths = Vec::new();
    let mut before_flag = true;
    while !fields.rest.is_empty() {
        let field = fields.next().ok()?;
        if field == b"-" {
            continue;
        }
        if field.starts_with(b"-") {
            before_flag = false;
        }
        if before_flag {
            paths.push(field);
        }
    }
    Some(paths)
}

fn copy_path(bytes: Vec<u8>, source: &Path, destination: &Path, files: &mut impl FilesFace) -> Result<(), Refusal> {
    if !(1..=256).contains(&bytes.len())
        || bytes.contains(&0)
        || bytes.split(|byte| *byte == b'/').any(|part| part.is_empty() || part == b"." || part == b"..")
    {
        return Err(refused(format!("recipe files: invalid path {}", String::from_utf8_lossy(&bytes))));
    }
    let path = OsString::from_vec(bytes);
    let from = source.join(&path);
    let to = destination.join(&path);
    let metadata = super::io(&from, files.facts(&from, false))?;
    if metadata.kind != PathKind::File {
        return Err(refused(format!("{}: not a regular file", from.display())));
    }
    let parent = to.parent().expect("recipe file's parent");
    super::io(parent, files.directory(parent, DirectoryMode::Parents))?;
    super::io(&from, files.copy(&from, &to))?;
    Ok(())
}

fn field_refusal(why: &str) -> Refusal {
    refused(format!("recipe files: {why}"))
}

struct Fields<'a> {
    rest: &'a [u8],
}
impl Fields<'_> {
    fn next(&mut self) -> Result<Vec<u8>, &'static str> {
        let bytes = if let Some(rest) = self.rest.strip_prefix(b"\"") {
            self.rest = rest;
            self.quoted()?
        } else {
            let end = self.rest.iter().position(|byte| *byte == b' ').unwrap_or(self.rest.len());
            let (word, rest) = self.rest.split_at(end);
            if word.is_empty()
                || !word.iter().all(|byte| (0x21..=0x7e).contains(byte) && *byte != b'"' && *byte != b'\\')
            {
                return Err("invalid word");
            }
            self.rest = rest;
            word.to_vec()
        };
        if !self.rest.is_empty() {
            self.rest = self.rest.strip_prefix(b" ").ok_or("expected space")?;
            if self.rest.is_empty() {
                return Err("trailing space");
            }
        }
        Ok(bytes)
    }

    fn byte(&mut self) -> Result<u8, &'static str> {
        let (&byte, rest) = self.rest.split_first().ok_or("unclosed quote")?;
        self.rest = rest;
        Ok(byte)
    }

    fn quoted(&mut self) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::new();
        loop {
            match self.byte()? {
                b'"' => return Ok(bytes),
                b'\\' => match self.byte()? {
                    escape @ (b'"' | b'\\') => bytes.push(escape),
                    b'x' => {
                        let high = hex(self.byte()?)?;
                        let low = hex(self.byte()?)?;
                        let byte = high * 16 + low;
                        if (0x20..=0x7e).contains(&byte) {
                            return Err("escaped printable byte");
                        }
                        bytes.push(byte);
                    }
                    _ => return Err("bad escape"),
                },
                byte @ 0x20..=0x7e => bytes.push(byte),
                _ => return Err("unescaped byte"),
            }
        }
    }
}

fn hex(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("bad hex digit"),
    }
}
