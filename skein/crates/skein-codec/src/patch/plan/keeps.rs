//! Byte validity and head-only rewriting (data/codec/patches.md, 3.1).
use super::{Comparison, Plan, Source, compare, conversion, payload};
use crate::schema::{DeclIx, FieldIx, Held, Schema, Shape, Type, TypeIx, VariantIx};

fn type_keeps(plan: &Plan, old: &Schema, new: &Schema, mut a: TypeIx, mut b: TypeIx) -> bool {
    match compare(&plan.olds, old, new, a, b) {
        Comparison::Equal | Comparison::Widen { integer: false } => {}
        Comparison::Widen { integer: true } | Comparison::Wrap | Comparison::Retype => return false,
    }
    for _ in 0..old.text().len() {
        let av = old.ty(a);
        let bv = new.ty(b);
        if held_record(av) || held_record(bv) {
            return false;
        }
        match av {
            Type::List { item, .. } => {
                let Type::List { item: bi, .. } = bv else { unreachable!("compared list levels") };
                a = item;
                b = bi;
            }
            Type::Option(item) => {
                let Type::Option(bi) = bv else { unreachable!("compared option levels") };
                a = item;
                b = bi;
            }
            Type::Named(at) => return conversion(plan, at).keeps,
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::Secret { .. } => {
                if let Some(at) = super::piece_type(av)
                    && !conversion(plan, at).keeps
                {
                    return false;
                }
                if let Some(at) = super::piece_type(bv) {
                    let Some(source) = *plan.olds.get(at.index()).expect("new piece item") else { return false };
                    if !conversion(plan, source).keeps {
                        return false;
                    }
                }
                return true;
            }
        }
    }
    unreachable!("wrapper chains bounded by schema text")
}

fn held_record(ty: Type) -> bool {
    match ty {
        Type::Bytes { held: Held::Record { .. }, .. } | Type::Secret { held: Held::Record { .. }, .. } => true,
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Text(_)
        | Type::Bytes { held: Held::Opaque | Held::Pieces(_), .. }
        | Type::Secret { held: Held::Opaque | Held::Pieces(_), .. }
        | Type::List { .. }
        | Type::Option(_)
        | Type::Named(_) => false,
    }
}

/// Compute after the declarations this one references, in the same pass as copies.
pub(super) fn fill(plan: &mut Plan, old: &Schema, new: &Schema, at: DeclIx, identity: bool) {
    let mut converted = conversion(plan, at);
    let Some(to) = converted.new else { return };
    if converted.site.is_some() {
        return;
    }
    match (old.declaration(at).shape, new.declaration(to).shape) {
        (
            Shape::Record { first: af, count: ac, versioned: av },
            Shape::Record { first: bf, count: bc, versioned: bv },
        ) => {
            if ac != bc || av != bv {
                return;
            }
            let mut all = true;
            let mut tail = true;
            for offset in 0..ac {
                let a = FieldIx::new(super::add(af.index(), offset));
                let b = FieldIx::new(super::add(bf.index(), offset));
                if *plan.sources.get(b.index()).expect("new field") != Source::Old(a) {
                    return;
                }
                let keeps = type_keeps(plan, old, new, old.field(a).ty, new.field(b).ty);
                all &= keeps;
                if offset == 0 {
                    tail &= match compare(&plan.olds, old, new, old.field(a).ty, new.field(b).ty) {
                        Comparison::Equal | Comparison::Widen { integer: false } => true,
                        Comparison::Widen { integer: true } | Comparison::Wrap | Comparison::Retype => false,
                    };
                } else {
                    tail &= keeps;
                }
            }
            converted.root_keeps = all;
            converted.tail_keeps = tail;
            converted.keeps = all && (!av || identity);
        }
        (Shape::Enum { first: af, count: ac }, Shape::Enum { first: bf, count: bc }) => {
            let mut all = ac <= bc;
            for offset in 0..ac {
                let a = VariantIx::new(super::add(af.index(), offset));
                let b = VariantIx::new(super::add(bf.index(), offset));
                let mut keeps = offset < bc && *plan.variants.get(a.index()).expect("old variant") == Some(b);
                if keeps {
                    let ar = old.variant(a).record;
                    keeps &= payload(&plan.olds, ar, new.variant(b).record);
                    if let Some(record) = ar {
                        keeps &= conversion(plan, record).keeps;
                    }
                }
                *plan.variant_keeps.get_mut(a.index()).expect("old variant") = keeps;
                all &= keeps;
            }
            converted.keeps = all;
            converted.root_keeps = all;
            converted.tail_keeps = true;
        }
        (Shape::Record { .. }, Shape::Enum { .. }) | (Shape::Enum { .. }, Shape::Record { .. }) => {}
    }
    *plan.declarations.get_mut(at.index()).expect("old declaration") = converted;
}

fn leading(schema: &Schema, root: DeclIx) -> Option<DeclIx> {
    match schema.declaration(root).shape {
        Shape::Enum { .. } => Some(root),
        Shape::Record { first, count, .. } => {
            if count == 0 {
                return None;
            }
            if let Type::Named(at) = schema.ty(schema.field(first).ty)
                && let Shape::Enum { .. } = schema.declaration(at).shape
            {
                return Some(at);
            }
            None
        }
    }
}

pub(in crate::patch) fn rehead(plan: &Plan, old: &Schema, root: DeclIx, record: &mut [u8]) -> bool {
    let converted = conversion(plan, root);
    let headed = match old.declaration(root).shape {
        Shape::Record { versioned, .. } => versioned,
        Shape::Enum { .. } => false,
    };
    if headed && crate::version_of(record) != Ok(plan.from) {
        return false;
    }
    let mut keeps = converted.root_keeps;
    if let Some(enumeration) = leading(old, root) {
        let Some(tag) = record.get(if headed { 2 } else { 0 }) else { return false };
        let Shape::Enum { first, count } = old.declaration(enumeration).shape else {
            unreachable!("leading enumeration")
        };
        if u32::from(*tag) >= count {
            return false;
        }
        let variant = super::add(first.index(), u32::from(*tag));
        keeps |= converted.tail_keeps && *plan.variant_keeps.get(variant).expect("old variant");
    }
    if keeps && headed {
        for (slot, byte) in record.iter_mut().zip(plan.to.to_be_bytes()) {
            *slot = byte;
        }
    }
    keeps
}
