use skein_lib::Duration;

/// The longest program a repository names, its absolute path
/// (protocol/channel.md, section 4).
pub const PROGRAM_BYTES: u32 = 4096;

/// The checkout child domain's limits (section 7), handed by its parent to
/// every step read-only.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Workspaces on the worker's disk: the cache's bound, and the most that
    /// may be held at once. A prepare that finds every one held is refused.
    pub workspaces: u32,
    /// Repositories a spec names: at least one.
    pub repositories: u32,
    /// The longest workstream key, repository name or remote, or branch.
    pub name_bytes: u32,
    /// The longest commit message, its title and body together.
    pub message_bytes: u32,
    /// Maximum originally or still conflicted paths per repository. Zero
    /// disables merge starts; other workspace preparation remains available.
    pub conflicts: u32,
    /// Maximum bytes of one full relative conflicted path.
    pub path_bytes: u32,
    /// How long an operation that reaches the forge may take: a clone, a
    /// fetch, a branch's creation, a push.
    pub remote_timeout: Duration,
    /// How long one on the worker's disk may take: making a workspace,
    /// checking out, merging, committing.
    pub local_timeout: Duration,
    /// Facts kept until the parent drains them. Beyond them, facts are dropped
    /// and counted.
    pub facts: u32,
}

/// Whether workspaces and repositories are nonzero and the hold-slot count
/// needed by construction is representable.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.workspaces != 0 && limits.repositories != 0 && holds(limits).is_some()
}

/// The hold slab's capacity: two for each workspace. A hold holds one
/// workspace, so at most one for each is live; a hold released in an
/// iteration is reclaimed only at its end, and one made in an iteration is
/// not released in it (its prepare waits for io), so the live and the
/// released ones of an iteration are no more than twice the workspaces.
pub(crate) fn holds(limits: &Limits) -> Option<u32> {
    limits.workspaces.checked_mul(2)
}
