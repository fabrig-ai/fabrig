//! Pure decisions over git's answers (temper/docs/design/release.md, 3).
use std::path::{Path, PathBuf};

/// Twelve lowercase hex digits, with tracked changes marked, or an unavailable identity.
pub fn word(head: Option<&[u8]>, status: Option<&[u8]>) -> String {
    let (Some(head), Some(status)) = (head, status) else {
        return "unknown".into();
    };
    let Ok(head) = std::str::from_utf8(head) else {
        return "unknown".into();
    };
    let head = head.trim();
    if head.len() != 12 || !head.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return "unknown".into();
    }
    if status.is_empty() { head.into() } else { format!("{head}-dirty") }
}

/// Git's per-worktree files, shared refs and the build script's own sources.
pub fn watched_paths(
    package: &Path,
    git_dir: Option<&str>,
    common_dir: Option<&str>,
    head_ref: Option<&str>,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let (Some(git_dir), Some(common_dir)) = (git_dir, common_dir) {
        let git_dir = package.join(git_dir.trim());
        let common_dir = package.join(common_dir.trim());
        paths.push(git_dir.join("HEAD"));
        paths.push(git_dir.join("index"));
        paths.push(common_dir.join("packed-refs"));
        if let Some(head_ref) = head_ref {
            paths.push(common_dir.join(head_ref.trim()));
        }
    }
    paths.push(PathBuf::from("build"));
    paths
}
