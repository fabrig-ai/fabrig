//! Declared environment names (skein/docs/foundation/cli.md, 5 and 9, check 5).
//! Tracked UTF-8 files outside docs name only variables declared by their tree,
//! in the set their names say. Tokens ending `_PATCH_SET` are Rust patch-set
//! constants, not variables. Generated `patches/mod.rs` modules are not read;
//! authored patch functions and context are. Every declaration is read or set outside docs.
//! The document parser reads skein/docs/design/runtime/shell.md, 14.3's headings and
//! `**Set:**` items alone; the meeting test checks its rendered form.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use skein_shell::{Fallback, Set, Setting, Settings, Table, TestTable, Variable, Way};

const OWN: &str = "tests/boundary/tests/cli_names.rs";
const TREES: &[(&str, &str)] =
    &[("skein", "SKEIN"), ("smith", "SMITH"), ("jig", "JIG"), ("temper", "TEMPER"), ("hearth", "HEARTH")];

/// What a check excepts, and the round or milestone that removes it
/// (skein/docs/foundation/cli.md, 9).
struct Exception {
    what: &'static str,
    until: &'static str,
}

const EXCEPTIONS: &[Exception] = &[
    Exception { what: "HEARTH_BLESS", until: "hearth's round" },
    Exception { what: "HEARTH_RECORD", until: "hearth's round" },
    Exception { what: "HEARTH_TASKS", until: "hearth's round" },
    Exception { what: "JIG_APPLIED", until: "jig's round" },
    Exception { what: "JIG_CORE_SETTINGS_SEEDS", until: "jig's round" },
    Exception { what: "JIG_LEVEL", until: "jig's round" },
    Exception { what: "JIG_ONLY", until: "jig's round" },
    Exception { what: "JIG_SETTINGS_PROPOSALS_SEED", until: "jig's round" },
    Exception { what: "JIG_SOURCE_CHECK", until: "jig's round" },
    Exception { what: "JIG_STEP", until: "jig's round" },
    Exception { what: "JIG_TEST_BUNDLE", until: "jig's round" },
    Exception { what: "JIG_WORLD_TRACE", until: "jig's round" },
    Exception { what: "JIG_WRAPPER", until: "jig's round" },
    Exception { what: "SKEIN_CODEGEN_WRITE", until: "skein's round" },
    Exception { what: "SKEIN_DIFF_WRITE", until: "skein's round" },
    Exception { what: "SKEIN_FORGEJO", until: "skein's round" },
    Exception { what: "SKEIN_LISTENER_TEST", until: "skein's round" },
    Exception { what: "SKEIN_PATCH_WRITE", until: "skein's round" },
    Exception { what: "SKEIN_STARVED_PASS_CHILD", until: "skein's round" },
    Exception { what: "SKEIN_STARVED_REFEREE_CHILD", until: "skein's round" },
    Exception { what: "SKEIN_TEST_CHROMIUM", until: "skein's round" },
    Exception { what: "SKEIN_TEST_IGNORED_HANGUP", until: "skein's round" },
    Exception { what: "SKEIN_TEST_LIVE_ANTHROPIC_KEY", until: "skein's round" },
    Exception { what: "SKEIN_TEST_LIVE_ANTHROPIC_MODEL", until: "skein's round" },
    Exception { what: "SKEIN_TEST_LIVE_CA_FILE", until: "skein's round" },
    Exception { what: "SKEIN_TEST_LIVE_OPENAI_AUTH", until: "skein's round" },
    Exception { what: "SKEIN_TEST_LIVE_OPENAI_MODEL", until: "skein's round" },
    Exception { what: "SKEIN_UI_EXAMPLE_BUNDLE", until: "skein's round" },
    Exception { what: "SKEIN_UPDATE_GOLDENS", until: "skein's round" },
    Exception { what: "SKEIN_ZERO_PATIENCE_CHILD", until: "skein's round" },
    Exception { what: "SMITH_BENCH_NONCE", until: "smith's round" },
    Exception { what: "SMITH_BENCH_PAIR_REQUEST", until: "smith's round" },
    Exception { what: "SMITH_BENCH_PAIR_ROOT", until: "smith's round" },
    Exception { what: "SMITH_BOUNDS", until: "smith's round" },
    Exception { what: "SMITH_FIXTURE_TOKEN", until: "smith's round" },
    Exception { what: "SMITH_FIXTURE_WALL_MS", until: "smith's round" },
    Exception { what: "SMITH_HOST_ID_SERIAL", until: "smith's round" },
    Exception { what: "SMITH_HOST_ID_TEST", until: "smith's round" },
    Exception { what: "SMITH_LIVE", until: "smith's round" },
    Exception { what: "SMITH_LIVE_CLAUDE", until: "smith's round" },
    Exception { what: "SMITH_MCP_SECRET", until: "smith's round" },
    Exception { what: "SMITH_REPORT_TEST_KEY", until: "smith's round" },
    Exception { what: "SMITH_REPOSITORY_REPLAY_ROOT", until: "smith's round" },
    Exception { what: "SMITH_SMOKE_NONCE_012", until: "smith's round" },
    Exception { what: "SMITH_TEST_ENVIRONMENT", until: "smith's round" },
    Exception { what: "SMITH_TEST_LIVE_GIT_REMOTE", until: "smith's round" },
    Exception { what: "SMITH_UNSET_FIXTURE", until: "smith's round" },
];

