//! Fixtures for the TUI's directory tests (temper/docs/design/client/tui/README.md, 2).
use temper_client_tui::Origin;

#[must_use]
pub fn origin() -> Origin {
    Origin::Http("127.0.0.1:8367".parse().expect("origin"))
}

pub mod deployment {
    use super::Origin;
    use std::path::{Path, PathBuf};

    pub fn prepare(directory: &Path, origin: &Origin, user: u32) -> Result<(PathBuf, bool), crate::start::Refused> {
        crate::start::deployment::prepare(directory, origin, user)
    }
}
