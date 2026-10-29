//! Schema and role validity, including representable frame and queue capacities
//! (channel.md, section 10).

use crate::{Limits, Role, Schema};

/// Whether these limits are representable for this schema and role:
/// the schema accepts them, and every frame and queue they size fits
/// (programming-model.md, 6.3).
#[must_use]
pub fn valid(schema: &Schema, role: Role, limits: &Limits) -> bool {
    schema.check(role, limits).is_ok() && capacity_bound(limits).is_some()
}

fn capacity_bound(limits: &Limits) -> Option<u32> {
    limits.output_frames.checked_add(3)?;
    let terms = limits.kinds.checked_mul(6)?.checked_add(4)?;
    let open = limits.credential.checked_add(14)?;
    open.max(terms).max(262).checked_add(8)
}
