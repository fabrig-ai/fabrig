//! Shared metadata and module-tree readers (tools/crates/design.md, 2.1 and 3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

/// One Cargo target, including whether Cargo builds its test binary (section 2.4).
#[derive(Debug)]
pub struct Target {
    pub name: String,
    pub kinds: Vec<String>,
    pub root: PathBuf,
    pub test: bool,
}

/// A workspace member and all its targets.
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub directory: PathBuf,
    pub targets: Vec<Target>,
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

/// Read every workspace member and target from Cargo's metadata JSON.
///
/// # Errors
/// Returns a diagnostic for malformed JSON, missing workspace members, or
/// unreadable fields. Package-field diagnostics name the package.
pub fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let metadata: Value = serde_json::from_str(metadata).map_err(|why| format!("metadata: {why}"))?;
    let members = metadata["workspace_members"].as_array().ok_or("metadata: no workspace_members array")?;
    let members = members
        .iter()
        .map(|member| member.as_str().ok_or("metadata: invalid workspace member"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut packages = Vec::new();
    for package in metadata["packages"].as_array().ok_or("metadata: no packages array")? {
        let id = package["id"].as_str().ok_or("metadata: no package id")?;
        if !members.contains(id) {
            continue;
        }
        let name = package["name"].as_str().ok_or("metadata: no package name")?;
        let manifest = package["manifest_path"].as_str().ok_or_else(|| format!("{name}: no manifest_path"))?;
        let directory = Path::new(manifest).parent().ok_or_else(|| format!("{name}: no manifest directory"))?;
        let mut targets = Vec::new();
        for target in package["targets"].as_array().ok_or_else(|| format!("{name}: no targets array"))? {
            let kinds = target["kind"].as_array().ok_or_else(|| format!("{name}: no target kind array"))?;
            let kinds = kinds
                .iter()
                .map(|kind| kind.as_str().map(str::to_owned).ok_or_else(|| format!("{name}: invalid target kind")))
                .collect::<Result<Vec<_>, _>>()?;
            let target_name = target["name"].as_str().ok_or_else(|| format!("{name}: no target name"))?;
            let source = target["src_path"].as_str().ok_or_else(|| format!("{name}: no target src_path"))?;
            let test = target["test"].as_bool().ok_or_else(|| format!("{name}: no target test flag"))?;
            targets.push(Target { name: target_name.to_owned(), kinds, root: normalized(Path::new(source)), test });
        }
        packages.push(Package { name: name.to_owned(), directory: normalized(directory), targets });
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
    Ok(packages)
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

/// Source files with their line counts, and every source-reading failure.
#[derive(Debug, Default)]
pub struct Tree {
    pub files: BTreeMap<PathBuf, usize>,
    pub failures: Vec<String>,
}

/// Read a target's sources by tools/crates/design.md, 2.1, including test modules.
/// Unreadable sources and unsupported includes are recorded in `Tree::failures`.
#[must_use]
pub fn module_tree(package: &Path, root: &Path) -> Tree {
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
