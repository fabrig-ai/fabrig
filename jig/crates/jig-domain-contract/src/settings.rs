//! Application settings items, judgments and defaults (jig/docs/design/domain/connectors.md, 2; engine/vocabulary.md, 2.6).

use alloc::boxed::Box;
use skein_lib::Token;

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

/// What the engine asks of an application settings connector.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Judgement: judge items in order for the revision they would make, answered by `Up::Judged`.
    Judge {
        owner: Token,
        revision: u64,
        /// Accepted proposal whose kept values Set items name, with empty bytes.
        proposal: Option<u64>,
        items: Box<[Item]>,
    },
    /// Judgement: judge restored values as proposal items, answered by `Up::Restored`.
    Restore { owner: Token, proposal: u64, items: Box<[Restored]> },
    /// Notification: keep committed values, using a proposal's kept values when named; no answer.
    Keep { revision: u64, proposal: Option<u64>, items: Box<[Item]> },
    /// Notification: keep a proposal's values, checked against the kinds' bounds alone; no answer.
    KeepProposal { proposal: u64, items: Box<[Item]> },
    /// Notification: erase a terminal proposal's values; no answer.
    DropProposal { proposal: u64 },
    /// Notification: erase replaced values below the history cutoff; no answer.
    ForgetReplaced { before: u64 },
    /// Notification: save the defaults, with no revision if none was needed; no answer.
    SaveDefaults { revision: Option<u64> },
}

/// A settings connector's judgements and defaults.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// At startup: a stored value of the connector's kinds it cannot hold.
    Unfit { kind: u32, scope: u32, number: u32 },
    /// The connector's judgement of the proposed items.
    Judged { owner: Token, judged: Box<[Judged]> },
    /// The connector's judgement of the restored items.
    Restored { owner: Token, judged: Box<[Judged]> },
    /// At startup: the connector's default states and those it no longer gives.
    Defaults { defaults: Box<[DefaultState]>, dropped: Box<[DroppedDefault]> },
}
