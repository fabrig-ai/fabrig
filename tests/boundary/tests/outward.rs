//! fabrig names nothing of whoever runs its development: no repository,
//! tool, machine or session of that lab (AGENTS.md, 2). The check reads
//! git's tracked files from the workspace root with `git ls-files`, so
//! `target/`, `.git/` and other untracked build output are never listed.
//! It skips symlinks, whose targets are tracked files read under their own
//! paths, and binary files: a NUL byte in the first 8000 bytes, as git decides.
//! It leaves out its own file, its inventory, and every `docs/plan/` directory
//! until the plans move out.
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
//! The inventory lists each file's maximum count; an unlisted file holds
//! none, and each count may only fall. Raising a count or adding a file
//! needs a `# Raised: <who, why>` line directly above its entry.
//! A line it cannot read is a failure, never a skip.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

const OWN: &str = "tests/boundary/tests/outward.rs";
const INVENTORY: &str = "tests/boundary/outward.txt";

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

/// Whether the check leaves out `path` (from the root, `/`-separated): its own file, its
/// inventory, and anything with a `docs/plan/` directory in it, at the root or in a tree.
fn left_out(path: &str) -> bool {
    path == OWN || path == INVENTORY || path.starts_with("docs/plan/") || path.contains("/docs/plan/")
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

/// `git -C root ls-files -z`, then each listed path not left out: a symlink is skipped,
/// anything else that is not a file is an error, a binary file is skipped.
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
        if metadata.is_symlink() {
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("{path}: neither a file nor a symlink"));
        }
        let bytes = fs::read(&absolute).map_err(|why| format!("{path}: {why}"))?;
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

/// An entry has one space, a path without whitespace and a positive decimal count.
fn entry(line: &str) -> Option<(&str, usize)> {
    let (path, count) = line.split_once(' ')?;
    if path.is_empty()
        || path.chars().any(char::is_whitespace)
        || path.starts_with('/')
        || path.split('/').any(|part| part == "." || part == ".." || part.is_empty())
        || count.starts_with('0')
        || count.is_empty()
        || !count.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some((path, count.parse().ok()?))
}

/// The inventory's entries: each listed path and its count.
fn inventory(source: &str) -> Result<BTreeMap<String, usize>, String> {
    let mut entries: BTreeMap<String, usize> = BTreeMap::new();
    for (number, line) in source.split_terminator('\n').enumerate() {
        if line.bytes().all(|byte| byte == b' ') || line.starts_with('#') {
            continue;
        }
        let error = |why| format!("{INVENTORY}:{}: {why}: {line}", number + 1);
        let (path, count) = entry(line).ok_or_else(|| error("an entry the check does not read"))?;
        if entries.contains_key(path) {
            return Err(error("listed twice"));
        }
        if entries.last_key_value().is_some_and(|(previous, _)| previous.as_str() > path) {
            return Err(error("out of order"));
        }
        entries.insert(path.to_owned(), count);
    }
    Ok(entries)
}

/// The inventory the tree needs, from the inventory's `source` and today's `counts`.
fn needed(source: &str, counts: &BTreeMap<String, usize>) -> String {
    let mut remaining: BTreeMap<_, _> =
        counts.iter().filter(|(_, count)| **count > 0).map(|(path, count)| (path.as_str(), *count)).collect();
    let mut output = String::new();
    let mut raised = String::new();
    for line in source.split_terminator('\n') {
        if line.starts_with("# Raised:") {
            writeln!(raised, "{line}").expect("writing to a string succeeds");
        } else if let Some((path, _)) = entry(line) {
            if let Some(&count) = counts.get(path).filter(|count| **count > 0) {
                while remaining.first_key_value().is_some_and(|(next, _)| *next < path) {
                    let (next, count) = remaining.pop_first().expect("the first entry exists");
                    writeln!(output, "{next} {count}").expect("writing to a string succeeds");
                }
                output.push_str(&raised);
                writeln!(output, "{path} {count}").expect("writing to a string succeeds");
                remaining.remove(path);
            }
            raised.clear();
        } else {
            output.push_str(&raised);
            raised.clear();
            writeln!(output, "{line}").expect("writing to a string succeeds");
        }
    }
    output.push_str(&raised);
    for (path, count) in remaining {
        writeln!(output, "{path} {count}").expect("writing to a string succeeds");
    }
    if output.is_empty() {
        output.push('\n');
    }
    output
}

/// The verdict on `found` against the inventory's `source`: the failure, when a file holds
/// more names than its entry allows. An unreadable `source` is a failure too.
fn verdict(found: &BTreeMap<String, Vec<(usize, String)>>, source: &str) -> Result<(), String> {
    let entries = inventory(source)?;
    let mut failure = String::new();
    for (path, names) in found {
        if names.len() <= entries.get(path).copied().unwrap_or(0) {
            continue;
        }
        if failure.is_empty() {
            writeln!(failure, "Files name the lab beyond {INVENTORY}; each count may only fall; raising one or adding a file needs a `# Raised: <who, why>` line directly above its entry.")
                .expect("writing to a string succeeds");
        }
        let allowed = entries.get(path).map_or_else(|| "not listed".to_owned(), usize::to_string);
        writeln!(failure, "{path}: {} names, inventory: {allowed}", names.len()).expect("writing to a string succeeds");
        for (line, name) in names {
            writeln!(failure, "{path}:{line}: {name}").expect("writing to a string succeeds");
        }
    }
    if failure.is_empty() {
        return Ok(());
    }
    let counts = found.iter().map(|(path, names)| (path.clone(), names.len())).collect();
    failure.push_str("The inventory the tree needs:\n");
    failure.push_str(&needed(source, &counts));
    Err(failure)
}

