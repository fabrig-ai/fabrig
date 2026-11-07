//! The TUI's two themes (temper/docs/design/client/README.md, 6.15).
use crate::look;
use alloc::boxed::Box;
use skein_ui_screen::{Arrange, Border, Glyphs, Paint, Place, RegionLook, Rgb, StyleLook, Theme};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Dark,
    Light,
}

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}
const fn fore(colour: Rgb) -> Paint {
    Paint { fore: Some(colour), ..Paint::NONE }
}

/// The named palette shared by the terminal and its journey pages (temper/docs/design/client/tui/journeys.md, 6.4).
#[must_use]
pub const fn palette(scheme: Scheme) -> [(&'static str, Rgb); 7] {
    match scheme {
        Scheme::Dark => [
            ("bg", rgb(0x19, 0x18, 0x16)),
            ("ink", rgb(0xe8, 0xe3, 0xda)),
            ("dim", rgb(0x9a, 0x93, 0x88)),
            ("faint", rgb(0x6a, 0x65, 0x5c)),
            ("sel", rgb(0x2a, 0x28, 0x25)),
            ("accent", rgb(0xe0, 0x8a, 0x68)),
            ("good", rgb(0x8f, 0xc2, 0x8f)),
        ],
        Scheme::Light => [
            ("bg", rgb(0xff, 0xfe, 0xfd)),
            ("ink", rgb(0x34, 0x31, 0x2c)),
            ("dim", rgb(0x7a, 0x73, 0x6a)),
            ("faint", rgb(0xb5, 0xae, 0xa3)),
            ("sel", rgb(0xf4, 0xf0, 0xe9)),
            ("accent", rgb(0xb8, 0x66, 0x4e)),
            ("good", rgb(0x3f, 0x7d, 0x4e)),
        ],
    }
}

#[must_use]
pub fn theme(scheme: Scheme) -> Theme {
    let [(_, bg), (_, ink), (_, dim), (_, faint), (_, sel), (_, accent), (_, good)] = palette(scheme);
    Theme {
        base: Paint { fore: Some(ink), back: Some(bg), ..Paint::NONE },
        focus: Paint { back: Some(sel), ..Paint::NONE },
        placeholder: fore(faint),
        regions: Box::new([
            RegionLook {
                region: look::TOP,
                place: Place::Top,
                arrange: Arrange::Lines,
                border: Some(Border {
                    corners: ['╭', '╮', '╰', '╯'],
                    across: '─',
                    down: '│',
                    paint: fore(faint),
                    title: true,
                }),
                pad: 1,
                paint: Paint::NONE,
            },
            RegionLook {
                region: look::BOTTOM,
                place: Place::Bottom,
                arrange: Arrange::Spread,
                border: None,
                pad: 1,
                paint: Paint::NONE,
            },
        ]),
        styles: Box::new([
            StyleLook { style: look::TITLE, paint: Paint { bold: true, ..fore(accent) } },
            StyleLook { style: look::HINT, paint: fore(dim) },
            StyleLook { style: look::LIVE, paint: fore(good) },
            StyleLook { style: look::WAITING, paint: fore(dim) },
            StyleLook { style: look::DOWN, paint: fore(accent) },
            StyleLook { style: look::REFUSED, paint: fore(accent) },
        ]),
        glyphs: Glyphs { focus: '❯', mask: '•' },
    }
}
