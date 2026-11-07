use super::fixtures::{self, LIMITS};
use crate::{Cell, Focus, Frame, Paint, Position, Size, render};
use skein_ui::{Builder, EntryKind, Role, Tree};

// The specification's frame sizes; the extra narrow case leaves only borders.
const BANDS: Size = Size { columns: 40, rows: 12 };
const BANDS_SMALL: Size = Size { columns: 20, rows: 5 };
const NARROW: Size = Size { columns: 1, rows: 3 };
const TEXT: Size = Size { columns: 24, rows: 8 };
const FIELDS: Size = Size { columns: 32, rows: 10 };
const SCROLL: Size = Size { columns: 30, rows: 8 };
const ROLES: Size = Size { columns: 80, rows: 24 };
// More rows and columns than SCROLL has; two rows expose upward caret scrolling.
const OVERSIZED: Size = Size { columns: 100, rows: 40 };
const TWO_FIELDS: Size = Size { columns: 32, rows: 2 };
const NO_ROWS: Size = Size { columns: 30, rows: 0 };
const NO_COLUMNS: Size = Size { columns: 0, rows: 8 };
// Ten cells force the area onto three rows, leaving the fourth for Tail.
const FIELD_EDGE: Size = Size { columns: 10, rows: 4 };

#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written frames are the renderer's independent oracle")]
#[expect(clippy::disallowed_types, reason = "lossy strings make a failed byte comparison readable in test diagnostics")]
fn golden_frames_lay_bands_text_fields_scroll_and_roles() {
    let bands = fixtures::bands();
    let text = fixtures::text();
    let fields = fixtures::fields();
    let scroll = fixtures::scroll();
    let roles = fixtures::roles();
    let theme = fixtures::theme();
    for (name, tree, size, node, caret, expected) in [
        (
            "bands",
            &bands,
            BANDS,
            Some(fixtures::id(&bands, b"Second")),
            0,
            include_bytes!("golden/bands.txt").as_slice(),
        ),
        ("narrow", &bands, NARROW, None, 0, include_bytes!("golden/narrow.txt").as_slice()),
        (
            "bands-small",
            &bands,
            BANDS_SMALL,
            Some(fixtures::id(&bands, b"Second")),
            0,
            include_bytes!("golden/bands-small.txt").as_slice(),
        ),
        ("text", &text, TEXT, None, 0, include_bytes!("golden/text.txt").as_slice()),
        (
            "fields",
            &fields,
            FIELDS,
            Some(fixtures::id(&fields, b"Name")),
            3,
            include_bytes!("golden/fields.txt").as_slice(),
        ),
        (
            "fields-scrolled",
            &fields,
            FIELDS,
            Some(fixtures::id(&fields, b"Overflow")),
            35,
            include_bytes!("golden/fields-scrolled.txt").as_slice(),
        ),
        (
            "scroll",
            &scroll,
            SCROLL,
            Some(fixtures::id(&scroll, b"End")),
            0,
            include_bytes!("golden/scroll.txt").as_slice(),
        ),
        ("roles", &roles, ROLES, None, 0, include_bytes!("golden/roles.txt").as_slice()),
    ] {
        let mut frame = Frame::new(size);
        let mut focus = Focus { node, caret, ..Focus::default() };
        render(tree, &theme, size, &mut focus, &mut frame);
        let printed = frame.print();
        assert_eq!(String::from_utf8_lossy(&printed), String::from_utf8_lossy(expected), "{name}");
        assert_eq!(printed.as_ref(), expected, "{name}");
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "focus, paint and caret rows share one test process")]
fn rendering_honours_focus_paints_scroll_and_frame_bounds() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Main);
    b.region(fixtures::TOP);
    b.style(fixtures::RED);
    b.open(Role::TextField);
    b.name(b"Requested");
    b.bind(fixtures::binding());
    b.style(fixtures::BLUE);
    b.value(b"abc", 0);
    b.focus(4);
    b.close();
    b.close();
    fixtures::button(&mut b, b"Old");
    b.close();
    let requests = fixtures::finish(b);
    let scroll = fixtures::scroll();
    let fields = fixtures::fields();
    let theme = fixtures::theme();
    let requested = fixtures::id(&requests, b"Requested");
    let old = fixtures::id(&requests, b"Old");
    let end = fixtures::id(&scroll, b"End");
    // The scroll story has 21 rows; eight visible rows need offset 13.
    let small = SCROLL;
    let large = OVERSIZED;
    for (name, tree, initial, size, most, expected, cursor) in [
        (
            "new request",
            &requests,
            Focus { node: Some(old), caret: 1, scroll: 0, honoured: 3 },
            small,
            small,
            Focus { node: Some(requested), caret: 3, scroll: 0, honoured: 4 },
            Some(Position { row: 1, column: 17 }),
        ),
        (
            "old request",
            &requests,
            Focus { node: Some(old), caret: 1, scroll: 0, honoured: 5 },
            small,
            small,
            Focus { node: Some(old), caret: 1, scroll: 0, honoured: 5 },
            None,
        ),
        (
            "equal request",
            &requests,
            Focus { node: Some(old), caret: 1, scroll: 0, honoured: 4 },
            small,
            small,
            Focus { node: Some(old), caret: 1, scroll: 0, honoured: 4 },
            None,
        ),
        (
            "gone focus",
            &Tree::new(0),
            Focus { node: Some(old), caret: 1, scroll: 99, honoured: 5 },
            small,
            small,
            Focus { node: None, caret: 1, scroll: 0, honoured: 5 },
            None,
        ),
        (
            "noninteractive focus",
            &requests,
            Focus { node: Some(requests.get(0).expect("page exists").id), caret: 0, scroll: 0, honoured: 5 },
            small,
            small,
            Focus { node: None, caret: 0, scroll: 0, honoured: 5 },
            None,
        ),
        (
            "least scroll",
            &scroll,
            Focus { node: Some(end), ..Focus::default() },
            small,
            small,
            Focus { node: Some(end), caret: 0, scroll: 13, honoured: 0 },
            None,
        ),
        (
            "visible focus preserves scroll",
            &scroll,
            Focus { node: Some(end), caret: 0, scroll: 13, honoured: 0 },
            small,
            small,
            Focus { node: Some(end), caret: 0, scroll: 13, honoured: 0 },
            None,
        ),
        (
            "focus above the viewport",
            &fields,
            Focus { node: Some(fixtures::id(&fields, b"Name")), caret: 3, scroll: 4, honoured: 0 },
            TWO_FIELDS,
            TWO_FIELDS,
            Focus { node: Some(fixtures::id(&fields, b"Name")), caret: 3, scroll: 0, honoured: 0 },
            Some(Position { row: 0, column: 11 }),
        ),
        (
            "zero rows",
            &scroll,
            Focus { node: Some(end), caret: 0, scroll: 99, honoured: 0 },
            NO_ROWS,
            NO_ROWS,
            Focus { node: Some(end), caret: 0, scroll: 0, honoured: 0 },
            None,
        ),
        (
            "zero columns",
            &scroll,
            Focus { node: Some(end), caret: 0, scroll: 99, honoured: 0 },
            NO_COLUMNS,
            NO_COLUMNS,
            Focus { node: Some(end), caret: 0, scroll: 0, honoured: 0 },
            None,
        ),
        (
            "oversized render",
            &scroll,
            Focus { node: Some(end), ..Focus::default() },
            large,
            small,
            Focus { node: Some(end), caret: 0, scroll: 13, honoured: 0 },
            None,
        ),
    ] {
        let mut focus = initial;
        let mut frame = Frame::new(most);
        render(tree, &theme, size, &mut focus, &mut frame);
        assert_eq!(focus, expected, "{name}");
        assert_eq!(frame.scroll(), expected.scroll, "{name}");
        assert_eq!(frame.cursor(), cursor, "{name}");
        assert_eq!(frame.size(), most, "{name}");
        let mut reference = Frame::new(most);
        let mut reference_focus = initial;
        render(tree, &theme, most, &mut reference_focus, &mut reference);
        assert_eq!(frame, reference, "{name}: each dimension is cut to most");
    }
    let under = Paint {
        fore: Some(crate::Rgb { red: 0, green: 255, blue: 0 }),
        back: Some(crate::Rgb { red: 255, green: 255, blue: 255 }),
        dim: true,
        ..Paint::NONE
    };
    for (name, region, expected) in [
        ("nearest token last", Paint::NONE, Paint { bold: true, ..fixtures::BLUE_PAINT }),
        (
            "tokens over region",
            under,
            Paint {
                bold: true,
                dim: true,
                back: Some(crate::Rgb { red: 255, green: 255, blue: 255 }),
                ..fixtures::BLUE_PAINT
            },
        ),
    ] {
        let mut theme = fixtures::theme();
        theme.regions.get_mut(0).expect("top look exists").paint = region;
        let mut frame = Frame::new(small);
        let mut focus = Focus { honoured: 5, ..Focus::default() };
        render(&requests, &theme, small, &mut focus, &mut frame);
        let cell = match frame.row(1) {
            Some(row) => row.get(3),
            None => None,
        };
        assert_eq!(cell, Some(&Cell { glyph: 'R', paint: expected }), "{name}");
    }
    for (name, role, value, entry, caret, expected, cursor) in [
        (
            "field end scroll",
            Role::TextField,
            b"abcdefghij".as_slice(),
            EntryKind::Text,
            Some(10),
            "> F: ghij\nTail\n\n\n",
            Some(Position { row: 0, column: 9 }),
        ),
        (
            "wide value caret",
            Role::TextField,
            "a界bc".as_bytes(),
            EntryKind::Text,
            Some(4),
            "> F: a界bc\nTail\n\n\n",
            Some(Position { row: 0, column: 8 }),
        ),
        (
            "mask per character",
            Role::TextField,
            "a\u{301}界".as_bytes(),
            EntryKind::Secret,
            None,
            "  F: ●●●\nTail\n\n\n",
            None,
        ),
        (
            "area caret wraps",
            Role::TextArea,
            b"one two three".as_slice(),
            EntryKind::Text,
            Some(13),
            "> F: one\n  two\n  three\nTail\n",
            Some(Position { row: 2, column: 7 }),
        ),
        (
            "field newline is space",
            Role::TextField,
            b"ab\ncd".as_slice(),
            EntryKind::Text,
            None,
            "  F: ab cd\nTail\n\n\n",
            None,
        ),
        (
            "placeholder caret",
            Role::TextField,
            b"".as_slice(),
            EntryKind::Text,
            Some(0),
            "> F: hint\nTail\n\n\n",
            Some(Position { row: 0, column: 5 }),
        ),
    ] {
        let tree = fixtures::field_tail(role, value, entry);
        let mut focus = match caret {
            Some(caret) => Focus { node: Some(fixtures::id(&tree, b"F")), caret, ..Focus::default() },
            None => Focus::default(),
        };
        let mut frame = Frame::new(FIELD_EDGE);
        render(&tree, &theme, FIELD_EDGE, &mut focus, &mut frame);
        assert_eq!(frame.text().as_ref(), expected.as_bytes(), "{name}");
        assert_eq!(frame.cursor(), cursor, "{name}");
    }
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    for text in [b"First".as_slice(), b"Last"] {
        b.open(Role::ContentInfo);
        b.region(fixtures::BOTTOM);
        fixtures::leaf(&mut b, Role::Status, text);
        b.close();
    }
    b.close();
    let footers = fixtures::finish(b);
    // The lowest footer gets the only row, before the earlier footer.
    let one_row = Size { rows: 1, columns: 10 };
    let mut frame = Frame::new(one_row);
    render(&footers, &theme, one_row, &mut Focus::default(), &mut frame);
    assert_eq!(frame.text().as_ref(), b"Last\n");
}
