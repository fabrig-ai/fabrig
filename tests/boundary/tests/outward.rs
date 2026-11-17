//! fabrig names nothing of whoever runs its development: no repository,
//! tool, machine or session of that lab (AGENTS.md, 1). The rule is absolute:
//! any name in a file the check reads fails it. The check reads git's tracked
//! files from the workspace root with `git ls-files`, so `target/`, `.git/`
//! and other untracked build output are never listed. A symlink's text is its
//! target's path, as git stores it.
//!
//! The list, matching, left-out paths and verdict are tools/outward/design.md, 2–4.
//! Names live in tools/outward/names.txt, which both this test and the build-free
//! text check read. Only that list and this file's cases spell names.
//!
//! A file it cannot read is a failure, never a skip.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

const OWN: &str = "tests/boundary/tests/outward.rs";

const LIST: &str = "tools/outward/names.txt";
const LIST_TEXT: &str = include_str!("../../../tools/outward/names.txt");

#[derive(Clone, Copy)]
enum Right {
    Word,
    Any,
    Digits,
}

struct Name {
    text: Vec<u8>,
    left: bool,
    right: Right,
}

fn parse(text: &str) -> Result<Vec<Name>, String> {
    let mut result = Vec::new();
    for (number, line) in text.split('\n').enumerate() {
        let malformed = || format!("names.txt:{}: malformed entry", number + 1);
        if !line.is_ascii() {
            return Err(malformed());
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let bytes = line.as_bytes();
        let Some((&left, rest)) = bytes.split_first() else { return Err(malformed()) };
        let (Some(b' '), Some(&right), Some(b' '), Some(name)) =
            (rest.first(), rest.get(1), rest.get(2), rest.get(3..))
        else {
            return Err(malformed());
        };
        let left = match left {
            b'w' => true,
            b'-' => false,
            _ => return Err(malformed()),
        };
        let right = match right {
            b'w' => Right::Word,
            b'-' => Right::Any,
            b'd' => Right::Digits,
            _ => return Err(malformed()),
        };
        if name.is_empty()
            || name.first() == Some(&b' ')
            || name.last() == Some(&b' ')
            || name.iter().any(|byte| !(0x20..=0x7e).contains(byte) || byte.is_ascii_uppercase())
        {
            return Err(malformed());
        }
        result.push(Name { text: name.to_vec(), left, right });
    }
    Ok(result)
}

fn entries() -> &'static [Name] {
    static ENTRIES: OnceLock<Vec<Name>> = OnceLock::new();
    ENTRIES.get_or_init(|| parse(LIST_TEXT).unwrap_or_else(|why| panic!("{why}")))
}

fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The lab's names on `line`, left to right, never overlapping, each as the line spells it.
fn names(line: &[u8]) -> Vec<&[u8]> {
    // Keep the list's precedence, but inspect only names with this initial byte.
    static INITIALS: OnceLock<[Vec<&'static Name>; 256]> = OnceLock::new();
    let initials = INITIALS.get_or_init(|| {
        let mut initials: [Vec<&Name>; 256] = std::array::from_fn(|_| Vec::new());
        for entry in entries() {
            initials[usize::from(entry.text[0].to_ascii_lowercase())].push(entry);
        }
        initials
    });
    let mut result = Vec::new();
    let mut start = 0;
    while start < line.len() {
        let left = start == 0 || !word(line[start - 1]);
        let rest = &line[start..];
        let length = initials[usize::from(rest[0].to_ascii_lowercase())].iter().find_map(|entry| {
            let name = &entry.text;
            if (entry.left && !left) || !rest.get(..name.len()).is_some_and(|text| text.eq_ignore_ascii_case(name)) {
                return None;
            }
            let end = start + name.len();
            match entry.right {
                Right::Any => Some(name.len()),
                Right::Word => (end == line.len() || !word(line[end])).then_some(name.len()),
                Right::Digits => {
                    let digits = line[end..].iter().take_while(|byte| byte.is_ascii_digit()).count();
                    (digits > 0).then_some(name.len() + digits)
                }
            }
        });
        if let Some(length) = length {
            result.push(&line[start..start + length]);
            start += length;
        } else {
            start += 1;
        }
    }
    result
}

fn prefix_key(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |key, byte| (key << 8) | u32::from(byte.to_ascii_lowercase()))
}

