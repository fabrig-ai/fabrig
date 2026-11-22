//! Lean modules (skein/docs/design/data/codec/README.md, 4).

use crate::{
    Declaration, Output, Schema, Type,
    analysis::Context,
    emit_limits::{self, LimitLevel, literal, pascal, snake},
    emit_types,
};
use core::fmt::Write;

fn line(out: &mut String, text: &str) {
    writeln!(out, "{text}").expect("generated source");
}

fn expectation(out: &mut String, lines: usize) {
    if lines > 100 {
        line(out, "    #[expect(clippy::too_many_lines, reason = \"one operation per schema field or limit\")]");
    }
}

fn declaration_name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Record(record) => &record.name,
        Declaration::Enum(enumeration) => &enumeration.name,
    }
}

fn constant(variant: &str) -> String {
    snake(variant).to_ascii_uppercase()
}
fn field_path(record: &str, field: &str) -> String {
    constant(&format!("{record}{}", pascal(field)))
}

fn paths(schema: &Schema, out: &mut String) {
    line(
        out,
        "// Paths: declaration and part positions, with one naming table.\npub mod paths {\n    use super::Path;",
    );
    let mut names = String::new();
    for (at, declaration) in schema.declarations.iter().enumerate() {
        let mut parts = Vec::new();
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    parts.push((field_path(&record.name, &field.name), format!("{}.{}", record.name, field.name)));
                }
                if record.versioned {
                    parts.push((constant(&format!("{}Version", record.name)), format!("{} (version)", record.name)));
                }
                parts.push((constant(&format!("{}Tail", record.name)), format!("{} (tail)", record.name)));
            }
            Declaration::Enum(enumeration) => {
                parts.push((constant(&format!("{}Tag", enumeration.name)), format!("{} (tag)", enumeration.name)));
            }
        }
        line(&mut names, "    &[");
        for (part, (name, text)) in parts.iter().enumerate() {
            line(out, &format!("    pub const {name}: Path = Path::new({at}, {part});"));
            line(&mut names, &format!("        \"{text}\","));
        }
        line(&mut names, "    ],");
    }
    line(out, "}\n\nconst PATH_NAMES: &[&[&str]] = &[");
    out.push_str(&names);
    line(
        out,
        "];\n\n/// The path as the schema spells it, or None outside this family's table.\n#[must_use]\npub fn name(path: Path) -> Option<&'static str> {\n    PATH_NAMES.get(usize::from(path.declaration))?.get(usize::from(path.part)).copied()\n}\n",
    );
}

fn limits(context: &Context<'_>, out: &mut String) {
    let bounds = &context.bounds;
    let copy = if bounds.len() > 2 { ", Copy" } else { "" };
    line(
        out,
        &format!(
            "// Limits: the owner's values, checked once against ceilings.\n#[derive(Clone{copy}, Debug, PartialEq, Eq, Hash)]\npub struct Limits {{"
        ),
    );
    for (name, _, _) in bounds {
        line(out, &format!("    pub {name}: u32,"));
    }
    line(out, "}\n\npub const CEILINGS: Limits = Limits {");
    for (name, ceiling, _) in bounds {
        line(out, &format!("    {name}: {},", literal(ceiling)));
    }
    line(out, "};\n\npub const LIMIT_NAMES: &[&str] = &[");
    for (name, _, _) in bounds {
        line(out, &format!("    \"{name}\","));
    }
    line(out, "];\n\nimpl Limits {");
    expectation(out, bounds.len().checked_add(4).expect("source length"));
    line(out, "    #[must_use]\n    pub fn values(&self) -> List<u32> {");
    if bounds.is_empty() {
        line(out, "        List::with_capacity(0)");
    } else {
        line(
            out,
            "        let mut values = List::with_capacity(u32::try_from(LIMIT_NAMES.len()).expect(\"schema limits fit u32\"));",
        );
        for (name, _, _) in bounds {
            line(out, &format!("        values.push(self.{name}).expect(\"room for every named limit\");"));
        }
        line(out, "        values");
    }
    line(out, "    }\n}\n");
    line(out, "const LIMIT_PATHS: &[Path] = &[");
    for (_, _, path) in bounds {
        line(out, &format!("    paths::{},", constant(path)));
    }
    line(
        out,
        "];\n\npub fn check_limits(limits: &Limits) -> Result<(), Problem> {\n    for ((value, ceiling), path) in limits.values().as_slice().iter().zip(CEILING_VALUES).zip(LIMIT_PATHS) {\n        if value > ceiling {\n            return Err(path.problem(skein_codec::Reason::Bound));\n        }\n    }\n    Ok(())\n}\n",
    );
}

