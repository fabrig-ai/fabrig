//! A recipe keeps its commit files (qa/places.md, 6; client/drive.md, 3.1 and 3.5).
//! Only the fields needed to copy files are read here; the place's drive tool
//! validates the whole recipe. Its grammar uses the journeys' quoting rules.
use crate::{Refusal, refused};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path};

pub(super) fn copy(lines: &[String], source: &Path, destination: &Path) -> Result<(), Refusal> {
    for line in lines {
        let Some(tail) = line.strip_prefix("forge commit forgejo ") else { continue };
        let mut fields = Fields { rest: tail.as_bytes() };
        fields.next().and_then(|_| fields.next()).map_err(field_refusal)?;
        while !fields.rest.is_empty() {
            let bytes = fields.next().map_err(field_refusal)?;
            if !(1..=256).contains(&bytes.len())
                || bytes.contains(&0)
                || bytes.split(|byte| *byte == b'/').any(|part| part.is_empty() || part == b"." || part == b"..")
            {
                return Err(refused(format!("recipe files: invalid path {}", String::from_utf8_lossy(&bytes))));
            }
            let path = OsString::from_vec(bytes);
            let from = source.join(&path);
            let to = destination.join(&path);
            let metadata = super::io(&from, fs::symlink_metadata(&from))?;
            if !metadata.is_file() {
                return Err(refused(format!("{}: not a regular file", from.display())));
            }
            let parent = to.parent().expect("recipe file's parent");
            super::io(parent, fs::create_dir_all(parent))?;
            super::io(&from, fs::copy(&from, &to))?;
        }
    }
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
