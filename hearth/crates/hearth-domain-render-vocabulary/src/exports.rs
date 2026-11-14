//! A task's typed exports depend on its settings and target alone (hearth/docs/design/protocol/format/README.md, 6.1).
use crate::evaluate::Entry;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exported {
    pub task: Box<[u8]>,
    pub values: Box<[Entry]>,
}