fn shape(context: &Context<'_>, ty: &Type, bound: &str) -> String {
    let limit = format!("limits.{bound}.min(CEILINGS.{bound})");
    match ty {
        Type::U8 => "field::U8".into(),
        Type::U16 => "field::U16".into(),
        Type::U32 => "field::U32".into(),
        Type::U64 => "field::U64".into(),
        Type::Bool => "field::Boolean".into(),
        Type::Duration => "field::Span".into(),
        Type::Fixed(width) => format!("field::Fixed::<{}>", literal(width)),
        Type::Bytes(_) => format!("field::Bytes {{ limit: {limit} }}"),
        Type::Secret(_) => format!("field::Hidden {{ limit: {limit} }}"),
        Type::Text(_) => format!("field::Text {{ limit: {limit} }}"),
        Type::List(_, item) => format!(
            "field::Sequence {{ limit: {limit}, minimum: {}, item: {} }}",
            literal(context.min_bytes(item)),
            shape(context, item, &emit_limits::child_name(bound, LimitLevel::Item))
        ),
        Type::Option(item) => format!(
            "field::Optional {{ item: {} }}",
            shape(context, item, &emit_limits::child_name(bound, LimitLevel::Some))
        ),
        Type::Named(_) => "field::Nested".into(),
    }
}

fn types(declaration: &Declaration, out: &mut String) {
    let name = declaration_name(declaration);
    line(out, &format!("// Type: {name}.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]"));
    match declaration {
        Declaration::Record(record) => {
            if emit_types::excessive_bools(record) {
                line(
                    out,
                    "#[expect(clippy::struct_excessive_bools, reason = \"the schema names each boolean independently\")]",
                );
            }
            line(out, &format!("pub struct {name} {{"));
            for field in &record.fields {
                line(out, &format!("    pub {}: {},", field.name, emit_types::rust_type(&field.ty)));
            }
        }
        Declaration::Enum(enumeration) => {
            line(out, &format!("pub enum {name} {{"));
            for variant in &enumeration.variants {
                let name = pascal(&variant.name);
                let payload = variant.record.as_ref().map_or_else(String::new, |record| {
                    if variant.boxed { format!("(Box<{record}>)") } else { format!("({record})") }
                });
                line(out, &format!("    {name}{payload},"));
            }
        }
    }
    line(out, "}\n");
}

fn implementation(
    context: &Context<'_>,
    declaration: &Declaration,
    node: usize,
    encoded_at_zero: u64,
    root: bool,
    out: &mut String,
) {
    let name = declaration_name(declaration);
    let tail = constant(&format!(
        "{name}{}",
        match declaration {
            Declaration::Record(_) => "Tail",
            Declaration::Enum(_) => "Tag",
        }
    ));
    line(
        out,
        &format!(
            "// Record: {name}.\nimpl {name} {{\n    /// The record's encoded bound with every limit it reaches set to zero.\n    pub const ENCODED_AT_ZERO: u64 = {encoded_at_zero};\n}}\n\nimpl Record for {name} {{\n    type Limits = Limits;\n    const TAIL: Path = paths::{tail};\n\n    fn encoded_bound(limits: &Limits) -> Option<u64> {{\n        Some(skein_codec::bounds::declaration(TABLE, limits.values().as_slice(), CEILING_VALUES, {node})?.encoded)\n    }}\n\n    fn decoded_bound(limits: &Limits) -> Option<u64> {{\n        Some(skein_codec::bounds::declaration(TABLE, limits.values().as_slice(), CEILING_VALUES, {node})?.decoded)\n    }}\n"
        ),
    );
    if root {
        // Compile the outer entry with its family, not with an unoptimized caller.
        // Release uses Record's default entry.
        line(
            out,
            "    #[cfg(debug_assertions)]\n    fn decode(limits: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {\n        let value = Self::read(limits, reader)?;\n        if !reader.is_empty() {\n            return Err(Self::TAIL.problem(skein_codec::Reason::Trailing));\n        }\n        Ok(value)\n    }\n",
        );
    }
    match declaration {
        Declaration::Record(record) => record_impl(context, record, out),
        Declaration::Enum(enumeration) => enum_impl(enumeration, out),
    }
    line(out, "}\n");
}

