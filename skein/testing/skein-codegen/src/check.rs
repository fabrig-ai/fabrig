//! Names required by generated Rust (data/codec/README.md, section 7).

use alloc::collections::BTreeSet;

use crate::{Declaration, Enumeration, Record, Schema, emit_limits, parse::Error};

fn error(line: usize, message: impl Into<String>) -> Error {
    Error { line, message: message.into() }
}

fn keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "macro_rules"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "union"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

fn valid_type_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters.next().is_some_and(|character| character.is_ascii_uppercase())
        && characters.all(|character| character.is_ascii_alphanumeric())
        && !keyword(name)
}

fn valid_member_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters.next().is_some_and(|character| character.is_ascii_lowercase())
        && characters.all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        && !keyword(name)
}

fn reserved_field_name(name: &str) -> bool {
    matches!(
        name,
        "new"
            | "skein_bools"
            | "check"
            | "measure"
            | "encode"
            | "decode"
            | "decode_from"
            | "into_parts"
            | "worst_case_bytes"
            | "worst_case_heap"
            | "worst_case_bytes_within"
            | "worst_case_heap_within"
            | "encoded_bound"
            | "decoded_bound"
            | "encoded_bound_within"
            | "decoded_bound_within"
    )
}

fn field_line_for_path(schema: &Schema, path: &str) -> usize {
    for declaration in &schema.declarations {
        if let Declaration::Record(record) = declaration {
            for field in &record.fields {
                if format!("{}{}", record.name, emit_limits::pascal(&field.name)) == path {
                    return field.line;
                }
            }
        }
    }
    unreachable!("each bound path belongs to a checked field")
}

fn path_position(position: usize, line: usize, part: &str) -> Result<(), Error> {
    if u16::try_from(position).is_err() {
        return Err(error(line, format!("{part} position exceeds a path's u16")));
    }
    Ok(())
}

fn record_positions(fields: usize, versioned: bool, line: usize) -> Result<(), Error> {
    if let Some(last) = fields.checked_sub(1) {
        path_position(last, line, "field")?;
    }
    if versioned {
        path_position(fields, line, "version")?;
    }
    let tail =
        fields.checked_add(usize::from(versioned)).ok_or_else(|| error(line, "tail position overflows usize"))?;
    path_position(tail, line, "tail")
}

fn record_names(record: &Record) -> Result<(), Error> {
    record_positions(record.fields.len(), record.versioned, record.line)?;
    for field in &record.fields {
        if !valid_member_name(&field.name) || reserved_field_name(&field.name) {
            return Err(error(field.line, format!("invalid generated field name {}", field.name)));
        }
    }
    Ok(())
}

fn enum_names(enumeration: &Enumeration) -> Result<(), Error> {
    let mut emitted_variants = BTreeSet::new();
    for variant in &enumeration.variants {
        if !valid_member_name(&variant.name) {
            return Err(error(variant.line, format!("invalid generated variant name {}", variant.name)));
        }
        let emitted = emit_limits::pascal(&variant.name);
        if !emitted_variants.insert(emitted.clone()) {
            return Err(error(variant.line, format!("generated variant {emitted} is used twice")));
        }
    }
    Ok(())
}

pub(crate) fn check(schema: &Schema) -> Result<(), Error> {
    let mut emitted_types = BTreeSet::from([
        "Box".to_string(),
        "Limits".to_string(),
        "List".to_string(),
        "Option".to_string(),
        "Path".to_string(),
        "Problem".to_string(),
        "Record".to_string(),
        "Result".to_string(),
    ]);
    let mut paths = BTreeSet::new();
    for (position, declaration) in schema.declarations.iter().enumerate() {
        let (name, line) = match declaration {
            Declaration::Record(record) => {
                record_names(record)?;
                (&record.name, record.line)
            }
            Declaration::Enum(enumeration) => {
                enum_names(enumeration)?;
                (&enumeration.name, enumeration.line)
            }
        };
        path_position(position, line, "declaration")?;
        if !valid_type_name(name) {
            return Err(error(line, format!("invalid generated type name {name}")));
        }
        if !emitted_types.insert(name.clone()) {
            return Err(error(line, format!("generated type name {name} is used twice")));
        }
        match declaration {
            Declaration::Record(record) => {
                let parts = format!("{}Parts", record.name);
                if !emitted_types.insert(parts.clone()) {
                    return Err(error(record.line, format!("generated type name {parts} is used twice")));
                }
                for field in &record.fields {
                    let path = format!("{}{}", record.name, emit_limits::pascal(&field.name));
                    if !paths.insert(path.clone()) {
                        return Err(error(field.line, format!("generated path {path} is used twice")));
                    }
                }
                let tail = format!("{}Tail", record.name);
                if !paths.insert(tail.clone()) {
                    return Err(error(record.line, format!("generated path {tail} is used twice")));
                }
                if record.versioned {
                    let version = format!("{}Version", record.name);
                    if !paths.insert(version.clone()) {
                        return Err(error(record.line, format!("generated path {version} is used twice")));
                    }
                }
            }
            Declaration::Enum(enumeration) => {
                let path = format!("{}Tag", enumeration.name);
                if !paths.insert(path.clone()) {
                    return Err(error(enumeration.line, format!("generated path {path} is used twice")));
                }
            }
        }
    }
    let mut bound_names = BTreeSet::new();
    for (name, _, path) in crate::analysis::Context::new(schema).bounds {
        if !bound_names.insert(name.clone()) {
            return Err(error(
                field_line_for_path(schema, &path),
                format!("generated limit name {name} is used twice"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{path_position, record_positions};

    #[test]
    fn codec_surface_names_are_reserved_in_every_family() {
        for name in ["Record", "Path", "Problem"] {
            for declaration in [format!("versioned record {name} {{}}"), format!("enum {name} {{ unit }}")] {
                let error = crate::parse(&format!("family names 1\n{declaration}"))
                    .expect_err("the codec's type names are reserved");
                assert_eq!(error.line, 2);
                assert_eq!(error.message, format!("generated type name {name} is used twice"));
            }
        }
    }

    #[test]
    fn declaration_positions_fit_a_path() {
        assert_eq!(path_position(65_535, 7, "declaration"), Ok(()));
        let error = path_position(65_536, 7, "declaration").expect_err("declaration past u16");
        assert_eq!(error.line, 7);
        assert_eq!(error.message, "declaration position exceeds a path's u16");
    }

    #[test]
    fn field_version_and_tail_positions_fit_a_path() {
        assert_eq!(record_positions(65_535, false, 8), Ok(()));
        assert_eq!(record_positions(65_534, true, 8), Ok(()));
        for (fields, versioned, part) in
            [(65_537, false, "field"), (65_536, true, "version"), (65_536, false, "tail"), (65_535, true, "tail")]
        {
            let error = record_positions(fields, versioned, 8).expect_err("part past u16");
            assert_eq!(error.line, 8);
            assert_eq!(error.message, format!("{part} position exceeds a path's u16"));
        }
    }
}
