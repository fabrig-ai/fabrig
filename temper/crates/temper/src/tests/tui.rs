use crate::{
    Command, commands, configure_with,
    tui::{self, Refused},
};

#[test]
fn bare_temper_and_temper_address_parse_and_refuse() {
    for (arguments, expected) in [
        (vec![], Ok(Command::Tui { address: None })),
        (
            vec!["http://127.0.0.1:8367"],
            Ok(Command::Tui { address: Some(Box::from(b"http://127.0.0.1:8367".as_slice())) }),
        ),
        (
            vec!["http://127.0.0.1:8367", "http://127.0.0.1:9000"],
            Err("temper: invalid arguments\nusage: temper [ADDRESS]\n".into()),
        ),
        (vec!["bogus"], Err("temper: no command bogus; temper --help lists them\n".into())),
    ] {
        let bytes = arguments.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
        assert_eq!(configure_with(&bytes, &[], b"/"), expected, "{arguments:?}");
    }
    for (why, line) in [
        (Refused::NotATerminal, "temper: the TUI needs a terminal on stdin and stdout\n"),
        (Refused::NotAnAddress(Box::from(b"ARG".as_slice())), "temper: not an address: ARG\n"),
        (Refused::Link, "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\n"),
        (Refused::NotLoopback, "temper: the engine's address must be on loopback\n"),
        (
            Refused::Directory("/s/temper/tui: cannot open its files".into()),
            "temper: /s/temper/tui: cannot open its files\n",
        ),
    ] {
        assert_eq!(why.line("temper"), line, "{why:?}");
    }
    assert_eq!(tui::terminal_refusal("temper", false), commands::unknown(&[]));
    assert_eq!(tui::terminal_refusal("temper", false), "temper: no command; temper --help lists them\n");
    assert_eq!(tui::terminal_refusal("temper", true), Refused::NotATerminal.line("temper"));
    let row = commands::find(&[]).expect("TUI row");
    assert!(commands::row(row).starts_with("usage: temper [ADDRESS]\n"));
    assert!(commands::reference().contains("## temper [ADDRESS]\n\n`temper [ADDRESS]`\n"));
}
