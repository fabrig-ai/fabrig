//! Public reading's identifiers and declared kinds (jig/docs/design/protocol/client/public-reads.md).
mod names;
use jig_client_protocol_wire::{PUBLIC_BOARD, PUBLIC_CARD, PUBLIC_PROJECTS, PUBLIC_VERSION, PublicIndexBounds};
pub use names::{Names, Of};

/// A declared public kind and its largest answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicKind {
    pub owner: u16,
    pub kind: u16,
    pub first: u16,
    pub last: u16,
    pub task: bool,
    pub bound: u32,
}
/// Jig's three kinds, bounded by the deployment's public index.
#[must_use]
pub fn jig_kinds(bounds: &PublicIndexBounds) -> [PublicKind; 3] {
    let [projects, board, card] = jig_client_protocol_wire::public_bounds(bounds);
    [
        PublicKind {
            owner: 0,
            kind: PUBLIC_PROJECTS,
            first: PUBLIC_VERSION,
            last: PUBLIC_VERSION,
            task: false,
            bound: projects,
        },
        PublicKind {
            owner: 0,
            kind: PUBLIC_BOARD,
            first: PUBLIC_VERSION,
            last: PUBLIC_VERSION,
            task: false,
            bound: board,
        },
        PublicKind {
            owner: 0,
            kind: PUBLIC_CARD,
            first: PUBLIC_VERSION,
            last: PUBLIC_VERSION,
            task: true,
            bound: card,
        },
    ]
}