/// The findings no exception names, and the exceptions that name no
/// finding, as one failure message; `None` when both are empty.
fn judged(check: &str, findings: &[String], exceptions: &[Exception]) -> Option<String> {
    let mut failures = Vec::new();
    for finding in findings {
        if !exceptions.iter().any(|exception| exception.what == finding) {
            failures.push(format!("{check}: {finding}"));
        }
    }
    for exception in exceptions {
        if !findings.iter().any(|finding| finding == exception.what) {
            failures.push(format!(
                "{check}: {} (until {}): remove it: the check's exceptions only shrink",
                exception.what, exception.until
            ));
        }
    }
    (!failures.is_empty()).then(|| failures.join("\n"))
}

/// Only the heading and the first set line after it belong to the parser.
fn document_entries(text: &str) -> Result<Vec<(&str, &str)>, String> {
    let mut entries = Vec::new();
    let mut pending = None;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("### `").and_then(|name| name.strip_suffix('`')) {
            if let Some(name) = pending {
                return Err(format!("{name}: no set line"));
            }
            pending = Some(name);
        } else if let Some(set) = line.strip_prefix("- **Set:** ")
            && let Some(name) = pending.take()
        {
            if !["public", "internal", "test"].contains(&set) {
                return Err(format!("{name}: unknown set {set:?}"));
            }
            entries.push((name, set));
        }
    }
    if let Some(name) = pending {
        return Err(format!("{name}: no set line"));
    }
    Ok(entries)
}

fn word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Search the prefix with the library's byte search, then require maximal words.
fn words_with<'a>(text: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> {
    text.match_indices(prefix).filter_map(move |(start, _)| {
        if start > 0 && word_byte(text.as_bytes()[start - 1]) {
            return None;
        }
        let rest = &text[start..];
        let end = rest.bytes().take_while(|byte| word_byte(*byte)).count();
        Some(&rest[..end])
    })
}

fn named_tree<'a>(name: &str, trees: &[(&'a str, &str)]) -> Option<&'a str> {
    trees.iter().find_map(|(tree, prefix)| {
        let suffix = name.strip_prefix(prefix)?.strip_prefix('_')?;
        (!suffix.is_empty()
            && suffix.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'))
        .then_some(*tree)
    })
}

fn named_set(name: &str) -> &str {
    let suffix = name.split_once('_').map_or("", |(_, suffix)| suffix);
    if suffix.starts_with("INTERNAL_") {
        "internal"
    } else if suffix.starts_with("TEST_") {
        "test"
    } else {
        "public"
    }
}

struct Declaration {
    tree: String,
    name: String,
    set: String,
}

#[derive(Default)]
struct Names {
    declarations: Vec<Declaration>,
    tokens: BTreeSet<String>,
    used: BTreeSet<String>,
}

impl Names {
    fn document(&mut self, tree: &str, text: &str) -> Result<(), String> {
        for (name, set) in document_entries(text)? {
            self.declarations.push(Declaration { tree: tree.to_owned(), name: name.to_owned(), set: set.to_owned() });
        }
        Ok(())
    }

    fn file(&mut self, text: &str, trees: &[(&str, &str)], declared: &BTreeSet<&str>) {
        for (_, prefix) in trees {
            for word in words_with(text, prefix) {
                if named_tree(word, trees).is_some() && !word.ends_with("_PATCH_SET") {
                    self.tokens.insert(word.to_owned());
                    self.used.insert(word.to_owned());
                }
            }
        }
        for name in declared {
            if words_with(text, name).any(|word| word == *name) {
                self.used.insert((*name).to_owned());
            }
        }
    }

