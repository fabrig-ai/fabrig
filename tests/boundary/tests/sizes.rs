//! Library and generated-family sizes (tools/crates/design.md, 2 and 3.1).
//! Cargo metadata selects workspace libraries. A lexical module-tree scan reads
//! their sources, including cfg(test), path attributes and source includes.
//! Libraries outside generated record code are bounded at 60,000 lines; families
//! sharing a package are bounded at 20,000, over every kept version. Lines include
//! blanks and comments. Exceptions only shrink: an unmatched exception fails too.
//! Metadata or a source the check cannot read, and unsupported includes, fail it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fabrig_boundary::{Tree, module_tree};
use serde_json::Value;
use skein_scratch::Scratch;

struct Exception {
    what: &'static str,
    until: &'static str,
}

const EXCEPTIONS: &[Exception] = &[
    Exception {
        what: "temper-client-view",
        until: "its split, not yet planned (temper/docs/design/draft/crate-graph.md, 5.1)",
    },
    Exception {
        what: "jig-domain-core",
        until: "Core's state grouped into owned parts (jig/docs/development/debt/D003-core-one-object.md), then a split along them",
    },
    Exception { what: "temper-engine-service", until: "G4" },
    Exception {
        what: "temper-client-domain",
        until: "its split, not yet planned (temper/docs/design/draft/crate-graph.md, 5.1)",
    },
    Exception { what: "temper-engine-domain-forge", until: "G5" },
    Exception {
        what: "jig-protocol-store",
        until: "its split, not yet planned (temper/docs/design/draft/crate-graph.md, 5.1); G2 moves only generated record code",
    },
    Exception {
        what: "temper-client-tools",
        until: "its split, not yet planned (temper/docs/design/draft/crate-graph.md, 5.1)",
    },
    Exception {
        what: "jig-client-view",
        until: "its split, not yet planned (temper/docs/design/draft/crate-graph.md, 5.1)",
    },
    Exception {
        what: "jig-client-protocol-wire (own)",
        until: "a crate of its own for public (tools/crates/design.md, 5)",
    },
    Exception {
        what: "temper-client-protocol-wire (own)",
        until: "a crate of its own for public (tools/crates/design.md, 5)",
    },
];

#[derive(Debug)]
struct Package {
    name: String,
    directory: PathBuf,
    root: PathBuf,
}

fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let mut libraries = Vec::new();
    for package in fabrig_boundary::packages(metadata)? {
        for target in package.targets {
            if target
                .kinds
                .iter()
                .any(|kind| ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"].contains(&kind.as_str()))
            {
                libraries.push(Package {
                    name: package.name.clone(),
                    directory: package.directory.clone(),
                    root: target.root,
                });
            }
        }
    }
    Ok(libraries)
}

fn record_family(path: &Path) -> Option<&str> {
    let stem = path.file_name()?.to_str()?.strip_suffix(".rs")?;
    let (family, version) = if let Some(version) = stem
        .strip_prefix('v')
        .filter(|version| !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit()))
    {
        ("(own)", version)
    } else {
        let (family, version) = stem.rsplit_once("_v")?;
        if family.is_empty()
            || !family.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return None;
        }
        (family, version)
    };
    (!version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())).then_some(family)
}

#[derive(Debug)]
struct Size {
    name: String,
    library: usize,
    families: BTreeMap<String, usize>,
    failures: Vec<String>,
}

fn measure(package: &Package) -> Size {
    let tree = module_tree(&package.directory, &package.root);
    let generated = package.directory.join("src/generated");
    let library = tree
        .files
        .iter()
        .filter(|(path, _)| !path.starts_with(&generated) || record_family(path).is_none())
        .map(|(_, lines)| lines)
        .sum();
    let mut size = Size { name: package.name.clone(), library, families: BTreeMap::new(), failures: tree.failures };
    match fs::read_dir(&generated) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if let Some(family) = record_family(&path) {
                            let lines = tree
                                .files
                                .get(&path)
                                .copied()
                                .map_or_else(|| fs::read_to_string(&path).map(|source| source.lines().count()), Ok);
                            match lines {
                                Ok(lines) => *size.families.entry(family.to_owned()).or_default() += lines,
                                Err(why) => size.failures.push(format!("{}: {why}", path.display())),
                            }
                        }
                    }
                    Err(why) => size.failures.push(format!("{}: {why}", generated.display())),
                }
            }
        }
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
        Err(why) => size.failures.push(format!("{}: {why}", generated.display())),
    }
    size
}

