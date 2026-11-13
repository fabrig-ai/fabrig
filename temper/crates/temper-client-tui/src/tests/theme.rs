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
#[expect(clippy::disallowed_methods, reason = "theme assertions inspect every optional paint slot")]
fn the_themes_are_valid_and_name_every_look() {
    assert_eq!(skein_ui::look::check(&[look::NAMES, temper_client_view_words::look::NAMES]), None);
    assert_eq!(look::OWNER.number, 65_538);
    assert_eq!(look::LINK, skein_lib::Token::new(1));
    assert_eq!(look::NAMES.prefix, b"tui");
    let mut failures = skein_lib::List::with_capacity(80);
    for (scheme, dim, faint, accent, good, background) in [
        (
            Scheme::Dark,
            rgb(0x9a, 0x93, 0x88),
            rgb(0x6a, 0x65, 0x5c),
            rgb(0xe0, 0x8a, 0x68),
            rgb(0x8f, 0xc2, 0x8f),
            rgb(0x19, 0x18, 0x16),
        ),
        (
            Scheme::Light,
            rgb(0x7a, 0x73, 0x6a),
            rgb(0xb5, 0xae, 0xa3),
            rgb(0xb8, 0x66, 0x4e),
            rgb(0x3f, 0x7d, 0x4e),
            rgb(0xff, 0xfe, 0xfd),
        ),
    ] {
        for background in [Some(background), None] {
            let back = background.map(theme::band);
            let theme = theme(scheme, background);
            assert!(theme.valid());
            assert_eq!(theme.placeholder, fore(faint));
            assert_eq!(theme.glyphs, Glyphs { focus: '❯', mask: '•' });
            if theme.active != (Paint { back, ..Paint::NONE }) {
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
                (2, "tui-sections", Place::Body, Arrange::Lines, None, 1),
                (3, "tui-working", Place::Bottom, Arrange::Lines, None, 0),
                (
                    4,
                    "tui-box",
                    Place::Bottom,
                    Arrange::Lines,
                    Some(Border { corners, across: '─', down: '│', paint: fore(faint), title: true, sides: false }),
                    0,
                ),
                (5, "tui-list", Place::Bottom, Arrange::Lines, None, 0),
                (
                    6,
                    "tui-pane",
                    Place::Side { percent: 50 },
                    Arrange::Lines,
                    Some(Border {
                        corners, across: '─', down: '│', paint: fore(faint), title: false, sides: false
                    }),
                    1,
                ),
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
                (
                    12,
                    "tui-said",
                    if back.is_some() { Paint { back, ..Paint::NONE } } else { fore(dim) },
                    Align::Start,
                    back.is_some(),
                    Some(Mark { glyph: '❯', paint: fore(if back.is_some() { dim } else { accent }) }),
                    None,
                    false,
                    true,
                ),
                (13, "tui-agent", Paint::NONE, Align::Start, false, star, None, false, true),
                (14, "tui-run", fore(dim), Align::Start, false, dim_mark, None, false, true),
                (15, "tui-run-failed", fore(accent), Align::Start, false, None, None, false, true),
                (16, "tui-dots", fore(faint), Align::Start, false, star, None, false, true),
                (17, "tui-running", fore(dim), Align::Start, false, star, None, false, true),
                (18, "tui-end", Paint { reverse: true, ..Paint::NONE }, Align::Start, false, None, None, false, true),
                (19, "tui-field", Paint::NONE, Align::Start, false, None, None, false, false),
                (
                    20,
                    "tui-list-head",
                    Paint { bold: true, ..Paint::NONE },
                    Align::Start,
                    false,
                    None,
                    None,
                    false,
                    true,
                ),
                (
                    21,
                    "tui-option",
                    Paint { bold: true, ..Paint::NONE },
                    Align::Start,
                    false,
                    None,
                    Some(fore(dim)),
                    false,
                    true,
                ),
                (22, "tui-note", fore(dim), Align::Start, false, None, None, false, true),
                (23, "tui-spaced", Paint::NONE, Align::Start, false, None, None, true, true),
                (
                    24,
                    "tui-command",
                    Paint { bold: true, ..Paint::NONE },
                    Align::Start,
                    false,
                    None,
                    Some(fore(dim)),
                    false,
                    true,
                ),
                (
                    25,
                    "tui-bullets",
                    Paint::NONE,
                    Align::Start,
                    false,
                    Some(Mark { glyph: '•', paint: Paint::NONE }),
                    None,
                    false,
                    true,
                ),
                (
                    26,
                    "tui-left-off",
                    Paint::NONE,
                    Align::Start,
                    false,
                    Some(Mark { glyph: ' ', paint: Paint::NONE }),
                    None,
                    false,
                    true,
                ),
                (
                    27,
                    "tui-picked",
                    Paint { back, ..fore(accent) },
                    Align::Start,
                    back.is_some(),
                    Some(Mark { glyph: '❯', paint: fore(accent) }),
                    None,
                    false,
                    true,
                ),
                (28, "tui-rule-title", Paint::NONE, Align::End, false, None, None, false, true),
                (
                    look::STEP_DONE.token,
                    "tui-step-done",
                    Paint::NONE,
                    Align::Start,
                    false,
                    Some(Mark { glyph: '✓', paint: fore(good) }),
                    None,
                    false,
                    true,
                ),
                (
                    look::STEP_NOW.token,
                    "tui-step-now",
                    Paint { bold: true, ..Paint::NONE },
                    Align::Start,
                    false,
                    Some(Mark { glyph: '▸', paint: fore(accent) }),
                    None,
                    false,
                    true,
                ),
                (
                    look::STEP_AHEAD.token,
                    "tui-step-ahead",
                    fore(dim),
                    Align::Start,
                    false,
                    Some(Mark { glyph: '·', paint: fore(faint) }),
                    None,
                    false,
                    true,
                ),
                (look::PLAYGROUND.token, "tui-playground", fore(accent), Align::End, false, None, None, false, true),
                (look::END_FAINT.token, "tui-end-faint", fore(faint), Align::End, false, None, None, false, true),
            ] {
                let style = Style { owner: look::OWNER, token };
                let active_name = if style == look::COMMAND { Some(Paint::NONE) } else { None };
                let want = StyleLook {
                    style,
                    paint,
                    align,
                    band,
                    label,
                    mark,
                    described,
                    active_name,
                    focused_value: if style == look::FIELD { Some(Paint::NONE) } else { None },
                    spaced,
                };
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
            if background.is_none() {
                assert_eq!(theme.base.back, None);
                assert_eq!(theme.focus.back, None);
                assert_eq!(theme.placeholder.back, None);
                assert!(theme.regions.iter().all(|look| look.paint.back.is_none()));
                assert!(theme.styles.iter().all(|look| look.paint.back.is_none()
                    && look.mark.is_none_or(|mark| mark.paint.back.is_none())
                    && look.described.is_none_or(|paint| paint.back.is_none())
                    && look.active_name.is_none_or(|paint| paint.back.is_none())));
            }
        }
    }
    assert!(failures.is_empty(), "theme rows: {failures:?}");
}

#[test]
fn the_palette_names_six_colours_in_order() {
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
    ] {
        let [bg, ink, dim, faint, accent, good] = colours;
        let want = [("bg", bg), ("ink", ink), ("dim", dim), ("faint", faint), ("accent", accent), ("good", good)];
        if theme::palette(scheme).as_slice() != want {
            failures.push(scheme).unwrap();
        }
    }
    assert!(failures.is_empty(), "palette rows: {failures:?}");
}

