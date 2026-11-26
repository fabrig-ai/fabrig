//! Application settings items, judgments and defaults (jig/docs/design/domain/connectors.md, 2).

use alloc::boxed::Box;

/// A default a connector stored that the application no longer gives
/// (jig/docs/design/domain/engine/settings-application.md, 1; jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DroppedDefault {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
}

/// A connector's judgment of one item (jig/docs/design/domain/engine/settings-application.md, 1;
/// jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Judged {
    Accept { size: u32, charters: Box<[u32]> },
    Same,
    Refuse { why: Box<[u8]> },
}

/// A connector's default metadata (jig/docs/design/domain/engine/settings-application.md, 1;
/// jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DefaultState {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub size: u32,
    pub charters: Box<[u32]>,
    pub differs: bool,
}

/// An edit whose value stays with its connector (jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Box<[u8]>),
    Remove,
    Reset,
}

/// One application setting handed to its connector (jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: Edit,
}

/// An application item a restore proposes (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Restored {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: RestoredEdit,
}

/// Where a restore takes an application's value (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RestoredEdit {
    Set { revision: u64 },
    Remove,
    Reset,
}
