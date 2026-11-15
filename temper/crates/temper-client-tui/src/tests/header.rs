//! The header's cells, places and terminal fallback (client/tui/pages/README.md, 3).
use super::fixtures as f;
use crate::{Local, Scheme, Told, Tui, look};
use skein_lib::{List, bytes::concat};
use skein_ui::Role;
use skein_ui_screen::{Focus, Frame, Size, render};

const VERSION: &[u8] = b"0.0.0 (b948f40d6df1)";

fn row(frame: &Frame, number: u16) -> Box<[u8]> {
    let mut bytes = List::with_capacity(u32::from(frame.size().columns).checked_mul(4).unwrap());
    for cell in frame.row(number).unwrap() {
        for byte in cell.glyph.encode_utf8(&mut [0; 4]).as_bytes() {
            bytes.push(*byte).unwrap();
        }
    }
    skein_lib::bytes::copy_of(bytes.as_slice().trim_ascii_end())
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the header's rows, cells, tokens and plain fallback")]
fn the_header_names_only_the_build_and_place() {
    let root = super::launchpad::home(Some(true), &[]);
    for (front, plain, version) in [
        (false, false, VERSION),
        (true, false, VERSION),
        (true, true, VERSION),
        (false, true, b"".as_slice()),
        (false, false, b"".as_slice()),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.header(plain, version.into());
        tui.step(Told::Local { local: Local::Signing });
        if front {
            tui.step(Told::Front { set_up: false, box_log: Box::default() });
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let tree = view.tree();
        let mut header = None;
        for node in tree.nodes() {
            if node.region == Some(look::HEADER) {
                header = Some(node);
            }
        }
        let header = header.unwrap();
        let start = f::index(tree, header.id);
        let nodes = &tree.nodes()[start..start + usize::try_from(header.size).unwrap()];
        let mut lines = List::with_capacity(3);
        for node in nodes {
            if node.role == Role::Paragraph {
                lines.push(node).unwrap();
            }
        }
        assert_eq!(lines.len(), 3);
        assert_eq!(f::text(tree, lines.get(0).unwrap()).as_ref(), b"");
        let prefix = if plain { "" } else { " ▄█▀▀   " };
        let build = if version.is_empty() { "" } else { " 0.0.0 (b948f40d6df1)" };
        let first = concat(&[prefix.as_bytes(), b"temper", build.as_bytes()]);
        let prefix = if plain { "" } else { "▄█▄█▄█  " };
        let place = if front { "temper box · a whole deployment on this machine" } else { "127.0.0.1:8367" };
        let second = concat(&[prefix.as_bytes(), place.as_bytes()]);
        assert_eq!(f::text(tree, lines.get(1).unwrap()).as_ref(), first.as_ref());
        assert_eq!(f::text(tree, lines.get(2).unwrap()).as_ref(), second.as_ref());
        let name = f::find(tree, Role::Text, b"temper");
        assert_eq!(name.styles.as_slice(), &[look::NAME]);
        if !version.is_empty() {
            assert_eq!(f::find(tree, Role::Text, b" 0.0.0 (b948f40d6df1)").styles.as_slice(), &[look::HINT]);
        }
        if front {
            for (words, style) in
                [("temper box", look::NAME), (" · ", look::FAINT), ("a whole deployment on this machine", look::HINT)]
            {
                let mut found = false;
                for node in nodes {
                    if node.role == Role::Text
                        && f::text(tree, node).as_ref() == words.as_bytes()
                        && node.styles.as_slice() == [style]
                    {
                        found = true;
                    }
                }
                assert!(found, "{words}: header token");
            }
        } else {
            assert_eq!(f::find(tree, Role::Text, f::ADDRESS).styles.as_slice(), &[look::HINT]);
        }
        if !plain {
            let mut tokens = List::with_capacity(10);
            for node in nodes {
                if node.role == Role::Text {
                    for style in &node.styles {
                        if [look::MARK_BLUE, look::MARK_CYAN, look::MARK_VIOLET].contains(style) {
                            tokens.push(*style).unwrap();
                        }
                    }
                }
            }
            assert_eq!(
                tokens.as_slice(),
                [
                    look::MARK_BLUE,
                    look::MARK_CYAN,
                    look::MARK_CYAN,
                    look::MARK_CYAN,
                    look::MARK_BLUE,
                    look::MARK_BLUE,
                    look::MARK_VIOLET,
                    look::MARK_VIOLET,
                    look::MARK_VIOLET,
                    look::MARK_VIOLET
                ]
            );
        }
        for scheme in [Scheme::Dark, Scheme::Light] {
            let size = Size { columns: 80, rows: 24 };
            let mut frame = Frame::new(size);
            render(tree, &crate::theme(scheme, None), size, &mut Focus::default(), &mut frame);
            assert_eq!(row(&frame, 0).as_ref(), b"");
            assert_eq!(row(&frame, 1), concat(&[b" ", &first]));
            assert_eq!(row(&frame, 2), concat(&[b" ", &second]));
            assert_eq!(row(&frame, 3).as_ref(), b"");
        }
    }
}

#[test]
fn the_narrow_header_cuts_the_ends_not_the_mark() {
    let root = super::launchpad::home(None, &[]);
    let mut tui = Tui::new(f::ADDRESS);
    tui.header(false, VERSION.into());
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    for columns in [10, 20] {
        let size = Size { columns, rows: 24 };
        let mut frame = Frame::new(size);
        render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut Focus::default(), &mut frame);
        assert_eq!(row(&frame, 0).as_ref(), b"");
        for (number, expected) in [(1, "  ▄█▀▀   temper 0.0.0 (b948f40d6df1)"), (2, " ▄█▄█▄█  127.0.0.1:8367")]
        {
            let cells = frame.row(number).unwrap();
            for (cell, glyph) in cells.iter().take(usize::from(columns - 1)).zip(expected.chars()) {
                assert_eq!(cell.glyph, glyph);
            }
            assert_eq!(cells.last().unwrap().glyph, ' ');
        }
        assert_eq!(row(&frame, 3).as_ref(), b"");
    }
}

#[test]
fn the_welcome_uses_the_header_and_spaces_its_link() {
    let (root, _) = f::starting();
    let mut tui = Tui::new(f::ADDRESS);
    tui.header(false, VERSION.into());
    tui.step(Told::Front { set_up: false, box_log: Box::default() });
    tui.step(Told::BoxLine {
        line: crate::BoxLine::Link { url: skein_lib::Secret::new(b"http://127.0.0.1:8367/link".as_slice().into()) },
    });
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    let size = Size { columns: 150, rows: 39 };
    let mut frame = Frame::new(size);
    render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut Focus::default(), &mut frame);
    for (number, expected) in [
        "",
        "  ▄█▀▀   temper 0.0.0 (b948f40d6df1)",
        " ▄█▄█▄█  temper box · a whole deployment on this machine",
        "",
        " Welcome. Set temper up in your browser: open this link on this machine.",
        "",
        " http://127.0.0.1:8367/link",
        "",
        " It signs you in as this temper's owner, once, within 2 hours.",
        "",
        " In the browser  1 Welcome · 2 Forge · 3 Model · 4 Ready",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(row(&frame, u16::try_from(number).unwrap()).as_ref(), expected.as_bytes());
    }
    assert_eq!(f::find(view.tree(), Role::Text, b"In the browser").styles.as_slice(), &[look::HEAD]);
    assert_eq!(
        f::find(view.tree(), Role::Text, "1 Welcome · 2 Forge · 3 Model · 4 Ready".as_bytes()).styles.as_slice(),
        &[look::HINT]
    );
}

#[test]
fn the_headline_is_the_first_unheaded_section_and_empty_is_omitted() {
    for brief in [Some(false), None] {
        let root = super::launchpad::home(brief, &[]);
        let mut view = f::view();
        f::build(&mut view, &Tui::new(f::ADDRESS), &root);
        let tree = view.tree();
        let mut spaced = None;
        for node in tree.nodes() {
            if node.role == Role::Group && node.styles.as_slice() == [look::SPACED] {
                spaced = Some(node);
                break;
            }
        }
        let spaced = spaced.unwrap();
        let section = &tree.nodes()[f::index(tree, spaced.id) + 1];
        assert_eq!(section.role, Role::Region);
        let start = f::index(tree, section.id);
        let nodes = &tree.nodes()[start..start + usize::try_from(section.size).unwrap()];
        let mut line = None;
        for node in nodes {
            if node.role == Role::Paragraph {
                line = Some(node);
                break;
            }
        }
        let line = line.unwrap();
        if brief.is_some() {
            assert!(!line.styles.as_slice().contains(&look::HEAD));
            assert!(f::text(tree, line).ends_with(b"Nothing needs you."));
        } else {
            assert_eq!(f::text(tree, line).as_ref(), b"Start");
        }
    }
}
