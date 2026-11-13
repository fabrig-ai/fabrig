//! Reverses of checked forward changes (data/codec/bridges.md, 2–3 and 5.3).

use super::{
    Block, Comparison, DeclIx, FieldIx, LineKind, Matching, Op, Parsed, PatchError, PatchWrong, Plan, Schema, Shape,
    Source, Span, Tables, Target, Type, TypeIx, VariantIx, add, compared, conversion, empty_plan, fill, keep_text,
    matching, parts, resolve, site_for, value,
};
use skein_lib::List;

fn inverted(matching: &Matching, old: &Schema) -> Matching {
    let mut inverse = Matching::new(old);
    for at in 0..matching.declarations.len() {
        if let Some(old) = matching.old(DeclIx::new(at)) {
            inverse.pair(Target::Declaration(old), Target::Declaration(DeclIx::new(at)));
        }
    }
    for at in 0..matching.fields.len() {
        if let Some(old) = *matching.fields.get(at).expect("field match") {
            inverse.pair(Target::Field(old), Target::Field(FieldIx::new(at)));
        }
    }
    for at in 0..matching.variants.len() {
        if let Some(old) = *matching.variants.get(at).expect("variant match") {
            inverse.pair(Target::Variant(old), Target::Variant(VariantIx::new(at)));
        }
    }
    inverse
}

/// Checks down lines even when the caller plans forward alone.
pub(super) fn down(
    tables: &Tables,
    patch: &[u8],
    parsed: &Parsed,
    block: Block,
    matching: &Matching,
) -> Result<Plan, PatchError> {
    let older_index = tables.schema_index(block.from).ok_or(block.error(PatchWrong::Unkept))?;
    let newer_index = tables.schema_index(block.to).ok_or(block.error(PatchWrong::Unkept))?;
    let older = tables.schemas.get(older_index).expect("kept older text");
    let newer = tables.schemas.get(newer_index).expect("kept newer text");
    let inverse = inverted(matching, older);
    let pool = parsed
        .tokens
        .len()
        .checked_mul(8)
        .expect("bounded tokens")
        .checked_add(u32::try_from(patch.len()).expect("text ceiling"))
        .expect("bounded value pool");
    let mut plan = empty_plan(newer, older, newer_index, older_index, parsed.tokens.len(), pool);
    plan.reverse = true;
    for &opposite in &matching.declarations {
        plan.opposites.push(opposite).expect("source declaration room");
    }
    keep_text(&mut plan, patch, parsed);
    for at in block.lines.start..block.lines.end {
        let line = *parsed.lines.get(at).expect("block line");
        match line.kind {
            LineKind::DownValue => {
                let (decl, target) = resolve(older, patch, line)?;
                let field = match target {
                    Target::Field(field) => field,
                    Target::Declaration(_) | Target::Variant(_) => return Err(line.error(PatchWrong::Site)),
                };
                if inverse.old(decl).is_none() || matching.used(target) {
                    return Err(line.error(PatchWrong::Site));
                }
                if site_for(&plan, Target::Declaration(decl)).is_some() {
                    return Err(line.error(PatchWrong::Site));
                }
                if *plan.sources.get(field.index()).expect("destination field") != Source::Written {
                    return Err(line.error(PatchWrong::Twice));
                }
                let source = value(&mut plan.pool, older, older.field(field).ty, patch, parsed, line.value)?;
                if source == Source::Written {
                    return Err(line.error(PatchWrong::Site));
                }
                *plan.sources.get_mut(field.index()).expect("destination field") = source;
            }
            LineKind::DownSite => {
                let (_, target) = resolve(newer, patch, line)?;
                let destination = matching.part(target).ok_or(line.error(PatchWrong::Site))?;
                let decl = match destination {
                    Target::Declaration(decl) => decl,
                    Target::Field(field) => field_owner(older, field),
                    Target::Variant(_) => return Err(line.error(PatchWrong::Site)),
                };
                super::check_site(&plan, &inverse, older, decl, destination, line)?;
                match destination {
                    Target::Declaration(_) => {
                        let (first, count) = parts(older.declaration(decl).shape);
                        match older.declaration(decl).shape {
                            Shape::Record { .. } => {
                                for field in first..add(first, count) {
                                    if *plan.sources.get(field).expect("destination field") != Source::Written {
                                        return Err(line.error(PatchWrong::Site));
                                    }
                                }
                            }
                            Shape::Enum { .. } => {}
                        }
                    }
                    Target::Field(_) | Target::Variant(_) => {}
                }
                let mut first = plan.refusals.len();
                let mut end = first;
                for (index, refusal) in plan.refusals.iter().enumerate() {
                    if refusal.site == line.index {
                        let index = u32::try_from(index).expect("bounded refusal index");
                        first = first.min(index);
                        end = add(index, 1);
                    }
                }
                plan.sites
                    .push(super::SitePlan { index: line.index, target: destination, refusals: (first, end) })
                    .expect("site room");
            }
            LineKind::Operation(_) | LineKind::Rename | LineKind::Site | LineKind::Refusal | LineKind::Breaking => {}
        }
    }
    Ok(plan)
}

