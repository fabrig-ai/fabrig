//! A streaming row buffer: storage follows columns and nodes, never text bytes
//! (skein/docs/design/ui-screen.md, 5.1 to 5.4).

use skein_lib::{List, usize_of};
use skein_ui::{EntryKind, Node, Role, Tree};

use super::{Info, words};
use crate::{Cell, Focus, Frame, Paint, Position, RegionLook, Theme, width};

#[derive(Clone, Copy, Debug)]
pub(super) struct Viewport {
    pub first: u32,
    pub origin: u16,
    pub height: u16,
    pub draw: bool,
}
impl Viewport {
    pub fn measure() -> Self {
        Self { first: 0, origin: 0, height: 0, draw: false }
    }
}

#[derive(Clone, Copy, Debug)]
struct Ink {
    cell: Cell,
    focused: bool,
    caret: bool,
}
impl Ink {
    fn blank(paint: Paint) -> Self {
        Self { cell: Cell { glyph: ' ', paint }, focused: false, caret: false }
    }
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    look: RegionLook,
    paint: Paint,
    left: u16,
    right: u16,
}

#[derive(Clone, Copy, Debug)]
struct PendingItem {
    at: u32,
    indent: u16,
    annotated: bool,
    description: bool,
}

#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "row occupancy, first-line capture, field clipping and focus tagging are independent"
)]
pub(super) struct Canvas {
    columns: u16,
    base: Paint,
    viewport: Viewport,
    cells: List<Ink>,
    first: List<Ink>,
    first_len: u16,
    row: u32,
    column: u16,
    indent: u16,
    continuation: u16,
    left: u16,
    right: u16,
    dirty: bool,
    focused: Option<(u32, u32)>,
    edges: List<Edge>,
    items: List<PendingItem>,
    single: bool,
    clipped: bool,
    tagged: bool,
    code: bool,
    quote: bool,
    cut: bool,
}

impl Canvas {
    pub fn new(columns: u16, base: Paint, viewport: Viewport, nodes: u32) -> Self {
        let mut cells = List::with_capacity(u32::from(columns));
        let mut first = List::with_capacity(u32::from(columns));
        for _column in 0..columns {
            cells.push(Ink::blank(base)).expect("one cell per column");
            first.push(Ink::blank(base)).expect("one cell per column");
        }
        Self {
            columns,
            base,
            viewport,
            cells,
            first,
            first_len: 0,
            row: 0,
            column: 0,
            indent: 0,
            continuation: 0,
            left: 0,
            right: columns,
            dirty: false,
            focused: None,
            edges: List::with_capacity(nodes),
            items: List::with_capacity(nodes),
            single: false,
            clipped: false,
            tagged: false,
            code: false,
            quote: false,
            cut: false,
        }
    }

    pub fn first_line(columns: u16, base: Paint, nodes: u32) -> Self {
        let mut out = Self::new(columns, base, Viewport::measure(), nodes);
        out.single = true;
        out
    }
    pub fn columns(&self) -> u16 {
        self.columns
    }
    pub fn room(&self) -> u16 {
        self.right.saturating_sub(self.left)
    }
    pub fn rows(&self) -> u32 {
        self.row
    }
    pub fn focus_rows(&self) -> Option<(u32, u32)> {
        self.focused
    }
    pub fn indent(&self) -> u16 {
        self.indent
    }
    pub fn first_length(&self) -> u16 {
        self.first_len
    }
    pub fn start(&mut self, indent: u16) {
        self.indent = indent;
        self.continuation = indent;
        if !self.dirty {
            self.column = self.left.saturating_add(indent).min(self.right);
        }
    }
    pub fn tag_focus(&mut self, focused: bool) {
        self.tagged = focused;
    }
    pub fn clip_line(&mut self, clipped: bool) {
        self.clipped = clipped;
    }
    pub fn block(&mut self, code: bool, quote: bool) {
        self.code = code;
        self.quote = quote;
    }
    pub fn line(&mut self) {
        self.dirty = self.dirty || self.room() > 0;
    }
    pub fn within_item(&self) -> bool {
        !self.items.is_empty()
    }
    pub fn description_indent(&self) -> u16 {
        match self.items.last() {
            Some(item) => item.indent,
            None => self.indent.saturating_add(2),
        }
    }
    pub fn continuation(&mut self, indent: u16) {
        self.continuation = indent;
    }
    pub fn item_annotation(&mut self, at: u32) {
        self.items
            .push(PendingItem { at, indent: self.continuation, annotated: false, description: true })
            .expect("pending items fit nodes");
    }

