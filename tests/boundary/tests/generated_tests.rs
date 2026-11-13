//! Generated record tests compile apart (tools/crates/design.md, 2.2, 2.4 and 3.4).
//! Exceptions name one record file or the library target, never source-reading
//! failures. An exception that covers no finding fails the check too.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fabrig_boundary::{Package, has_test_code, packages};
use skein_scratch::Scratch;

struct Exception {
    package: &'static str,
    what: &'static str,
    reason: &'static str,
    until: &'static str,
}

const fn library(package: &'static str, reason: &'static str) -> Exception {
    Exception {
        package,
        what: "library target",
        reason,
        until: "its library's tests needing private items move or use public items alone",
    }
}

const fn older(package: &'static str, file: &'static str) -> Exception {
    Exception {
        package,
        what: file,
        reason: "an older version's record file, keeping its test module until C2 removes it",
        until: "milestone C2 of temper/docs/design/draft/gate-compile.execution.md removes the older version",
    }
}

const EXCEPTIONS: &[Exception] = &[
    library("jig-client-protocol-wire", "private wire and public-message helpers have library tests"),
    library("jig-protocol-worker-wire", "private wire, translation and spoken helpers have library tests"),
    library("temper-client-protocol-wire", "private wire and public-message helpers have library tests"),
    library("temper-protocol-worker-wire", "private wire and record helpers have library tests"),
    library("smith-charter", "private record helpers have library tests"),
    library("smith-channel", "private kind and record helpers have library tests"),
    library("smith-transcript", "private record helpers have library tests"),
    library("temper-protocol-records", "patch golden tests remain in the library"),
    library("skein-codec-tests", "patch golden tests remain in the library"),
    older("temper-protocol-store-codec", "src/generated/v16.rs"),
    older("temper-protocol-store-codec", "src/generated/v17.rs"),
    older("temper-protocol-store-codec", "src/generated/v18.rs"),
    older("temper-protocol-store-codec", "src/generated/v19.rs"),
    older("jig-protocol-store-people", "src/generated/people_v16.rs"),
    older("smith-transcript", "src/generated/v6.rs"),
    older("smith-channel", "src/generated/v7.rs"),
    older("jig-protocol-worker-wire", "src/generated/v12.rs"),
];

#[derive(Default)]
struct Check {
    rules: BTreeMap<(String, String), Vec<String>>,
    failures: Vec<String>,
}

impl Check {
    fn finding(&mut self, package: &Package, what: &str, message: String) {
        self.rules.entry((package.name.clone(), what.to_owned())).or_default().push(message);
    }

    fn unreadable(&mut self, package: &Package, file: &Path, why: &std::io::Error) {
        self.failures.push(format!("{}: {}: {why}", package.name, file.display()));
    }

    fn finish(mut self, exceptions: &[Exception]) -> Vec<String> {
        for exception in exceptions {
            if self.rules.remove(&(exception.package.to_owned(), exception.what.to_owned())).is_none() {
                self.failures.push(format!(
                    "{}: {} ({}; until {}): remove it: the check's exceptions only shrink",
                    exception.package, exception.what, exception.reason, exception.until
                ));
            }
        }
        self.failures.extend(self.rules.into_values().flatten());
        self.failures
    }
}

/// An absent directory is empty; any other failure, including a dangling
/// symlink or a file in place of the directory, must not hide record code.
fn directory(package: &Package, relative: &str, check: &mut Check) -> Vec<PathBuf> {
    let path = package.directory.join(relative);
    match fs::symlink_metadata(&path) {
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(why) => {
            check.unreadable(package, &path, &why);
            return Vec::new();
        }
        Ok(_) => (),
    }
    let mut files = Vec::new();
    match fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => files.push(entry.path()),
                    Err(why) => check.unreadable(package, &path, &why),
                }
            }
        }
        Err(why) => check.unreadable(package, &path, &why),
    }
    files.sort();
    files
}

