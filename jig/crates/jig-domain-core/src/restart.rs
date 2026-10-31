//! The core's cold-start script (domain/engine.md, 6; domain/root.md, 8).
//! The application performs each requested step and reports that exact step
//! done. Only the core chooses the next step or opens decisions.

mod census;
mod cursor;
pub use census::{Census, ReadResults};
pub use cursor::restore_cursor;

/// Ordered live ranges of the cold start (protocol/store.md, 4.3 and 7.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RestoreRange {
    /// The deployment header, first.
    Deployment,
    /// The live settings, before the children restore.
    Settings,
    /// The accounts' counter, accounts and sources, before the child starts.
    Accounts,
    /// All people's live families.
    People,
    /// The tasks child's live families.
    Tasks,
    /// The core's remaining live families.
    Core,
    /// Inbox owners named by durable indexes after read positions are restored.
    Census(Census),
}

impl RestoreRange {
    /// The fixed order of `LoadCore`'s ranges.
    pub const LIVE: [Self; 6] =
        [Self::Deployment, Self::Settings, Self::Accounts, Self::People, Self::Tasks, Self::Core];

    /// Predicate checked against the store's intervals (protocol/store.md, 4.3).
    #[must_use]
    pub fn admits(&self, key: &crate::Key) -> bool {
        match self {
            Self::Census(census) => return census.admits(key),
            Self::Deployment | Self::Settings | Self::Accounts | Self::People | Self::Tasks | Self::Core => {}
        }
        match key {
            crate::Key::Core(crate::CoreKey::Settings(key)) => {
                *self == Self::Settings
                    && match key {
                        crate::settings::Key::History(_) => false,
                        crate::settings::Key::Default(_)
                        | crate::settings::Key::Changed(_)
                        | crate::settings::Key::Held(_)
                        | crate::settings::Key::Charter { .. }
                        | crate::settings::Key::Proposal { .. }
                        | crate::settings::Key::Extent => true,
                    }
            }
            crate::Key::Core(crate::CoreKey::Deployment) => *self == Self::Deployment,
            crate::Key::Core(key) => *self == Self::Core && key.live(),
            crate::Key::People(key) => *self == Self::People && key.live(),
            crate::Key::Tasks(key) => *self == Self::Tasks && key.live(),
            crate::Key::Accounts(_) => *self == Self::Accounts,
            crate::Key::Notes(_) => false,
        }
    }
}
