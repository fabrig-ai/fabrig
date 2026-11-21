//! Git and agent-output parsers over text (qa/README.md, 6).
use temper_qa_tool::{
    faces::{Commit, parse_commits, trailers},
    runner::parse_tokens,
    testing::id,
    watch::parse_landing,
};

#[test]
fn git_text_parses_parents_final_trailers_and_subjects_from_the_end() {
    let text = format!("{}\n{} {}\n{} {} {}\n", id(0), id(1), id(0), id(2), id(0), id(1));
    assert_eq!(
        parse_commits(&text).expect("rev-list"),
        vec![
            Commit { name: id(0), parents: vec![] },
            Commit { name: id(1), parents: vec![id(0)] },
            Commit { name: id(2), parents: vec![id(0), id(1)] },
        ]
    );
    assert!(parse_commits("not-a-commit\n").is_err());
    assert_eq!(parse_commits("").expect("empty").len(), 0);
    let subject = "Merge pull request 'Title (x)' (#12) from temper5/g7 into main";
    let landing = parse_landing(subject, "main").expect("goal subject");
    assert_eq!((landing.goal, landing.pull, landing.title.as_str()), (7, Some(12), "Title (x)"));
    let hostile_title =
        "Merge pull request 'Title ' (#9) from someone/g2 into main (x)' (#12) from temper5/g7 into main";
    assert_eq!(
        parse_landing(hostile_title, "main").expect("last suffix").title,
        "Title ' (#9) from someone/g2 into main (x)"
    );
    for message in [
        "Merge pull request 'Change' (#12) from temper5/c9 into temper5/g7",
        "Merge branch 'main' into temper5/g7",
        "Merge pull request 'Change' (#12) from temper5/g7 into temper5/g8",
        "Ordinary subject\n\nGoal: 7\n\nNot a trailer",
        "Ordinary subject\n\nGoal: 7\nnot a trailer",
    ] {
        assert!(parse_landing(message, "main").is_none(), "{message}");
    }
    let message = format!("{subject}\n\nGoal: 8\nPull-request: 19\nQA: work\n");
    let landing = parse_landing(&message, "main").expect("trailers take precedence");
    assert_eq!((landing.goal, landing.pull), (8, Some(19)));
    assert_eq!(trailers(&message).last(), Some(&("QA".into(), "work".into())));
    assert_eq!(parse_landing("A merge\n \nGoal: 7\nPull-request: 12\n", "main").expect("blank separator").goal, 7);
    assert!(parse_landing("A merge\n\nGoal: +7\n", "main").is_none());
    assert_eq!(parse_tokens("tokens 2\nbody\ntokens 1234\n\n"), Some(1234));
    for text in ["tokens +1", "tokens -1", "tokens 1\nlast words", "", "tokens 1.0"] {
        assert_eq!(parse_tokens(text), None);
    }
}
