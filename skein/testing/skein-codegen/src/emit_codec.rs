#![expect(clippy::format_push_string, reason = "generator assembles reviewed source fragments")]

//! Measured, canonical wire operations for data/codec/README.md, section 4.

use crate::emit_functions::{self, Member, Method, Statement, Tail};
use crate::{Declaration, Enumeration, Record, Type, analysis::Context, emit_limits, emit_types};

fn member(record: &Record, name: &str) -> String {
    match emit_types::bool_index(record, name) {
        Some(_) => format!("self.{name}()"),
        None => format!("self.{name}"),
    }
}

fn add_size(size: &str, out: &mut Vec<String>) {
    out.push(format!("        size = size.checked_add({size}).expect(\"schema ceilings fit u32\");\n"));
}

fn measure_type(ty: &Type, schema: &Context<'_>, value: &str, depth: usize, out: &mut Vec<String>) {
    match ty {
        Type::U8 | Type::Bool => add_size("1", out),
        Type::U16 => add_size("2", out),
        Type::U32 => add_size("4", out),
        Type::U64 | Type::Duration => add_size("8", out),
        Type::Fixed(bound) => add_size(&emit_limits::literal(*bound), out),
        Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => {
            add_size("4", out);
            add_size(&format!("u32::try_from({value}.len()).expect(\"field ceiling fits u32\")"), out);
        }
        Type::List(_, item) => {
            add_size("4", out);
            match schema.fixed_bytes(item) {
                Some(fixed) => add_size(
                    &format!(
                        "{value}.len().checked_mul({}).expect(\"schema ceilings fit u32\")",
                        emit_limits::literal(fixed)
                    ),
                    out,
                ),
                None => {
                    let item_name = format!("item_{depth}");
                    let mut body = Vec::new();
                    measure_type(item, schema, &item_name, depth.checked_add(1).expect("schema depth"), &mut body);
                    out.push(format!(
                        "        for {item_name} in {value}.as_slice() {{\n{}        }}\n",
                        body.concat()
                    ));
                }
            }
        }
        Type::Option(item) => {
            add_size("1", out);
            let item_name = format!("some_{depth}");
            let head = if schema.fixed_bytes(item).is_some() {
                format!("if {value}.is_some()")
            } else {
                format!("if let Some({item_name}) = {value}")
            };
            let mut body = Vec::new();
            measure_type(item, schema, &item_name, depth.checked_add(1).expect("schema depth"), &mut body);
            out.push(format!("        {head} {{\n{}        }}\n", body.concat()));
        }
        Type::Named(_) => match schema.fixed_bytes(ty) {
            Some(fixed) => add_size(&emit_limits::literal(fixed), out),
            None => add_size(&format!("{value}.measure()"), out),
        },
    }
}

fn encode_type(ty: &Type, value: &str, depth: usize, out: &mut Vec<String>) {
    match ty {
        Type::U8 => out.push(format!("        writer.put(&[*{value}])?;\n")),
        Type::U16 | Type::U32 | Type::U64 => {
            out.push(format!("        writer.put(&{value}.to_be_bytes())?;\n"));
        }
        Type::Bool => out.push(format!("        writer.put(&[u8::from(*{value})])?;\n")),
        Type::Duration => out.push(format!("        writer.put(&{value}.as_nanos().to_be_bytes())?;\n")),
        Type::Fixed(_) => out.push(format!("        writer.put({value})?;\n")),
        Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => {
            out.push(format!(
                "        writer.put(&u32::try_from({value}.len()).expect(\"field ceiling fits u32\").to_be_bytes())?;\n"
            ));
            let bytes =
                if matches!(ty, Type::Secret(_)) { format!("{value}.expose()") } else { format!("{value}.as_ref()") };
            out.push(format!("        writer.put({bytes})?;\n"));
        }
        Type::List(_, item) => {
            out.push(format!("        writer.put(&{value}.len().to_be_bytes())?;\n"));
            let item_name = format!("item_{depth}");
            let mut body = Vec::new();
            encode_type(item, &item_name, depth.checked_add(1).expect("schema depth"), &mut body);
            out.push(format!("        for {item_name} in {value}.as_slice() {{\n{}        }}\n", body.concat()));
        }
        Type::Option(item) => {
            let item_name = format!("some_{depth}");
            let mut body = Vec::new();
            encode_type(item, &item_name, depth.checked_add(1).expect("schema depth"), &mut body);
            out.push(format!("        match {value} {{\n            Some({item_name}) => {{\n                writer.put(&[1_u8])?;\n{}            }}\n            None => writer.put(&[0_u8])?,\n        }}\n", body.concat()));
        }
        Type::Named(_) => out.push(format!("        {value}.encode(writer)?;\n")),
    }
}

