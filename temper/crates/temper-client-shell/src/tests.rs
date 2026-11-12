use super::{CLIENT, tui_argument};

#[test]
fn the_bare_form_takes_one_loopback_address() {
    for (words, expected) in [
        (vec![], Ok(None)),
        (vec!["http://127.0.0.1:8367"], Ok(Some(b"http://127.0.0.1:8367".as_slice()))),
        (vec!["frob"], Err((2, "temper: not an address: frob\nusage: temper [ADDRESS]\n".into()))),
        (vec!["http://127.0.0.1:8367", "extra"], Err((2, "temper: invalid arguments\nusage: temper [ADDRESS]\n".into()))),
        (vec!["http://192.0.2.1:8367"], Err((2, "temper: the engine's address must be on loopback\nusage: temper [ADDRESS]\n".into()))),
        (vec!["http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"], Err((2, "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\nusage: temper [ADDRESS]\n".into()))),
    ] {
        let arguments: Vec<Box<[u8]>> = words.iter().map(|word| word.as_bytes().into()).collect();
        assert_eq!(tui_argument(&arguments), expected, "{words:?}");
    }
}

#[test]
fn the_arguments_document_is_the_clients_table() {
    let rendered = CLIENT.arguments_document();
    let path = skein_scratch::paths::tree().join("docs/design/cli/temper-args.md");
    if temper_test_env::rewrite() {
        std::fs::write(path, rendered).expect("write arguments document");
    } else {
        assert_eq!(
            std::fs::read_to_string(path).expect("arguments document"),
            rendered,
            "regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
        );
    }
}

#[test]
fn the_tui_refusals_explain_the_terminal_and_sign_in() {
    use temper_client_shell_tui::start::{Refused, terminal_refusal};
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
        assert_eq!(why.line(CLIENT.binary), line, "{why:?}");
    }
    assert_eq!(terminal_refusal(CLIENT.binary, false), "temper: no command; temper --help lists them\n");
    assert_eq!(terminal_refusal(CLIENT.binary, true), Refused::NotATerminal.line(CLIENT.binary));
}
#[test]
fn tui_recording_variable_is_internal_and_read() {
    for reach in [skein_shell::Reach::Words(&[]), skein_shell::Reach::Words(&["box"])] {
        let tui = CLIENT.commands.iter().find(|command| command.reach == reach).expect("a client TUI command");
        let record = tui
            .variables
            .iter()
            .find(|variable| variable.name == "TEMPER_INTERNAL_RECORD")
            .expect("TUI commands must declare the recording variable");
        assert_eq!(record.set, skein_shell::Set::Internal, "recording variable is internal");
        assert_eq!(record.way, skein_shell::Way::Reads, "recording variable is read");
    }
}
