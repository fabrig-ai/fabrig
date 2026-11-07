//! Full-screen keys and drawing (skein/docs/design/ui-screen.md, 6 to 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod keys;
pub use keys::{Decoder, Key};
mod command;
pub use command::{Command, command};
mod draw;
pub use draw::{Depth, ENTER, LEAVE, draw, draw_bound};
#[cfg(test)]
mod tests;