fn record_impl(context: &Context<'_>, record: &crate::Record, out: &mut String) {
    let arg = if record.fields.is_empty() { "_limits" } else { "limits" };
    let writer = if record.fields.is_empty() && !record.versioned { "_out" } else { "out" };
    expectation(out, record.fields.len().checked_add(3).expect("source length"));
    line(
        out,
        &format!("    fn write(&self, {arg}: &Limits, {writer}: &mut skein_lib::Writer) -> Result<(), Problem> {{"),
    );
    if record.versioned {
        line(
            out,
            &format!(
                "        field::put(out, &VERSION.to_be_bytes(), paths::{})?;",
                constant(&format!("{}Version", record.name))
            ),
        );
    }
    for field in &record.fields {
        let shape = shape(context, &field.ty, &emit_limits::bound_name(&record.name, &field.name));
        line(
            out,
            &format!(
                "        field::write({shape}, &self.{}, {arg}, out, paths::{})?;",
                field.name,
                field_path(&record.name, &field.name)
            ),
        );
    }
    line(out, "        Ok(())\n    }\n");
    expectation(out, record.fields.len().checked_add(4).expect("source length"));
    let reader = if record.fields.is_empty() && !record.versioned { "_reader" } else { "reader" };
    line(
        out,
        &format!("    fn read({arg}: &Limits, {reader}: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {{"),
    );
    if record.versioned {
        line(
            out,
            &format!(
                "        field::version(reader, VERSION, paths::{})?;",
                constant(&format!("{}Version", record.name))
            ),
        );
    }
    line(out, "        Ok(Self {");
    for field in &record.fields {
        let shape = shape(context, &field.ty, &emit_limits::bound_name(&record.name, &field.name));
        line(
            out,
            &format!(
                "            {}: field::read({shape}, {arg}, reader, paths::{})?,",
                field.name,
                field_path(&record.name, &field.name)
            ),
        );
    }
    line(out, "        })\n    }");
}

fn enum_impl(enumeration: &crate::Enumeration, out: &mut String) {
    let path = constant(&format!("{}Tag", enumeration.name));
    let arg = if enumeration.variants.iter().any(|variant| variant.record.is_some()) { "limits" } else { "_limits" };
    expectation(out, enumeration.variants.len().checked_add(3).expect("source length"));
    line(
        out,
        &format!(
            "    fn write(&self, {arg}: &Limits, out: &mut skein_lib::Writer) -> Result<(), Problem> {{\n        match self {{"
        ),
    );
    for (tag, variant) in enumeration.variants.iter().enumerate() {
        let name = pascal(&variant.name);
        if variant.record.is_some() {
            line(
                out,
                &format!(
                    "            Self::{name}(value) => {{ field::put(out, &[{tag}], paths::{path})?; value.write(limits, out) }},"
                ),
            );
        } else {
            line(out, &format!("            Self::{name} => field::put(out, &[{tag}], paths::{path}),"));
        }
    }
    line(out, "        }\n    }\n");
    expectation(out, enumeration.variants.len().checked_add(5).expect("source length"));
    line(
        out,
        &format!(
            "    fn read({arg}: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {{\n        let tag = reader.u8().ok_or(Problem {{ path: paths::{path}, reason: skein_codec::Reason::Short }})?;\n        match tag {{"
        ),
    );
    for (tag, variant) in enumeration.variants.iter().enumerate() {
        let name = pascal(&variant.name);
        let payload = variant.record.as_ref().map_or_else(String::new, |record| {
            if variant.boxed {
                format!("(Box::new({record}::read(limits, reader)?))")
            } else {
                format!("({record}::read(limits, reader)?)")
            }
        });
        line(out, &format!("            {tag} => Ok(Self::{name}{payload}),"));
    }
    if enumeration.variants.len() < 256 {
        line(
            out,
            &format!(
                "            {}..=u8::MAX => Err(Problem {{ path: paths::{path}, reason: skein_codec::Reason::Tag }}),",
                enumeration.variants.len()
            ),
        );
    }
    line(out, "        }\n    }");
}