fn field_owner(schema: &Schema, field: FieldIx) -> DeclIx {
    for at in 0..schema.declarations() {
        let decl = DeclIx::new(at);
        match schema.declaration(decl).shape {
            Shape::Record { first, count, .. } => {
                if field.index() >= first.index() && field.index() < add(first.index(), count) {
                    return decl;
                }
            }
            Shape::Enum { .. } => {}
        }
    }
    unreachable!("field belongs to schema")
}

pub(in crate::patch) fn reverse(
    tables: &Tables,
    patch: &[u8],
    parsed: &Parsed,
    block: Block,
) -> Result<Plan, PatchError> {
    let forward = super::build(tables, patch, parsed, block)?;
    for at in block.lines.start..block.lines.end {
        let line = *parsed.lines.get(at).expect("block line");
        if line.kind == LineKind::Breaking {
            return Err(line.error(PatchWrong::Breaking));
        }
    }
    let older = tables.schemas.get(forward.old).expect("older text");
    let newer = tables.schemas.get(forward.new).expect("newer text");
    let mut errors = List::with_capacity(parsed.lines.len());
    for _ in 0..parsed.lines.len() {
        errors.push(None).expect("line error room");
    }
    let matching = matching(older, newer, patch, parsed, block, &mut errors);
    let mut plan = down(tables, patch, parsed, block, &matching)?;
    let inverse = inverted(&matching, older);
    // A missing authored reverse names the forward operation, not the family header.
    for at in block.lines.start..block.lines.end {
        let line = *parsed.lines.get(at).expect("block line");
        let op = match line.kind {
            LineKind::Operation(op) => op,
            LineKind::Rename
            | LineKind::Site
            | LineKind::Refusal
            | LineKind::DownValue
            | LineKind::DownSite
            | LineKind::Breaking => continue,
        };
        let schema = if op == Op::Drop { older } else { newer };
        let (decl, target) = resolve(schema, patch, line)?;
        let destination = if op == Op::Drop { Some(target) } else { matching.part(target) };
        let destination_decl = if op == Op::Drop { Some(decl) } else { matching.old(decl) };
        let whole = match destination_decl {
            Some(decl) => site_for(&plan, Target::Declaration(decl)).is_some(),
            None => false,
        };
        let authored = whole
            || match destination {
                Some(target) => site_for(&plan, target).is_some(),
                None => false,
            };
        let needs = match op {
            Op::Drop => match target {
                Target::Field(field) => {
                    *plan.sources.get(field.index()).expect("dropped field source") == Source::Written
                }
                Target::Declaration(_) | Target::Variant(_) => false,
            },
            Op::Reshape | Op::Retype | Op::Payload => true,
            Op::Add | Op::Versioned | Op::Unversioned | Op::Widen | Op::Wrap | Op::Order | Op::Written => false,
        };
        if needs && !authored {
            return Err(line.error(PatchWrong::Irreversible));
        }
    }
    // Constants for fields absent in the older text are compared before dropping them.
    for at in 0..matching.fields.len() {
        if matching.fields.get(at) == Some(&None) {
            match *forward.sources.get(at).expect("forward source") {
                Source::Value { start, end } => {
                    let begin = plan.pool.len();
                    for byte in forward
                        .pool
                        .as_slice()
                        .get(skein_lib::usize_of(start)..skein_lib::usize_of(end))
                        .expect("forward value")
                    {
                        plan.pool.push(*byte).expect("value pool room");
                    }
                    let span = Span { start: begin, end: plan.pool.len() };
                    *plan.guards.get_mut(at).expect("source field guard") = Some(span);
                }
                Source::Old(_) | Source::Site(_) | Source::Written => {}
            }
        }
    }
    if let Err(wrong) = fill(&mut plan, newer, older, &inverse, false) {
        return Err(block.error(wrong));
    }
    Ok(plan)
}

