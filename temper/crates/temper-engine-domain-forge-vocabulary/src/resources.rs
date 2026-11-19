//! Resource roles and hold admission (jig's domain/connectors.md, section 3).
//! Forge repositories expose no counted pool resources. Landing branches and
//! participating objects are shared; private task branches have one holder.

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldKind {
    Shared,
    Exclusive { wait: bool },
}
/// Admission rule for the forge's private branch kind.
pub const BRANCH_HOLD: HoldKind = HoldKind::Exclusive { wait: true };

/// Admission rule for a goal's own branch (domain/forge/branches.md, 1).
pub const GOAL_BRANCH_HOLD: HoldKind = HoldKind::Exclusive { wait: false };

/// The hub sees an opaque, bounded name for a forge branch. Forge path syntax
/// stays here; the task child only compares these literal bytes.
pub fn hub_name(connector: u16, name: &crate::Name, limits: &tasks::Limits) -> Option<tasks::Name> {
    let crate::What::Branch(parts) = &name.what else { return None };
    if limits.hold_segments < 3 {
        return None;
    }
    let mut branch = List::with_capacity(limits.hold_bytes);
    for (at, part) in parts.iter().enumerate() {
        if at != 0 {
            branch.push(b'/').ok()?;
        }
        for byte in part.iter().copied() {
            branch.push(byte).ok()?;
        }
    }
    if branch.len().checked_add(6)? > limits.hold_bytes {
        return None;
    }
    Some(tasks::Name {
        connector,
        path: Box::new([
            Box::from(name.forge.to_be_bytes()),
            Box::from(name.repository.to_be_bytes()),
            branch.into_boxed(),
        ]),
    })
}