#[test]
fn no_file_names_the_lab_beyond_the_inventory() {
    let root = skein_scratch::paths::workspace();
    let scan = scan(&root).unwrap_or_else(|why| panic!("the workspace's files: {why}"));
    for prefix in ["skein/", "smith/", "jig/", "temper/", "tools/", "tests/"] {
        assert!(scan.read.iter().any(|path| path.starts_with(prefix)), "the check reads {prefix}");
    }
    assert!(scan.read.iter().any(|path| path == "AGENTS.md"), "the check reads AGENTS.md");
    eprintln!("Read {} text files ({} bytes).", scan.read.len(), scan.bytes);
    eprintln!("Found {} names in {} files.", scan.found.values().map(Vec::len).sum::<usize>(), scan.found.len());
    let source = fs::read_to_string(root.join(INVENTORY)).unwrap_or_else(|why| panic!("{INVENTORY}: {why}"));
    if let Err(why) = verdict(&scan.found, &source) {
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
fn the_check_leaves_out_plans_its_own_file_and_its_inventory() {
    for (path, expected) in [
        ("temper/docs/plan/outward-rule/inputs/request.md", true),
        ("docs/plan/a.md", true),
        (OWN, true),
        (INVENTORY, true),
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
fn an_inventory_line_the_check_does_not_read_fails_it() {
    for (source, number, why) in [
        ("a.md", 1, "does not read"),
        ("a.md x", 1, "does not read"),
        ("a.md 0", 1, "does not read"),
        ("a.md 01", 1, "does not read"),
        ("a.md 1 2", 1, "does not read"),
        ("a.md  1", 1, "does not read"),
        ("b.md 1\na.md 1", 2, "out of order"),
        ("a.md 1\na.md 2", 2, "listed twice"),
        ("a.md 1\nb.md 1\na.md 1", 3, "listed twice"),
        ("a\tb.md 1", 1, "does not read"),
    ] {
        let failure = inventory(source).expect_err("the inventory fails");
        let line = source.lines().nth(number - 1).expect("the failing line exists");
        assert!(
            failure.contains(&format!(":{number}:")) && failure.contains(line) && failure.contains(why),
            "{failure}"
        );
    }
    for source in ["# c\n\n.config/x 2\nA.md 1\nb/c.md 10\n", "# c\n   \n.config/x 2\nA.md 1\nb/c.md 10"] {
        assert_eq!(
            inventory(source),
            Ok(BTreeMap::from([(".config/x".to_owned(), 2), ("A.md".to_owned(), 1), ("b/c.md".to_owned(), 10)]))
        );
    }
}

#[test]
fn fewer_names_than_listed_pass() {
    for source in ["a.md 3\ngone.md 2\n", "a.md 1\n", "# Raised: lead, a reason\na.md 2\n"] {
        assert_eq!(verdict(&BTreeMap::from([("a.md".to_owned(), vec![(1, "fab".to_owned())])]), source), Ok(()));
    }
}

#[test]
fn a_file_beyond_its_count_fails_with_its_names_and_the_needed_inventory() {
    for source in ["# The header.\n\na.md 1\nc.md 3\n", "# The header.\n\na.md 1\nc.md 3"] {
        let found = BTreeMap::from([
            ("a.md".to_owned(), vec![(2, "fab".to_owned()), (7, "heavy cargo".to_owned())]),
            ("b.md".to_owned(), vec![(1, "hel8".to_owned())]),
            ("c.md".to_owned(), vec![(4, "/home/free/".to_owned())]),
        ]);
        let failure = verdict(&found, source).expect_err("the counts exceed the inventory");
        for text in [
            "a.md:2: fab",
            "a.md:7: heavy cargo",
            "b.md:1: hel8",
            "# Raised: <who, why>",
            "a.md: 2 names, inventory: 1",
            "b.md: 1 names, inventory: not listed",
            "The inventory the tree needs:\n# The header.\n\na.md 2\nb.md 1\nc.md 1\n",
        ] {
            assert!(failure.contains(text), "missing {text:?}: {failure}");
        }
        assert!(!failure.contains("c.md:4"), "{failure}");
    }
}

#[test]
fn the_needed_inventory_keeps_the_comments_and_takes_todays_counts() {
    for (source, counts, expected) in [
        (
            "# The header.\n\na.md 3\n# Raised: lead, a reason\nb.md 2\n# Raised: lead, another\nc.md 1\nd.md 1\n",
            vec![("a.md", 1), ("b.md", 4), ("bb.md", 2), ("d.md", 1), ("e.md", 5)],
            "# The header.\n\na.md 1\n# Raised: lead, a reason\nb.md 4\nbb.md 2\nd.md 1\ne.md 5\n",
        ),
        (
            "# header\n# Raised: first\n# Raised: second\nb.md 1\n# tail\n",
            vec![("a.md", 1), ("b.md", 2), ("c.md", 3)],
            "# header\na.md 1\n# Raised: first\n# Raised: second\nb.md 2\n# tail\nc.md 3\n",
        ),
        ("# Raised: kept\n\n# Raised: dropped\na.md 1\n# tail", vec![], "# Raised: kept\n\n# tail\n"),
        ("a.md 1\n", vec![("a.md", 0)], "\n"),
        ("", vec![("a.md", 1)], "a.md 1\n"),
    ] {
        let counts = counts.into_iter().map(|(path, count)| (path.to_owned(), count)).collect();
        assert_eq!(needed(source, &counts), expected, "{source:?}");
    }
}