fn site_total(plan: &Plan, index: u32) -> bool {
    for site in &plan.sites {
        if site.index == index {
            return site.refusals.0 == site.refusals.1;
        }
    }
    false
}

fn type_total(plan: &Plan, schema: &Schema, mut ty: TypeIx) -> bool {
    for _ in 0..schema.text().len() {
        match schema.ty(ty) {
            Type::Option(item) | Type::List { item, .. } => ty = item,
            Type::Named(at)
            | Type::Bytes { held: super::Held::Pieces(at), .. }
            | Type::Secret { held: super::Held::Pieces(at), .. } => return conversion(plan, at).total,
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Bytes { held: super::Held::Opaque | super::Held::Record { .. }, .. }
            | Type::Secret { held: super::Held::Opaque | super::Held::Record { .. }, .. } => return true,
        }
    }
    false
}

pub(super) fn totals(plan: &mut Plan, old: &Schema, new: &Schema, matching: &Matching) {
    for at in 0..old.declarations() {
        let decl = DeclIx::new(at);
        let converted = conversion(plan, decl);
        let mut total = converted.new.is_some();
        match converted.site {
            Some(index) => total = site_total(plan, index),
            None => match old.declaration(decl).shape {
                Shape::Record { count: old_count, .. } => {
                    let mut matched = 0_u32;
                    if let Some(new_decl) = converted.new {
                        match new.declaration(new_decl).shape {
                            Shape::Record { first, count, .. } => {
                                for offset in 0..count {
                                    let field = FieldIx::new(add(first.index(), offset));
                                    match *plan.sources.get(field.index()).expect("destination field source") {
                                        Source::Old(source) => {
                                            matched = add(matched, 1);
                                            let a = old.field(source).ty;
                                            let b = new.field(field).ty;
                                            total &= compared(plan, old, new, a, b) == Comparison::Equal
                                                && type_total(plan, old, a);
                                        }
                                        Source::Site(index) => {
                                            total &= site_total(plan, index);
                                            if matching.part(Target::Field(field)).is_some() {
                                                matched = add(matched, 1);
                                            }
                                        }
                                        Source::Value { .. } | Source::Written => {}
                                    }
                                }
                            }
                            Shape::Enum { .. } => total = false,
                        }
                    }
                    if plan.reverse && matched != old_count {
                        total = false;
                    }
                }
                Shape::Enum { first, count } => {
                    for offset in 0..count {
                        let variant = VariantIx::new(add(first.index(), offset));
                        total &= plan.variants.get(variant.index()).copied().flatten().is_some();
                        if let Some(record) = old.variant(variant).record {
                            total &= conversion(plan, record).total;
                        }
                    }
                }
            },
        }
        plan.declarations.get_mut(at).expect("declaration slot").total = total;
    }
}
