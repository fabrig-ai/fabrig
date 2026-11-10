//! Shared client fixtures (skein/docs/design/testing/README.md, 6).
use skein_forge::api as client;
use skein_forge_client::Limits;
use skein_lib::Duration;

pub const LIMITS: Limits = Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 20,
    window: Duration::from_secs(10),
    op_bytes: 256,
    answer_bytes: 4096,
    rows: 2,
    inbox: 32,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 4,
    write_attempts: 5,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 3,
    repositories: 1,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    facts: 8,
};

#[must_use]
pub fn commit(number: u64) -> client::Commit {
    let mut id = [0; 32];
    id[..20].copy_from_slice(&skein_fake_forge_domain::object_bytes(number));
    id
}
