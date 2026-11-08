//! The TUI's two themes (temper/docs/design/client/README.md, 6.15).
use crate::look;
use alloc::boxed::Box;
use skein_ui_screen::{Align, Arrange, Border, Glyphs, Mark, Paint, Place, RegionLook, Rgb, StyleLook, Theme};

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
pub const fn palette(scheme: Scheme) -> [(&'static str, Rgb); 7] {
    match scheme {
        Scheme::Dark => [
            ("bg", rgb(0x19, 0x18, 0x16)),
            ("ink", rgb(0xe8, 0xe3, 0xda)),
            ("dim", rgb(0x9a, 0x93, 0x88)),
            ("faint", rgb(0x6a, 0x65, 0x5c)),
            ("accent", rgb(0xe0, 0x8a, 0x68)),
            ("good", rgb(0x8f, 0xc2, 0x8f)),
            ("sel", rgb(0x2a, 0x28, 0x25)),
        ],
        Scheme::Light => [
            ("bg", rgb(0xff, 0xfe, 0xfd)),
            ("ink", rgb(0x34, 0x31, 0x2c)),
            ("dim", rgb(0x7a, 0x73, 0x6a)),
            ("faint", rgb(0xb5, 0xae, 0xa3)),
            ("accent", rgb(0xb8, 0x66, 0x4e)),
            ("good", rgb(0x3f, 0x7d, 0x4e)),
            ("sel", rgb(0xf4, 0xf0, 0xe9)),
        ],
    }
}

// Every token starts with the screen's neutral layout, then names only its differences.
fn style(style: skein_ui::Style, paint: Paint) -> StyleLook {
    StyleLook {
        style,
        paint,
        align: Align::Start,
        band: false,
        label: true,
        mark: None,
        described: None,
        spaced: false,
    }
}
fn region(region: skein_ui::Region, place: Place, arrange: Arrange, pad: u16, border: Option<Border>) -> RegionLook {
    RegionLook { region, place, arrange, pad, border, paint: Paint::NONE }
}
#[must_use]
pub fn theme(scheme: Scheme) -> Theme {
    let [_, _, (_, dim), (_, faint), (_, accent), (_, good), (_, sel)] = palette(scheme);
    let border =
        Border {
            corners: ['╭', '╮', '╰', '╯'], across: '─', down: '│', paint: fore(faint), title: true, sides: true
        };
    let star = Some(Mark { glyph: '✦', paint: fore(accent) });
    let bold = Paint { bold: true, ..Paint::NONE };
    Theme {
        base: Paint::NONE,
        focus: fore(accent),
        placeholder: fore(faint),
        active: Paint { back: Some(sel), ..Paint::NONE },
        regions: Box::new([
            region(look::TOP, Place::Top, Arrange::Lines, 1, Some(border)),
            region(look::BOTTOM, Place::Bottom, Arrange::Spread, 1, None),
            region(look::CHAT_TOP, Place::Top, Arrange::Spread, 1, None),
            region(look::SECTIONS, Place::Body, Arrange::Lines, 1, None),
            region(look::WORKING, Place::Bottom, Arrange::Lines, 0, None),
            region(look::BOX, Place::Bottom, Arrange::Lines, 0, Some(Border { title: false, sides: false, ..border })),
            region(look::LIST, Place::Bottom, Arrange::Lines, 0, None),
        ]),
        styles: Box::new([
            style(look::TITLE, Paint { bold: true, ..fore(accent) }),
            style(look::HINT, fore(dim)),
            style(look::LIVE, fore(good)),
            style(look::WAITING, fore(dim)),
            style(look::DOWN, fore(accent)),
            style(look::REFUSED, fore(accent)),
            style(look::NAME, bold),
            style(look::FAINT, fore(faint)),
            StyleLook { align: Align::End, ..style(look::ADDRESS, fore(dim)) },
            StyleLook { align: Align::End, ..style(look::BYLINE, fore(dim)) },
            style(look::HEAD, bold),
            style(look::SIGNAL, fore(accent)),
            style(look::STAR, fore(accent)),
            StyleLook {
                band: true,
                mark: Some(Mark { glyph: '❯', paint: fore(dim) }),
                ..style(look::SAID, Paint { back: Some(sel), ..Paint::NONE })
            },
            StyleLook { mark: star, ..style(look::AGENT, Paint::NONE) },
            StyleLook { mark: Some(Mark { glyph: '▸', paint: fore(dim) }), ..style(look::RUN, fore(dim)) },
            style(look::RUN_FAILED, fore(accent)),
            StyleLook { mark: star, ..style(look::DOTS, fore(faint)) },
            StyleLook { mark: star, ..style(look::RUNNING, fore(dim)) },
            style(look::END, Paint { reverse: true, ..Paint::NONE }),
            StyleLook { label: false, ..style(look::FIELD, Paint::NONE) },
            style(look::LIST_HEAD, bold),
            StyleLook { described: Some(fore(dim)), ..style(look::OPTION, bold) },
            style(look::NOTE, fore(dim)),
            StyleLook { spaced: true, ..style(look::SPACED, Paint::NONE) },
        ]),
        glyphs: Glyphs { focus: '❯', mask: '•' },
    }
}