struct Table<'a> {
    nodes: Vec<(String, u64)>,
    named: alloc::collections::BTreeMap<&'a str, usize>,
    limits: alloc::collections::BTreeMap<&'a str, usize>,
}

enum ZeroBound<'a> {
    Scalar(u64),
    LengthPrefix,
    Option(usize),
    Boxed(usize),
    Record { prefix: u64, fields: &'a [usize] },
    Enum(&'a [usize]),
}

impl Table<'_> {
    fn bound(&self, at: usize) -> u64 {
        self.nodes.get(at).expect("earlier bounds node").1
    }

    fn encoded_at_zero(&self, node: ZeroBound<'_>) -> Option<u64> {
        match node {
            ZeroBound::Scalar(size) => Some(size),
            ZeroBound::LengthPrefix => Some(4),
            ZeroBound::Option(item) => 1_u64.checked_add(self.bound(item)),
            ZeroBound::Boxed(item) => Some(self.bound(item)),
            ZeroBound::Record { prefix, fields } => {
                fields.iter().try_fold(prefix, |sum, field| sum.checked_add(self.bound(*field)))
            }
            ZeroBound::Enum(variants) => {
                1_u64.checked_add(variants.iter().map(|variant| self.bound(*variant)).max().unwrap_or(0))
            }
        }
    }

    fn push(&mut self, text: String, node: ZeroBound<'_>) -> usize {
        let at = self.nodes.len();
        let encoded_at_zero = self.encoded_at_zero(node).expect("a schema's encoded bound at zero limits fits u64");
        self.nodes.push((text, encoded_at_zero));
        at
    }

    fn scalar(&mut self, size: u64) -> usize {
        self.push(format!("skein_codec::bounds::Node::Scalar({})", literal(size)), ZeroBound::Scalar(size))
    }

    fn ty(&mut self, ty: &Type, bound: &str) -> usize {
        match ty {
            Type::U8 | Type::Bool => self.scalar(1),
            Type::U16 => self.scalar(2),
            Type::U32 => self.scalar(4),
            Type::U64 | Type::Duration => self.scalar(8),
            Type::Fixed(width) => self.scalar(u64::from(*width)),
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => self.push(
                format!(
                    "skein_codec::bounds::Node::Bytes {{ limit: {} }}",
                    self.limits.get(bound).expect("limit collected")
                ),
                ZeroBound::LengthPrefix,
            ),
            Type::List(_, item) => {
                let at = self.ty(item, &emit_limits::child_name(bound, LimitLevel::Item));
                self.push(
                    format!(
                        "skein_codec::bounds::Node::List {{ limit: {}, item: {at}, size: size_of::<{}>() }}",
                        self.limits.get(bound).expect("limit collected"),
                        emit_types::rust_type(item)
                    ),
                    ZeroBound::LengthPrefix,
                )
            }
            Type::Option(item) => {
                let at = self.ty(item, &emit_limits::child_name(bound, LimitLevel::Some));
                self.push(format!("skein_codec::bounds::Node::Option {{ item: {at} }}"), ZeroBound::Option(at))
            }
            Type::Named(name) => *self.named.get(name.as_str()).expect("earlier declaration"),
        }
    }
}

