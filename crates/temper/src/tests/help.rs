use crate::{Command, configure_with, help};
use std::{fs, os::unix::ffi::OsStrExt};
use temper_client_tools::jig::call::Parsed;

fn fenced_lines(text: &str) -> Vec<&str> {
    let mut fenced = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if fenced {
            lines.push(line);
        }
    }
    lines
}

#[test]
fn every_ctl_line_in_the_topics_parses() {
    let mut checked = 0;
    for topic in &temper_docs::TOPICS {
        for line in fenced_lines(topic.text) {
            let words = line.split_ascii_whitespace().take_while(|word| !word.starts_with('#')).collect::<Vec<_>>();
            if words.get(0..2) != Some(&["temper", "ctl"]) {
                continue;
            }
            let arguments = words[2..].iter().map(|word| word.as_bytes()).collect::<Vec<_>>();
            let (_, parsed) =
                crate::ctl::parse(&arguments).unwrap_or_else(|why| panic!("{}: {line}: {why}", topic.name));
            assert!(matches!(parsed, Parsed::Call { .. } | Parsed::File { .. }), "{}: {line}: a tool", topic.name);
            checked += 1;
        }
    }
    assert!(checked > 0, "checked at least one ctl line");
}

fn configured(words: &[&str]) -> help::Request {
    let words = words.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
    let Command::Help { request } = configure_with(&words, &[], b"/unused").expect("help arguments") else {
        panic!("help command")
    };
    request
}

#[test]
fn help_lists_topics_before_init_with_no_environment_or_configuration() {
    let directory = std::env::temp_dir().join(format!("temper-help-{}", std::process::id()));
    fs::create_dir(&directory).expect("empty working directory");
    assert_eq!(fs::read_dir(&directory).expect("working directory").count(), 0);
    let command = configure_with(&[Box::from(b"help".as_slice())], &[], directory.as_os_str().as_bytes())
        .expect("help before init");
    let Command::Help { request } = command else { panic!("help command") };
    let answer = help::answer(&request).expect("topic list");
    let topic = temper_docs::find(b"box").expect("box topic");
    assert!(answer.starts_with("Walks\n"));
    assert!(answer.contains(&format!("  box: {}. {}\n", topic.title, topic.summary)));
    assert!(answer.ends_with("\ntemper help NAME prints a topic; temper help --search WORDS... finds its sections.\n"));
    assert_eq!(fs::read_dir(&directory).expect("still empty working directory").count(), 0);
    fs::remove_dir(directory).expect("remove empty working directory");
}

#[test]
fn help_prints_a_whole_topic_and_names_an_unknown_one() {
    assert_eq!(
        help::answer(&configured(&["help", "server"])).expect("server topic").as_bytes(),
        include_bytes!("../../../../docs/help/server.md")
    );
    assert_eq!(
        help::answer(&configured(&["help", "nope"])),
        Err("temper: no topic nope; temper help lists them\n".into())
    );
}

#[test]
fn help_search_names_the_server_section_or_says_nothing_matches() {
    let text = help::answer(&configured(&["help", "--search", "WireGuard"])).expect("WireGuard section");
    assert!(text.lines().any(|line| line.starts_with("server: 7. The workers' host over WireGuard: ")), "{text}");
    assert!(text.contains("WireGuard"), "{text}");
    assert_eq!(
        help::answer(&configured(&["help", "--search", "zzzznotadocword"])),
        Err("temper: nothing matches\n".into())
    );
}

#[test]
fn help_search_joins_words_and_prints_each_search_refusal() {
    assert_eq!(
        configured(&["help", "--search", "owned", "Forgejo"]),
        help::Request::Search(Box::from(b"owned Forgejo".as_slice()))
    );
    for (words, error) in [
        (" \t", "nothing to search for"),
        ("a a a a a a a a a", "at most 8 words"),
        ("123456789012345678901234567890123", "a word is at most 32 bytes"),
    ] {
        assert_eq!(
            help::answer(&help::Request::Search(Box::from(words.as_bytes()))),
            Err(format!("temper: {error}\n"))
        );
    }
}

#[test]
fn help_requires_one_topic_or_search_words() {
    for words in [vec!["help", "--search"], vec!["help", "a", "b"], vec!["help", "--unknown"]] {
        let words = words.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
        assert_eq!(
            configure_with(&words, &[], b"/unused"),
            Err(format!("temper: invalid arguments\n{}", crate::usage()))
        );
    }
}
