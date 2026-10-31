//! The reference UI's windows, folds and page bounds (jig/docs/design/client/domain/view.md, 18.9).
use skein_lib::Duration;

/// Presentation capacities and durations added by the reference UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    pub notices: u32,
    pub notice_life: Duration,
    pub linger: Duration,
    pub save_delay: Duration,
    pub intents: u32,

    pub objects: u32,
    pub sources: u32,
    /// Per-task composers kept across navigation and restart.
    pub drafts: u32,
    pub roles: u32,
    /// Whole pending or sent word snapshots kept for the displayed conversation.
    pub conversation_words: u32,
    /// Maximum configured navigation destinations.
    pub navigation_entries: u32,
    /// The largest unread update count displayed without a lower-bound mark.
    pub updates_count: u32,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Config, client: &jig_client::DomainLimits) -> bool {
    if client.watches == 0
        || limits.notice_life == Duration::ZERO
        || limits.linger == Duration::ZERO
        || limits.save_delay == Duration::ZERO
        || limits.sources == 0
        || limits.conversation_words < limits.intents
    {
        return false;
    }
    timers(limits, client).is_some()
        && tree_names(client).is_some()
        && crate::state::page::conversations::valid(client)
        && crate::state::page::inbox::valid(client)
        && crate::state::page::task::valid(client)
        && crate::state::page::project::valid(client)
        && crate::tree::notices::kept::valid(limits, client)
        && crate::state::objects::valid(limits)
        && crate::state::windows::valid(client)
}

/// One deadline per lingering object, visible notice and saved record,
/// plus the minute tick. Checked before any container is allocated.
#[must_use]
pub fn timers(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    limits.objects.checked_add(limits.notices)?.checked_add(client.records)?.checked_add(1)
}

/// One task and every bounded proposal name per retained tree row.
pub(crate) fn tree_names(client: &jig_client::DomainLimits) -> Option<u32> {
    client.rows.checked_mul(client.rows.checked_add(1)?)?.checked_add(1)
}
