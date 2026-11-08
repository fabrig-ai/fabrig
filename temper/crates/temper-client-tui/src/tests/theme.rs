use crate::{Scheme, look, theme};
use skein_ui::{Region, Style};
use skein_ui_screen::{Align, Arrange, Border, Glyphs, Mark, Paint, Place, RegionLook, Rgb, StyleLook};

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}
fn fore(colour: Rgb) -> Paint {
    Paint { fore: Some(colour), ..Paint::NONE }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the design's region and token rows check both complete themes")]
fn the_themes_are_valid_and_name_every_look() {
    assert_eq!(skein_ui::look::check(&[look::NAMES, temper_client_view::look::NAMES]), None);
    assert_eq!(look::OWNER.number, 65_538);
    assert_eq!(look::LINK, skein_lib::Token::new(1));
    assert_eq!(look::QUIT, skein_lib::Token::new(2));
    assert_eq!(look::NAMES.prefix, b"tui");
    let mut failures = skein_lib::List::with_capacity(80);
    for (scheme, dim, faint, accent, good, sel) in [
        (
            Scheme::Dark,
            rgb(0x9a, 0x93, 0x88),
            rgb(0x6a, 0x65, 0x5c),
            rgb(0xe0, 0x8a, 0x68),
            rgb(0x8f, 0xc2, 0x8f),
            rgb(0x2a, 0x28, 0x25),
        ),
        (
            Scheme::Light,
            rgb(0x7a, 0x73, 0x6a),
            rgb(0xb5, 0xae, 0xa3),
            rgb(0xb8, 0x66, 0x4e),
            rgb(0x3f, 0x7d, 0x4e),
            rgb(0xf4, 0xf0, 0xe9),
        ),
    ] {
        let theme = theme(scheme);
        assert!(theme.valid());
        assert_eq!(theme.placeholder, fore(faint));
        assert_eq!(theme.glyphs, Glyphs { focus: '❯', mask: '•' });
        if theme.active != (Paint { back: Some(sel), ..Paint::NONE }) {
            failures.push((scheme, "active")).unwrap();
        }
        let corners = ['╭', '╮', '╰', '╯'];
        for (number, name, place, arrange, border, pad) in [
            (
                0,
                "tui-top",
                Place::Top,
                Arrange::Lines,
                Some(Border { corners, across: '─', down: '│', paint: fore(faint), title: true, sides: true }),
                1,
            ),
            (1, "tui-bottom", Place::Bottom, Arrange::Spread, None, 1),
            (2, "tui-chat-top", Place::Top, Arrange::Spread, None, 1),
            (3, "tui-sections", Place::Body, Arrange::Lines, None, 1),
            (4, "tui-working", Place::Bottom, Arrange::Lines, None, 0),
            (
                5,
                "tui-box",
                Place::Bottom,
                Arrange::Lines,
                Some(Border { corners, across: '─', down: '│', paint: fore(faint), title: false, sides: false }),
                0,
            ),
            (6, "tui-list", Place::Bottom, Arrange::Lines, None, 0),
        ] {
            let region = Region { owner: look::OWNER, number };
            let want = RegionLook { region, place, arrange, border, pad, paint: Paint::NONE };
            let mut found = None;
            for row in &theme.regions {
                if row.region == region {
                    found = Some(row);
                }
            }
            if found != Some(&want) || look::NAMES.region(region) != Some(name.as_bytes()) {
                failures.push((scheme, name)).unwrap();
            }
        }
        let dim_mark = Some(Mark { glyph: '▸', paint: fore(dim) });
        let star = Some(Mark { glyph: '✦', paint: fore(accent) });
        for (token, name, paint, align, band, mark, described, spaced, label) in [
            (0, "tui-title", Paint { bold: true, ..fore(accent) }, Align::Start, false, None, None, false, true),
            (1, "tui-hint", fore(dim), Align::Start, false, None, None, false, true),
            (2, "tui-live", fore(good), Align::Start, false, None, None, false, true),
            (3, "tui-waiting", fore(dim), Align::Start, false, None, None, false, true),
            (4, "tui-down", fore(accent), Align::Start, false, None, None, false, true),
            (5, "tui-refused", fore(accent), Align::Start, false, None, None, false, true),
            (6, "tui-name", Paint { bold: true, ..Paint::NONE }, Align::Start, false, None, None, false, true),
            (7, "tui-faint", fore(faint), Align::Start, false, None, None, false, true),
            (8, "tui-address", fore(dim), Align::End, false, None, None, false, true),
            (9, "tui-byline", fore(dim), Align::End, false, None, None, false, true),
            (10, "tui-head", Paint { bold: true, ..Paint::NONE }, Align::Start, false, None, None, false, true),
            (11, "tui-signal", fore(accent), Align::Start, false, None, None, false, true),
            (12, "tui-star", fore(accent), Align::Start, false, None, None, false, true),
            (
                13,
                "tui-said",
                Paint { back: Some(sel), ..Paint::NONE },
                Align::Start,
                true,
                Some(Mark { glyph: '❯', paint: fore(dim) }),
                None,
                false,
                true,
            ),
            (14, "tui-agent", Paint::NONE, Align::Start, false, star, None, false, true),
            (15, "tui-run", fore(dim), Align::Start, false, dim_mark, None, false, true),
            (16, "tui-run-failed", fore(accent), Align::Start, false, None, None, false, true),
            (17, "tui-dots", fore(faint), Align::Start, false, star, None, false, true),
            (18, "tui-running", fore(dim), Align::Start, false, star, None, false, true),
            (19, "tui-end", Paint { reverse: true, ..Paint::NONE }, Align::Start, false, None, None, false, true),
            (20, "tui-field", Paint::NONE, Align::Start, false, None, None, false, false),
            (21, "tui-list-head", Paint { bold: true, ..Paint::NONE }, Align::Start, false, None, None, false, true),
            (
                22,
                "tui-option",
                Paint { bold: true, ..Paint::NONE },
                Align::Start,
                false,
                None,
                Some(fore(dim)),
                false,
                true,
            ),
            (23, "tui-note", fore(dim), Align::Start, false, None, None, false, true),
            (24, "tui-spaced", Paint::NONE, Align::Start, false, None, None, true, true),
        ] {
            let style = Style { owner: look::OWNER, token };
            let want = StyleLook { style, paint, align, band, label, mark, described, spaced };
            let mut found = None;
            for row in &theme.styles {
                if row.style == style {
                    found = Some(row);
                }
            }
            if found != Some(&want) || look::NAMES.style(style) != Some(name.as_bytes()) {
                failures.push((scheme, name)).unwrap();
            }
        }
    }
    assert!(failures.is_empty(), "theme rows: {failures:?}");
}

#[test]
fn the_palette_names_seven_colours_in_order() {
    let mut failures = skein_lib::List::with_capacity(2);
    for (scheme, colours) in [
        (
            Scheme::Dark,
            [
                rgb(0x19, 0x18, 0x16),
                rgb(0xe8, 0xe3, 0xda),
                rgb(0x9a, 0x93, 0x88),
                rgb(0x6a, 0x65, 0x5c),
                rgb(0xe0, 0x8a, 0x68),
                rgb(0x8f, 0xc2, 0x8f),
                rgb(0x2a, 0x28, 0x25),
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
                rgb(0xf4, 0xf0, 0xe9),
            ],
        ),
    ] {
        let [bg, ink, dim, faint, accent, good, sel] = colours;
        let want = [
            ("bg", bg),
            ("ink", ink),
            ("dim", dim),
            ("faint", faint),
            ("accent", accent),
            ("good", good),
            ("sel", sel),
        ];
        if theme::palette(scheme).as_slice() != want {
            failures.push(scheme).unwrap();
        }
    }
    assert!(failures.is_empty(), "palette rows: {failures:?}");
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
