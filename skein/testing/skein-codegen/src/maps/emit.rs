//! Plain, exhaustive translation functions (data/codec/maps.md, 3-5).

use alloc::collections::{BTreeMap, BTreeSet};

use super::{Item, Mapping, Maps, Mode, Shape, check, error, name};
use crate::{Declaration, Enumeration, Error, Field, Record, Schema, Type, Variant, pascal, snake};

#[derive(Clone, Copy, Default)]
struct Reach {
    refuses: bool,
    context: bool,
}

impl Reach {
    fn include(&mut self, other: Self) {
        self.refuses |= other.refuses;
        self.context |= other.context;
    }

    fn result(self, value: String) -> String {
        if self.refuses { format!("Ok({value})") } else { value }
    }
}

struct Emitter<'a> {
    schema: &'a Schema,
    maps: &'a Maps,
    domain: bool,
    reach: Reach,
    // A schema only names earlier declarations. Keep each direction's reach
    // once it has been emitted, so nested calls inherit just those effects.
    generated: BTreeMap<(&'a str, bool), Reach>,
    converters_called: BTreeSet<String>,
    hand_called: BTreeSet<String>,
}

struct Binding<'a> {
    field: &'a Field,
    source: String,
    source_name: &'a str,
    target: &'a str,
    via: Option<&'a str>,
    shape: Shape,
}

impl<'a> Emitter<'a> {
    fn record_type(&self, name: &str) -> String {
        format!("{}::{name}", self.maps.path(&self.maps.records))
    }

    fn path(&self, record: &str, field: &str) -> String {
        let constant = snake(&format!("{record}{}", pascal(field))).to_ascii_uppercase();
        format!("{}::paths::{constant}", self.maps.path(&self.maps.records))
    }

