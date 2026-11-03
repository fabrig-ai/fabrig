//! Inspect the author-owned schema tree (temper/docs/design/client/forge.md, 11.4).
#![expect(clippy::disallowed_types, reason = "the schema parser's author-owned tree uses String and Vec")]
#![expect(clippy::disallowed_macros, reason = "the author-owned schema shape is a formatted text golden")]
use alloc::{collections::BTreeSet, string::String};
use core::fmt::Write;
use skein_codegen::{Declaration, Schema, Type};

fn name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Record(record) => &record.name,
        Declaration::Enum(enumeration) => &enumeration.name,
    }
}

fn leaf(mut ty: &Type) -> &Type {
    loop {
        match ty {
            Type::List(_, item) | Type::Option(item) => ty = item,
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Bytes(_)
            | Type::Secret(_)
            | Type::Text(_)
            | Type::Named(_) => return ty,
        }
    }
}

fn references(declaration: &Declaration) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    match declaration {
        Declaration::Record(record) => {
            for field in &record.fields {
                if let Type::Named(name) = leaf(&field.ty) {
                    names.insert(name.clone());
                }
            }
        }
        Declaration::Enum(enumeration) => {
            for variant in &enumeration.variants {
                if let Some(name) = &variant.record {
                    names.insert(name.clone());
                }
            }
        }
    }
    names
}

pub(super) fn closed(source: &str, panels: &str) -> Result<(), String> {
    let schema = match skein_codegen::parse(source) {
        Ok(schema) => schema,
        Err(error) => return Err(alloc::format!("{error}")),
    };
    let panels = skein_codegen::parse(panels).expect("panel schema parses");
    let mut own = BTreeSet::new();
    for declaration in &schema.declarations {
        own.insert(name(declaration));
    }
    for declaration in &schema.declarations {
        for referenced in references(declaration) {
            if !own.contains(referenced.as_str()) {
                return Err(alloc::format!("foreign type {referenced}"));
            }
        }
        for panel in &panels.declarations {
            if let Declaration::Record(panel) = panel
                && name(declaration) == panel.name
            {
                return Err(alloc::format!("panel record {}", panel.name));
            }
        }
    }
    Ok(())
}

fn reachable(schema: &Schema) -> BTreeSet<String> {
    let mut reached =
        BTreeSet::from([String::from("PublicChange"), String::from("PublicLandings"), String::from("PublicLanding")]);
    // The parser requires a type before any use, so one reverse pass closes the roots.
    for declaration in schema.declarations.iter().rev() {
        if reached.contains(name(declaration)) {
            reached.extend(references(declaration));
        }
    }
    reached
}

fn spelling(mut ty: &Type) -> String {
    let mut text = String::new();
    loop {
        match ty {
            Type::List(bound, item) => {
                write!(text, "list {bound} ").expect("string write");
                ty = item;
                continue;
            }
            Type::Option(item) => {
                text.push_str("option ");
                ty = item;
                continue;
            }
            Type::U8 => text.push_str("u8"),
            Type::U16 => text.push_str("u16"),
            Type::U32 => text.push_str("u32"),
            Type::U64 => text.push_str("u64"),
            Type::Bool => text.push_str("bool"),
            Type::Duration => text.push_str("duration"),
            Type::Fixed(bound) => write!(text, "fixed {bound}").expect("string write"),
            Type::Bytes(bound) => write!(text, "bytes {bound}").expect("string write"),
            Type::Secret(bound) => write!(text, "secret {bound}").expect("string write"),
            Type::Text(bound) => write!(text, "text {bound}").expect("string write"),
            Type::Named(name) => text.push_str(name),
        }
        return text;
    }
}

pub(super) fn listing(source: &str) -> String {
    let schema = skein_codegen::parse(source).expect("schema parses");
    let reached = reachable(&schema);
    let mut text = alloc::format!("family {} {}\n", schema.family, schema.version);
    for declaration in &schema.declarations {
        if !reached.contains(name(declaration)) {
            continue;
        }
        match declaration {
            Declaration::Record(record) => {
                writeln!(text, "record {} versioned={}", record.name, record.versioned).expect("string write");
                for field in &record.fields {
                    writeln!(text, "field {} {} type={} tag=- default=-", record.name, field.name, spelling(&field.ty))
                        .expect("string write");
                }
            }
            Declaration::Enum(enumeration) => {
                for (tag, variant) in enumeration.variants.iter().enumerate() {
                    let payload = variant.record.as_deref().unwrap_or("-");
                    let boxed = if variant.boxed { "boxed " } else { "" };
                    writeln!(text, "variant {} {} tag={tag} payload={boxed}{payload}", enumeration.name, variant.name)
                        .expect("string write");
                }
            }
        }
    }
    text
}

pub(super) fn addresses_only(source: &str) {
    let schema = skein_codegen::parse(source).expect("schema parses");
    let addresses = [
        ("PublicChange", "repository"),
        ("PublicChange", "pull"),
        ("PublicQueue", "repository"),
        ("PublicGateProof", "run"),
        ("PublicMember", "repository"),
        ("PublicMember", "pull"),
    ];
    for declaration in &schema.declarations {
        if let Declaration::Record(record) = declaration {
            for field in &record.fields {
                match leaf(&field.ty) {
                    Type::Bytes(bound) | Type::Text(bound) => {
                        assert!(
                            addresses.contains(&(record.name.as_str(), field.name.as_str())),
                            "{}.{}",
                            record.name,
                            field.name
                        );
                        assert_eq!(*bound, crate::ADDRESS_BYTES);
                    }
                    Type::Secret(_) => unreachable!("public schema has no secrets"),
                    Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::Bool
                    | Type::Duration
                    | Type::Fixed(_)
                    | Type::Named(_) => {}
                    Type::List(_, _) | Type::Option(_) => unreachable!("leaf unwraps containers"),
                }
            }
        }
    }
}
