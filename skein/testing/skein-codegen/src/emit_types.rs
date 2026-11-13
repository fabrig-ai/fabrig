#![expect(clippy::format_push_string, reason = "generator assembles reviewed source fragments")]

//! Sealed record and enum values for data/codec/README.md, section 4.

use crate::emit_functions::{self, Method, Statement};
use crate::{Declaration, Enumeration, Field, Record, Type, analysis::Context, emit_limits};

pub(crate) fn rust_type(ty: &Type) -> String {
    match ty {
        Type::U8 => "u8".into(),
        Type::U16 => "u16".into(),
        Type::U32 => "u32".into(),
        Type::U64 => "u64".into(),
        Type::Bool => "bool".into(),
        Type::Duration => "skein_lib::Duration".into(),
        Type::Fixed(bound) => format!("[u8; {}]", emit_limits::literal(*bound)),
        Type::Secret(_) => "skein_lib::Secret".into(),
        Type::Bytes(_) | Type::Text(_) => "Box<[u8]>".into(),
        Type::List(_, item) => format!("List<{}>", rust_type(item)),
        Type::Option(item) => format!("Option<{}>", rust_type(item)),
        Type::Named(name) => name.clone(),
    }
}

fn accessor_type(ty: &Type) -> String {
    match ty {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration => rust_type(ty),
        Type::Secret(_) => "&skein_lib::Secret".into(),
        Type::Bytes(_) | Type::Text(_) => "&[u8]".into(),
        Type::Fixed(_) | Type::List(_, _) | Type::Option(_) | Type::Named(_) => format!("&{}", rust_type(ty)),
    }
}

fn accessor_expr(field: &Field, bool_index: Option<usize>, bool_count: usize) -> String {
    if let Some(index) = bool_index {
        let pattern = (0..bool_count)
            .map(|position| if position == index { "value" } else { "_" })
            .collect::<Vec<_>>()
            .join(", ");
        return format!("{{ let [{pattern}] = self.skein_bools; value }}");
    }
    match field.ty {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration => format!("self.{}", field.name),
        Type::Fixed(_)
        | Type::List(_, _)
        | Type::Option(_)
        | Type::Named(_)
        | Type::Bytes(_)
        | Type::Secret(_)
        | Type::Text(_) => {
            format!("&self.{}", field.name)
        }
    }
}

pub(crate) fn grouped_bools(record: &Record) -> bool {
    record.fields.iter().filter(|field| matches!(field.ty, Type::Bool)).count() > 3
}

pub(crate) fn bool_index(record: &Record, name: &str) -> Option<usize> {
    if !grouped_bools(record) {
        return None;
    }
    record.fields.iter().filter(|field| matches!(field.ty, Type::Bool)).position(|field| field.name == name)
}

struct Guard {
    condition: String,
    action: String,
}

impl Guard {
    fn text(&self) -> String {
        if self.condition.is_empty() {
            self.action.clone()
        } else {
            format!("        if {} {{ {} }}\n", self.condition, self.action)
        }
    }
}

