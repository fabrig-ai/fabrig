//! Proposal facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 4.
use super::{Body, Object};

/// Pending owner facts share the original typed proposal identity.
#[must_use]
#[cfg(test)]
pub fn owner_facts(object: &Object) -> Option<&jig_client::contract::OwnerProposal> {
    match &object.body {
        Some(Body::OwnerProposal(document)) => Some(document),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