fn findings(sizes: &[Size], library_bound: usize, family_bound: usize, exceptions: &[Exception]) -> Vec<String> {
    let mut over = BTreeMap::new();
    let mut failures = Vec::new();
    for size in sizes {
        failures.extend(size.failures.iter().map(|why| format!("{}: {why}", size.name)));
        if size.library > library_bound {
            over.insert(size.name.clone(), format!("{}: {} lines, bound {library_bound}", size.name, size.library));
        }
        if size.families.len() > 1 {
            for (family, lines) in &size.families {
                if *lines > family_bound {
                    let what = format!("{} {family}", size.name);
                    over.insert(what.clone(), format!("{what}: {lines} lines, bound {family_bound}"));
                }
            }
        }
    }
    for (what, finding) in &over {
        if !exceptions.iter().any(|exception| exception.what == what) {
            failures.push(finding.clone());
        }
    }
    for exception in exceptions {
        if !over.contains_key(exception.what) {
            failures.push(format!(
                "{} (until {}): remove it: the check's exceptions only shrink",
                exception.what, exception.until
            ));
        }
    }
    failures
}

#[test]
fn workspace_libraries_and_families_fit() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let packages = packages(&String::from_utf8(output.stdout).expect("cargo metadata is UTF-8"))
        .expect("workspace metadata reads");
    let sizes: Vec<_> = packages.iter().map(measure).collect();
    for size in &sizes {
        if size.library > 60_000 {
            eprintln!("{}: {} library lines", size.name, size.library);
        }
        if size.families.len() > 1 {
            for (family, lines) in &size.families {
                if *lines > 20_000 {
                    eprintln!("{} {family}: {lines} family lines", size.name);
                }
            }
        }
    }
    let failures = findings(&sizes, 60_000, 20_000, EXCEPTIONS);
    assert!(failures.is_empty(), "Size findings:\n{}", failures.join("\n"));
}

fn scratch(files: &[(&str, &str)]) -> Scratch {
    let scratch = Scratch::new("boundary-sizes");
    for (name, source) in files {
        let path = scratch.path().join(name);
        fs::create_dir_all(path.parent().expect("fixture directory")).expect("create fixture directory");
        fs::write(path, source).expect("write fixture source");
    }
    scratch
}

fn package(scratch: &Scratch) -> Package {
    Package {
        name: "demo".to_owned(),
        directory: scratch.path().to_path_buf(),
        root: scratch.path().join("src/lib.rs"),
    }
}

fn reached(tree: &Tree, scratch: &Scratch) -> BTreeMap<String, usize> {
    tree.files
        .iter()
        .map(|(path, lines)| {
            (path.strip_prefix(scratch.path()).expect("fixture path").to_string_lossy().into_owned(), *lines)
        })
        .collect()
}

fn expected(files: &[(&str, usize)]) -> BTreeMap<String, usize> {
    files.iter().map(|(path, lines)| ((*path).to_owned(), *lines)).collect()
}

