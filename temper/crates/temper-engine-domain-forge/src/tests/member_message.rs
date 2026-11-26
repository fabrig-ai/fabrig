//! Squash trailer sources and bounds (temper/docs/design/protocol/tooling.md, 4).
use crate::member_message::{Kind, Member, TRAILER_BYTES, commit_message, names_head, trailers};

const HEAD: [u8; 32] = [0x11; 32];
const HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";

fn member<'a>(words: &'a [u8], head_trailers: &'a [u8]) -> Member<'a> {
    Member { title: b"TITLE", words, change: 17960, pull: 6808, head: HEAD, goal: Some(17816), head_trailers }
}

#[test]
fn the_head_supplies_look_trailers_to_the_squash_alone() {
    let head = trailers(
        b"Show a model's week\n\nAdded the week to accounts.\n\n## Look account\n\n\
          | Page | Look |\n| --- | --- |\n| Accounts | Model week |\n\n\
          Look-request: usage-model-week\nLook-landed: usage-model-week\n",
    );
    let member = member(b"Accounts names a model's week.\n\n## Left\n\nnone", &head);
    assert_eq!(
        commit_message(&member, Kind::Squash).as_ref(),
        format!(
            "TITLE\n\nAccounts names a model's week.\n\nLook-request: usage-model-week\n\
             Look-landed: usage-model-week\nChange: 17960\nPull-request: 6808\nHead: {HEX}\nGoal: 17816"
        )
        .as_bytes(),
    );
    assert_eq!(
        commit_message(&member, Kind::Merge { into_main: true }).as_ref(),
        format!(
            "TITLE\n\nAccounts names a model's week.\n\nChange: 17960\nPull-request: 6808\nHead: {HEX}\nGoal: 17816"
        )
        .as_bytes(),
    );
}

#[test]
fn the_words_win_only_when_they_keep_trailers() {
    let head = trailers(b"Subject\n\nLook-request: b");
    for (words, kept) in [
        (b"Summary.\n\nLook-request: a".as_slice(), "Look-request: a"),
        (b"Summary.\n\nChange: 1".as_slice(), "Look-request: b"),
    ] {
        let member = member(words, &head);
        assert_eq!(
            commit_message(&member, Kind::Squash).as_ref(),
            format!("TITLE\n\nSummary.\n\n{kept}\nChange: 17960\nPull-request: 6808\nHead: {HEX}\nGoal: 17816")
                .as_bytes(),
        );
    }
}

#[test]
fn temper_alone_writes_its_trailers() {
    let words = b"Summary.\n\nChange: 1\nPull-request: 2\nHead: 3\nGoal: 4\nlook-request: x\nhead: y\nGOAL: z";
    assert_eq!(trailers(words).as_ref(), b"look-request: x");
    for (goal, expected_goal) in [(Some(17816), "\nGoal: 17816"), (None, "")] {
        let mut member = member(words, &[]);
        member.goal = goal;
        let squash = commit_message(&member, Kind::Squash);
        assert_eq!(
            squash.as_ref(),
            format!(
                "TITLE\n\nSummary.\n\nlook-request: x\nChange: 17960\nPull-request: 6808\nHead: {HEX}{expected_goal}"
            )
            .as_bytes(),
        );
        assert!(names_head(&squash, HEAD));
    }
}

#[test]
fn only_a_last_paragraph_of_trailer_lines_is_kept() {
    let cases: &[(&[u8], &[u8])] = &[
        (b"Subject\n\nKey: v\nA line of prose.", b""),
        (b"Key: v", b""),
        (b"\n\n \t\nKey: v", b""),
        (b"Subject\n\nKey:value", b""),
        (b"Subject\n\nKey: ", b""),
        (b"Subject\n\n1Key: v", b""),
        (b"Subject\n\nKe y: v", b""),
        (b"Subject\n\nKey: v\0alue", b""),
        (b"Subject\n\nKey: v\xffalue", b""),
        (b"Subject\n\nLook-request: a \r", b"Look-request: a"),
        (b"Subject\n\t \r\nKey: v\n\n\t \r\n", b"Key: v"),
        (b"Subject\n\nKey: a\nKey: b\nKey: a", b"Key: a\nKey: b\nKey: a"),
        ("Subject\n\u{2003}\nKey-2: été\u{2003}\n\u{2003}".as_bytes(), "Key-2: été".as_bytes()),
    ];
    for (source, expected) in cases {
        assert_eq!(trailers(source).as_ref(), *expected, "{source:?}");
    }
}

#[test]
fn the_description_gives_way_before_whole_trailer_lines() {
    let title = "T".repeat(256);
    let words = format!("{}tail", "word ".repeat(204));
    assert_eq!(words.len(), 1024);
    let a = format!("Look-request: {}", "a".repeat(70));
    let b = format!("Look-landed: {}", "b".repeat(71));
    let pair = format!("{a}\n{b}\n");
    let required = format!("Change: {0}\nPull-request: {0}\nHead: {HEX}\nGoal: {0}", u64::MAX);

    let kept = format!("{}{a}", pair.repeat(19));
    let head = trailers(format!("Subject\n\n{kept}").as_bytes());
    let mut member = Member {
        title: title.as_bytes(),
        words: words.as_bytes(),
        head_trailers: &head,
        change: u64::MAX,
        pull: u64::MAX,
        head: HEAD,
        goal: Some(u64::MAX),
    };
    let squash = commit_message(&member, Kind::Squash);
    assert_eq!(squash.len(), 4096);
    let line = format!("{}word\n", "word ".repeat(13));
    let wrapped = format!("{}word wo…", line.repeat(5));
    assert_eq!(squash.as_ref(), format!("{title}\n\n{wrapped}\n\n{kept}\n{required}").as_bytes());

    let source = format!("Subject\n\n{}", pair.repeat(30));
    let head = trailers(source.as_bytes());
    assert!(head.len() <= TRAILER_BYTES);
    assert_eq!(head.as_ref(), pair.repeat(24).trim_end().as_bytes(), "only whole lines within the extractor bound");
    assert!(trailers(format!("{source}Prose after the bound.").as_bytes()).is_empty());
    member.head_trailers = &head;
    let squash = commit_message(&member, Kind::Squash);
    assert!(squash.len() <= 4096);
    assert_eq!(squash.as_ref(), format!("{title}\n\n{}{a}\n{required}", pair.repeat(21)).as_bytes());
    assert!(names_head(&squash, HEAD));

    let words = "é".repeat(512);
    for (description_room, paragraph) in [(8, "\n\néé…"), (4, "")] {
        let prefix = "Look-request: ";
        let value_bytes = 4096 - title.len() - required.len() - prefix.len() - 5 - description_room;
        let line = format!("{prefix}{}", "a".repeat(value_bytes));
        let head = trailers(format!("Subject\n\n{line}").as_bytes());
        let member = Member { words: words.as_bytes(), head_trailers: &head, ..member };
        assert_eq!(
            commit_message(&member, Kind::Squash).as_ref(),
            format!("{title}{paragraph}\n\n{line}\n{required}").as_bytes(),
            "{description_room} bytes left for the description",
        );
    }
}
