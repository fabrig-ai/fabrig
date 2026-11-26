//! Styles and border boxes at the three viewports (temper/docs/design/client/journey-writer.md, 6.2, step 7).

use std::collections::BTreeMap;

use skein_browser::boundary::{Element, Property, Rect};
use skein_lib::Token;

use crate::host::browser::Command;

use super::{browser::Browser, html::Tag, report::PageReport, rules::Names};

const WIDTHS: [u32; 3] = [1440, 1000, 420];
const HEIGHT: u32 = 900;

#[derive(Clone, Copy)]
pub(super) enum Properties<'a> {
    Journey,
    Declared(&'a Names<'a>),
}

pub(super) struct Matched<'a> {
    pub(super) requested: &'a [Element],
    pub(super) landed: &'a [Element],
    pub(super) tags: &'a [Tag<'a>],
    pub(super) pairs: &'a [Option<usize>],
    pub(super) touched: &'a [Option<Properties<'a>>],
}

impl Matched<'_> {
    pub(super) fn compare(&self, browser: &mut Browser<'_>, opened: [Token; 2]) -> Result<Option<PageReport>, String> {
        let mut report = None;
        let touched = self
            .touched
            .iter()
            .enumerate()
            .filter_map(|(index, &properties)| properties.map(|properties| (index, properties)));
        let paired = touched
            .clone()
            .filter_map(|(index, properties)| self.pairs[index].map(|other| ((index, other), properties)));
        if paired.clone().next().is_some() {
            let report = report.insert(PageReport::default());
            for width in WIDTHS {
                for page in opened {
                    browser.done(Command::Viewport { page, width, height: HEIGHT })?;
                }
                let width = width.to_string();
                for (pair, properties) in paired.clone() {
                    self.measure(browser, opened, pair, properties, &width, report)?;
                }
            }
        }
        for (index, properties) in touched {
            if self.pairs[index].is_none() && matches!(properties, Properties::Journey) {
                report.get_or_insert_with(PageReport::default).difference(
                    &browser.item,
                    "-",
                    &self.label(index),
                    b"pair",
                    (b"present", b"none"),
                );
            }
        }
        Ok(report)
    }

    fn label(&self, index: usize) -> String {
        let tag = &self.tags[index];
        let name = String::from_utf8_lossy(&self.requested[index].name);
        let mut label = format!("{} {name}", tag.line);
        for class in tag.classes.split_ascii_whitespace() {
            label.push('.');
            label.push_str(class);
        }
        label
    }

    fn measure(
        &self,
        browser: &mut Browser<'_>,
        [request, print]: [Token; 2],
        (from, to): (usize, usize),
        properties: Properties<'_>,
        width: &str,
        report: &mut PageReport,
    ) -> Result<(), String> {
        let left = self.requested[from].node;
        let right = self.landed[to].node;
        let left_style = browser.style(request, left)?;
        let right_style = browser.style(print, right)?;
        let mut values = Values::new();
        insert(&mut values, &left_style, 0);
        insert(&mut values, &right_style, 1);
        let boxes;
        if matches!(properties, Properties::Journey) {
            let left_box = browser.rect(request, left)?;
            let right_box = browser.rect(print, right)?;
            if different_boxes(left_box, right_box) {
                boxes = [box_value(left_box), box_value(right_box)];
                values.insert(b"box", [Some(boxes[0].as_bytes()), Some(boxes[1].as_bytes())]);
            }
        }
        let label = self.label(from);
        for (&property, &[left, right]) in &values {
            if left != right && properties.includes(property, &values) {
                report.difference(
                    &browser.item,
                    width,
                    &label,
                    property,
                    (left.unwrap_or_default(), right.unwrap_or_default()),
                );
            }
        }
        Ok(())
    }
}

type Values<'a> = BTreeMap<&'a [u8], [Option<&'a [u8]>; 2]>;

fn insert<'a>(map: &mut Values<'a>, properties: &'a [Property], side: usize) {
    for property in properties {
        map.entry(&property.name).or_default()[side] = Some(&property.value);
    }
}

impl Properties<'_> {
    fn includes(self, property: &[u8], computed: &Values<'_>) -> bool {
        match self {
            Self::Journey => {
                !property.starts_with(b"--")
                    && !matches!(
                        property,
                        b"width"
                            | b"height"
                            | b"inline-size"
                            | b"block-size"
                            | b"transform-origin"
                            | b"perspective-origin"
                    )
            }
            Self::Declared(names) => names.iter().any(|name| {
                let name = name.as_bytes();
                if computed.contains_key(name) {
                    property == name
                } else if let Some((_, longhands)) = SHORTHANDS.iter().find(|(short, _)| *short == name) {
                    longhands.contains(&property)
                } else {
                    property.strip_prefix(name).is_some_and(|tail| tail.starts_with(b"-"))
                }
            }),
        }
    }
}

// A shorthand's longhands need not share its prefix, and a property
// sharing its prefix need not be among its longhands.
const SHORTHANDS: &[(&[u8], &[&[u8]])] = &[
    (b"padding", &[b"padding-top", b"padding-right", b"padding-bottom", b"padding-left"]),
    (b"margin", &[b"margin-top", b"margin-right", b"margin-bottom", b"margin-left"]),
    (b"inset", &[b"top", b"right", b"bottom", b"left"]),
    (b"gap", &[b"row-gap", b"column-gap"]),
    (
        b"border",
        &[
            b"border-top-width",
            b"border-right-width",
            b"border-bottom-width",
            b"border-left-width",
            b"border-top-style",
            b"border-right-style",
            b"border-bottom-style",
            b"border-left-style",
            b"border-top-color",
            b"border-right-color",
            b"border-bottom-color",
            b"border-left-color",
        ],
    ),
    (b"border-width", &[b"border-top-width", b"border-right-width", b"border-bottom-width", b"border-left-width"]),
    (b"border-style", &[b"border-top-style", b"border-right-style", b"border-bottom-style", b"border-left-style"]),
    (b"border-color", &[b"border-top-color", b"border-right-color", b"border-bottom-color", b"border-left-color"]),
    (
        b"border-radius",
        &[
            b"border-top-left-radius",
            b"border-top-right-radius",
            b"border-bottom-right-radius",
            b"border-bottom-left-radius",
        ],
    ),
    (b"flex", &[b"flex-grow", b"flex-shrink", b"flex-basis"]),
    (b"flex-flow", &[b"flex-direction", b"flex-wrap"]),
    (b"place-content", &[b"align-content", b"justify-content"]),
    (b"place-items", &[b"align-items", b"justify-items"]),
    (b"place-self", &[b"align-self", b"justify-self"]),
];

fn different_boxes(left: Option<Rect>, right: Option<Rect>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.x.abs_diff(right.x) > 1
                || left.y.abs_diff(right.y) > 1
                || left.width.abs_diff(right.width) > 1
                || left.height.abs_diff(right.height) > 1
        }
        _ => left.is_some() != right.is_some(),
    }
}

fn box_value(rect: Option<Rect>) -> String {
    rect.map_or_else(|| "none".into(), |Rect { x, y, width, height }| format!("{x},{y} {width}x{height}"))
}
