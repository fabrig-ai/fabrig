//! Checked composition of child, queue, and table capacities (root.md, 3).
use crate::first;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client_view::Config,
    pub first: first::Limits,
    pub second_parts: u32,
    pub client: jig_client::Limits,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    jig_client_view::valid(&limits.jig, &limits.client.domain)
        && first::valid(&limits.first)
        && jig_client::valid(&limits.client)
}
