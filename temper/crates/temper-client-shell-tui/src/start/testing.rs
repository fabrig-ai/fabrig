//! Fixtures for the TUI's directory tests (temper/docs/design/client/tui/directory.md, 1).
use temper_client_tui::Origin;

#[must_use]
pub fn origin() -> Origin {
    Origin::Http("127.0.0.1:8367".parse().expect("origin"))
}
