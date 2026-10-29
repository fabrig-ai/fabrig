//! What the forge admits at most (programming-model.md, sections 6.3 and 7).

/// The forge's limits, part of its configuration. What would pass one is
/// refused: a name, title, body or content too long as too large, and one
/// more of anything a repository holds as full.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Repositories on the forge.
    pub repositories: u32,
    /// Users with a permission on a repository, and users calling within a
    /// rate window.
    pub users: u32,
    /// Bypassing users in the repository's one protection rule.
    pub bypassing_users: u32,
    /// Labels defined in a repository.
    pub labels: u32,
    /// Items in a repository; comments on an item, and items it depends on.
    pub items: u32,
    pub comments: u32,
    pub dependencies: u32,
    /// Reviews of a pull request.
    pub reviews: u32,
    /// Branches of a repository.
    pub branches: u32,
    /// Commits in the forge's store, and files in a commit's tree.
    pub commits: u32,
    pub files: u32,
    /// Commits of a repository with statuses, and contexts on each.
    pub statuses: u32,
    pub contexts: u32,
    /// Dispatch-only workflow definitions and retained dispatched runs per repository.
    pub workflows: u32,
    pub workflow_runs: u32,
    /// Wiki pages of a repository.
    pub pages: u32,
    /// The most bytes of a name: a repository's, a branch's, a label's, a
    /// context's, a wiki page's, or a path.
    pub name_bytes: u32,
    pub title_bytes: u32,
    /// The most bytes of an item's, a comment's or a review's body.
    pub body_bytes: u32,
    /// The most bytes of a file's content, or a wiki page's.
    pub content_bytes: u32,
    /// The most entries a listing answers at once: items, comments, or wiki
    /// page names.
    pub page_size: u32,
    /// Calls held at once, until they are answered and reclaimed. A call
    /// beyond them is answered at once as unavailable.
    pub calls: u32,
    /// Webhook deliveries in flight. A change beyond them is not delivered.
    pub hooks: u32,
    /// Observations kept until a world drains them. Beyond them, they are
    /// dropped and counted.
    pub observations: u32,
}

/// The timers armed at most: one per call, one per delivery, and one per
/// context of each commit with statuses, one per dispatched job, and one queue
/// wake per repository.
pub(crate) fn timers(limits: &Limits) -> Option<u32> {
    let checks = limits.repositories.checked_mul(limits.statuses)?.checked_mul(limits.contexts)?;
    let workflows = limits.repositories.checked_mul(limits.workflow_runs)?.checked_mul(limits.contexts)?;
    limits
        .calls
        .checked_add(limits.hooks)?
        .checked_add(checks)?
        .checked_add(workflows)?
        .checked_add(limits.repositories)
}

/// The comments a repository holds at most.
pub(crate) fn comments(limits: &Limits) -> Option<u32> {
    limits.items.checked_mul(limits.comments)
}

/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// Timer, comment, directory and run capacities fit their u32 counters.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    timers(limits).is_some()
        && comments(limits).is_some()
        && directory_capacity(limits).is_some()
        && limits.files.checked_add(1).is_some()
        && limits.statuses.checked_add(limits.workflow_runs).is_some()
}

fn directory_capacity(limits: &Limits) -> Option<u32> {
    limits.files.checked_mul(limits.name_bytes)?.checked_add(1)
}