/// The filename rule of tools/crates/design.md, 2.2, not generated glue.
fn record(path: &Path) -> bool {
    let Some(stem) = path.file_name().and_then(|name| name.to_str()).and_then(|name| name.strip_suffix(".rs")) else {
        return false;
    };
    let version =
        if let Some(version) = stem.strip_prefix('v').filter(|version| version.bytes().all(|b| b.is_ascii_digit())) {
            version
        } else if let Some((family, version)) = stem.rsplit_once("_v") {
            if family.is_empty()
                || !family.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            {
                return false;
            }
            version
        } else {
            return false;
        };
    !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
}

fn inspect(package: &Package, check: &mut Check) {
    let records: Vec<_> = directory(package, "src/generated", check)
        .into_iter()
        .filter(|file| {
            if !record(file) {
                return false;
            }
            match fs::metadata(file) {
                Ok(metadata) => metadata.is_file(),
                Err(why) => {
                    check.unreadable(package, file, &why);
                    false
                }
            }
        })
        .collect();
    if records.is_empty() {
        return;
    }
    let tests = directory(package, "tests/generated", check);
    let root = package.directory.join("tests/generated/main.rs");
    let target = package.targets.iter().any(|target| {
        target.name == "generated"
            && target.test
            && target.kinds.iter().any(|kind| kind == "test")
            && target.root == root
    });
    if target {
        if let Err(why) = fs::read_to_string(&root) {
            check.unreadable(package, &root, &why);
        }
    } else {
        check.finding(
            package,
            "generated target",
            format!("{}: missing generated test target at tests/generated/main.rs", package.name),
        );
    }
    for file in records {
        let name = file.file_name().expect("record filename");
        let relative = format!("src/generated/{}", name.to_string_lossy());
        match fs::read_to_string(&file) {
            Ok(source) => {
                if has_test_code(&source) {
                    check.finding(
                        package,
                        &relative,
                        format!("{}: {relative}: test module in generated record code", package.name),
                    );
                }
            }
            Err(why) => check.unreadable(package, &file, &why),
        }
        let test = package.directory.join("tests/generated").join(name);
        if tests.contains(&test) {
            if let Err(why) = fs::read_to_string(&test) {
                check.unreadable(package, &test, &why);
            }
        } else {
            check.finding(
                package,
                &relative,
                format!("{}: missing tests/generated/{}", package.name, name.to_string_lossy()),
            );
        }
    }
    if package.targets.iter().any(|target| {
        target.test
            && target
                .kinds
                .iter()
                .any(|kind| ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"].contains(&kind.as_str()))
    }) {
        check.finding(
            package,
            "library target",
            format!("{}: library test target builds generated record code", package.name),
        );
    }
}

fn findings(metadata: &str, exceptions: &[Exception]) -> Result<Vec<String>, String> {
    let mut check = Check::default();
    for package in packages(metadata)? {
        inspect(&package, &mut check);
    }
    Ok(check.finish(exceptions))
}

#[test]
fn workspace_generated_tests_compile_apart() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let metadata = String::from_utf8(output.stdout).expect("cargo metadata is UTF-8");
    for finding in findings(&metadata, &[]).expect("workspace metadata reads") {
        eprintln!("{finding}");
    }
    let failures = findings(&metadata, EXCEPTIONS).expect("workspace metadata reads");
    assert!(failures.is_empty(), "Generated-test findings:\n{}", failures.join("\n"));
}

fn scratch(files: &[(&str, &str)]) -> Scratch {
    let scratch = Scratch::new("boundary-generated-tests");
    for (name, source) in files {
        let path = scratch.path().join(name);
        fs::create_dir_all(path.parent().expect("fixture directory")).expect("create fixture directory");
        fs::write(path, source).expect("write fixture source");
    }
    scratch
}

fn fixture_metadata(scratch: &Scratch) -> serde_json::Value {
    serde_json::json!({
        "workspace_members": ["a"],
        "packages": [{
            "id": "a", "name": "a", "manifest_path": scratch.path().join("Cargo.toml"),
            "targets": [
                {"name": "a", "kind": ["lib"], "src_path": scratch.path().join("src/lib.rs"), "test": false},
                {"name": "generated", "kind": ["test"], "src_path": scratch.path().join("tests/generated/main.rs"), "test": true}
            ]
        }]
    })
}

fn complete(source: &str) -> Scratch {
    scratch(&[("src/generated/v2.rs", source), ("tests/generated/main.rs", "mod v2;"), ("tests/generated/v2.rs", "")])
}

