//! Plain, exhaustive translation functions (data/codec/maps.md, 3-5).

use core::fmt::Write;

use super::{Item, Mapping, Maps, Mode, check, name};
use crate::{Declaration, Field, Record, Schema, Type, pascal, snake};

struct Emitter<'a> {
    schema: &'a Schema,
    maps: &'a Maps,
    out: String,
    next: usize,
    domain: bool,
}

impl Emitter<'_> {
    fn line(&mut self, text: &str) {
        writeln!(self.out, "{text}").expect("write generated translation");
    }

    fn local(&mut self) -> String {
        let name = format!("mapped_{}", self.next);
        self.next = self.next.checked_add(1).expect("generated locals fit usize");
        name
    }

    fn record_type(&self, name: &str) -> String {
        format!("{}::{name}", self.maps.path(&self.maps.records))
    }

    fn path(&self, record: &str, field: &str) -> String {
        let constant = snake(&format!("{record}{}", pascal(field))).to_ascii_uppercase();
        format!("{}::paths::{constant}", self.maps.path(&self.maps.records))
    }

    fn refusal( path: &str, reason: &str) -> String {
        format!("{path}.problem(skein_codec::Reason::{reason})")
    }

    fn convert(&mut self, input: &str, via: &str, path: &str) -> String {
        let output = self.local();
        let direction = if self.domain { "domain" } else { "record" };
        let via = self.maps.path(via);
        self.line(&format!("let {output} = match {via}_to_{direction}({input}) {{"));
        self.line("Ok(value) => value,");
        self.line(&format!("Err(reason) => return Err({path}.problem(reason)),"));
        self.line("};");
        output
    }

    fn value(&mut self, ty: &Type, input: &str, via: Option<&str>, path: &str) -> String {
        match ty {
            Type::Option(inner) => {
                let output = self.local();
                let held = self.local();
                self.line(&format!("let {output} = match {input} {{"));
                self.line(&format!("Some({held}) => {{"));
                let converted = self.value(inner, &held, via, path);
                self.line(&format!("Some({converted})"));
                self.line("},\nNone => None,\n};");
                output
            }
            Type::List(_, inner) => {
                let output = self.local();
                let count = self.local();
                if self.domain {
                    self.line(&format!("let {count} = {input}.len();"));
                } else {
                    self.line(&format!("let {count} = match u32::try_from({input}.iter().len()) {{"));
                    self.line("Ok(count) => count,");
                    self.line(&format!("Err(_error) => return Err({}),", Self::refusal(path, "Bound")));
                    self.line("};");
                }
                self.line(&format!("let mut {output} = skein_lib::List::with_capacity({count});"));
                let held = self.local();
                let iteration = if self.domain { format!("{input}.into_boxed()") } else { format!("{input}.iter()") };
                self.line(&format!("for {held} in {iteration} {{"));
                let converted = self.value(inner, &held, via, path);
                self.line(&format!("if {output}.push({converted}).is_err() {{ return Err({}); }}", Self::refusal(path, "Bound")));
                self.line("}");
                if self.domain { format!("core::convert::From::from({output})") } else { output }
            }
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) | Type::Bytes(_) | Type::Secret(_) | Type::Text(_) | Type::Named(_) => {
                if let Some(via) = via {
                    let converted = self.convert(input, via, path);
                    return if self.domain { format!("core::convert::From::from({converted})") } else { converted };
                }
                if let Type::Named(name) = ty {
                    return self.named(name, input);
                }
                if self.domain {
                    format!("core::convert::From::from({input})")
                } else {
                    let output = self.local();
                    let ty = crate::emit_types::rust_type(ty).replace("Box<", "alloc::boxed::Box<");
                    self.line(&format!("let {output}: {ty} = Clone::clone({input});"));
                    output
                }
            }
        }
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
                    let held = self.local();
                    self.line(&format!("let {record_type} {{ {field}: {held} }} = {input};"));
                    self.value(ty, &held, via.as_deref(), &path)
                } else {
                    let value = self.value(ty, input, via.as_deref(), &path);
                    format!("{record_type} {{ {field}: {value} }}")
                }
            }
            Mode::Mirror(_) | Mode::Hand => {
                let prefix = if matches!(mapping.mode, Mode::Hand) {
                    format!("{}::", self.maps.path(self.maps.hand.as_deref().expect("validated by-hand module")))
                } else { String::new() };
                let direction = if self.domain { "domain" } else { "record" };
                let call = format!("{prefix}{}_to_{direction}({input}, context)?", snake(name));
                if self.domain { format!("core::convert::From::from({call})") } else { call }
            }
            Mode::Payload => unreachable!("payload-only map has no value"),
        }
    }

    fn fields(&mut self, record: &Record, mapping: &Mapping, overrides: &[Item], from: &str, to: &str, input: &str) -> String {
        let mut pattern = Vec::new();
        let mut fields = Vec::new();
        for Field { name, .. } in &record.fields {
            let default = check::item(&mapping.items, name);
            let over = check::item(overrides, name);
            let rename = over.and_then(|item| item.rename.as_deref()).or_else(|| default.and_then(|item| item.rename.as_deref())).unwrap_or(name);
            let source = if self.domain { name.as_str() } else { rename };
            let target = if self.domain { rename } else { name.as_str() };
            let local = self.local();
            pattern.push(format!("{source}: {local}"));
            fields.push((name.as_str(), target.to_owned(), local, over.and_then(|item| item.via.as_deref()).or_else(|| default.and_then(|item| item.via.as_deref()))));
        }
        self.line(&format!("let {from} {{ {} }} = {input};", pattern.join(", ")));
        let mut output = Vec::new();
        for (name, target, input, via) in fields {
            let field = record.fields.iter().find(|field| field.name == name).expect("record field");
            let value = self.value(&field.ty, &input, via, &self.path(&record.name, name));
            output.push(format!("{target}: {value}"));
        }
        let value = format!("{to} {{ {} }}", output.join(", "));
        value
    }

    fn function(&mut self, declaration: &Declaration, mapping: &Mapping, domain: &str) {
        self.next = 0;
        let decl_name = name(declaration);
        let record = self.record_type(decl_name);
        let direction = if self.domain { "domain" } else { "record" };
        let parameter = if self.domain { format!("record: {record}") } else { format!("value: &{domain}") };
        let result = if self.domain { domain } else { &record };
        let problem = self.record_type("Problem");
        self.line(&format!("// Maps line {}: {decl_name}.", mapping.line));
        self.line(&format!("pub fn {}_to_{direction}({parameter}, context: &{}) -> Result<{result}, {problem}> {{", snake(decl_name), self.maps.path(&self.maps.context)));
        self.line("let _context = context;");
        let input = if self.domain { "record" } else { "value" };
        match declaration {
            Declaration::Record(record_decl) => {
                let (from, to) = if self.domain { (record.as_str(), domain) } else { (domain, record.as_str()) };
                let value = self.fields(record_decl, mapping, &[], from, to, input);
                self.line(&format!("Ok({value})"));
            }
            Declaration::Enum(enumeration) => {
                self.line(&format!("match {input} {{"));
                for variant in &enumeration.variants {
                    let settings = check::item(&mapping.items, &variant.name);
                    let record_variant = pascal(&variant.name);
                    let domain_variant = settings.and_then(|item| item.rename.as_deref()).unwrap_or(&record_variant);
                    let (from, to) = if self.domain {
                        (format!("{record}::{record_variant}"), format!("{domain}::{domain_variant}"))
                    } else {
                        (format!("{domain}::{domain_variant}"), format!("{record}::{record_variant}"))
                    };
                    if let Some(payload) = &variant.record {
                        let held = self.local();
                        let tuple = settings.is_some_and(|item| item.tuple);
                        if self.domain || tuple {
                            self.line(&format!("{from}({held}) => {{"));
                        } else {
                            // Bind the entire struct variant without a rest pattern.
                            let payload_record = check::record(self.schema, payload);
                            let payload_map = self.maps.mapping(payload).expect("validated payload");
                            let overrides = settings.and_then(|item| item.block.as_deref()).unwrap_or_default();
                            let fields: Vec<_> = payload_record.fields.iter().map(|field| {
                                let default = check::item(&payload_map.items, &field.name);
                                let over = check::item(overrides, &field.name);
                                let name = over.and_then(|item| item.rename.as_deref()).or_else(|| default.and_then(|item| item.rename.as_deref())).unwrap_or(&field.name);
                                format!("{name}: {}", field.name)
                            }).collect();
                            self.line(&format!("{from} {{ {} }} => {{", fields.join(", ")));
                        }
                        let translated = if tuple {
                            let input = if self.domain && variant.boxed { format!("*{held}") } else { held };
                            let value = self.named(payload, &input);
                            let value = if !self.domain && variant.boxed { format!("alloc::boxed::Box::new({value})") } else { value };
                            format!("{to}({value})")
                        } else {
                            let payload_record = check::record(self.schema, payload);
                            let payload_map = self.maps.mapping(payload).expect("validated payload");
                            let overrides = settings.and_then(|item| item.block.as_deref()).unwrap_or_default();
                            if self.domain {
                                let input = if variant.boxed { format!("*{held}") } else { held };
                                self.fields(payload_record, payload_map, overrides, &self.record_type(payload), &to, &input)
                            } else {
                                let mut output = Vec::new();
                                for field in &payload_record.fields {
                                    let default = check::item(&payload_map.items, &field.name);
                                    let over = check::item(overrides, &field.name);
                                    let via = over.and_then(|item| item.via.as_deref()).or_else(|| default.and_then(|item| item.via.as_deref()));
                                    let value = self.value(&field.ty, &field.name, via, &self.path(payload, &field.name));
                                    output.push(format!("{}: {value}", field.name));
                                }
                                let value = format!("{} {{ {} }}", self.record_type(payload), output.join(", "));
                                let value = if variant.boxed { format!("alloc::boxed::Box::new({value})") } else { value };
                                format!("{to}({value})")
                            }
                        };
                        self.line(&format!("Ok({translated})\n}},"));
                    } else {
                        self.line(&format!("{from} => Ok({to}),"));
                    }
                }
                self.line("}");
            }
        }
        self.line("}\n");
    }
}

pub(super) fn generate(schema: &Schema, maps: &Maps) -> String {
    let mut emitter = Emitter { schema, maps, out: String::new(), next: 0, domain: false };
    emitter.line(&format!("// Generated by skein-codegen from {} {} and its maps. Do not edit.", schema.family, schema.version));
    emitter.line("// Translation pairs (skein/docs/design/data/codec/maps.md, 4).\n");
    for declaration in &schema.declarations {
        if let Some(mapping) = maps.mapping(name(declaration))
            && let Mode::Mirror(path) = &mapping.mode {
                let domain = maps.path(path);
                emitter.domain = false;
                emitter.function(declaration, mapping, &domain);
                emitter.domain = true;
                emitter.function(declaration, mapping, &domain);
        }
    }
    emitter.out
}
