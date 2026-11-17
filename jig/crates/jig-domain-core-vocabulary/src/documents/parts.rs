//! Application parts of task rows (jig/docs/design/domain/engine/views.md, 2).

use super::TaskLine;
use alloc::boxed::Box;
use skein_lib::{List, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent declared inputs of a part kind, domain/engine/views.md, 2"
)]
pub struct PartInputs {
    pub line: bool,
    pub notes: bool,
    pub delegates: bool,
    pub delegate_notes: bool,
    pub records: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartKind {
    pub owner: u16,
    pub kind: u32,
    pub connector: u16,
    pub bytes: u32,
    pub inputs: PartInputs,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartAsk {
    pub owner: u16,
    pub kind: u32,
    pub version: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PartFailure {
    Busy,
    TooLarge,
    Unreadable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RowPart {
    Made(Box<[u8]>),
    Unknown,
    Failed(PartFailure),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MadePart {
    Made(Box<[u8]>),
    Failed(PartFailure),
}

/// Row-major request; each row's kinds retain the query's order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartsAsk {
    pub token: Token,
    pub party: u64,
    pub kinds: List<PartAsk>,
    pub rows: List<TaskLine>,
}
