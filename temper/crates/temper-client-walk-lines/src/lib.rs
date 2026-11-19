//! Walk lines and page printers (temper/docs/design/client/journey-walks.md, 3.1).

pub mod line;
pub mod print;
pub mod tui_print;
pub mod walk;

/// Whether NAME is a journey's name, `[a-z][a-z0-9-]*` (client/web/journeys.md, 5.1).
#[must_use]
pub fn named(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
