use alloc::boxed::Box;
use skein_ui::{Owner, Region, Style};

use crate::{
    Arrange, Border, Cell, Frame, Glyphs, Mark, Paint, Place, Position, RegionLook, Rgb, Size, StyleLook, Theme, width,
};

const RED: Rgb = Rgb { red: 0xab, green: 0xcd, blue: 0xef };
const GREEN: Rgb = Rgb { red: 0x01, green: 0x23, blue: 0x45 };
const INK: Paint = Paint { fore: Some(RED), bold: true, italic: true, ..Paint::NONE };
const PAPER: Paint = Paint { back: Some(GREEN), dim: true, underline: true, reverse: true, ..Paint::NONE };
const BASE: Paint = Paint { back: Some(Rgb { red: 0, green: 0, blue: 0 }), ..Paint::NONE };
const BORDER: Border =
    Border { corners: ['╭', '╮', '╰', '╯'], across: '─', down: '│', paint: INK, title: true, sides: true };
const REGION: RegionLook = RegionLook {
    region: Region { owner: Owner { number: 1 }, number: 1 },
    place: Place::Body,
    arrange: Arrange::Lines,
    border: Some(BORDER),
    pad: 1,
    paint: PAPER,
};
const STYLE: StyleLook = StyleLook {
    style: Style { owner: Owner { number: 1 }, token: 1 },
    paint: INK,
    align: crate::Align::Start,
    band: false,
    label: true,
    mark: None,
    described: None,
    active_name: None,
    focused_value: None,
    spaced: false,
};
const GLYPHS: Glyphs = Glyphs { focus: '>', mask: '●' };

#[test]
fn paint_over_keeps_colours_and_adds_every_attribute() {
    for (name, over, under, expected) in [
        ("colour over none", INK, Paint::NONE, INK),
        ("none over colour", Paint::NONE, PAPER, PAPER),
        (
            "attributes from under",
            INK,
            PAPER,
            Paint {
                fore: Some(RED),
                back: Some(GREEN),
                bold: true,
                dim: true,
                italic: true,
                underline: true,
                reverse: true,
            },
        ),
        (
            "attributes from over",
            PAPER,
            INK,
            Paint {
                fore: Some(RED),
                back: Some(GREEN),
                bold: true,
                dim: true,
                italic: true,
                underline: true,
                reverse: true,
            },
        ),
        (
            "both colours replaced",
            Paint { fore: Some(GREEN), back: Some(RED), ..Paint::NONE },
            Paint { fore: Some(RED), back: Some(GREEN), ..Paint::NONE },
            Paint { fore: Some(GREEN), back: Some(RED), ..Paint::NONE },
        ),
    ] {
        assert_eq!(over.over(under), expected, "{name}");
    }
}

