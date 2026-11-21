//! Generated records hold no tests, and codec libraries build no test target
//! (tools/crates/design.md, 2.2, 2.4 and 3.4).
//! Exceptions name one record file or the library target, never source-reading
//! failures. An exception that covers no finding fails the check too.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fabrig_boundary::{Package, has_test_code, packages, record_source};
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

fn inspect(package: &Package, check: &mut Check) {
    let mut has_records = false;
    for file in directory(package, "src/generated", check) {
        match record_source(&file) {
            Ok(Some(source)) => {
                has_records = true;
                let name = file.file_name().expect("record filename");
                let relative = format!("src/generated/{}", name.to_string_lossy());
                if has_test_code(&source) {
                    check.finding(
                        package,
                        &relative,
                        format!("{}: {relative}: test module in generated record code", package.name),
                    );
                }
            }
            Ok(None) => (),
            Err(why) => check.unreadable(package, &file, &why),
        }
    }
    if !has_records {
        return;
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

fn scan(metadata: &str) -> Result<Check, String> {
    let mut check = Check::default();
    for package in packages(metadata)? {
        inspect(&package, &mut check);
    }
    Ok(check)
}

fn findings(metadata: &str, exceptions: &[Exception]) -> Result<Vec<String>, String> {
    Ok(scan(metadata)?.finish(exceptions))
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
    let check = scan(&metadata).expect("workspace metadata reads");
    for finding in check.failures.iter().chain(check.rules.values().flatten()) {
        eprintln!("{finding}");
    }
    let failures = check.finish(EXCEPTIONS);
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
                {"name": "a", "kind": ["lib"], "src_path": scratch.path().join("src/lib.rs"), "test": false}
            ]
        }]
    })
}

fn record_fixture(source: &str) -> Scratch {
    scratch(&[(
        "src/generated/people.rs",
        &format!("// Generated by skein-codegen from people 2. Do not edit.\n{source}"),
    )])
}

#[test]
fn test_tokens_in_record_code_name_package_and_file() {
    for source in [
        "#[cfg(test)] mod tests {}",
        "#[test] fn check() {}",
        "#[cfg /* gap */ ( test )] mod tests {}",
        "# [ test ] fn check() {}",
    ] {
        let scratch = record_fixture(source);
        assert_eq!(
            findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata"),
            ["a: src/generated/people.rs: test module in generated record code"],
            "{source}"
        );
    }
}

#[test]
fn comments_and_literals_hold_no_test_code() {
    let scratch = record_fixture(
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
fn library_test_flag_is_checked_for_every_library_kind() {
    let scratch = record_fixture("");
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
fn only_record_headers_select_packages() {
    for (file, header, is_record) in [
        ("people.rs", "// Generated by skein-codegen from people 2. Do not edit.\n", true),
        ("layouts.rs", "// glue\n", false),
        ("people.rs", "// Generated by skein-codegen from people 2 and its maps. Do not edit.\n", false),
        ("people.rs", "// glue\n", false),
        ("people.txt", "// Generated by skein-codegen from people 2. Do not edit.\n", false),
    ] {
        let scratch = scratch(&[(&format!("src/generated/{file}"), &format!("{header}#[cfg(test)] mod tests {{}}"))]);
        let failures = findings(&fixture_metadata(&scratch).to_string(), &[]).expect("fixture metadata");
        let expected = if is_record {
            vec![format!("a: src/generated/{file}: test module in generated record code")]
        } else {
            vec![]
        };
        assert_eq!(failures, expected, "{file}: {header}");
    }
    let scratch = scratch(&[("src/generated/layouts.rs", "#[test] fn glue_test() {}")]);
    fs::create_dir(scratch.path().join("src/generated/people.rs")).expect("record-shaped directory");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"] = serde_json::json!([]);
    assert_eq!(findings(&metadata.to_string(), &[]).expect("fixture metadata"), Vec::<String>::new());
}

#[test]
fn exceptions_cover_only_the_named_file_or_library_and_must_match() {
    let scratch = record_fixture("#[cfg(test)] mod tests {}");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"][0]["test"] = serde_json::json!(true);
    let file = older("a", "src/generated/people.rs");
    let lib = library("a", "fixture private tests");
    assert_eq!(
        findings(&metadata.to_string(), &[file]).expect("fixture metadata"),
        ["a: library test target builds generated record code"]
    );
    assert_eq!(
        findings(&metadata.to_string(), &[lib]).expect("fixture metadata"),
        ["a: src/generated/people.rs: test module in generated record code"]
    );
    assert_eq!(
        findings(
            &metadata.to_string(),
            &[older("a", "src/generated/people.rs"), library("a", "fixture private tests")]
        )
        .expect("fixture metadata"),
        Vec::<String>::new()
    );
    let scratch = record_fixture("");
    let failures = findings(
        &fixture_metadata(&scratch).to_string(),
        &[older("a", "src/generated/people.rs"), library("gone", "fixture private tests")],
    )
    .expect("fixture metadata");
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].starts_with("a: src/generated/people.rs") && failures[0].contains("exceptions only shrink"),
        "{failures:?}"
    );
    assert!(
        failures[1].starts_with("gone: library target") && failures[1].contains("exceptions only shrink"),
        "{failures:?}"
    );
}

#[test]
fn unreadable_directories_and_files_are_never_excused() {
    let not_directory = scratch(&[("src/generated", "not a directory")]);
    let failures = findings(&fixture_metadata(&not_directory).to_string(), &[]).expect("fixture metadata");
    assert!(
        failures.iter().any(|failure| failure.starts_with("a: ")
            && failure.contains("src/generated")
            && failure.contains("Not a directory")),
        "{failures:?}"
    );
    let non_utf8 = record_fixture("#[cfg(test)] mod tests {}");
    fs::write(non_utf8.path().join("src/generated/people.rs"), [0xff]).expect("non-UTF-8 source");
    let failures = findings(&fixture_metadata(&non_utf8).to_string(), &[older("a", "src/generated/people.rs")])
        .expect("fixture metadata");
    assert!(
        failures.iter().any(|failure| failure.starts_with("a: ")
            && failure.contains("src/generated/people.rs")
            && failure.contains("UTF-8")),
        "{failures:?}"
    );
}

#[test]
fn unreadable_metadata_fails_before_scanning() {
    assert!(findings("not JSON", &[]).expect_err("bad metadata").contains("metadata"));
    let scratch = record_fixture("");
    let mut metadata = fixture_metadata(&scratch);
    metadata["packages"][0]["targets"][0]["test"] = serde_json::Value::Null;
    assert_eq!(findings(&metadata.to_string(), &[]).expect_err("missing flag"), "a: no target test flag");
}
