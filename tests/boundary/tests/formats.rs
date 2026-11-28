//! Released format fixtures keep their paths and bytes
//! (temper/docs/design/protocol/configuration/file-formats.md, 7).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use skein_scratch::paths;

const FORMATS: &str = "temper/docs/examples/formats/";
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
    if let Some(commit) = fabrig_boundary::gate_landing_base() {
        return commit;
    }
    let args = if git(root, &["rev-parse", "--verify", "MERGE_HEAD^{commit}"]).is_ok() {
        vec!["rev-parse", "MERGE_HEAD"]
    } else if git(root, &["rev-parse", "--verify", "origin/main^{commit}"]).is_ok() {
        vec!["merge-base", "HEAD", "origin/main"]
    } else {
        vec!["rev-parse", "HEAD"]
    };
    let reference = String::from_utf8(git(root, &args).expect("read the landed base")).expect("git's base is UTF-8");
    let reference = reference.trim().to_owned();
    eprintln!("format fixture base: {reference} (git {})", args.join(" "));
    reference
}

fn landed(root: &Path, base: &str) -> Files {
    let listed = git(root, &["ls-tree", "-r", "--name-only", "-z", base, "--", FORMATS]).expect("list landed fixtures");
    listed
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path).expect("Git path is UTF-8");
            let bytes = git(root, &["show", &format!("{base}:{path}")]).expect("read landed fixture bytes");
            (path.to_owned(), bytes)
        })
        .collect()
}

fn read_fixture(root: &Path, name: &str) -> io::Result<Option<Vec<u8>>> {
    let mut path = root.to_path_buf();
    let mut parts = Path::new(name).components().peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        let kind = match path.symlink_metadata() {
            Ok(metadata) => metadata.file_type(),
            Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(why) => return Err(why),
        };
        if parts.peek().is_some() {
            if !kind.is_dir() {
                return Ok(None);
            }
        } else if kind.is_file() {
            return fs::read(path).map(Some);
        } else if kind.is_symlink() {
            // Git stores the link's target path as its bytes, not the file it points to.
            return fs::read_link(path).map(|target| Some(target.as_os_str().as_encoded_bytes().to_vec()));
        }
    }
    Ok(None)
}

fn current(root: &Path, base: &Files) -> Files {
    // Only landed paths need reading; additions are unrestricted by this check.
    base.keys()
        .filter_map(|path| {
            read_fixture(root, path)
                .unwrap_or_else(|why| panic!("{path}: read current fixture: {why}"))
                .map(|bytes| (path.clone(), bytes))
        })
        .collect()
}

fn check(base: &Files, tree: &Files) -> Vec<String> {
    let mut failures = Vec::new();
    for (path, bytes) in base {
        if !path.starts_with(FORMATS) {
            continue;
        }
        let change = match tree.get(path) {
            Some(current) if current == bytes => continue,
            Some(_) => "changed",
            None => "removed",
        };
        failures.push(format!("{path}: {change} since its release; a format's fixtures are never edited"));
    }
    failures
}

#[test]
fn released_format_fixtures_are_frozen() {
    let root = paths::workspace();
    let base = base(&root);
    let landed = landed(&root, &base);
    let failures = check(&landed, &current(&root, &landed));
    assert!(failures.is_empty(), "base {base}:\n{}", failures.join("\n"));
}

const BOX: &str = "temper/docs/examples/formats/temper-toml/1/box.toml";
const ENGINE: &str = "temper/docs/examples/formats/temper-toml/1/engine.toml";
const NEXT_BOX: &str = "temper/docs/examples/formats/temper-toml/2/box.toml";
const OUTSIDE: &str = "temper/docs/examples/formats-other/box.toml";
const KEPT_BYTES: &[u8] = b"# fixture\x80\n";
const CHANGED_BYTES: &[u8] = b"# fixture\x81\n";

fn files(entries: &[(&str, &[u8])]) -> Files {
    entries.iter().map(|(path, bytes)| ((*path).to_owned(), bytes.to_vec())).collect()
}

#[test]
fn changing_one_byte_of_a_released_fixture_is_refused() {
    let failures = check(&files(&[(BOX, KEPT_BYTES)]), &files(&[(BOX, CHANGED_BYTES)]));
    assert_eq!(
        failures,
        [
            "temper/docs/examples/formats/temper-toml/1/box.toml: changed since its release; a format's fixtures are never edited"
        ]
    );
}

#[test]
fn removing_a_released_fixture_is_refused() {
    let failures = check(&files(&[(BOX, KEPT_BYTES)]), &Files::new());
    assert_eq!(
        failures,
        [
            "temper/docs/examples/formats/temper-toml/1/box.toml: removed since its release; a format's fixtures are never edited"
        ]
    );
}

#[test]
fn additions_and_changes_outside_released_fixtures_pass() {
    let kept = (BOX, KEPT_BYTES);
    for (case, base, tree) in [
        ("file added", files(&[kept]), files(&[kept, (ENGINE, KEPT_BYTES)])),
        ("directory added", files(&[kept]), files(&[kept, (NEXT_BOX, KEPT_BYTES)])),
        ("outside file edited", files(&[kept, (OUTSIDE, KEPT_BYTES)]), files(&[kept, (OUTSIDE, CHANGED_BYTES)])),
    ] {
        let failures = check(&base, &tree);
        assert!(failures.is_empty(), "{case}: {}", failures.join("\n"));
    }
}
