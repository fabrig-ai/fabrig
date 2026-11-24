//! Lean modules (skein/docs/design/data/codec/README.md, 4).

use crate::{
    Declaration, Output, Schema, Type,
    analysis::Context,
    emit_limits::{self, LimitLevel, literal, pascal, snake},
    emit_types,
};
use alloc::collections::BTreeMap;
use core::fmt::Write;

fn line(out: &mut String, text: &str) {
    writeln!(out, "{text}").expect("generated source");
}

fn layout(out: &mut String, digest: [u8; 32]) {
    line(out, "pub const LAYOUT: [u8; 32] = [");
    for chunk in digest.chunks(19) {
        out.push_str("   ");
        for byte in chunk {
            write!(out, " 0x{byte:02x},").expect("generated layout byte");
        }
        out.push('\n');
    }
    line(out, "];\n");
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
    line(out, "// Paths: names as the schema spells them.\npub mod paths {\n    use super::Path;");
    for declaration in &schema.declarations {
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
        for (name, text) in parts {
            line(out, &format!("    pub const {name}: Path = Path::new({text:?});"));
        }
    }
    line(out, "}\n");
}

fn limits(context: &Context<'_>, out: &mut String) {
    let bounds = &context.bounds;
    let copy = if bounds.len() > 2 { ", Copy" } else { "" };
    line(
        out,
        &format!(
            "// Limits: the owner's values, checked once against ceilings.\n#[repr(C)]\n#[derive(Clone{copy}, Debug, PartialEq, Eq, Hash)]\npub struct Limits {{"
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
    line(out, "    #[must_use]\n    pub fn values(&self) -> List<u32> {\n        List::from_boxed([");
    for (name, _, _) in bounds {
        line(out, &format!("            self.{name},"));
    }
    line(out, "        ].into())\n    }\n");
    expectation(out, bounds.len().checked_add(5).expect("source length"));
    // The binding keeps the signature unchanged for a family without limits.
    line(
        out,
        "    #[must_use]\n    pub fn at_most(&self, most: u32) -> Limits {\n        let _: u32 = most;\n        Limits {",
    );
    for (name, _, _) in bounds {
        line(out, &format!("            {name}: self.{name}.min(most),"));
    }
    line(out, "        }\n    }\n}\n");
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
    shape: Shape<'_>,
    encoded_at_zero: u64,
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
    line(out, &format!("// Record: {name}.\nimpl {name} {{"));
    line(out, "    const SHAPE: &'static [skein_codec::bounds::Node] = &[");
    for (node, _) in shape.nodes {
        line(out, &format!("        {node},"));
    }
    line(out, "    ];\n");
    line(
        out,
        &format!(
            "    /// The record's encoded bound with every limit it reaches set to zero.\n    pub const ENCODED_AT_ZERO: u64 = {encoded_at_zero};\n}}\n\nimpl Record for {name} {{\n    type Limits = Limits;\n    const TAIL: Path = paths::{tail};\n"
        ),
    );
    for bound in ["encoded", "decoded"] {
        line(
            out,
            &format!(
                "    fn {bound}_bound(limits: &Limits) -> Option<u64> {{\n        Some(skein_codec::bounds::declaration(DECLARATIONS, limits.values().as_slice(), CEILING_VALUES, {name:?})?.{bound})\n    }}\n"
            ),
        );
    }
    // Compile the outer entry with its family, not with an unoptimized caller.
    // Emit it even for nested types: inserting a user of a type does not change its code.
    line(
        out,
        "    #[cfg(debug_assertions)]\n    fn decode(limits: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {\n        let value = Self::read(limits, reader)?;\n        if !reader.is_empty() {\n            return Err(Self::TAIL.problem(skein_codec::Reason::Trailing));\n        }\n        Ok(value)\n    }\n",
    );
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

struct Shape<'a> {
    nodes: Vec<(String, u64)>,
    named: &'a BTreeMap<&'a str, u64>,
}

enum ZeroBound<'a> {
    Fixed(u64),
    LengthPrefix,
    Option(usize),
    Boxed(usize),
    Record { prefix: u64, fields: &'a [usize] },
    Enum(&'a [usize]),
}

impl Shape<'_> {
    fn bound(&self, at: usize) -> u64 {
        self.nodes.get(at).expect("earlier bounds node").1
    }

    fn encoded_at_zero(&self, node: ZeroBound<'_>) -> Option<u64> {
        match node {
            ZeroBound::Fixed(size) => Some(size),
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
        self.push(format!("skein_codec::bounds::Node::Scalar({})", literal(size)), ZeroBound::Fixed(size))
    }

    fn named(&mut self, name: &str) -> usize {
        self.push(
            format!("skein_codec::bounds::Node::Named({name:?})"),
            ZeroBound::Fixed(*self.named.get(name).expect("earlier declaration")),
        )
    }

    fn ty(&mut self, ty: &Type, bound: &str) -> usize {
        match ty {
            Type::U8 | Type::Bool => self.scalar(1),
            Type::U16 => self.scalar(2),
            Type::U32 => self.scalar(4),
            Type::U64 | Type::Duration => self.scalar(8),
            Type::Fixed(width) => self.scalar(u64::from(*width)),
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => self.push(
                format!("skein_codec::bounds::Node::Bytes {{ limit: core::mem::offset_of!(Limits, {bound}) }}"),
                ZeroBound::LengthPrefix,
            ),
            Type::List(_, item) => {
                let at = self.ty(item, &emit_limits::child_name(bound, LimitLevel::Item));
                self.push(
                    format!(
                        "skein_codec::bounds::Node::List {{ limit: core::mem::offset_of!(Limits, {bound}), item: {at}, size: size_of::<{}>() }}",
                        emit_types::rust_type(item)
                    ),
                    ZeroBound::LengthPrefix,
                )
            }
            Type::Option(item) => {
                let at = self.ty(item, &emit_limits::child_name(bound, LimitLevel::Some));
                self.push(format!("skein_codec::bounds::Node::Option {{ item: {at} }}"), ZeroBound::Option(at))
            }
            Type::Named(name) => self.named(name),
        }
    }
}

fn shape_for<'a>(declaration: &Declaration, named: &'a BTreeMap<&str, u64>) -> (Shape<'a>, u64) {
    let mut shape = Shape { nodes: Vec::new(), named };
    let root = match declaration {
        Declaration::Record(record) => {
            let fields = record
                .fields
                .iter()
                .map(|field| shape.ty(&field.ty, &emit_limits::bound_name(&record.name, &field.name)))
                .collect::<Vec<_>>();
            let prefix = if record.versioned { 2 } else { 0 };
            shape.push(
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
                        let at = shape.named(name);
                        if variant.boxed {
                            shape.push(
                                format!("skein_codec::bounds::Node::Boxed {{ item: {at}, size: size_of::<{name}>() }}"),
                                ZeroBound::Boxed(at),
                            )
                        } else {
                            at
                        }
                    }
                    None => shape.scalar(0),
                })
                .collect::<Vec<_>>();
            shape.push(
                format!("skein_codec::bounds::Node::Enum {{ variants: &{variants:?} }}"),
                ZeroBound::Enum(&variants),
            )
        }
    };
    let encoded_at_zero = shape.bound(root);
    (shape, encoded_at_zero)
}

