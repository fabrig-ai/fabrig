//! Bounds shared by GitHub's two sides (forge/github.md, section 2).
pub use crate::limits::{Limits, Problem};
/// The depth needed by a logged delivery's wrapped payload.
pub const DEPTH: u32 = 10;
/// Checks that both sides can honour the limits.
pub fn check(limits: &Limits) -> Result<(), Problem> {
    crate::limits::check(limits, DEPTH)
}
