//! Kept host-tool contracts never change and their versions have no gaps
//! (temper/docs/design/protocol/wires.md, 6).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use skein_scratch::paths;

const CONTRACT: &str = "temper/crates/temper-engine-smith/contract";
type Files = BTreeMap<String, Vec<u8>>;

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
    let args = if git(root, &["rev-parse", "--verify", "origin/main^{commit}"]).is_ok() {
        vec!["merge-base", "HEAD", "origin/main"]
    } else {
        vec!["rev-parse", "HEAD"]
    };
    let reference = String::from_utf8(git(root, &args).expect("read the landed base")).expect("git's base is UTF-8");
    let reference = reference.trim().to_owned();
    eprintln!("host-tool contract base: {reference} (git {})", args.join(" "));
    reference
}

#[expect(clippy::case_sensitive_file_extension_comparisons, reason = "kept contract names use the exact .txt suffix")]
fn contract_name(name: &str) -> bool {
    name.starts_with("host-tools-v") && name.ends_with(".txt")
}

fn landed(root: &Path, base: &str) -> Files {
    let listed =
        git(root, &["ls-tree", "-r", "--name-only", "-z", base, "--", CONTRACT]).expect("list base's kept contracts");
    let mut files = Files::new();
    for bytes in listed.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(bytes).expect("git path is UTF-8");
        let name = Path::new(path).file_name().expect("contract filename").to_str().expect("UTF-8 filename");
        if contract_name(name) {
            files
                .insert(name.to_owned(), git(root, &["show", &format!("{base}:{path}")]).expect("read kept base text"));
        }
    }
    files
}

fn current(root: &Path) -> Files {
    let mut files = Files::new();
    for entry in fs::read_dir(root.join(CONTRACT)).expect("read kept contract directory") {
        let entry = entry.expect("read contract entry");
        let name = entry.file_name().into_string().expect("UTF-8 filename");
        if contract_name(&name) {
            files.insert(name, fs::read(entry.path()).expect("read kept text"));
        }
    }
    files
}

fn check(base: &Files, tree: &Files) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, bytes) in base {
        if tree.get(name) != Some(bytes) {
            failures.push(format!("landed contract changed or removed: {name}"));
        }
    }
    let mut versions = BTreeSet::new();
    for name in tree.keys() {
        let version = name
            .strip_prefix("host-tools-v")
            .and_then(|tail| tail.strip_suffix(".txt"))
            .and_then(|number| number.parse::<u16>().ok());
        match version {
            Some(number) if number != 0 && *name == format!("host-tools-v{number}.txt") => {
                versions.insert(number);
            }
            Some(_) | None => failures.push(format!("invalid contract version: {name}")),
        }
    }
    let greatest = versions.last().copied().unwrap_or(1);
    for version in 1..=greatest {
        if !versions.contains(&version) {
            failures.push(format!("missing kept contract: host-tools-v{version}.txt"));
        }
    }
    failures
}

#[test]
fn host_tool_contract_history_is_kept() {
    let root = paths::workspace();
    let base = base(&root);
    let failures = check(&landed(&root, &base), &current(&root));
    assert!(failures.is_empty(), "base {base}:\n{}", failures.join("\n"));
}

#[test]
fn contract_mutations_preserve_history_and_contiguous_versions() {
    let base = Files::from([("host-tools-v1.txt".to_owned(), b"landed\n".to_vec())]);
    let mut tree = base.clone();
    tree.insert("host-tools-v1.txt".to_owned(), b"edited\n".to_vec());
    assert_eq!(check(&base, &tree), ["landed contract changed or removed: host-tools-v1.txt"]);

    tree.clear();
    assert!(check(&base, &tree).contains(&"landed contract changed or removed: host-tools-v1.txt".to_owned()));

    tree = base.clone();
    tree.insert("host-tools-v3.txt".to_owned(), b"new\n".to_vec());
    assert_eq!(check(&base, &tree), ["missing kept contract: host-tools-v2.txt"]);

    tree.remove("host-tools-v3.txt");
    tree.insert("host-tools-v2.txt".to_owned(), b"new\n".to_vec());
    assert!(check(&base, &tree).is_empty(), "a new version preserves the base and fills the sequence");
}
