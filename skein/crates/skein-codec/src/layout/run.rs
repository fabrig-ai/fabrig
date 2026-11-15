//! Explicit span-reader and conversion stacks (data/codec/layouts.md, 3).
use super::{Inner, Kind, Misconverted, Plan, ReadKind, Source, Variant, Workspace, add};
use crate::schema::Span;
use skein_lib::{Buffer, usize_of};

#[derive(Clone, Copy, Debug)]
pub(super) enum Frame {
    Empty,
    Enter { node: u32, span: Span },
    Record { node: u32, next: u32 },
    List { item: u32, left: u32, rest: Span },
    Pieces { item: u32, rest: Span, length: u32, bound: u32 },
    Length { at: u32, bound: u32 },
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Scan {
    Empty,
    Enter(u32),
    Record { read: u32, next: u32 },
    List { item: u32, left: u32 },
}

fn bytes(old: &[u8], span: Span) -> Result<&[u8], Misconverted> {
    old.get(usize_of(span.start)..usize_of(span.end)).ok_or(Misconverted::Short)
}
fn take(old: &[u8], pos: &mut u32, end: u32, length: u32) -> Result<Span, Misconverted> {
    let start = *pos;
    let next = start.checked_add(length).ok_or(Misconverted::Short)?;
    if next > end {
        return Err(Misconverted::Short);
    }
    bytes(old, Span { start, end: next })?;
    *pos = next;
    Ok(Span { start, end: next })
}
fn byte(old: &[u8], pos: &mut u32, end: u32) -> Result<u8, Misconverted> {
    let span = take(old, pos, end, 1)?;
    Ok(*bytes(old, span)?.first().expect("one byte read"))
}
fn count(old: &[u8], pos: &mut u32, end: u32, bound: u32) -> Result<u32, Misconverted> {
    let span = take(old, pos, end, 4)?;
    let value = u32::from_be_bytes(bytes(old, span)?.try_into().expect("four bytes read"));
    if value > bound {
        return Err(Misconverted::Bound);
    }
    Ok(value)
}
fn put(out: &mut Buffer, value: &[u8]) -> Result<(), Misconverted> {
    match out.put(value) {
        Ok(()) => Ok(()),
        Err(_) => Err(Misconverted::Room),
    }
}
fn push(work: &mut Workspace, depth: &mut u32, frame: Frame) {
    *work.frames.get_mut(*depth).expect("planned conversion depth") = frame;
    *depth = add(*depth, 1);
}
fn push_scan(work: &mut Workspace, depth: &mut u32, frame: Scan) {
    *work.scans.get_mut(*depth).expect("planned reader depth") = frame;
    *depth = add(*depth, 1);
}
fn span_slot(level: u32, fields: u32, offset: u32) -> u32 {
    add(level.checked_mul(fields).expect("fitted span table"), offset)
}
fn check_end(pos: u32, end: u32) -> Result<(), Misconverted> {
    if pos == end { Ok(()) } else { Err(Misconverted::Trailing) }
}
fn length(out: &mut Buffer, at: u32, bound: u32) -> Result<(), Misconverted> {
    let length = out.len().checked_sub(add(at, 4)).expect("reserved length");
    if length > bound {
        return Err(Misconverted::Bound);
    }
    out.set_u32(at, length);
    Ok(())
}

pub(super) fn convert(plan: &Plan, old: &[u8], out: &mut Buffer, work: &mut Workspace) -> Result<(), Misconverted> {
    let Ok(end) = u32::try_from(old.len()) else { return Err(Misconverted::Room) };
    if plan.keeps() {
        let mut pos = 0;
        if plan.old_head {
            take(old, &mut pos, end, 2)?;
        }
        if let Some(tag) = plan.tag {
            let mut tag_pos = pos;
            if byte(old, &mut tag_pos, end)? != tag {
                return Err(Misconverted::Tag);
            }
        }
        if let Some(head) = plan.head {
            put(out, &head.to_be_bytes())?;
        }
        return put(out, bytes(old, Span { start: pos, end })?);
    }
    let mut depth = 0;
    push(work, &mut depth, Frame::Enter { node: plan.root, span: Span { start: 0, end } });
    for _ in 0..turns(end, out.capacity(), plan.nodes.len()) {
        if depth == 0 {
            return Ok(());
        }
        depth = depth.checked_sub(1).expect("nonempty stack");
        let frame = *work.frames.get(depth).expect("active conversion frame");
        match frame {
            Frame::Empty => unreachable!("every active frame is written by this pass"),
            Frame::Enter { node, span } => enter(plan, old, out, work, &mut depth, node, span)?,
            Frame::Record { node, next } => record(plan, out, work, &mut depth, node, next)?,
            Frame::List { item, left, rest } => {
                let pos = rest.start;
                if left == 0 {
                    check_end(pos, rest.end)?;
                    continue;
                }
                let end = scan(plan, old, plan.node(item).read, rest, work)?;
                push(
                    work,
                    &mut depth,
                    Frame::List {
                        item,
                        left: left.checked_sub(1).expect("remaining item"),
                        rest: Span { start: end, end: rest.end },
                    },
                );
                push(work, &mut depth, Frame::Enter { node: item, span: Span { start: pos, end } });
            }
            Frame::Pieces { item, rest, length: outer, bound } => {
                let mut pos = rest.start;
                if pos == rest.end {
                    length(out, outer, bound)?;
                    continue;
                }
                let len = count(old, &mut pos, rest.end, u32::MAX)?;
                let span = take(old, &mut pos, rest.end, len)?;
                push(
                    work,
                    &mut depth,
                    Frame::Pieces { item, rest: Span { start: pos, end: rest.end }, length: outer, bound },
                );
                let at = out.len();
                put(out, &[0; 4])?;
                push(work, &mut depth, Frame::Length { at, bound: u32::MAX });
                push(work, &mut depth, Frame::Enter { node: item, span });
            }
            Frame::Length { at, bound } => length(out, at, bound)?,
        }
    }
    Err(Misconverted::Room)
}

#[expect(clippy::too_many_lines, reason = "explicit conversions (data/codec/layouts.md, 3)")]
fn enter(
    plan: &Plan,
    old: &[u8],
    out: &mut Buffer,
    work: &mut Workspace,
    depth: &mut u32,
    at: u32,
    span: Span,
) -> Result<(), Misconverted> {
    let node = plan.node(at);
    if node.keeps && at != plan.root {
        return put(out, bytes(old, span)?);
    }
    if node.empty {
        return check_end(span.start, span.end);
    }
    match &node.kind {
        Kind::Pending => unreachable!("derive finishes every conversion"),
        Kind::Copy => put(out, bytes(old, span)?),
        Kind::Integer { old: width, new } => {
            if span.end.checked_sub(span.start) != Some(*width) {
                return Err(Misconverted::Short);
            }
            let zeros = new.checked_sub(*width).expect("widened integer");
            put(out, [0; 8].get(..usize_of(zeros)).expect("at most eight padding bytes"))?;
            put(out, bytes(old, span)?)
        }
        Kind::Named(child) => {
            push(work, depth, Frame::Enter { node: *child, span });
            Ok(())
        }
        Kind::Wrap(child) => {
            put(out, &[1])?;
            push(work, depth, Frame::Enter { node: *child, span });
            Ok(())
        }
        Kind::Option(child) => {
            let mut pos = span.start;
            let tag = byte(old, &mut pos, span.end)?;
            if tag > 1 {
                return Err(Misconverted::Tag);
            }
            put(out, &[tag])?;
            if tag == 0 {
                return check_end(pos, span.end);
            }
            push(work, depth, Frame::Enter { node: *child, span: Span { start: pos, end: span.end } });
            Ok(())
        }
        Kind::List { bound, item } => {
            let mut pos = span.start;
            let count = count(old, &mut pos, span.end, *bound)?;
            precheck(plan, plan.node(*item).read, count, span.end.checked_sub(pos).expect("input cursor"))?;
            put(out, &count.to_be_bytes())?;
            if plan.node(*item).empty {
                return check_end(pos, span.end);
            }
            push(work, depth, Frame::List { item: *item, left: count, rest: Span { start: pos, end: span.end } });
            Ok(())
        }
        Kind::Record { old_head, head, .. } => {
            let fields = read_fields(plan, node.read);
            let mut pos = span.start;
            if *old_head {
                take(old, &mut pos, span.end, 2)?;
            }
            for offset in 0..fields.len() {
                let read = *fields.get(offset).expect("old field reader");
                let end = scan(plan, old, read, Span { start: pos, end: span.end }, work)?;
                *work.spans.get_mut(span_slot(*depth, work.fields, offset)).expect("fitted record span table") =
                    Span { start: pos, end };
                pos = end;
            }
            check_end(pos, span.end)?;
            if let Some(head) = head {
                put(out, &head.to_be_bytes())?;
            }
            push(work, depth, Frame::Record { node: at, next: 0 });
            Ok(())
        }
        Kind::Enum(variants) => {
            let mut pos = span.start;
            let tag = byte(old, &mut pos, span.end)?;
            let variant = variants.get(u32::from(tag)).copied().flatten().ok_or(Misconverted::Tag)?;
            let (tag, payload) = match variant {
                Variant::Dropped(index) => return Err(Misconverted::Dropped(index)),
                Variant::Kept { tag, payload, .. } => (tag, payload),
            };
            put(out, &[tag])?;
            if let Some(payload) = payload {
                push(work, depth, Frame::Enter { node: payload, span: Span { start: pos, end: span.end } });
                Ok(())
            } else {
                check_end(pos, span.end)
            }
        }
        Kind::Content { bound, inner } => {
            let mut pos = span.start;
            let len = count(old, &mut pos, span.end, u32::MAX)?;
            let content = take(old, &mut pos, span.end, len)?;
            check_end(pos, span.end)?;
            let at = out.len();
            put(out, &[0; 4])?;
            match inner {
                Inner::Held(child) => {
                    push(work, depth, Frame::Length { at, bound: *bound });
                    push(work, depth, Frame::Enter { node: *child, span: content });
                }
                Inner::Pieces(item) => {
                    if plan.node(*item).keeps {
                        put(out, bytes(old, content)?)?;
                        length(out, at, *bound)?;
                    } else {
                        push(work, depth, Frame::Pieces { item: *item, rest: content, length: at, bound: *bound });
                    }
                }
            }
            Ok(())
        }
    }
}
fn record(
    plan: &Plan,
    out: &mut Buffer,
    work: &mut Workspace,
    depth: &mut u32,
    at: u32,
    next: u32,
) -> Result<(), Misconverted> {
    let fields = match &plan.node(at).kind {
        Kind::Record { fields, .. } => fields,
        Kind::Pending
        | Kind::Copy
        | Kind::Integer { .. }
        | Kind::Named(_)
        | Kind::Wrap(_)
        | Kind::Option(_)
        | Kind::List { .. }
        | Kind::Enum(_)
        | Kind::Content { .. } => unreachable!("record continuation"),
    };
    let Some(field) = fields.get(next) else { return Ok(()) };
    let level = *depth;
    push(work, depth, Frame::Record { node: at, next: add(next, 1) });
    match &field.source {
        Source::Old(offset) => {
            let span =
                *work.spans.get(span_slot(level, work.fields, *offset)).expect("old field was scanned by this record");
            push(work, depth, Frame::Enter { node: field.node.expect("matched field conversion"), span });
            Ok(())
        }
        Source::Default(bytes) => put(out, bytes),
    }
}
fn precheck(plan: &Plan, read: u32, count: u32, remaining: u32) -> Result<(), Misconverted> {
    let minimum = plan.read(read).minimum;
    if minimum > 0 && count > remaining.checked_div(minimum).expect("positive minimum") {
        return Err(Misconverted::Short);
    }
    Ok(())
}

fn scan(plan: &Plan, old: &[u8], at: u32, span: Span, work: &mut Workspace) -> Result<u32, Misconverted> {
    let mut pos = span.start;
    let mut depth = 0;
    push_scan(work, &mut depth, Scan::Enter(at));
    for _ in 0..turns(span.end.checked_sub(span.start).expect("input span"), 0, plan.reads.len()) {
        if depth == 0 {
            return Ok(pos);
        }
        depth = depth.checked_sub(1).expect("active reader frame");
        let frame = *work.scans.get(depth).expect("active reader frame");
        match frame {
            Scan::Empty => unreachable!("each active reader frame was written by this scan"),
            Scan::Enter(read) => {
                if plan.read(read).minimum == 0 {
                    continue;
                }
                match &plan.read(read).kind {
                    ReadKind::Pending => unreachable!("derive finished every reader"),
                    ReadKind::Named(child) => push_scan(work, &mut depth, Scan::Enter(*child)),
                    ReadKind::Fixed(length) => {
                        take(old, &mut pos, span.end, *length)?;
                    }
                    ReadKind::Bool => {
                        if byte(old, &mut pos, span.end)? > 1 {
                            return Err(Misconverted::Bool);
                        }
                    }
                    ReadKind::Content(bound) => {
                        let length = count(old, &mut pos, span.end, *bound)?;
                        take(old, &mut pos, span.end, length)?;
                    }
                    ReadKind::Option(child) => {
                        let tag = byte(old, &mut pos, span.end)?;
                        if tag > 1 {
                            return Err(Misconverted::Tag);
                        }
                        if tag == 1 {
                            push_scan(work, &mut depth, Scan::Enter(*child));
                        }
                    }
                    ReadKind::List { bound, item } => {
                        let count = count(old, &mut pos, span.end, *bound)?;
                        precheck(plan, *item, count, span.end.checked_sub(pos).expect("input cursor"))?;
                        if plan.read(*item).minimum != 0 {
                            push_scan(work, &mut depth, Scan::List { item: *item, left: count });
                        }
                    }
                    ReadKind::Record { head, .. } => {
                        if *head {
                            take(old, &mut pos, span.end, 2)?;
                        }
                        push_scan(work, &mut depth, Scan::Record { read, next: 0 });
                    }
                    ReadKind::Enum { variants, only } => {
                        let tag = byte(old, &mut pos, span.end)?;
                        if let Some(only) = only
                            && tag != *only
                        {
                            return Err(Misconverted::Tag);
                        }
                        let child = variants.get(u32::from(tag)).ok_or(Misconverted::Tag)?;
                        if let Some(child) = child {
                            push_scan(work, &mut depth, Scan::Enter(*child));
                        }
                    }
                }
            }
            Scan::Record { read, next } => {
                let fields = read_fields(plan, read);
                if let Some(&child) = fields.get(next) {
                    push_scan(work, &mut depth, Scan::Record { read, next: add(next, 1) });
                    push_scan(work, &mut depth, Scan::Enter(child));
                }
            }
            Scan::List { item, left } => {
                if left > 0 {
                    push_scan(
                        work,
                        &mut depth,
                        Scan::List { item, left: left.checked_sub(1).expect("remaining item") },
                    );
                    push_scan(work, &mut depth, Scan::Enter(item));
                }
            }
        }
    }
    Err(Misconverted::Room)
}

fn read_fields(plan: &Plan, at: u32) -> &skein_lib::List<u32> {
    match &plan.read(at).kind {
        ReadKind::Record { fields, .. } => fields,
        ReadKind::Pending
        | ReadKind::Named(_)
        | ReadKind::Fixed(_)
        | ReadKind::Bool
        | ReadKind::Content(_)
        | ReadKind::Option(_)
        | ReadKind::List { .. }
        | ReadKind::Enum { .. } => unreachable!("record span reader"),
    }
}
fn turns(input: u32, output: u32, nodes: u32) -> u64 {
    // Zero-byte trees are skipped. Every other traversal consumes input or room;
    // between two consumed bytes there are at most the reach's nodes and depth.
    let bytes = u64::from(input)
        .checked_add(u64::from(output))
        .expect("two u32 lengths")
        .checked_add(1)
        .expect("two u32 lengths plus one");
    let nodes = u64::from(nodes).checked_add(1).expect("u32 node count plus one");
    bytes.saturating_mul(nodes).saturating_mul(2)
}
