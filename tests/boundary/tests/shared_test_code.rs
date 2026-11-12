//! Files compiled by several test binaries of one package (tools/crates/design.md, 3.2).
//! Test binaries are exactly the targets with `test = true`; each tree reads by
//! section 2.1's rules. Exceptions cover a package's files at or below a path,
//! and fail when they cover no shared file. Unreadable sources always fail.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fabrig_boundary::{Package, module_tree, packages};
use serde_json::{Value, json};
use skein_scratch::Scratch;

struct Exception {
    package: &'static str,
    path: &'static str,
    until: &'static str,
}

const EXCEPTIONS: &[Exception] = &[
    Exception {
        package: "smith",
        path: "smith/tests/end-to-end/support",
        until: "G8: smith/testing/smith-end-to-end-kit",
    },
    Exception {
        package: "skein-browser-world",
        path: "skein/tests/browser/tests/support",
        until: "G8: skein/testing/skein-browser-rig",
    },
    Exception {
        package: "skein-browser-world",
        path: "skein/tests/browser/src/pages.rs",
        until: "G8: skein/testing/skein-browser-rig",
    },
    Exception {
        package: "jig-test",
        path: "jig/testing/jig-test/tests/support",
        until: "G8: jig/testing/jig-test-kit",
    },
    Exception {
        package: "jig-test",
        path: "jig/testing/jig-protocol-accounts-kit/src/visitor.rs",
        until: "G8: jig/testing/jig-protocol-accounts-kit, as a dependency",
    },
    Exception {
        package: "temper-real-loop-world",
        path: "temper/tests/real-loop/tests/support/mod.rs",
        until: "G8: the world's library's terminal module",
    },
    Exception {
        package: "temper-client-shell-terminal",
        path: "temper/crates/temper-client-shell-terminal/tests/support/world.rs",
        until: "its area, when it next works on its library's tests (tools/crates/design.md, 3.2)",
    },
    Exception {
        package: "temper-engine-domain-ways",
        path: "temper/crates/temper-engine-domain-ways/src/tests/invariants.rs",
        until: "its area, when it next works on its library's tests (tools/crates/design.md, 3.2)",
    },
];

struct Sharing {
    package: String,
    files: BTreeMap<PathBuf, Vec<String>>,
    failures: Vec<String>,
}

fn measure(workspace: &Path, package: &Package) -> Sharing {
    let mut sharing = Sharing { package: package.name.clone(), files: BTreeMap::new(), failures: Vec::new() };
    for target in package.targets.iter().filter(|target| target.test) {
        let tree = module_tree(&package.directory, &target.root);
        let binary = format!("{} {}", target.kinds.join("/"), target.name);
        sharing.failures.extend(tree.failures.into_iter().map(|why| format!("{binary}: {why}")));
        for file in tree.files.keys() {
            match file.strip_prefix(workspace) {
                Ok(path) => sharing.files.entry(path.to_path_buf()).or_default().push(binary.clone()),
                Err(why) => sharing.failures.push(format!("{binary}: {}: {why}", file.display())),
            }
        }
    }
    sharing.files.retain(|_, binaries| binaries.len() >= 2);
    sharing
}

fn covers(exception: &Exception, sharing: &Sharing, file: &Path) -> bool {
    exception.package == sharing.package && file.starts_with(exception.path)
}

fn findings(sharings: &[Sharing], exceptions: &[Exception]) -> Vec<String> {
    let mut failures = Vec::new();
    for sharing in sharings {
        failures.extend(sharing.failures.iter().map(|why| format!("{}: {why}", sharing.package)));
        for (file, binaries) in &sharing.files {
            if !exceptions.iter().any(|exception| covers(exception, sharing, file)) {
                failures.push(format!("{}: {}: {}", sharing.package, file.display(), binaries.join(", ")));
            }
        }
    }
    for exception in exceptions {
        if !sharings.iter().any(|sharing| sharing.files.keys().any(|file| covers(exception, sharing, file))) {
            failures.push(format!(
                "{}: {} (until {}): remove it: the check's exceptions only shrink",
                exception.package, exception.path, exception.until
            ));
        }
    }
    failures
}

