//! The view's events (temper/docs/design/client/README.md, 6.14 and 6.17).
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },
    SignIn,
    Send,
    Command { command: temper_client_domain::pages::terminal::Command },
    Act { act: temper_client_domain::pages::Act },
    Aside { object: jig_client::contract::ObjectKey },
    Dismiss,
    Stop,
    Stay,
    Log,
    LeftOff { task: Option<u64> },
    Own { command: Own },
    Open { address: Box<[u8]> },
    Walk { object: jig_client::contract::ObjectKey, step: WalkStep },
    EndWalk,
}

/// The TUI's own command (temper/docs/design/client/README.md, 6.17).
pub const LOG_COMMAND: &[u8] = b"/log";

/// The TUI's own commands (temper/docs/design/client/README.md, 6.14).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Own {
    Change,
    Ask,
    Brief,
    Needs,
    Open,
}
impl Own {
    pub const ALL: [Own; 5] = [Own::Change, Own::Ask, Own::Brief, Own::Needs, Own::Open];
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Change => b"/change",
            Self::Ask => b"/ask",
            Self::Brief => b"/brief",
            Self::Needs => b"/needs",
            Self::Open => b"/open",
        }
    }
}
/// An act on one need's prompt (temper/docs/design/client/README.md, 6.14).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WalkStep {
    Choose { words: Box<[u8]> },
    Open { address: Box<[u8]> },
    Next,
}
