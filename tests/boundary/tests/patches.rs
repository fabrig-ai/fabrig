//! Stored changes bring their patches (temper/docs/design/protocol/baseline.md, 7).
//! Landed files come from git, as in skein/docs/design/data/codec/command.md, 8;
//! the owner inventory also covers every crate with `schema/kept/`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use skein_codegen::patch::{self, Finding, Landed};
use skein_scratch::{Scratch, paths};

const OWNERS: &[&str] = &[
    "skein/crates/skein-oauth",
    "smith/crates/smith-transcript",
    "jig/crates/jig-protocol-store",
    "jig/crates/jig-protocol-accounts",
    "temper/crates/temper-protocol-store",
    "temper/crates/temper-protocol-records",
];
const RECORDS: &str = "temper/crates/temper-protocol-records";

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|why| format!("git {}: {why}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!("git {}: {}: {}", args.join(" "), output.status, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

fn base(root: &Path) -> String {
    // During a landing merge, the incoming target already owns its kept patches.
    let args = if git(root, &["rev-parse", "--verify", "MERGE_HEAD^{commit}"]).is_ok() {
        vec!["rev-parse", "MERGE_HEAD"]
    } else if git(root, &["rev-parse", "--verify", "origin/main^{commit}"]).is_ok() {
        vec!["merge-base", "HEAD", "origin/main"]
    } else {
        vec!["rev-parse", "HEAD"]
    };
    let reference = String::from_utf8(git(root, &args).expect("read the landed base")).expect("git's base is UTF-8");
    let reference = reference.trim().to_owned();
    eprintln!("patch check base: {reference} (git {})", args.join(" "));
    reference
}

fn landed(owner: &Path, base: &str) -> Result<Landed, String> {
    let paths = git(owner, &["ls-tree", "-r", "--name-only", "-z", base, "--", "patches", "schema/kept"])?;
    let mut files = BTreeMap::new();
    for bytes in paths.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let name = std::str::from_utf8(bytes).map_err(|why| format!("git path is not UTF-8: {why}"))?;
        let path = PathBuf::from(name);
        if path.components().any(|part| !matches!(part, Component::Normal(_)))
            || !(path.starts_with("patches") || path.starts_with("schema/kept"))
        {
            return Err(format!("git returned a path outside the crate: {name}"));
        }
        let object = format!("{base}:./{name}");
        files.insert(path, git(owner, &["show", &object])?);
    }
    Ok(Landed::new(files))
}

#[test]
#[ignore = "65 s on w1, a third of the focused suite: skipped while the focused suite is made faster; run it by hand (the user, 2026-11-12)"]
fn stored_changes_have_patches() {
    let root = paths::workspace();
    let base = base(&root);
    let mut failures = Vec::new();
    for owner in OWNERS {
        let path = root.join(owner);
        let result =
            landed(&path, &base).and_then(|landed| patch::check(&path, &landed).map_err(|why| why.to_string()));
        match result {
            Ok(findings) => {
                for finding in findings {
                    failures.push(format!("{owner} (base {base}): {finding}"));
                }
            }
            Err(why) => failures.push(format!("{owner} (base {base}): {why}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn patch_modules_are_current() {
    let root = paths::workspace();
    let base = base(&root);
    let mut failures = Vec::new();
    for owner in OWNERS {
        if let Err(why) = patch::current(&root.join(owner)) {
            failures.push(format!("{owner} (base {base}): {why}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn owner_list_covers_every_kept_crate() {
    let root = paths::workspace();
    let mut found = BTreeSet::new();
    for tree in fs::read_dir(&root).expect("read workspace directories") {
        let crates = tree.expect("read workspace entry").path().join("crates");
        if !crates.is_dir() {
            continue;
        }
        for owner in fs::read_dir(crates).expect("read a tree's crates") {
            let owner = owner.expect("read crate entry").path();
            if owner.join("schema/kept").is_dir() {
                found.insert(owner.strip_prefix(&root).expect("crate under workspace").to_path_buf());
            }
        }
    }
    let listed: BTreeSet<_> = OWNERS.iter().map(PathBuf::from).collect();
    assert_eq!(listed, found, "every crate with schema/kept/ must be checked");
}

fn copy_directory(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create scratch crate directory");
    for entry in fs::read_dir(source).expect("read owner directory") {
        let entry = entry.expect("read owner entry");
        let destination = target.join(entry.file_name());
        if entry.file_type().expect("read owner file type").is_dir() {
            copy_directory(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).expect("copy owner file");
        }
    }
}

fn records_copy() -> (Scratch, Landed) {
    let scratch = Scratch::new("boundary-patches");
    copy_directory(&paths::workspace().join(RECORDS), scratch.path());
    let landed = Landed::read(scratch.path()).expect("scratch copy's files before the edit");
    (scratch, landed)
}

#[test]
fn added_field_without_patch_is_unpatched() {
    let (scratch, landed) = records_copy();
    let schema = scratch.path().join("schema/secrets-v1.schema");
    let source = fs::read_to_string(&schema).expect("read current secrets schema");
    let changed = source.replace("record User { value: u64 }", "record User { value: u64 added: u64 }");
    assert_ne!(source, changed, "the mutation must add a field");
    fs::write(schema, changed).expect("add a field without a patch");
    let findings = patch::check(scratch.path(), &landed).expect("check the mutated crate");
    assert!(
        findings
            .iter()
            .any(|finding| matches!(finding, Finding::Unpatched { family, .. } if family == "temper-secrets")),
        "expected Unpatched for temper-secrets, got {findings:?}"
    );
}

#[test]
fn edited_landed_kept_text_is_changed() {
    let (scratch, landed) = records_copy();
    let path = Path::new("schema/kept/temper-secrets-v1.schema");
    let schema = scratch.path().join(path);
    let mut source = fs::read_to_string(&schema).expect("read kept secrets schema");
    source.push_str("\n# A landed kept text was edited.\n");
    fs::write(schema, source).expect("edit the landed kept text");
    let findings = patch::check(scratch.path(), &landed).expect("check the mutated crate");
    assert!(
        findings.contains(&Finding::Changed { path: path.to_path_buf() }),
        "expected Changed for {}, got {findings:?}",
        path.display()
    );
}
