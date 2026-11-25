//! Configuring stage one: `temper/docs/design/testing/checks-stage-one.md`, 1.

mod cli;
pub use cli::{TABLE, main};

pub mod check;
pub mod collect;
pub mod evidence;
pub mod golden;
pub mod judge;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[doc(hidden)]
pub mod testing {
    pub(crate) use crate::collect::{EvidenceSink, collect_with};
    pub(crate) use crate::evidence::{EvidenceSource, read_with};
}
