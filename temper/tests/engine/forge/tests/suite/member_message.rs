//! Exact member messages and their recognition (temper/docs/design/protocol/tooling.md, 4).
use temper_engine_domain_forge::member_message::{Member, commit_message, names_head};

const HEAD: &str = "1111111111111111111111111111111111111111";
const TITLE: &[u8] = b"Add the squash trailers";

fn head() -> skein_forge::api::Commit {
    let mut head = [0; 32];
    head[..20].fill(0x11);
    head
}

#[test]
fn member_messages_have_the_exact_title_paragraph_and_trailers() {
    // Inputs, the description bytes they give, and the train's destination.
    let cases = [
        (
            "goal branch",
            b"Squashes each change.\n\nMore text.".to_vec(),
            Some(3),
            false,
            "Squashes each change.".to_owned(),
        ),
        ("main", b"Squashes each change.\n\nMore text.".to_vec(), Some(3), true, "Squashes each change.".to_owned()),
        ("no goal", b"Squashes each change.".to_vec(), None, true, "Squashes each change.".to_owned()),
        ("no words", Vec::new(), Some(3), false, String::new()),
        ("headings", b"\n## Summary\n\nFirst.".to_vec(), Some(3), false, "First.".to_owned()),
        ("empty paragraph", b"\n # Summary\n\t\n".to_vec(), None, false, String::new()),
        (
            "wrap at spaces",
            "word ".repeat(18).into_bytes(),
            None,
            false,
            format!("{}\n{}", "word ".repeat(13) + "word", "word ".repeat(3) + "word"),
        ),
        (
            "long word",
            format!("before {} after", "a".repeat(73)).into_bytes(),
            None,
            false,
            format!("before\n{}\nafter", "a".repeat(73)),
        ),
        (
            "character columns",
            format!("{} éé", "é".repeat(70)).into_bytes(),
            None,
            false,
            format!("{}\néé", "é".repeat(70)),
        ),
        ("byte boundary", "é".repeat(600).into_bytes(), None, false, "é".repeat(510) + "…"),
        ("exact byte bound", "a".repeat(1024).into_bytes(), None, false, "a".repeat(1024)),
        ("past byte bound", "a".repeat(1025).into_bytes(), None, false, "a".repeat(1021) + "…"),
        (
            "controls",
            b"First\0\x01\t word.\r\n  Joined.\n\nIgnored.".to_vec(),
            None,
            false,
            "First word. Joined.".to_owned(),
        ),
        ("invalid UTF-8", b"Valid.\xff Ignored.".to_vec(), None, false, "Valid.".to_owned()),
    ];
    for (name, words, goal, main, description) in cases {
        let member = Member { title: TITLE, words: &words, change: 12, pull: 7, head: head(), goal };
        let paragraph = if description.is_empty() { String::new() } else { format!("\n\n{description}") };
        let text = format!("Add the squash trailers{paragraph}\n\nChange: 12\nPull-request: 7\nHead: {HEAD}");
        let squash = if goal.is_some() { format!("{text}\nGoal: 3") } else { text.clone() };
        let message = if main && goal.is_some() { squash.clone() } else { text };
        assert_eq!(commit_message(&member, true).as_ref(), squash.as_bytes(), "{name}: squash");
        assert_eq!(commit_message(&member, main).as_ref(), message.as_bytes(), "{name}: message");
    }
}

#[test]
fn only_the_exact_head_line_in_the_last_paragraph_names_a_head() {
    let cases = [
        (format!("Title\n\nChange: 12\nPull-request: 7\nHead: {HEAD}\nGoal: 3"), true),
        (format!("Title\n\nHead: {HEAD}\n"), true),
        (format!("Title\n\nHead: {HEAD}\n\nDescription."), false),
        (format!("Title\n\nTemper-Head: {HEAD}"), false),
        ("Title\n\nHead: 2222222222222222222222222222222222222222".to_owned(), false),
        (format!("Title\n\nHead: {}", &HEAD[..39]), false),
        (format!("Title\n\nHead: {HEAD}000000000000000000000000"), false),
        (format!("Title\n\nHead: {HEAD} extra"), false),
        (format!("Title\n\n Head: {HEAD}"), false),
        (format!("Title\n\nHead: {HEAD}\n\n"), false),
    ];
    for (message, expected) in cases {
        assert_eq!(names_head(message.as_bytes(), head()), expected, "{message:?}");
    }
    let sha256 = [0x22; 32];
    let member = Member { title: TITLE, words: b"", change: 12, pull: 7, head: sha256, goal: None };
    let message = commit_message(&member, true);
    assert_eq!(
        message.as_ref(),
        format!("Add the squash trailers\n\nChange: 12\nPull-request: 7\nHead: {}", "22".repeat(32)).as_bytes()
    );
    assert!(names_head(&message, sha256));
    assert!(!names_head(format!("Title\n\nHead: {}", "22".repeat(20)).as_bytes(), sha256));
}