#[test]
#[expect(clippy::disallowed_types, reason = "all invalid glyph rows report their failures together")]
fn theme_valid_refuses_duplicate_looks_and_unsuitable_glyphs() {
    use core::fmt::Write;
    let mut failures = String::new();
    for (name, regions, styles, glyphs, valid) in [
        ("valid", &[REGION][..], &[STYLE][..], GLYPHS, true),
        ("empty looks", &[][..], &[][..], GLYPHS, true),
        ("duplicate region", &[REGION, REGION][..], &[STYLE][..], GLYPHS, false),
        ("duplicate style", &[REGION][..], &[STYLE, STYLE][..], GLYPHS, false),
        (
            "regions owned separately",
            &[REGION, RegionLook { region: Region { owner: Owner { number: 2 }, number: 1 }, ..REGION }][..],
            &[STYLE][..],
            GLYPHS,
            true,
        ),
        (
            "styles owned separately",
            &[REGION][..],
            &[STYLE, StyleLook { style: Style { owner: Owner { number: 2 }, token: 1 }, ..STYLE }][..],
            GLYPHS,
            true,
        ),
        (
            "control mark",
            &[REGION][..],
            &[StyleLook { mark: Some(Mark { glyph: '\n', paint: INK }), ..STYLE }][..],
            GLYPHS,
            false,
        ),
        (
            "wide mark",
            &[REGION][..],
            &[StyleLook { mark: Some(Mark { glyph: '中', paint: INK }), ..STYLE }][..],
            GLYPHS,
            false,
        ),
        (
            "combining mark",
            &[REGION][..],
            &[StyleLook { mark: Some(Mark { glyph: '\u{301}', paint: INK }), ..STYLE }][..],
            GLYPHS,
            false,
        ),
        ("wide focus", &[REGION][..], &[STYLE][..], Glyphs { focus: '中', ..GLYPHS }, false),
        ("control mask", &[REGION][..], &[STYLE][..], Glyphs { mask: '\u{7f}', ..GLYPHS }, false),
        ("combining mask", &[REGION][..], &[STYLE][..], Glyphs { mask: '\u{301}', ..GLYPHS }, false),
        (
            "wide border corner",
            &[RegionLook { border: Some(Border { corners: ['╭', '╮', '╰', '中'], ..BORDER }), ..REGION }][..],
            &[STYLE][..],
            GLYPHS,
            false,
        ),
        (
            "control border across",
            &[RegionLook { border: Some(Border { across: '\n', ..BORDER }), ..REGION }][..],
            &[STYLE][..],
            GLYPHS,
            false,
        ),
        (
            "control corner without sides",
            &[RegionLook {
                border: Some(Border { sides: false, corners: ['\n', '╮', '╰', '╯'], ..BORDER }), ..REGION
            }][..],
            &[STYLE][..],
            GLYPHS,
            false,
        ),
        (
            "zero border down",
            &[RegionLook { border: Some(Border { down: '\u{200b}', ..BORDER }), ..REGION }][..],
            &[STYLE][..],
            GLYPHS,
            false,
        ),
    ] {
        let theme = Theme {
            base: BASE,
            focus: INK,
            placeholder: PAPER,
            active: Paint::NONE,
            regions: Box::from(regions),
            styles: Box::from(styles),
            glyphs,
        };
        if theme.valid() != valid {
            writeln!(failures, "{name}: expected valid={valid}").expect("diagnostic writes");
        }
    }
    assert!(failures.is_empty(), "{failures}");
}

#[test]
fn width_counts_wide_combining_and_other_characters() {
    for (glyph, cells) in [
        ('A', 1),
        ('é', 1),
        ('\u{0301}', 0),
        ('中', 2),
        ('가', 2),
        ('！', 2),
        ('●', 1),
        ('─', 1),
        ('😀', 2),
        ('\u{02ff}', 1),
        ('\u{0300}', 0),
        ('\u{036f}', 0),
        ('\u{0370}', 1),
        ('\u{093e}', 0),
        ('\u{20dd}', 0),
        ('\u{200a}', 1),
        ('\u{200b}', 0),
        ('\u{200f}', 0),
        ('\u{2010}', 1),
        ('\u{205f}', 1),
        ('\u{2060}', 0),
        ('\u{2064}', 0),
        ('\u{2065}', 1),
        ('\u{fdff}', 1),
        ('\u{fe00}', 0),
        ('\u{fe0f}', 0),
        ('\u{fe10}', 2),
        ('\u{fefe}', 1),
        ('\u{feff}', 0),
        ('\u{ff00}', 1),
        ('\u{10ffff}', 1),
        ('\0', 1),
    ] {
        assert_eq!(width(glyph), cells, "U+{:04X}", u32::from(glyph));
    }
}

// A small maximum lets both sides of each frame bound be reached.
const MOST: Size = Size { rows: 2, columns: 3 };
const FILL: Cell = Cell { glyph: 'f', paint: PAPER };

