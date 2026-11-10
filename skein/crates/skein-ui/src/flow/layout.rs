//! Lays a flow out as a scene (skein/docs/design/ui/scenes.md, 5).

use super::{ARROW, BOX, CAPTION, Direction, KIND, NAME, Node, Nodes, STRONG, decode};
use crate::scene::{Anchor, Command, Point, Scene};
use crate::{DrawingKind, Style};

/// Draw a flow for its width, independently of its measured height.
pub fn draw(kind: DrawingKind, data: &[u8], width: u16, _height: u16, scene: &mut Scene) -> bool {
    if kind != KIND {
        return false;
    }
    let Some((direction, nodes)) = decode(data) else {
        return false;
    };
    let Some(layout) = fit(direction, nodes.clone(), u32::from(width)) else {
        return false;
    };
    paint(&layout, nodes, scene).is_some()
}

struct Layout {
    direction: Direction,
    width: u32,
    height: u16,
    nodes: u32,
}

fn text_width(bytes: &[u8], twice: u32) -> Option<u32> {
    let mut count = 0_u32;
    for chunk in bytes.utf8_chunks() {
        count = count.checked_add(u32::try_from(chunk.valid().chars().count()).ok()?)?;
        if !chunk.invalid().is_empty() {
            count = count.checked_add(1)?;
        }
    }
    count.checked_mul(twice)?.checked_add(1)?.checked_div(2)
}

fn fit(direction: Direction, nodes: Nodes<'_>, available: u32) -> Option<Layout> {
    let mut count = 0_u32;
    let mut name = 0_u32;
    let mut caption = 0_u32;
    let mut has_caption = false;
    for node in nodes {
        count = count.checked_add(1)?;
        name = name.max(text_width(node.name, 15)?);
        if let Some(bytes) = node.caption {
            has_caption = true;
            caption = caption.max(text_width(bytes, 13)?);
        }
    }
    let gaps = count.checked_sub(1)?;
    match direction {
        Direction::LeftToRight => {
            let width = 120.max(name.checked_add(24)?).max(caption.checked_add(8)?);
            let total = count.checked_mul(width)?.checked_add(gaps.checked_mul(28)?)?.checked_add(2)?;
            if total <= available {
                return Some(Layout { direction, width, height: if has_caption { 74 } else { 46 }, nodes: count });
            }
        }
        Direction::TopToBottom => {}
    }
    let width = 160.max(name.checked_add(24)?);
    let mut total = width.checked_add(2)?;
    if has_caption {
        total = total.checked_add(16)?.checked_add(caption)?;
    }
    let height = count.checked_mul(40)?.checked_add(gaps.checked_mul(22)?)?.checked_add(2)?;
    if total > available {
        return None;
    }
    Some(Layout { direction: Direction::TopToBottom, width, height: u16::try_from(height).ok()?, nodes: count })
}

fn point(x: u32, y: u32) -> Option<Point> {
    Some(Point { x: u16::try_from(x).ok()?, y: u16::try_from(y).ok()? })
}

fn paint(layout: &Layout, nodes: Nodes<'_>, scene: &mut Scene) -> Option<()> {
    scene.set_height(layout.height);
    for (index, node) in nodes.enumerate() {
        let index = u32::try_from(index).ok()?;
        let last = index.checked_add(1)? == layout.nodes;
        match layout.direction {
            Direction::LeftToRight => horizontal(layout.width, index, node, last, scene)?,
            Direction::TopToBottom => vertical(layout.width, index, node, last, scene)?,
        }
    }
    Some(())
}

fn box_and_name(scene: &mut Scene, node: Node<'_>, corner: Point, size: Point, name: Point) -> Option<()> {
    let box_styles: &[Style] = if node.strong { &[BOX, STRONG] } else { &[BOX] };
    if !scene.rect(corner, size, 8, box_styles) {
        return None;
    }
    let name_styles: &[Style] = if node.strong { &[NAME, STRONG] } else { &[NAME] };
    if !scene.text(name, Anchor::Middle, node.name, name_styles) {
        return None;
    }
    Some(())
}

fn horizontal(width: u32, index: u32, node: Node<'_>, last: bool, scene: &mut Scene) -> Option<()> {
    let left = index.checked_mul(width.checked_add(28)?)?.checked_add(1)?;
    let centre = left.checked_add(width.checked_div(2)?)?;
    box_and_name(scene, node, point(left, 1)?, point(width, 44)?, point(centre, 28)?)?;
    if let Some(caption) = node.caption
        && !scene.text(point(centre, 67)?, Anchor::Middle, caption, &[CAPTION])
    {
        return None;
    }
    if !last {
        let start = left.checked_add(width)?.checked_add(4)?;
        let tip = start.checked_add(20)?;
        let back = tip.checked_sub(6)?;
        if !scene.path(
            &[
                Command::Move(point(start, 23)?),
                Command::Line(point(tip, 23)?),
                Command::Move(point(back, 18)?),
                Command::Line(point(tip, 23)?),
                Command::Line(point(back, 28)?),
            ],
            &[ARROW],
        ) {
            return None;
        }
    }
    Some(())
}

fn vertical(width: u32, index: u32, node: Node<'_>, last: bool, scene: &mut Scene) -> Option<()> {
    let top = index.checked_mul(62)?.checked_add(1)?;
    let centre = width.checked_div(2)?.checked_add(1)?;
    let baseline = top.checked_add(25)?;
    box_and_name(scene, node, point(1, top)?, point(width, 40)?, point(centre, baseline)?)?;
    if let Some(caption) = node.caption
        && !scene.text(point(width.checked_add(17)?, baseline)?, Anchor::Start, caption, &[CAPTION])
    {
        return None;
    }
    if !last {
        let start = top.checked_add(44)?;
        let tip = start.checked_add(14)?;
        let back = tip.checked_sub(6)?;
        if !scene.path(
            &[
                Command::Move(point(centre, start)?),
                Command::Line(point(centre, tip)?),
                Command::Move(point(centre.checked_sub(5)?, back)?),
                Command::Line(point(centre, tip)?),
                Command::Line(point(centre.checked_add(5)?, back)?),
            ],
            &[ARROW],
        ) {
            return None;
        }
    }
    Some(())
}
