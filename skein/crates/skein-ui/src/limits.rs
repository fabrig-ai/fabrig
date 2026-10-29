//! Configured UI storage (skein's ui.md, section 15).
//! The caller derives these capacities from the domain and its own fragments.

/// Capacities checked before each builder allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub nodes: u32,
    pub depth: u32,
    /// Name, description, placeholder, runs, field value and external target together on a node.
    pub text_per_node: u32,
    /// Name, description, placeholder, runs, field value and external target together across the tree.
    pub text_bytes: u32,
    pub drawing_bytes: u32,
    pub styles_per_node: u32,
    pub patches: u32,
    pub bindings: u32,
    pub slots: u32,
    pub markdown_depth: u32,
    pub scene_marks: u32,
    pub scene_bytes: u32,
}

/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// Checks the scene's style capacity and one line's Markdown workspace bound.
/// The remaining capacities are already represented by `u32` limits.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.scene_marks.checked_mul(limits.styles_per_node).is_some()
        && crate::markdown::markdown_bound(limits.text_bytes, limits.markdown_depth).is_some()
}
