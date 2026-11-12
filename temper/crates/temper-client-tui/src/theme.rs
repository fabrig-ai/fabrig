//! The TUI's two themes (temper/docs/design/client/tui/theme.md and temper/docs/design/client/tui/box.md).
use crate::look;
use alloc::boxed::Box;
use skein_ui_screen::{Align, Arrange, Border, Glyphs, Mark, Paint, Place, RegionLook, Rgb, StyleLook, Theme};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Dark,
    Light,
}

impl Scheme {
    /// The terminal's background luma (temper/docs/design/client/tui/theme.md).
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

/// The theme kept in the TUI's small store (temper/docs/design/client/tui/theme.md).
pub const THEME_KEY: &[u8] = b"theme";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeChoice {
    Auto,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [ThemeChoice::Auto, ThemeChoice::Light, ThemeChoice::Dark];

    #[must_use]
    pub const fn print(self) -> &'static [u8] {
        match self {
            Self::Auto => b"auto",
            Self::Light => b"light",
            Self::Dark => b"dark",
        }
    }

    #[must_use]
    pub fn read(bytes: &[u8]) -> ThemeChoice {
        if bytes == b"light" {
            Self::Light
        } else if bytes == b"dark" {
            Self::Dark
        } else {
            Self::Auto
        }
    }

    #[must_use]
    pub const fn scheme(self, terminal: Option<Scheme>) -> Scheme {
        match self {
            Self::Auto => match terminal {
                Some(scheme) => scheme,
                None => Scheme::Dark,
            },
            Self::Light => Scheme::Light,
            Self::Dark => Scheme::Dark,
        }
    }
}

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}
const fn fore(colour: Rgb) -> Paint {
    Paint { fore: Some(colour), ..Paint::NONE }
}

pub const BAND_STEP: u8 = 20;

/// Move each channel toward the ink, saturating (temper/docs/design/client/tui/theme.md).
#[must_use]
pub const fn band(background: Rgb) -> Rgb {
    match Scheme::of_background(background) {
        Scheme::Dark => Rgb {
            red: background.red.saturating_add(BAND_STEP),
            green: background.green.saturating_add(BAND_STEP),
            blue: background.blue.saturating_add(BAND_STEP),
        },
        Scheme::Light => Rgb {
            red: background.red.saturating_sub(BAND_STEP),
            green: background.green.saturating_sub(BAND_STEP),
            blue: background.blue.saturating_sub(BAND_STEP),
        },
    }
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
        active_name: None,
        focused_value: None,
        spaced: false,
    }
}
fn region(region: skein_ui::Region, place: Place, arrange: Arrange, pad: u16, border: Option<Border>) -> RegionLook {
    RegionLook { region, place, arrange, pad, border, paint: Paint::NONE }
}
#[must_use]
#[expect(clippy::manual_map, reason = "step code spells out the optional background")]
pub fn theme(scheme: Scheme, background: Option<Rgb>) -> Theme {
    let [_, _, (_, dim), (_, faint), (_, accent), (_, good)] = palette(scheme);
    let back = match background {
        Some(background) => Some(band(background)),
        None => None,
    };
    let border =
        Border {
            corners: ['╭', '╮', '╰', '╯'], across: '─', down: '│', paint: fore(faint), title: true, sides: true
        };
    Theme {
        base: Paint::NONE,
        focus: fore(accent),
        placeholder: fore(faint),
        active: Paint { back, ..Paint::NONE },
        regions: Box::new([
            region(look::TOP, Place::Top, Arrange::Lines, 1, Some(border)),
            region(look::BOTTOM, Place::Bottom, Arrange::Spread, 1, None),
            region(look::SECTIONS, Place::Body, Arrange::Lines, 1, None),
            region(look::WORKING, Place::Bottom, Arrange::Lines, 0, None),
            region(look::BOX, Place::Bottom, Arrange::Lines, 0, Some(Border { title: true, sides: false, ..border })),
            region(look::LIST, Place::Bottom, Arrange::Lines, 0, None),
            region(
                look::PANE,
                Place::Side { percent: 50 },
                Arrange::Lines,
                1,
                Some(Border { title: false, sides: false, ..border }),
            ),
        ]),
        styles: styles(dim, faint, accent, good, back),
        glyphs: Glyphs { focus: '❯', mask: '•' },
    }
}

fn styles(dim: Rgb, faint: Rgb, accent: Rgb, good: Rgb, back: Option<Rgb>) -> Box<[StyleLook]> {
    let star = Some(Mark { glyph: '✦', paint: fore(accent) });
    let bold = Paint { bold: true, ..Paint::NONE };
    Box::new([
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
        StyleLook {
            band: back.is_some(),
            mark: Some(Mark { glyph: '❯', paint: fore(if back.is_some() { dim } else { accent }) }),
            ..style(look::SAID, if back.is_some() { Paint { back, ..Paint::NONE } } else { fore(dim) })
        },
        StyleLook { mark: star, ..style(look::AGENT, Paint::NONE) },
        StyleLook { mark: Some(Mark { glyph: '▸', paint: fore(dim) }), ..style(look::RUN, fore(dim)) },
        style(look::RUN_FAILED, fore(accent)),
        StyleLook { mark: star, ..style(look::DOTS, fore(faint)) },
        StyleLook { mark: star, ..style(look::RUNNING, fore(dim)) },
        style(look::END, Paint { reverse: true, ..Paint::NONE }),
        StyleLook { label: false, focused_value: Some(Paint::NONE), ..style(look::FIELD, Paint::NONE) },
        style(look::LIST_HEAD, bold),
        style(look::WHY, fore(accent)),
        StyleLook { described: Some(fore(dim)), ..style(look::OPTION, bold) },
        StyleLook { described: Some(fore(dim)), active_name: Some(Paint::NONE), ..style(look::COMMAND, bold) },
        style(look::NOTE, fore(dim)),
        StyleLook { spaced: true, ..style(look::SPACED, Paint::NONE) },
        StyleLook { mark: Some(Mark { glyph: '✓', paint: fore(good) }), ..style(look::STEP_DONE, Paint::NONE) },
        StyleLook { mark: Some(Mark { glyph: '▸', paint: fore(accent) }), ..style(look::STEP_NOW, bold) },
        StyleLook { mark: Some(Mark { glyph: '·', paint: fore(faint) }), ..style(look::STEP_AHEAD, fore(dim)) },
        StyleLook { align: Align::End, ..style(look::PLAYGROUND, fore(accent)) },
        StyleLook { align: Align::End, ..style(look::END_FAINT, fore(faint)) },
        StyleLook { mark: Some(Mark { glyph: '•', paint: Paint::NONE }), ..style(look::BULLETS, Paint::NONE) },
        StyleLook { mark: Some(Mark { glyph: ' ', paint: Paint::NONE }), ..style(look::LEFT_OFF, Paint::NONE) },
        StyleLook {
            band: back.is_some(),
            mark: Some(Mark { glyph: '❯', paint: fore(accent) }),
            ..style(look::PICKED, Paint { back, ..fore(accent) })
        },
        StyleLook { align: Align::End, ..style(look::RULE_TITLE, Paint::NONE) },
    ])
}
