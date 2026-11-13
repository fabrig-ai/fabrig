//! Closure and reachable-shape checks (client/domain/contract/README.md, section 10).
#![expect(clippy::disallowed_types, reason = "schema tests inspect the generator's owned declaration tree")]
#![expect(clippy::disallowed_methods, reason = "ordinary schema tests collect and compare the parser's declarations")]
#![expect(clippy::disallowed_macros, reason = "schema listings format author-owned types and tags for the text golden")]

use alloc::collections::BTreeSet;
use core::fmt::Write;
use skein_codegen::{Declaration, Schema, Type, parse};
use std::{fs, path::PathBuf};

const ROOTS: [&str; 4] = ["PublicProjects", "PublicBoard", "PublicCard", "PublicApplication"];

#[expect(clippy::disallowed_methods, reason = "jig's round: reads CARGO_MANIFEST_DIR to locate the public schema")]
fn package() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo supplies the running package"))
}

fn source() -> String {
    fs::read_to_string(package().join("schema/public-v1.schema")).expect("public schema readable")
}

fn golden() -> String {
    fs::read_to_string(package().join("schema/public-v1.fields")).expect("public fields golden readable")
}

fn name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Record(record) => &record.name,
        Declaration::Enum(enumeration) => &enumeration.name,
    }
}

fn named(ty: &Type) -> Option<&str> {
    match ty {
        Type::Named(name) => Some(name),
        Type::List(_, item) | Type::Option(item) => named(item),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Bytes(_)
        | Type::Secret(_)
        | Type::Text(_) => None,
    }
}

fn reachable(schema: &Schema) -> BTreeSet<String> {
    let mut pending: Vec<String> = ROOTS.into_iter().map(String::from).collect();
    let mut reached = BTreeSet::new();
    while let Some(next) = pending.pop() {
        if !reached.insert(next.clone()) {
            continue;
        }
        let declaration = schema
            .declarations
            .iter()
            .find(|declaration| name(declaration) == next)
            .expect("a public root or parsed reference is declared");
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    if let Some(name) = named(&field.ty) {
                        pending.push(name.into());
                    }
                }
            }
            Declaration::Enum(enumeration) => {
                for variant in &enumeration.variants {
                    if let Some(record) = &variant.record {
                        pending.push(record.clone());
                    }
                }
            }
        }
    }
    reached
}

fn spelling(ty: &Type) -> String {
    match ty {
        Type::U8 => "u8".into(),
        Type::U16 => "u16".into(),
        Type::U32 => "u32".into(),
        Type::U64 => "u64".into(),
        Type::Bool => "bool".into(),
        Type::Duration => "duration".into(),
        Type::Fixed(bound) => format!("fixed {bound}"),
        Type::Bytes(bound) => format!("bytes {bound}"),
        Type::Secret(bound) => format!("secret {bound}"),
        Type::Text(bound) => format!("text {bound}"),
        Type::List(bound, item) => format!("list {bound} {}", spelling(item)),
        Type::Option(item) => format!("option {}", spelling(item)),
        Type::Named(name) => name.clone(),
    }
}

// Records are positional: no field tags or defaults. Enum tags start at zero.
fn listing(schema: &Schema) -> String {
    let reached = reachable(schema);
    let mut listing = String::new();
    for declaration in &schema.declarations {
        if !reached.contains(name(declaration)) {
            continue;
        }
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    writeln!(listing, "record {} {} {} tag=- default=-", record.name, field.name, spelling(&field.ty))
                        .expect("writing a schema listing to a string");
                }
            }
            Declaration::Enum(enumeration) => {
                for (tag, variant) in enumeration.variants.iter().enumerate() {
                    writeln!(
                        listing,
                        "enum {} {} tag={tag} payload={}",
                        enumeration.name,
                        variant.name,
                        variant.record.as_deref().unwrap_or("-")
                    )
                    .expect("writing a schema listing to a string");
                }
            }
        }
    }
    listing
}

fn client_schema() -> Schema {
    let mut source = fs::read_to_string(package().join("schema/client-v21.schema")).expect("client schema");
    source.push_str(&fs::read_to_string(package().join("schema/client-v21-part1.schema")).expect("client schema part"));
    parse(&source).expect("client schema valid")
}

