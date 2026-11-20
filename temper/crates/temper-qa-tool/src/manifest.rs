//! Writing and reading sha256sum's byte-oriented manifest rows (qa/builds.md, 4).
use crate::{Refusal, refused};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Component, Path, PathBuf},
};

/// Write a checksum row, escaping backslashes, newlines and carriage returns.
pub(crate) fn line(hex: &str, path: &Path) -> Vec<u8> {
    let name = path.as_os_str().as_bytes();
    let escaped = name.iter().any(|byte| matches!(byte, b'\\' | b'\n' | b'\r'));
    let mut line = Vec::new();
    if escaped {
        line.push(b'\\');
    }
    line.extend_from_slice(hex.as_bytes());
    line.extend_from_slice(b"  ");
    for byte in name {
        match byte {
            b'\\' => line.extend_from_slice(b"\\\\"),
            b'\n' => line.extend_from_slice(b"\\n"),
            b'\r' => line.extend_from_slice(b"\\r"),
            _ => line.push(*byte),
        }
    }
    line.push(b'\n');
    line
}

/// Read a kept manifest with its required commit line.
pub(crate) fn parse(bytes: &[u8], commit: &str) -> Result<BTreeMap<PathBuf, String>, Refusal> {
    let mut lines = bytes.splitn(2, |byte| *byte == b'\n');
    if lines.next() != Some(format!("commit {commit}").as_bytes()) {
        return Err(refused("commit differs from build"));
    }
    rows(lines.next().unwrap_or_default())
}

/// Read checksum rows in path order, without a commit line.
pub(crate) fn rows(bytes: &[u8]) -> Result<BTreeMap<PathBuf, String>, Refusal> {
    let mut files = BTreeMap::new();
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let (escaped, line) = line.strip_prefix(b"\\").map_or((false, line), |rest| (true, rest));
        let (hash, path) = line.split_at_checked(64).ok_or_else(|| refused("invalid checksum line"))?;
        let path = path.strip_prefix(b"  ").ok_or_else(|| refused("invalid checksum line"))?;
        if !hash.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            return Err(refused("invalid checksum"));
        }
        let mut name = Vec::new();
        let mut bytes = path.iter();
        while let Some(byte) = bytes.next() {
            name.push(if escaped && *byte == b'\\' {
                match bytes.next() {
                    Some(b'\\') => b'\\',
                    Some(b'n') => b'\n',
                    Some(b'r') => b'\r',
                    Some(_) | None => return Err(refused("invalid escaped path")),
                }
            } else {
                *byte
            });
        }
        let path = PathBuf::from(OsString::from_vec(name));
        if path.as_os_str().is_empty() || path.components().any(|part| !matches!(part, Component::Normal(_))) {
            return Err(refused("invalid manifest path"));
        }
        let hash = String::from_utf8(hash.to_vec()).map_err(|why| refused(why.to_string()))?;
        if files.insert(path.clone(), hash).is_some() {
            return Err(refused(format!("{}: duplicate manifest path", path.display())));
        }
    }
    Ok(files)
}
