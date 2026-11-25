//! Current family texts are embedded only by their public owner constants
//! (skein/docs/design/data/codec/golden.md, section 1).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use syn::buffer::{Cursor, TokenBuffer};
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
// Translation rows for the same schema add to its allowed files.
fn owners(list: &str) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut owners = BTreeMap::<_, Vec<PathBuf>>::new();
    for line in list.lines() {
        let mut paths = line.split('#').next().expect("before comment").split_whitespace();
        let Some(module) = paths.next() else { continue };
        let codec = module.split("/src/").next().expect("codec crate");
        for schema in paths {
            // Translation rows end in maps, which are not embedded schema texts.
            if Path::new(schema).extension().is_some_and(|extension| extension == "maps") {
                continue;
            }
            let owner = schema.split("/schema/").next().expect("schema owner");
            let allowed = owners.entry(PathBuf::from(schema)).or_default();
            for file in [
                Path::new(owner).join("src/lib.rs"),
                Path::new(codec).join("src/lib.rs"),
                Path::new(owner).join("src/patch_set.rs"),
            ] {
                if !allowed.contains(&file) {
                    allowed.push(file);
                }
            }
        }
    }
    owners
}

struct Embeds<'a> {
    file: &'a Path,
    owners: &'a BTreeMap<PathBuf, Vec<PathBuf>>,
    source: &'a str,
    inputs: std::vec::IntoIter<(&'a str, usize)>,
    constant: Option<String>,
    at_root: bool,
    seen: Vec<PathBuf>,
    failures: Vec<String>,
}

impl Embeds<'_> {
    fn include(&mut self) {
        let (input, offset) = self.inputs.next().expect("token and syntax walks see the same literal include");
        let literal = syn::parse_str::<syn::LitStr>(input).expect("a string token");
        let line = self.source[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
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
    fn tokens(&mut self, mut tokens: Cursor<'_>) {
        while !tokens.eof() {
            if let Some((name, after_name)) = tokens.ident()
                && name == "include_str"
                && let Some((punctuation, after_bang)) = after_name.punct()
                && punctuation.as_char() == '!'
                && let Some((group, _, _, after_group)) = after_bang.any_group()
            {
                if syn::parse2::<syn::LitStr>(group.token_stream()).is_ok() {
                    self.include();
                } else {
                    self.tokens(group);
                }
                tokens = after_group;
            } else if let Some((group, _, _, after_group)) = tokens.any_group() {
                self.tokens(group);
                tokens = after_group;
            } else {
                tokens = tokens.token_tree().expect("nonempty cursor").1;
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
        if mac.path.is_ident("include_str") && syn::parse2::<syn::LitStr>(mac.tokens.clone()).is_ok() {
            self.include();
        } else {
            let tokens = TokenBuffer::new2(mac.tokens.clone());
            self.tokens(tokens.begin());
        }
    }
}

fn inspect(file: &Path, source: &str, owners: &BTreeMap<PathBuf, Vec<PathBuf>>) -> (Vec<PathBuf>, Vec<String>) {
    let syntax = syn::parse_file(source).unwrap_or_else(|why| panic!("{}: {why}", file.display()));
    let mut embeds = Embeds {
        file,
        owners,
        source,
        inputs: fabrig_boundary::schema_includes(source).into_iter(),
        constant: None,
        at_root: true,
        seen: Vec::new(),
        failures: Vec::new(),
    };
    embeds.visit_file(&syntax);
    (embeds.seen, embeds.failures)
}

#[test]
fn schema_includes_are_confined_to_the_owner_constant() {
    let owners = owners(
        "owner/src/generated/current.rs owner/schema/current-v1.schema owner/schema/current-v1-part1.schema owner/schema/current.maps",
    );
    assert_eq!(owners.len(), 2, "maps are not schema texts");
    let owner = Path::new("owner/src/lib.rs");
    let good = r#"pub const SCHEMA: &str = concat!(include_str!("../schema/current-v1.schema"), include_str!("../schema/current-v1-part1.schema"));"#;
    let (seen, failures) = inspect(owner, good, &owners);
    assert_eq!(seen.len(), 2);
    assert!(failures.is_empty(), "{failures:?}");
    let bad = r##"// include_str!("../owner/schema/current-v1.schema")
const TEXT: &str = r#"include_str!("../owner/schema/current-v1.schema")"#;
fn fixture() { family!(
 include_str! {"../owner/schema/current-v1.schema"}); }"##;
    let (_, failures) = inspect(Path::new("tests/fixture.rs"), bad, &owners);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].starts_with("tests/fixture.rs:4:"), "{failures:?}");
    for source in [
        "const SCHEMA: &str = include_str!(\"../schema/current-v1.schema\");",
        "pub const WRONG: &str = include_str!(\"../schema/current-v1.schema\");",
        "fn fixture() { include_str!(\"../schema/current-v1.schema\"); }",
    ] {
        assert_eq!(inspect(owner, source, &owners).1.len(), 1, "{source}");
    }
    let (_, failures) = inspect(Path::new("other/src/lib.rs"), good, &owners);
    assert!(failures.is_empty(), "unlisted texts are outside this check");

    let shared_owners = self::owners(
        "a-codec/src/generated/x.rs owner/schema/x-v1.schema\n\
         owner/src/generated/x.rs owner/schema/x-v1.schema owner/schema/x.maps",
    );
    let source = r#"pub const SCHEMA: &str = include_str!("../../owner/schema/x-v1.schema");"#;
    let (seen, failures) = inspect(Path::new("a-codec/src/lib.rs"), source, &shared_owners);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(seen, [PathBuf::from("owner/schema/x-v1.schema")]);

    let (seen, failures) = inspect(Path::new("b-codec/src/lib.rs"), source, &shared_owners);
    assert!(seen.is_empty(), "{seen:?}");
    assert_eq!(
        failures,
        ["b-codec/src/lib.rs:1: owner/schema/x-v1.schema must be embedded only in its owner's public schema constant"]
    );
}

#[test]
fn each_current_schema_text_is_embedded_once_by_its_owner() {
    let root = skein_scratch::paths::workspace();
    let list = fs::read_to_string(root.join("tools/merge/codecs.txt")).expect("codec list");
    let owners = owners(&list);
    let output =
        Command::new("git").args(["ls-files", "-z", "--", "*.rs"]).current_dir(&root).output().expect("tracked Rust");
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
