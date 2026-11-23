//! Largest schema values and their roots (data/codec/golden.md, section 1).

use alloc::collections::{BTreeMap, BTreeSet};

use crate::{Declaration, Enumeration, Error, Record, Schema, Type, Variant, analysis::Context, emit_limits};

fn name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Record(record) => &record.name,
        Declaration::Enum(enumeration) => &enumeration.name,
    }
}

fn named<'a>(ty: &'a Type, used: &mut BTreeSet<&'a str>) {
    match ty {
        Type::Named(name) => {
            used.insert(name);
        }
        Type::List(_, item) | Type::Option(item) => named(item, used),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Bytes(_)
        | Type::Secret(_)
        | Type::Text(_) => {}
    }
}

/// Declarations no other declaration names, in declaration order.
/// Held bytes and secrets are opaque to their owner's decoder.
#[must_use]
pub fn roots(schema: &Schema) -> Vec<String> {
    let mut used = BTreeSet::new();
    for declaration in &schema.declarations {
        match declaration {
            Declaration::Record(record) => {
                for field in &record.fields {
                    named(&field.ty, &mut used);
                }
            }
            Declaration::Enum(enumeration) => {
                used.extend(enumeration.variants.iter().filter_map(|variant| variant.record.as_deref()));
            }
        }
    }
    schema.declarations.iter().map(name).filter(|name| !used.contains(name)).map(str::to_owned).collect()
}

/// The declaration's largest encoding at limits in the generated `LIMIT_NAMES` order.
///
/// # Errors
/// Returns an error for an unknown declaration, a wrong number of limits, or a
/// limit above its schema ceiling. The schema must have been validated by [`crate::parse`].
pub fn largest(schema: &Schema, declaration: &str, limits: &[u32]) -> Result<Vec<u8>, Error> {
    let context = Context::new(schema);
    let mut builder = Builder::new(&context, declaration, limits)?;
    builder.build(declaration);
    Ok(builder.values.remove(declaration).expect("built declaration"))
}

/// Each variant's largest encoding in schema order, or a record's single largest encoding.
/// Nested enumerations use the longest encoding, as [`largest`] does.
///
/// # Errors
/// Returns an error for an unknown declaration, a wrong number of limits, or a
/// limit above its schema ceiling. The schema must have been validated by [`crate::parse`].
pub fn largest_variants(schema: &Schema, declaration: &str, limits: &[u32]) -> Result<Vec<Vec<u8>>, Error> {
    let context = Context::new(schema);
    let mut builder = Builder::new(&context, declaration, limits)?;
    match context.declaration(declaration) {
        Declaration::Record(record) => Ok(vec![builder.record(record)]),
        Declaration::Enum(enumeration) => Ok(enumeration
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| builder.variant(index, variant))
            .collect()),
    }
}

struct Builder<'a> {
    context: &'a Context<'a>,
    bounds: BTreeMap<&'a str, u32>,
    values: BTreeMap<String, Vec<u8>>,
}

impl<'a> Builder<'a> {
    fn new(context: &'a Context<'a>, declaration: &str, limits: &[u32]) -> Result<Self, Error> {
        if !context.declarations.iter().any(|candidate| name(candidate) == declaration) {
            return Err(Error { line: 0, message: format!("unknown declaration {declaration}") });
        }
        if limits.len() != context.bounds.len() {
            return Err(Error {
                line: 0,
                message: format!("expected {} limits, got {}", context.bounds.len(), limits.len()),
            });
        }
        let mut bounds = BTreeMap::new();
        for ((name, ceiling, _), limit) in context.bounds.iter().zip(limits) {
            if limit > ceiling {
                return Err(Error { line: 0, message: format!("limit {name} is {limit}, above ceiling {ceiling}") });
            }
            bounds.insert(name.as_str(), *limit);
        }
        Ok(Self { context, bounds, values: BTreeMap::new() })
    }

    // Named values are built once, including payloads inspected while choosing a variant.
    fn build(&mut self, name: &str) {
        if self.values.contains_key(name) {
            return;
        }
        let bytes = match self.context.declaration(name) {
            Declaration::Record(record) => self.record(record),
            Declaration::Enum(enumeration) => self.enumeration(enumeration),
        };
        self.values.insert(name.to_owned(), bytes);
    }

