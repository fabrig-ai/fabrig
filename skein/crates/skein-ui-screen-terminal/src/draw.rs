//! Frame differences as terminal bytes (skein/docs/design/ui/screen.md, 7 and 8).
use skein_lib::{Decimal, List};
use skein_ui_screen::{Frame, Paint, Position, Rgb, Size};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth {
    True,
    Indexed,
}

pub const ENTER: &[u8] = b"\x1b[?1049h\x1b[?25l\x1b[?2004h\x1b[?7l\x1b[2J\x1b[H";
pub const LEAVE: &[u8] = b"\x1b[?2004l\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l";
pub const ASK_BACKGROUND: &[u8] = b"\x1b]11;?\x1b\\";

pub fn draw(last: Option<&Frame>, next: &Frame, depth: Depth, out: &mut List<u8>) {
    let last = match last {
        Some(last) if last.size() == next.size() => Some(last),
        Some(_) | None => {
            put(out, b"\x1b[2J");
            None
        }
    };
    let mut paint = None;
    for row in 0..next.size().rows {
        let cells = next.row(row).expect("row inside frame");
        let previous = match last {
            Some(last) => last.row(row),
            None => None,
        };
        let mut run = false;
        for (column, cell) in cells.iter().enumerate() {
            let unchanged = match previous {
                Some(previous) => previous.get(column) == Some(cell),
                None => false,
            };
            if unchanged {
                run = false;
                continue;
            }
            if cell.glyph == '\0' {
                continue;
            }
            if !run {
                position(out, Position { row, column: u16::try_from(column).expect("column fits frame") });
                run = true;
            }
            if paint != Some(cell.paint) {
                painting(out, cell.paint, depth);
                paint = Some(cell.paint);
            }
            let mut buffer = [0; 4];
            put(out, cell.glyph.encode_utf8(&mut buffer).as_bytes());
        }
    }
    match next.cursor() {
        Some(cursor) => {
            position(out, cursor);
            put(out, b"\x1b[?25h");
        }
        None => put(out, b"\x1b[?25l"),
    }
}

/// Per cell: absolute move (14), reset and both RGB colours with every
/// attribute (62), and UTF-8 (4). Clear (4) and the final cursor (20) follow.
#[must_use]
pub fn draw_bound(most: Size) -> Option<u32> {
    u32::from(most.rows).checked_mul(u32::from(most.columns))?.checked_mul(80)?.checked_add(24)
}
fn put(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("output has draw_bound room");
    }
}
fn number(out: &mut List<u8>, value: u64) {
    let digits = Decimal::of(value);
    put(out, digits.as_bytes());
}
fn position(out: &mut List<u8>, at: Position) {
    put(out, b"\x1b[");
    number(out, u64::from(at.row).saturating_add(1));
    put(out, b";");
    number(out, u64::from(at.column).saturating_add(1));
    put(out, b"H");
}
fn painting(out: &mut List<u8>, paint: Paint, depth: Depth) {
    put(out, b"\x1b[0m");
    for (set, code) in
        [(paint.bold, b'1'), (paint.dim, b'2'), (paint.italic, b'3'), (paint.underline, b'4'), (paint.reverse, b'7')]
    {
        if set {
            put(out, &[0x1b, b'[', code, b'm']);
        }
    }
    if let Some(rgb) = paint.fore {
        colour(out, rgb, true, depth);
    }
    if let Some(rgb) = paint.back {
        colour(out, rgb, false, depth);
    }
}
fn colour(out: &mut List<u8>, rgb: Rgb, fore: bool, depth: Depth) {
    put(out, if fore { b"\x1b[38;" } else { b"\x1b[48;" });
    match depth {
        Depth::True => {
            put(out, b"2;");
            number(out, u64::from(rgb.red));
            put(out, b";");
            number(out, u64::from(rgb.green));
            put(out, b";");
            number(out, u64::from(rgb.blue));
        }
        Depth::Indexed => {
            put(out, b"5;");
            number(out, u64::from(indexed(rgb)));
        }
    }
    put(out, b"m");
}
fn distance(left: Rgb, right: Rgb) -> u32 {
    let red = u32::from(left.red.abs_diff(right.red));
    let green = u32::from(left.green.abs_diff(right.green));
    let blue = u32::from(left.blue.abs_diff(right.blue));
    red.saturating_mul(red).saturating_add(green.saturating_mul(green)).saturating_add(blue.saturating_mul(blue))
}
fn level(value: u8) -> u8 {
    if value == 0 { 0 } else { 55_u8.saturating_add(value.saturating_mul(40)) }
}
fn indexed(rgb: Rgb) -> u8 {
    // The first sixteen colours are terminal-defined. Compare the fixed cube
    // and ramp alone, breaking equal distances by their lower palette index.
    let mut best = 16;
    let mut least = u32::MAX;
    for red in 0_u8..6 {
        for green in 0_u8..6 {
            for blue in 0_u8..6 {
                let distance = distance(rgb, Rgb { red: level(red), green: level(green), blue: level(blue) });
                if distance < least {
                    least = distance;
                    best = 16_u8
                        .saturating_add(red.saturating_mul(36))
                        .saturating_add(green.saturating_mul(6))
                        .saturating_add(blue);
                }
            }
        }
    }
    for step in 0_u8..24 {
        let gray = 8_u8.saturating_add(step.saturating_mul(10));
        let distance = distance(rgb, Rgb { red: gray, green: gray, blue: gray });
        if distance < least {
            least = distance;
            best = 232_u8.saturating_add(step);
        }
    }
    best
}