    pub fn row_annotation(&mut self, at: u32) {
        self.items
            .push(PendingItem { at, indent: self.continuation, annotated: false, description: false })
            .expect("pending rows fit nodes");
    }

    pub fn control(&mut self, focused: bool, mark: char, paint: Paint, frame: &mut Frame) {
        if self.single {
            return;
        }
        self.glyph(if focused { mark } else { ' ' }, paint, focused, frame);
        self.glyph(' ', paint, focused, frame);
        self.continuation = self.indent.saturating_add(2);
    }

    pub fn safe(&mut self, bytes: &[u8], paint: Paint, focused: bool, frame: &mut Frame) {
        let mut next = 0;
        for at in 0..bytes.len() {
            if at < next {
                continue;
            }
            let (glyph, length) = decode(bytes.get(at..).expect("input offset exists"));
            next = at.saturating_add(length);
            self.glyph(glyph, paint, focused, frame);
        }
    }

    pub fn content(
        &mut self,
        bytes: &[u8],
        paint: Paint,
        focused: bool,
        tree: &Tree,
        info: &List<Info>,
        frame: &mut Frame,
    ) {
        let mut begin = 0;
        for (at, byte) in bytes.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            self.safe(bytes.get(begin..at).expect("content segment exists"), paint, focused, frame);
            self.annotate_items(tree, info, frame);
            self.safe(b"\n", paint, focused, frame);
            if self.quote {
                self.safe(b"> ", paint, focused, frame);
            }
            begin = at.saturating_add(1);
        }
        self.safe(bytes.get(begin..).expect("content suffix exists"), paint, focused, frame);
    }

    fn glyph(&mut self, glyph: char, paint: Paint, focused: bool, frame: &mut Frame) {
        let focused = focused || self.tagged;
        if self.single && self.row != 0 {
            return;
        }
        if glyph == '\n' && self.clipped {
            return;
        }
        if glyph == '\n' {
            self.blank_line(frame);
            self.dirty = true;
            return;
        }
        if self.code && self.cut {
            return;
        }
        let pending_caret = match self.cells.get(u32::from(self.column)) {
            Some(ink) => ink.caret,
            None => false,
        };
        let count = u16::from(width(glyph));
        if count == 0 || self.room() == 0 {
            return;
        }
        let start = self.left.saturating_add(self.continuation).min(self.right.saturating_sub(1));
        if count > self.right.saturating_sub(start) {
            self.cut = self.code;
            return;
        }
        if self.column.saturating_add(count) > self.right {
            if self.single || self.clipped || self.code {
                self.cut = self.code;
                return;
            }
            if let Some(ink) = self.cells.get_mut(u32::from(self.column)) {
                ink.caret = false;
            }
            if glyph == ' ' {
                self.commit(frame);
                if pending_caret {
                    self.caret(paint, frame);
                }
                return;
            }
            self.wrap(frame);
        }
        if self.column.saturating_add(count) > self.right {
            return;
        }
        let caret = pending_caret
            || match self.cells.get(u32::from(self.column)) {
                Some(ink) => ink.caret,
                None => false,
            };
        self.set(self.column, Ink { cell: Cell { glyph, paint }, focused, caret });
        self.column = self.column.saturating_add(1);
        if count == 2 {
            self.set(self.column, Ink { cell: Cell { glyph: '\0', paint }, focused, caret: false });
            self.column = self.column.saturating_add(1);
        }
        self.dirty = true;
    }

    fn wrap(&mut self, frame: &mut Frame) {
        let begin = self.left.saturating_add(self.continuation).min(self.right.saturating_sub(1));
        let mut space = None;
        for column in begin..self.column {
            if self.cells.get(u32::from(column)).expect("column exists").cell.glyph == ' ' {
                space = Some(column);
            }
        }
        if let Some(space) = space {
            let old_end = self.column;
            // A bounded row holds the suffix while the preceding row is committed.
            let mut tail = List::with_capacity(u32::from(self.columns));
            for column in space.saturating_add(1)..old_end {
                tail.push(*self.cells.get(u32::from(column)).expect("suffix exists")).expect("suffix fits row");
            }
            for column in space..old_end {
                self.set(column, Ink::blank(self.background()));
            }
            self.column = space;
            self.commit(frame);
            self.column = begin;
            for ink in &tail {
                self.set(self.column, *ink);
                self.column = self.column.saturating_add(1);
            }
            self.dirty = !tail.is_empty();
        } else {
            self.commit(frame);
            self.column = begin;
        }
    }

    fn set(&mut self, column: u16, ink: Ink) {
        if let Some(cell) = self.cells.get_mut(u32::from(column)) {
            *cell = ink;
        }
    }
    fn background(&self) -> Paint {
        match self.edges.last() {
            Some(edge) => edge.paint,
            None => self.base,
        }
    }
    fn reset(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Ink::blank(self.base);
        }
        for edge in &self.edges {
            for column in edge.left..edge.right {
                if let Some(cell) = self.cells.get_mut(u32::from(column)) {
                    *cell = Ink::blank(edge.paint);
                }
            }
            if let Some(border) = edge.look.border {
                for column in [edge.left, edge.right.saturating_sub(1)] {
                    if column < edge.right
                        && let Some(cell) = self.cells.get_mut(u32::from(column))
                    {
                        *cell = Ink {
                            cell: Cell { glyph: border.down, paint: border.paint.over(edge.paint) },
                            focused: false,
                            caret: false,
                        };
                    }
                }
            }
        }
        self.column = self.left.saturating_add(self.continuation).min(self.right);
        self.dirty = false;
        self.cut = false;
    }
    fn commit(&mut self, frame: &mut Frame) {
        let visible = self.row >= self.viewport.first
            && self.row.saturating_sub(self.viewport.first) < u32::from(self.viewport.height);
        let screen_row = self
            .viewport
            .origin
            .saturating_add(u16::try_from(self.row.saturating_sub(self.viewport.first)).unwrap_or(u16::MAX));
        if self.row == 0 {
            self.first_len = self.column.saturating_sub(self.left);
            for (saved, cell) in self.first.iter_mut().zip(self.cells.iter()) {
                *saved = *cell;
            }
        }
        for (column, ink) in (0..self.columns).zip(self.cells.iter()) {
            if ink.focused {
                self.focused = Some(match self.focused {
                    Some((first, end)) => (first, end.max(self.row.saturating_add(1))),
                    None => (self.row, self.row.saturating_add(1)),
                });
            }
            if visible && self.viewport.draw {
                frame.put(Position { row: screen_row, column }, ink.cell);
                if ink.caret {
                    frame.set_cursor(Some(Position { row: screen_row, column }));
                }
            }
        }
        self.row = self.row.saturating_add(1);
        self.reset();
    }
    pub fn finish(&mut self, frame: &mut Frame) {
        if self.dirty {
            self.commit(frame);
        }
    }
    pub fn blank_line(&mut self, frame: &mut Frame) {
        self.commit(frame);
    }

    fn annotate_items(&mut self, tree: &Tree, info: &List<Info>, frame: &mut Frame) {
        let mut items = core::mem::replace(&mut self.items, List::with_capacity(tree.len()));
        for item in items.iter_mut() {
            if !item.annotated {
                let n = tree.get(item.at).expect("item exists");
                let paint = info.get(item.at).expect("item information exists").paint;
                words::states(n, paint, self, frame);
                words::position(tree, item.at, info, paint, self, frame);
                item.annotated = true;
            }
        }
        self.items = items;
    }

    pub fn finish_annotated(&mut self, tree: &Tree, info: &List<Info>, frame: &mut Frame) {
        self.annotate_items(tree, info, frame);
        let mut items = core::mem::replace(&mut self.items, List::with_capacity(tree.len()));
        self.finish(frame);
        let indent = self.indent;
        for item in &items {
            let n = tree.get(item.at).expect("item exists");
            if item.description
                && let Some(description) = &n.description
            {
                self.start(item.indent);
                self.safe(description, info.get(item.at).expect("item information exists").paint, false, frame);
                self.finish(frame);
            }
        }
        self.start(indent);
        items.clear();
        self.items = items;
    }

    pub fn enter_region(&mut self, look: RegionLook, paint: Paint, frame: &mut Frame) {
        self.finish(frame);
        let edge = Edge { look, paint, left: self.left, right: self.right };
        self.edges.push(edge).expect("open regions fit nodes");
        self.reset();
        if look.border.is_some() {
            self.border(look, true, frame);
        }
        for _row in 0..look.pad {
            self.blank_line(frame);
        }
        let inset = look.pad.saturating_add(u16::from(look.border.is_some()));
        self.left = self.left.saturating_add(inset).min(self.right);
        self.right = self.right.saturating_sub(inset).max(self.left);
        self.column = self.left;
        // If border and padding exhaust the width, only the clipped border is drawn.
    }
    pub fn leave_region(&mut self, look: RegionLook, frame: &mut Frame) {
        self.finish(frame);
        for _row in 0..look.pad {
            self.blank_line(frame);
        }
        if look.border.is_some() {
            self.border(look, false, frame);
        }
        let edge = *self.edges.last().expect("region is open");
        self.edges.truncate(self.edges.len().saturating_sub(1));
        self.left = edge.left;
        self.right = edge.right;
        self.reset();
    }
    fn border(&mut self, look: RegionLook, top: bool, frame: &mut Frame) {
        let Some(border) = look.border else { return };
        let edge = *self.edges.last().expect("region is open");
        let paint = border.paint.over(edge.paint);
        for column in edge.left..edge.right {
            self.set(column, Ink { cell: Cell { glyph: border.across, paint }, focused: false, caret: false });
        }
        let [tl, tr, bl, br] = border.corners;
        if edge.left < edge.right {
            self.set(
                edge.left,
                Ink { cell: Cell { glyph: if top { tl } else { bl }, paint }, focused: false, caret: false },
            );
            if edge.right.saturating_sub(edge.left) > 1 {
                self.set(
                    edge.right.saturating_sub(1),
                    Ink { cell: Cell { glyph: if top { tr } else { br }, paint }, focused: false, caret: false },
                );
            }
        }
        self.column = edge.right;
        self.blank_line(frame);
    }
    pub fn title(&mut self, title: &Canvas, look: RegionLook, frame: &mut Frame) {
        // The top border precedes the padding already measured on entry.
        let title_row = self.row.saturating_sub(u32::from(look.pad)).saturating_sub(1);
        let Some(border) = look.border else { return };
        let edge = *self.edges.last().expect("region is open");
        let limit = edge.right.saturating_sub(1);
        let mut column = edge.left.saturating_add(2);
        let paint = border.paint.over(edge.paint);
        if title_row < self.viewport.first
            || title_row.saturating_sub(self.viewport.first) >= u32::from(self.viewport.height)
            || !self.viewport.draw
        {
            return;
        }
        let row = self.viewport.origin.saturating_add(
            u16::try_from(title_row.saturating_sub(self.viewport.first)).expect("visible row fits u16"),
        );
        if column < limit {
            frame.put(Position { row, column }, Cell { glyph: ' ', paint });
            column = column.saturating_add(1);
        }
        for ink in title.first.iter().take(usize::from(title.first_len)) {
            if column >= limit {
                break;
            }
            if width(ink.cell.glyph) == 2 && column.saturating_add(1) >= limit {
                break;
            }
            frame.put(Position { row, column }, Cell { glyph: ink.cell.glyph, paint });
            column = column.saturating_add(1);
        }
        if column < limit {
            frame.put(Position { row, column }, Cell { glyph: ' ', paint });
        }
    }
    pub fn spread_line(&mut self, line: &Canvas, offset: u16, length: u16) {
        let end = length.min(line.first_len);
        for (column, ink) in (0..end).zip(line.first.iter()) {
            let at = self.left.saturating_add(offset).saturating_add(column);
            if at >= self.right {
                break;
            }
            if width(ink.cell.glyph) == 2 && column.saturating_add(1) >= end {
                break;
            }
            self.set(at, *ink);
        }
        self.column = self.right;
        self.dirty = true;
    }

    pub fn field(&mut self, node: &Node, focus: &Focus, theme: &Theme, paint: Paint, frame: &mut Frame) {
        let focused = focus.node == Some(node.id);
        let value = match &node.value {
            Some(value) => value.text.as_ref(),
            None => b"",
        };
        let masked = node.entry == Some(EntryKind::Secret);
        let room = self.right.saturating_sub(self.column);
        if room == 0 {
            return;
        }
        let caret = usize_of(focus.caret).min(value.len());
        let mut before = 0_u32;
        let mut next = 0;
        for at in 0..caret {
            if at < next {
                continue;
            }
            let (glyph, length) = decode(value.get(at..).expect("value offset exists"));
            next = at.saturating_add(length);
            before = before.saturating_add(if masked { 1 } else { u32::from(width(glyph)) });
        }
        let offset = if node.role == Role::TextField && focused {
            before.saturating_sub(u32::from(room).saturating_sub(1))
        } else {
            0
        };
        let start = self.column;
        let mut cells = 0_u32;
        next = 0;
        if value.is_empty() {
            if focused {
                self.caret(paint, frame);
            }
            if let Some(placeholder) = &node.placeholder {
                self.field_bytes(placeholder, theme.placeholder.over(paint), focused, start, frame);
            }
            self.dirty = true;
            return;
        }
        for at in 0..value.len() {
            if at < next {
                continue;
            }
            let (mut glyph, length) = decode(value.get(at..).expect("value offset exists"));
            next = at.saturating_add(length);
            if masked {
                glyph = theme.glyphs.mask;
            }
            let count = u32::from(width(glyph));
            if cells < offset {
                cells = cells.saturating_add(count);
                continue;
            }
            if focused && at == caret {
                self.caret(paint, frame);
            }
            if node.role == Role::TextField {
                if glyph == '\n' {
                    glyph = ' ';
                }
                if self.column.saturating_add(u16::from(width(glyph))) > self.right {
                    break;
                }
            }
            self.glyph(glyph, paint, focused, frame);
        }
        if focused && caret == value.len() {
            self.caret(paint, frame);
        }
    }
    fn field_bytes(&mut self, bytes: &[u8], paint: Paint, focused: bool, start: u16, frame: &mut Frame) {
        let mut next = 0;
        for at in 0..bytes.len() {
            if at < next {
                continue;
            }
            let (glyph, length) = decode(bytes.get(at..).expect("placeholder offset exists"));
            next = at.saturating_add(length);
            if self.column.saturating_add(u16::from(width(glyph))) > self.right {
                break;
            }
            self.glyph(glyph, paint, focused, frame);
        }
        if focused && let Some(ink) = self.cells.get_mut(u32::from(start)) {
            ink.caret = true;
        }
    }
    fn caret(&mut self, paint: Paint, frame: &mut Frame) {
        if self.column >= self.right {
            if self.clipped {
                return;
            }
            self.wrap(frame);
        }
        if let Some(ink) = self.cells.get_mut(u32::from(self.column)) {
            ink.caret = true;
            ink.focused = true;
            ink.cell.paint = paint;
        }
        self.dirty = true;
    }
}

