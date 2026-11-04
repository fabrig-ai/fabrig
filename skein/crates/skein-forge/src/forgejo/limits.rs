//! Bounds shared by both sides.
pub use crate::limits::{Limits, Problem};
/// The depth needed by Forgejo's documents.
pub const DEPTH: u32 = 8;
/// Checks that both sides can honour the limits.
pub fn check(limits: &Limits) -> Result<(), Problem> {
    crate::limits::check(limits, DEPTH)
}