/// A case-folded prefix set derived from the list, never another list of names.
fn may_hold_name(text: &[u8]) -> bool {
    static PREFIXES: OnceLock<(usize, Vec<u32>, Box<[bool]>)> = OnceLock::new();
    let (width, prefixes, pairs) = PREFIXES.get_or_init(|| {
        let width = entries().iter().map(|entry| entry.text.len()).min().unwrap_or(1).min(4);
        let mut prefixes: Vec<u32> = entries().iter().map(|entry| prefix_key(&entry.text[..width])).collect();
        prefixes.sort_unstable();
        prefixes.dedup();
        let mut pairs = vec![false; 65_536].into_boxed_slice();
        for entry in entries() {
            let pair = prefix_key(&entry.text[..width.min(2)]);
            pairs[usize::try_from(pair).expect("two-byte prefix")] = true;
        }
        (width, prefixes, pairs)
    });
    text.windows(*width).any(|part| {
        let pair = if *width == 1 {
            usize::from(part[0].to_ascii_lowercase())
        } else {
            (usize::from(part[0].to_ascii_lowercase()) << 8) | usize::from(part[1].to_ascii_lowercase())
        };
        pairs[pair] && prefixes.binary_search(&prefix_key(part)).is_ok()
    })
}

/// Every name in a file's `text`: its line, from 1, and its text.
fn found(text: &[u8]) -> Vec<(usize, String)> {
    if !may_hold_name(text) {
        return Vec::new();
    }
    text.split(|byte| *byte == b'\n')
        .enumerate()
        .filter(|(_, line)| may_hold_name(line))
        .flat_map(|(line, text)| {
            names(text).into_iter().map(move |name| (line + 1, String::from_utf8_lossy(name).into_owned()))
        })
        .collect()
}

/// Whether the check leaves out `path` (from the root, `/`-separated): the list, its own file, and
/// anything inside a plan's directory, `*/docs/plan/<draft>/`.
fn left_out(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    path == LIST || path == OWN || matches!(parts.as_slice(), [_, "docs", "plan", _, _, ..])
}

/// Whether a file's `bytes` are binary: a NUL among the first 8000.
fn binary(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(8000)].contains(&0)
}

/// What the scan read: each text file's path and byte count, and the names of those that hold any.
struct Scan {
    read: Vec<String>,
    bytes: usize,
    found: BTreeMap<String, Vec<(usize, String)>>,
}

/// `git -C root ls-files -z`, then each listed path not left out: a symlink's text is its
/// target path as git stores it; anything else that is not a file is an error.
/// A binary file is skipped.
fn scan(root: &Path) -> Result<Scan, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|why| format!("git ls-files: {why}"))?;
    if !output.status.success() {
        return Err(format!("git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr)));
    }
    let mut scan = Scan { read: Vec::new(), bytes: 0, found: BTreeMap::new() };
    for raw in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(raw).map_err(|why| format!("git ls-files: path {raw:?} is not UTF-8: {why}"))?;
        if left_out(path) {
            continue;
        }
        let absolute = root.join(path);
        let metadata = fs::symlink_metadata(&absolute).map_err(|why| format!("{path}: {why}"))?;
        let bytes = if metadata.is_symlink() {
            fs::read_link(&absolute).map_err(|why| format!("{path}: {why}"))?.into_os_string().into_encoded_bytes()
        } else if metadata.is_file() {
            fs::read(&absolute).map_err(|why| format!("{path}: {why}"))?
        } else {
            return Err(format!("{path}: neither a file nor a symlink"));
        };
        if binary(&bytes) {
            continue;
        }
        scan.read.push(path.to_owned());
        scan.bytes += bytes.len();
        let names = found(&bytes);
        if !names.is_empty() {
            scan.found.insert(path.to_owned(), names);
        }
    }
    Ok(scan)
}

/// The verdict on `found`: a failure listing each file, line and name, if any file holds one.
fn verdict(found: &BTreeMap<String, Vec<(usize, String)>>) -> Result<(), String> {
    if found.is_empty() {
        return Ok(());
    }
    let mut failure = String::from("Files name the lab running fabrig's development (AGENTS.md, 1):\n");
    for (path, names) in found {
        for (line, name) in names {
            writeln!(failure, "{path}:{line}: {name}").expect("writing to a string succeeds");
        }
    }
    Err(failure)
}

#[test]
fn malformed_lists_name_the_line() {
    for line in [
        "x w name",
        "w x name",
        "w w",
        "w w ",
        "w  w name",
        "w w name ",
        "w w é",
        "w w Name",
        "w w name\r",
        "w w na\tme",
        "# é",
    ] {
        let text = format!("# list\n\n{line}\n");
        let Err(error) = parse(&text) else { panic!("a malformed line must fail") };
        assert_eq!(error, "names.txt:3: malformed entry", "{line:?}");
    }
    assert_eq!(parse("# list\n\nw w name\n").expect("a valid entry").len(), 1);
}