fn short(path: &str) -> String {
    format!("Problem {{ path: Path::{path}, reason: skein_codec::Reason::Short }}")
}

fn empty_unversioned_record(schema: &Context<'_>, name: &str) -> bool {
    match schema.declaration(name) {
        Declaration::Record(record) => record.fields.is_empty() && !record.versioned,
        Declaration::Enum(_) => false,
    }
}

fn decode_type(ty: &Type, schema: &Context<'_>, bound_name: &str, path: &str, depth: usize) -> String {
    let short = short(path);
    match ty {
        Type::U8 => format!("reader.u8().ok_or({short})?"),
        Type::U16 => format!("reader.u16().ok_or({short})?"),
        Type::U32 => format!("reader.u32().ok_or({short})?"),
        Type::U64 => format!("reader.u64().ok_or({short})?"),
        Type::Bool => format!(
            "match reader.u8().ok_or({short})? {{ 0 => false, 1 => true, _ => return Err(Problem {{ path: Path::{path}, reason: skein_codec::Reason::Bool }}) }}"
        ),
        Type::Duration => format!("skein_lib::Duration::from_nanos(reader.u64().ok_or({short})?)"),
        Type::Fixed(bound) => format!(
            "{{ let bytes = reader.bytes({}).ok_or({short})?; bytes.try_into().expect(\"fixed length checked\") }}",
            emit_limits::literal(*bound)
        ),
        Type::Secret(_) => {
            format!("skein_lib::Secret::new({})", decode_type(&Type::Bytes(0), schema, bound_name, path, depth))
        }
        Type::Bytes(_) => format!(
            "{{ let length = match skein_codec::read_len(reader, limits.{bound_name}.min(CEILINGS.{bound_name})) {{ Ok(length) => length, Err(reason) => return Err(Problem {{ path: Path::{path}, reason }}), }}; Box::from(reader.bytes(length).ok_or({short})?) }}"
        ),
        Type::Text(_) => format!(
            "match skein_codec::read_text(reader, limits.{bound_name}.min(CEILINGS.{bound_name})) {{ Ok(text) => text, Err(reason) => return Err(Problem {{ path: Path::{path}, reason }}), }}"
        ),
        Type::List(_, item) => {
            let item_expr = decode_type(
                item,
                schema,
                &emit_limits::child_name(bound_name, emit_limits::LimitLevel::Item),
                path,
                depth.checked_add(1).expect("schema depth"),
            );
            let list_name = format!("items_{depth}");
            let minimum = schema.min_bytes(item);
            let precheck = match minimum {
                0 => String::new(),
                1 => format!("if count > reader.remaining() {{ return Err({short}); }} "),
                _ => format!(
                    "if count > reader.remaining() / {} {{ return Err({short}); }} ",
                    emit_limits::literal(minimum)
                ),
            };
            format!(
                "{{ let count = match skein_codec::read_count(reader, limits.{bound_name}.min(CEILINGS.{bound_name})) {{ Ok(count) => count, Err(reason) => return Err(Problem {{ path: Path::{path}, reason }}), }}; {precheck}let mut {list_name} = List::with_capacity(count); for _index in 0_u32..count {{ let item = {item_expr}; {list_name}.push(item).expect(\"count within capacity\"); }} {list_name} }}"
            )
        }
        Type::Option(item) => {
            let item_expr = decode_type(
                item,
                schema,
                &emit_limits::child_name(bound_name, emit_limits::LimitLevel::Some),
                path,
                depth.checked_add(1).expect("schema depth"),
            );
            format!(
                "match reader.u8().ok_or({short})? {{ 0 => None, 1 => Some({item_expr}), _ => return Err(Problem {{ path: Path::{path}, reason: skein_codec::Reason::Tag }}) }}"
            )
        }
        Type::Named(name) => {
            if empty_unversioned_record(schema, name) {
                format!("{name}::decode_from(limits, reader)")
            } else {
                format!("{name}::decode_from(limits, reader)?")
            }
        }
    }
}

fn emit_fields(
    schema: &Context<'_>,
    record: &Record,
    method: &mut Method,
    emit: impl Fn(&Type, &str, usize, &mut Vec<String>),
) {
    for field in &record.fields {
        if method.name != "measure" || schema.fixed_bytes(&field.ty).is_none() {
            method.body.push(Statement::new(
                format!("        let field_{} = &{};\n", field.name, member(record, &field.name)),
                true,
            ));
        }
        let mut text = Vec::new();
        emit(&field.ty, &format!("field_{}", field.name), 0, &mut text);
        method.body.extend(text.into_iter().map(|text| Statement::new(text, false)));
    }
}

