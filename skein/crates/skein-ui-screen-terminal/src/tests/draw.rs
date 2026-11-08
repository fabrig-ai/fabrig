use crate::{Depth, ENTER, LEAVE, draw, draw_bound};
use skein_lib::List;
use skein_ui_screen::{Cell, Frame, Paint, Position, Rgb, Size};

// Small enough to read the complete byte oracle; enough columns for two runs.
const SMALL: Size = Size { rows: 1, columns: 4 };
fn frame(text: &str, paint: Paint) -> Frame {
    let mut frame = Frame::new(SMALL);
    frame.clear(SMALL, Cell::BLANK);
    for (column, glyph) in text.chars().enumerate() {
        assert!(
            frame.put(Position { row: 0, column: u16::try_from(column).expect("small column") }, Cell { glyph, paint })
        );
    }
    frame
}
fn drawn(last: Option<&Frame>, next: &Frame, depth: Depth) -> List<u8> {
    let mut bytes = List::with_capacity(draw_bound(next.most()).expect("small bound"));
    draw(last, next, depth, &mut bytes);
    bytes
}
#[test]
fn first_and_resized_frames_clear_and_write_every_cell() {
    let next = frame("abcd", Paint::NONE);
    let mut smaller = Frame::new(SMALL);
    smaller.clear(Size { rows: 1, columns: 1 }, Cell::BLANK);
    for last in [None, Some(&smaller)] {
        assert_eq!(drawn(last, &next, Depth::True).as_slice(), b"\x1b[2J\x1b[1;1H\x1b[0mabcd\x1b[?25l");
    }
    let mut two = Frame::new(Size { rows: 2, columns: 1 });
    two.clear(Size { rows: 2, columns: 1 }, Cell::BLANK);
    assert_eq!(drawn(None, &two, Depth::True).as_slice(), b"\x1b[2J\x1b[1;1H\x1b[0m \x1b[2;1H \x1b[?25l");
}
#[test]
fn differences_write_only_changed_runs_and_leave_cursor_last() {
    let last = frame("abcd", Paint::NONE);
    for (text, expected) in [
        ("abcd", b"\x1b[?25l".as_slice()),
        ("aXYd", b"\x1b[1;2H\x1b[0mXY\x1b[?25l"),
        ("XbYd", b"\x1b[1;1H\x1b[0mX\x1b[1;3HY\x1b[?25l"),
        ("abc ", b"\x1b[1;4H\x1b[0m \x1b[?25l"),
    ] {
        let next = frame(text, Paint::NONE);
        assert_eq!(drawn(Some(&last), &next, Depth::True).as_slice(), expected, "{text}");
    }
    let mut next = frame("aXYd", Paint::NONE);
    next.set_cursor(Some(Position { row: 0, column: 2 }));
    assert_eq!(drawn(Some(&last), &next, Depth::True).as_slice(), b"\x1b[1;2H\x1b[0mXY\x1b[1;3H\x1b[?25h");
    let mut same = last.clone();
    same.set_cursor(Some(Position { row: 0, column: 0 }));
    assert_eq!(drawn(Some(&last), &same, Depth::True).as_slice(), b"\x1b[1;1H\x1b[?25h");
}
#[test]
fn paints_change_where_needed_and_reset_to_terminal_defaults() {
    let red = Rgb { red: 255, green: 0, blue: 0 };
    for depth in [Depth::True, Depth::Indexed] {
        let mut defaults = frame("abcd", Paint::NONE);
        assert!(defaults.put(
            Position { row: 0, column: 0 },
            Cell { glyph: 'a', paint: Paint { fore: Some(red), back: Some(red), ..Paint::NONE } }
        ));
        let bytes = drawn(None, &defaults, depth);
        let start = skein_lib::bytes::find(bytes.as_slice(), b"a")
            .expect("coloured cell exists")
            .checked_add(1)
            .expect("small output");
        let tail = bytes.as_slice().get(start..).expect("default cells follow");
        assert_eq!(tail, b"\x1b[0mbcd\x1b[?25l", "default colours follow the reset alone: {depth:?}");
    }
    let paint =
        Paint { fore: Some(red), bold: true, dim: true, italic: true, underline: true, reverse: true, ..Paint::NONE };
    let mut next = frame("abcd", paint);
    assert!(next.put(Position { row: 0, column: 2 }, Cell { glyph: 'c', paint: Paint::NONE }));
    assert!(next.put(Position { row: 0, column: 3 }, Cell { glyph: 'd', paint: Paint::NONE }));
    assert_eq!(
        drawn(None, &next, Depth::True).as_slice(),
        b"\x1b[2J\x1b[1;1H\x1b[0m\x1b[1m\x1b[2m\x1b[3m\x1b[4m\x1b[7m\x1b[38;2;255;0;0mab\x1b[0mcd\x1b[?25l"
    );
    let last = frame("abcd", Paint::NONE);
    let next = frame("abcd", paint);
    assert_eq!(
        drawn(Some(&last), &next, Depth::Indexed).as_slice(),
        b"\x1b[1;1H\x1b[0m\x1b[1m\x1b[2m\x1b[3m\x1b[4m\x1b[7m\x1b[38;5;196mabcd\x1b[?25l"
    );
}
#[test]
fn true_and_indexed_colours_follow_rgb_and_nearest_palette_entries() {
    for (rgb, true_bytes, indexed_bytes) in [
        (
            Rgb { red: 255, green: 0, blue: 0 },
            b"\x1b[38;2;255;0;0m\x1b[48;2;255;0;0m".as_slice(),
            b"\x1b[38;5;196m\x1b[48;5;196m".as_slice(),
        ),
        (
            Rgb { red: 128, green: 128, blue: 128 },
            b"\x1b[38;2;128;128;128m\x1b[48;2;128;128;128m",
            b"\x1b[38;5;244m\x1b[48;5;244m",
        ),
        (
            Rgb { red: 224, green: 138, blue: 104 },
            b"\x1b[38;2;224;138;104m\x1b[48;2;224;138;104m",
            b"\x1b[38;5;173m\x1b[48;5;173m",
        ),
        (Rgb { red: 0, green: 0, blue: 0 }, b"\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m", b"\x1b[38;5;16m\x1b[48;5;16m"),
        (
            Rgb { red: 255, green: 255, blue: 255 },
            b"\x1b[38;2;255;255;255m\x1b[48;2;255;255;255m",
            b"\x1b[38;5;231m\x1b[48;5;231m",
        ),
    ] {
        let next = frame("abcd", Paint { fore: Some(rgb), back: Some(rgb), ..Paint::NONE });
        for (depth, colours) in [(Depth::True, true_bytes), (Depth::Indexed, indexed_bytes)] {
            let expected = skein_lib::bytes::concat(&[b"\x1b[2J\x1b[1;1H\x1b[0m", colours, b"abcd\x1b[?25l"]);
            assert_eq!(drawn(None, &next, depth).as_slice(), &*expected, "{rgb:?}, {depth:?}");
        }
    }
}
#[test]
fn wide_characters_write_once_and_a_removed_tail_is_erased() {
    let last = frame("界\0cd", Paint::NONE);
    assert_eq!(drawn(None, &last, Depth::True).as_slice(), "\x1b[2J\x1b[1;1H\x1b[0m界cd\x1b[?25l".as_bytes());
    let next = frame("a cd", Paint::NONE);
    assert_eq!(drawn(Some(&last), &next, Depth::True).as_slice(), b"\x1b[1;1H\x1b[0ma \x1b[?25l");
}
#[test]
fn drawing_bound_holds_full_paint_changes_and_refuses_overflow() {
    // Enough columns to cover two-digit moves; the final cursor uses both
    // largest coordinates. Every cell has all attributes and six RGB channels.
    let most = Size { rows: 3, columns: 12 };
    let a = Rgb { red: 100, green: 200, blue: 255 };
    let b = Rgb { red: 255, green: 100, blue: 200 };
    let mut next = Frame::new(most);
    next.clear(most, Cell::BLANK);
    for row in 0..most.rows {
        for column in 0..most.columns {
            let fore = if column % 2 == 0 { a } else { b };
            let back = if column % 2 == 0 { b } else { a };
            let paint = Paint {
                fore: Some(fore),
                back: Some(back),
                bold: true,
                dim: true,
                italic: true,
                underline: true,
                reverse: true,
            };
            assert!(next.put(Position { row, column }, Cell { glyph: '🙂', paint }));
        }
    }
    next.set_cursor(Some(Position { row: 2, column: 11 }));
    let mut last = Frame::new(most);
    last.clear(most, Cell::BLANK);
    for previous in [None, Some(&last)] {
        for depth in [Depth::True, Depth::Indexed] {
            let bytes = drawn(previous, &next, depth);
            assert!(bytes.len() <= draw_bound(most).expect("small bound"));
        }
    }
    // The bound is 80 per cell plus 24; these are either side of its u32 edge.
    assert!(draw_bound(Size { rows: 820, columns: 65472 }).is_some());
    assert!(draw_bound(Size { rows: 820, columns: 65473 }).is_none());
    assert_eq!(draw_bound(Size { rows: u16::MAX, columns: u16::MAX }), None);
    assert_eq!(draw_bound(Size { rows: 0, columns: 0 }), Some(24));
}
#[test]
fn entering_and_leaving_are_the_designs_exact_mode_bytes() {
    assert_eq!(ENTER, b"\x1b[?1049h\x1b[?25l\x1b[?2004h\x1b[?7l\x1b[2J\x1b[H");
    assert_eq!(LEAVE, b"\x1b[?2004l\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l");
}
