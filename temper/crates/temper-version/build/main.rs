//! Reads the build's commit (temper/docs/design/release.md, 3).
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

mod commit;

fn git(package: &Path, arguments: &[&str]) -> Option<Output> {
    Command::new("git").args(arguments).current_dir(package).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()
}

fn answer(output: Option<Output>) -> Option<Vec<u8>> {
    output.filter(|output| output.status.success()).map(|output| output.stdout)
}

#[expect(clippy::disallowed_methods, reason = "build script: Cargo's CARGO_MANIFEST_DIR locates the build commit")]
fn main() {
    let package = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    let git_dir = answer(git(&package, &["rev-parse", "--git-dir"])).and_then(|bytes| String::from_utf8(bytes).ok());
    let common_dir =
        answer(git(&package, &["rev-parse", "--git-common-dir"])).and_then(|bytes| String::from_utf8(bytes).ok());
    let symbolic = git(&package, &["symbolic-ref", "-q", "HEAD"]);
    // Exit 1 is a detached HEAD; other failures leave the identity unavailable.
    let ref_known = symbolic.as_ref().is_some_and(|output| output.status.success() || output.status.code() == Some(1));
    let head_ref = answer(symbolic).and_then(|bytes| String::from_utf8(bytes).ok());
    let head = if git_dir.is_some() && common_dir.is_some() && ref_known {
        answer(git(&package, &["rev-parse", "--short=12", "HEAD"]))
    } else {
        None
    };
    let status = answer(git(&package, &["status", "--porcelain", "--untracked-files=no"]));
    println!("cargo:rustc-env=TEMPER_INTERNAL_COMMIT={}", commit::word(head.as_deref(), status.as_deref()));
    for path in commit::watched_paths(&package, git_dir.as_deref(), common_dir.as_deref(), head_ref.as_deref()) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
