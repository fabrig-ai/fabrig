//! Bounded source text values (jig/docs/design/client/domain/contract/README.md, 4).
use alloc::boxed::Box;

/// Which source words a task text read retrieves.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskTextField {
    /// The task's stored specification words.
    Specification,
    /// Its override, or the charter instructions at its recorded settings revision.
    Instructions,
    /// The kept terminal report words, without overview truncation.
    Result,
}
/// One UTF-8 chunk with byte offsets into the complete source.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskText {
    pub task: u64,
    pub field: TaskTextField,
    pub from: u64,
    pub total: u64,
    pub next: Option<u64>,
    pub bytes: Box<[u8]>,
}