fn emit_record(schema: &Context<'_>, record: &Record, out: &mut String) {
    let mut helpers = String::new();
    if record.fields.is_empty() && !record.versioned {
        out.push_str(&format!("impl {} {{\n    /// Measures this empty record's wire encoding.\n    #[must_use]\n    pub fn measure(&self) -> u32 {{ 0_u32 }}\n\n", record.name));
    } else {
        out.push_str(&format!(
            "impl {} {{\n    /// Measures this record's wire encoding.\n    #[must_use]\n",
            record.name
        ));
        let mut method = Method::new("measure", "    pub fn measure(&self) -> u32 {\n", "        size\n");
        method.head.push("        let mut size = 0_u32;\n".into());
        if record.versioned {
            let mut text = Vec::new();
            add_size("2", &mut text);
            method.body.extend(text.into_iter().map(|text| Statement::new(text, false)));
        }
        emit_fields(schema, record, &mut method, |ty, value, depth, text| {
            measure_type(ty, schema, value, depth, text);
        });
        emit_functions::render(record, &method, out, &mut helpers);
        out.push('\n');
    }
    out.push_str("    /// Writes into a writer with room for the measured bytes.\n");
    let empty = record.fields.is_empty() && !record.versioned;
    let mut method = Method::new(
        "encode",
        "    pub fn encode(&self, writer: &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow> {\n",
        if empty { "        writer.put(&[])\n" } else { "        Ok(())\n" },
    );
    if record.versioned {
        method.body.push(Statement::new(
            format!("        writer.put(&{}_u16.to_be_bytes())?;\n", emit_limits::literal(schema.version)),
            false,
        ));
    }
    emit_fields(schema, record, &mut method, encode_type);
    emit_functions::render(record, &method, out, &mut helpers);
    out.push_str("\n    /// Reads a whole record and refuses trailing bytes.\n    pub fn decode(limits: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {\n");
    if record.fields.is_empty() && !record.versioned {
        out.push_str("        let value = Self::decode_from(limits, reader);\n");
    } else {
        out.push_str("        let value = Self::decode_from(limits, reader)?;\n");
    }
    out.push_str(&format!("        if !reader.is_empty() {{ return Err(Problem {{ path: Path::{}Tail, reason: skein_codec::Reason::Trailing }}); }}\n        Ok(value)\n    }}\n\n", record.name));
    if record.fields.is_empty() && !record.versioned {
        out.push_str("    fn decode_from(_limits: &Limits, _reader: &mut skein_lib::Reader<'_>) -> Self {\n        Self {}\n    }\n}\n\n");
        return;
    }
    let limits = if record.fields.iter().any(|field| uses_limits(&field.ty)) { "limits" } else { "_limits" };
    let mut method = Method::new(
        "decode_from",
        &format!(
            "    fn decode_from({limits}: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {{\n"
        ),
        "",
    );
    if record.versioned {
        method.head.push(format!(
            "        let version = reader.u16().ok_or({})?;\n",
            short(&format!("{}Version", record.name))
        ));
        method.head.push(format!("        if version != {} {{ return Err(Problem {{ path: Path::{}Version, reason: skein_codec::Reason::Version }}); }}\n", emit_limits::literal(schema.version), record.name));
    }
    for field in &record.fields {
        let bound_name = emit_limits::bound_name(&record.name, &field.name);
        let path = format!("{}{}", record.name, emit_limits::pascal(&field.name));
        let expr = decode_type(&field.ty, schema, &bound_name, &path, 0);
        method.body.push(Statement {
            text: format!("        let decoded_{} = {expr};\n", field.name),
            boundary: true,
            decoded: Some(field.name.clone()),
        });
    }
    let mut members = record
        .fields
        .iter()
        .filter(|field| !emit_types::grouped_bools(record) || !matches!(field.ty, Type::Bool))
        .map(|field| Member::Field(field.name.clone()))
        .collect::<Vec<_>>();
    if emit_types::grouped_bools(record) {
        members.insert(
            0,
            Member::Bools(
                record
                    .fields
                    .iter()
                    .filter(|field| matches!(field.ty, Type::Bool))
                    .map(|field| field.name.clone())
                    .collect(),
            ),
        );
    }
    method.tail = Tail::Members(members);
    emit_functions::render(record, &method, out, &mut helpers);
    out.push_str("}\n\n");
    out.push_str(&helpers);
}

