//! Pure goal closer reports (temper/docs/design/domain/forge/branches.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod policy;
mod report;
pub mod relay;
mod selection;
pub use policy::{Child, Closer, Decision, Facts, Hold, Running, Step, Stepped, step};
pub use selection::{PullSelection, SELECTION_JOBS, SelectionProgress, selection_answer, selection_read};
#[cfg(test)]
mod tests;

pub use report::{
    LOG_START_BYTES, Landed, Landing, Report, SELECTION_LINE_BYTES, SELECTION_PREFIX, Selection, Unlanded, Why,
    first_line, left_section, selection_line, words,
};
