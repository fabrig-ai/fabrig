//! The process judge reads a screen even when no write contains the words whole.
use skein_lib::List;
use skein_ui_screen::{Cell, Frame, Position, Size};
use skein_ui_screen_terminal::{Depth, draw, draw_bound};
use temper_box_kit::screen::Screen;

fn row(frame: &mut Frame, number: u16, words: &str) {
    let mut glyphs = words.chars();
    for column in 0..frame.size().columns {
        let cell = Cell { glyph: glyphs.next().unwrap_or(' '), ..Cell::BLANK };
        assert!(frame.put(Position { row: number, column }, cell));
    }
}

fn read(screen: &mut Screen, last: &Frame, next: &Frame, raw: &mut Vec<u8>) {
    let mut bytes = List::with_capacity(draw_bound(next.size()).expect("draw bound"));
    draw(Some(last), next, Depth::Indexed, &mut bytes);
    raw.extend_from_slice(bytes.as_slice());
    // Reads can split both UTF-8 glyphs and cursor/paint sequences.
    for byte in bytes.as_slice() {
        screen.read(&[*byte]);
    }
}

#[test]
fn incremental_echo_and_an_account_wait_redraw_are_read_as_the_working_chat() {
    let size = Size { columns: 80, rows: 24 };
    let mut screen = Screen::new(size.columns, size.rows);
    screen.watch(b"Add CSV export to widget");
    screen.watch(b"Working");
    let mut frame = Frame::new(size);
    frame.clear(size, Cell::BLANK);
    let mut raw = Vec::new();
    let goal = "Add CSV export to widget";
    for end in 1..=goal.len() {
        let before = frame.clone();
        row(&mut frame, 2, &goal[..end]);
        read(&mut screen, &before, &frame, &mut raw);
    }
    let before = frame.clone();
    row(&mut frame, 19, "✦ Waiting for an account");
    read(&mut screen, &before, &frame, &mut raw);
    assert!(screen.has(b"Waiting for an account"));
    let before = frame.clone();
    row(&mut frame, 19, "✦ Working");
    read(&mut screen, &before, &frame, &mut raw);

    assert!(skein_lib::bytes::find(&raw, goal.as_bytes()).is_none(), "the echo arrived one key per frame");
    assert!(skein_lib::bytes::find(&raw, b"Working").is_none(), "the redraw retained W and ing from Waiting");
    assert!(screen.has(goal.as_bytes()), "{}", screen.text());
    assert!(screen.has(b"Working"), "{}", screen.text());
    assert!(!screen.has(b"Waiting for an account"), "old screen words were erased");

    let before = frame.clone();
    row(&mut frame, 19, "✦ Waiting for your words");
    let mut bytes = List::with_capacity(draw_bound(size).expect("draw bound"));
    draw(Some(&before), &frame, Depth::Indexed, &mut bytes);
    // The next draw can be coalesced into the same terminal read as Working.
    let mut coalesced = Screen::new(size.columns, size.rows);
    coalesced.watch(b"Working");
    let mut first = List::with_capacity(draw_bound(size).expect("draw bound"));
    draw(None, &before, Depth::Indexed, &mut first);
    let both = [first.as_slice(), bytes.as_slice()].concat();
    coalesced.read(&both);
    assert!(!coalesced.has(b"Working"));
    assert!(coalesced.seen(b"Working"), "a complete displayed frame is remembered across a later redraw");
    assert!(screen.seen(goal.as_bytes()));
    assert!(screen.seen(b"Working"));
}