#[test]
fn the_text_check_and_rust_give_one_verdict() {
    let scratch = skein_scratch::Scratch::new("outward-agreement");
    let root = scratch.path();
    git(root, &["init", "-q"]);
    let mut text = String::new();
    for entry in entries() {
        let name = std::str::from_utf8(&entry.text).expect("ASCII name");
        let suffix = match entry.right {
            Right::Digits => "123suffix",
            Right::Any | Right::Word => "",
        };
        writeln!(text, "({}{suffix})\r", name.to_ascii_uppercase()).expect("writing text");
        writeln!(text, "x{name}{suffix}!").expect("writing text with a left word neighbour");
        writeln!(text, "!{name}{suffix}_").expect("writing text with a right word neighbour");
    }
    text.push_str("fab and hel8\n");
    fs::write(root.join("names.md"), &text).expect("names of every entry kind");
    fs::write(root.join("binary"), [b"\0".as_slice(), text.as_bytes()].concat()).expect("a binary file");
    let plan = root.join("temper/docs/plan/example");
    fs::create_dir_all(&plan).expect("a plan directory");
    fs::write(plan.join("run.md"), &text).expect("a plan's names");
    fs::create_dir_all(root.join("tools/outward")).expect("the list's directory");
    fs::write(root.join(LIST), LIST_TEXT).expect("the list");
    std::os::unix::fs::symlink("/home/free/elsewhere", root.join("link")).expect("a target with a name");
    git(root, &["add", "."]);
    let command = skein_scratch::paths::workspace().join("tools/outward/outward.py");
    let check =
        || Command::new("python3").args(["-I", "-B"]).arg(&command).current_dir(root).output().expect("the text check");
    let rust = scan(root).expect("scanning the scratch repository");
    let failure = verdict(&rust.found).expect_err("the names must fail");
    let python = check();
    assert_eq!(python.status.code(), Some(1), "{}", String::from_utf8_lossy(&python.stderr));
    assert_eq!(String::from_utf8(python.stdout).expect("UTF-8 verdict"), failure);
    assert_eq!(python.stderr.as_slice(), b"");

    fs::write(root.join("names.md"), "Product prose\n").expect("taking the names out");
    fs::remove_file(root.join("link")).expect("taking out the symlink's name");
    std::os::unix::fs::symlink("product/elsewhere", root.join("link")).expect("a product target");
    assert_eq!(verdict(&scan(root).expect("scanning clean text").found), Ok(()));
    let python = check();
    assert_eq!(python.status.code(), Some(0), "{}", String::from_utf8_lossy(&python.stderr));
    assert_eq!(python.stdout.as_slice(), b"");
    assert_eq!(python.stderr.as_slice(), b"");
}

#[test]
fn no_file_names_the_lab() {
    let root = skein_scratch::paths::workspace();
    let scan = scan(&root).unwrap_or_else(|why| panic!("the workspace's files: {why}"));
    for prefix in ["skein/", "smith/", "jig/", "temper/", "hearth/", "tools/", "tests/"] {
        assert!(scan.read.iter().any(|path| path.starts_with(prefix)), "the check reads {prefix}");
    }
    assert!(scan.read.iter().any(|path| path == "AGENTS.md"), "the check reads AGENTS.md");
    eprintln!("Read {} text files ({} bytes).", scan.read.len(), scan.bytes);
    eprintln!("Found {} names in {} files.", scan.found.values().map(Vec::len).sum::<usize>(), scan.found.len());
    if let Err(why) = verdict(&scan.found) {
        panic!("{why}");
    }
}

