use super::fixtures;

use super::fixtures::chat::{
    CLOSE_GRACE,
    closing,
};



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
