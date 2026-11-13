//! Resource roles and hold admission (jig's domain/connectors.md, section 3).
//! Forge repositories expose no counted pool resources. Landing branches and
//! participating objects are shared; private task branches have one holder.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldKind {
    Shared,
    Exclusive { wait: bool },
}
/// Admission rule for the forge's private branch kind.
pub const BRANCH_HOLD: HoldKind = HoldKind::Exclusive { wait: true };

/// Admission rule for a goal's own branch (domain/forge/branches.md, 1).
pub const GOAL_BRANCH_HOLD: HoldKind = HoldKind::Exclusive { wait: false };
