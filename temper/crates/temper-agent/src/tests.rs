#[path = "../build/commit.rs"]
mod commit;

use super::{Command, LIST, PROGRAM_CHILD, RELAY, configure};
use temper_shell::{help::Request, rows};

#[test]
fn every_row_parses_and_the_rest_is_refused() {
    for (words, expected) in [
        (vec![], Err("temper-agent: no command; temper-agent --help lists them\n".into())),
        (vec!["--help"], Ok(Command::Overview)),
        (vec!["--version"], Ok(Command::Version)),
        (vec!["help"], Ok(Command::Help { request: Request::Index })),
        (vec!["help", "tui"], Ok(Command::Help { request: Request::Page(Box::from(b"tui".as_slice())) })),
        (
            vec!["help", "--search", "sign", "out"],
            Ok(Command::Help { request: Request::Search(Box::from(b"sign out".as_slice())) }),
        ),
        (vec!["help", "--help"], Ok(Command::Usage(&rows::HELP))),
        (
            vec!["help", "-x"],
            Err("temper-agent help: invalid arguments\nusage: temper-agent help [NAME | --search WORDS...]\n".into()),
        ),
        (vec!["--list", "a.json"], Ok(Command::List { configuration: "a.json".into() })),
        (vec!["--list", "--help"], Ok(Command::Usage(&LIST))),
        (vec!["--list"], Err("temper-agent --list: invalid arguments\nusage: temper-agent --list AGENT.json\n".into())),
        (
            vec!["--list", "a.json", "b.json"],
            Err("temper-agent --list: invalid arguments\nusage: temper-agent --list AGENT.json\n".into()),
        ),
        (vec!["--mcp-relay", "--help"], Ok(Command::Usage(&RELAY))),
        (
            vec!["--mcp-relay", "127.0.0.1:8080"],
            Err("temper-agent --mcp-relay: invalid arguments\nusage: temper-agent --mcp-relay 127.0.0.1:PORT\n".into()),
        ),
        (
            vec!["--mcp-relay"],
            Err("temper-agent --mcp-relay: invalid arguments\nusage: temper-agent --mcp-relay 127.0.0.1:PORT\n".into()),
        ),
        (vec!["--program-child", "--help"], Ok(Command::Usage(&PROGRAM_CHILD))),
        (
            vec!["--program-child"],
            Err("temper-agent --program-child: invalid arguments\nusage: temper-agent --program-child PARENT PROGRAM ARGUMENTS...\n".into()),
        ),
        (
            vec!["--program-child", "1", "program", "argument"],
            Err("temper-agent --program-child: invalid arguments\nusage: temper-agent --program-child PARENT PROGRAM ARGUMENTS...\n".into()),
        ),
        (vec!["a.json"], Ok(Command::Run { configuration: "a.json".into() })),
        (vec!["./help"], Ok(Command::Run { configuration: "./help".into() })),
        (vec!["http://a.json"], Ok(Command::Run { configuration: "http://a.json".into() })),
        (vec!["a.json", "b.json"], Err("temper-agent: invalid arguments\nusage: temper-agent AGENT.json\n".into())),
        (
            vec!["http://a.json", "b.json"],
            Err("temper-agent: invalid arguments\nusage: temper-agent AGENT.json\n".into()),
        ),
        (vec!["-x"], Err("temper-agent: no command -x; temper-agent --help lists them\n".into())),
        (
            vec!["--version", "extra"],
            Err("temper-agent: no command --version; temper-agent --help lists them\n".into()),
        ),
        (vec!["--help", "extra"], Err("temper-agent: no command --help; temper-agent --help lists them\n".into())),
    ] {
        let arguments: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
        assert_eq!(configure(&arguments), expected, "{words:?}");
    }
}

#[test]
fn the_commit_word_is_twelve_hex_digits_dirty_or_unknown() {
    const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
    for (head, dirty, expected) in [
        (Some(HEAD), false, "0123456789ab"),
        (Some(HEAD), true, "0123456789ab-dirty"),
        (None, false, "unknown"),
        (None, true, "unknown"),
        (Some("0123456789a"), false, "unknown"),
        (Some("0123456789ag"), false, "unknown"),
        (Some("0123456789aB"), false, "unknown"),
        (Some("0123456789abg"), false, "unknown"),
        (Some("0123456789ab"), false, "0123456789ab"),
    ] {
        assert_eq!(commit::word(head, dirty), expected, "head {head:?}, dirty {dirty}");
    }
}

#[test]
fn the_version_line_names_the_binary_its_version_and_commit() {
    let line = super::version();
    let prefix = format!("temper-agent {} (", env!("CARGO_PKG_VERSION"));
    let word = line.strip_prefix(&prefix).and_then(|tail| tail.strip_suffix(")\n")).expect("version line");
    let (head, dirty) = match word.strip_suffix("-dirty") {
        Some(head) => (head, true),
        None => (word, false),
    };
    let head = if head == "unknown" { None } else { Some(head) };
    assert_eq!(word, commit::word(head, dirty));
    assert_eq!(line, format!("{prefix}{})\n", env!("TEMPER_AGENT_COMMIT")));
}

#[test]
fn the_dump_directory_is_set_by_a_value_and_made_absolute() {
    use std::{ffi::OsString, path::PathBuf};
    let working = std::env::current_dir().expect("test working directory");
    for (row, value, expected) in [
        ("absent", None, None),
        ("empty", Some(OsString::new()), None),
        ("relative", Some(OsString::from("request-dumps")), Some(working.join("request-dumps"))),
        ("absolute", Some(OsString::from("/request-dumps")), Some(PathBuf::from("/request-dumps"))),
    ] {
        assert_eq!(super::dump_directory(value), expected, "{row}");
    }
    assert_eq!(super::DUMP_BOUND, 4_294_967_296);
}
