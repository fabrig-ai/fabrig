//! Landed drafts (temper/docs/design/domain/forge/drafts.md, sections 1–3).
use alloc::boxed::Box;
use skein_forge::api::{self, Commit};
use skein_lib::Wall;

pub const DRAFTS: u32 = 64;
#[must_use]
pub fn drafts_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 39) | u64::from(project)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftRepository {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DraftChange {
    Ready { chat: Option<u64> },
    NotReady,
    Started { goal: u64 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SetDraft {
    pub project: u32,
    pub repository: Option<DraftRepository>,
    pub path: Box<[u8]>,
    pub change: DraftChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandedDraft {
    pub repository: api::Repository,
    pub path: Box<[u8]>,
    pub project: u32,
    pub branch: Box<[u8]>,
    pub commit: Commit,
    pub title: Box<[u8]>,
    pub writer: u64,
    pub at: Wall,
    pub chat: Option<u64>,
    pub goal: Option<u64>,
    pub ready: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DraftRefusal {
    NoRepository,
    Path,
    Missing,
    NotAFile,
    Unreachable,
    Unknown,
    Full,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SetDraftAnswer {
    Kept(DraftView),
    Refused(DraftRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftView {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub row: LandedDraft,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftsView {
    pub project: u32,
    pub rows: Box<[DraftView]>,
}
#[must_use]
#[expect(clippy::disallowed_methods, reason = "drafts.md requires validating the UTF-8 path at connector admission")]
pub fn draft_path(path: &[u8]) -> bool {
    if path.is_empty() || path.len() > 1024 || !path.ends_with(b".md") || core::str::from_utf8(path).is_err() {
        return false;
    }
    let mut start = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            if !segment(path.get(start..index).expect("path segment")) {
                return false;
            }
            start = index.saturating_add(1);
        }
    }
    segment(path.get(start..).expect("last segment"))
}
fn segment(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes != b"." && bytes != b".."
}
