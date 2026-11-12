//! Library and generated-family sizes (tools/crates/design.md, 2 and 3.1).
//! Cargo metadata selects workspace libraries. A lexical module-tree scan reads
//! their sources, including cfg(test), path attributes and source includes.
//! Libraries outside generated record code are bounded at 60,000 lines; families
//! sharing a package are bounded at 20,000, over every kept version. Lines include
//! blanks and comments. Exceptions only shrink: an unmatched exception fails too.
//! Metadata or a source the check cannot read, and unsupported includes, fail it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

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
    Exception { what: "jig-domain-core", until: "G6" },
    Exception { what: "temper-service", until: "G4, or the worker's crates (temper/docs/design/draft/cli.md, 6)" },
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
    Exception { what: "jig-protocol-store people", until: "G2" },
    Exception { what: "jig-protocol-store tasks", until: "G2" },
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

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => assert!(result.pop(), "{} leaves its root", path.display()),
            Component::CurDir => (),
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => result.push(part),
        }
    }
    result
}

fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let metadata: Value = serde_json::from_str(metadata).map_err(|why| format!("metadata: {why}"))?;
    let members = metadata["workspace_members"].as_array().ok_or("metadata: no workspace_members array")?;
    let members = members
        .iter()
        .map(|member| member.as_str().ok_or("metadata: invalid workspace member"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut libraries = Vec::new();
    for package in metadata["packages"].as_array().ok_or("metadata: no packages array")? {
        let id = package["id"].as_str().ok_or("metadata: no package id")?;
        if !members.contains(id) {
            continue;
        }
        let name = package["name"].as_str().ok_or("metadata: no package name")?;
        let manifest = package["manifest_path"].as_str().ok_or_else(|| format!("{name}: no manifest_path"))?;
        let directory = Path::new(manifest).parent().ok_or_else(|| format!("{name}: no manifest directory"))?;
        for target in package["targets"].as_array().ok_or_else(|| format!("{name}: no targets array"))? {
            let kinds = target["kind"].as_array().ok_or_else(|| format!("{name}: no target kind array"))?;
            let kinds = kinds
                .iter()
                .map(|kind| kind.as_str().ok_or_else(|| format!("{name}: invalid target kind")))
                .collect::<Result<Vec<_>, _>>()?;
            if kinds.iter().any(|kind| ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"].contains(kind)) {
                let source = target["src_path"].as_str().ok_or_else(|| format!("{name}: no library src_path"))?;
                libraries.push(Package {
                    name: name.to_owned(),
                    directory: normalized(directory),
                    root: normalized(Path::new(source)),
                });
            }
        }
    }
    if !members.iter().all(|id| {
        metadata["packages"]
            .as_array()
            .expect("packages checked")
            .iter()
            .any(|package| package["id"].as_str() == Some(id))
    }) {
        return Err("metadata: workspace member missing from packages".to_owned());
    }
    Ok(libraries)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    Word(&'a str),
    String(&'a str),
    Symbol(u8),
    Literal,
}

/// A byte scan, not a Rust parse. Only path and include strings are decoded.
fn tokens(source: &str) -> Vec<Token<'_>> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let rest = &bytes[at..];
        if rest[0].is_ascii_whitespace() {
            at += 1;
        } else if rest.starts_with(b"//") {
            at += rest.iter().position(|byte| *byte == b'\n').unwrap_or(rest.len());
        } else if rest.starts_with(b"/*") {
            at += 2;
            let mut depth = 1;
            while at < bytes.len() && depth > 0 {
                if bytes[at..].starts_with(b"/*") {
                    depth += 1;
                    at += 2;
                } else if bytes[at..].starts_with(b"*/") {
                    depth -= 1;
                    at += 2;
                } else {
                    at += 1;
                }
            }
        } else if let Some((length, string)) = literal(rest) {
            result.push(if string { Token::String(&source[at..at + length]) } else { Token::Literal });
            at += length;
        } else if rest[0].is_ascii_alphabetic() || rest[0] == b'_' || rest[0] >= 128 {
            let length = rest
                .iter()
                .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte < 128)
                .unwrap_or(rest.len());
            result.push(Token::Word(&source[at..at + length]));
            at += length;
        } else {
            result.push(Token::Symbol(rest[0]));
            at += 1;
        }
    }
    result
}

