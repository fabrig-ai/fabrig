use super::{CLIENT, Command, configure};
use temper_shell::table;

const IDENTITY: &str = "temper 0.0.0 (0123456789ab)";
const LOOPBACK: &str = "http://127.0.0.1:8367";
const HTTPS: &str = "https://dev.fabrig.ai";

fn arguments(words: &[&str]) -> Box<[Box<[u8]>]> {
    words.iter().map(|word| word.as_bytes().into()).collect()
}

fn invocation(arguments: &[Box<[u8]>]) -> Result<Command<'_>, (u8, String)> {
    configure(CLIENT.reach(b"temper", arguments), IDENTITY)
}

fn printed(words: &[&str]) -> String {
    let arguments = arguments(words);
    let Command::Print(text) = invocation(&arguments).expect("a print command") else {
        panic!("{words:?} did not select a print command");
    };
    text
}

#[test]
fn overview_aliases_list_only_public_commands() {
    let page = printed(&["--help"]);
    assert_eq!(page, CLIENT.help());
    for words in [["-h"], ["help"]] {
        assert_eq!(printed(&words), page, "{words:?}");
    }
    let (_, commands) = page.split_once("\ncommands:\n").expect("commands section");
    let names: Vec<_> = commands
        .split("\n\n")
        .next()
        .expect("command lines")
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(names, ["box", "help"]);
    assert!(page.contains("  temper [ADDRESS]\n"));
}

#[test]
fn command_help_prints_its_page_and_internal_notice() {
    for (words, row, usage, internal) in [
        (&["ctl", "--help"][..], &table::CTL, "temper ctl", true),
        (&["box", "--help"][..], &super::box_binary::BOX, "temper box [ARGUMENT...]", false),
        (
            &["internal", "fetch-box", "--help"][..],
            &super::box_binary::FETCH,
            "temper internal fetch-box [--cache DIR]",
            true,
        ),
    ] {
        let page = printed(words);
        assert_eq!(page, CLIENT.command_help(row), "{words:?}");
        assert!(page.contains(usage), "{words:?}");
        assert_eq!(page.starts_with("internal:"), internal, "{words:?}");
    }
}

#[test]
fn help_prints_the_clients_commands_and_the_pages_index() {
    let commands = printed(&["help", "commands"]);
    assert_eq!(commands, table::commands_page(&CLIENT));
    for form in ["box [ARGUMENT...]", "internal fetch-box [--cache DIR]"] {
        assert!(commands.contains(form), "{form}");
    }
    let index = temper_shell::help::answer(CLIENT.binary, &temper_shell::help::Request::Index).expect("index");
    assert_eq!(printed(&["help", "pages"]), index);
}

#[test]
fn version_prints_the_supplied_identity_with_a_newline() {
    assert_eq!(printed(&["--version"]), "temper 0.0.0 (0123456789ab)\n");
}

#[test]
fn help_refuses_invalid_arguments_with_its_usage() {
    assert_eq!(
        invocation(&arguments(&["help", "-x"])),
        Err((2, "temper help: invalid arguments\nusage: temper help [NAME | pages | --search WORDS...]\n".into()))
    );
}

#[test]
fn the_bare_form_takes_one_engine_origin() {
    for (words, expected) in [(&[][..], None), (&[LOOPBACK][..], Some(LOOPBACK)), (&[HTTPS][..], Some(HTTPS))] {
        assert_eq!(invocation(&arguments(words)), Ok(Command::Tui(expected.map(str::as_bytes))), "{words:?}");
    }
}

#[test]
fn the_bare_form_refuses_extra_words_with_its_usage() {
    for words in [["--version", "extra"], [LOOPBACK, "extra"]] {
        assert_eq!(
            invocation(&arguments(&words)),
            Err((2, "temper: invalid arguments\nusage: temper [ADDRESS]\n".into())),
            "{words:?}"
        );
    }
}

#[test]
fn the_bare_form_explains_why_each_address_is_refused() {
    for (word, why) in [
        ("frob", "not an address: frob"),
        ("nope", "not an address: nope"),
        ("http://dev.fabrig.ai", "http:// reaches an engine on this machine alone: use https://NAME"),
        ("https://127.0.0.1", "https:// needs the deployment's name"),
        ("https://dev.fabrig.ai/x", "not an address: https://dev.fabrig.ai/x"),
        ("http://10.0.0.1:8367", "the engine's address must be on loopback"),
        ("http://192.0.2.1:8367", "the engine's address must be on loopback"),
        (
            "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "a sign-in link is never an argument: run temper http://HOST:PORT and paste it",
        ),
    ] {
        assert_eq!(
            invocation(&arguments(&[word])),
            Err((2, format!("temper: {why}\nusage: temper [ADDRESS]\n"))),
            "{word}"
        );
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
    use temper_client_shell_tui::start::Refused;
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
