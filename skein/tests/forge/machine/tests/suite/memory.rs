use skein_forge_connection::limits::Forges;
use skein_forge_machine_world::memory::{self, Saturation};

// Holders shared by the full scenarios; each row names its differences.
const FULL: Saturation = Saturation {
    forges: 2,
    calls: 2,
    connections: 4,
    repositories: 2,
    users: 8,
    hooks: 4,
    installations: 6,
    minted_tokens: 12,
    held_answers: 2,
    file_rows: 8,
    compare_commits: 1,
    reviewers: 8,
    queued: 1,
    raw_bytes: 65408,
    held_hints: 0,
};

#[test]
fn full_forgejo_holders_stay_under_the_ceiling() {
    // Component peak while filling every Forgejo fixture holder.
    const CEILING: u64 = 320_000;
    check(
        Forges { forgejo: 2, github: 0 },
        Saturation {
            connections: 2,
            installations: 0,
            minted_tokens: 0,
            compare_commits: 8,
            reviewers: 0,
            queued: 0,
            held_hints: 4,
            ..FULL
        },
        CEILING,
    );
}
#[test]
fn full_github_holders_stay_under_the_ceiling() {
    // Component peak while filling every GitHub fixture holder.
    const CEILING: u64 = 400_000;
    check(Forges { forgejo: 0, github: 2 }, Saturation { ..FULL }, CEILING);
}
#[test]
fn full_mixed_holders_stay_under_the_ceiling() {
    // Component peak while filling every mixed fixture holder.
    const CEILING: u64 = 370_000;
    check(
        Forges { forgejo: 1, github: 1 },
        Saturation {
            connections: 3,
            repositories: 4,
            users: 16,
            installations: 3,
            minted_tokens: 6,
            held_hints: 4,
            ..FULL
        },
        CEILING,
    );
}
fn check(forges: Forges, saturation: Saturation, ceiling: u64) {
    let full = memory::full_heap(forges, ceiling);
    assert_eq!(full.saturation(), saturation);
    assert_eq!((full.op_bytes, full.body_bytes, full.rows, full.inner), (512, 1042, 8, 8));
    assert_eq!(full.answer_bytes, 65_536);
    if forges.github > 0 {
        assert_eq!(full.key_bytes, 1192);
        assert!(full.jwt_bytes > 0);
    }
    assert!(full.owned > 0 && full.owned <= ceiling);
    println!(
        "{forges:?}: owned={}, ceiling={ceiling}, margin={}, saturation={:?}, key_bytes={}, jwt_bytes={}",
        full.owned,
        ceiling - full.owned,
        full.saturation(),
        full.key_bytes,
        full.jwt_bytes
    );
}
