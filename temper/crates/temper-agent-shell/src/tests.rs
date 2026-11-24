//! Dispatch without a process (temper/docs/design/cli/README.md, 7.3 and 10).
use super::{AGENT, Command, LIST, LIST_WORDS, PROGRAM_CHILD, RELAY, RUN, configure, prefixed, run_read};

const VERSION: &str = "0.0.0";
const COMMIT: &str = "0123456789ab";

fn invocation(run_as: &str, words: &[&str]) -> Result<Command, (u8, String)> {
    let arguments: Vec<Box<[u8]>> = words.iter().map(|word| word.as_bytes().into()).collect();
    configure(AGENT.reach(run_as.as_bytes(), &arguments), VERSION, COMMIT)
}

fn printed(words: &[&str]) -> String {
    let Command::Print(text) = invocation(AGENT.binary, words).expect("a print command") else {
        panic!("{words:?} did not select a print command");
    };
    text
}

#[test]
fn overview_aliases_list_only_help_and_the_standard_flags() {
    let page = printed(&["--help"]);
    assert_eq!(page, AGENT.help());
    for words in [["-h"], ["help"]] {
        assert_eq!(printed(&words), page, "{words:?}");
    }
    let public: Vec<_> = AGENT
        .commands
        .iter()
        .filter(|command| command.set == skein_shell::Set::Public)
        .map(|command| command.reach)
        .collect();
    assert_eq!(public, [skein_shell::Reach::Words(&["help"])]);
    assert_eq!(AGENT.settings.len(), 0);
    assert_eq!(
        page.split("\n\n").find(|section| section.starts_with("flags:")),
        Some(
            "flags:\n  -h, --help                         Print this.\n  --version                          Print the name, version and commit."
        )
    );
}

#[test]
fn delegated_command_help_prints_an_internal_notice() {
    for (word, command) in [("--list", LIST), ("--mcp-relay", RELAY), ("--program-child", PROGRAM_CHILD)] {
        let page = printed(&[word, "--help"]);
        assert_eq!(page, AGENT.command_help(&command), "{word}");
        let notice = page.split("\n\n").next().expect("notice").split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            notice, "internal: not part of temper-agent's public command line; it may change without notice.",
            "{word}"
        );
    }
}

#[test]
fn the_version_line_names_the_binary_its_version_and_commit() {
    assert_eq!(printed(&["--version"]), "temper-agent 0.0.0 (0123456789ab)\n");
}