#[test]
fn frame_bounds_cut_clear_and_refuse_outside_the_size() {
    for (name, clear, size, at, row, expected_row, written) in [
        ("new", None, Size { rows: 0, columns: 0 }, Position { row: 0, column: 0 }, 0, None, false),
        ("at both bounds", Some(MOST), MOST, Position { row: 1, column: 2 }, 1, Some(&[FILL; 3][..]), true),
        (
            "cut rows",
            Some(Size { rows: 3, columns: 2 }),
            Size { rows: 2, columns: 2 },
            Position { row: 1, column: 1 },
            1,
            Some(&[FILL; 2][..]),
            true,
        ),
        (
            "cut columns",
            Some(Size { rows: 1, columns: 4 }),
            Size { rows: 1, columns: 3 },
            Position { row: 0, column: 2 },
            0,
            Some(&[FILL; 3][..]),
            true,
        ),
        (
            "past active row",
            Some(Size { rows: 1, columns: 2 }),
            Size { rows: 1, columns: 2 },
            Position { row: 1, column: 0 },
            1,
            None,
            false,
        ),
        (
            "past active column",
            Some(Size { rows: 1, columns: 2 }),
            Size { rows: 1, columns: 2 },
            Position { row: 0, column: 2 },
            0,
            Some(&[FILL; 2][..]),
            false,
        ),
        (
            "no columns",
            Some(Size { rows: 2, columns: 0 }),
            Size { rows: 2, columns: 0 },
            Position { row: 0, column: 0 },
            1,
            Some(&[][..]),
            false,
        ),
        (
            "no rows",
            Some(Size { rows: 0, columns: 3 }),
            Size { rows: 0, columns: 3 },
            Position { row: 0, column: 0 },
            0,
            None,
            false,
        ),
    ] {
        let mut frame = Frame::new(MOST);
        frame.set_scroll(7);
        if let Some(size) = clear {
            frame.set_cursor(Some(Position { row: 0, column: 0 }));
            frame.clear(size, FILL);
        }
        assert_eq!(frame.most(), MOST, "{name}");
        assert_eq!(frame.size(), size, "{name}");
        assert_eq!(frame.row(row), expected_row, "{name}");
        assert_eq!(frame.cursor(), None, "{name}");
        assert_eq!(frame.scroll(), 7, "{name}");
        assert_eq!(frame.put(at, Cell::BLANK), written, "{name}");
        if written {
            let cell = match frame.row(at.row) {
                Some(cells) => cells.get(usize::from(at.column)),
                None => None,
            };
            assert_eq!(cell, Some(&Cell::BLANK), "{name}");
        }
    }
}

fn golden_frame() -> Frame {
    // The acceptance golden is three rows of six cells.
    let size = Size { rows: 3, columns: 6 };
    let mut frame = Frame::new(size);
    frame.clear(size, Cell { paint: BASE, ..Cell::BLANK });
    for (row, column, glyph, paint) in [
        (0, 0, 'A', INK),
        (0, 1, '中', PAPER),
        (0, 2, '\0', PAPER),
        (0, 3, 'Z', INK),
        (2, 1, 'é', PAPER),
        (2, 2, ' ', INK),
        (2, 4, '!', INK),
    ] {
        frame.put(Position { row, column }, Cell { glyph, paint });
    }
    frame.set_cursor(Some(Position { row: 2, column: 5 }));
    frame
}

#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written golden bytes fix the frame's print contract")]
fn frame_print_names_paints_and_writes_its_text_and_cursor() {
    let mut unpainted = Frame::new(Size { rows: 1, columns: 2 });
    unpainted.clear(Size { rows: 1, columns: 2 }, Cell { paint: BASE, ..Cell::BLANK });
    unpainted.put(Position { row: 0, column: 1 }, Cell::BLANK);
    for (name, frame, text, print) in [
        ("golden", golden_frame(), "A中Z\n\n é  !\n".as_bytes(), include_bytes!("tests/golden/print.txt").as_slice()),
        ("empty", Frame::new(MOST), b"".as_slice(), b"--\n--\nno cursor\n".as_slice()),
        ("no attributes", unpainted, b"\n".as_slice(), b"\n--\n.a\n--\na:\nno cursor\n".as_slice()),
    ] {
        assert_eq!(frame.text().as_ref(), text, "{name}");
        assert_eq!(frame.print().as_ref(), print, "{name}");
    }
}

mod fixtures;
mod render;
