//! Task log names and history records (jig/docs/design/protocol/store/logs.md, 1–3).
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

pub mod record;
pub(crate) mod write;

pub const EXTENTS: [u8; 2] = [0, 9];

#[must_use]
pub fn history(task: u64) -> Option<u64> {
    task.checked_mul(2)
}

#[must_use]
pub fn turns(task: u64) -> Option<u64> {
    history(task)?.checked_add(1)
}

pub(crate) fn extent_key(log: u64) -> Box<[u8]> {
    let mut writer = Writer::new(10);
    writer.put(&EXTENTS).expect("fixed extent key");
    writer.put(&log.to_be_bytes()).expect("fixed extent key");
    writer.finish()
}

pub(crate) fn extent(bytes: &[u8]) -> Option<u64> {
    let mut reader = Reader::new(bytes);
    let length = reader.u64()?;
    if !reader.is_empty() {
        return None;
    }
    Some(length)
}

/// Skein-kv's extent row has no component-family head (logs.md, 1).
pub(crate) fn extent_row(key: &[u8], value: &[u8]) -> Option<(u64, u64)> {
    let mut reader = Reader::new(key);
    if reader.u16()? != 9 {
        return None;
    }
    let log = reader.u64()?;
    if !reader.is_empty() {
        return None;
    }
    Some((log, extent(value)?))
}
