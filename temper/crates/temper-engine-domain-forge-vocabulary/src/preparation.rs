//! The worker's per-member prepare outcomes (domain/landing.md, 5.2).
use alloc::boxed::Box;
use temper_engine_domain_forge_landing::Outcome;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Prepared {
    /// Position in the landing, counted from zero.
    pub member: u16,
    pub outcome: Outcome,
    pub files: Box<[u8]>,
}
