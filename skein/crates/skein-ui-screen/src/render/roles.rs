//! The text shell's remaining role layouts in cells (skein/docs/design/ui-screen.md, 5.3).

use skein_lib::List;
use skein_ui::{Role, Tree};

use super::{Info, canvas::Canvas, content, words};
use crate::{Frame, Theme};

pub(super) fn named(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    focused: bool,
    out: &mut Canvas,
    frame: &mut Frame,
) -> u16 {
    let n = tree.get(at).expect("named node exists");
    let paint = info.get(at).expect("named node information exists").paint;
    let indent = out.indent();
    if let Some(name) = &n.name {
        out.content(name, paint, focused, tree, info, frame);
        words::colon(name, paint, out, frame);
        if n.role == Role::Disclosure
            && let Some(description) = &n.description
        {
            out.continuation(indent.saturating_add(2));
            out.safe(b" ", paint, focused, frame);
            out.content(
                description,
                info.get(at).expect("description information exists").description_paint(),
                focused,
                tree,
                info,
                frame,
            );
        }
        words::states(n, paint, out, frame);
        out.finish_annotated(tree, info, frame);
        indent.saturating_add(2)
    } else {
        indent
    }
}

#[expect(clippy::too_many_arguments, reason = "items borrow the role walk's inputs and ordered number")]
pub(super) fn item(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &Theme,
    focused: bool,
    number: Option<u64>,
    out: &mut Canvas,
    frame: &mut Frame,
) -> u16 {
    let n = tree.get(at).expect("item exists");
    let paint = info.get(at).expect("item information exists").paint;
    let indent = out.indent();
    if let Some(number) = number {
        out.safe(skein_lib::Decimal::of(number).as_bytes(), paint, focused, frame);
        out.safe(b". ", paint, focused, frame);
    } else if let Some(mark) = info.get(at).expect("item information exists").mark {
        let mut bytes = [0; 4];
        out.safe(mark.glyph.encode_utf8(&mut bytes).as_bytes(), mark.paint.over(paint), focused, frame);
        out.safe(b" ", paint, focused, frame);
    } else {
        out.safe(b"- ", paint, focused, frame);
    }
    out.continuation(indent.saturating_add(2));
    out.item_annotation(at);
    if n.size == 1 {
        content(tree, at, info, theme, focused, out, frame);
    } else if n.role == Role::Item {
        let end = item_text_end(tree, at);
        let mut next = at.saturating_add(1);
        for child in at.saturating_add(1)..end {
            if child < next {
                continue;
            }
            let text = tree.get(child).expect("item text exists");
            content(tree, child, info, theme, focused, out, frame);
            words::states(text, info.get(child).expect("text information exists").paint, out, frame);
            next = child.saturating_add(text.size);
        }
        if end > at.saturating_add(1) {
            out.finish_annotated(tree, info, frame);
        }
    }
    indent.saturating_add(2)
}

/// The leading text children share the item's marked line; its first block ends it.
pub(super) fn item_text_end(tree: &Tree, at: u32) -> u32 {
    let item = tree.get(at).expect("item exists");
    let mut next = at.saturating_add(1);
    for child in at.saturating_add(1)..at.saturating_add(item.size) {
        if child < next {
            continue;
        }
        let node = tree.get(child).expect("item child exists");
        if node.role != Role::Text {
            break;
        }
        next = child.saturating_add(node.size);
    }
    next
}

pub(super) fn empty_header_row(tree: &Tree, at: u32) -> bool {
    let row = tree.get(at).expect("row exists");
    let mut header = false;
    for child in at.saturating_add(1)..at.saturating_add(row.size) {
        let cell = tree.get(child).expect("row descendant exists");
        if cell.role.is_row_data() {
            return false;
        }
        if cell.role != Role::ColumnHeader {
            continue;
        }
        header = true;
        if let Some(name) = &cell.name
            && !name.is_empty()
        {
            return false;
        }
        for index in child..child.saturating_add(cell.size) {
            let n = tree.get(index).expect("header descendant exists");
            if !n.text.is_empty() || n.external.is_some() {
                return false;
            }
        }
    }
    header
}

pub(super) fn row(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &Theme,
    focused: bool,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    if empty_header_row(tree, at) {
        return;
    }
    let n = tree.get(at).expect("row exists");
    let paint = info.get(at).expect("row information exists").paint;
    out.line();
    out.row_annotation(at);
    let mut first = true;
    for child in at.saturating_add(1)..at.saturating_add(n.size) {
        let cell = tree.get(child).expect("row descendant exists");
        if cell.role.is_cell() {
            if !first {
                out.safe(b" | ", paint, focused, frame);
            }
            content(tree, child, info, theme, focused, out, frame);
            words::sort(cell, info.get(child).expect("cell information exists").paint, out, frame);
            first = false;
        }
    }
    out.finish_annotated(tree, info, frame);
}

pub(super) fn line(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &Theme,
    focused: bool,
    out: &mut Canvas,
    frame: &mut Frame,
) -> Option<u16> {
    let n = tree.get(at).expect("line exists");
    let paint = info.get(at).expect("line information exists").paint;
    let extra = if n.role == Role::Code { 4 } else { 0 };
    out.start(out.indent().saturating_add(extra));
    out.line();
    if n.role == Role::Progress {
        if let Some(name) = &n.name {
            out.content(name, paint, focused, tree, info, frame);
        }
        out.safe(b": ", paint, focused, frame);
    }
    // Code keeps its first columns, without wrapping a misleading new code line
    // (skein/docs/design/ui-screen.md, 5.3).
    out.block(n.role == Role::Code, n.role == Role::Quote);
    if n.role == Role::Quote {
        out.safe(b"> ", paint, focused, frame);
    }
    if n.role == Role::Image {
        if let Some(name) = &n.name {
            out.content(name, paint, focused, tree, info, frame);
        }
    } else {
        content(tree, at, info, theme, focused, out, frame);
    }
    if n.role == Role::Term {
        out.safe(b":", paint, focused, frame);
    }
    words::sort(n, paint, out, frame);
    words::states(n, paint, out, frame);
    out.finish_annotated(tree, info, frame);
    out.block(false, false);
    None
}
