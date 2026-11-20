use super::fixtures;

use super::fixtures::chat::{CLOSE_GRACE, closing};

use skein_ui::Role;

#[test]
fn a_chats_heading_is_its_first_sentence_or_line_cut_on_a_word() {
    for (title, short) in [
        ("Ship the update", "Ship the update"),
        (
            "okay let's design a small http server in python, using std lib. It should show a page with details about the moon cycle, where are we now in the cycle and when the next full moon will happen",
            "okay let's design a small http server in python, using std lib.",
        ),
        ("Why does it fail? It ran yesterday.", "Why does it fail?"),
        ("Ship it! Then rest.", "Ship it!"),
        ("Fix the build\nthen the docs. And more", "Fix the build"),
        ("Fix it\r\nthen", "Fix it"),
        ("Read 3.5 in lib.rs first. Then", "Read 3.5 in lib.rs first."),
        ("  \n\tIndented first line\nsecond", "Indented first line"),
        (
            "Show the moon cycle on one page, with where we are now and the next moon",
            "Show the moon cycle on one page, with where we are now and the next moon",
        ),
        (
            "Show the moon cycle on one page, with where we are now and the next one again soon",
            "Show the moon cycle on one page, with where we are now and the next one…",
        ),
        (
            "Please write a small http server in python for the moon cycle page, using the standard library",
            "Please write a small http server in python for the moon cycle page…",
        ),
        (&"x".repeat(100), "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx…"),
        (&"é".repeat(100), "ééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé…"),
        (&"é".repeat(72), "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé"),
        ("", ""),
    ] {
        assert_eq!(crate::common::chat::short_title(title.as_bytes()).as_ref(), short.as_bytes(), "{title:?}");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "test constructs Unicode titles and their expected cuts")]
fn a_row_title_is_cut_on_a_word_within_eighty_characters() {
    for (title, expected) in [
        ("Fix the cut", "Fix the cut"),
        ("Short. Another sentence", "Short. Another sentence"),
        (&"abcd ".repeat(20), "abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd…"),
        (&"é".repeat(80), &"é".repeat(80)),
        (&"é".repeat(81), &format!("{}…", "é".repeat(79))),
        (&format!("{} end", "x".repeat(79)), &format!("{}…", "x".repeat(79))),
        (&format!("{}, more", "x".repeat(78)), &format!("{},…", "x".repeat(78))),
        ("", ""),
    ] {
        assert_eq!(crate::common::chat::row_title(title.as_bytes()).as_ref(), expected.as_bytes(), "{title:?}");
        assert_eq!(crate::shared::row_title(title.as_bytes()).as_ref(), expected.as_bytes(), "{title:?}");
    }
}

#[test]
fn only_complete_words_with_no_body_are_only_the_title() {
    use jig_client::contract as c;

    for (input, cut, expected) in [
        ("Fix the cut", false, true),
        ("Fix the cut\n \r\n\t", false, true),
        ("Fix the cut\nMore.", false, false),
        ("Fix the cut\n\u{a0}", false, false),
        ("Fix the cut", true, false),
        ("Fix the cut\n \r\n\t", true, false),
        ("", true, false),
        ("", false, true),
    ] {
        let words = c::DocumentText { bytes: c::Text::new(input.as_bytes().into(), 128).unwrap(), cut };
        assert_eq!(crate::common::chat::only_title(&words), expected, "{input:?}, cut={cut}");
        assert_eq!(crate::shared::only_title(&words), expected, "{input:?}, cut={cut}");
    }
}

#[test]
fn words_after_the_title_keep_body_bytes_but_not_trailing_blanks_alone() {
    use jig_client::contract as c;
    for (input, expected) in [
        ("Release the runner", ""),
        ("Release the runner\n \t\r\n", ""),
        ("Release the runner\nIt waits on CI.", "It waits on CI."),
        ("Release the runner\n\n More.\n", "\n More.\n"),
        ("Release the runner\n\u{a0}", "\u{a0}"),
    ] {
        for cut in [true, false] {
            let words = c::DocumentText { bytes: c::Text::new(input.as_bytes().into(), 128).unwrap(), cut };
            assert_eq!(crate::shared::after_title(&words), expected.as_bytes(), "{input:?}, cut={cut}");
        }
    }
}

#[test]
fn the_close_confirmation_names_each_live_delegates_fate() {
    let tree = closing(Some(CLOSE_GRACE));
    let text = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&text, b"T12 is cancelled.").is_some());
    assert!(skein_lib::bytes::find(&text, b"T13 moves to you.").is_some());
}

#[test]
fn the_close_confirmation_offers_each_live_delegates_choice() {
    let tree = closing(Some(CLOSE_GRACE));
    assert!(fixtures::find(&tree, Role::Button, b"Take it instead").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Cancel it instead").is_some());
}
