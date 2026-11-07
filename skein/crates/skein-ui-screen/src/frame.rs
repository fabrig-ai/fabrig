//! The bounded grid and its golden print (skein/docs/design/ui-screen.md, section 4).

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, usize_of};

use crate::{Paint, Rgb};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Size {
    pub rows: u16,
    pub columns: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Position {
    pub row: u16,
    pub column: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cell {
    /// U+0000 follows a wide glyph and is drawn as nothing.
    pub glyph: char,
    pub paint: Paint,
}

impl Cell {
    pub const BLANK: Cell = Cell { glyph: ' ', paint: Paint::NONE };
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frame {
    most: Size,
    size: Size,
    cells: List<Cell>,
    cursor: Option<Position>,
    scroll: u32,
    base: Paint,
}

impl Frame {
    #[must_use]
    pub fn new(most: Size) -> Frame {
        let count = u32::from(most.rows).checked_mul(u32::from(most.columns)).expect("two u16 sides fit a u32");
        let mut cells = List::with_capacity(count);
        for _cell in 0..count {
            cells.push(Cell::BLANK).expect("the most cells fit their capacity");
        }
        Frame { most, size: Size { rows: 0, columns: 0 }, cells, cursor: None, scroll: 0, base: Paint::NONE }
    }

    #[must_use]
    pub const fn most(&self) -> Size {
        self.most
    }

    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Clears to `size` (at most `most`, each side), every cell `fill`; the cursor none.
    pub fn clear(&mut self, size: Size, fill: Cell) {
        self.size = Size { rows: size.rows.min(self.most.rows), columns: size.columns.min(self.most.columns) };
        for cell in self.cells.iter_mut() {
            *cell = fill;
        }
        self.cursor = None;
        self.base = fill.paint;
    }

    #[must_use]
    pub fn row(&self, row: u16) -> Option<&[Cell]> {
        if row >= self.size.rows {
            return None;
        }
        let start = self.offset(Position { row, column: 0 });
        let end = start.checked_add(u32::from(self.size.columns)).expect("the row fits the most cells");
        self.cells.as_slice().get(usize_of(start)..usize_of(end))
    }

    /// Writes one cell; false past the size.
    pub fn put(&mut self, at: Position, cell: Cell) -> bool {
        if at.row >= self.size.rows || at.column >= self.size.columns {
            return false;
        }
        let offset = self.offset(at);
        *self.cells.get_mut(offset).expect("a position within size fits the most cells") = cell;
        true
    }

    #[must_use]
    pub const fn cursor(&self) -> Option<Position> {
        self.cursor
    }

    pub fn set_cursor(&mut self, at: Option<Position>) {
        self.cursor = at;
    }

    #[must_use]
    pub const fn scroll(&self) -> u32 {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: u32) {
        self.scroll = scroll;
    }

    /// UTF-8 rows, omitting wide continuations and trailing spaces.
    #[must_use]
    pub fn text(&self) -> Box<[u8]> {
        let mut measured = Bytes::measure();
        self.text_into(&mut measured);
        let mut out = Bytes::write(measured.len);
        self.text_into(&mut out);
        out.finish()
    }

    /// Text, cell paints, a paint legend and the cursor, as section 4 specifies.
    #[must_use]
    pub fn print(&self) -> Box<[u8]> {
        let mut measured = Bytes::measure();
        self.print_into(&mut measured);
        let mut out = Bytes::write(measured.len);
        self.print_into(&mut out);
        out.finish()
    }

    fn offset(&self, at: Position) -> u32 {
        u32::from(at.row)
            .checked_mul(u32::from(self.most.columns))
            .expect("two u16 sides fit a u32")
            .checked_add(u32::from(at.column))
            .expect("the position fits the most cells")
    }

    fn text_into(&self, out: &mut Bytes) {
        for row in 0..self.size.rows {
            let cells = self.row(row).expect("a row within the size exists");
            let mut end = 0;
            for (at, cell) in cells.iter().enumerate().rev() {
                if cell.glyph != ' ' && cell.glyph != '\0' {
                    end = at.checked_add(1).expect("a cell's end fits its row");
                    break;
                }
            }
            for cell in cells.get(..end).expect("the text ends within its row") {
                if cell.glyph != '\0' {
                    let mut utf8 = [0; 4];
                    let glyph = cell.glyph.encode_utf8(&mut utf8);
                    out.put(glyph.as_bytes());
                }
            }
            out.put(b"\n");
        }
    }

    fn print_into(&self, out: &mut Bytes) {
        self.text_into(out);
        out.put(b"--\n");
        let mut paints = List::with_capacity(52);
        for row in 0..self.size.rows {
            for cell in self.row(row).expect("a row within the size exists") {
                let letter = if cell.paint == self.base { b'.' } else { paint_letter(cell.paint, &mut paints) };
                out.put(&[letter]);
            }
            out.put(b"\n");
        }
        out.put(b"--\n");
        for (paint, letter) in paints.iter().zip((b'a'..=b'z').chain(b'A'..=b'Z')) {
            out.put(&[letter]);
            out.put(b":");
            paint_into(*paint, out);
            out.put(b"\n");
        }
        match self.cursor {
            Some(at) => {
                out.put(b"cursor ");
                let row = Decimal::of(u64::from(at.row));
                out.put(row.as_bytes());
                out.put(b" ");
                let column = Decimal::of(u64::from(at.column));
                out.put(column.as_bytes());
                out.put(b"\n");
            }
            None => out.put(b"no cursor\n"),
        }
    }
}

fn paint_letter(paint: Paint, paints: &mut List<Paint>) -> u8 {
    for (known, letter) in paints.iter().zip((b'a'..=b'z').chain(b'A'..=b'Z')) {
        if paint == *known {
            return letter;
        }
    }
    let letter = (b'a'..=b'z').chain(b'A'..=b'Z').nth(usize_of(paints.len()));
    match letter {
        Some(letter) => {
            paints.push(paint).expect("the alphabet has room for this paint");
            letter
        }
        None => b'?',
    }
}

fn paint_into(paint: Paint, out: &mut Bytes) {
    if let Some(fore) = paint.fore {
        out.put(b" fore #");
        rgb_into(fore, out);
    }
    if let Some(back) = paint.back {
        out.put(b" back #");
        rgb_into(back, out);
    }
    for (set, word) in [
        (paint.bold, b" bold".as_slice()),
        (paint.dim, b" dim"),
        (paint.italic, b" italic"),
        (paint.underline, b" underline"),
        (paint.reverse, b" reverse"),
    ] {
        if set {
            out.put(word);
        }
    }
}

fn rgb_into(rgb: Rgb, out: &mut Bytes) {
    for component in [rgb.red, rgb.green, rgb.blue] {
        for digit in [component >> 4_u8, component & 0x0f] {
            let hex =
                if digit < 10 { b'0'.saturating_add(digit) } else { b'A'.saturating_add(digit.saturating_sub(10)) };
            out.put(&[hex]);
        }
    }
}

/// The same writes measure first, then fill a writer allocated at that length.
struct Bytes {
    len: usize,
    writer: Option<Writer>,
}

impl Bytes {
    fn measure() -> Bytes {
        Bytes { len: 0, writer: None }
    }

    fn write(len: usize) -> Bytes {
        Bytes { len: 0, writer: Some(Writer::new(len)) }
    }

    fn put(&mut self, bytes: &[u8]) {
        self.len = self.len.checked_add(bytes.len()).expect("the frame's print fits addressable memory");
        if let Some(writer) = &mut self.writer {
            writer.put(bytes).expect("the same writes fill the measured length");
        }
    }

    fn finish(self) -> Box<[u8]> {
        self.writer.expect("the writing pass has a writer").finish()
    }
}