/// Returns the entire literal's length, distinguishing strings from byte/C strings
/// and characters. An apostrophe introducing a lifetime is not a character.
fn literal(bytes: &[u8]) -> Option<(usize, bool)> {
    let prefix = usize::from(bytes.starts_with(b"b") || bytes.starts_with(b"c"));
    let rest = &bytes[prefix..];
    if rest.starts_with(b"r") {
        let hashes = rest[1..].iter().take_while(|byte| **byte == b'#').count();
        if rest.get(1 + hashes) == Some(&b'"') {
            let mut at = prefix + hashes + 2;
            while at < bytes.len() {
                if bytes[at] == b'"'
                    && bytes.get(at + 1..at + 1 + hashes).is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
                {
                    return Some((at + 1 + hashes, prefix == 0));
                }
                at += 1;
            }
            return Some((bytes.len(), prefix == 0));
        }
    }
    if rest.starts_with(b"\"") {
        let mut at = prefix + 1;
        while at < bytes.len() {
            if bytes[at] == b'"' {
                return Some((at + 1, prefix == 0));
            }
            at += if bytes[at] == b'\\' { 2 } else { 1 };
        }
        return Some((bytes.len(), prefix == 0));
    }
    if rest.starts_with(b"'") {
        let mut at = prefix + 1;
        if bytes.get(at) == Some(&b'\\') {
            at += 2;
            if bytes.get(at - 1) == Some(&b'u') && bytes.get(at) == Some(&b'{') {
                at += bytes[at..].iter().position(|byte| *byte == b'}')? + 1;
            } else if bytes.get(at - 1) == Some(&b'x') {
                at += 2;
            }
        } else {
            let first = *bytes.get(at)?;
            at += if first < 128 {
                1
            } else if first < 224 {
                2
            } else if first < 240 {
                3
            } else {
                4
            };
        }
        if bytes.get(at) == Some(&b'\'') {
            return Some((at + 1, false));
        }
    }
    None
}

fn string(token: &Token<'_>) -> Option<String> {
    if let Token::String(text) = token {
        syn::parse_str::<syn::LitStr>(text).ok().map(|literal| literal.value())
    } else {
        None
    }
}