pub(crate) fn uses_limits(ty: &Type) -> bool {
    match ty {
        Type::Bytes(_) | Type::Secret(_) | Type::Text(_) | Type::List(_, _) | Type::Named(_) => true,
        Type::Option(item) => uses_limits(item),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => false,
    }
}

fn emit_enum(schema: &Context<'_>, enumeration: &Enumeration, out: &mut String) {
    let limits = if enumeration.variants.iter().any(|variant| variant.record.is_some()) { "limits" } else { "_limits" };
    out.push_str(&format!("impl {} {{\n    /// Measures this variant's wire encoding.\n    #[must_use]\n    pub fn measure(&self) -> u32 {{\n        match self {{\n", enumeration.name));
    let unit_variants = enumeration
        .variants
        .iter()
        .filter(|variant| variant.record.is_none())
        .map(|variant| format!("Self::{}", emit_limits::pascal(&variant.name)))
        .collect::<Vec<_>>();
    for variant in &enumeration.variants {
        if let Some(record) = &variant.record {
            if enumeration
                .variants
                .iter()
                .take_while(|earlier| *earlier != variant)
                .any(|earlier| earlier.record.as_ref() == Some(record) && earlier.boxed == variant.boxed)
            {
                continue;
            }
            let patterns = enumeration
                .variants
                .iter()
                .filter(|candidate| candidate.record.as_ref() == Some(record) && candidate.boxed == variant.boxed)
                .map(|candidate| format!("Self::{}(record)", emit_limits::pascal(&candidate.name)))
                .collect::<Vec<_>>()
                .join(" | ");
            out.push_str(&format!("            {patterns} => 1_u32.checked_add({record}::measure(record)).expect(\"schema ceilings fit u32\"),\n"));
        }
    }
    if !unit_variants.is_empty() {
        out.push_str(&format!("            {} => 1,\n", unit_variants.join(" | ")));
    }
    out.push_str("        }\n    }\n\n    /// Writes this variant's tag and payload.\n    pub fn encode(&self, writer: &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow> {\n        match self {\n");
    for (index, variant) in enumeration.variants.iter().enumerate() {
        let name = emit_limits::pascal(&variant.name);
        let tag = u8::try_from(index).expect("at most 256 variants");
        match &variant.record {
            Some(_) => out.push_str(&format!(
                "            Self::{name}(record) => {{ writer.put(&[{tag}_u8])?; record.encode(writer)?; }}\n"
            )),
            None => out.push_str(&format!("            Self::{name} => writer.put(&[{tag}_u8])?,\n")),
        }
    }
    out.push_str("        }\n        Ok(())\n    }\n\n    /// Reads a whole variant and refuses trailing bytes.\n    pub fn decode(limits: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {\n");
    out.push_str("        let value = Self::decode_from(limits, reader)?;\n");
    out.push_str(&format!("        if !reader.is_empty() {{ return Err(Problem {{ path: Path::{}Tag, reason: skein_codec::Reason::Trailing }}); }}\n        Ok(value)\n    }}\n\n", enumeration.name));
    out.push_str(&format!("    fn decode_from({limits}: &Limits, reader: &mut skein_lib::Reader<'_>) -> Result<Self, Problem> {{\n        let tag = reader.u8().ok_or({})?;\n        let value = match tag {{\n", short(&format!("{}Tag", enumeration.name))));
    for (index, variant) in enumeration.variants.iter().enumerate() {
        let name = emit_limits::pascal(&variant.name);
        let tag = u8::try_from(index).expect("at most 256 variants");
        match &variant.record {
            Some(record) => {
                let suffix = if empty_unversioned_record(schema, record) { "" } else { "?" };
                let payload = format!("{record}::decode_from(limits, reader){suffix}");
                let payload = if variant.boxed { format!("Box::new({payload})") } else { payload };
                out.push_str(&format!("            {tag} => Self::{name}({payload}),\n"));
            }
            None => out.push_str(&format!("            {tag} => Self::{name},\n")),
        }
    }
    out.push_str(&format!("            _ => return Err(Problem {{ path: Path::{}Tag, reason: skein_codec::Reason::Tag }}),\n        }};\n        Ok(value)\n    }}\n}}\n\n", enumeration.name));
}

pub(crate) fn emit(schema: &Context<'_>, out: &mut String) {
    for declaration in schema.declarations {
        match declaration {
            Declaration::Record(record) => emit_record(schema, record, out),
            Declaration::Enum(enumeration) => emit_enum(schema, enumeration, out),
        }
    }
}
