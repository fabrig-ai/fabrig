use crate::{Scheme, look, theme};
use skein_ui_screen::{Arrange, Border, Glyphs, Paint, Place, Rgb};

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}
fn fore(colour: Rgb) -> Paint {
    Paint { fore: Some(colour), ..Paint::NONE }
}

#[test]
fn the_themes_are_valid_and_name_every_look() {
    assert_eq!(skein_ui::look::check(&[look::NAMES, temper_client_view::look::NAMES]), None);
    assert_eq!(look::OWNER.number, 65_538);
    assert_eq!(look::LINK, skein_lib::Token::new(1));
    assert_eq!(look::QUIT, skein_lib::Token::new(2));
    assert_eq!(look::NAMES.prefix, b"tui");
    for (scheme, dim, faint, accent, good) in [
        (Scheme::Dark, rgb(0x9a, 0x93, 0x88), rgb(0x6a, 0x65, 0x5c), rgb(0xe0, 0x8a, 0x68), rgb(0x8f, 0xc2, 0x8f)),
        (Scheme::Light, rgb(0x7a, 0x73, 0x6a), rgb(0xb5, 0xae, 0xa3), rgb(0xb8, 0x66, 0x4e), rgb(0x3f, 0x7d, 0x4e)),
    ] {
        let theme = theme(scheme);
        assert!(theme.valid());
        assert_eq!(theme.placeholder, fore(faint));
        assert_eq!(theme.glyphs, Glyphs { focus: '❯', mask: '•' });
        for (style, name, paint) in [
            (look::TITLE, b"tui-title".as_slice(), Paint { bold: true, ..fore(accent) }),
            (look::HINT, b"tui-hint", fore(dim)),
            (look::LIVE, b"tui-live", fore(good)),
            (look::WAITING, b"tui-waiting", fore(dim)),
            (look::DOWN, b"tui-down", fore(accent)),
            (look::REFUSED, b"tui-refused", fore(accent)),
        ] {
            assert_eq!(look::NAMES.style(style), Some(name));
            let mut found = None;
            for candidate in &theme.styles {
                if candidate.style == style {
                    found = Some(candidate.paint);
                }
            }
            assert_eq!(found, Some(paint));
        }
        for (region, name, place, arrange, border) in [
            (
                look::TOP,
                b"tui-top".as_slice(),
                Place::Top,
                Arrange::Lines,
                Some(Border {
                    corners: ['╭', '╮', '╰', '╯'], across: '─', down: '│', paint: fore(faint), title: true
                }),
            ),
            (look::BOTTOM, b"tui-bottom", Place::Bottom, Arrange::Spread, None),
        ] {
            assert_eq!(look::NAMES.region(region), Some(name));
            let mut found = None;
            for candidate in &theme.regions {
                if candidate.region == region {
                    found = Some(candidate);
                }
            }
            let found = found.expect("named region has a look");
            assert_eq!(
                (found.place, found.arrange, found.border, found.pad, found.paint),
                (place, arrange, border, 1, Paint::NONE)
            );
        }
        assert_eq!(theme.styles.len(), look::NAMES.styles.len());
        assert_eq!(theme.regions.len(), look::NAMES.regions.len());
    }
}

#[test]
fn the_palette_names_six_colours_in_order() {
    let rows = [
        (
            Scheme::Dark,
            [
                rgb(0x19, 0x18, 0x16),
                rgb(0xe8, 0xe3, 0xda),
                rgb(0x9a, 0x93, 0x88),
                rgb(0x6a, 0x65, 0x5c),
                rgb(0xe0, 0x8a, 0x68),
                rgb(0x8f, 0xc2, 0x8f),
            ],
        ),
        (
            Scheme::Light,
            [
                rgb(0xff, 0xfe, 0xfd),
                rgb(0x34, 0x31, 0x2c),
                rgb(0x7a, 0x73, 0x6a),
                rgb(0xb5, 0xae, 0xa3),
                rgb(0xb8, 0x66, 0x4e),
                rgb(0x3f, 0x7d, 0x4e),
            ],
        ),
    ];
    for (scheme, [bg, ink, dim, faint, accent, good]) in rows {
        assert_eq!(
            theme::palette(scheme).as_slice(),
            [("bg", bg), ("ink", ink), ("dim", dim), ("faint", faint), ("accent", accent), ("good", good)],
            "{scheme:?}"
        );
    }
}

#[test]
fn the_base_leaves_the_terminals_colours_alone() {
    assert_eq!([Scheme::Dark, Scheme::Light].map(|scheme| theme(scheme).base), [Paint::NONE, Paint::NONE]);
}

#[test]
fn the_focus_names_only_the_accent_foreground() {
    for (scheme, accent) in [(Scheme::Dark, rgb(0xe0, 0x8a, 0x68)), (Scheme::Light, rgb(0xb8, 0x66, 0x4e))] {
        assert_eq!(theme(scheme).focus, fore(accent), "{scheme:?}");
    }
}

#[test]
fn the_background_luma_chooses_the_scheme() {
    // Grays straddle half of white: 127 * 10_000 = 1_270_000,
    // 128 * 10_000 = 1_280_000; the threshold is 1_275_000.
    // Black, white, two solarized grounds, two theme grounds, two grays.
    const BACKGROUNDS: usize = 8;
    const ROWS: [(Rgb, Scheme); BACKGROUNDS] = [
        (rgb(0, 0, 0), Scheme::Dark),
        (rgb(255, 255, 255), Scheme::Light),
        (rgb(0x00, 0x2b, 0x36), Scheme::Dark),
        (rgb(0xfd, 0xf6, 0xe3), Scheme::Light),
        (rgb(0x19, 0x18, 0x16), Scheme::Dark),
        (rgb(0xff, 0xfe, 0xfd), Scheme::Light),
        (rgb(0x7f, 0x7f, 0x7f), Scheme::Dark),
        (rgb(0x80, 0x80, 0x80), Scheme::Light),
    ];
    const CLASSIFIED: [Scheme; BACKGROUNDS] = [
        Scheme::of_background(ROWS[0].0),
        Scheme::of_background(ROWS[1].0),
        Scheme::of_background(ROWS[2].0),
        Scheme::of_background(ROWS[3].0),
        Scheme::of_background(ROWS[4].0),
        Scheme::of_background(ROWS[5].0),
        Scheme::of_background(ROWS[6].0),
        Scheme::of_background(ROWS[7].0),
    ];
    assert_eq!(CLASSIFIED, ROWS.map(|(_, scheme)| scheme));
}