#[test]
fn test_tokens_in_record_code_name_package_and_file() {
    for source in [
        "#[cfg(test)] mod tests {}",
        "#[test] fn check() {}",
        "#[cfg /* gap */ ( test )] mod tests {}",
        "# [ test ] fn check() {}",
    ] {
        let scratch = complete(source);
        assert_eq!(
            findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata"),
            ["a: src/generated/v2.rs: test module in generated record code"],
            "{source}"
        );
    }
}

#[test]
fn comments_and_literals_hold_no_test_code() {
    let scratch = complete(
        r####"
        // #[cfg(test)] #[test]
        /* #[cfg(test)] /* nested */ #[test] */
        const TEXT: &str = "#[cfg(test)] #[test] \\\"";
        const RAW: &str = r###"#[cfg(test)] #[test]"###;
        const BYTES: &[u8] = b"#[cfg(test)] #[test]";
        const RAW_BYTES: &[u8] = br#"#[cfg(test)] #[test]"#;
        const C: &CStr = c"#[cfg(test)] #[test]";
        const RAW_C: &CStr = cr#"#[cfg(test)] #[test]"#;
        const CHARACTER: char = '\'';
        const UNICODE: char = '\u{1234}';
        const BYTE: u8 = b'\x23';
        fn lifetime<'a>() {}
        #[cfg(testing)] fn not_a_test() {}
        #[test_case] fn not_a_test_either() {}
    "####,
    );
    assert_eq!(findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata"), Vec::<String>::new());
}

#[test]
fn generated_target_requires_name_kind_test_flag_and_root() {
    let scratch = complete("");
    let metadata = fixture_metadata(&scratch);
    assert_eq!(findings(&metadata.to_string(), &[]).expect("fixture metadata"), Vec::<String>::new());
    for (field, value) in [
        ("name", serde_json::json!("other")),
        ("kind", serde_json::json!(["bin"])),
        ("test", serde_json::json!(false)),
        ("src_path", serde_json::json!(scratch.path().join("tests/other.rs"))),
    ] {
        let mut metadata = metadata.clone();
        metadata["packages"][0]["targets"][1][field] = value;
        assert_eq!(
            findings(&metadata.to_string(), &[]).expect("fixture metadata"),
            ["a: missing generated test target at tests/generated/main.rs"],
            "{field}"
        );
    }
    let mut metadata = metadata;
    metadata["packages"][0]["targets"].as_array_mut().expect("targets").pop();
    assert_eq!(
        findings(&metadata.to_string(), &[]).expect("fixture metadata"),
        ["a: missing generated test target at tests/generated/main.rs"]
    );
}

#[test]
fn every_record_file_needs_a_test_file() {
    let scratch = scratch(&[
        ("src/generated/v2.rs", ""),
        ("src/generated/people_v3.rs", ""),
        ("tests/generated/main.rs", "mod v2;"),
        ("tests/generated/v2.rs", ""),
    ]);
    let metadata = fixture_metadata(&scratch).to_string();
    assert_eq!(findings(&metadata, &[]).expect("fixture metadata"), ["a: missing tests/generated/people_v3.rs"]);
    fs::remove_file(scratch.path().join("tests/generated/v2.rs")).expect("remove test file");
    assert_eq!(
        findings(&metadata, &[]).expect("fixture metadata"),
        ["a: missing tests/generated/people_v3.rs", "a: missing tests/generated/v2.rs"]
    );
}

#[test]
fn library_test_flag_is_checked_for_every_library_kind() {
    let scratch = complete("");
    for kind in ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro", "bin"] {
        let mut metadata = fixture_metadata(&scratch);
        metadata["packages"][0]["targets"][0]["kind"] = serde_json::json!([kind]);
        assert_eq!(findings(&metadata.to_string(), &[]).expect("fixture metadata"), Vec::<String>::new());
        metadata["packages"][0]["targets"][0]["test"] = serde_json::json!(true);
        let expected = if kind == "bin" { vec![] } else { vec!["a: library test target builds generated record code"] };
        assert_eq!(findings(&metadata.to_string(), &[]).expect("fixture metadata"), expected, "{kind}");
    }
}