#[test]
fn every_row_parses_and_the_rest_is_refused() {
    for (run_as, words, expected) in [
        ("temper-agent", vec![], Err((2, "temper-agent: no command; temper-agent --help lists them\n".into()))),
        (
            "temper-agent",
            vec!["help", "tui"],
            Ok(Command::Print(temper_shell::table::help(&AGENT, &[Box::from(b"tui".as_slice())]).expect("page"))),
        ),
        (
            "temper-agent",
            vec!["help", "--search", "sign", "out"],
            Ok(Command::Print(
                temper_shell::table::help(
                    &AGENT,
                    &[Box::from(b"--search".as_slice()), Box::from(b"sign".as_slice()), Box::from(b"out".as_slice())],
                )
                .expect("search"),
            )),
        ),
        ("temper-agent", vec!["help", "commands"], Ok(Command::Print(temper_shell::table::commands_page(&AGENT)))),
        (
            "temper-agent",
            vec!["help", "pages"],
            Ok(Command::Print(temper_shell::table::help(&AGENT, &[Box::from(b"pages".as_slice())]).expect("index"))),
        ),
        ("temper-agent", vec!["help", "--help"], Ok(Command::Print(AGENT.command_help(&temper_shell::table::HELP)))),
        (
            "temper-agent",
            vec!["help", "-x"],
            Err((
                2,
                "temper-agent help: invalid arguments\nusage: temper-agent help [NAME | pages | --search WORDS...]\n"
                    .into(),
            )),
        ),
        ("temper-agent", vec!["--list", "a.json"], Ok(Command::List { configuration: "a.json".into() })),
        (
            "temper-agent",
            vec!["--list"],
            Err((2, "temper-agent --list: invalid arguments\nusage: temper-agent --list AGENT.json\n".into())),
        ),
        (
            "temper-agent",
            vec!["--list", "a.json", "b.json"],
            Err((2, "temper-agent --list: invalid arguments\nusage: temper-agent --list AGENT.json\n".into())),
        ),
        ("temper-agent", vec!["--mcp-relay", "127.0.0.1:8080"], Ok(Command::Relay)),
        ("temper-agent", vec!["--mcp-relay"], Ok(Command::Relay)),
        ("temper-agent", vec!["--program-child"], Ok(Command::ProgramChild)),
        ("temper-agent", vec!["--program-child", "1", "program", "argument"], Ok(Command::ProgramChild)),
        ("temper-agent", vec!["a.json"], Ok(Command::Run { configuration: "a.json".into() })),
        ("temper-agent", vec!["./help"], Ok(Command::Run { configuration: "./help".into() })),
        (
            "temper-agent",
            vec!["a.json", "b.json"],
            Err((2, "temper-agent: invalid arguments\nusage: temper-agent AGENT.json\n".into())),
        ),
        (
            "temper-agent",
            vec!["--frob"],
            Err((2, "temper-agent: no command --frob; temper-agent --help lists them\n".into())),
        ),
        (
            "temper-agent",
            vec!["--version", "extra"],
            Err((2, "temper-agent: no command --version; temper-agent --help lists them\n".into())),
        ),
        (
            "temper-agent",
            vec!["--help", "extra"],
            Err((2, "temper-agent: no command --help; temper-agent --help lists them\n".into())),
        ),
        ("/launchers/claude", vec![], Ok(Command::Claude { arguments: Box::new([]) })),
        (
            "/launchers/claude",
            vec!["--help"],
            Ok(Command::Claude { arguments: Box::new([Box::from(b"--help".as_slice())]) }),
        ),
    ] {
        assert_eq!(invocation(run_as, &words), expected, "{run_as}: {words:?}");
    }
    let arguments = [Box::from(b"--frob".as_slice())];
    assert_eq!(
        configure(skein_shell::Reached::Command(&RUN, &arguments), VERSION, COMMIT),
        Err((2, AGENT.unknown(&arguments)))
    );
    assert_eq!(
        configure(skein_shell::Reached::Unknown("no match\n".into()), VERSION, COMMIT),
        Err((2, "no match\n".into()))
    );
}

const ABSENT: &str = "configuration metadata: /agent/missing.json: file absent";

#[test]
fn a_missing_run_document_names_smith_then_the_agent() {
    let mut errors = Vec::new();
    let refusal = run_read(Err(ABSENT.into()), AGENT.binary, &mut errors).err().expect("read refusal");
    errors.extend_from_slice(refusal.as_bytes());
    assert_eq!(errors, b"smith: configuration metadata: /agent/missing.json: file absent\ntemper-agent: configuration metadata: /agent/missing.json: file absent\n");
}

#[test]
fn a_missing_list_document_names_only_the_list_command() {
    assert_eq!(
        prefixed(LIST_WORDS, ABSENT),
        "temper-agent --list: configuration metadata: /agent/missing.json: file absent\n"
    );
}

#[test]
fn the_dump_directory_is_set_by_a_value_and_made_absolute() {
    use std::path::PathBuf;
    let working = Ok(PathBuf::from("/agent"));
    for (row, value, expected) in [
        ("absent", None, None),
        ("empty", Some(""), None),
        ("relative", Some("request-dumps"), Some(PathBuf::from("/agent/request-dumps"))),
        ("absolute", Some("/request-dumps"), Some(PathBuf::from("/request-dumps"))),
    ] {
        let environment: Vec<temper_shell::directories::EnvironmentEntry> = value
            .into_iter()
            .map(|value| (Box::from(b"TEMPER_INTERNAL_DUMP_LLM".as_slice()), Box::from(value.as_bytes())))
            .collect();
        assert_eq!(super::dump_directory(&environment, &working), Ok(expected), "{row}");
    }
    assert_eq!(super::DUMP_BOUND, 4_294_967_296);
}
