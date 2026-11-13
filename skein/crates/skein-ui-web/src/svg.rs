//! Scenes as SVG, with all visual choices in the stylesheet (ui/web/README.md, 6).

use skein_lib::usize_of;
use skein_ui::{
    Style,
    look::Names,
    scene::{Anchor, Mark, Scene},
};

use crate::dom::Element;

/// A refused scene is an empty fragment, so the page keeps its fallback.
/// `ids` is a stem unique in the document; a fade's gradient is `<ids>-<mark index>` (ui/web/README.md, 6).
#[must_use]
pub fn draw(scene: &Scene, names: &[Names<'_>], width: u16, height: u16, ids: &str) -> Element {
    if scene.refused() {
        return Element::new("");
    }
    let width = scene.width().unwrap_or(width);
    let height = scene.height().unwrap_or(height);
    let mut svg = Element::new("svg");
    svg.set("aria-hidden", "true");
    svg.set("width", &width.to_string());
    svg.set("height", &height.to_string());
    svg.set("viewBox", &format!("0 0 {width} {height}"));
    let mut defs = Element::new("defs");
    for index in 0..scene.marks().len() {
        let index = u32::try_from(index).expect("scene mark index fits u32");
        if let Some(fade) = scene.mark_fade(index) {
            let mut gradient = Element::new("linearGradient");
            gradient.set("id", &format!("{ids}-{index}"));
            gradient.set("gradientUnits", "userSpaceOnUse");
            numbers(&mut gradient, &[("x1", fade.from.x), ("y1", fade.from.y), ("x2", fade.to.x), ("y2", fade.to.y)]);
            for stop in fade.stops() {
                let mut element = Element::new("stop");
                element.set("offset", &format!("{}.{:03}", stop.offset / 1000, stop.offset % 1000));
                classes(&mut element, stop.styles, names);
                gradient.append(element);
            }
            defs.append(gradient);
        }
    }
    if !defs.children.is_empty() {
        svg.append(defs);
    }
    let mut index = 0;
    marks(scene, names, &mut index, scene.marks().len(), ids, &mut svg);
    svg
}

fn marks(scene: &Scene, names: &[Names<'_>], index: &mut usize, end: usize, ids: &str, parent: &mut Element) {
    while *index < end {
        let position = *index;
        let mark = scene.marks()[position];
        let number = u32::try_from(position).expect("scene mark index fits u32");
        *index += 1;
        let mut element = mark_element(mark, scene.data(number));
        classes(&mut element, scene.styles(number), names);
        if let Some(act) = scene.mark_act(number) {
            let mut hex = String::new();
            for byte in act {
                use std::fmt::Write;
                write!(hex, "{byte:02x}").expect("writing a string succeeds");
            }
            element.set("data-act", &hex);
        }
        if scene.mark_fade(number).is_some() {
            element.set("stroke", &format!("url(#{ids}-{number})"));
            element.set("data-fade", "");
        }
        if let Some(hint) = scene.mark_hint(number) {
            let mut title = Element::new("title");
            title.text(hint);
            element.append(title);
        }
        match mark {
            Mark::Group { marks: count, .. } => {
                let group_end = (*index + usize_of(count)).min(end);
                marks(scene, names, index, group_end, ids, &mut element);
            }
            Mark::Text { .. } => element.text(scene.data(number)),
            Mark::Line { .. }
            | Mark::Polyline { .. }
            | Mark::Path { .. }
            | Mark::Rectangle { .. }
            | Mark::Circle { .. } => {}
        }
        parent.append(element);
    }
}

fn classes(element: &mut Element, styles: &[Style], names: &[Names<'_>]) {
    let classes: Vec<_> = styles
        .iter()
        .filter_map(|style| names.iter().find_map(|owner| owner.style(*style)))
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect();
    if !classes.is_empty() {
        element.set("class", &classes.join(" "));
    }
}

fn mark_element(mark: Mark, data: &[u8]) -> Element {
    match mark {
        Mark::Group { offset, .. } => {
            let mut e = Element::new("g");
            e.set("transform", &format!("translate({} {})", offset.x, offset.y));
            e
        }
        Mark::Line { from, to } => {
            let mut e = Element::new("line");
            numbers(&mut e, &[("x1", from.x), ("y1", from.y), ("x2", to.x), ("y2", to.y)]);
            e
        }
        Mark::Polyline { .. } => {
            let mut e = Element::new("polyline");
            let points: Vec<_> = data
                .as_chunks::<4>()
                .0
                .iter()
                .map(|&[x_low, x_high, y_low, y_high]| {
                    format!("{},{}", u16::from_le_bytes([x_low, x_high]), u16::from_le_bytes([y_low, y_high]))
                })
                .collect();
            e.set("points", &points.join(" "));
            e
        }
        Mark::Path { .. } => {
            let mut e = Element::new("path");
            e.set("d", &path(data));
            e
        }
        Mark::Rectangle { corner, size, radius } => {
            let mut e = Element::new("rect");
            numbers(&mut e, &[("x", corner.x), ("y", corner.y), ("width", size.x), ("height", size.y), ("rx", radius)]);
            e
        }
        Mark::Circle { centre, radius } => {
            let mut e = Element::new("circle");
            numbers(&mut e, &[("cx", centre.x), ("cy", centre.y), ("r", radius)]);
            e
        }
        Mark::Text { at, anchor } => {
            let mut e = Element::new("text");
            numbers(&mut e, &[("x", at.x), ("y", at.y)]);
            e.set(
                "text-anchor",
                match anchor {
                    Anchor::Start => "start",
                    Anchor::Middle => "middle",
                    Anchor::End => "end",
                },
            );
            e
        }
    }
}

fn numbers(e: &mut Element, values: &[(&str, u16)]) {
    for (name, value) in values {
        e.set(name, &value.to_string());
    }
}

fn path(mut data: &[u8]) -> String {
    let mut commands = Vec::new();
    while let Some((&command, rest)) = data.split_first() {
        let points = match command {
            b'M' | b'L' => 1,
            b'Q' => 2,
            b'C' => 3,
            b'Z' => 0,
            _ => unreachable!("scene path command"),
        };
        let (coordinates, rest) = rest.split_at(points * 4);
        let mut text = char::from(command).to_string();
        for &[x_low, x_high, y_low, y_high] in coordinates.as_chunks::<4>().0 {
            use std::fmt::Write;
            write!(text, " {} {}", u16::from_le_bytes([x_low, x_high]), u16::from_le_bytes([y_low, y_high]))
                .expect("writing a string succeeds");
        }
        commands.push(text);
        data = rest;
    }
    commands.join(" ")
}
