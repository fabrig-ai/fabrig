//! Question facts carry bounded words and their cut marker; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 6.
use super::{Body, Object};
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&jig_client::contract::Question> {
    match &object.body {
        Some(Body::Question(document)) => Some(document),
        Some(
            Body::Ended(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::Escalation(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
