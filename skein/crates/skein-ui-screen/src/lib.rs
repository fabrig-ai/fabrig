//! The full-screen shell's theme, frame and focus (skein/docs/design/ui-screen.md, sections 3 to 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(
    clippy::disallowed_macros,
    reason = "the focus contract requires deriving Default for no node and zero offsets"
)]

extern crate alloc;

mod focus;
mod frame;
mod render;
mod theme;
mod width;

pub use focus::Focus;
pub use frame::{Cell, Frame, Position, Size};
pub use render::render;
pub use theme::{Align, Arrange, Border, Glyphs, Paint, Place, RegionLook, Rgb, StyleLook, Theme};
pub use width::width;

#[cfg(test)]
mod tests;
