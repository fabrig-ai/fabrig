//! Whole history rows in source page order (client/domain/contract/README.md, 4).
use super::{AttemptLine, HistoryLine, HoldLine};
use alloc::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HistoryRow {
    /// A semantic change named by its task and revision.
    Change(Box<HistoryLine>),
    /// A run named by its task and attempt number.
    Attempt(Box<AttemptLine>),
    /// A held phase named by its task and phase change position.
    Hold(Box<HoldLine>),
}
