//! Styles and border boxes at the three viewports (temper/docs/design/client/journey-writer.md, 6.2, step 7).

use std::collections::BTreeMap;

use skein_browser::boundary::{Element, Property, Rect};
use skein_lib::Token;

use crate::host::browser::Command;

use super::{browser::Browser, html::Tag, report::PageReport};

const WIDTHS: [u32; 3] = [1440, 1000, 420];
const HEIGHT: u32 = 900;

pub(super) struct Matched<'a> {
    pub(super) requested: &'a [Element],
    pub(super) landed: &'a [Element],
    pub(super) tags: &'a [Tag<'a>],
    pub(super) pairs: &'a [Option<usize>],
    pub(super) touched: &'a [bool],
}

impl Matched<'_> {
    pub(super) fn compare(&self, browser: &mut Browser<'_>, opened: [Token; 2]) -> Result<PageReport, String> {
        let mut report = PageReport::default();
        let touched = self.touched.iter().enumerate().filter(|&(_, &changed)| changed);
        if touched.clone().any(|(index, _)| self.pairs[index].is_some()) {
            for width in WIDTHS {
                for page in opened {
                    browser.done(Command::Viewport { page, width, height: HEIGHT })?;
                }
                let width = width.to_string();
                for (index, _) in touched.clone() {
                    if let Some(other) = self.pairs[index] {
                        self.measure(browser, opened, (index, other), &width, &mut report)?;
                    }
                }
            }
        }
        for (index, _) in touched {
            if self.pairs[index].is_none() {
                report.difference(&browser.item, "-", &self.label(index), b"pair", (b"present", b"none"));
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
        width: &str,
        report: &mut PageReport,
    ) -> Result<(), String> {
        let left = self.requested[from].node;
        let right = self.landed[to].node;
        let left_style = browser.style(request, left)?;
        let right_style = browser.style(print, right)?;
        let left_box = browser.rect(request, left)?;
        let right_box = browser.rect(print, right)?;
        let mut values = BTreeMap::new();
        insert(&mut values, &left_style, 0);
        insert(&mut values, &right_style, 1);
        let boxes;
        if different_boxes(left_box, right_box) {
            boxes = [box_value(left_box), box_value(right_box)];
            values.insert(b"box", [Some(boxes[0].as_bytes()), Some(boxes[1].as_bytes())]);
        }
        let label = self.label(from);
        for (property, [left, right]) in values {
            if left != right {
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

fn insert<'a>(map: &mut BTreeMap<&'a [u8], [Option<&'a [u8]>; 2]>, properties: &'a [Property], side: usize) {
    for property in properties {
        if !property.name.starts_with(b"--") {
            map.entry(&property.name).or_default()[side] = Some(&property.value);
        }
    }
}

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