fn table(context: &Context<'_>, named: &BTreeMap<&str, u64>, out: &mut String) {
    line(out, "// Bounds: one postordered shape per declaration, sorted by name.");
    line(out, "pub static DECLARATIONS: &[skein_codec::bounds::Declaration] = &[");
    for name in named.keys() {
        line(out, &format!("    skein_codec::bounds::Declaration {{ name: {name:?}, nodes: {name}::SHAPE }},"));
    }
    line(out, "];\n\nstatic CEILING_VALUES: &[u32] = &[");
    for (name, _, _) in &context.bounds {
        line(out, &format!("    CEILINGS.{name},"));
    }
    line(out, "];");
    for name in ["encoded", "decoded"] {
        line(
            out,
            &format!(
                "\n#[must_use]\npub fn {name}_bound(limits: &Limits) -> Option<u64> {{\n    Some(skein_codec::bounds::family(DECLARATIONS, limits.values().as_slice(), CEILING_VALUES)?.{name})\n}}"
            ),
        );
    }
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
    line(&mut out, &format!("pub const VERSION: u16 = {};", literal(schema.version)));
    layout(&mut out, schema.layout);
    limits(&context, &mut out);
    paths(schema, &mut out);
    let mut named = BTreeMap::new();
    for declaration in context.declarations {
        types(declaration, &mut out);
        let (shape, encoded_at_zero) = shape_for(declaration, &named);
        implementation(&context, declaration, shape, encoded_at_zero, &mut out);
        named.insert(declaration_name(declaration), encoded_at_zero);
    }
    table(&context, &named, &mut out);
    Output { records: out }
}
