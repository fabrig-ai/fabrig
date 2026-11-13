//! Whole judgement curves, each actor's fade (temper/docs/design/client/pages/chat/conversation.md, 3).

use super::{Layout, Position, judgement};
use crate::{
    look::{
        WAYS_BASE, WAYS_FADE_AGENT, WAYS_FADE_AGENT_FAINT, WAYS_FADE_BASE, WAYS_FADE_PERSON, WAYS_FADE_PERSON_FAINT,
        WAYS_RISE_AGENT, WAYS_RISE_PERSON,
    },
    paths::{Mark, STOPS, Stop},
};
use skein_lib::List;
use skein_ui::{
    Style,
    scene::{Command, FadeStop, Point, Scene},
};

pub(super) fn draw(stops: &[Stop<'_>], layout: &Layout, scene: &mut Scene) -> Option<()> {
    // A join has at most two whole curves, each a move and a cubic.
    let capacity = STOPS.checked_mul(4)?;
    let mut base = List::with_capacity(STOPS.checked_mul(2)?);
    let mut person = List::with_capacity(capacity);
    let mut agent = List::with_capacity(capacity);
    let reach = layout.step()?.checked_mul(42)?.checked_div(100)?;
    let mut previous = layout.position(layout.along(0, false)?, judgement(stops.first()?.mark))?;
    for (index, stop) in stops.iter().enumerate().skip(1) {
        let index = u32::try_from(index).ok()?;
        let end = stop.mark == Mark::Bar && index.checked_add(1)? == u32::try_from(stops.len()).ok()?;
        let along = layout.along(index, end)?;
        let next = layout.position(along, judgement(stop.mark))?;
        let before = stops.get(usize::try_from(index.checked_sub(1)?).ok()?)?.mark;
        if judgement(before) {
            let a = layout.along(index.checked_sub(1)?, false)?;
            let foot = layout.position(a.checked_add(reach)?, false)?;
            curve(previous, foot, layout.across, rise(before, &mut person, &mut agent)?)?;
            previous = foot;
        }
        let foot = if judgement(stop.mark) { layout.position(along.checked_sub(reach)?, false)? } else { next };
        base.push(Command::Move(previous.point()?)).ok()?;
        base.push(Command::Line(foot.point()?)).ok()?;
        if judgement(stop.mark) {
            curve(foot, next, layout.across, rise(stop.mark, &mut person, &mut agent)?)?;
        }
        previous = next;
    }
    if !scene.path(base.as_slice(), &[WAYS_BASE]) {
        return None;
    }
    faded(&person, WAYS_RISE_PERSON, WAYS_FADE_PERSON, WAYS_FADE_PERSON_FAINT, layout, scene)?;
    faded(&agent, WAYS_RISE_AGENT, WAYS_FADE_AGENT, WAYS_FADE_AGENT_FAINT, layout, scene)
}

fn rise<'a>(mark: Mark, person: &'a mut List<Command>, agent: &'a mut List<Command>) -> Option<&'a mut List<Command>> {
    match mark {
        Mark::Person => Some(person),
        Mark::Agent => Some(agent),
        Mark::Code | Mark::Bar => None,
    }
}

fn faded(
    commands: &List<Command>,
    style: Style,
    strong: Style,
    faint: Style,
    layout: &Layout,
    scene: &mut Scene,
) -> Option<()> {
    if commands.is_empty() {
        return Some(());
    }
    let (from, to) = if layout.across {
        (Point { x: 0, y: 40 }, Point { x: 0, y: 106 })
    } else {
        let offset = u16::try_from(layout.offset.checked_div(100)?).ok()?;
        (Point { x: offset.checked_add(206)?, y: 0 }, Point { x: offset.checked_add(150)?, y: 0 })
    };
    if !scene.path(commands.as_slice(), &[style])
        || !scene.fade(
            from,
            to,
            &[
                FadeStop { offset: 0, styles: &[strong] },
                FadeStop { offset: 500, styles: &[faint] },
                FadeStop { offset: 1000, styles: &[WAYS_FADE_BASE] },
            ],
        )
    {
        return None;
    }
    Some(())
}

fn curve(start: Position, end: Position, across: bool, path: &mut List<Command>) -> Option<()> {
    let (first_control, second_control) = if across {
        let middle = start.x.checked_add(end.x)?.checked_div(2)?;
        (Position { x: middle, y: start.y }, Position { x: middle, y: end.y })
    } else {
        let middle = start.y.checked_add(end.y)?.checked_div(2)?;
        (Position { x: start.x, y: middle }, Position { x: end.x, y: middle })
    };
    path.push(Command::Move(start.point()?)).ok()?;
    path.push(Command::Cubic(first_control.point()?, second_control.point()?, end.point()?)).ok()?;
    Some(())
}
