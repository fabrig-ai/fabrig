//! The deployment agent's build identity (temper/docs/design/protocol/README.md, 4.3).
#![forbid(unsafe_code)]

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod commit;

fn git(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn rerun(path: &Path) {
    if path.is_file() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn identity(root: &Path) -> Option<(String, bool)> {
    let head = git(root, &["rev-parse", "HEAD"])?;
    let status = git(root, &["--no-optional-locks", "status", "--porcelain", "--untracked-files=no"])?;
    let directories = git(root, &["rev-parse", "--path-format=absolute", "--git-dir", "--git-common-dir"])?;
    let mut directories = directories.lines();
    let directory = PathBuf::from(directories.next()?);
    let common = PathBuf::from(directories.next()?);
    rerun(&directory.join("HEAD"));
    rerun(&directory.join("index"));
    if let Some(reference) = git(root, &["symbolic-ref", "-q", "HEAD"]) {
        rerun(&common.join(reference.trim()));
    }
    rerun(&common.join("packed-refs"));
    Some((head.trim().into(), !status.is_empty()))
}

fn main() {
    println!("cargo:rerun-if-changed=build/");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    let word = match identity(&root) {
        Some((head, dirty)) => commit::word(Some(&head), dirty),
        None => commit::word(None, false),
    };
    println!("cargo:rustc-env=TEMPER_AGENT_COMMIT={word}");
}