/// One malformed byte becomes one replacement; no decoded text is collected.
fn decode(bytes: &[u8]) -> (char, usize) {
    let Some(lead) = bytes.first().copied() else { return ('\u{fffd}', 0) };
    if lead < 128 {
        return (
            match lead {
                9 => ' ',
                10 => '\n',
                0..=31 | 127 => '\u{fffd}',
                _ => char::from(lead),
            },
            1,
        );
    }
    let (length, mut value, minimum) = match lead {
        194..=223 => (2, u32::from(lead & 0x1f), 0x80),
        224..=239 => (3, u32::from(lead & 0x0f), 0x800),
        240..=244 => (4, u32::from(lead & 0x07), 0x10000),
        _ => return ('\u{fffd}', 1),
    };
    for index in 1..length {
        let Some(byte) = bytes.get(index).copied() else { return ('\u{fffd}', 1) };
        if !(128..=191).contains(&byte) {
            return ('\u{fffd}', 1);
        }
        value = (value << 6_u32) | u32::from(byte & 0x3f);
    }
    if value < minimum {
        return ('\u{fffd}', 1);
    }
    match char::from_u32(value) {
        Some(glyph) => (if glyph.is_control() { '\u{fffd}' } else { glyph }, length),
        None => ('\u{fffd}', 1),
    }
}