#[test]
fn module_files_and_nested_directories() {
    let scratch = scratch(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", "mod c;\n// a\n\n"),
        ("src/b/mod.rs", "// b\n\n\n"),
        ("src/a/c.rs", "// c\n\n\nlast"),
        ("src/unreached.rs", "not part of the library\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(
        reached(&tree, &scratch),
        expected(&[("src/lib.rs", 2), ("src/a.rs", 3), ("src/b/mod.rs", 3), ("src/a/c.rs", 4)])
    );
    assert_eq!(findings(&[measure(&package)], 10, 20, &[]), ["demo: 12 lines, bound 10"]);
}

#[test]
fn path_attributes_and_test_modules() {
    let scratch = scratch(&[
        ("src/lib.rs", "#[path = \"../extra/x.rs\"] pub mod x;\n#[cfg(test)]\nmod tests;\n"),
        ("extra/x.rs", "#[path = \"sibling.rs\"] mod sibling;\n"),
        ("extra/sibling.rs", "// sibling\n"),
        ("src/tests.rs", "// tests\n\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(
        reached(&tree, &scratch),
        expected(&[("src/lib.rs", 3), ("extra/x.rs", 1), ("extra/sibling.rs", 1), ("src/tests.rs", 2)])
    );
    assert_eq!(measure(&package).library, 7);
}

#[test]
fn inline_modules_follow_reference_path_rules() {
    let scratch = scratch(&[
        (
            "src/lib.rs",
            "mod inner { mod deep; #[path = \"other.rs\"] mod other; }\nmod a;\nmod b;\n#[path = \"thread_files\"] mod thread { #[path = \"tls.rs\"] mod local; }\n",
        ),
        ("src/inner/deep.rs", "// deep\n"),
        ("src/inner/other.rs", "// other\n"),
        ("src/a.rs", "mod inline { #[path = \"other.rs\"] mod other; }\n"),
        ("src/a/inline/other.rs", "// non-mod-rs\n"),
        ("src/b/mod.rs", "mod inline { #[path = \"other.rs\"] mod other; }\n"),
        ("src/b/inline/other.rs", "// mod-rs\n"),
        ("src/thread_files/tls.rs", "// explicit inline directory\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(
        reached(&tree, &scratch),
        expected(&[
            ("src/lib.rs", 4),
            ("src/inner/deep.rs", 1),
            ("src/inner/other.rs", 1),
            ("src/a.rs", 1),
            ("src/a/inline/other.rs", 1),
            ("src/b/mod.rs", 1),
            ("src/b/inline/other.rs", 1),
            ("src/thread_files/tls.rs", 1)
        ])
    );
    assert_eq!(measure(&package).library, 11);
}

#[test]
fn includes_follow_sources_skip_build_output_and_report_other_forms() {
    let scratch = scratch(&[
        (
            "src/lib.rs",
            "include!(\"../extra/direct.rs\");\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/extra/package.rs\"));\ninclude!(concat!(env!(\"OUT_DIR\"), \"/unread.rs\"));\ninclude!(env!(\"OTHER\"));\ninclude!(concat!(\"not\", \"supported\"));\ninclude!{\"other-form.rs\"}\ninclude_str!(\"missing.txt\"); include_bytes!(\"missing.bin\");\n",
        ),
        ("extra/direct.rs", "include!(r#\"nested.rs\"#);\n"),
        ("extra/package.rs", "// package\n"),
        ("extra/nested.rs", "// nested\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert_eq!(tree.failures, vec![format!("{}: unsupported include! form", package.root.display()); 3]);
    assert_eq!(
        reached(&tree, &scratch),
        expected(&[("src/lib.rs", 7), ("extra/direct.rs", 1), ("extra/package.rs", 1), ("extra/nested.rs", 1)])
    );
    assert_eq!(measure(&package).library, 10);
}

#[test]
fn comments_and_literals_name_no_modules() {
    let source = r##"// mod ghost;
/// mod ghost;
//! mod ghost;
/* mod ghost; /* nested */ mod ghost; */
/** mod ghost; */
const S: &str = "\" mod ghost;";
const R: &str = r#"mod ghost; \""#;
const B: &[u8] = b"mod ghost;";
const BR: &[u8] = br#"mod ghost;"#;
const C: &CStr = c"mod ghost;";
const CR: &CStr = cr#"mod ghost;"#;
const CH: char = '\'';
const UNICODE: char = '🦀';
fn lifetime<'a>(x: &'a str) { let c = b'\x7b'; }
mod real;
"##;
    let scratch = scratch(&[("src/lib.rs", source), ("src/real.rs", "// real\n")]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(reached(&tree, &scratch), expected(&[("src/lib.rs", 15), ("src/real.rs", 1)]));
    assert_eq!(measure(&package).library, 16);
}

#[test]
fn missing_sources_fail_and_repeated_paths_count_once() {
    let scratch = scratch(&[
        (
            "src/lib.rs",
            "mod a;\n#[path = \"./a.rs\"] mod again;\ninclude!(\"../src/a.rs\");\nmod missing;\nmod also_missing;\n",
        ),
        ("src/a.rs", "include!(\"lib.rs\");\n// cycle\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert_eq!(tree.failures.len(), 2, "{:?}", tree.failures);
    for name in ["missing.rs", "also_missing.rs"] {
        assert!(tree.failures.iter().any(|why| why.contains(name)), "{:?}", tree.failures);
    }
    assert_eq!(reached(&tree, &scratch), expected(&[("src/lib.rs", 5), ("src/a.rs", 2)]));
    assert_eq!(measure(&package).library, 7);
    assert_eq!(findings(&[measure(&package)], 10, 20, &[]).len(), 2);
}

#[test]
fn pattern_counts_tests_but_not_generated_records() {
    let record = "// record\n".repeat(30);
    let scratch = scratch(&[
        ("src/lib.rs", "mod a;\n#[cfg(test)] mod tests;\n\n#[path = \"generated/people_v1.rs\"] mod people;\n\n"),
        ("src/a.rs", "// a\n\n\n"),
        ("src/tests.rs", "// tests\n\n\n\n"),
        ("src/generated/people_v1.rs", &record),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(
        reached(&tree, &scratch),
        expected(&[("src/lib.rs", 5), ("src/a.rs", 3), ("src/tests.rs", 4), ("src/generated/people_v1.rs", 30)])
    );
    let size = measure(&package);
    assert_eq!(size.library, 12);
    assert_eq!(size.families, BTreeMap::from([("people".to_owned(), 30)]));
}

#[test]
fn families_sum_all_versions_and_glue_stays_in_library() {
    for (name, family) in [
        ("view_v12.rs", Some("view")),
        ("a_2_v0.rs", Some("a_2")),
        ("v.rs", None),
        ("_v1.rs", None),
        ("people_v1.txt", None),
        ("people_v1x.rs", None),
    ] {
        assert_eq!(record_family(Path::new(name)), family, "{name}");
    }
    let scratch = scratch(&[
        ("src/lib.rs", "#[path = \"generated/layouts.rs\"] mod layouts;\n"),
        ("src/generated/people_v1.rs", "// v1\n\n"),
        ("src/generated/people_v2.rs", "// v2\n\n\n"),
        ("src/generated/tasks_v3.rs", "// tasks\n"),
        ("src/generated/v4.rs", "// own\n\n\n\n"),
        ("src/generated/layouts.rs", "// glue\n\n"),
        ("src/generated/UPPER_v1.rs", "not a record\n"),
        ("src/generated/no_v.rs", "not a version\n"),
    ]);
    let package = package(&scratch);
    let tree = module_tree(&package.directory, &package.root);
    assert!(tree.failures.is_empty(), "{:?}", tree.failures);
    assert_eq!(reached(&tree, &scratch), expected(&[("src/lib.rs", 1), ("src/generated/layouts.rs", 2)]));
    let size = measure(&package);
    assert_eq!(size.library, 3);
    assert_eq!(
        size.families,
        BTreeMap::from([("people".to_owned(), 5), ("tasks".to_owned(), 1), ("(own)".to_owned(), 4)])
    );
    assert_eq!(findings(&[size], 3, 4, &[]), ["demo people: 5 lines, bound 4"]);
}

#[test]
fn a_single_family_has_no_family_bound() {
    for version in ["v1.rs", "people_v1.rs"] {
        let scratch = scratch(&[("src/lib.rs", "// root\n")]);
        let generated = scratch.path().join("src/generated");
        fs::create_dir(&generated).expect("generated directory");
        fs::write(generated.join(version), "// record\n\n").expect("record source");
        fs::write(generated.join(version.replace("v1", "v2")), "// second version\n\n\n")
            .expect("second record source");
        let package = package(&scratch);
        let tree = module_tree(&package.directory, &package.root);
        assert_eq!(tree.failures, Vec::<String>::new());
        assert_eq!(reached(&tree, &scratch), expected(&[("src/lib.rs", 1)]));
        let size = measure(&package);
        assert_eq!(size.library, 1);
        assert_eq!(size.families.values().copied().collect::<Vec<_>>(), [5]);
        assert_eq!(findings(&[size], 1, 1, &[]), Vec::<String>::new());
    }
}

#[test]
fn bounds_and_exceptions_only_shrink_and_report_every_finding() {
    let mut size = Size {
        name: "demo".to_owned(),
        library: 10,
        families: BTreeMap::from([("people".to_owned(), 4), ("tasks".to_owned(), 4)]),
        failures: vec![],
    };
    assert_eq!(findings(&[size], 10, 4, &[]), Vec::<String>::new());
    size = Size {
        name: "demo".to_owned(),
        library: 12,
        families: BTreeMap::from([("people".to_owned(), 6), ("tasks".to_owned(), 5)]),
        failures: vec![],
    };
    assert_eq!(
        findings(
            &[size],
            10,
            4,
            &[Exception { what: "gone", until: "split" }, Exception { what: "demo vanished", until: "G2" }]
        ),
        [
            "demo: 12 lines, bound 10",
            "demo people: 6 lines, bound 4",
            "demo tasks: 5 lines, bound 4",
            "gone (until split): remove it: the check's exceptions only shrink",
            "demo vanished (until G2): remove it: the check's exceptions only shrink"
        ]
    );
    let size = Size {
        name: "demo".to_owned(),
        library: 12,
        families: BTreeMap::from([("people".to_owned(), 6), ("tasks".to_owned(), 4)]),
        failures: vec![],
    };
    assert_eq!(
        findings(
            &[size],
            10,
            4,
            &[Exception { what: "demo", until: "split" }, Exception { what: "demo people", until: "G2" }]
        ),
        Vec::<String>::new()
    );
    let size = Size {
        name: "demo".to_owned(),
        library: 10,
        families: BTreeMap::from([("people".to_owned(), 4), ("tasks".to_owned(), 4)]),
        failures: vec![],
    };
    assert_eq!(
        findings(
            &[size],
            10,
            4,
            &[Exception { what: "demo", until: "split" }, Exception { what: "demo people", until: "G2" }]
        ),
        [
            "demo (until split): remove it: the check's exceptions only shrink",
            "demo people (until G2): remove it: the check's exceptions only shrink"
        ]
    );
}

#[test]
fn metadata_selects_only_workspace_library_kinds_and_requires_readable_fields() {
    let targets: Vec<_> = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro", "bin", "test"]
        .into_iter()
        .map(|kind| serde_json::json!({"name": kind, "kind": [kind], "src_path": "/workspace/demo/src/../src/root.rs", "test": true}))
        .collect();
    let mut metadata = serde_json::json!({"workspace_members": ["demo"], "packages": [{"id": "demo", "name": "demo", "manifest_path": "/workspace/demo/Cargo.toml", "targets": targets}, {"id": "external"}]});
    let libraries = packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(libraries.len(), 6);
    assert!(libraries.iter().all(|package| package.name == "demo"
        && package.directory == Path::new("/workspace/demo")
        && package.root == Path::new("/workspace/demo/src/root.rs")));
    metadata["packages"][0]["targets"][0]["src_path"] = Value::Null;
    assert!(packages(&metadata.to_string()).expect_err("missing source").contains("src_path"));
    assert!(packages("not JSON").is_err());
    assert!(packages("{}").is_err());
}

#[test]
fn metadata_reader_keeps_every_member_and_target_and_requires_their_fields() {
    let mut metadata = serde_json::json!({
        "workspace_members": ["demo", "empty"],
        "packages": [
            {"id": "demo", "name": "demo", "manifest_path": "/workspace/demo/Cargo.toml", "targets": [
                {"name": "a", "kind": ["example"], "src_path": "/workspace/demo/examples/../examples/a.rs", "test": false},
                {"name": "demo", "kind": ["rlib", "staticlib"], "src_path": "/workspace/demo/src/lib.rs", "test": true}
            ]},
            {"id": "empty", "name": "empty", "manifest_path": "/workspace/empty/Cargo.toml", "targets": []},
            {"id": "external"}
        ]
    });
    let members = fabrig_boundary::packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].name, "demo");
    assert_eq!(members[0].directory, Path::new("/workspace/demo"));
    assert_eq!(members[1].name, "empty");
    assert!(members[1].targets.is_empty());
    let targets = &members[0].targets;
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].name, "a");
    assert_eq!(targets[0].kinds, ["example"]);
    assert_eq!(targets[0].root, Path::new("/workspace/demo/examples/a.rs"));
    assert!(!targets[0].test);
    assert_eq!(targets[1].kinds, ["rlib", "staticlib"]);
    assert!(targets[1].test);
    for field in ["name", "kind", "src_path", "test"] {
        let mut broken = metadata.clone();
        broken["packages"][0]["targets"][0][field] = Value::Null;
        let why = fabrig_boundary::packages(&broken.to_string()).expect_err("missing target field");
        assert!(why.starts_with("demo:") && why.contains(field), "{why}");
    }
    for field in ["manifest_path", "targets"] {
        let mut broken = metadata.clone();
        broken["packages"][0][field] = Value::Null;
        let why = fabrig_boundary::packages(&broken.to_string()).expect_err("missing package field");
        assert!(why.starts_with("demo:") && why.contains(field), "{why}");
    }
    metadata["workspace_members"] = serde_json::json!(["absent"]);
    assert_eq!(
        fabrig_boundary::packages(&metadata.to_string()).expect_err("missing member"),
        "metadata: workspace member missing from packages"
    );
}