#[test]
fn workspace_test_binaries_share_only_excepted_sources() {
    let workspace = skein_scratch::paths::workspace();
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(&workspace)
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let packages = packages(&String::from_utf8(output.stdout).expect("cargo metadata is UTF-8"))
        .expect("workspace metadata reads");
    let sharings: Vec<_> = packages.iter().map(|package| measure(&workspace, package)).collect();
    for exception in EXCEPTIONS {
        let count: usize = sharings
            .iter()
            .map(|sharing| sharing.files.keys().filter(|file| covers(exception, sharing, file)).count())
            .sum();
        eprintln!("{}: {}: {count} shared files", exception.package, exception.path);
    }
    let failures = findings(&sharings, EXCEPTIONS);
    assert!(failures.is_empty(), "Shared test code findings:\n{}", failures.join("\n"));
}

fn scratch(files: &[(&str, &str)]) -> Scratch {
    let scratch = Scratch::new("boundary-shared-test-code");
    for (name, source) in files {
        let path = scratch.path().join(name);
        fs::create_dir_all(path.parent().expect("fixture directory")).expect("create fixture directory");
        fs::write(path, source).expect("write fixture source");
    }
    scratch
}

fn target(scratch: &Scratch, kind: &str, name: &str, root: &str, test: bool) -> Value {
    json!({"kind": [kind], "name": name, "src_path": scratch.path().join(root), "test": test})
}

fn package(scratch: &Scratch, name: &str, directory: &str, targets: &[Value]) -> Value {
    json!({"id": name, "name": name, "manifest_path": scratch.path().join(directory).join("Cargo.toml"), "targets": targets})
}

fn scan(scratch: &Scratch, members: &[Value]) -> Vec<Sharing> {
    let metadata = json!({"workspace_members": members.iter().map(|package| &package["id"]).collect::<Vec<_>>(), "packages": members});
    packages(&metadata.to_string())
        .expect("fixture metadata")
        .iter()
        .map(|package| measure(scratch.path(), package))
        .collect()
}

#[test]
fn each_shared_file_names_its_package_and_all_binaries() {
    let scratch = scratch(&[
        ("demo/tests/a.rs", "#[path = \"support/mod.rs\"] mod support;"),
        ("demo/tests/b.rs", "#[path = \"support/mod.rs\"] mod support;"),
        ("demo/tests/c.rs", "#[path = \"support/helper.rs\"] mod helper;"),
        ("demo/tests/support/mod.rs", "mod helper;"),
        ("demo/tests/support/helper.rs", "// helper"),
    ]);
    let sharings = scan(
        &scratch,
        &[package(
            &scratch,
            "demo",
            "demo",
            &[
                target(&scratch, "test", "a", "demo/tests/a.rs", true),
                target(&scratch, "test", "b", "demo/tests/b.rs", true),
                target(&scratch, "bin", "c", "demo/tests/c.rs", true),
            ],
        )],
    );
    assert_eq!(
        findings(&sharings, &[]),
        [
            "demo: demo/tests/support/helper.rs: test a, test b, bin c",
            "demo: demo/tests/support/mod.rs: test a, test b",
        ]
    );
}

#[test]
fn test_flag_alone_selects_test_binaries() {
    let scratch = scratch(&[
        ("demo/src/lib.rs", "#[cfg(test)] #[path = \"../tests/support.rs\"] mod support;"),
        ("demo/tests/a.rs", "mod support;"),
        ("demo/tests/support.rs", "// shared"),
    ]);
    for kind in ["lib", "bin", "test", "bench", "example"] {
        for enabled in [false, true] {
            let sharings = scan(
                &scratch,
                &[package(
                    &scratch,
                    "demo",
                    "demo",
                    &[
                        target(&scratch, kind, "demo", "demo/src/lib.rs", enabled),
                        target(&scratch, "test", "a", "demo/tests/a.rs", true),
                    ],
                )],
            );
            let expected =
                if enabled { vec![format!("demo: demo/tests/support.rs: {kind} demo, test a")] } else { vec![] };
            assert_eq!(findings(&sharings, &[]), expected, "{kind}, test = {enabled}");
        }
    }
}

