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
            out.content(description, paint, focused, tree, info, frame);
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
    } else {
        out.safe(b"- ", paint, focused, frame);
    }
    out.continuation(indent.saturating_add(2));
    out.item_annotation(at);
    if n.size == 1 {
        content(tree, at, info, theme, focused, out, frame);
    }
    indent.saturating_add(2)
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