    fn findings(&self, trees: &[(&str, &str)]) -> Vec<String> {
        let mut findings = BTreeSet::new();
        let declared: BTreeSet<&str> = self.declarations.iter().map(|entry| entry.name.as_str()).collect();
        for token in &self.tokens {
            if !declared.contains(token.as_str()) {
                findings.insert(token.clone());
            }
        }
        for entry in &self.declarations {
            if let Some(tree) = named_tree(&entry.name, trees) {
                if tree != entry.tree {
                    findings.insert(format!("{}: declared under {}, named {tree}", entry.name, entry.tree));
                }
                let set = named_set(&entry.name);
                if set != entry.set {
                    findings.insert(format!("{}: declared {}, named {set}", entry.name, entry.set));
                }
            }
            if !self.used.contains(&entry.name) {
                findings.insert(format!("{}: declared, never read or set", entry.name));
            }
        }
        findings.into_iter().collect()
    }
}

fn document_tree(path: &str) -> Option<&str> {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        [tree, "docs", "design", "cli", file]
            if TREES.iter().any(|(name, _)| name == tree) && file.ends_with("-env.md") =>
        {
            Some(tree)
        }
        _ => None,
    }
}

/// Only the patch command's generated module, not authored patch sources.
fn generated_patch_module(path: &str) -> bool {
    Path::new(path).ends_with("patches/mod.rs")
}

struct Scan {
    names: Names,
    skipped: Vec<String>,
    read: usize,
}

/// Documents are read first, then sources once each, using git's tracked paths.
/// A symlink contributes its target's text, without following it outside the tree.
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
    let paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|raw| std::str::from_utf8(raw).map_err(|why| format!("git ls-files: path {raw:?}: {why}")))
        .collect::<Result<Vec<_>, _>>()?;
    let mut scan = Scan { names: Names::default(), skipped: Vec::new(), read: 0 };
    for path in &paths {
        if let Some(tree) = document_tree(path) {
            let text = fs::read_to_string(root.join(path)).map_err(|why| format!("{path}: {why}"))?;
            scan.names.document(tree, &text).map_err(|why| format!("{path}: {why}"))?;
        }
    }
    let declared: BTreeSet<String> = scan.names.declarations.iter().map(|entry| entry.name.clone()).collect();
    let declared = declared.iter().map(String::as_str).collect();
    for path in paths {
        if path == OWN || path.split('/').any(|part| part == "docs") || generated_patch_module(path) {
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
        match std::str::from_utf8(&bytes) {
            Ok(text) => {
                scan.read += 1;
                scan.names.file(text, TREES, &declared);
            }
            Err(_) => scan.skipped.push(path.to_owned()),
        }
    }
    Ok(scan)
}

#[test]
fn every_name_is_declared() {
    let scan = scan(&skein_scratch::paths::workspace()).expect("the workspace's names read");
    eprintln!("Read {} UTF-8 source files; skipped {} non-UTF-8 files.", scan.read, scan.skipped.len());
    let findings = scan.names.findings(TREES);
    if let Some(why) = judged("declared names", &findings, EXCEPTIONS) {
        panic!("{why}");
    }
}

#[test]
fn only_generated_patch_modules_are_skipped() {
    for (path, skipped) in [
        ("jig/crates/jig-protocol-store/patches/mod.rs", true),
        ("jig/crates/jig-protocol-store/patches/p0001.rs", false),
        ("jig/crates/jig-protocol-store/patches/context.rs", false),
        ("jig/crates/jig-protocol-store/src/mod.rs", false),
        ("jig/crates/jig-protocol-store/other-patches/mod.rs", false),
    ] {
        assert_eq!(generated_patch_module(path), skipped, "{path}");
    }
}

#[test]
fn patch_set_constants_are_not_environment_names_but_patch_settings_are() {
    let mut names = Names::default();
    names.file(
        "JIG_PATCH_SET SKEIN_PATCH_SET SMITH_PATCH_SET TEMPER_PATCH_SET HEARTH_PATCH_SET JIG_PATCH_SETTING",
        TREES,
        &BTreeSet::new(),
    );
    assert_eq!(names.findings(TREES), ["JIG_PATCH_SETTING"]);
}

