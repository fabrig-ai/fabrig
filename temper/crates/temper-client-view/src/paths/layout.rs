//! The two rows and judgement rises (temper/docs/design/client/README.md, 5.16).

use super::{KIND, Mark, Stop, decode};
use crate::look::{WAYS_AGENT, WAYS_AUTH, WAYS_BAR, WAYS_BASE, WAYS_CODE, WAYS_PERSON, WAYS_WORDS};
use skein_lib::List;
use skein_ui::{
    DrawingKind,
    scene::{Anchor, Command, Point, Scene},
};

const LEFT: u16 = 12;
const MARGINS: u16 = 24;
const STEP: u16 = 48;
const UPPER: u16 = 40;
const BASE: u16 = 106;
const BOUNDARY: u16 = 72;
const HEIGHT: u16 = 140;

/// Draw for the available width, independently of the measured height.
pub fn draw(kind: DrawingKind, data: &[u8], width: u16, _height: u16, scene: &mut Scene) -> bool {
    if kind != KIND {
        return false;
    }
    let Some(stops) = decode(data) else {
        return false;
    };
    paint(stops.as_slice(), width, scene).is_some()
}

fn row(mark: Mark) -> u16 {
    match mark {
        Mark::Person | Mark::Agent => UPPER,
        Mark::Code | Mark::Bar => BASE,
    }
}

fn point(x: u16, y: u16) -> Point {
    Point { x, y }
}

fn paint(stops: &[Stop<'_>], width: u16, scene: &mut Scene) -> Option<()> {
    let count = u16::try_from(stops.len()).ok()?;
    let gaps = count.checked_sub(1)?;
    let span = width.checked_sub(MARGINS)?;
    if span < gaps.checked_mul(STEP)? {
        return None;
    }
    let first = stops.first()?;
    let mut previous = point(LEFT, row(first.mark));
    let mut commands = List::with_capacity(u32::from(gaps).checked_mul(3)?.checked_add(1)?);
    commands.push(Command::Move(previous)).ok()?;
    for (index, stop) in stops.iter().enumerate().skip(1) {
        let index = u32::try_from(index).ok()?;
        let offset = u32::from(span).checked_mul(index)?.checked_div(u32::from(gaps))?;
        let x = LEFT.checked_add(u16::try_from(offset).ok()?)?;
        let quarter = x.checked_sub(previous.x)?.checked_div(4)?;
        let down = previous.x.checked_add(quarter)?;
        let up = x.checked_sub(quarter)?;
        commands.push(Command::Cubic(point(down, previous.y), point(down, BASE), point(down, BASE))).ok()?;
        commands.push(Command::Line(point(up, BASE))).ok()?;
        commands.push(Command::Cubic(point(up, BASE), point(up, row(stop.mark)), point(x, row(stop.mark)))).ok()?;
        previous = point(x, row(stop.mark));
    }
    if !scene.line(point(0, BOUNDARY), point(width, BOUNDARY), &[WAYS_AUTH])
        || !scene.path(commands.as_slice(), &[WAYS_BASE])
    {
        return None;
    }
    for (index, stop) in stops.iter().enumerate() {
        let index = u32::try_from(index).ok()?;
        let offset = if gaps == 0 { 0 } else { u32::from(span).checked_mul(index)?.checked_div(u32::from(gaps))? };
        let x = LEFT.checked_add(u16::try_from(offset).ok()?)?;
        stop_mark(*stop, x, scene)?;
        let y = match stop.mark {
            Mark::Person | Mark::Agent => 22,
            Mark::Code => 131,
            Mark::Bar => 88,
        };
        let anchor = if index == 0 {
            Anchor::Start
        } else if index == u32::from(gaps) {
            Anchor::End
        } else {
            Anchor::Middle
        };
        let words_x = if index == 0 { 2 } else { x };
        if !scene.text(point(words_x, y), anchor, stop.words, &[WAYS_WORDS]) {
            return None;
        }
    }
    scene.set_height(HEIGHT);
    Some(())
}

fn stop_mark(stop: Stop<'_>, x: u16, scene: &mut Scene) -> Option<()> {
    let drawn = match stop.mark {
        Mark::Person => scene.circle(point(x, UPPER), 7, &[WAYS_PERSON]),
        Mark::Agent => scene.path(
            &[
                Command::Move(point(x, 31)),
                Command::Quadratic(point(x.checked_add(2)?, 38), point(x.checked_add(9)?, UPPER)),
                Command::Quadratic(point(x.checked_add(2)?, 42), point(x, 49)),
                Command::Quadratic(point(x.checked_sub(2)?, 42), point(x.checked_sub(9)?, UPPER)),
                Command::Quadratic(point(x.checked_sub(2)?, 38), point(x, 31)),
                Command::Close,
            ],
            &[WAYS_AGENT],
        ),
        Mark::Code => scene.rect(point(x.checked_sub(5)?, BASE.checked_sub(5)?), point(10, 10), 2, &[WAYS_CODE]),
        Mark::Bar => scene.rect(point(x.checked_sub(1)?, BASE.checked_sub(11)?), point(3, 22), 1, &[WAYS_BAR]),
    };
    drawn.then_some(())
}
