use super::{COMMANDS, Command, NAME, configure};
use temper_shell::{commands::help, help::Request, rows};

#[test]
fn every_row_parses_and_the_rest_is_refused() {
    for (words, expected) in [
        (vec![], Ok(Command::Tui { address: None })),
        (
            vec!["http://127.0.0.1:8367"],
            Ok(Command::Tui { address: Some(Box::from(b"http://127.0.0.1:8367".as_slice())) }),
        ),
        (
            vec!["http://127.0.0.1:8367", "http://127.0.0.1:8368"],
            Err("temper-client: invalid arguments\nusage: temper-client [ADDRESS]\n".into()),
        ),
        (vec!["bogus"], Err("temper-client: no command bogus; temper-client --help lists them\n".into())),
        (vec!["ctl", "DIR", "me"], Ok(Command::Ctl { arguments: vec!["DIR".into(), "me".into()] })),
        (vec!["ctl", "--help"], Ok(Command::Usage(&rows::CTL))),
        (vec!["help"], Ok(Command::Help { request: Request::Index })),
        (vec!["help", "tui"], Ok(Command::Help { request: Request::Page(Box::from(b"tui".as_slice())) })),
        (
            vec!["help", "--search", "sign", "out"],
            Ok(Command::Help { request: Request::Search(Box::from(b"sign out".as_slice())) }),
        ),
        (
            vec!["help", "-x"],
            Err("temper-client help: invalid arguments\nusage: temper-client help [NAME | --search WORDS...]\n".into()),
        ),
        (vec!["--help"], Ok(Command::Overview)),
        (vec!["--version"], Ok(Command::Version)),
        (
            vec!["--version", "extra"],
            Err("temper-client: no command --version; temper-client --help lists them\n".into()),
        ),
        (vec!["box"], Err("temper-client: no command box; temper-client --help lists them\n".into())),
        (vec!["serve"], Err("temper-client: no command serve; temper-client --help lists them\n".into())),
        (vec!["init"], Err("temper-client: no command init; temper-client --help lists them\n".into())),
        (
            vec!["open", "DIR", "http://127.0.0.1:8367"],
            Err("temper-client: no command open; temper-client --help lists them\n".into()),
        ),
        (
            vec!["internal", "agent", "A.json"],
            Err("temper-client: no command internal; temper-client --help lists them\n".into()),
        ),
        (vec!["fake", "git"], Err("temper-client: no command fake; temper-client --help lists them\n".into())),
        (
            vec!["program-login", "NAME"],
            Err("temper-client: no command program-login; temper-client --help lists them\n".into()),
        ),
        (vec!["forge", "fetch", "1"], Err("temper-client: no command forge; temper-client --help lists them\n".into())),
    ] {
        let arguments: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
        assert_eq!(configure(&arguments), expected, "{words:?}");
    }
}

#[test]
fn the_tui_refusals_explain_the_terminal_and_sign_in() {
    use temper_client_shell_tui::start::{Refused, terminal_refusal};
    for (why, line) in [
        (Refused::NotATerminal, "temper-client: the TUI needs a terminal on stdin and stdout\n"),
        (Refused::NotAnAddress(Box::from(b"ARG".as_slice())), "temper-client: not an address: ARG\n"),
        (
            Refused::Link,
            "temper-client: a sign-in link is never an argument: run temper-client http://HOST:PORT and paste it\n",
        ),
        (Refused::NotLoopback, "temper-client: the engine's address must be on loopback\n"),
        (
            Refused::Directory("/s/temper/tui: cannot open its files".into()),
            "temper-client: /s/temper/tui: cannot open its files\n",
        ),
    ] {
        assert_eq!(why.line(NAME), line, "{why:?}");
    }
    assert_eq!(terminal_refusal(NAME, false), "temper-client: no command; temper-client --help lists them\n");
    assert_eq!(terminal_refusal(NAME, true), Refused::NotATerminal.line(NAME));
}

#[test]
fn the_help_lists_the_clients_rows() {
    let text = help(NAME, &COMMANDS);
    for (line, present) in [
        ("  temper-client [ADDRESS]\n", true),
        ("  temper-client ctl DIR GROUP VERB [ARGUMENT...] [--json]\n", true),
        ("  temper-client help [NAME | --search WORDS...]\n", true),
        ("  temper-client box ", false),
        ("  temper-client serve ", false),
        ("  temper-client open ", false),
    ] {
        assert_eq!(text.contains(line), present, "{line:?}: {text}");
    }
}

#[test]
fn the_version_line_names_the_binary_its_version_and_commit() {
    let line = super::version();
    assert_eq!(line, format!("temper-client {} ({})\n", temper_version::VERSION, temper_version::COMMIT));
}
