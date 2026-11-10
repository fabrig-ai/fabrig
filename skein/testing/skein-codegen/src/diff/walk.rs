//! Spell the plan's changes, place renames and propose pairings (data/codec/diff.md, 1–3).

use super::spell::{label, name, parts, spell, word};
use super::{Change, Diff, DiffError, Fill, Kind, Op, Rename};
use skein_codec::patch::{self, Comparison, Matching, Target};
use skein_codec::schema::{DeclIx, Schema, Shape, Type};

pub(super) fn matching(old: &Schema, new: &Schema, renames: &[Rename]) -> Result<Matching, DiffError> {
    let mut matching = Matching::new(new);
    let mut errors = Vec::new();
    for part in [false, true] {
        for (index, rename) in renames.iter().enumerate() {
            let checked = match rename {
                Rename::Declaration { old: from, new: to } if !part => {
                    matching.declaration_rename(old, new, from.as_bytes(), to.as_bytes())
                }
                Rename::Part { decl, old: from, new: to } if part => {
                    matching.part_rename(old, new, decl.as_bytes(), from.as_bytes(), to.as_bytes())
                }
                Rename::Declaration { .. } | Rename::Part { .. } => continue,
            };
            match checked {
                Ok((new, old)) => matching.pair(new, old),
                Err(_) => errors.push(index),
            }
        }
        if part {
            matching.pair_parts(old, new);
        } else {
            matching.pair_declarations(old, new);
        }
    }
    if let Some(index) = errors.into_iter().min() { Err(DiffError::Rename(index)) } else { Ok(matching) }
}

fn same_part(matching: &Matching, old: &Schema, new: &Schema, a: Target, b: Target) -> bool {
    match (a, b) {
        (Target::Field(a), Target::Field(b)) => {
            matching.compare(old, new, old.field(a).ty, new.field(b).ty) == Comparison::Equal
        }
        (Target::Variant(a), Target::Variant(b)) => matching.payload(old.variant(a).record, new.variant(b).record),
        _ => false,
    }
}
fn same_layout(matching: &Matching, old: &Schema, new: &Schema, a: DeclIx, b: DeclIx) -> bool {
    let same = match (old.declaration(a).shape, new.declaration(b).shape) {
        (Shape::Record { versioned: av, count: an, .. }, Shape::Record { versioned: bv, count: bn, .. }) => {
            av == bv && an == bn
        }
        (Shape::Enum { count: an, .. }, Shape::Enum { count: bn, .. }) => an == bn,
        _ => false,
    };
    same && parts(old, a).into_iter().zip(parts(new, b)).all(|(a, b)| same_part(matching, old, new, a, b))
}
fn proposals(matching: &Matching, old: &Schema, new: &Schema) -> Vec<Rename> {
    let mut result = Vec::new();
    let kept = (0..old.declarations()).map(DeclIx::new).filter(|at| !matching.used(Target::Declaration(*at)));
    let current = (0..new.declarations()).map(DeclIx::new).filter(|at| matching.old(*at).is_none());
    for (a, b) in kept.zip(current) {
        if same_layout(matching, old, new, a, b) {
            result.push(Rename::Declaration { old: name(old, a.index()), new: name(new, b.index()) });
        }
    }
    for at in (0..new.declarations()).map(DeclIx::new) {
        let Some(kept) = matching.old(at) else {
            continue;
        };
        let a = parts(old, kept).into_iter().filter(|part| !matching.used(*part));
        let b = parts(new, at).into_iter().filter(|part| matching.part(*part).is_none());
        for (a, b) in a.zip(b) {
            if same_part(matching, old, new, a, b) {
                result.push(Rename::Part { decl: name(new, at.index()), old: label(old, a), new: label(new, b) });
            }
        }
    }
    result
}

fn position(schema: &Schema, target: Target) -> (u32, u32) {
    match target {
        Target::Declaration(at) => (at.index(), 0),
        Target::Field(at) => (owner(schema, target).index(), at.index().checked_add(1).expect("bounded field table")),
        Target::Variant(at) => {
            (owner(schema, target).index(), at.index().checked_add(1).expect("bounded variant table"))
        }
    }
}
fn owner(schema: &Schema, target: Target) -> DeclIx {
    if let Target::Declaration(at) = target {
        return at;
    }
    (0..schema.declarations())
        .map(DeclIx::new)
        .find(|at| match (schema.declaration(*at).shape, target) {
            (Shape::Record { first, count, .. }, Target::Field(at)) => {
                (first.index()..first.index().checked_add(count).expect("bounded parts table")).contains(&at.index())
            }
            (Shape::Enum { first, count }, Target::Variant(at)) => {
                (first.index()..first.index().checked_add(count).expect("bounded parts table")).contains(&at.index())
            }
            _ => false,
        })
        .expect("part belongs to a declaration")
}
fn current_owner(old: &Schema, new: &Schema, matching: &Matching, change: &patch::Change) -> DeclIx {
    if change.op != patch::Op::Drop {
        return owner(new, change.target);
    }
    let kept = owner(old, change.target);
    (0..new.declarations()).map(DeclIx::new).find(|at| matching.old(*at) == Some(kept)).expect("matched owner")
}

