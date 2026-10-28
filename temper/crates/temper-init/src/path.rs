//! Paths are checked before any output is built (protocol/setup.md, 8.2).
use crate::{Fact, Reason, Refusal};
use alloc::boxed::Box;
use skein_lib::Writer;

#[expect(clippy::disallowed_methods, reason = "init is a protocol boundary: its paths must be JSON text")]
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

pub(crate) fn join(directory: &[u8], name: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(joined_len(directory, name));
    writer.put(directory).expect("the directory was counted");
    writer.put(separator(directory, name)).expect("the separator was counted");
    writer.put(name).expect("the name was counted");
    writer.finish()
}