fn guards(schema: &Context<'_>, ty: &Type, value: &str, bound_name: &str, path: &str, depth: usize) -> Vec<Guard> {
    let problem = format!("Problem {{ path: Path::{path}, reason: skein_codec::Reason::Bound }}");
    let mut out = Vec::new();
    let next = depth.checked_add(1).expect("schema nesting fits usize");
    match ty {
        Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => {
            out.push(Guard { condition: format!("{value}.len() > usize::try_from(limits.{bound_name}.min(CEILINGS.{bound_name})).expect(\"u32 fits usize\")"), action: format!("return Err({problem});") });
            if let Type::Text(_) = ty {
                out.push(Guard {
                    condition: format!("!skein_codec::text_is_valid({value}.as_ref())"),
                    action: format!("return Err(Problem {{ path: Path::{path}, reason: skein_codec::Reason::Utf8 }});"),
                });
            }
        }
        Type::List(_, item) | Type::Option(item) => {
            let optional = matches!(ty, Type::Option(_));
            if !optional {
                out.push(Guard {
                    condition: format!("{value}.capacity() > limits.{bound_name}.min(CEILINGS.{bound_name})"),
                    action: format!("return Err({problem});"),
                });
            }
            if !schema.has_bounds(item) {
                return out;
            }
            let suffix = if optional { "some" } else { "item" };
            let item_name = format!("{suffix}_{depth}");
            let level = if optional { emit_limits::LimitLevel::Some } else { emit_limits::LimitLevel::Item };
            let bound = emit_limits::child_name(bound_name, level);
            let mut inner = guards(schema, item, &item_name, &bound, path, next);
            if optional && inner.len() == 1 && inner.first().is_some_and(|guard| !guard.condition.is_empty()) {
                let guard = inner.first_mut().expect("one guard");
                guard.condition = format!("let Some({item_name}) = {value}.as_ref() && {}", guard.condition);
                out.extend(inner);
            } else {
                let inner = inner.iter().map(Guard::text).collect::<String>();
                let head = if optional {
                    format!("if let Some({item_name}) = {value}.as_ref()")
                } else {
                    format!("for {item_name} in {value}.as_slice()")
                };
                out.push(Guard { condition: String::new(), action: format!("        {head} {{\n{inner}        }}\n") });
            }
        }
        Type::Named(_) => {
            if schema.has_bounds(ty) {
                out.push(Guard { condition: String::new(), action: format!("        {value}.check(limits)?;\n") });
            }
        }
        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => {}
    }
    out
}

fn emit_check(schema: &Context<'_>, record: &Record, field: &Field, out: &mut Vec<Statement>) {
    let bound = emit_limits::bound_name(&record.name, &field.name);
    let path = format!("{}{}", record.name, emit_limits::pascal(&field.name));
    let value = format!("self.{}", field.name);
    out.extend(
        guards(schema, &field.ty, &value, &bound, &path, 0).iter().map(|guard| Statement::new(guard.text(), true)),
    );
}

fn emit_record(schema: &Context<'_>, record: &Record, out: &mut String) {
    let grouped = grouped_bools(record);
    let bool_count = record.fields.iter().filter(|field| matches!(field.ty, Type::Bool)).count();
    out.push_str(&format!(
        "/// Movable fields of `{}`.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]\npub struct {}Parts {{\n",
        record.name, record.name
    ));
    if grouped {
        out.push_str(&format!("    /// Boolean fields in schema order.\n    pub skein_bools: [bool; {bool_count}],\n"));
    }
    for field in &record.fields {
        if grouped && matches!(field.ty, Type::Bool) {
            continue;
        }
        out.push_str(&format!(
            "    /// The `{}` field.\n    pub {}: {},\n",
            field.name,
            field.name,
            rust_type(&field.ty)
        ));
    }
    out.push_str("}\n\n");
    out.push_str(&format!(
        "/// `{}` in this codec family.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]\npub struct {} {{\n",
        record.name, record.name
    ));
    if grouped {
        out.push_str(&format!("    skein_bools: [bool; {bool_count}],\n"));
    }
    for field in &record.fields {
        if grouped && matches!(field.ty, Type::Bool) {
            continue;
        }
        out.push_str(&format!("    {}: {},\n", field.name, rust_type(&field.ty)));
    }
    out.push_str("}\n\n");
    out.push_str(&format!("impl {} {{\n", record.name));
    let mut names = record
        .fields
        .iter()
        .filter(|field| !grouped || !matches!(field.ty, Type::Bool))
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    if grouped {
        names.insert(0, "skein_bools");
    }
    let members = names.join(", ");
    let has_bounds = schema.has_bounds(&Type::Named(record.name.clone()));
    let check = if has_bounds { "        value.check(limits)?;\n" } else { "" };
    let limits = if has_bounds { "limits" } else { "_limits" };
    if schema.bounds.is_empty() {
        out.push_str(&format!("    /// Makes a value; this family has no adjustable bounds.\n    #[must_use]\n    pub fn new(_limits: &Limits, parts: {}Parts) -> Self {{\n        let {}Parts {{ {members} }} = parts;\n        Self {{ {members} }}\n    }}\n\n", record.name, record.name));
    } else {
        out.push_str(&format!("    /// Makes a value within the given limits.\n    pub fn new({limits}: &Limits, parts: {}Parts) -> Result<Self, Problem> {{\n        let {}Parts {{ {members} }} = parts;\n        let value = Self {{ {members} }};\n{check}        Ok(value)\n    }}\n\n", record.name, record.name));
    }
    for field in &record.fields {
        out.push_str(&format!(
            "    /// Reads the `{}` field.\n    #[must_use]\n    pub fn {}(&self) -> {} {{ {} }}\n\n",
            field.name,
            field.name,
            accessor_type(&field.ty),
            accessor_expr(field, bool_index(record, &field.name), bool_count)
        ));
    }
    let moved = names.iter().map(|name| format!("{name}: self.{name}")).collect::<Vec<_>>().join(", ");
    out.push_str(&format!("    /// Moves the fields out without copying.\n    #[must_use]\n    pub fn into_parts(self) -> {}Parts {{ {}Parts {{ {moved} }} }}\n\n", record.name, record.name));
    let mut helpers = String::new();
    if has_bounds {
        let mut method =
            Method::new("check", "    fn check(&self, limits: &Limits) -> Result<(), Problem> {\n", "        Ok(())\n");
        for field in &record.fields {
            emit_check(schema, record, field, &mut method.body);
        }
        emit_functions::render(record, &method, out, &mut helpers);
    }
    out.push_str("}\n\n");
    out.push_str(&helpers);
}

