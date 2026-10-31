//! Run adoption and decision opening (jig/docs/design/domain/engine.md, 6, 8).
use crate::Core;

impl Core {
    /// True once a start (either interface) has opened decisions.
    #[must_use]
    pub fn start_opened(&self) -> bool {
        self.opened
    }
}
