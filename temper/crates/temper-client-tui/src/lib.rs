//! The TUI's pages on temper's client root (temper/docs/design/client/tui/README.md, 1, and temper/docs/design/client/tui/directory.md, 1, and temper/docs/design/client/tui/host.md, 1 and temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/tui/theme.md and temper/docs/design/client/tui/README.md, 4 and temper/docs/design/client/tui/box.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod approval;
mod box_line;
mod box_stage;
mod choices;
mod event;
pub mod limits;
pub mod look;
mod marks;
mod note;
mod origin;
mod place;
mod state;
pub mod theme;
mod view;
pub mod words;
pub use approval::{approval_address, may_ask_again, may_type_instead};
pub use box_line::BoxLine;
pub use box_stage::{BoxEnd, BoxStage};
pub use choices::{ReplyChoice, ReplyChoices, reply_choices};
pub use event::{EXIT, Event, LOG_COMMAND, Own, QUIT, WalkStep, command_name};
pub use limits::limits;
pub use marks::marked;
pub use note::Note;
pub use origin::{Origin, Unaddressed};
pub use place::place;
pub use state::{BoxState, Browser, Closing, Handshake, Lending, Local, Notice, Refusal, Signing, Told, Tui, Walk};
pub use theme::{Scheme, theme};
pub use view::{
    Arrow, HALVES, View, arrow, build, chat_number, escape, event, focus, goal_offered, landing, offered, offered_own,
    pane_key, refused, reply_number, sign_in_drawn, walk_key, work_runs,
};

#[cfg(test)]
mod tests;
