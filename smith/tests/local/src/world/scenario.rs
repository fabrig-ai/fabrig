//! Facts for each scripted local-host story (testing-strategy.md, section 10).
//! Constructors read one row rather than maintaining independent scenario groups.

/// One scripted local-host behavior, independent of persistence and capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    /// A chat without files.
    Chat,
    /// Finish before reading a follow-up.
    Finishing,
    /// A report from a plain workspace.
    Workspace,
    /// Delegate a read-only task.
    Delegating,
    /// Deliver a changed plain directory.
    PlainChange,
    /// Deliver an unchanged plain directory.
    PlainNothing,
    /// Commit a checked change.
    GitChange,
    /// Refuse remaining merge markers.
    GitMarker,
    /// Push a committed change.
    PushLands,
    /// Refuse a stale remote.
    PushStale,
    /// Fail after the first of two commits.
    SecondFails,
    /// Deliver before a final report.
    MidReport,
    /// Deliver twice before a final report.
    TwoDeliveries,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Files {
    None,
    Plain,
    Git,
    Merge,
    TwoGit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResultKind {
    Report,
    Change,
    MidReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    None,
    Current,
    Stale,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Facts {
    pub(super) files: Files,
    pub(super) result: ResultKind,
    pub(super) instructions: &'static [u8],
    pub(super) checks: bool,
    pub(super) delegate: bool,
    pub(super) target: Target,
}

const CHAT: Facts = Facts {
    files: Files::None,
    result: ResultKind::Report,
    instructions: b"@chat Assist the person.",
    checks: false,
    delegate: false,
    target: Target::None,
};
const CHANGE: Facts = Facts {
    files: Files::Git,
    result: ResultKind::Change,
    instructions: b"@local-change Make a checked change.",
    checks: true,
    ..CHAT
};

impl Scenario {
    pub(super) fn facts(self) -> Facts {
        match self {
            Self::Chat => CHAT,
            Self::Finishing => Facts { instructions: b"@local-finish", ..CHAT },
            Self::Workspace => Facts {
                files: Files::Plain,
                checks: true,
                instructions: b"@local-workspace Assist the person.",
                ..CHAT
            },
            Self::Delegating => Facts {
                files: Files::Plain,
                checks: true,
                delegate: true,
                instructions: b"@local-delegate Delegate a reading, then answer.",
                ..CHAT
            },
            Self::PlainChange => Facts { files: Files::Plain, ..CHANGE },
            Self::PlainNothing => Facts {
                files: Files::Plain,
                checks: false,
                instructions: b"@local-nothing Declare an unchanged result.",
                ..CHANGE
            },
            Self::GitChange => CHANGE,
            Self::GitMarker => Facts {
                files: Files::Merge,
                checks: false,
                instructions: b"@local-marker Resolve the merge.",
                ..CHANGE
            },
            Self::PushLands => Facts { target: Target::Current, ..CHANGE },
            Self::PushStale => Facts { target: Target::Stale, ..CHANGE },
            Self::SecondFails => Facts { files: Files::TwoGit, ..CHANGE },
            Self::MidReport => {
                Facts { result: ResultKind::MidReport, instructions: b"@midreport Deliver then report.", ..CHANGE }
            }
            Self::TwoDeliveries => Facts {
                result: ResultKind::MidReport,
                instructions: b"@local-two-deliveries Deliver twice, then report.",
                ..CHANGE
            },
        }
    }
}