fn change_position(old: &Schema, new: &Schema, matching: &Matching, change: &patch::Change) -> (u32, u32) {
    match (change.op, change.target) {
        (patch::Op::Drop, Target::Declaration(at)) => (new.declarations(), at.index()),
        (patch::Op::Drop, Target::Field(_) | Target::Variant(_)) => {
            (current_owner(old, new, matching, change).index(), u32::MAX)
        }
        (patch::Op::Order, Target::Declaration(at)) => (at.index(), u32::MAX),
        _ => position(new, change.target),
    }
}

pub(super) fn diff(old: &Schema, new: &Schema, matching: &Matching) -> Diff {
    let mut renames = Vec::new();
    for at in (0..new.declarations()).map(DeclIx::new) {
        if matching.old(at).is_none() {
            continue;
        }
        for target in core::iter::once(Target::Declaration(at)).chain(parts(new, at)) {
            let Some(source) = matching.part(target) else {
                continue;
            };
            let from = label(old, source);
            let to = label(new, target);
            if from != to {
                let op = match target {
                    Target::Declaration(_) => Op::RenameDeclaration { old: from, new: to },
                    Target::Field(_) | Target::Variant(_) => {
                        Op::RenamePart { decl: name(new, at.index()), old: from, new: to }
                    }
                };
                renames.push((position(new, target), Change { kind: Kind::NoneNeeded, op }));
            }
        }
    }
    let mut result = Diff {
        family: word(new, new.family()),
        kept: old.version(),
        current: new.version(),
        changes: Vec::new(),
        proposed: proposals(matching, old, new),
    };
    let mut renames = renames.into_iter().peekable();
    for change in &patch::changes(old, new, matching) {
        let at = change_position(old, new, matching, change);
        while renames.peek().is_some_and(|(position, _)| *position <= at) {
            result.changes.push(renames.next().expect("peeked rename").1);
        }
        let op = operation(old, new, matching, change);
        let declaration = matches!(change.target, Target::Declaration(_));
        let kind = if change.written.is_some() {
            Kind::Written
        } else if change.implicit || (declaration && matches!(change.op, patch::Op::Add | patch::Op::Drop)) {
            Kind::NoneNeeded
        } else {
            Kind::Derived
        };
        result.changes.push(Change { kind, op });
    }
    result.changes.extend(renames.map(|(_, change)| change));
    result
}

fn operation(old: &Schema, new: &Schema, matching: &Matching, change: &patch::Change) -> Op {
    use patch::Op as P;
    let schema = if change.op == P::Drop { old } else { new };
    let part = label(schema, change.target);
    let decl = if let (P::Drop, Target::Declaration(_)) = (change.op, change.target) {
        part.clone()
    } else {
        name(new, current_owner(old, new, matching, change).index())
    };
    match (change.op, change.target) {
        (P::Add, Target::Declaration(_)) => Op::AddDeclaration { decl },
        (P::Drop, Target::Declaration(_)) => Op::DropDeclaration { decl },
        (P::Reshape, Target::Declaration(_)) => Op::Reshape { decl },
        (P::Versioned | P::Unversioned, Target::Declaration(_)) => {
            Op::Versioned { decl, versioned: change.op == P::Versioned }
        }
        (P::Add, Target::Field(at)) => {
            let ty = new.field(at).ty;
            let value = if matches!(new.ty(ty), Type::Option(_)) { Fill::Absent } else { Fill::Asked };
            Op::AddField { decl, field: part, ty: spell(new, ty), value }
        }
        (P::Drop, Target::Field(_)) => Op::DropField { decl, field: part },
        (P::Add, Target::Variant(_)) => Op::AddVariant { decl, variant: part },
        (P::Drop, Target::Variant(_)) => Op::DropVariant { decl, variant: part },
        (P::Widen | P::Wrap | P::Retype, Target::Field(at)) => {
            let Some(Target::Field(source)) = matching.part(change.target) else {
                unreachable!("matched field");
            };
            let kept = spell(old, old.field(source).ty);
            let current = spell(new, new.field(at).ty);
            match change.op {
                P::Widen => Op::Widen { decl, field: part, kept, current },
                P::Wrap => Op::Wrap { decl, field: part, kept, current },
                P::Retype => Op::Retype { decl, field: part, kept, current },
                P::Add | P::Drop | P::Reshape | P::Versioned | P::Unversioned | P::Order | P::Payload | P::Written => {
                    unreachable!("type change")
                }
            }
        }
        (P::Payload, Target::Variant(at)) => {
            let Some(Target::Variant(source)) = matching.part(change.target) else {
                unreachable!("matched variant");
            };
            Op::Payload {
                decl,
                variant: part,
                kept: old.variant(source).record.map(|at| name(old, at.index())),
                current: new.variant(at).record.map(|at| name(new, at.index())),
            }
        }
        (P::Order, Target::Declaration(at)) => {
            let parts = parts(new, at)
                .into_iter()
                .filter(|part| matching.part(*part).is_some())
                .map(|part| label(new, part))
                .collect();
            Op::Order { decl, parts }
        }
        _ => unreachable!("operation has its planned target"),
    }
}
