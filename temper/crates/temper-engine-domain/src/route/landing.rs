//! Test worlds' landing rules translated to the forge's defaults. These values
//! belong to the application; people carries only authority requirements
//! (domain/forge/change/gates.md, 2; jig's domain/authority.md, sections 6 and 10).

use alloc::boxed::Box;
use jig_domain_authority::Pattern;

/// Validity of a forge gate or review verdict at a later landing head.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Freshness {
    Exact,
    Clean,
}

/// One numbered required or advisory forge gate.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Gate {
    pub number: u32,
    pub blocking: bool,
    pub freshness: Freshness,
}

/// Required count of distinct reviewers with a project role.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Approval {
    pub role: u32,
    pub people: u32,
    pub freshness: Freshness,
}

/// Forge branch checks for test worlds' builders alone.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingRule {
    pub connector: u16,
    pub kind: u16,
    pub pattern: Pattern,
    /// Whether this rule holds a merge through the standing requirements.
    pub enforce: bool,
    pub ci: bool,
    pub up_to_date: bool,
    pub gates: Box<[Gate]>,
    pub approvals: Box<[Approval]>,
    pub sets: Box<[u8]>,
}
