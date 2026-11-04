//! fabrig names nothing of whoever runs its development: no repository,
//! tool, machine or session of that lab (AGENTS.md, 1). The rule is absolute:
//! any name in a file the check reads fails it. The check reads git's tracked
//! files from the workspace root with `git ls-files`, so `target/`, `.git/`
//! and other untracked build output are never listed. A symlink's text is its
//! target's path, as git stores it.
//!
//! Three things are left out and nothing else: the directories
//! `*/docs/plan/<draft>/`, the transient run files of whoever runs a plan, each
//! of which leaves main when its plan retires; this file, which spells the
//! names; and binary files, a NUL byte in the first 8000 bytes, as git decides.
//!
//! Names match ASCII case-insensitively, with ASCII word boundaries where
//! required: `fabrig-ai/agents`, `agents repo`, `agents repos`, `agents
//! repository`, `agents repositories`, `fab`, `FAB_HOME`, `heavy cargo`,
//! `heavy env`, `heavy --`, `"heavy"`, `'heavy'`, `hel8`, `hetzner` followed
//! by digits, `debian-16gb`, `status.fabrig.ai`, `~/src/fabrig/`,
//! `~/src/rust/tmp/`, `/home/free/`, `fabrig/qa`, `fabrig/dogfood`,
//! `fabrig/debt`, `agents/loop`, `agents/status`, `agents/creds`, `agents/ops`.
//! A bare `director`, a bare `heavy` and a generic `/home/<user>/` are left
//! off on purpose: they have product meanings.
//!
//! A file it cannot read is a failure, never a skip.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

const OWN: &str = "tests/boundary/tests/outward.rs";

/// Each fixed name, and whether its left and right ends need word boundaries.
const FIXED: &[(&[u8], bool, bool)] = &[
    (b"fabrig-ai/agents", false, true),
    (b"agents repository", true, true),
    (b"agents repositories", true, true),
    (b"agents repos", true, true),
    (b"agents repo", true, true),
    (b"fab", true, true),
    (b"fab_home", true, true),
    (b"heavy cargo", true, true),
    (b"heavy env", true, true),
    (b"heavy --", true, false),
    (b"\"heavy\"", false, false),
    (b"'heavy'", false, false),
    (b"hel8", true, true),
    (b"debian-16gb", true, true),
    (b"status.fabrig.ai", true, true),
    (b"~/src/fabrig/", false, false),
    (b"~/src/rust/tmp/", false, false),
    (b"/home/free/", false, false),
    (b"fabrig/qa", true, true),
    (b"fabrig/dogfood", true, true),
    (b"fabrig/debt", true, true),
    (b"agents/loop", true, true),
    (b"agents/status", true, true),
    (b"agents/creds", true, true),
    (b"agents/ops", true, true),
];

fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The lab's names on `line`, left to right, never overlapping, each as the line spells it.
fn names(line: &[u8]) -> Vec<&[u8]> {
    let mut result = Vec::new();
    let mut start = 0;
    while start < line.len() {
        let left = start == 0 || !word(line[start - 1]);
        let rest = &line[start..];
        let fixed = FIXED.iter().filter(|(name, _, _)| name[0].eq_ignore_ascii_case(&rest[0])).find_map(
            |&(name, before, after)| {
                let end = start + name.len();
                (rest.get(..name.len()).is_some_and(|text| text.eq_ignore_ascii_case(name))
                    && (!before || left)
                    && (!after || end == line.len() || !word(line[end])))
                .then_some(name.len())
            },
        );
        let length = fixed.or_else(|| {
            if !left || !rest.get(..7).is_some_and(|text| text.eq_ignore_ascii_case(b"hetzner")) {
                return None;
            }
            let digits = rest[7..].iter().take_while(|byte| byte.is_ascii_digit()).count();
            (digits > 0).then_some(7 + digits)
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

/// Every name contains one of these four-byte prefixes, or ends in `fab`.
fn may_hold_name(text: &[u8]) -> bool {
    text.get(text.len().saturating_sub(3)..).is_some_and(|tail| tail.eq_ignore_ascii_case(b"fab"))
        || text.windows(4).any(|part| {
            matches!(
                [
                    part[0].to_ascii_lowercase(),
                    part[1].to_ascii_lowercase(),
                    part[2].to_ascii_lowercase(),
                    part[3].to_ascii_lowercase()
                ],
                [b'f', b'a', b'b', _]
                    | [b'a', b'g', b'e', b'n']
                    | [b'h', b'e', b'a', b'v']
                    | [b'h', b'e', b'l', b'8']
                    | [b'h', b'e', b't', b'z']
                    | [b'd', b'e', b'b', b'i']
                    | [b'~', b'/', b's', b'r']
                    | [b'/', b'h', b'o', b'm']
            )
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

/// Whether the check leaves out `path` (from the root, `/`-separated): its own file, and
/// anything inside a plan's directory, `*/docs/plan/<draft>/`.
fn left_out(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    path == OWN || matches!(parts.as_slice(), [_, "docs", "plan", _, _, ..])
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
fn no_file_names_the_lab() {
    let root = skein_scratch::paths::workspace();
    let scan = scan(&root).unwrap_or_else(|why| panic!("the workspace's files: {why}"));
    for prefix in ["skein/", "smith/", "jig/", "temper/", "tools/", "tests/"] {
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