    fn record(&mut self, record: &Record) -> Vec<u8> {
        let version_len = if record.versioned { 2_usize } else { 0 };
        let length = record.fields.iter().fold(version_len, |length, field| {
            length
                .checked_add(self.value_len(&field.ty, &emit_limits::bound_name(&record.name, &field.name)))
                .expect("checked ceiling")
        });
        let mut bytes = Vec::with_capacity(length);
        if record.versioned {
            bytes.extend_from_slice(&self.context.version.to_be_bytes());
        }
        for field in &record.fields {
            self.write(&field.ty, &emit_limits::bound_name(&record.name, &field.name), &mut bytes);
        }
        bytes
    }

    fn enumeration(&mut self, enumeration: &Enumeration) -> Vec<u8> {
        let mut chosen = 0;
        let mut longest = 0;
        for (index, variant) in enumeration.variants.iter().enumerate() {
            let length = variant.record.as_ref().map_or(0, |name| {
                self.build(name);
                self.values.get(name).expect("built payload").len()
            });
            if length > longest {
                chosen = index;
                longest = length;
            }
        }
        self.variant(chosen, enumeration.variants.get(chosen).expect("enum nonempty"))
    }

    fn variant(&mut self, index: usize, variant: &Variant) -> Vec<u8> {
        let payload = variant.record.as_ref().map(|name| {
            self.build(name);
            self.values.get(name).expect("built payload")
        });
        let length = payload.map_or(0, Vec::len).checked_add(1).expect("checked ceiling");
        let mut bytes = Vec::with_capacity(length);
        bytes.push(u8::try_from(index).expect("at most 256 variants"));
        if let Some(payload) = payload {
            bytes.extend_from_slice(payload);
        }
        bytes
    }

    // Sizing also prepares named values, so writing only copies their cached bytes.
    fn value_len(&mut self, ty: &Type, name: &str) -> usize {
        match ty {
            Type::U8 | Type::Bool => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 | Type::Duration => 8,
            Type::Fixed(length) => usize::try_from(*length).expect("u32 fits usize"),
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => {
                usize::try_from(*self.bounds.get(name).expect("field limit"))
                    .expect("u32 fits usize")
                    .checked_add(4)
                    .expect("checked ceiling")
            }
            Type::List(_, item) => {
                let count = *self.bounds.get(name).expect("list count limit");
                let item_len = if count == 0 {
                    0
                } else {
                    self.value_len(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Item))
                };
                item_len
                    .checked_mul(usize::try_from(count).expect("u32 fits usize"))
                    .and_then(|length| length.checked_add(4))
                    .expect("checked ceiling")
            }
            Type::Option(item) => self
                .value_len(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Some))
                .checked_add(1)
                .expect("checked ceiling"),
            Type::Named(name) => {
                self.build(name);
                self.values.get(name).expect("built declaration").len()
            }
        }
    }

    fn write(&self, ty: &Type, name: &str, bytes: &mut Vec<u8>) {
        match ty {
            Type::U8 => bytes.push(u8::MAX),
            Type::U16 => bytes.extend_from_slice(&u16::MAX.to_be_bytes()),
            Type::U32 => bytes.extend_from_slice(&u32::MAX.to_be_bytes()),
            Type::U64 | Type::Duration => bytes.extend_from_slice(&u64::MAX.to_be_bytes()),
            Type::Bool => bytes.push(1),
            Type::Fixed(length) => fill(bytes, *length, 0xa5),
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => {
                let length = *self.bounds.get(name).expect("field limit");
                bytes.extend_from_slice(&length.to_be_bytes());
                fill(bytes, length, if let Type::Text(_) = ty { b'a' } else { 0xa5 });
            }
            Type::List(_, item) => {
                let count = *self.bounds.get(name).expect("list count limit");
                bytes.extend_from_slice(&count.to_be_bytes());
                if count > 0 {
                    let start = bytes.len();
                    self.write(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Item), bytes);
                    let end = bytes.len();
                    // Empty record items have no bytes to copy, however large the count.
                    if start != end {
                        for _ in 1..count {
                            bytes.extend_from_within(start..end);
                        }
                    }
                }
            }
            Type::Option(item) => {
                bytes.push(1);
                self.write(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Some), bytes);
            }
            Type::Named(name) => bytes.extend_from_slice(self.values.get(name).expect("built declaration")),
        }
    }
}

fn fill(bytes: &mut Vec<u8>, count: u32, value: u8) {
    let end = bytes.len().checked_add(usize::try_from(count).expect("u32 fits usize")).expect("checked ceiling");
    bytes.resize(end, value);
}