#[test]
fn each_name_of_the_lab_is_found() {
    for (line, expected) in [
        ("xFABRIG-ai/AGENTS!", vec!["FABRIG-ai/AGENTS"]),
        ("The agents repository's fab/cpu.py", vec!["agents repository", "fab"]),
        ("agents repo; AGENTS REPOS; agents repositories", vec!["agents repo", "AGENTS REPOS", "agents repositories"]),
        ("FAB_HOME=x heavy --gate", vec!["FAB_HOME", "heavy --"]),
        ("heavy cargo, HeAvY EnV", vec!["heavy cargo", "HeAvY EnV"]),
        (r#"["heavy", 'heavy']"#, vec!["\"heavy\"", "'heavy'"]),
        ("hel8, hetzner12 and debian-16gb-hel1-1", vec!["hel8", "hetzner12", "debian-16gb"]),
        ("HETZNER1234suffix status.fabrig.ai", vec!["HETZNER1234", "status.fabrig.ai"]),
        ("~/src/fabrig/x ~/src/rust/tmp/x /home/free/x", vec!["~/src/fabrig/", "~/src/rust/tmp/", "/home/free/"]),
        ("fabrig/qa fabrig/dogfood fabrig/debt", vec!["fabrig/qa", "fabrig/dogfood", "fabrig/debt"]),
        (
            "agents/loop agents/status agents/creds agents/ops",
            vec!["agents/loop", "agents/status", "agents/creds", "agents/ops"],
        ),
        ("(fab) fab's /fab/ éfabé", vec!["fab", "fab", "fab", "fab"]),
    ] {
        let expected: Vec<&[u8]> = expected.iter().map(|name| name.as_bytes()).collect();
        assert_eq!(names(line.as_bytes()), expected, "{line}");
        assert_eq!(
            found(line.as_bytes()),
            expected.iter().map(|name| (1, String::from_utf8_lossy(name).into_owned())).collect::<Vec<_>>(),
            "{line}"
        );
    }
    assert_eq!(found(b"one\nfab two\r\nhel8"), [(2, "fab".to_owned()), (3, "hel8".to_owned())]);
}

#[test]
fn names_on_a_line_never_overlap() {
    for (line, expected) in
        [("/home/free/src/rust/tmp/x", "/home/free/"), ("~/src/fabrig/qa", "~/src/fabrig/"), ("bin/fab", "fab")]
    {
        assert_eq!(names(line.as_bytes()), [expected.as_bytes()], "{line}");
    }
}

#[test]
fn product_words_pass() {
    for line in [
        "tool-heavy",
        "the director task",
        "/home/<user>/",
        "fabric",
        "fabrig",
        "prefab",
        "a heavy test",
        "a heavy environment",
        "heavy cargoes",
        "spawned agents report their errors",
        "the agent's repository",
        "smith/benchmarks/agents/codex",
        "hetzner",
        "hel80",
        "/home/freebsd/",
        "~/src/fabrig",
        "_fab fab_ 1fab fab2",
        "agents repository_",
        "heavy  cargo",
        "fabrig-ai/agents_report",
    ] {
        assert!(names(line.as_bytes()).is_empty(), "{line}");
    }
}

#[test]
fn the_check_leaves_out_plan_directories_and_its_own_file() {
    for (path, expected) in [
        ("temper/docs/plan/outward-rule/inputs/request.md", true),
        ("smith/docs/plan/sign-in-wait/plan.toml", true),
        (OWN, true),
        (LIST, true),
        ("temper/docs/plan/a.md", false),
        ("docs/plan/a/b.md", false),
        ("temper/crates/a/docs/plan/b/c.md", false),
        ("temper/docs/planning.md", false),
        ("temper/docs/design/plan/a.md", false),
        ("tests/boundary/tests/direction.rs", false),
    ] {
        assert_eq!(left_out(path), expected, "{path}");
    }
}

#[test]
fn a_nul_in_the_first_8000_bytes_makes_a_file_binary() {
    for (nul, expected) in [(Some(0), true), (Some(7999), true), (Some(8000), false), (None, false)] {
        let mut bytes = vec![b'x'; 8001];
        if let Some(index) = nul {
            bytes[index] = 0;
        }
        assert_eq!(binary(&bytes), expected, "NUL at {nul:?}");
    }
}

#[test]
fn no_name_passes() {
    assert_eq!(verdict(&BTreeMap::new()), Ok(()));
}

#[test]
fn any_name_fails_and_the_failure_lists_each_file_line_and_match() {
    let found = BTreeMap::from([
        ("a.md".to_owned(), vec![(2, "fab".to_owned()), (7, "heavy cargo".to_owned())]),
        ("b.md".to_owned(), vec![(1, "hel8".to_owned())]),
    ]);
    let failure = verdict(&found).expect_err("any name fails");
    for text in ["a.md:2: fab", "a.md:7: heavy cargo", "b.md:1: hel8"] {
        assert!(failure.contains(text), "missing {text:?}: {failure}");
    }
}

#[test]
fn a_symlinks_target_is_read_as_its_text() {
    let scratch = skein_scratch::Scratch::new("outward-symlink");
    git(scratch.path(), &["init", "-q"]);
    std::os::unix::fs::symlink("/home/free/elsewhere", scratch.path().join("link")).expect("creating the symlink");
    git(scratch.path(), &["add", "link"]);
    let scan = scan(scratch.path()).expect("reading the symlink's text");
    assert_eq!(scan.found, BTreeMap::from([("link".to_owned(), vec![(1, "/home/free/".to_owned())])]));
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git").arg("-C").arg(root).args(args).output().expect("running git");
    assert!(output.status.success(), "git {args:?}: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
}
