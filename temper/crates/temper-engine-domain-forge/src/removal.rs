//! Forge removal state (temper/docs/design/draft/playground-reset.md, 4).
//! The empty state reserves the body for its owning task.
#[derive(Debug)]
pub(crate) struct State;

impl State {
    pub(crate) const fn new(_l: &crate::Limits) -> Self {
        Self
    }
}