    fn direction(&self) -> &'static str {
        if self.domain { "domain" } else { "record" }
    }

    fn convert(&mut self, input: &str, via: &str, path: &str) -> String {
        let function = format!("{}_to_{}", self.maps.path(via), self.direction());
        let refuses = self.maps.refuses.contains_key(&function);
        let call = format!("{function}({input})");
        self.converters_called.insert(function);
        self.reach.refuses |= refuses;
        if refuses {
            format!("match {call} {{ Ok(value) => value, Err(reason) => return Err({path}.problem(reason)) }}")
        } else {
            call
        }
    }

    fn value(&mut self, ty: &Type, input: &str, via: Option<&str>, shape: Shape, path: &str) -> String {
        match ty {
            Type::Option(inner) => {
                let converted = self.value(inner, "item", via, shape, path);
                // Bind before constructing Some: this stays a step, not an
                // Option::map callback, even for a plain conversion.
                format!(
                    "match {input} {{\nSome(item) => {{\nlet converted = {converted};\nSome(converted)\n}}\nNone => None,\n}}"
                )
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

    fn list(&mut self, inner: &Type, input: &str, via: Option<&str>, shape: Shape, path: &str) -> String {
        // A plain-value list already is the domain list, unless the maps mark a slice.
        if via.is_none() && plain(inner) && shape == Shape::Same {
            return if self.domain { input.to_owned() } else { format!("{input}.clone()") };
        }
        let count = if self.domain || shape == Shape::Same {
            "source.len()".to_owned()
        } else {
            self.reach.refuses = true;
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
            "{{\nlet source = {input};\nlet count = {count};\nlet mut translated = skein_lib::List::with_capacity(count);\nfor item in {iteration} {{\nlet converted = {converted};\nassert!(translated.push(converted).is_ok(), \"one output per source item\");\n}}\n{result}\n}}"
        )
    }

    fn named(&mut self, name: &str, input: &str) -> String {
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
                let stem = format!("{}_to_{}", snake(name), self.direction());
                let (function, reach) = if matches!(mapping.mode, Mode::Hand) {
                    let module = self.maps.hand.as_deref().expect("validated by-hand module");
                    let function = format!("{}::{stem}", self.maps.path(module));
                    let reach = Reach {
                        refuses: self.maps.refuses.contains_key(&function),
                        context: self.maps.with_context.contains_key(&function),
                    };
                    self.hand_called.insert(function.clone());
                    (function, reach)
                } else {
                    (stem, *self.generated.get(&(name, self.domain)).expect("earlier generated pair"))
                };
                self.reach.include(reach);
                let context = if reach.context { ", context" } else { "" };
                let question = if reach.refuses { "?" } else { "" };
                format!("{function}({input}{context}){question}")
            }
            Mode::Payload => unreachable!("payload-only map has no value"),
        }
    }

    fn bindings<'b>(&self, record: &'b Record, mapping: &'b Mapping, overrides: &'b [Item]) -> Vec<Binding<'b>> {
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

    fn fields(&mut self, record: &Record, bindings: &[Binding<'_>], to: &str) -> String {
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

    fn variant(&mut self, variant: &Variant, mapping: &Mapping, domain: &str) -> String {
        let settings = check::item(&mapping.items, &variant.name);
        let record_variant = pascal(&variant.name);
        let domain_variant = settings.and_then(|item| item.rename.as_deref()).unwrap_or(&record_variant);
        let record = format!("{}::{record_variant}", self.record_type(&mapping.name));
        let domain = format!("{domain}::{domain_variant}");
        let (from, to) = if self.domain { (&record, &domain) } else { (&domain, &record) };
        let Some(payload) = &variant.record else {
            return format!("{from} => {to},\n");
        };
        let record_type = self.record_type(payload);
        let tuple = settings.is_some_and(|item| item.tuple);
        if tuple {
            let input = if self.domain && variant.boxed { "*held" } else { "held" };
            let value = self.named(payload, input);
            let box_output =
                if self.domain { settings.is_some_and(|item| item.shape == Shape::Boxed) } else { variant.boxed };
            let value = if box_output { format!("Box::new({value})") } else { value };
            return format!("{from}(held) => {to}({value}),\n");
        }
        let payload_record = check::record(self.schema, payload);
        let payload_map = self.maps.mapping(payload).expect("validated payload");
        let overrides = settings.and_then(|item| item.block.as_deref()).unwrap_or_default();
        let bindings = self.bindings(payload_record, payload_map, overrides);
        let pattern = Self::pattern(&bindings);
        if self.domain {
            let input = if variant.boxed { "*held" } else { "held" };
            let value = self.fields(payload_record, &bindings, to);
            format!("{from}(held) => {{\nlet {record_type} {{ {pattern} }} = {input};\n{value}\n}},\n")
        } else {
            let value = self.fields(payload_record, &bindings, &record_type);
            let value = if variant.boxed { format!("Box::new({value})") } else { value };
            format!("{from} {{ {pattern} }} => {to}({value}),\n")
        }
    }

    fn enumeration(&mut self, enumeration: &Enumeration, mapping: &Mapping, domain: &str, input: &str) -> String {
        let variants: String =
            enumeration.variants.iter().map(|variant| self.variant(variant, mapping, domain)).collect();
        format!("match {input} {{\n{variants}}}")
    }

    fn function(&mut self, declaration: &'a Declaration, mapping: &Mapping, domain: &str) -> String {
        self.reach = Reach::default();
        let decl_name = name(declaration);
        let record = self.record_type(decl_name);
        let direction = self.direction();
        let parameter = if self.domain { format!("record: {record}") } else { format!("value: &{domain}") };
        let result = if self.domain { domain } else { &record };
        let input = if self.domain { "record" } else { "value" };
        let body = match declaration {
            Declaration::Record(record_decl) => {
                let (from, to) = if self.domain { (record.as_str(), domain) } else { (domain, record.as_str()) };
                let bindings = self.bindings(record_decl, mapping, &[]);
                let pattern = Self::pattern(&bindings);
                let value = self.fields(record_decl, &bindings, to);
                let value = self.reach.result(value);
                format!("let {from} {{ {pattern} }} = {input};\n{value}")
            }
            Declaration::Enum(enumeration) => {
                let value = self.enumeration(enumeration, mapping, domain, input);
                self.reach.result(value)
            }
        };
        self.generated.insert((decl_name, self.domain), self.reach);
        let context = if self.reach.context {
            let (_, ty) = self.maps.context.as_ref().expect("validated context type");
            format!(", context: &{}", self.maps.path(ty))
        } else {
            String::new()
        };
        let result = if self.reach.refuses {
            format!("Result<{result}, {}>", self.record_type("Problem"))
        } else {
            result.to_owned()
        };
        // Clippy excludes the outer braces and blank or comment-only lines.
        // The emitted body has no block comments, only whole-line comments.
        let indented = indent(&body);
        let lines = indented.lines().filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//")).count();
        let expectation = if lines > 100 {
            "#[expect(clippy::too_many_lines, reason = \"one translation step per field or variant (data/codec/maps.md, 4.1)\")]\n"
        } else {
            ""
        };
        let must_use = if self.reach.refuses { "" } else { "#[must_use]\n" };
        format!(
            "{expectation}{must_use}pub fn {}_to_{direction}({parameter}{context}) -> {result} {{\n{body}\n}}\n\n",
            snake(decl_name)
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

pub(super) fn generate(schema: &Schema, maps: &Maps) -> Result<String, Error> {
    let mut emitter = Emitter {
        schema,
        maps,
        domain: false,
        reach: Reach::default(),
        generated: BTreeMap::new(),
        converters_called: BTreeSet::new(),
        hand_called: BTreeSet::new(),
    };
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
    for (path, line) in &maps.refuses {
        if !emitter.converters_called.contains(path) && !emitter.hand_called.contains(path) {
            return Err(error(schema, *line, format!("refuses names an uncalled function {path}")));
        }
    }
    for (path, line) in &maps.with_context {
        if !emitter.hand_called.contains(path) {
            return Err(error(schema, *line, format!("with context names an uncalled by-hand function {path}")));
        }
    }
    let imports = if body.contains("Box::") { "use alloc::boxed::Box;\n\n" } else { "" };
    let output = format!(
        "// Generated by skein-codegen from {} {} and its maps. Do not edit.\n// Translation pairs (skein/docs/design/data/codec/maps.md, 4).\n\n{imports}{body}",
        schema.family, schema.version
    );
    Ok(indent(&output))
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
