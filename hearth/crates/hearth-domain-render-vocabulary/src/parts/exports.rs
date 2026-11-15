//! Canonical exports, with integers as decimal text (hearth/docs/design/domain/render/spec.md, 1.4, 1.6).
use crate::{evaluate::Value, exports::Exported};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exports {
    pub tasks: Box<[Exported]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub entries: Box<[ExportChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ExportChange {
    pub task: Box<[u8]>,
    pub name: Box<[u8]>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
