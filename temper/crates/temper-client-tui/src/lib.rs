//! The TUI's pages on temper's client root (temper/docs/design/client/README.md, 6.11 to 6.17).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod approval;
mod box_line;
mod event;
pub mod limits;
pub mod look;
mod note;
mod state;
pub mod theme;
mod view;
pub mod words;
pub use approval::{approval_address, may_ask_again};
pub use box_line::BoxLine;
pub use event::{Event, LOG_COMMAND};
pub use limits::limits;
pub use note::Note;
pub use state::{BoxState, Browser, Local, Refusal, Signing, Told, Tui};
pub use theme::{Scheme, theme};
pub use view::{View, build, escape, event, offered, work_runs};

#[cfg(test)]
mod tests;
