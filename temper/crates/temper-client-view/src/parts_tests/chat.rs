use super::fixtures;

use fixtures::chat::{
    ATTEMPT, BYTES, CLOSE_GRACE, COMMITTED, GRACES, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, chat_root,
    closing, messaging_root, projected_tree as tree, proposal_root, provisional, provisional_with_text, rejecting_chat,
};

use jig_client::{contract as c, thread::Thread};

use skein_lib::{List, Wall};

use skein_ui::Role;


pub(super) use super::fixtures::chat_helpers::{
    asking_task, asking_task_root, asking_task_root_owned, assert_run_line, deliver_chat, long_line_page, model_choice,
    model_effort, model_line_page, model_line_root, named_position, pause_chip, structured_thread,
};


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
