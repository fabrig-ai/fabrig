//! Paths are checked before any output is built (protocol/setup/init.md, 2).
use crate::{Fact, Reason, Refusal};

#[expect(clippy::disallowed_methods, reason = "init validates its machine paths as UTF-8")]
pub(crate) fn check(fact: Fact, path: &[u8], name: &[u8]) -> Result<(), Refusal> {
    if !path.starts_with(b"/") {
        return Err(Refusal { fact, reason: Reason::Relative });
    }
    if core::str::from_utf8(path).is_err() {
        return Err(Refusal { fact, reason: Reason::Text });
    }
    if path.len() > 4096 || joined_len(path, name) > 4096 {
        return Err(Refusal { fact, reason: Reason::Bound });
    }
    Ok(())
}

fn separator(directory: &[u8], name: &[u8]) -> &'static [u8] {
    if directory.ends_with(b"/") || name.is_empty() { b"" } else { b"/" }
}

fn joined_len(directory: &[u8], name: &[u8]) -> usize {
    directory.len().saturating_add(separator(directory, name).len()).saturating_add(name.len())
}
