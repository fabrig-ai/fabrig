//! Keep: the child routing shape and exhaustive connector variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::infrastructure;
/// Application-configured child and client bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client_view::Config,
    pub infrastructure: infrastructure::Limits,
    pub observability: crate::observability::Limits,
    pub client: jig_client::Limits,
}

/// Whether the root's child limits are usable.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    jig_client_view::valid(&limits.jig, &limits.client.domain)
        && infrastructure::valid(&limits.infrastructure, &limits.client.domain)
        && crate::observability::valid(&limits.observability, &limits.client.domain)
        && jig_client::valid(&limits.client)
}