#[test]
fn the_base_leaves_the_terminals_colours_alone() {
    assert_eq!([Scheme::Dark, Scheme::Light].map(|scheme| theme(scheme, None).base), [Paint::NONE, Paint::NONE]);
}

#[test]
fn the_focus_names_only_the_accent_foreground() {
    for (scheme, accent) in [(Scheme::Dark, rgb(0xe0, 0x8a, 0x68)), (Scheme::Light, rgb(0xb8, 0x66, 0x4e))] {
        assert_eq!(theme(scheme, None).focus, fore(accent), "{scheme:?}");
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

#[test]
fn the_band_steps_toward_the_ink_and_saturates() {
    assert_eq!(theme::BAND_STEP, 20);
    for (background, expected) in [
        (rgb(0x19, 0x18, 0x16), rgb(0x2d, 0x2c, 0x2a)),
        (rgb(0xff, 0xfe, 0xfd), rgb(0xeb, 0xea, 0xe9)),
        (rgb(0x00, 0x2b, 0x36), rgb(0x14, 0x3f, 0x4a)),
        (rgb(0xff, 0x00, 0x00), rgb(0xff, 0x14, 0x14)),
        (rgb(0x00, 0xff, 0xff), rgb(0x00, 0xeb, 0xeb)),
    ] {
        assert_eq!(theme::band(background), expected, "{background:?}");
    }
}

#[test]
fn a_theme_choice_prints_reads_and_draws_its_scheme() {
    use theme::{THEME_KEY, ThemeChoice};
    assert_eq!(THEME_KEY, b"theme");
    assert_eq!(ThemeChoice::ALL, [ThemeChoice::Auto, ThemeChoice::Light, ThemeChoice::Dark]);
    for (choice, bytes) in
        [(ThemeChoice::Auto, b"auto".as_slice()), (ThemeChoice::Light, b"light"), (ThemeChoice::Dark, b"dark")]
    {
        assert_eq!(choice.print(), bytes);
        assert_eq!(ThemeChoice::read(bytes), choice);
        for terminal in [None, Some(Scheme::Light), Some(Scheme::Dark)] {
            let expected = match choice {
                ThemeChoice::Auto => terminal.unwrap_or(Scheme::Dark),
                ThemeChoice::Light => Scheme::Light,
                ThemeChoice::Dark => Scheme::Dark,
            };
            assert_eq!(choice.scheme(terminal), expected, "{choice:?}, {terminal:?}");
        }
    }
    for bytes in [b"".as_slice(), b"junk", b"Light", b"dark\n"] {
        assert_eq!(ThemeChoice::read(bytes), ThemeChoice::Auto);
    }
}