#[test]
fn only_record_filenames_select_packages() {
    for (file, is_record) in [
        ("v12.rs", true),
        ("team_2_v03.rs", true),
        ("layouts.rs", false),
        ("People_v1.rs", false),
        ("people_v.rs", false),
        ("people_v1.txt", false),
        ("v.rs", false),
        ("_v1.rs", false),
    ] {
        let scratch = scratch(&[
            (&format!("src/generated/{file}"), "#[cfg(test)] mod tests {}"),
            ("tests/generated/main.rs", ""),
            (&format!("tests/generated/{file}"), ""),
        ]);
        let failures = findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata");
        let expected = if is_record {
            vec![format!("a: src/generated/{file}: test module in generated record code")]
        } else {
            vec![]
        };
        assert_eq!(failures, expected, "{file}");
    }
    let scratch = scratch(&[("src/generated/layouts.rs", "#[test] fn glue_test() {}")]);
    fs::create_dir(scratch.path().join("src/generated/v2.rs")).expect("record-shaped directory");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"] = serde_json::json!([]);
    assert_eq!(findings(&metadata.to_string(), &[]).expect("fixture metadata"), Vec::<String>::new());
}

#[test]
fn exceptions_cover_only_the_named_file_or_library_and_must_match() {
    let scratch = complete("#[cfg(test)] mod tests {}");
    fs::remove_file(scratch.path().join("tests/generated/v2.rs")).expect("remove test file");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"][0]["test"] = serde_json::json!(true);
    let file = older("a", "src/generated/v2.rs");
    let lib = library("a", "fixture private tests");
    assert_eq!(
        findings(&metadata.to_string(), &[file]).expect("fixture metadata"),
        ["a: library test target builds generated record code"]
    );
    assert_eq!(
        findings(&metadata.to_string(), &[lib]).expect("fixture metadata"),
        ["a: src/generated/v2.rs: test module in generated record code", "a: missing tests/generated/v2.rs"]
    );
    assert_eq!(
        findings(&metadata.to_string(), &[older("a", "src/generated/v2.rs"), library("a", "fixture private tests")])
            .expect("fixture metadata"),
        Vec::<String>::new()
    );
    let scratch = complete("");
    let failures = findings(
        &fixture_metadata(&scratch).to_string(),
        &[older("a", "src/generated/v2.rs"), library("gone", "fixture private tests")],
    )
    .expect("fixture metadata");
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].starts_with("a: src/generated/v2.rs") && failures[0].contains("exceptions only shrink"),
        "{failures:?}"
    );
    assert!(
        failures[1].starts_with("gone: library target") && failures[1].contains("exceptions only shrink"),
        "{failures:?}"
    );
}

#[test]
fn unreadable_directories_and_files_are_never_excused() {
    for relative in ["src/generated", "tests/generated"] {
        let scratch = if relative == "src/generated" {
            scratch(&[(relative, "not a directory")])
        } else {
            scratch(&[("src/generated/v2.rs", ""), (relative, "not a directory")])
        };
        let failures = findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata");
        assert!(
            failures.iter().any(|failure| failure.starts_with("a: ")
                && failure.contains(relative)
                && failure.contains("Not a directory")),
            "{failures:?}"
        );
    }
    for relative in ["src/generated/v2.rs", "tests/generated/v2.rs", "tests/generated/main.rs"] {
        let scratch = complete("#[cfg(test)] mod tests {}");
        fs::write(scratch.path().join(relative), [0xff]).expect("non-UTF-8 source");
        let failures = findings(&fixture_metadata(&scratch).to_string(), &[older("a", "src/generated/v2.rs")])
            .expect("fixture metadata");
        assert!(
            failures
                .iter()
                .any(|failure| failure.starts_with("a: ") && failure.contains(relative) && failure.contains("UTF-8")),
            "{failures:?}"
        );
    }
}

#[test]
fn unreadable_metadata_fails_before_scanning() {
    assert!(findings("not JSON", &[]).expect_err("bad metadata").contains("metadata"));
    let scratch = complete("");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"][0]["test"] = serde_json::Value::Null;
    assert_eq!(findings(&metadata.to_string(), &[]).expect_err("missing flag"), "a: no target test flag");
}
