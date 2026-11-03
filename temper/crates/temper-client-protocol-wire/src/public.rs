//! Visitor records bounded by the panels (temper/docs/design/client/forge.md, 11.4).
use crate::{ADDRESS_BYTES, Bounds, TASKS_AHEAD, public_v1};

/// Set the visitor codec's limits from the panels, refusing invalid panel bounds.
#[must_use]
pub fn public_limits(bounds: &Bounds) -> Option<public_v1::Limits> {
    let panels = crate::limits(bounds)?;
    let limits = public_v1::Limits {
        public_change_repository: ADDRESS_BYTES,
        public_change_pull_some: panels.pull_address,
        public_change_gates: panels.change_panel_gates,
        public_landing_line_members: panels.landing_line_changes,
        public_landing_line_landed: panels.landing_line_landed,
        public_queue_repository: ADDRESS_BYTES,
        public_queue_changes: TASKS_AHEAD,
        public_landings_landings: panels.landings_panel_landings,
        public_landings_queues: panels.landings_panel_landings,
        public_gate_proof_run_some: panels.receipted_address,
        public_member_repository: ADDRESS_BYTES,
        public_member_pull_some: panels.pull_address,
        public_member_gates: panels.member_gates,
        public_landing_members: panels.landing_panel_members,
    };
    match public_v1::check_limits(&limits) {
        Ok(()) => Some(limits),
        Err(_) => None,
    }
}

/// Maximum encoded change, landings and landing answers; zero for invalid bounds.
#[must_use]
pub fn public_bounds(bounds: &Bounds) -> [u32; 3] {
    measured_bounds(bounds).unwrap_or([0; 3])
}

fn measured_bounds(bounds: &Bounds) -> Option<[u32; 3]> {
    let limits = public_limits(bounds)?;
    Some([
        u32::try_from(public_v1::PublicChange::encoded_bound(&limits)?).ok()?,
        u32::try_from(public_v1::PublicLandings::encoded_bound(&limits)?).ok()?,
        u32::try_from(public_v1::PublicLanding::encoded_bound(&limits)?).ok()?,
    ])
}

#[cfg(test)]
#[path = "tests/public.rs"]
mod tests;
