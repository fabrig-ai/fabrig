//! Only the kits can record checks (`skein/docs/design/testing/checks.md`, 1).

use std::{collections::BTreeSet, fs, process::Command};
use syn::visit::{self, Visit};

const KIT_FILES: &str = include_str!("check_counts.txt");

#[derive(Default)]
struct Tests {
    declares_test: bool,
}

impl<'ast> Visit<'ast> for Tests {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.declares_test |= item.attrs.iter().any(|attribute| attribute.path().is_ident("test"));
        visit::visit_item_fn(self, item);
    }
}

fn reference_line(source: &str) -> Option<usize> {
    source.contains("skein_checks").then(|| fabrig_boundary::identifier_line(source, "skein_checks")).flatten()
}

fn finding(path: &str, line: usize, source: &str, listed: bool) -> Option<String> {
    if !listed {
        return Some(format!("{path}:{line}: only listed kit files may use skein_checks"));
    }
    let syntax = syn::parse_file(source).unwrap_or_else(|error| panic!("{path}: {error}"));
    let mut tests = Tests::default();
    tests.visit_file(&syntax);
    tests.declares_test.then(|| format!("{path}:{line}: a kit file that counts cannot also declare a test"))
}

#[test]
fn only_listed_kit_files_use_the_counter() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z", "--", "*.rs"])
        .output()
        .expect("list tracked Rust sources");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let mut remaining: BTreeSet<_> = KIT_FILES.lines().collect();
    assert_eq!(remaining.len(), KIT_FILES.lines().count(), "each kit file is listed once");
    let mut findings = Vec::new();
    for path in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(path).expect("tracked path is UTF-8");
        let source = fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"));
        if let Some(line) = reference_line(&source)
            && let Some(finding) = finding(path, line, &source, remaining.remove(path))
        {
            findings.push(finding);
        }
    }
    for path in remaining {
        findings.push(format!("{path}: listed kit file has no counter use"));
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

#[test]
fn direct_and_imported_uses_are_reserved_to_helpers() {
    let path = "tree/tests/suite.rs";
    for source in [
        "#[test] fn story() { skein_checks :: count(\"seed\", 1); }",
        "use ::skein_checks::{count as record}; #[test] fn story() { record(\"seed\", 1); }",
        "extern crate skein_checks as checks; #[test] fn story() { checks::count(\"seed\", 1); }",
    ] {
        let line = reference_line(source).expect("the counter reference is found");
        assert_eq!(
            finding(path, line, source, false).as_deref(),
            Some("tree/tests/suite.rs:1: only listed kit files may use skein_checks")
        );
        assert!(finding(path, line, source, true).is_some(), "a listed file cannot count for its own test");
    }
    let helper = "fn run() { skein_checks::count(\"seed\", 1); }";
    assert_eq!(finding(path, reference_line(helper).expect("the counter reference is found"), helper, true), None);
    assert_eq!(reference_line("// skein_checks::count\nconst EXAMPLE: &str = \"skein_checks\";"), None);
}