#[test]
fn names_are_judged() {
    let trees = &[("demo", "DEMO"), ("other", "OTHER")];
    for (label, tree, document, files, expected) in [
        ("declared", "demo", "### `DEMO_CONFIG`\n- **Set:** public", "DEMO_CONFIG", vec![]),
        ("undeclared", "demo", "", "DEMO_CONFIG", vec!["DEMO_CONFIG"]),
        (
            "wrong set",
            "demo",
            "### `DEMO_CONFIG`\n- **Set:** internal",
            "DEMO_CONFIG",
            vec!["DEMO_CONFIG: declared internal, named public"],
        ),
        (
            "unread",
            "demo",
            "### `DEMO_CONFIG`\n- **Set:** public",
            "",
            vec!["DEMO_CONFIG: declared, never read or set"],
        ),
        (
            "wrong tree",
            "other",
            "### `DEMO_CONFIG`\n- **Set:** public",
            "DEMO_CONFIG",
            vec!["DEMO_CONFIG: declared under other, named demo"],
        ),
        ("internal", "demo", "### `DEMO_INTERNAL_DUMP`\n- **Set:** internal", "DEMO_INTERNAL_DUMP", vec![]),
        ("test", "demo", "### `DEMO_TEST_RATE`\n- **Set:** test", "DEMO_TEST_RATE", vec![]),
        ("maximal words", "demo", "", "xDEMO_CONFIG DEMO_CONFIGx DEMO_ 1DEMO_CONFIG DEMO_éDEMO_A", vec!["DEMO_A"]),
        ("digits and underscores", "demo", "", "(DEMO_2_X) DEMO__", vec!["DEMO_2_X", "DEMO__"]),
        (
            "first set only",
            "demo",
            "- **Set:** test\n### `DEMO_CONFIG`\ntext\n- **Set:** public\n- **Set:** internal",
            "DEMO_CONFIG",
            vec![],
        ),
        ("host declaration", "demo", "### `HOST_HOME`\n- **Set:** public", "HOST_HOME", vec![]),
        (
            "host unread",
            "demo",
            "### `HOST_HOME`\n- **Set:** public",
            "",
            vec!["HOST_HOME: declared, never read or set"],
        ),
        ("duplicate use", "demo", "", "DEMO_CONFIG DEMO_CONFIG", vec!["DEMO_CONFIG"]),
    ] {
        let mut names = Names::default();
        names.document(tree, document).expect("the row's document parses");
        let declared: BTreeSet<String> = names.declarations.iter().map(|entry| entry.name.clone()).collect();
        names.file(files, trees, &declared.iter().map(String::as_str).collect());
        assert_eq!(names.findings(trees), expected, "{label}");
    }
}

const DIRECTORIES: Settings = Settings {
    tool: "demo",
    about: "The deployment's directory.",
    table: &[Setting {
        name: "config",
        value: "DIR",
        about: "The configuration directory.",
        fallback: Fallback::Text("config"),
        set: Set::Public,
        twin: Some("DEMO_CONFIG"),
        notes: "",
    }],
};

const JOB: Variable = Variable {
    name: "DEMO_JOB",
    set: Set::Public,
    form: "ID",
    line: "The job passed to child programs.",
    absent: "There is no job.",
    notes: "",
    way: Way::Sets,
};

const DUMP: Variable = Variable { name: "DEMO_INTERNAL_DUMP", set: Set::Internal, way: Way::Reads, ..JOB };
const TEST_RATE: Variable = Variable { name: "DEMO_TEST_RATE", set: Set::Test, way: Way::Reads, ..JOB };

const BOX: Table = Table {
    binary: "demo-box",
    line: "A demonstration box.",
    version: "0.1.0",
    commit: "0123456789ab",
    commands: &[],
    settings: &[&DIRECTORIES],
    variables: &[&JOB, &DUMP],
    more: "",
};
const ENGINE: Table = Table { binary: "demo-engine", variables: &[], ..BOX };
const TESTS: TestTable = TestTable { tree: "demo", variables: &[TEST_RATE] };

#[test]
fn rendered_documents_read_back() {
    let tree = &[&BOX, &ENGINE];
    for (label, document, expected) in [
        ("box", BOX.environment_document(tree), vec![("DEMO_JOB", "public"), ("DEMO_INTERNAL_DUMP", "internal")]),
        ("engine", ENGINE.environment_document(tree), vec![]),
        ("common", Table::common_document("demo", tree), vec![("DEMO_CONFIG", "public")]),
        ("test", TESTS.document(), vec![("DEMO_TEST_RATE", "test")]),
    ] {
        let entries = document_entries(&document).unwrap_or_else(|why| panic!("{label}: {why}"));
        assert_eq!(entries, expected, "{label}: {document}");
    }
}