fn emit_enum(schema: &Context<'_>, enumeration: &Enumeration, out: &mut String) {
    out.push_str(&format!(
        "/// `{}` in this codec family.\n#[derive(Clone, Debug, PartialEq, Eq, Hash)]\npub enum {} {{\n",
        enumeration.name, enumeration.name
    ));
    for variant in &enumeration.variants {
        let name = emit_limits::pascal(&variant.name);
        match &variant.record {
            Some(record) => {
                let record = if variant.boxed { format!("Box<{record}>") } else { record.clone() };
                out.push_str(&format!("    /// `{}` carrying `{}`.\n    {name}({record}),\n", variant.name, record));
            }
            None => out.push_str(&format!("    /// `{}` without a payload.\n    {name},\n", variant.name)),
        }
    }
    out.push_str("}\n\n");
    if schema.bounds.is_empty() {
        out.push_str(&format!("impl {} {{\n    /// Accepts a variant; this family has no adjustable bounds.\n    #[must_use]\n    pub fn new(_limits: &Limits, value: Self) -> Self {{ value }}\n}}\n\n", enumeration.name));
        return;
    }
    let has_bounds = schema.has_bounds(&Type::Named(enumeration.name.clone()));
    let check = if has_bounds { "        value.check(limits)?;\n" } else { "" };
    let limits = if has_bounds { "limits" } else { "_limits" };
    out.push_str(&format!("impl {} {{\n    /// Checks the payload against the given limits.\n    pub fn new({limits}: &Limits, value: Self) -> Result<Self, Problem> {{\n{check}        Ok(value)\n    }}\n\n", enumeration.name));
    if !has_bounds {
        out.push_str("}\n\n");
        return;
    }
    out.push_str("    fn check(&self, limits: &Limits) -> Result<(), Problem> {\n        match self {\n");
    // Payloads without bounds share the unit variants' arm.
    let unbounded_variants = enumeration
        .variants
        .iter()
        .filter(|variant| variant.record.as_ref().is_none_or(|name| !schema.has_bounds(&Type::Named(name.clone()))))
        .map(|variant| {
            let name = emit_limits::pascal(&variant.name);
            if variant.record.is_some() { format!("Self::{name}(_)") } else { format!("Self::{name}") }
        })
        .collect::<Vec<_>>();
    for variant in &enumeration.variants {
        if let Some(record) = &variant.record {
            if !schema.has_bounds(&Type::Named(record.clone()))
                || enumeration
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
            out.push_str(&format!("            {patterns} => {record}::check(record, limits),\n"));
        }
    }
    if !unbounded_variants.is_empty() {
        out.push_str(&format!("            {} => Ok(()),\n", unbounded_variants.join(" | ")));
    }
    out.push_str("        }?;\n");
    out.push_str("        Ok(())\n    }\n}\n\n");
}

pub(crate) fn emit(schema: &Context<'_>, out: &mut String) {
    for declaration in schema.declarations {
        match declaration {
            Declaration::Record(record) => emit_record(schema, record, out),
            Declaration::Enum(enumeration) => emit_enum(schema, enumeration, out),
        }
    }
}
