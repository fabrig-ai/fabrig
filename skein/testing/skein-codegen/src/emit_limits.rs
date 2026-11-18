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
    let mut names = String::new();
    for declaration in schema.declarations {
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    let variant = format!("{}{}", record.name, pascal(&field.name));
                    out.push_str(&format!(
                        "    /// The `{}` field of `{}`.\n    {variant},\n",
                        field.name, record.name
                    ));
                    names.push_str(&format!("            Self::{variant} => \"{}.{}\",\n", record.name, field.name));
                }
                if record.versioned {
                    out.push_str(&format!("    /// The version of `{}`.\n    {}Version,\n", record.name, record.name));
                    names.push_str(&format!(
                        "            Self::{}Version => \"{} (version)\",\n",
                        record.name, record.name
                    ));
                }
                out.push_str(&format!("    /// Bytes after `{}`.\n    {}Tail,\n", record.name, record.name));
                names.push_str(&format!("            Self::{}Tail => \"{} (tail)\",\n", record.name, record.name));
            }
            Declaration::Enum(enumeration) => {
                out.push_str(&format!("    /// The tag of `{}`.\n    {}Tag,\n", enumeration.name, enumeration.name));
                names.push_str(&format!(
                    "            Self::{}Tag => \"{} (tag)\",\n",
                    enumeration.name, enumeration.name
                ));
            }
        }
    }
    out.push_str("}\n\nimpl Path {\n    /// The path as the schema spells it (skein/docs/design/data/codec/README.md, 4).\n    #[must_use]\n");
    if names.lines().count() > 98 {
        out.push_str(
            "    #[expect(clippy::too_many_lines, reason = \"each schema path has one exhaustive naming arm\")]\n",
        );
    }
    out.push_str("    pub const fn name(self) -> &'static str {\n        match self {\n");
    out.push_str(&names);
    out.push_str("        }\n    }\n}\n\n/// A decoding or bound failure at a schema path.\n#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub struct Problem {\n    pub path: Path,\n    pub reason: skein_codec::Reason,\n}\n\n");
    emit_check(schema, out);
}

fn emit_names(schema: &Context<'_>, out: &mut String) {
    out.push_str("/// The limits' field names, in field order.\npub const LIMIT_NAMES: &[&str] = &[\n");
    for (name, _, _) in &schema.bounds {
        out.push_str(&format!("    \"{name}\",\n"));
    }
    out.push_str("];\n\nimpl Limits {\n    /// The limits' values, in `LIMIT_NAMES` order.\n    #[must_use]\n    pub fn values(&self) -> List<u32> {\n");
    match schema.first_limits() {
        Some(first) => out.push_str(&format!("        let mut values = List::with_capacity(u32::try_from(LIMIT_NAMES.len()).expect(\"schema limits fit u32\"));\n        self.values_for_{}_record(&mut values);\n        values\n", snake(first))),
        None => out.push_str("        List::with_capacity(0)\n"),
    }
    out.push_str("    }\n}\n\n");
}

/// One public family check where its owner makes the limits.
pub(crate) fn emit_check(schema: &Context<'_>, out: &mut String) {
    out.push_str("/// Checks the family limits, naming the first limit past its ceiling.\n");
    match schema.first_limits() {
        Some(first) => out.push_str(&format!("pub fn check_limits(limits: &Limits) -> Result<(), Problem> {{\n    check_{}_record_limits(limits)\n}}\n\n", snake(first))),
        None => out.push_str("pub fn check_limits(_limits: &Limits) -> Result<(), Problem> {\n    Ok(())\n}\n\n"),
    }
}

fn checks(bounds: &[(String, u32, String)], out: &mut String) {
    for (name, _, path) in bounds {
        out.push_str(&format!("    if limits.{name} > CEILINGS.{name} {{ return Err(Problem {{ path: Path::{path}, reason: skein_codec::Reason::Bound }}); }}\n"));
    }
}

fn values(bounds: &[(String, u32, String)], out: &mut String) {
    for (name, _, _) in bounds {
        out.push_str(&format!("        values.push(self.{name}).expect(\"room for every named limit\");\n"));
    }
}

/// A bounded record owns its checks and ordered values, chained to the next record.
pub(crate) fn emit_record(schema: &Context<'_>, record: &crate::Record, out: &mut String) {
    let (bounds, next) = schema.record_limits(&record.name);
    if bounds.is_empty() {
        return;
    }
    let name = snake(&record.name);
    let mut helpers = String::new();
    let split = bounds.len() >= 96;
    let start = out.len();
    out.push_str(&format!("fn check_{name}_record_limits(limits: &Limits) -> Result<(), Problem> {{\n"));
    if split {
        for group in bounds.chunk_by(|left, right| left.2 == right.2) {
            let path = snake(&group.first().expect("nonempty field bounds").2);
            out.push_str(&format!("    check_{path}_field_limits(limits)?;\n"));
            helpers.push_str(&format!("\nfn check_{path}_field_limits(limits: &Limits) -> Result<(), Problem> {{\n"));
            checks(group, &mut helpers);
            helpers.push_str("    Ok(())\n}\n");
        }
    } else {
        checks(bounds, out);
    }
    match next {
        Some(next) => out.push_str(&format!("    check_{}_record_limits(limits)\n}}\n\n", snake(next))),
        None => out.push_str("    Ok(())\n}\n\n"),
    }
    let function = out.split_off(start);
    out.push_str(&crate::emit_functions::pack_dispatches(function, "_field_limits(limits)"));
    out.push_str(&helpers);
    helpers.clear();
    out.push_str("impl Limits {\n");
    let start = out.len();
    out.push_str(&format!("    fn values_for_{name}_record(&self, values: &mut List<u32>) {{\n"));
    if split {
        for group in bounds.chunk_by(|left, right| left.2 == right.2) {
            let path = snake(&group.first().expect("nonempty field bounds").2);
            out.push_str(&format!("        self.values_for_{path}_field(values);\n"));
            helpers.push_str(&format!(
                "\nimpl Limits {{\n    fn values_for_{path}_field(&self, values: &mut List<u32>) {{\n"
            ));
            values(group, &mut helpers);
            helpers.push_str("    }\n}\n");
        }
    } else {
        values(bounds, out);
    }
    if let Some(next) = next {
        out.push_str(&format!("        self.values_for_{}_record(values);\n", snake(next)));
    }
    out.push_str("    }\n");
    let function = out.split_off(start);
    out.push_str(&crate::emit_functions::pack_dispatches(function, "self.values_for_"));
    out.push_str("}\n\n");
    out.push_str(&helpers);
}