#[test]
fn the_public_schema_names_nothing_from_elsewhere() {
    // parse rejects every reference not declared in this schema, including enum payloads.
    let schema = parse(&source()).expect("public schema is closed");
    let private: BTreeSet<_> =
        client_schema().declarations.iter().map(|declaration| name(declaration).to_string()).collect();
    assert_eq!((&*schema.family, schema.version), ("jig-public-wire", crate::PUBLIC_VERSION));
    for declaration in &schema.declarations {
        assert!(!private.contains(name(declaration)), "private declaration {}", name(declaration));
    }
}

#[test]
fn the_fields_golden_is_the_reachable_wire_shape() {
    let schema = parse(&source()).expect("public schema valid");
    let reached = reachable(&schema);
    let declared: BTreeSet<_> = schema.declarations.iter().map(|declaration| name(declaration).to_string()).collect();
    let expected: BTreeSet<_> = [
        "PublicExecutor",
        "PublicPhase",
        "PublicHold",
        "PublicCounts",
        "PublicLine",
        "PublicProject",
        "PublicProjects",
        "PublicGoal",
        "PublicBoard",
        "PublicCard",
        "PublicApplication",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(reached, expected, "all and only the public records and enums are reached");
    assert_eq!(declared, reached, "no unrelated declarations");
    assert_eq!(listing(&schema), golden());
}

fn add_member(source: &str, declaration: &str, member: &str) -> String {
    let schema = parse(source).expect("unmodified schema valid");
    let line = match schema.declarations.iter().find(|item| name(item) == declaration).expect("mutation target") {
        Declaration::Record(record) => record.line,
        Declaration::Enum(enumeration) => enumeration.line,
    };
    let mut changed = String::new();
    for (index, text) in source.lines().enumerate() {
        if index.checked_add(1).expect("schema line fits usize") == line {
            let (before, after) = text.split_once('{').expect("declaration opens on its line");
            changed.push_str(before);
            changed.push('{');
            changed.push_str(member);
            changed.push_str(after);
        } else {
            changed.push_str(text);
        }
        changed.push('\n');
    }
    changed
}

#[test]
fn a_nested_field_added_changes_the_golden() {
    let source = source();
    for (declaration, member) in [("PublicLine", " added: u64 "), ("PublicPhase", " added, ")] {
        let changed = add_member(&source, declaration, member);
        assert_ne!(listing(&parse(&changed).expect("mutated schema valid")), golden(), "mutation in {declaration}");
    }
}

#[test]
fn a_name_from_elsewhere_breaks_the_closure() {
    let changed = add_member(&source(), "PublicLine", " private: MeProject ");
    let error = parse(&changed).expect_err("private reference must break closure");
    assert_eq!(error.message, "type MeProject used before declaration");
}

fn carries_text(ty: &Type) -> bool {
    match ty {
        Type::Text(_) | Type::Bytes(_) | Type::Secret(_) => true,
        Type::List(_, item) | Type::Option(item) => carries_text(item),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Named(_) => false,
    }
}

#[test]
fn no_public_record_carries_text() {
    let schema = parse(&source()).expect("public schema valid");
    let mut fields = BTreeSet::new();
    for declaration in &schema.declarations {
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    if carries_text(&field.ty) {
                        assert!(fields.insert(format!("{}.{}", record.name, field.name)), "unique field");
                    }
                }
            }
            Declaration::Enum(_) => {}
        }
    }
    let expected: BTreeSet<_> =
        ["PublicLine.title", "PublicProject.name", "PublicBoard.name", "PublicApplication.record"]
            .into_iter()
            .map(String::from)
            .collect();
    assert_eq!(fields, expected, "only titles, project names and the application envelope carry text or bytes");
}

fn variants(schema: &Schema, target: &str) -> Vec<String> {
    match schema.declarations.iter().find(|item| name(item) == target).expect("enum exists") {
        Declaration::Enum(enumeration) => enumeration.variants.iter().map(|variant| variant.name.clone()).collect(),
        Declaration::Record(_) => unreachable!("enum target"),
    }
}

#[test]
fn public_phase_and_hold_kinds_match_the_client_contract() {
    let schema = parse(&source()).expect("public schema valid");
    let client = client_schema();
    let phases: Vec<_> = variants(&client, "Phase").into_iter().filter(|phase| phase != "ended").collect();
    assert_eq!(variants(&schema, "PublicPhase"), phases);
    assert_eq!(variants(&schema, "PublicHold"), variants(&client, "HoldCategory"));
    for declaration in &schema.declarations {
        match declaration {
            Declaration::Enum(enumeration) => {
                for variant in &enumeration.variants {
                    assert_eq!(variant.record, None, "public kind {} is unit", variant.name);
                }
            }
            Declaration::Record(_) => {}
        }
    }
}
