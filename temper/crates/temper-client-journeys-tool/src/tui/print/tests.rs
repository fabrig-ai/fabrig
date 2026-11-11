use super::*;
use skein_ui_screen::{Cell, Size};

fn marked(key: Option<Key>) -> Mark {
    let size = Size { rows: 2, columns: 8 };
    let mut frame = Frame::new(size);
    frame.clear(size, Cell { glyph: ' ', paint: theme(Scheme::Dark, None).base });
    Mark { frame, live: None, key }
}

#[test]
fn rows_group_paints_escape_text_and_wrap_the_cursor() {
    let mut mark = marked(None);
    let base = theme(Scheme::Dark, None).base;
    let accent = Paint { fore: Some(palette(Scheme::Dark)[4].1), ..base };
    for (column, glyph, paint) in
        [(0, '<', base), (1, '>', base), (2, '&', accent), (3, '"', accent), (4, '界', base), (5, '\0', base)]
    {
        assert!(mark.frame.put(Position { row: 0, column }, Cell { glyph, paint }));
    }
    mark.frame.set_cursor(Some(Position { row: 0, column: 3 }));
    let files = pages("sample", Scheme::Dark, &[mark]);
    assert_eq!(files[0].document, "<>&\"界\n\n");
    let html = &files[1].document;
    assert!(
        html.contains(
            ">\n&lt;&gt;<span class=\"f-accent\">&amp;<span class=\"cursor\">&quot;</span></span>界  \n        \n</pre>"
        ),
        "{html}"
    );
    assert!(html.contains("<title>sample 01 · temper</title>"));
    assert!(html.contains("<pre class=\"tui tui-dark\">"));
    assert!(html.contains("href=\"../theme.css\"><link rel=\"stylesheet\" href=\"../overlay.css\">"));
    assert!(html.contains("<script src=\"../tui-journey.js\" defer>"));
}

#[test]
fn palette_colours_and_each_attribute_have_classes() {
    for (paint, expected) in [
        (
            Paint { fore: Some(palette(Scheme::Light)[4].1), back: Some(palette(Scheme::Light)[3].1), ..Paint::NONE },
            "<span class=\"f-accent b-faint\">",
        ),
        (Paint { bold: true, ..Paint::NONE }, "<span class=\"bold\">"),
        (Paint { dim: true, ..Paint::NONE }, "<span class=\"dim\">"),
        (Paint { italic: true, ..Paint::NONE }, "<span class=\"italic\">"),
        (Paint { underline: true, ..Paint::NONE }, "<span class=\"underline\">"),
        (Paint { reverse: true, ..Paint::NONE }, "<span class=\"reverse\">"),
        (
            Paint {
                fore: Some(Rgb { red: 1, green: 2, blue: 3 }),
                back: Some(Rgb { red: 4, green: 5, blue: 6 }),
                ..Paint::NONE
            },
            "<span style=\"color:#010203;background:#040506\">",
        ),
    ] {
        assert_eq!(paint_span(paint, Scheme::Light), expected);
    }
}

#[test]
fn a_keys_page_links_and_a_mark_keys_has_neighbours_alone() {
    for (key, name) in
        [(Key::Enter, "enter"), (Key::Char('n'), "n"), (Key::Char('"'), "&quot;"), (Key::Control(b'c'), "control c")]
    {
        let files = pages("keys", Scheme::Light, &[marked(Some(key)), marked(None), marked(None)]);
        let first = &files[1].document;
        assert!(first.contains(&format!(
            "<pre class=\"tui tui-light\" data-journey-key=\"{name}\" data-journey-next=\"02.html\">"
        )));
        assert!(!first.contains("rel=\"prev\""));
        let middle = &files[3].document;
        assert!(middle.contains("<link rel=\"prev\" href=\"01.html\">"));
        assert!(middle.contains("<link rel=\"next\" href=\"03.html\">"));
        assert!(!middle.contains("data-journey-"));
        assert!(!files[5].document.contains("rel=\"next\""));
    }
}

#[test]
fn beside_regenerates_the_palette_and_script_but_preserves_overlay() {
    let scratch = skein_scratch::Scratch::new("tui-print-beside");
    beside(scratch.path()).expect("beside");
    assert_eq!(fs::read(scratch.path().join("overlay.css")).expect("empty overlay"), b"");
    fs::write(scratch.path().join("overlay.css"), b"request").expect("overlay request");
    fs::write(scratch.path().join("theme.css"), b"old").expect("old palette");
    fs::write(scratch.path().join("tui-journey.js"), b"old").expect("old script");
    beside(scratch.path()).expect("reprint beside");
    assert_eq!(fs::read(scratch.path().join("overlay.css")).expect("preserved overlay"), b"request");
    let css = fs::read_to_string(scratch.path().join("theme.css")).expect("palette");
    assert_eq!(css, stylesheet());
    assert!(css.contains("pre.tui { color: var(--ink); background: var(--bg);"));
    assert!(css.contains(".tui-dark {\n  --bg: #191816;\n  --ink: #E8E3DA;"));
    assert!(css.contains(".tui-light {\n  --bg: #FFFEFD;\n  --ink: #34312C;"));
    assert_eq!(fs::read_to_string(scratch.path().join("tui-journey.js")).expect("script"), SCRIPT);
}
