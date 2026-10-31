//! Playground records and reset (temper/docs/design/draft/playground-reset.md, 6.1–6.3).
use crate::{Domain, Playground, PlaygroundConfig};

impl Domain {
    pub fn configure_playground(&mut self, config: Option<PlaygroundConfig>) {
        self.playground_config = config;
    }

    #[must_use]
    pub fn playground(&self) -> Option<&Playground> {
        self.playground.as_ref()
    }
}
