//! Declaration and bound metadata computed once for a generation (data/codec/README.md, 6).

use alloc::collections::BTreeMap;

use crate::{Declaration, Schema, Type, emit_limits};

pub(crate) struct Context<'a> {
    pub(crate) declarations: &'a [Declaration],
    pub(crate) version: u16,
    pub(crate) bounds: Vec<(String, u32, String)>,
    named: BTreeMap<&'a str, &'a Declaration>,
    bounded: BTreeMap<&'a str, bool>,
    minimum: BTreeMap<&'a str, u32>,
    fixed: BTreeMap<&'a str, bool>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(schema: &'a Schema) -> Self {
        let mut context = Self {
            declarations: &schema.declarations,
            version: schema.version,
            bounds: Vec::new(),
            named: BTreeMap::new(),
            bounded: BTreeMap::new(),
            minimum: BTreeMap::new(),
            fixed: BTreeMap::new(),
        };
        for declaration in &schema.declarations {
            let (name, bounded, minimum, fixed) = match declaration {
                Declaration::Record(record) => {
                    let mut minimum = if record.versioned { 2_u32 } else { 0_u32 };
                    for field in &record.fields {
                        let prefix = emit_limits::bound_name(&record.name, &field.name);
                        let path = format!("{}{}", record.name, emit_limits::pascal(&field.name));
                        collect_bounds(&field.ty, &prefix, &path, &mut context.bounds);
                        minimum = minimum.checked_add(context.min_bytes(&field.ty)).expect("checked ceiling");
                    }
                    (
                        record.name.as_str(),
                        record.fields.iter().any(|field| context.has_bounds(&field.ty)),
                        minimum,
                        record.fields.iter().all(|field| context.fixed_bytes(&field.ty).is_some()),
                    )
                }
                Declaration::Enum(enumeration) => {
                    let bounded = enumeration.variants.iter().any(|variant| {
                        variant.record.as_ref().is_some_and(|name| context.has_bounds(&Type::Named(name.clone())))
                    });
                    let minimum = enumeration
                        .variants
                        .iter()
                        .map(|variant| {
                            variant.record.as_ref().map_or(0, |name| context.min_bytes(&Type::Named(name.clone())))
                        })
                        .min()
                        .expect("enum nonempty")
                        .checked_add(1)
                        .expect("checked ceiling");
                    (
                        enumeration.name.as_str(),
                        bounded,
                        minimum,
                        enumeration.variants.iter().all(|variant| variant.record.is_none()),
                    )
                }
            };
            context.named.insert(name, declaration);
            context.bounded.insert(name, bounded);
            context.minimum.insert(name, minimum);
            context.fixed.insert(name, fixed);
        }
        context
    }

    pub(crate) fn declaration(&self, name: &str) -> &Declaration {
        self.named.get(name).expect("named declaration checked")
    }

    pub(crate) fn has_bounds(&self, ty: &Type) -> bool {
        match ty {
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) | Type::List(_, _) => true,
            Type::Option(item) => self.has_bounds(item),
            Type::Named(name) => *self.bounded.get(name.as_str()).expect("earlier declaration checked"),
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => false,
        }
    }

    pub(crate) fn min_bytes(&self, ty: &Type) -> u32 {
        match ty {
            Type::U8 | Type::Bool | Type::Option(_) => 1,
            Type::U16 => 2,
            Type::U32 | Type::Bytes(_) | Type::Secret(_) | Type::Text(_) | Type::List(_, _) => 4,
            Type::U64 | Type::Duration => 8,
            Type::Fixed(bound) => *bound,
            Type::Named(name) => *self.minimum.get(name.as_str()).expect("earlier declaration checked"),
        }
    }

    /// Exact wire bytes when every value of the type has the same width.
    pub(crate) fn fixed_bytes(&self, ty: &Type) -> Option<u32> {
        let fixed = match ty {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Bool | Type::Duration | Type::Fixed(_) => true,
            Type::Named(name) => *self.fixed.get(name.as_str()).expect("earlier declaration checked"),
            Type::Bytes(_) | Type::Secret(_) | Type::Text(_) | Type::List(_, _) | Type::Option(_) => false,
        };
        fixed.then(|| self.min_bytes(ty))
    }
}

fn collect_bounds(ty: &Type, name: &str, path: &str, result: &mut Vec<(String, u32, String)>) {
    match ty {
        Type::Bytes(bound) | Type::Secret(bound) | Type::Text(bound) => result.push((name.into(), *bound, path.into())),
        Type::List(bound, item) => {
            result.push((name.into(), *bound, path.into()));
            collect_bounds(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Item), path, result);
        }
        Type::Option(item) => {
            collect_bounds(item, &emit_limits::child_name(name, emit_limits::LimitLevel::Some), path, result);
        }
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Named(_) => {}
    }
}
