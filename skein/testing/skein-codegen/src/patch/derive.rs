//! The run and check share matching, frozen texts and version verdicts.

use alloc::collections::{BTreeMap, BTreeSet};
use core::fmt::Write;
use skein_codec::schema::{self, FieldIx, Held, Schema, Shape, Type, VariantIx};

use super::Error;
use super::files::{Crate, Text, patch_path};
use super::text::{Block, Line, PatchText};
use crate::diff::{self, Diff, Fill, Kind, Op, Rename, Version};

pub(super) fn patches(tree: &Crate) -> Result<BTreeMap<u32, PatchText>, Error> {
    tree.patches
        .iter()
        .map(|(number, source)| {
            let patch = PatchText::read(source).ok_or_else(|| Error::Schema {
                path: patch_path(*number),
                error: "patch statements do not read".into(),
            })?;
            Ok((*number, patch))
        })
        .collect()
}

pub(super) fn frozen(
    patches: &BTreeMap<u32, PatchText>,
    base: &Crate,
    wires: &BTreeSet<String>,
) -> BTreeSet<(String, u16)> {
    let wire_texts = base
        .kept
        .iter()
        .filter(|(family, _)| wires.contains(*family))
        .flat_map(|(family, texts)| texts.keys().map(|version| (family.clone(), *version)));
    patches
        .values()
        .flat_map(|patch| {
            patch.blocks.iter().flat_map(|block| [(block.family.clone(), block.from), (block.family.clone(), block.to)])
        })
        .chain(wire_texts)
        .collect()
}

pub(super) fn branch(tree: &Crate, base: &Crate) -> Result<Option<u32>, Error> {
    let own: Vec<_> = tree.patches.keys().filter(|number| !base.patches.contains_key(number)).copied().collect();
    if own.len() > 1 {
        return Err(Error::Unlanded { patches: own });
    }
    Ok(own.first().copied().or_else(|| {
        tree.patches
            .iter()
            .rev()
            .find_map(|(number, text)| base.patches.get(number).filter(|landed| *landed != text).map(|_| *number))
    }))
}

pub(super) fn rename(line: &str) -> Option<Rename> {
    let words: Vec<_> = line.split_whitespace().collect();
    let ["rename", old, new] = words.as_slice() else { return None };
    Some(if let Some((decl, old)) = old.split_once('.') {
        Rename::Part { decl: decl.into(), old: old.into(), new: (*new).into() }
    } else {
        Rename::Declaration { old: (*old).into(), new: (*new).into() }
    })
}

pub(super) fn confirmed(tree: &Crate, family: &str, blocks: &[&Block], given: &[(String, Rename)]) -> Vec<Rename> {
    let waiting = tree.renames.get(family).into_iter().flat_map(|text| text.lines()).filter_map(rename);
    let written = blocks.iter().flat_map(|block| &block.lines).filter_map(|line| match line {
        Line::Op(line) => rename(line),
        Line::Site { .. } => None,
    });
    let options = given.iter().filter(|(name, _)| name == family).map(|(_, rename)| rename.clone());
    let mut result = Vec::new();
    for rename in waiting.chain(options).chain(written) {
        if !result.contains(&rename) {
            result.push(rename);
        }
    }
    result
}

pub(super) fn schema(text: &Text) -> Result<Schema, Error> {
    schema::parse(text.source.as_bytes()).map_err(|error| Error::Schema {
        path: text.path.clone(),
        error: format!("schema refused at line {}", error.line),
    })
}

pub(super) fn compare(kept: &Text, current: &Text, renames: &[Rename]) -> Result<Diff, Error> {
    diff::diff(&schema(kept)?, &schema(current)?, renames)
        .map_err(|error| Error::Diff { family: current.family.clone(), error })
}

