//! The TUI's two themes (temper/docs/design/client/README.md, 6.15).
use crate::look;
use alloc::boxed::Box;
use skein_ui_screen::{Arrange, Border, Glyphs, Paint, Place, RegionLook, Rgb, StyleLook, Theme};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Dark,
    Light,
}

impl Scheme {
    /// The terminal's background luma (temper/docs/design/client/README.md, 6.15).
    #[must_use]
    pub const fn of_background(background: Rgb) -> Self {
        let red = u32::from_be_bytes([0, 0, 0, background.red]);
        let green = u32::from_be_bytes([0, 0, 0, background.green]);
        let blue = u32::from_be_bytes([0, 0, 0, background.blue]);
        // Eight-bit components bound the sum by 2_550_000.
        let luma = red
            .saturating_mul(2126)
            .saturating_add(green.saturating_mul(7152))
            .saturating_add(blue.saturating_mul(722));
        if luma > 1_275_000 { Self::Light } else { Self::Dark }
    }
}

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}
const fn fore(colour: Rgb) -> Paint {
    Paint { fore: Some(colour), ..Paint::NONE }
}

/// The named palette shared by the terminal and its journey pages (temper/docs/design/client/tui/journeys.md, 6.4).
#[must_use]
pub const fn palette(scheme: Scheme) -> [(&'static str, Rgb); 6] {
    match scheme {
        Scheme::Dark => [
            ("bg", rgb(0x19, 0x18, 0x16)),
            ("ink", rgb(0xe8, 0xe3, 0xda)),
            ("dim", rgb(0x9a, 0x93, 0x88)),
            ("faint", rgb(0x6a, 0x65, 0x5c)),
            ("accent", rgb(0xe0, 0x8a, 0x68)),
            ("good", rgb(0x8f, 0xc2, 0x8f)),
        ],
        Scheme::Light => [
            ("bg", rgb(0xff, 0xfe, 0xfd)),
            ("ink", rgb(0x34, 0x31, 0x2c)),
            ("dim", rgb(0x7a, 0x73, 0x6a)),
            ("faint", rgb(0xb5, 0xae, 0xa3)),
            ("accent", rgb(0xb8, 0x66, 0x4e)),
            ("good", rgb(0x3f, 0x7d, 0x4e)),
        ],
    }
}

#[must_use]
pub fn theme(scheme: Scheme) -> Theme {
    let [_, _, (_, dim), (_, faint), (_, accent), (_, good)] = palette(scheme);
    Theme {
        base: Paint::NONE,
        focus: fore(accent),
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
