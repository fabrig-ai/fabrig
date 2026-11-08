//! Width alone chooses the path's layout (temper/docs/design/client/README.md, 5.16).

use super::{Decoded, KIND, Mark, decode};
use crate::look::{WAYS_AUTH, WAYS_CARET};
use skein_ui::{
    DrawingKind,
    scene::{Point, Scene},
};

mod line;
mod marks;

/// Coordinates retain hundredths until the scene rounds down to whole pixels.
#[derive(Clone, Copy)]
struct Position {
    x: u32,
    y: u32,
}

impl Position {
    fn point(self) -> Option<Point> {
        Some(Point {
            x: u16::try_from(self.x.checked_div(100)?).ok()?,
            y: u16::try_from(self.y.checked_div(100)?).ok()?,
        })
    }
}

struct Layout {
    across: bool,
    span: u32,
    gaps: u32,
    offset: u32,
    end: u32,
    height: u16,
}

impl Layout {
    fn new(data: &Decoded<'_>, width: u16) -> Option<Layout> {
        let count = data.stops.len();
        let bar = data.stops.last()?.mark == Mark::Bar;
        let places = count.checked_sub(u32::from(bar))?;
        let gaps = places.saturating_sub(1);
        let span = u32::from(width).saturating_sub(80);
        let across = width >= 560 && span >= gaps.checked_mul(48)?;
        let end = if across {
            u32::from(width).checked_sub(8)?.checked_mul(100)?
        } else {
            places
                .saturating_sub(1)
                .checked_mul(40)?
                .checked_add(16)?
                .checked_add(if bar { 38 } else { 0 })?
                .checked_mul(100)?
        };
        let height = if across {
            if data.open.is_some() { 154 } else { 140 }
        } else {
            u16::try_from(end.checked_div(100)?.checked_add(if bar { 14 } else { 24 })?).ok()?
        };
        Some(Layout {
            across,
            span: span.checked_mul(100)?,
            gaps,
            offset: u32::from(width).saturating_sub(360).checked_div(2)?.checked_mul(100)?,
            end,
            height,
        })
    }

    fn along(&self, index: u32, end: bool) -> Option<u32> {
        if end {
            return Some(self.end);
        }
        if self.across {
            let offset = if self.gaps == 0 { 0 } else { self.span.checked_mul(index)?.checked_div(self.gaps)? };
            1200_u32.checked_add(offset)
        } else {
            index.checked_mul(4000)?.checked_add(1600)
        }
    }

    fn position(&self, along: u32, judgement: bool) -> Option<Position> {
        if self.across {
            Some(Position { x: along, y: if judgement { 4000 } else { 10600 } })
        } else {
            Some(Position { x: self.offset.checked_add(if judgement { 20600 } else { 15000 })?, y: along })
        }
    }

    fn step(&self) -> Option<u32> {
        if self.across {
            if self.gaps == 0 { Some(4800) } else { self.span.checked_div(self.gaps) }
        } else {
            Some(4000)
        }
    }
}

fn judgement(mark: Mark) -> bool {
    match mark {
        Mark::Person | Mark::Agent => true,
        Mark::Code | Mark::Bar => false,
    }
}

/// Draw for the available width, independently of the measured height.
pub fn draw(kind: DrawingKind, data: &[u8], width: u16, _height: u16, scene: &mut Scene) -> bool {
    if kind != KIND {
        return false;
    }
    let Some(data) = decode(data) else {
        return false;
    };
    paint(&data, width, scene).is_some()
}

fn paint(data: &Decoded<'_>, width: u16, scene: &mut Scene) -> Option<()> {
    let layout = Layout::new(data, width)?;
    let (from, to) = if layout.across {
        (Point { x: 0, y: 72 }, Point { x: width, y: 72 })
    } else {
        let x = u16::try_from(layout.offset.checked_div(100)?.checked_add(178)?).ok()?;
        (Point { x, y: 0 }, Point { x, y: u16::try_from(layout.end.checked_div(100)?).ok()? })
    };
    if !scene.line(from, to, &[WAYS_AUTH]) {
        return None;
    }
    line::draw(data.stops.as_slice(), &layout, scene)?;
    for (index, stop) in data.stops.iter().enumerate() {
        let index = u32::try_from(index).ok()?;
        let end = stop.mark == Mark::Bar && index.checked_add(1)? == data.stops.len();
        let at = layout.position(layout.along(index, end)?, judgement(stop.mark))?;
        marks::draw(*stop, index, end, at, data, &layout, scene)?;
    }
    if layout.across
        && let Some(index) = data.open
    {
        let x = u16::try_from(layout.along(index, false)?.checked_div(100)?).ok()?;
        if !scene.polyline(
            &[Point { x: x.saturating_sub(6), y: 154 }, Point { x, y: 148 }, Point { x: x.checked_add(6)?, y: 154 }],
            &[WAYS_CARET],
        ) {
            return None;
        }
    }
    scene.set_height(layout.height);
    Some(())
}
