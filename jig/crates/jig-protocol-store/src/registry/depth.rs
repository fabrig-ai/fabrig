//! A conservative depth proof for identical inputs (protocol/store/layouts.md, 5.3).
//! Heights include declarations, containers and resolved contents, as in codec's
//! `layout/derive.rs::finish`. Unknown heights fall back to derivation; no plan is built.
use skein_codec::{
    layout::MAX_DEPTH,
    schema::{DeclIx, FieldIx, Held, MAX_HELD, Schema, Shape, Type, TypeIx, VariantIx},
};
use skein_lib::List;

/// A broad root uses its whole declaration's height: a deep unselected sibling
/// may prevent this proof, but cannot admit a layout the plan would refuse.
pub(super) fn heights(primary: &Schema, held: &[&Schema]) -> List<u32> {
    let mut texts = List::uncapped();
    texts.push(primary).expect("primary schema");
    for &schema in held {
        texts.push(schema).expect("held schema");
    }
    let mut heights = List::uncapped();
    for schema in &texts {
        let mut declarations = List::with_capacity(schema.declarations());
        for _ in 0..schema.declarations() {
            declarations.push(0).expect("one height per declaration");
        }
        heights.push(declarations).expect("one table per text");
    }
    // Local names precede their users. Each pass can resolve another held
    // family along a path; layout validation already bounded those paths.
    for _ in 0..=MAX_HELD {
        let mut changed = false;
        for source in 0..texts.len() {
            let schema = texts.get(source).expect("text table");
            for at in 0..schema.declarations() {
                if height(&heights, source, DeclIx::new(at)).is_some() {
                    continue;
                }
                if let Some(depth) = declaration(schema, DeclIx::new(at), source, held, &heights)
                    && depth <= MAX_DEPTH
                {
                    *heights.get_mut(source).expect("text table").get_mut(at).expect("declaration slot") = depth;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    heights.swap_remove(0).expect("primary heights")
}

fn height(heights: &List<List<u32>>, source: u32, decl: DeclIx) -> Option<u32> {
    let height = *heights.get(source)?.get(decl.index())?;
    if height == 0 { None } else { Some(height) }
}

fn declaration(schema: &Schema, decl: DeclIx, source: u32, held: &[&Schema], heights: &List<List<u32>>) -> Option<u32> {
    let mut depth = 1;
    match schema.declaration(decl).shape {
        Shape::Record { first, count, .. } => {
            for offset in 0..count {
                let field = schema.field(FieldIx::new(first.index().checked_add(offset)?));
                let child = ty(schema, field.ty, source, held, heights)?;
                depth = depth.max(child.checked_add(1)?);
            }
        }
        Shape::Enum { first, count } => {
            for offset in 0..count {
                let variant = schema.variant(VariantIx::new(first.index().checked_add(offset)?));
                if let Some(record) = variant.record {
                    depth = depth.max(height(heights, source, record)?.checked_add(1)?);
                }
            }
        }
    }
    Some(depth)
}

fn ty(schema: &Schema, mut at: TypeIx, source: u32, held: &[&Schema], heights: &List<List<u32>>) -> Option<u32> {
    let mut wrappers = 0_u32;
    for _ in 0..MAX_DEPTH {
        let terminal = match schema.ty(at) {
            Type::List { item, .. } | Type::Option(item) => {
                wrappers = wrappers.checked_add(1)?;
                at = item;
                continue;
            }
            // Named readers add no workspace depth, but occupy a dependency
            // round in finish's bounded propagation. Count them conservatively.
            Type::Named(decl) => height(heights, source, decl)?.checked_add(1)?,
            Type::Bytes { held: Held::Pieces(decl), .. } | Type::Secret { held: Held::Pieces(decl), .. } => {
                height(heights, source, decl)?.checked_add(2)?
            }
            Type::Bytes { held: Held::Record { family, name }, .. }
            | Type::Secret { held: Held::Record { family, name }, .. } => {
                let mut depth = 1;
                for (index, &text) in held.iter().enumerate() {
                    if text.spelled(text.family()) == schema.spelled(family) {
                        let source = u32::try_from(index).ok()?.checked_add(1)?;
                        let decl = text.find(schema.spelled(name))?;
                        depth = height(heights, source, decl)?.checked_add(1)?;
                        break;
                    }
                }
                depth
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Bytes { held: Held::Opaque, .. }
            | Type::Secret { held: Held::Opaque, .. } => 1,
        };
        return terminal.checked_add(wrappers);
    }
    None
}
