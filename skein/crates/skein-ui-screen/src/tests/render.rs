use super::fixtures::{self, LIMITS};
use crate::{Cell, Focus, Frame, Paint, Position, Size, render};
use skein_ui::{Builder, EntryKind, Role, State, Tree};

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
// ui-screen.md, section 9, and the task's whole-tree frame at six 24-row pages.
const EVERY: Size = Size { columns: 80, rows: 24 };
const EVERY_TALL: Size = Size { columns: 80, rows: 144 };
// A titled frame, two empty body rows and one spread footer row.
const PADDED_BANDS: Size = Size { columns: 16, rows: 6 };

#[test]
fn region_padding_keeps_columns_clear_without_adding_rows() {
    let tree = fixtures::styled_bands();
    let mut theme = fixtures::theme();
    for region in &mut theme.regions {
        region.pad = 1;
    }
    let mut frame = Frame::new(PADDED_BANDS);
    render(&tree, &theme, PADDED_BANDS, &mut Focus::default(), &mut frame);
    assert_eq!(frame.text().as_ref(), b"+- Title ------+\n| Inside       |\n+--------------+\n\n\n End\n");
}

#[test]
fn border_titles_keep_their_heading_paint() {
    let tree = fixtures::styled_bands();
    let mut frame = Frame::new(PADDED_BANDS);
    render(&tree, &fixtures::theme(), PADDED_BANDS, &mut Focus::default(), &mut frame);
    // The title follows the corner, an across glyph and a space.
    assert_eq!(frame.row(0).expect("top border exists").get(3), Some(&Cell { glyph: 'T', paint: fixtures::RED_PAINT }));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written frames are the independent every-role oracle")]
#[expect(clippy::disallowed_types, reason = "lossy strings make the golden's failed byte comparison readable")]
fn every_role_state_and_property_is_laid() {
    let tree = fixtures::every();
    let theme = fixtures::theme();
    let check = fixtures::selected_check(&tree);
    for (name, size, expected) in [
        ("every", EVERY, include_bytes!("golden/every.txt").as_slice()),
        ("every-tall", EVERY_TALL, include_bytes!("golden/every-tall.txt").as_slice()),
    ] {
        // Keep the check in view in the short frame; the fixture's field request
        // has already been honoured. The tall frame shows the same focus in context.
        let mut focus = Focus { node: Some(check), honoured: 1, ..Focus::default() };
        let mut frame = Frame::new(size);
        render(&tree, &theme, size, &mut focus, &mut frame);
        let printed = frame.print();
        assert_eq!(String::from_utf8_lossy(&printed), String::from_utf8_lossy(expected), "{name}");
        assert_eq!(printed.as_ref(), expected, "{name}");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written frames are the renderer's independent oracle")]
#[expect(clippy::disallowed_types, reason = "lossy strings make a failed byte comparison readable in test diagnostics")]
fn golden_frames_lay_bands_text_fields_scroll_and_roles() {
    let looks = fixtures::looks();
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
        ("looks", &looks, ROLES, Some(fixtures::id(&looks, b"Ask")), 0, include_bytes!("golden/looks.txt").as_slice()),
        (
            "looks-narrow",
            &looks,
            Size { columns: 30, rows: 12 },
            Some(fixtures::id(&looks, b"Ask")),
            0,
            include_bytes!("golden/looks-narrow.txt").as_slice(),
        ),
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

    // The option uses its select's disabled state; every other control owns it.
    let control_size = Size { columns: 40, rows: 2 };
    for (role, state, disabled, bound, available, expected) in [
        (Role::Choice, State::Selected, false, true, true, "> (*) Choose\n\n"),
        (Role::Choice, State::Current, false, true, true, "> ( ) Choose (current)\n\n"),
        (Role::Check, State::Selected, false, true, true, "> [x] Choose\n\n"),
        (Role::Check, State::Pressed, false, true, true, "> [x] Choose\n\n"),
        (Role::Check, State::Required, false, true, true, "> [ ] Choose (required)\n\n"),
        (Role::Option, State::Selected, false, true, true, "Select:\n  > (*) Choose\n"),
        (Role::Option, State::Current, false, true, true, "Select:\n  > ( ) Choose (current)\n"),
        (Role::Tab, State::Selected, false, true, true, "> Choose (selected)\n\n"),
        (Role::MenuItem, State::Pressed, false, true, true, "> Choose (pressed)\n\n"),
        (Role::Check, State::Pressed, true, true, false, "  [x] Choose (disabled)\n\n"),
        (Role::Option, State::Selected, true, true, false, "Select: (disabled)\n    (*) Choose (disabled)\n"),
        (Role::Option, State::Selected, false, false, false, "Select:\n    (*) Choose\n"),
    ] {
        let tree = fixtures::role_control(role, state, disabled, bound);
        let id = fixtures::id(&tree, b"Choose");
        let mut focus = Focus { node: Some(id), ..Focus::default() };
        let mut frame = Frame::new(control_size);
        render(&tree, &theme, control_size, &mut focus, &mut frame);
        assert_eq!(focus.node, available.then_some(id), "{role:?} {state:?} disabled={disabled} bound={bound}");
        assert_eq!(frame.text().as_ref(), expected.as_bytes(), "{role:?} {state:?} disabled={disabled} bound={bound}");
        assert_eq!(frame.cursor(), None, "non-field controls have no caret");
        let text_column = if role == Role::Option { 4 } else { 2 };
        let text_row = u16::from(role == Role::Option);
        let cell = frame.row(text_row).expect("control row exists").get(text_column).expect("mark or name exists");
        assert_eq!(cell.paint.reverse, available, "the focused mark and text take the focus paint");
    }

    // Rows name semantic edges; sizes expose each complete layout with no padding rows.
    for (name, tree, size, expected) in [
        (
            "disclosure descriptions",
            fixtures::disclosures(),
            Size { columns: 20, rows: 5 },
            "unnamed description\nchild\nMore? first\n  next (expanded)\n  child\n",
        ),
        (
            "empty headers and data, outside headers",
            fixtures::table_edges(),
            Size { columns: 40, rows: 8 },
            "table\nheader row\nempty header\n |\ndata (selected) (2 of 9)\ncontinued\nSorted (descending)\noutside row\n",
        ),
        (
            "row wraps at its room",
            fixtures::wrapped_row(),
            Size { columns: 20, rows: 2 },
            "alpha beta gamma\ndelta | epsilon\n",
        ),
        (
            "code cuts, quote marks repeat, progress and unnamed fallback",
            fixtures::blocks(),
            Size { columns: 12, rows: 7 },
            "    12345678\n    abcdefg\n    ok\n> one\n> two\nWork:\nfallback\n",
        ),
        (
            "whole log, menu, nested tree positions and first-line states",
            fixtures::log_menu_tree(),
            Size { columns: 40, rows: 11 },
            "Log:\n  oldest\n  newest\nMenu:\n    Run\n- branch (expanded) (2 of 3)\n  continued\n  item description\n  - inner (1)\n- Details: (expanded) (3 of 3)\n    detail\n",
        ),
    ] {
        let mut frame = Frame::new(size);
        render(&tree, &theme, size, &mut Focus::default(), &mut frame);
        assert_eq!(frame.text().as_ref(), expected.as_bytes(), "{name}");
    }
}

#[test]
#[expect(clippy::disallowed_types, reason = "the diagnostic collects every failed table row")]
fn rendering_lays_end_alignment_bands_and_title_parts() {
    use core::fmt::Write;
    let mut failures = String::new();
    let theme = fixtures::theme();
    // Eight columns expose the exact-width edge and leave a four-cell padded interior.
    let line_size = Size { columns: 8, rows: 1 };
    let band_size = Size { columns: 8, rows: 4 };
    let band_paint = Paint { back: fixtures::BAND_PAINT.back, ..fixtures::RED_PAINT };
    let edge_paint = Paint { back: fixtures::REGION_PAINT.back, ..fixtures::RED_PAINT };
    for (name, tree, size, expected, cells) in [
        ("exact room", fixtures::look_line(b"12345678", fixtures::END, None), line_size, "12345678\n", &[][..]),
        ("nearest end under start", fixtures::look_line(b"End", fixtures::END, None), line_size, "     End\n", &[][..]),
        (
            "band over region",
            fixtures::look_line(b"a\nb", fixtures::BAND, Some(fixtures::LOOKS_TOP)),
            band_size,
            "╭──────╮\n│ a    │\n│ b    │\n╰──────╯\n",
            &[
                (Position { row: 1, column: 0 }, Cell { glyph: '│', paint: edge_paint }),
                (Position { row: 1, column: 2 }, Cell { glyph: 'a', paint: band_paint }),
                (Position { row: 1, column: 5 }, Cell { glyph: ' ', paint: band_paint }),
                (Position { row: 2, column: 5 }, Cell { glyph: ' ', paint: band_paint }),
            ][..],
        ),
        (
            "title before end",
            fixtures::end_title(),
            Size { columns: 12, rows: 2 },
            "+- Title --+\n+----------+\n",
            &[][..],
        ),
    ] {
        let mut frame = Frame::new(size);
        render(&tree, &theme, size, &mut Focus::default(), &mut frame);
        if frame.text().as_ref() != expected.as_bytes() {
            writeln!(failures, "{name}: text differs from {expected}").expect("diagnostic writes");
        }
        for (at, cell) in cells {
            let drawn = match frame.row(at.row) {
                Some(row) => row.get(usize::from(at.column)),
                None => None,
            };
            if drawn != Some(cell) {
                writeln!(failures, "{name}: paint or glyph differs at {} {}", at.row, at.column)
                    .expect("diagnostic writes");
            }
        }
    }
    assert!(failures.is_empty(), "{failures}");
}
