//! Supplied amendment settings (client/domain/objects.md, 4 and 11;
//! domain/tasks.md, 7). The enclosing document supplies current or retained
//! context; this value never reconstructs instructions from today's defaults.
use super::{AuthorityDocument, DocumentText, PersonAddress, SpecificationDocument, WakePolicy};
use skein_lib::List;

/// Whole owner state, distinct from a selected request delta and its revisions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AmendmentState {
    pub addressed: Option<PersonAddress>,
    pub taken_by: Option<u64>,
    /// Current proposals carry effective text; history carries the stored override.
    /// Absence remains absence, including an unknown retained effective text.
    pub instructions: Option<DocumentText>,
    pub spec: SpecificationDocument,
    pub authority: AuthorityDocument,
    pub wake: WakePolicy,
    pub dependencies: List<u64>,
    /// Dependency-list omission only; other loss belongs to each nested document.
    pub cut: bool,
}
