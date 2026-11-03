//! Application-owned settings (jig/docs/design/domain/engine.md, 17.5).

use alloc::boxed::Box;

/// Where a kind's values are kept (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KindScope {
    Deployment,
    Project,
}

/// A declared application kind and its bounds (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Kind {
    pub number: u32,
    pub scope: KindScope,
    pub connector: u16,
    pub value_bytes: u32,
    pub per_scope: u32,
    pub charters: u8,
}

/// An edit whose value stays with its connector (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Box<[u8]>),
    Remove,
    Reset,
}

/// One application setting handed to its connector (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: Edit,
}
