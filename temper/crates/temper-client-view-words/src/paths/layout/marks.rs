//! Stops, their acts and their words (temper/docs/design/client/pages/chat/conversation.md, 3).

use super::{Decoded, Layout, Position, judgement};
use crate::{
    look::{
        WAYS_AGENT, WAYS_CODE, WAYS_END, WAYS_HALO, WAYS_HIT, WAYS_LANDING, WAYS_MAIN, WAYS_MAYBE, WAYS_OPEN,
        WAYS_PERSON, WAYS_STOP, WAYS_WHY, WAYS_WORDS,
    },
    paths::{Mark, Stop},
};
use skein_ui::scene::{Anchor, Command, Point, Scene};

pub(super) fn draw(
    stop: Stop<'_>,
    index: u32,
    end: bool,
    at: Position,
    data: &Decoded<'_>,
    layout: &Layout,
    scene: &mut Scene,
) -> Option<()> {
    let bar = stop.mark == Mark::Bar;
    let open = data.open == Some(index);
    if bar && !end && !scene.group(Point { x: 0, y: 0 }, 2, &[WAYS_LANDING]) {
        return None;
    }
    if data.pressed && !bar {
        if !scene.group(
            Point { x: 0, y: 0 },
            (if open { 4_u32 } else { 3 }).checked_add(u32::from(!stop.over.is_empty()))?,
            &[WAYS_STOP],
        ) {
            return None;
        }
        let act = [u8::try_from(index).ok()?];
        if !scene.act(&act) {
            return None;
        }
        hit(at, judgement(stop.mark), layout.across, scene)?;
        if !scene.act(&act) {
            return None;
        }
        if open && !scene.circle(at.point()?, 15, &[WAYS_HALO]) {
            return None;
        }
    }
    mark(stop, at, layout.across, scene)?;
    let (words, anchor) = if layout.across {
        (
            Position {
                x: if index == 0 && !bar { 200 } else { at.x },
                y: match stop.mark {
                    Mark::Person | Mark::Agent => 2200,
                    Mark::Code => 13100,
                    Mark::Bar => 8800,
                },
            },
            if end {
                Anchor::End
            } else if index == 0 && !bar {
                Anchor::Start
            } else {
                Anchor::Middle
            },
        )
    } else {
        let upper = judgement(stop.mark);
        (
            Position {
                x: if upper { at.x.checked_add(1700)? } else { at.x.checked_sub(if end { 1800 } else { 1700 })? },
                y: at.y.checked_add(400)?,
            },
            if upper { Anchor::Start } else { Anchor::End },
        )
    };
    let styles = if open {
        &[WAYS_WORDS, WAYS_OPEN][..]
    } else if bar {
        &[WAYS_WORDS, WAYS_END][..]
    } else {
        &[WAYS_WORDS][..]
    };
    if !scene.text(words.point()?, anchor, stop.words, styles) {
        return None;
    }
    if !stop.over.is_empty() {
        let (at, anchor) = if layout.across {
            (Position { x: at.x, y: 900 }, Anchor::Middle)
        } else {
            (Position { x: words.x, y: words.y.checked_add(1200)? }, anchor)
        };
        if !scene.text(at.point()?, anchor, stop.over, &[WAYS_WHY]) {
            return None;
        }
    }
    Some(())
}

fn hit(at: Position, upper: bool, across: bool, scene: &mut Scene) -> Option<()> {
    let (corner, size) = if across {
        // Scene coordinates are unsigned: clip the first hit area at the box's left edge.
        let left = at.x.saturating_sub(3000);
        (
            Position { x: left, y: if upper { 600 } else { 9400 } },
            Point {
                x: u16::try_from(at.x.checked_add(3000)?.checked_sub(left)?.checked_div(100)?).ok()?,
                y: if upper { 52 } else { 44 },
            },
        )
    } else {
        (
            Position { x: at.x.checked_sub(if upper { 1200 } else { 14000 })?, y: at.y.checked_sub(1400)? },
            Point { x: if upper { 150 } else { 152 }, y: 28 },
        )
    };
    scene.rect(corner.point()?, size, 0, &[WAYS_HIT]).then_some(())
}

fn mark(stop: Stop<'_>, at: Position, across: bool, scene: &mut Scene) -> Option<()> {
    let agent = if stop.needed { &[WAYS_AGENT, WAYS_MAYBE][..] } else { &[WAYS_AGENT][..] };
    let code = if stop.needed { &[WAYS_CODE, WAYS_MAYBE][..] } else { &[WAYS_CODE][..] };
    let drawn = match stop.mark {
        Mark::Person => scene.circle(at.point()?, 7, &[WAYS_PERSON]),
        Mark::Agent => scene.path(
            &[
                Command::Move(Position { x: at.x, y: at.y.checked_sub(900)? }.point()?),
                Command::Quadratic(
                    Position { x: at.x.checked_add(180)?, y: at.y.checked_sub(180)? }.point()?,
                    Position { x: at.x.checked_add(900)?, y: at.y }.point()?,
                ),
                Command::Quadratic(
                    Position { x: at.x.checked_add(180)?, y: at.y.checked_add(180)? }.point()?,
                    Position { x: at.x, y: at.y.checked_add(900)? }.point()?,
                ),
                Command::Quadratic(
                    Position { x: at.x.checked_sub(180)?, y: at.y.checked_add(180)? }.point()?,
                    Position { x: at.x.checked_sub(900)?, y: at.y }.point()?,
                ),
                Command::Quadratic(
                    Position { x: at.x.checked_sub(180)?, y: at.y.checked_sub(180)? }.point()?,
                    Position { x: at.x, y: at.y.checked_sub(900)? }.point()?,
                ),
                Command::Close,
            ],
            agent,
        ),
        Mark::Code => scene.rect(
            Position { x: at.x.checked_sub(500)?, y: at.y.checked_sub(500)? }.point()?,
            Point { x: 10, y: 10 },
            2,
            code,
        ),
        Mark::Bar => {
            let (dx, dy, size) =
                if across { (150, 1100, Point { x: 3, y: 22 }) } else { (1100, 150, Point { x: 22, y: 3 }) };
            scene.rect(Position { x: at.x.checked_sub(dx)?, y: at.y.checked_sub(dy)? }.point()?, size, 1, &[WAYS_MAIN])
        }
    };
    drawn.then_some(())
}
