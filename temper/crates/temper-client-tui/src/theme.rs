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

#[must_use]
pub fn theme(scheme: Scheme) -> Theme {
    let [bg, ink, dim, faint, sel, accent, good] = match scheme {
        Scheme::Dark => [
            rgb(0x19, 0x18, 0x16),
            rgb(0xe8, 0xe3, 0xda),
            rgb(0x9a, 0x93, 0x88),
            rgb(0x6a, 0x65, 0x5c),
            rgb(0x2a, 0x28, 0x25),
            rgb(0xe0, 0x8a, 0x68),
            rgb(0x8f, 0xc2, 0x8f),
        ],
        Scheme::Light => [
            rgb(0xff, 0xfe, 0xfd),
            rgb(0x34, 0x31, 0x2c),
            rgb(0x7a, 0x73, 0x6a),
            rgb(0xb5, 0xae, 0xa3),
            rgb(0xf4, 0xf0, 0xe9),
            rgb(0xb8, 0x66, 0x4e),
            rgb(0x3f, 0x7d, 0x4e),
        ],
    };
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