fn table(context: &Context<'_>, out: &mut String) -> Vec<(usize, u64)> {
    let mut table = Table {
        nodes: Vec::new(),
        named: alloc::collections::BTreeMap::new(),
        limits: context.bounds.iter().enumerate().map(|(at, (name, _, _))| (name.as_str(), at)).collect(),
    };
    let mut roots = Vec::new();
    for declaration in context.declarations {
        let at = match declaration {
            Declaration::Record(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| table.ty(&field.ty, &emit_limits::bound_name(&record.name, &field.name)))
                    .collect::<Vec<_>>();
                let prefix = if record.versioned { 2 } else { 0 };
                table.push(
                    format!("skein_codec::bounds::Node::Record {{ prefix: {prefix}, fields: &{fields:?} }}"),
                    ZeroBound::Record { prefix, fields: &fields },
                )
            }
            Declaration::Enum(enumeration) => {
                let variants = enumeration
                    .variants
                    .iter()
                    .map(|variant| match &variant.record {
                        Some(name) => {
                            let at = *table.named.get(name.as_str()).expect("earlier record");
                            if variant.boxed {
                                table.push(
                                    format!(
                                        "skein_codec::bounds::Node::Boxed {{ item: {at}, size: size_of::<{name}>() }}"
                                    ),
                                    ZeroBound::Boxed(at),
                                )
                            } else {
                                at
                            }
                        }
                        None => table.scalar(0),
                    })
                    .collect::<Vec<_>>();
                table.push(
                    format!("skein_codec::bounds::Node::Enum {{ variants: &{variants:?} }}"),
                    ZeroBound::Enum(&variants),
                )
            }
        };
        table.named.insert(declaration_name(declaration), at);
        roots.push((at, table.bound(at)));
    }
    line(
        out,
        "// Table: postordered shapes, with compiler-known heap slot sizes.\nstatic TABLE: &[skein_codec::bounds::Node] = &[",
    );
    for (node, _) in table.nodes {
        line(out, &format!("    {node},"));
    }
    line(out, "];\n\nconst DECLARATIONS: &[u32] = &[");
    for (at, _) in &roots {
        line(out, &format!("    {at},"));
    }
    line(out, "];\n\nstatic CEILING_VALUES: &[u32] = &[");
    for (_, ceiling, _) in &context.bounds {
        line(out, &format!("    {},", literal(ceiling)));
    }
    line(out, "];\n");
    for name in ["encoded", "decoded"] {
        line(
            out,
            &format!(
                "#[must_use]\npub fn {name}_bound(limits: &Limits) -> Option<u64> {{\n    Some(skein_codec::bounds::family(TABLE, limits.values().as_slice(), CEILING_VALUES, DECLARATIONS)?.{name})\n}}\n"
            ),
        );
    }
    roots
}

pub(crate) fn generate(schema: &Schema) -> Output {
    let context = Context::new(schema);
    let mut out = format!("// Generated by skein-codegen from {} {}. Do not edit.\n\n", schema.family, schema.version);
    // Box is also used by Record's default encoding, but only field and variant
    // types need an import here.
    let needs_box = schema.declarations.iter().any(|declaration| match declaration {
        Declaration::Record(record) => {
            record.fields.iter().any(|field| emit_types::rust_type(&field.ty).contains("Box<"))
        }
        Declaration::Enum(enumeration) => enumeration.variants.iter().any(|variant| variant.boxed),
    });
    if needs_box {
        line(&mut out, "use alloc::boxed::Box;");
    }
    line(
        &mut out,
        "use skein_lib::List;\nuse skein_codec::field;\npub use skein_codec::Record;\npub type Path = skein_codec::Path<Limits>;\npub type Problem = skein_codec::Problem<Limits>;\n",
    );
    line(&mut out, &format!("pub const VERSION: u16 = {};\n", literal(schema.version)));
    limits(&context, &mut out);
    paths(schema, &mut out);
    let nodes = table(&context, &mut out);
    let roots = crate::roots(schema);
    for (declaration, (node, encoded_at_zero)) in schema.declarations.iter().zip(nodes) {
        types(declaration, &mut out);
        implementation(
            &context,
            declaration,
            node,
            encoded_at_zero,
            roots.iter().any(|root| root == declaration_name(declaration)),
            &mut out,
        );
    }
    Output { records: out }
}