/// An include consumes its whole token group, even when it is unsupported.
fn include_path(tokens: &[Token<'_>], package: &Path, file: &Path) -> Result<Option<PathBuf>, String> {
    let tokens = tokens.strip_suffix(&[Token::Symbol(b',')]).unwrap_or(tokens);
    if tokens.windows(5).any(|window| {
        window.starts_with(&[Token::Word("env"), Token::Symbol(b'!'), Token::Symbol(b'(')])
            && string(&window[3]).as_deref() == Some("OUT_DIR")
            && window[4] == Token::Symbol(b')')
    }) {
        return Ok(None);
    }
    if let [value @ Token::String(_)] = tokens {
        return Ok(Some(file.parent().expect("source directory").join(string(value).expect("string literal"))));
    }
    if let [
        Token::Word("concat"),
        Token::Symbol(b'!'),
        Token::Symbol(b'('),
        Token::Word("env"),
        Token::Symbol(b'!'),
        Token::Symbol(b'('),
        variable,
        Token::Symbol(b')'),
        Token::Symbol(b','),
        suffix,
        Token::Symbol(b')'),
    ] = tokens
        && string(variable).as_deref() == Some("CARGO_MANIFEST_DIR")
        && let Some(suffix) = string(suffix)
        && let Some(relative) = suffix.strip_prefix('/')
    {
        return Ok(Some(package.join(relative)));
    }
    Err(format!("{}: unsupported include! form", file.display()))
}

/// Both directories are needed by the Reference's items.mod.outlined.path rules:
/// top-level path attributes use the file's parent, while default child modules
/// and path attributes inside inline modules use the module's directory.
#[derive(Clone)]
struct Directories {
    module: PathBuf,
    attribute: PathBuf,
}

fn source_modules(package: &Path, file: &Path, root: bool, source: &str) -> (Vec<PathBuf>, Vec<String>) {
    let parent = file.parent().expect("source directory");
    let module = if root || file.file_name().is_some_and(|name| name == "mod.rs") {
        parent.to_path_buf()
    } else {
        parent.join(file.file_stem().expect("source file stem"))
    };
    let mut directories = Directories { module, attribute: parent.to_path_buf() };
    let mut braces = Vec::new();
    let mut path = None;
    let tokens = tokens(source);
    let mut files = Vec::new();
    let mut failures = Vec::new();
    let mut at = 0;
    while at < tokens.len() {
        let rest = &tokens[at..];
        if rest.starts_with(&[Token::Symbol(b'#'), Token::Symbol(b'[')]) {
            let start = at + 2;
            at = group_end(&tokens, at + 1, b'[', b']');
            if let [Token::Word("path"), Token::Symbol(b'='), value] = &tokens[start..at - 1] {
                path = string(value);
            }
        } else if let [Token::Word("mod"), Token::Word(name), Token::Symbol(end @ (b';' | b'{')), ..] = rest {
            let explicit = path.take().map(|path| directories.attribute.join(path));
            if *end == b'{' {
                let module = explicit.unwrap_or_else(|| directories.module.join(name));
                braces.push(directories);
                directories = Directories { attribute: module.clone(), module };
            } else if let Some(explicit) = explicit {
                files.push(explicit);
            } else {
                let flat = directories.module.join(format!("{name}.rs"));
                let nested = directories.module.join(name).join("mod.rs");
                if flat.is_file() && nested.is_file() {
                    failures.push(format!(
                        "{}: mod {name} has both {} and {}",
                        file.display(),
                        flat.display(),
                        nested.display()
                    ));
                } else {
                    files.push(if nested.is_file() { nested } else { flat });
                }
            }
            at += 3;
        } else if rest.starts_with(&[Token::Word("include"), Token::Symbol(b'!')]) {
            let start = at + 3;
            let delimiters = match rest.get(2) {
                Some(Token::Symbol(b'(')) => Some((b'(', b')')),
                Some(Token::Symbol(b'[')) => Some((b'[', b']')),
                Some(Token::Symbol(b'{')) => Some((b'{', b'}')),
                Some(Token::Word(_) | Token::String(_) | Token::Symbol(_) | Token::Literal) | None => None,
            };
            let included = if let Some((open, close)) = delimiters {
                at = group_end(&tokens, at + 2, open, close);
                if open == b'(' {
                    include_path(&tokens[start..at - 1], package, file)
                } else {
                    Err(format!("{}: unsupported include! form", file.display()))
                }
            } else {
                at += 2;
                Err(format!("{}: unsupported include! form", file.display()))
            };
            match included {
                Ok(Some(path)) => files.push(path),
                Ok(None) => (),
                Err(why) => failures.push(why),
            }
        } else {
            if rest[0] == Token::Symbol(b'{') {
                braces.push(directories.clone());
            } else if rest[0] == Token::Symbol(b'}') {
                if let Some(previous) = braces.pop() {
                    directories = previous;
                }
                path = None;
            } else if rest[0] == Token::Symbol(b';') {
                path = None;
            }
            at += 1;
        }
    }
    (files, failures)
}

fn group_end(tokens: &[Token<'_>], start: usize, open: u8, close: u8) -> usize {
    let mut depth = 0;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if *token == Token::Symbol(open) {
            depth += 1;
        } else if *token == Token::Symbol(close) {
            depth -= 1;
            if depth == 0 {
                return index + 1;
            }
        }
    }
    tokens.len()
}

#[derive(Debug, Default)]
struct Tree {
    files: BTreeMap<PathBuf, usize>,
    failures: Vec<String>,
}

fn module_tree(package: &Path, root: &Path) -> Tree {
    let mut tree = Tree::default();
    let mut seen = BTreeSet::new();
    let mut pending = vec![(normalized(root), true)];
    while let Some((file, root)) = pending.pop() {
        if !seen.insert(file.clone()) {
            continue;
        }
        match fs::read_to_string(&file) {
            Ok(source) => {
                tree.files.insert(file.clone(), source.lines().count());
                let (children, failures) = source_modules(package, &file, root, &source);
                tree.failures.extend(failures);
                pending.extend(children.into_iter().map(|path| (normalized(&path), false)));
            }
            Err(why) => tree.failures.push(format!("{}: {why}", file.display())),
        }
    }
    tree
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
        assert!(tree.failures.is_empty());
        assert_eq!(reached(&tree, &scratch), expected(&[("src/lib.rs", 1)]));
        let size = measure(&package);
        assert_eq!(size.library, 1);
        assert_eq!(size.families.values().copied().collect::<Vec<_>>(), [5]);
        assert!(findings(&[size], 1, 1, &[]).is_empty());
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
    assert!(findings(&[size], 10, 4, &[]).is_empty());
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
    assert!(
        findings(
            &[size],
            10,
            4,
            &[Exception { what: "demo", until: "split" }, Exception { what: "demo people", until: "G2" }]
        )
        .is_empty()
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
        .map(|kind| serde_json::json!({"kind": [kind], "src_path": "/workspace/demo/src/../src/root.rs"}))
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