#[test]
fn repeated_paths_within_one_binary_are_one_file() {
    let scratch = scratch(&[
        ("tests/a.rs", "mod support; #[path = \"./support.rs\"] mod again; include!(\"support.rs\");"),
        ("tests/support.rs", "// one binary"),
    ]);
    let sharings =
        scan(&scratch, &[package(&scratch, "demo", "", &[target(&scratch, "test", "a", "tests/a.rs", true)])]);
    assert!(findings(&sharings, &[]).is_empty());
}

#[test]
fn files_reached_by_different_packages_are_not_shared_within_either() {
    let scratch = scratch(&[
        ("one/tests/a.rs", "#[path = \"../../two/tests/support.rs\"] mod support;"),
        ("two/tests/b.rs", "mod support;"),
        ("two/tests/support.rs", "// cross-package source"),
    ]);
    let sharings = scan(
        &scratch,
        &[
            package(&scratch, "one", "one", &[target(&scratch, "test", "a", "one/tests/a.rs", true)]),
            package(&scratch, "two", "two", &[target(&scratch, "test", "b", "two/tests/b.rs", true)]),
        ],
    );
    assert!(findings(&sharings, &[]).is_empty());
}

#[test]
fn exceptions_cover_only_the_named_packages_path_and_must_cover_a_finding() {
    let scratch = scratch(&[
        ("tests/a.rs", "mod support; mod support_extra;"),
        ("tests/b.rs", "mod support; mod support_extra;"),
        ("tests/support/mod.rs", "mod helper;"),
        ("tests/support/helper.rs", "// helper"),
        ("tests/support_extra.rs", "// outside directory"),
    ]);
    let sharings = scan(
        &scratch,
        &[package(
            &scratch,
            "demo",
            "",
            &[target(&scratch, "test", "a", "tests/a.rs", true), target(&scratch, "test", "b", "tests/b.rs", true)],
        )],
    );
    let exceptions = [
        Exception { package: "demo", path: "tests/support", until: "kit" },
        Exception { package: "demo", path: "tests/support_extra.rs", until: "move tests" },
    ];
    assert!(findings(&sharings, &exceptions).is_empty());
    assert_eq!(findings(&sharings, &exceptions[..1]), ["demo: tests/support_extra.rs: test a, test b"]);
    let wrong = [Exception { package: "other", path: "tests/support", until: "kit" }];
    assert_eq!(
        findings(&sharings, &wrong),
        [
            "demo: tests/support/helper.rs: test a, test b",
            "demo: tests/support/mod.rs: test a, test b",
            "demo: tests/support_extra.rs: test a, test b",
            "other: tests/support (until kit): remove it: the check's exceptions only shrink",
        ]
    );
    let single = scan(&scratch, &[package(&scratch, "demo", "", &[target(&scratch, "test", "a", "tests/a.rs", true)])]);
    assert_eq!(
        findings(&single, &exceptions),
        [
            "demo: tests/support (until kit): remove it: the check's exceptions only shrink",
            "demo: tests/support_extra.rs (until move tests): remove it: the check's exceptions only shrink",
        ]
    );
}

#[test]
fn source_failures_are_never_excepted_or_skipped() {
    let scratch = scratch(&[("tests/a.rs", "mod missing; include!(env!(\"OTHER\"));")]);
    let sharings =
        scan(&scratch, &[package(&scratch, "demo", "", &[target(&scratch, "test", "a", "tests/a.rs", true)])]);
    let failures = findings(&sharings, &[Exception { package: "demo", path: "tests", until: "kit" }]);
    assert_eq!(failures.len(), 3, "{failures:?}");
    assert!(failures.iter().any(|why| why.starts_with("demo: test a:") && why.contains("missing.rs")), "{failures:?}");
    assert!(
        failures.iter().any(|why| why.starts_with("demo: test a:") && why.ends_with("unsupported include! form")),
        "{failures:?}"
    );
    assert_eq!(failures[2], "demo: tests (until kit): remove it: the check's exceptions only shrink");
}
