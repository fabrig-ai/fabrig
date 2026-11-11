//! The view's events (temper/docs/design/client/tui/pages.md and temper/docs/design/client/tui/box.md).
use alloc::boxed::Box;
use temper_client_domain::pages::terminal::Command;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },
    SignIn,
    Send,
    Command { command: Command },
    Act { act: temper_client_domain::pages::Act },
    Aside { object: jig_client::contract::ObjectKey },
    Answer { words: Box<[u8]> },
    ReplyAside { turn: jig_client::contract::TurnName },
    Dismiss,
    Stop,
    Stay,
    Log,
    LeftOff { task: Option<u64> },
    Own { command: Own },
    Open { address: Box<[u8]> },
    Walk { object: jig_client::contract::ObjectKey, step: WalkStep },
    EndWalk,
    Theme { choice: Option<crate::theme::ThemeChoice> },
    Way { template: u32 },
}

/// The TUI's own command (temper/docs/design/client/tui/box.md).
pub const LOG_COMMAND: &[u8] = b"/log";

/// The TUI's closing command (temper/docs/design/client/tui/pages.md).
pub const QUIT: &[u8] = b"/quit";
/// Sent whole, the same closing command, never listed (client/tui/pages.md).
pub const EXIT: &[u8] = b"/exit";

/// The TUI's own commands (temper/docs/design/client/tui/pages.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Own {
    Draft,
    Change,
    Ask,
    Brief,
    Needs,
    Open,
    Theme,
}
impl Own {
    pub const ALL: [Own; 7] = [Own::Draft, Own::Change, Own::Ask, Own::Brief, Own::Needs, Own::Open, Own::Theme];
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Draft => b"/draft",
            Self::Change => b"/change",
            Self::Ask => b"/ask",
            Self::Brief => b"/brief",
            Self::Needs => b"/needs",
            Self::Open => b"/open",
            Self::Theme => b"/theme",
        }
    }
}
/// The TUI's name of a root command: `QUIT` for `Command::Exit`, else `command.name()`.
#[must_use]
pub fn command_name(command: Command) -> &'static [u8] {
    match command {
        Command::Exit => QUIT,
        Command::Model
        | Command::Effort
        | Command::Draft
        | Command::Pause
        | Command::Close
        | Command::StartOver
        | Command::Goal
        | Command::New
        | Command::Chats
        | Command::Theme
        | Command::Font
        | Command::Help => command.name(),
    }
}
/// An act on one need's prompt (temper/docs/design/client/tui/pages.md).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WalkStep {
    Choose { words: Box<[u8]> },
    Open { address: Box<[u8]> },
    Next,
}
