#![expect(clippy::format_push_string, reason = "generator assembles reviewed source fragments")]

//! Limits and typed field paths for data/codec/README.md, section 4.

use crate::{Declaration, analysis::Context};
pub(crate) use skein_codec::schema::LimitLevel;

pub(crate) fn bound_name(record: &str, field: &str) -> String {
    String::from_utf8(skein_codec::schema::limit_name(record.as_bytes(), field.as_bytes()).into_vec())
        .expect("schema identifiers are ASCII")
}

pub(crate) fn child_name(parent: &str, level: LimitLevel) -> String {
    String::from_utf8(skein_codec::schema::limit_child(parent.as_bytes(), level).into_vec())
        .expect("schema identifiers are ASCII")
}

/// Lowers capitals into words separated by underscores, mapping dots and hyphens to underscores.
#[must_use]
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(if character == '.' || character == '-' { '_' } else { character.to_ascii_lowercase() });
    }
    out
}

/// Uppercases the first character and each character after an underscore, removing underscores.
#[must_use]
pub fn pascal(name: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for character in name.chars() {
        if character == '_' {
            upper = true;
        } else if upper {
            out.push(character.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(character);
        }
    }
    out
}

pub(crate) fn literal(value: impl ToString) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && digits.len().checked_sub(index).expect("index within digits").is_multiple_of(3) {
            grouped.push('_');
        }
        grouped.push(digit);
    }
    grouped
}

pub(crate) fn emit(schema: &Context<'_>, out: &mut String) {
    let bounds = &schema.bounds;
    if bounds.is_empty() {
        out.push_str("/// The family has no adjustable bounds.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]\npub struct Limits {\n");
    } else if bounds.len() <= 2 {
        out.push_str("/// The family's adjustable limits, each no larger than its ceiling.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]\npub struct Limits {\n");
    } else {
        out.push_str("/// The family's adjustable limits, each no larger than its ceiling.\n#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub struct Limits {\n");
    }
    for (name, _, _) in bounds {
        out.push_str(&format!("    pub {name}: u32,\n"));
    }
    out.push_str("}\n\n/// The schema's maximum limits.\npub const CEILINGS: Limits = Limits {\n");
    for (name, bound, _) in bounds {
        out.push_str(&format!("    {name}: {},\n", literal(*bound)));
    }
    out.push_str("};\n\n");
    emit_names(schema, out);
    out.push_str("/// A field or tag that caused a codec problem.\n#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum Path {\n");
    for declaration in schema.declarations {
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    let variant = format!("{}{}", record.name, pascal(&field.name));
                    out.push_str(&format!(
                        "    /// The `{}` field of `{}`.\n    {variant},\n",
                        field.name, record.name
                    ));
                }
                if record.versioned {
                    out.push_str(&format!("    /// The version of `{}`.\n    {}Version,\n", record.name, record.name));
                }
                out.push_str(&format!("    /// Bytes after `{}`.\n    {}Tail,\n", record.name, record.name));
            }
            Declaration::Enum(enumeration) => {
                out.push_str(&format!("    /// The tag of `{}`.\n    {}Tag,\n", enumeration.name, enumeration.name));
            }
        }
    }
    out.push_str("}\n\n/// A decoding or bound failure at a schema path.\n#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub struct Problem {\n    pub path: Path,\n    pub reason: skein_codec::Reason,\n}\n\n");
    emit_check(schema, out);
}

fn emit_names(schema: &Context<'_>, out: &mut String) {
    let count = schema.bounds.len();
    out.push_str(&format!(
        "/// The limits' field names, in field order.\npub const LIMIT_NAMES: [&str; {count}] = [\n"
    ));
    for (name, _, _) in &schema.bounds {
        out.push_str(&format!("    \"{name}\",\n"));
    }
    out.push_str("];\n\nimpl Limits {\n    /// The limits' values, in `LIMIT_NAMES` order.\n    #[must_use]\n");
    out.push_str(&format!("    pub fn values(&self) -> [u32; {count}] {{\n        [\n"));
    for chunk in schema.bounds.chunks(8) {
        let values = chunk.iter().map(|(name, _, _)| format!("self.{name}")).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("            {values},\n"));
    }
    out.push_str("        ]\n    }\n}\n\n");
}

/// One public family check where its owner makes the limits.
pub(crate) fn emit_check(schema: &Context<'_>, out: &mut String) {
    out.push_str("/// Checks the family limits, naming the first limit past its ceiling.\n");
    let limits = if schema.bounds.is_empty() { "_limits" } else { "limits" };
    out.push_str(&format!("pub fn check_limits({limits}: &Limits) -> Result<(), Problem> {{\n"));
    for (index, _) in schema.bounds.chunks(90).enumerate() {
        out.push_str(&format!("    check_limits_{index}(limits)?;\n"));
    }
    out.push_str("    Ok(())\n}\n\n");
    for (index, chunk) in schema.bounds.chunks(90).enumerate() {
        out.push_str(&format!("fn check_limits_{index}(limits: &Limits) -> Result<(), Problem> {{\n"));
        for (name, _, path) in chunk {
            out.push_str(&format!("    if limits.{name} > CEILINGS.{name} {{ return Err(Problem {{ path: Path::{path}, reason: skein_codec::Reason::Bound }}); }}\n"));
        }
        out.push_str("    Ok(())\n}\n\n");
    }
}
