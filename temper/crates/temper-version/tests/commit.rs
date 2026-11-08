//! Build identity decisions on git answers (temper/docs/design/release.md, 3).
#[path = "../build/commit.rs"]
mod commit;

use std::path::{Path, PathBuf};

#[test]
fn commit_word_requires_a_lowercase_head_and_a_successful_status() {
    for (head, status, expected) in [
        (Some("012345abcdef"), Some(""), "012345abcdef"),
        (Some("012345abcdef"), Some(" M tracked.rs\n"), "012345abcdef-dirty"),
        (None, None, "unknown"),
        (None, Some(""), "unknown"),
        (Some("012345abcdef"), None, "unknown"),
        (Some(""), Some(""), "unknown"),
        (Some("012345abcde"), Some(""), "unknown"),
        (Some("012345abcdef0"), Some(""), "unknown"),
        (Some("012345abcdeF"), Some(""), "unknown"),
        (Some("012345abcdeg"), Some(""), "unknown"),
        (Some("012345abcdef\n"), Some(""), "012345abcdef"),
    ] {
        assert_eq!(commit::word(head.map(str::as_bytes), status.map(str::as_bytes)), expected, "{head:?}, {status:?}");
    }
    assert_eq!(commit::word(Some(b"012345abcde\xff"), Some(b"")), "unknown");
    assert_eq!(commit::word(Some(b"012345abcdef"), Some(b"\xff")), "012345abcdef-dirty");
}

#[test]
fn watched_paths_follow_git_dirs_and_the_named_ref() {
    for (git_dir, common_dir, head_ref, expected) in [
        (
            Some("git"),
            Some("git"),
            Some("refs/heads/topic\n"),
            vec![
                "/package/git/HEAD",
                "/package/git/index",
                "/package/git/packed-refs",
                "/package/git/refs/heads/topic",
                "build",
            ],
        ),
        (
            Some("/repo/git\n"),
            Some("/repo/git\n"),
            Some("refs/heads/topic"),
            vec!["/repo/git/HEAD", "/repo/git/index", "/repo/git/packed-refs", "/repo/git/refs/heads/topic", "build"],
        ),
        (
            Some("/repo/git/worktrees/topic"),
            Some("/repo/git"),
            Some("refs/heads/topic"),
            vec![
                "/repo/git/worktrees/topic/HEAD",
                "/repo/git/worktrees/topic/index",
                "/repo/git/packed-refs",
                "/repo/git/refs/heads/topic",
                "build",
            ],
        ),
        (
            Some("git"),
            Some("/repo/git"),
            None,
            vec!["/package/git/HEAD", "/package/git/index", "/repo/git/packed-refs", "build"],
        ),
        (None, None, None, vec!["build"]),
    ] {
        assert_eq!(
            commit::watched_paths(Path::new("/package"), git_dir, common_dir, head_ref),
            expected.into_iter().map(PathBuf::from).collect::<Vec<_>>(),
            "{git_dir:?}, {common_dir:?}, {head_ref:?}"
        );
    }
}

#[test]
fn library_prints_the_binary_name_version_and_commit() {
    use temper_version::{COMMIT, VERSION, line};

    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(line("temper-engine"), format!("temper-engine {VERSION} ({COMMIT})"));
    let head = COMMIT.strip_suffix("-dirty").unwrap_or(COMMIT);
    assert!(
        COMMIT == "unknown"
            || (head.len() == 12 && head.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))),
        "{COMMIT}"
    );
}
