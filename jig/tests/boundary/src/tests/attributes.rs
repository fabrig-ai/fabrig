use std::{
    fs,
    path::{Path, PathBuf},
};

fn source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("source directory is readable") {
        let entry = entry.expect("directory entry is readable");
        let path = entry.path();
        if entry.file_type().expect("entry has a file type").is_dir() {
            if entry.file_name() != "target" && entry.file_name() != "generated" {
                source_files(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn breaches(path: &Path, source: &str) -> Vec<String> {
    let patterns = [["#", "![allow"].concat(), ["#", "[allow"].concat()];
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| patterns.iter().any(|pattern| line.contains(pattern)))
        .map(|(line, _)| format!("{}:{}: lint exception must use expect", path.display(), line + 1))
        .collect()
}

#[test]
fn kit_lint_exceptions_use_expect() {
    let root = super::kit_root();
    let mut files = Vec::new();
    source_files(&root, &mut files);
    files.sort();
    let failures: Vec<_> = files
        .iter()
        .flat_map(|path| {
            let source = fs::read_to_string(path).expect("source is readable");
            breaches(path.strip_prefix(&root).expect("source is under kit root"), &source)
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn attribute_fixture_reports_inner_and_outer_exceptions() {
    let path = Path::new("fixture/source.rs");
    let source = ["// fixture\n#", "![allow(dead_code)]\n#", "[allow(unused)]\n"].concat();
    assert_eq!(
        breaches(path, &source),
        ["fixture/source.rs:2: lint exception must use expect", "fixture/source.rs:3: lint exception must use expect",]
    );
    assert!(breaches(path, "#![expect(dead_code, reason = \"fixture\")]\n").is_empty());
}
