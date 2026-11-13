//! A goal's reds (temper/docs/design/domain/forge/reds.md, 2, 3 and 6).
pub use crate::Filer as Recorder;
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Red {
    pub goal: u64,
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
    pub why: Box<[u8]>,
    pub by: Recorder,
    pub at: u64,
    pub generation: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RecordRed {
    pub goal: u64,
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
    pub why: Box<[u8]>,
    pub by: Recorder,
    pub at: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedRecorded {
    pub generation: u64,
    pub replaced: Option<Red>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RedRefusal {
    NoGoal,
    Full { bound: u32 },
    NoSuch,
}

/// The check's opaque, single-line key, without surrounding whitespace (reds.md, 2).
#[must_use]
#[expect(clippy::disallowed_methods, reason = "the red decoding boundary judges UTF-8 and whitespace")]
pub fn key_fits(key: &[u8], most: u32) -> bool {
    if !crate::settings::words_fit(key, most) {
        return false;
    }
    let Ok(text) = core::str::from_utf8(key) else { return false };
    text.trim() == text
}
