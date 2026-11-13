//! Client versions move with layouts (temper/docs/design/protocol/wires.md, 8).

use std::{collections::BTreeMap, fmt, fs, path::Path, process::Command};

use skein_codec::schema;
use skein_scratch::paths;

const FAMILIES: &[(&str, &str)] = &[
    ("jig-client-wire", "jig/crates/jig-client-protocol-wire"),
    ("jig-public-wire", "jig/crates/jig-client-protocol-wire"),
    ("jig-engine-cursor", "jig/crates/jig-protocol-engine-clients"),
    ("temper-client-wire", "temper/crates/temper-client-protocol-wire"),
    ("temper-public-wire", "temper/crates/temper-client-protocol-wire"),
];

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    family: String,
    version: u16,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} version {} changed layout: move its version", self.family, self.version)
    }
}

fn layouts(base_text: &str, current_text: &str) -> Option<Finding> {
    let base = schema::parse(base_text.as_bytes()).expect("valid base family");
    let current = schema::parse(current_text.as_bytes()).expect("valid current family");
    assert_eq!(base.spelled(base.family()), current.spelled(current.family()), "compare the same family");
    if base.version() == current.version() && base.fingerprints().family() != current.fingerprints().family() {
        Some(Finding {
            family: String::from_utf8(current.spelled(current.family()).to_vec()).expect("family name is UTF-8"),
            version: current.version(),
        })
    } else {
        None
    }
}

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
    eprintln!("client layout check base: {reference} (git {})", args.join(" "));
    reference
}

fn current_files(owner: &Path) -> BTreeMap<String, String> {
    fs::read_dir(owner.join("schema"))
        .expect("read client schemas")
        .map(|entry| entry.expect("read schema entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "schema") && path.is_file())
        .map(|path| {
            let name = path.file_name().expect("schema file name").to_str().expect("UTF-8 file name").to_owned();
            (name, fs::read_to_string(path).expect("read schema text"))
        })
        .collect()
}

fn base_files(owner: &Path, base: &str) -> BTreeMap<String, String> {
    let files = git(owner, &["ls-tree", "-r", "--name-only", "-z", base, "--", "schema"]).expect("list base schemas");
    files
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| std::str::from_utf8(bytes).expect("UTF-8 git path"))
        .map(Path::new)
        .filter(|path| {
            path.parent() == Some(Path::new("schema")) && path.extension().is_some_and(|ext| ext == "schema")
        })
        .map(|path| {
            let object = format!("{base}:./{}", path.display());
            let text = String::from_utf8(git(owner, &["show", &object]).expect("read base schema"))
                .expect("UTF-8 base schema");
            (path.file_name().expect("schema file name").to_str().expect("UTF-8 file name").to_owned(), text)
        })
        .collect()
}

fn header(text: &str) -> Option<&str> {
    let line = text.lines().map(str::trim).find(|line| !line.is_empty() && !line.starts_with('#'))?;
    let mut words = line.split_whitespace();
    (words.next()? == "family").then(|| words.next()).flatten()
}

fn family_text(files: &BTreeMap<String, String>, family: &str) -> Option<String> {
    let mut roots = files.iter().filter(|(_, text)| header(text) == Some(family));
    let (name, text) = roots.next()?;
    assert!(roots.next().is_none(), "more than one current header for {family}");
    let mut text = text.clone();
    // The client generator concatenates the header file then its part1 fragment
    // (jig/tests/codec-families.rs), without inserting a separator.
    let prefix = format!("{}-part", name.strip_suffix(".schema").expect("schema extension"));
    for (_, fragment) in files.iter().filter(|(name, _)| name.starts_with(&prefix)) {
        text.push_str(fragment);
    }
    Some(text)
}

#[test]
fn client_layout_changes_move_their_versions() {
    let root = paths::workspace();
    let base = base(&root);
    let mut failures = Vec::new();
    for (family, owner) in FAMILIES {
        let owner = root.join(owner);
        let current = family_text(&current_files(&owner), family).expect("current family header exists");
        if let Some(before) = family_text(&base_files(&owner, &base), family)
            && let Some(finding) = layouts(&before, &current)
        {
            failures.push(format!("{finding} (base {base})"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const BEFORE: &str = "family jig-client-wire 19\n# A client record.\nrecord Value { value: u64 }\n";

#[test]
fn a_field_added_at_the_same_version_needs_a_version_move() {
    let changed = BEFORE.replace("value: u64", "value: u64 added: u32");
    let finding = layouts(BEFORE, &changed).expect("a field added without a version move is refused");
    assert_eq!(finding, Finding { family: "jig-client-wire".into(), version: 19 });
    assert_eq!(finding.to_string(), "jig-client-wire version 19 changed layout: move its version");
}

#[test]
fn a_field_added_with_its_version_moved_passes() {
    let changed = BEFORE.replace("value: u64", "value: u64 added: u32").replace("wire 19", "wire 20");
    assert_eq!(layouts(BEFORE, &changed), None);
}

#[test]
fn a_comment_change_passes() {
    let changed = BEFORE.replace("A client record.", "Another comment.");
    assert_eq!(layouts(BEFORE, &changed), None);
}

#[test]
fn headers_find_families_and_include_their_fragments() {
    let files = BTreeMap::from([
        ("client-v19.schema".into(), BEFORE.into()),
        ("client-v19-part1.schema".into(), "record Tail { value: Value }\n".into()),
        ("public-v1.schema".into(), "family jig-public-wire 1\nrecord Public { value: u8 }\n".into()),
    ]);
    let text = family_text(&files, "jig-client-wire").expect("client found by its header");
    assert!(schema::parse(text.as_bytes()).expect("whole family parses").find(b"Tail").is_some());
    let changed = text.replace("value: Value", "value: Value added: u32");
    assert!(layouts(&text, &changed).is_some(), "a fragment's change is part of the layout");
    assert!(family_text(&files, "temper-client-wire").is_none(), "an absent base family passes");
    assert_eq!(family_text(&files, "jig-public-wire"), files.get("public-v1.schema").cloned());
}
