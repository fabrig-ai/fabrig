//! Patching at open (protocol/store/opening.md 2 and 3): the owners' patch
//! sets, the plan across them, one machine per kept family, a record's
//! walk, and the phase that runs them over a copy.
pub mod baseline;
pub(crate) mod check;
pub(crate) mod machines;
pub(crate) mod phase;
pub(crate) mod plan;
pub mod sets;
pub(crate) mod switch;
pub(crate) mod walk;

/// Records rewritten and page commits made by one patch (protocol/store/opening.md 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Counts {
    pub records: u64,
    pub commits: u64,
}
