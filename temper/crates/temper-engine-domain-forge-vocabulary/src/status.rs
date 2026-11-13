//! Project landings, sittings and briefs (temper/docs/design/domain/forge/attention.md, 2, 3).
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

/// A new sitting past this (temper/docs/design/domain/forge/attention.md, 2).
pub const SITTING: Duration = Duration::from_secs(1800);
pub const BRIEF_SENTENCE_BYTES: usize = 200;
pub const BRIEF_LEDE_BYTES: usize = 600;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Brief {
    pub sentence: Box<[u8]>,
    pub lede: Box<[u8]>,
    pub at: Wall,
    pub landed: u64,
}
/// A status run in flight (temper/docs/design/domain/forge/attention.md, 3); task none until the root has made it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StatusRunning {
    pub task: Option<u64>,
    pub landed: u64,
    pub since: Wall,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectFacts {
    pub project: u32,
    pub landed: u64,
    pub last_landed: Wall,
    pub brief: Option<Brief>,
    pub running: Option<StatusRunning>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Looked {
    pub party: u64,
    pub project: u32,
    pub previous: Option<Wall>,
    pub latest: Wall,
    pub seen: u64,
    pub next_seen: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LookedAnswer {
    Looked,
    TooLarge,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StatusView {
    pub project: u32,
    pub landed: u64,
    pub last_landed: Option<Wall>,
    pub brief: Option<Brief>,
    pub previous: Option<Wall>,
    pub seen: u64,
    pub queued: u32,
    pub red: bool,
    pub branch: Box<[u8]>,
}
/// The subject status, beside `changes_subject`.
#[must_use]
pub fn status_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 34) | u64::from(project)
}
/// The first paragraph and remaining words of a status report (temper/docs/design/domain/forge/attention.md, 3).
#[must_use]
pub fn status_paragraphs(words: &[u8]) -> (&[u8], &[u8]) {
    let mut from = 0;
    for (at, byte) in words.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        let after = at.checked_add(1).expect("bounded paragraph cursor");
        if words.get(from..at).expect("report line").trim_ascii().is_empty() {
            return (
                words.get(..from).expect("first paragraph").trim_ascii(),
                words.get(after..).expect("remaining paragraphs").trim_ascii(),
            );
        }
        from = after;
    }
    (words.trim_ascii(), b"")
}
