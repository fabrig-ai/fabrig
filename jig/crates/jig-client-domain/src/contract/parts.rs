//! Application parts of task rows (jig/docs/design/client/domain/contract/parts.md, 3).

use super::NotServed;
use crate::boundary::Family;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PartAsk {
    pub owner: u16,
    pub kind: u32,
    pub version: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartFailure {
    Busy,
    TooLarge,
    Unreadable,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowPart {
    Value(Family),
    NotServed(NotServed),
    Unknown,
    Failed(PartFailure),
}