/// A refresh records confirmed names in the kept text until this branch lands.
/// Recover those pairings only when the kept edit needs no layout conversion.
pub(super) fn refreshed_renames(kept: &Text, tree: &Crate) -> Result<Vec<Rename>, Error> {
    let Some(refreshed) = tree.kept.get(&kept.family).and_then(|texts| texts.get(&kept.version)) else {
        return Ok(Vec::new());
    };
    if refreshed.source == kept.source {
        return Ok(Vec::new());
    }
    let first = compare(kept, refreshed, &[])?;
    let renames = proposals(kept, refreshed, &[], &first)?;
    if needs(&compare(kept, refreshed, &renames)?, false, false) { Ok(Vec::new()) } else { Ok(renames) }
}

/// Proposals affect only later proposal rounds, never the returned diff.
pub(super) fn proposals(kept: &Text, current: &Text, confirmed: &[Rename], first: &Diff) -> Result<Vec<Rename>, Error> {
    let mut accepted = confirmed.to_vec();
    let mut proposed = Vec::new();
    let mut round = first.clone();
    loop {
        let next: Vec<_> = round.proposed.iter().filter(|rename| !accepted.contains(rename)).cloned().collect();
        if next.is_empty() {
            return Ok(proposed);
        }
        accepted.extend(next.iter().cloned());
        proposed.extend(next);
        round = compare(kept, current, &accepted)?;
    }
}

/// Compares wire layouts with the current header at the kept version, including
/// `versioned` records (data/codec/command.md, 2; data/codec/README.md, 9.1).
/// Schema tokens contain no quoted text; comments and whitespace are not layout.
pub(super) fn layout_changed(kept: &Text, current: &Text) -> Result<bool, Error> {
    let tokens = current.source.lines().flat_map(|line| line.split('#').next().unwrap_or_default().split_whitespace());
    let source =
        format!("family {} {}\n{}", current.family, kept.version, tokens.skip(3).collect::<Vec<_>>().join(" "));
    let normalized = Text { source, ..current.clone() };
    Ok(schema(kept)?.fingerprints().family() != schema(&normalized)?.fingerprints().family())
}

pub(super) fn needs(diff: &Diff, frozen: bool, layout_changed: bool) -> bool {
    layout_changed
        || diff.changes.iter().any(|change| {
            change.kind != Kind::NoneNeeded
                || (frozen
                    && match &change.op {
                        Op::RenameDeclaration { .. } | Op::RenamePart { .. } | Op::DropDeclaration { .. } => false,
                        Op::AddDeclaration { .. }
                        | Op::Reshape { .. }
                        | Op::Versioned { .. }
                        | Op::AddField { .. }
                        | Op::DropField { .. }
                        | Op::Widen { .. }
                        | Op::Wrap { .. }
                        | Op::Retype { .. }
                        | Op::Order { .. }
                        | Op::AddVariant { .. }
                        | Op::DropVariant { .. }
                        | Op::Payload { .. } => true,
                    })
        })
}

pub(super) fn verdict(diff: &Diff, needs: bool) -> Version {
    if diff.current == diff.kept {
        if needs { Version::Unmoved } else { Version::Kept }
    } else if diff.kept.checked_add(1) == Some(diff.current) {
        if needs { Version::Moved } else { Version::Needless }
    } else {
        Version::Wrong
    }
}

pub(super) fn target_exists(schema: &Schema, target: &str) -> bool {
    let (decl, part) = target.split_once('.').map_or((target, None), |(decl, part)| (decl, Some(part)));
    let Some(decl) = schema.find(decl.as_bytes()) else { return false };
    let Some(part) = part else { return true };
    match schema.declaration(decl).shape {
        Shape::Record { first, count, .. } => (first.index()
            ..first.index().checked_add(count).expect("bounded fields"))
            .any(|at| schema.spelled(schema.field(FieldIx::new(at)).name) == part.as_bytes()),
        Shape::Enum { first, count } => (first.index()..first.index().checked_add(count).expect("bounded variants"))
            .any(|at| schema.spelled(schema.variant(VariantIx::new(at)).name) == part.as_bytes()),
    }
}

