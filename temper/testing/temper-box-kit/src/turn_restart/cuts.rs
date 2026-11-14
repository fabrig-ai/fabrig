//! Real process cuts around the worker's prepare and the landing's turn
//! (temper/docs/design/domain/landing.md, 5.2 and 15).
use super::{JobBarrier, Path, Proof, Throwaway, fs, json, write_private};
use std::os::unix::fs::PermissionsExt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Cut {
    Preparing,
    Prepared,
    Turn,
    Ordinary,
}

/// Pause only the real git push of a landing prefix. A stop kills this worker's
/// command; the next serve recovers its job on the same real repository.
pub fn configure(root: &Path, repository: u64) {
    super::configure(root, repository);
    let directory = root.to_str().expect("scratch path").replace('\'', "'\\''");
    let script = format!(r#"#!/bin/sh
case " $* " in
  *" push "*refs/heads/temper/*/l*)
    barrier='{directory}'
    : > "$barrier/prepare-started"
    while [ ! -f "$barrier/prepare-release" ]; do
      kill -0 {} 2>/dev/null || exit 1
      sleep 0.1
    done
    ;;
esac
exec /usr/bin/git "$@"
"#, std::process::id());
    let path = root.join("prepare-git");
    write_private(&path, script.as_bytes());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable git barrier");
    let path = root.join("temper.json");
    let mut value = serde_json::from_slice::<serde_json::Value>(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["workers"][0]["git"] = json!(root.join("prepare-git"));
    write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}

#[must_use]
pub fn preparing(root: &Path, forge: &Throwaway, barrier: &JobBarrier) -> Proof {
    super::run_to(root, forge, barrier, None, false, Cut::Preparing).0.expect("preparing cut")
}

#[must_use]
pub fn prepared(root: &Path, forge: &Throwaway, barrier: &JobBarrier, previous: &Proof) -> Proof {
    write_private(&root.join("prepare-release"), b"");
    super::run_to(root, forge, barrier, Some(previous), false, Cut::Prepared).0.expect("recovered prepare proof")
}

#[must_use]
pub fn turn(root: &Path, forge: &Throwaway, barrier: &JobBarrier, previous: &Proof) -> Proof {
    super::run_to(root, forge, barrier, Some(previous), false, Cut::Turn).0.expect("turn cut")
}
