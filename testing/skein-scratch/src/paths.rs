//! Where a test finds its package, its workspace, its package's binaries
//! and the target's scratch space: when it runs (testing-strategy.md, 8).
//!
//! `env!("CARGO_MANIFEST_DIR")`, `env!("CARGO_BIN_EXE_<name>")` and
//! `env!("CARGO_TARGET_TMPDIR")` compile the worktree's path into a test,
//! and the build cache, which keys every input, then shares no test
//! executable between worktrees or machines. Cargo and nextest set the
//! first two in a test's environment when it runs; the third is the
//! target's `tmp`, beside the test's executable. They are the only
//! variables a test reads from its environment: they say where things are,
//! not how the run goes.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::fail;

/// The running test's package directory, which holds its `Cargo.toml`.
#[must_use]
pub fn package() -> PathBuf {
    PathBuf::from(variable("CARGO_MANIFEST_DIR"))
}

/// The workspace's root: the running test's package directory, or the
/// nearest above it, that holds `Cargo.lock`. A library that other
/// packages' tests call locates its files from here, since [`package`] is
/// theirs.
#[must_use]
pub fn workspace() -> PathBuf {
    let package = package();
    for directory in package.ancestors() {
        if directory.join("Cargo.lock").is_file() {
            return directory.to_path_buf();
        }
    }
    fail(&format!("no Cargo.lock in {} or above it", package.display()))
}

/// The running test's package's binary `name`.
#[must_use]
pub fn binary(name: &str) -> PathBuf {
    PathBuf::from(variable(&format!("CARGO_BIN_EXE_{name}")))
}

/// The target's scratch space, made if absent: `tmp` in the target
/// directory, which holds the profile's directory and its `deps`, where the
/// test's executable is. Cargo names it `CARGO_TARGET_TMPDIR` when it compiles
/// an integration test.
#[must_use]
pub fn target_tmp() -> PathBuf {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => fail(&format!("the test's executable: {error}")),
    };
    let Some(target) = executable.ancestors().nth(3) else {
        fail(&format!("no target directory above {}", executable.display()))
    };
    let tmp = target.join("tmp");
    if let Err(error) = fs::create_dir_all(&tmp) {
        fail(&format!("the target's scratch space {}: {error}", tmp.display()));
    }
    tmp
}

fn variable(name: &str) -> OsString {
    match std::env::var_os(name) {
        Some(value) => value,
        None => fail(&format!("{name} is unset: cargo and nextest set it when a test runs")),
    }
}