fn declaration(line: &str) -> &str {
    let mut words = line.split_whitespace();
    let operation = words.next().unwrap_or_default();
    let target = words.next().unwrap_or_default();
    if operation == "rename" && !target.contains('.') {
        words.next().unwrap_or(target)
    } else {
        target.split_once('.').map_or(target, |(decl, _)| decl)
    }
}

/// Whether an added field's type takes no value a patch can spell (data/codec/patches.md,
/// 2): a record, an enumeration whose every variant holds one, a secret,
/// a held record.
fn takes_no_value(schema: &Schema, decl: &str, field: &str) -> bool {
    let Some(decl) = schema.find(decl.as_bytes()) else { return false };
    let Shape::Record { first, count, .. } = schema.declaration(decl).shape else { return false };
    let Some(field) = (first.index()..first.index().checked_add(count).expect("bounded fields"))
        .map(|at| schema.field(FieldIx::new(at)))
        .find(|candidate| schema.spelled(candidate.name) == field.as_bytes())
    else {
        return false;
    };
    match schema.ty(field.ty) {
        Type::Secret { .. } => true,
        Type::Bytes { held, .. } => matches!(held, Held::Record { .. }),
        Type::Named(named) => match schema.declaration(named).shape {
            Shape::Record { .. } => true,
            Shape::Enum { first, count } => (first.index()
                ..first.index().checked_add(count).expect("bounded variants"))
                .all(|at| schema.variant(VariantIx::new(at)).record.is_some()),
        },
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Text(_)
        | Type::List { .. }
        | Type::Option(_) => false,
    }
}

/// Rebuilds operations, retaining values only for the identical added type;
/// an added field whose type takes no value is `written` at its own site,
/// or at its declaration's.
#[expect(
    clippy::too_many_lines,
    reason = "derives forward operations and preserves the author's sites and down lines together"
)]
pub(super) fn block(diff: &Diff, current: &Text, old: Option<&Block>) -> Result<Block, Error> {
    let schema = schema(current)?;
    let mut operations = Vec::new();
    let mut valueless = Vec::new();
    for change in &diff.changes {
        let mut line = change.op.to_string();
        if let Op::AddField { decl, field, ty, value } = &change.op {
            let prefix = format!("add {decl}.{field} {ty} ");
            if let Some(value) = old.into_iter().flat_map(|block| &block.lines).find_map(|line| match line {
                Line::Op(line) => line.strip_prefix(&prefix),
                Line::Site { .. } => None,
            }) {
                line = format!("{prefix}{value}");
            } else if *value == Fill::Asked && takes_no_value(&schema, decl, field) {
                line = format!("{prefix}written");
            }
            if line == format!("{prefix}written") {
                valueless.push(format!("{decl}.{field}"));
            }
        }
        operations.push(line);
    }
    let mut sites: Vec<Line> = old
        .into_iter()
        .flat_map(|block| &block.lines)
        .filter(|line| match line {
            Line::Site { target, down, .. } => !down && target_exists(&schema, target),
            Line::Op(_) => false,
        })
        .cloned()
        .collect();
    for target in diff.written() {
        // A field site may supply a retype that the diff names as a declaration site.
        let field_retypes: Vec<_> = diff
            .changes
            .iter()
            .filter_map(|change| match &change.op {
                Op::Retype { decl, field, .. } if *decl == target => Some(format!("{decl}.{field}")),
                Op::AddDeclaration { .. }
                | Op::DropDeclaration { .. }
                | Op::RenameDeclaration { .. }
                | Op::Reshape { .. }
                | Op::Versioned { .. }
                | Op::AddField { .. }
                | Op::DropField { .. }
                | Op::RenamePart { .. }
                | Op::Widen { .. }
                | Op::Wrap { .. }
                | Op::Retype { .. }
                | Op::Order { .. }
                | Op::AddVariant { .. }
                | Op::DropVariant { .. }
                | Op::Payload { .. } => None,
            })
            .collect();
        let only_retypes = diff
            .changes
            .iter()
            .filter(|change| change.kind == Kind::Written)
            .filter(|change| declaration(&change.op.to_string()) == target)
            .all(|change| matches!(&change.op, Op::Retype { .. }));
        let provided = sites.iter().any(|line| match line {
            Line::Site { target: name, .. } => *name == target,
            Line::Op(_) => false,
        }) || (only_retypes
            && !field_retypes.is_empty()
            && field_retypes.iter().all(|field| {
                sites.iter().any(|line| match line {
                    Line::Site { target, .. } => target == field,
                    Line::Op(_) => false,
                })
            }));
        if !provided {
            sites.push(Line::Site { target, refusals: Vec::new(), down: false });
        }
    }
    // A declaration that is a site gives its added fields too, and has no field site (11.3).
    for target in valueless {
        let decl = target.split('.').next().unwrap_or_default();
        let site = |name: &str| sites.iter().any(|line| matches!(line, Line::Site { target, .. } if target == name));
        if !site(&target) && !site(decl) {
            sites.push(Line::Site { target, refusals: Vec::new(), down: false });
        }
    }
    let mut lines = Vec::new();
    for (index, operation) in operations.iter().enumerate() {
        lines.push(Line::Op(operation.clone()));
        let decl = declaration(operation);
        if !operations.iter().skip(index.checked_add(1).expect("bounded lines")).any(|line| declaration(line) == decl) {
            let mut remaining = Vec::new();
            for site in sites {
                let Line::Site { target, .. } = &site else { unreachable!("only sites collected") };
                if target.split('.').next() == Some(decl) {
                    lines.push(site);
                } else {
                    remaining.push(site);
                }
            }
            sites = remaining;
        }
    }
    lines.extend(sites);
    if let Some(old) = old {
        keep_down(old, &schema, &mut lines);
    }
    Ok(Block { family: diff.family.clone(), from: diff.kept, to: diff.current, lines })
}

