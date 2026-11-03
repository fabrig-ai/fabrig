//! Public answer reservations (jig/docs/design/client/domain/contract.md, section 12).

use crate::public_v1;

/// The public index's collection bounds, within the public schema's ceilings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PublicIndexBounds {
    pub goals: u16,
    pub ended: u16,
    pub lines: u16,
    pub depth: u16,
    pub projects: u32,
}

/// Fills every index-backed list limit, refusing bounds past the schema's ceilings.
#[must_use]
pub fn public_limits(bounds: &PublicIndexBounds) -> Option<public_v1::Limits> {
    let limits = public_v1::Limits {
        public_projects_projects: bounds.projects,
        public_board_live: u32::from(bounds.goals),
        public_board_ended: u32::from(bounds.ended),
        public_goal_tree: u32::from(bounds.lines),
        public_card_lineage: u32::from(bounds.depth),
        public_card_delegates: u32::from(bounds.lines),
        ..public_v1::CEILINGS
    };
    match public_v1::check_limits(&limits) {
        Ok(()) => Some(limits),
        Err(_) => None,
    }
}

/// Largest projects, board and card encodings; invalid bounds refuse a reservation.
#[must_use]
pub fn public_bounds(bounds: &PublicIndexBounds) -> [u32; 3] {
    let Some(limits) = public_limits(bounds) else { return [u32::MAX; 3] };
    [
        answer_bound(public_v1::PublicProjects::encoded_bound(&limits)),
        answer_bound(public_v1::PublicBoard::encoded_bound(&limits)),
        answer_bound(public_v1::PublicCard::encoded_bound(&limits)),
    ]
}

fn answer_bound(bound: Option<u64>) -> u32 {
    let Some(bound) = bound else { return u32::MAX };
    u32::try_from(bound).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "tests/public.rs"]
mod tests;
