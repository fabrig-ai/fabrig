//! Hosted git settings (temper/docs/design/playground.md, 3).
use skein_lib::{Duration, List, Secret};
/// The hosted git's synthetic token.
pub const TOKEN: &[u8] = b"fake-git-identity-token";
/// The hosted git's identity.
#[must_use]
pub fn identity() -> temper_worker_protocol_git::Identity {
    temper_worker_protocol_git::Identity {
        token: Secret::new(TOKEN.into()),
        login: b"worker".as_slice().into(),
        name: b"Worker".as_slice().into(),
        email: b"worker@example.invalid".as_slice().into(),
    }
}
/// The hosted git's table.
#[must_use]
pub fn table(full: &[u8], protected: Option<&[u8]>) -> skein_fake_forge_forgejo::Config {
    let mut tokens = List::with_capacity(2);
    tokens.push((Secret::new(TOKEN.into()), 1)).expect("two tokens");
    tokens.push((Secret::new(b"fake-git-stranger-token".as_slice().into()), 2)).expect("two tokens");
    let mut users = List::with_capacity(2);
    users.push((1, b"worker".as_slice().into())).expect("two users");
    users.push((2, b"stranger".as_slice().into())).expect("two users");
    let mut repositories = List::with_capacity(1);
    repositories
        .push(skein_fake_forge_forgejo::Repository { id: 2, full: full.into(), protected: protected.map(Into::into) })
        .expect("one repository");
    skein_fake_forge_forgejo::Config {
        base: b"http://127.0.0.1:3000".as_slice().into(),
        tokens,
        users,
        repositories,
        max_response_items: 8,
        default_paging_num: 30,
        sources: Box::new([]),
        hooks: None,
        faults: skein_fake_forge_forgejo::Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
/// The hosted git's forge capacities.
pub const LIMITS: skein_fake_forge_domain::Limits = skein_fake_forge_domain::Limits {
    workflows: 0,
    workflow_runs: 0,
    repositories: 16,
    users: 4,
    labels: 1,
    items: 8,
    comments: 16,
    dependencies: 1,
    reviews: 8,
    branches: 64,
    commits: 4096,
    files: 256,
    statuses: 8,
    contexts: 4,
    bypassing_users: 0,
    pages: 1,
    name_bytes: 256,
    title_bytes: 256,
    body_bytes: 16384,
    content_bytes: 65_536,
    page_size: 1,
    calls: 1,
    hooks: 1,
    observations: 128,
};

/// The hosted git's calm forge configuration.
#[must_use]
pub fn config() -> skein_fake_forge_domain::Config { skein_fake_forge_domain::Config {
    setup_faults: 0,
    lost: 0,
    made_checks: skein_fake_forge_domain::Config::test().made_checks,
    limits: LIMITS,
    latency_min: Duration::ZERO,
    latency_max: Duration::ZERO,
    settle_min: Duration::from_secs(2),
    settle_max: Duration::from_secs(2),
    late: 0,
    late_min: Duration::ZERO,
    late_max: Duration::ZERO,
    unavailable: 0,
    timeouts: 0,
    landing: 0,
    land_min: Duration::ZERO,
    land_max: Duration::ZERO,
    rate_limit: 0,
    rate_window: Duration::ZERO,
    ci: 3,
    hook_min: Duration::ZERO,
    hook_max: Duration::ZERO,
    hooks_late: 0,
    hooks_lost: 0,
    resolution: Duration::ZERO,
    skew: skein_fake_forge_domain::Skew::None,
    status_updates: false,
    edit_updates: false,
} }
