//! Two tones of the judgement curves (temper/docs/design/client/README.md, 5.16).

use super::{Layout, Position, judgement};
use crate::{
    look::{WAYS_BASE, WAYS_RISE},
    paths::{Mark, STOPS, Stop},
};
use skein_lib::List;
use skein_ui::scene::{Command, Scene};

pub(super) fn draw(stops: &[Stop<'_>], layout: &Layout, scene: &mut Scene) -> Option<()> {
    // Each join: two split curves (move and cubic per tone) and a base segment.
    let capacity = STOPS.checked_mul(10)?;
    let mut base = List::with_capacity(capacity);
    let mut rise = List::with_capacity(capacity);
    let reach = layout.step()?.checked_mul(42)?.checked_div(100)?;
    let mut previous = layout.position(layout.along(0, false)?, judgement(stops.first()?.mark))?;
    for (index, stop) in stops.iter().enumerate().skip(1) {
        let index = u32::try_from(index).ok()?;
        let end = stop.mark == Mark::Bar && index.checked_add(1)? == u32::try_from(stops.len()).ok()?;
        let along = layout.along(index, end)?;
        let next = layout.position(along, judgement(stop.mark))?;
        let was_upper = judgement(stops.get(usize::try_from(index.checked_sub(1)?).ok()?)?.mark);
        if was_upper {
            let a = layout.along(index.checked_sub(1)?, false)?;
            let foot = layout.position(a.checked_add(reach)?, false)?;
            curve(previous, foot, layout.across, true, &mut base, &mut rise)?;
            previous = foot;
        }
        let foot = if judgement(stop.mark) { layout.position(along.checked_sub(reach)?, false)? } else { next };
        base.push(Command::Move(previous.point()?)).ok()?;
        base.push(Command::Line(foot.point()?)).ok()?;
        if judgement(stop.mark) {
            curve(foot, next, layout.across, false, &mut base, &mut rise)?;
        }
        previous = next;
    }
    if !scene.path(base.as_slice(), &[WAYS_BASE]) || !scene.path(rise.as_slice(), &[WAYS_RISE]) {
        return None;
    }
    Some(())
}

// De Casteljau subdivision at the dotted line. Wide curves cross at
// t≈.489898 (falling) or its complement; the vertical curve at exactly .5.
// Hundredths are retained, with the join fixed on the boundary itself.
fn curve(
    start: Position,
    end: Position,
    across: bool,
    falling: bool,
    base: &mut List<Command>,
    rise: &mut List<Command>,
) -> Option<()> {
    let (first_control, second_control) = if across {
        let middle = start.x.checked_add(end.x)?.checked_div(2)?;
        (Position { x: middle, y: start.y }, Position { x: middle, y: end.y })
    } else {
        let middle = start.y.checked_add(end.y)?.checked_div(2)?;
        (Position { x: start.x, y: middle }, Position { x: end.x, y: middle })
    };
    let fraction = if across { if falling { 489_898 } else { 510_102 } } else { 500_000 };
    let near_start = mix(start, first_control, fraction)?;
    let middle = mix(first_control, second_control, fraction)?;
    let near_end = mix(second_control, end, fraction)?;
    let left_control = mix(near_start, middle, fraction)?;
    let right_control = mix(middle, near_end, fraction)?;
    let mut join = mix(left_control, right_control, fraction)?;
    if across {
        join.y = 7200;
    } else {
        join.x = start.x.checked_add(end.x)?.checked_div(2)?;
    }
    let (leading, trailing) = if falling { (rise, base) } else { (base, rise) };
    leading.push(Command::Move(start.point()?)).ok()?;
    leading.push(Command::Cubic(near_start.point()?, left_control.point()?, join.point()?)).ok()?;
    trailing.push(Command::Move(join.point()?)).ok()?;
    trailing.push(Command::Cubic(right_control.point()?, near_end.point()?, end.point()?)).ok()?;
    Some(())
}

fn mix(a: Position, b: Position, t: u64) -> Option<Position> {
    Some(Position { x: coordinate(a.x, b.x, t)?, y: coordinate(a.y, b.y, t)? })
}
fn coordinate(a: u32, b: u32, t: u64) -> Option<u32> {
    let left = u64::from(a).checked_mul(1_000_000_u64.checked_sub(t)?)?;
    let right = u64::from(b).checked_mul(t)?;
    u32::try_from(left.checked_add(right)?.checked_div(1_000_000)?).ok()
}
