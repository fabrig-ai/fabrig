//! Plain, exhaustive translation functions (data/codec/maps.md, 3-5).

use super::{Item, Mapping, Maps, Mode, Shape, check, name};
use crate::{Declaration, Enumeration, Field, Record, Schema, Type, Variant, pascal, snake};

struct Emitter<'a> {
    schema: &'a Schema,
    maps: &'a Maps,
    domain: bool,
}

struct Binding<'a> {
    field: &'a Field,
    source: String,
    source_name: &'a str,
    target: &'a str,
    via: Option<&'a str>,
    shape: Shape,
}

impl Emitter<'_> {
    fn record_type(&self, name: &str) -> String {
        format!("{}::{name}", self.maps.path(&self.maps.records))
    }

    fn path(&self, record: &str, field: &str) -> String {
        let constant = snake(&format!("{record}{}", pascal(field))).to_ascii_uppercase();
        format!("{}::paths::{constant}", self.maps.path(&self.maps.records))
    }

    fn convert(&self, input: &str, via: &str, path: &str) -> String {
        let direction = if self.domain { "domain" } else { "record" };
        let via = self.maps.path(via);
        format!(
            "match {via}_to_{direction}({input}) {{ Ok(value) => value, Err(reason) => return Err({path}.problem(reason)) }}"
        )
    }

    fn value(&self, ty: &Type, input: &str, via: Option<&str>, shape: Shape, path: &str) -> String {
        match ty {
            Type::Option(inner) => {
                let converted = self.value(inner, "item", via, shape, path);
                format!("match {input} {{\nSome(item) => Some({converted}),\nNone => None,\n}}")
            }
            Type::List(_, inner) => self.list(inner, input, via, shape, path),
            Type::Named(name) => {
                let value = self.named(name, input);
                if self.domain && shape == Shape::Boxed { format!("Box::new({value})") } else { value }
            }
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => match via {
                Some(via) => self.convert(input, via, path),
                None if self.domain => input.to_owned(),
                None => format!("*{input}"),
            },
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => match via {
                Some(via) => self.convert(input, via, path),
                None if self.domain => input.to_owned(),
                None => format!("{input}.clone()"),
            },
        }
    }

    fn list(&self, inner: &Type, input: &str, via: Option<&str>, shape: Shape, path: &str) -> String {
        // A plain-value list already is the domain list, unless the maps mark a slice.
        if via.is_none() && plain(inner) && shape == Shape::Same {
            return if self.domain { input.to_owned() } else { format!("{input}.clone()") };
        }
        let count = if self.domain || shape == Shape::Same {
            "source.len()".to_owned()
        } else {
            format!(
                "match u32::try_from(source.len()) {{ Ok(count) => count, Err(_error) => return Err({path}.problem(skein_codec::Reason::Bound)) }}"
            )
        };
        let iteration = if self.domain {
            "source.into_boxed()"
        } else if shape == Shape::Slice {
            "source.as_ref()"
        } else {
            "source"
        };
        let converted = self.value(inner, "item", via, Shape::Same, path);
        let result = if self.domain && shape == Shape::Slice { "Box::from(translated)" } else { "translated" };
        format!(
            "{{\nlet source = {input};\nlet count = {count};\nlet mut translated = skein_lib::List::with_capacity(count);\nfor item in {iteration} {{\nlet converted = {converted};\nif translated.push(converted).is_err() {{ return Err({path}.problem(skein_codec::Reason::Bound)); }}\n}}\n{result}\n}}"
        )
    }

    fn named(&self, name: &str, input: &str) -> String {
        let mapping = self.maps.mapping(name).expect("validated dependency");
        match &mapping.mode {
            Mode::Stands { field, via } => {
                let record = check::record(self.schema, name);
                let ty = &record.fields.first().expect("one-field stands for").ty;
                let path = self.path(name, field);
                let record_type = self.record_type(name);
                if self.domain {
                    let value = self.value(ty, "held", via.as_deref(), Shape::Same, &path);
                    format!("{{\nlet {record_type} {{ {field}: held }} = {input};\n{value}\n}}")
                } else {
                    let value = self.value(ty, input, via.as_deref(), Shape::Same, &path);
                    format!("{record_type} {{ {field}: {value} }}")
                }
            }
            Mode::Mirror(_) | Mode::Hand => {
                let prefix = if matches!(mapping.mode, Mode::Hand) {
                    format!("{}::", self.maps.path(self.maps.hand.as_deref().expect("validated by-hand module")))
                } else {
                    String::new()
                };
                let direction = if self.domain { "domain" } else { "record" };
                format!("{prefix}{}_to_{direction}({input}, context)?", snake(name))
            }
            Mode::Payload => unreachable!("payload-only map has no value"),
        }
    }

    fn bindings<'a>(&self, record: &'a Record, mapping: &'a Mapping, overrides: &'a [Item]) -> Vec<Binding<'a>> {
        record
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let default = check::item(&mapping.items, &field.name);
                let over = check::item(overrides, &field.name);
                let rename = over
                    .and_then(|item| item.rename.as_deref())
                    .or_else(|| default.and_then(|item| item.rename.as_deref()))
                    .unwrap_or(&field.name);
                Binding {
                    field,
                    source: format!("field_{index:06}"),
                    source_name: if self.domain { &field.name } else { rename },
                    target: if self.domain { rename } else { &field.name },
                    via: over
                        .and_then(|item| item.via.as_deref())
                        .or_else(|| default.and_then(|item| item.via.as_deref())),
                    shape: over
                        .filter(|item| item.shape != Shape::Same)
                        .or(default)
                        .map_or(Shape::Same, |item| item.shape),
                }
            })
            .collect()
    }

    fn pattern(bindings: &[Binding<'_>]) -> String {
        bindings
            .iter()
            .map(|binding| format!("{}: {}", binding.source_name, binding.source))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn fields(&self, record: &Record, bindings: &[Binding<'_>], to: &str) -> String {
        let fields: Vec<_> = bindings
            .iter()
            .map(|binding| {
                let value = self.value(
                    &binding.field.ty,
                    &binding.source,
                    binding.via,
                    binding.shape,
                    &self.path(&record.name, &binding.field.name),
                );
                format!("{}: {value}", binding.target)
            })
            .collect();
        format!("{to} {{\n{}\n}}", fields.join(",\n"))
    }

    fn variant(&self, variant: &Variant, mapping: &Mapping, domain: &str) -> String {
        let settings = check::item(&mapping.items, &variant.name);
        let record_variant = pascal(&variant.name);
        let domain_variant = settings.and_then(|item| item.rename.as_deref()).unwrap_or(&record_variant);
        let record = format!("{}::{record_variant}", self.record_type(&mapping.name));
        let domain = format!("{domain}::{domain_variant}");
        let (from, to) = if self.domain { (&record, &domain) } else { (&domain, &record) };
        let Some(payload) = &variant.record else {
            return format!("{from} => Ok({to}),\n");
        };
        let record_type = self.record_type(payload);
        let tuple = settings.is_some_and(|item| item.tuple);
        if tuple {
            let input = if self.domain && variant.boxed { "*held" } else { "held" };
            let value = self.named(payload, input);
            let box_output =
                if self.domain { settings.is_some_and(|item| item.shape == Shape::Boxed) } else { variant.boxed };
            let value = if box_output { format!("Box::new({value})") } else { value };
            return format!("{from}(held) => Ok({to}({value})),\n");
        }
        let payload_record = check::record(self.schema, payload);
        let payload_map = self.maps.mapping(payload).expect("validated payload");
        let overrides = settings.and_then(|item| item.block.as_deref()).unwrap_or_default();
        let bindings = self.bindings(payload_record, payload_map, overrides);
        let pattern = Self::pattern(&bindings);
        if self.domain {
            let input = if variant.boxed { "*held" } else { "held" };
            let value = self.fields(payload_record, &bindings, to);
            format!("{from}(held) => {{\nlet {record_type} {{ {pattern} }} = {input};\nOk({value})\n}},\n")
        } else {
            let value = self.fields(payload_record, &bindings, &record_type);
            let value = if variant.boxed { format!("Box::new({value})") } else { value };
            format!("{from} {{ {pattern} }} => Ok({to}({value})),\n")
        }
    }

    fn enumeration(&self, enumeration: &Enumeration, mapping: &Mapping, domain: &str, input: &str) -> String {
        let variants: String =
            enumeration.variants.iter().map(|variant| self.variant(variant, mapping, domain)).collect();
        format!("match {input} {{\n{variants}}}")
    }

    fn function(&self, declaration: &Declaration, mapping: &Mapping, domain: &str) -> String {
        let decl_name = name(declaration);
        let record = self.record_type(decl_name);
        let direction = if self.domain { "domain" } else { "record" };
        let parameter = if self.domain { format!("record: {record}") } else { format!("value: &{domain}") };
        let result = if self.domain { domain } else { &record };
        let problem = self.record_type("Problem");
        let input = if self.domain { "record" } else { "value" };
        let body = match declaration {
            Declaration::Record(record_decl) => {
                let (from, to) = if self.domain { (record.as_str(), domain) } else { (domain, record.as_str()) };
                let bindings = self.bindings(record_decl, mapping, &[]);
                let pattern = Self::pattern(&bindings);
                let value = self.fields(record_decl, &bindings, to);
                format!("let {from} {{ {pattern} }} = {input};\nOk({value})")
            }
            Declaration::Enum(enumeration) => self.enumeration(enumeration, mapping, domain, input),
        };
        let context = if body.contains(", context)") { "context" } else { "_context" };
        format!(
            "// Maps line {}: {decl_name}.\npub fn {}_to_{direction}({parameter}, {context}: &{}) -> Result<{result}, {problem}> {{\n{body}\n}}\n\n",
            mapping.line,
            snake(decl_name),
            self.maps.path(&self.maps.context)
        )
    }
}

fn plain(ty: &Type) -> bool {
    match ty {
        Type::Named(_) => false,
        Type::Option(inner) | Type::List(_, inner) => plain(inner),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Bytes(_)
        | Type::Secret(_)
        | Type::Text(_) => true,
    }
}

pub(super) fn generate(schema: &Schema, maps: &Maps) -> String {
    let mut emitter = Emitter { schema, maps, domain: false };
    let mut body = String::new();
    for declaration in &schema.declarations {
        if let Some(mapping) = maps.mapping(name(declaration))
            && let Mode::Mirror(path) = &mapping.mode
        {
            let domain = maps.path(path);
            emitter.domain = false;
            body.push_str(&emitter.function(declaration, mapping, &domain));
            emitter.domain = true;
            body.push_str(&emitter.function(declaration, mapping, &domain));
        }
    }
    let imports = if body.contains("Box::") { "use alloc::boxed::Box;\n\n" } else { "" };
    let output = format!(
        "// Generated by skein-codegen from {} {} and its maps. Do not edit.\n// Translation pairs (skein/docs/design/data/codec/maps.md, 4).\n\n{imports}{body}",
        schema.family, schema.version
    );
    indent(&output)
}

// Expressions have balanced braces; keep their nested steps readable without
// making generation depend on a particular installed rustfmt.
fn indent(source: &str) -> String {
    let mut output = String::new();
    let mut depth = 0_usize;
    for line in source.lines() {
        let closing = line.chars().take_while(|character| *character == '}').count();
        let indentation = depth.checked_sub(closing).expect("balanced emitted expression");
        if !line.is_empty() {
            output.push_str(&"    ".repeat(indentation));
            output.push_str(line);
        }
        output.push('\n');
        let opened = line.chars().filter(|character| *character == '{').count();
        let closed = line.chars().filter(|character| *character == '}').count();
        depth =
            depth.checked_add(opened).and_then(|depth| depth.checked_sub(closed)).expect("balanced emitted expression");
    }
    output
}