// Reinsert author-owned down lines at their surviving anchors, preserving text
// order (and comments when the resulting patch is unchanged).
fn keep_down(old: &Block, schema: &Schema, lines: &mut Vec<Line>) {
    for (index, line) in old.lines.iter().enumerate() {
        let keep = match line {
            Line::Site { target, down, .. } => *down && target_exists(schema, target),
            Line::Op(line) => {
                line.starts_with("breaking ")
                    || line.strip_prefix("down ").is_some_and(|rest| {
                        let target = rest.split_whitespace().next().unwrap_or_default();
                        lines.contains(&Line::Op(format!("drop {target}")))
                    })
            }
        };
        if keep {
            let at = old
                .lines
                .iter()
                .take(index)
                .rev()
                .find_map(|anchor| lines.iter().position(|line| line == anchor))
                .map_or(0, |at| at.checked_add(1).expect("bounded patch lines"));
            lines.insert(at, line.clone());
        }
    }
}

pub(super) fn render(patch: &PatchText) -> String {
    let mut output = format!("patch 1\nowner {} {}", patch.owner, patch.level);
    if !patch.after.is_empty() {
        output.push_str(" after");
    }
    for (owner, level) in &patch.after {
        write!(output, " {owner} {level}").expect("String accepts formatting");
    }
    output.push('\n');
    for mark in &patch.marks {
        writeln!(output, "mark {mark}").expect("String accepts formatting");
    }
    for block in &patch.blocks {
        writeln!(output, "\nfamily {} {} to {}", block.family, block.from, block.to)
            .expect("String accepts formatting");
        for line in &block.lines {
            match line {
                Line::Op(line) => writeln!(output, "{line}").expect("String accepts formatting"),
                Line::Site { target, refusals, down } => {
                    let prefix = if *down { "down " } else { "" };
                    writeln!(output, "{prefix}written {target}").expect("String accepts formatting");
                    for refusal in refusals {
                        writeln!(output, "refuses {} {}", refusal.name, refusal.text)
                            .expect("String accepts formatting");
                    }
                }
            }
        }
    }
    output
}
