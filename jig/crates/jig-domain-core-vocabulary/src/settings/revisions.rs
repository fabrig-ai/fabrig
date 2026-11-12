//! Revision history keys (domain/engine/settings.md, 7).

/// A held revision's key and the history row it enters as.
#[must_use]
pub fn history_copy(number: u32) -> (crate::Key, crate::Key) {
    use crate::{CoreKey, Key, settings};
    (
        Key::Core(CoreKey::Settings(settings::Key::Held(number))),
        Key::Core(CoreKey::Settings(settings::Key::History(number))),
    )
}
