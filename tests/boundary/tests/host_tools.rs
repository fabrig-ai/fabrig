//! Kept host-tool contracts, contiguous versions and breaking floors
//! (temper/docs/design/protocol/wires.md, 7 and 11).

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

fn tracked_name(name: &str) -> bool {
    name == "floor" || contract_name(name) || (name.starts_with("host-tools-v") && name.ends_with(".breaking"))
}

fn landed(root: &Path, base: &str) -> Files {
    let listed =
        git(root, &["ls-tree", "-r", "--name-only", "-z", base, "--", CONTRACT]).expect("list base's kept contracts");
    let mut files = Files::new();
    for bytes in listed.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(bytes).expect("git path is UTF-8");
        let name = Path::new(path).file_name().expect("contract filename").to_str().expect("UTF-8 filename");
        if tracked_name(name) {
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
        if tracked_name(&name) {
            files.insert(name, fs::read(entry.path()).expect("read kept text"));
        }
    }
    files
}

fn check(base: &Files, tree: &Files) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, bytes) in base {
        if contract_name(name) && tree.get(name) != Some(bytes) {
            failures.push(format!("landed contract changed or removed: {name}"));
        }
    }
    let mut versions = BTreeSet::new();
    for name in tree.keys().filter(|name| contract_name(name)) {
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
    check_floor(base, tree, greatest, &mut failures);
    failures
}

fn floor(files: &Files) -> Result<u16, String> {
    let Some(bytes) = files.get("floor") else { return Err("missing contract/floor".to_owned()) };
    let number = std::str::from_utf8(bytes)
        .ok()
        .and_then(|text| text.strip_suffix('\n'))
        .and_then(|text| text.parse::<u16>().ok());
    match number {
        Some(number) if number != 0 && *bytes == format!("{number}\n").as_bytes() => Ok(number),
        Some(_) | None => Err("contract/floor must be one positive decimal line".to_owned()),
    }
}

fn check_floor(base: &Files, tree: &Files, current: u16, failures: &mut Vec<String>) {
    // A base predating contract/floor answers from version 1.
    let previous_floor = if base.contains_key("floor") { floor(base).expect("the landed floor is valid") } else { 1 };
    let floor = match floor(tree) {
        Ok(floor) => floor,
        Err(why) => {
            failures.push(why);
            return;
        }
    };
    if floor < previous_floor {
        failures.push(format!("contract/floor lowered from {previous_floor} to {floor}"));
    }
    if floor > current {
        failures.push(format!("contract/floor {floor} exceeds current version {current}"));
    }
    let previous = base
        .keys()
        .filter_map(|name| name.strip_prefix("host-tools-v")?.strip_suffix(".txt")?.parse::<u16>().ok())
        .max()
        .unwrap_or(1);
    if current > previous {
        let breaking = format!("host-tools-v{current}.breaking");
        if let Some(reason) = tree.get(&breaking) {
            if std::str::from_utf8(reason).unwrap_or_default().trim().is_empty() {
                failures.push(format!("{breaking} must hold the breaking move's reason"));
            }
            if floor != current {
                failures.push(format!("{breaking} requires contract/floor {current}, not {floor}"));
            }
        } else if floor > previous {
            failures.push(format!("contract/floor {floor} passes previous version {previous}: add {breaking} with the reason, or keep the floor at or below {previous}"));
        }
    }
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
    let base =
        Files::from([("host-tools-v1.txt".to_owned(), b"landed\n".to_vec()), ("floor".to_owned(), b"1\n".to_vec())]);
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

#[test]
fn floor_mutations_require_a_breaking_move_and_never_lower_the_floor() {
    let scratch = skein_scratch::Scratch::new("host-tools-floor");
    let root = scratch.path();
    let directory = root.join(CONTRACT);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("host-tools-v1.txt"), b"one\n").unwrap();
    fs::write(directory.join("floor"), b"1\n").unwrap();
    let base = current(root);
    fs::write(directory.join("host-tools-v2.txt"), b"two\n").unwrap();
    assert!(check(&base, &current(root)).is_empty(), "a compatible version keeps the floor");
    fs::write(directory.join("floor"), b"2\n").unwrap();
    assert_eq!(
        check(&base, &current(root)),
        [
            "contract/floor 2 passes previous version 1: add host-tools-v2.breaking with the reason, or keep the floor at or below 1"
        ]
    );
    fs::write(directory.join("host-tools-v2.breaking"), b"The old meaning cannot be preserved.\n").unwrap();
    assert!(check(&base, &current(root)).is_empty(), "the declared break moves the floor to the new version");
    fs::write(directory.join("floor"), b"1\n").unwrap();
    assert_eq!(check(&base, &current(root)), ["host-tools-v2.breaking requires contract/floor 2, not 1"]);
    fs::write(directory.join("floor"), b"2\n").unwrap();
    let raised = current(root);
    fs::write(directory.join("floor"), b"1\n").unwrap();
    assert_eq!(check(&raised, &current(root)), ["contract/floor lowered from 2 to 1"]);
    fs::write(directory.join("floor"), b"3\n").unwrap();
    assert_eq!(check(&raised, &current(root)), ["contract/floor 3 exceeds current version 2"]);
    for invalid in [b"0\n".as_slice(), b"01\n", b"1", b"1\n\n"] {
        fs::write(directory.join("floor"), invalid).unwrap();
        assert_eq!(check(&base, &current(root)), ["contract/floor must be one positive decimal line"]);
    }
    fs::remove_file(directory.join("floor")).unwrap();
    assert_eq!(check(&base, &current(root)), ["missing contract/floor"]);
    fs::write(directory.join("floor"), b"2\n").unwrap();
    fs::write(directory.join("host-tools-v2.breaking"), b"\n").unwrap();
    assert_eq!(check(&base, &current(root)), ["host-tools-v2.breaking must hold the breaking move's reason"]);
}
