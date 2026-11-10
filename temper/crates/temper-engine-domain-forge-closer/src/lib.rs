//! Pure goal closer reports (temper/docs/design/domain/forge.md, 11.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod report;
#[cfg(test)]
mod tests;

pub use report::{
    LOG_START_BYTES, Landed, Landing, Report, SELECTION_LINE_BYTES, SELECTION_PREFIX, Selection, Unlanded, Why,
    first_line, left_section, selection_line, words,
};
