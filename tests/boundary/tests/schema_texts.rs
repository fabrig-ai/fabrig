//! Current family texts are embedded only by their public owner constants
//! (skein/docs/design/data/codec/README.md, section 6).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use proc_macro2::{TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::visit::Visit;

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => assert!(result.pop(), "schema path stays in the repository"),
            Component::CurDir => (),
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => result.push(part),
        }
    }
    result
}

// A schema may be exposed by its codec when tests cannot link its owner.
fn owners(list: &str) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut owners = BTreeMap::new();
    for line in list.lines() {
        let mut paths = line.split('#').next().expect("before comment").split_whitespace();
        let Some(module) = paths.next() else { continue };
        let codec = module.split("/src/").next().expect("codec crate");
        for schema in paths {
            let owner = schema.split("/schema/").next().expect("schema owner");
            owners.insert(
                PathBuf::from(schema),
                vec![
                    Path::new(owner).join("src/lib.rs"),
                    Path::new(codec).join("src/lib.rs"),
                    Path::new(owner).join("src/patch_set.rs"),
                ],
            );
        }
    }
    owners
}

struct Embeds<'a> {
    file: &'a Path,
    owners: &'a BTreeMap<PathBuf, Vec<PathBuf>>,
    constant: Option<String>,
    at_root: bool,
    seen: Vec<PathBuf>,
    failures: Vec<String>,
}

impl Embeds<'_> {
    fn include(&mut self, tokens: TokenStream, line: usize) {
        let Ok(literal) = syn::parse2::<syn::LitStr>(tokens) else { return };
        let schema = normalized(&self.file.parent().expect("source parent").join(literal.value()));
        let Some(owners) = self.owners.get(&schema) else { return };
        let allowed = self.constant.as_deref().is_some_and(|name| {
            if self.file.ends_with("src/patch_set.rs") {
                name == "STORE_FAMILIES" || name == "OWNER_SET"
            } else {
                name == "SCHEMA" || name.ends_with("_SCHEMA")
            }
        });
        if allowed && owners.iter().any(|owner| owner == self.file) {
            self.seen.push(schema);
        } else {
            self.failures.push(format!(
                "{}:{line}: {} must be embedded only in its owner's public schema constant",
                self.file.display(),
                schema.display()
            ));
        }
    }

    // Visit tokens too: includes inside concat!, family! and macro_rules! are
    // not expressions in syn's tree. Comments and string contents are not tokens.
    fn tokens(&mut self, tokens: TokenStream) {
        let mut tokens = tokens.into_iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                TokenTree::Ident(name) if name == "include_str" => {
                    if tokens.peek().is_some_and(|token| matches!(token, TokenTree::Punct(p) if p.as_char() == '!')) {
                        tokens.next();
                        if let Some(TokenTree::Group(group)) = tokens.next() {
                            self.include(group.stream(), name.span().start().line);
                        }
                    }
                }
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => (),
            }
        }
    }
}

impl<'ast> Visit<'ast> for Embeds<'_> {
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        let previous = self.constant.take();
        if self.at_root && matches!(item.vis, syn::Visibility::Public(_)) {
            self.constant = Some(item.ident.to_string());
        }
        syn::visit::visit_item_const(self, item);
        self.constant = previous;
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let previous = self.at_root;
        self.at_root = false;
        syn::visit::visit_item_mod(self, item);
        self.at_root = previous;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let previous = self.at_root;
        self.at_root = false;
        syn::visit::visit_item_fn(self, item);
        self.at_root = previous;
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("include_str") {
            self.include(mac.tokens.clone(), mac.path.span().start().line);
        } else {
            self.tokens(mac.tokens.clone());
        }
    }
}

fn inspect(file: &Path, source: &str, owners: &BTreeMap<PathBuf, Vec<PathBuf>>) -> (Vec<PathBuf>, Vec<String>) {
    let syntax = syn::parse_file(source).unwrap_or_else(|why| panic!("{}: {why}", file.display()));
    let mut embeds = Embeds { file, owners, constant: None, at_root: true, seen: Vec::new(), failures: Vec::new() };
    embeds.visit_file(&syntax);
    (embeds.seen, embeds.failures)
}

#[test]
fn schema_includes_are_confined_to_the_owner_constant() {
    let owners = owners("owner/src/generated/current.rs owner/schema/current-v1.schema owner/schema/current-v1-part1.schema");
    let owner = Path::new("owner/src/lib.rs");
    let good = r#"pub const SCHEMA: &str = concat!(include_str!("../schema/current-v1.schema"), include_str!("../schema/current-v1-part1.schema"));"#;
    let (seen, failures) = inspect(owner, good, &owners);
    assert_eq!(seen.len(), 2);
    assert!(failures.is_empty(), "{failures:?}");
    let bad = "fn fixture() { family!(\n include_str! {\"../owner/schema/current-v1.schema\"}); }";
    let (_, failures) = inspect(Path::new("tests/fixture.rs"), bad, &owners);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].starts_with("tests/fixture.rs:2:"), "{failures:?}");
    for source in [
        "const SCHEMA: &str = include_str!(\"../schema/current-v1.schema\");",
        "pub const WRONG: &str = include_str!(\"../schema/current-v1.schema\");",
        "fn fixture() { include_str!(\"../schema/current-v1.schema\"); }",
    ] {
        assert_eq!(inspect(owner, source, &owners).1.len(), 1, "{source}");
    }
    let (_, failures) = inspect(Path::new("other/src/lib.rs"), good, &owners);
    assert!(failures.is_empty(), "unlisted texts are outside this check");
}

#[test]
fn each_current_schema_text_is_embedded_once_by_its_owner() {
    let root = skein_scratch::paths::workspace();
    let list = fs::read_to_string(root.join("tools/merge/codecs.txt")).expect("codec list");
    let owners = owners(&list);
    let output = Command::new("git").args(["ls-files", "-z", "--", "*.rs"]).current_dir(&root).output().expect("tracked Rust");
    assert!(output.status.success(), "git ls-files: {}", String::from_utf8_lossy(&output.stderr));
    let mut counts = BTreeMap::<_, usize>::new();
    let mut failures = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let file = Path::new(std::str::from_utf8(raw).expect("UTF-8 tracked path"));
        let source = fs::read_to_string(root.join(file)).expect("tracked Rust source");
        if !source.contains("include_str") || !source.contains(".schema") {
            continue;
        }
        let (seen, refused) = inspect(file, &source, &owners);
        failures.extend(refused);
        for schema in seen {
            *counts.entry(schema).or_default() += 1;
        }
    }
    for schema in owners.keys() {
        let count = counts.get(schema).copied().unwrap_or(0);
        if count != 1 {
            failures.push(format!("{}: expected one owner embed, found {count}", schema.display()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
