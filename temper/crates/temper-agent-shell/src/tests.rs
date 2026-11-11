//! Dispatch without a process (temper/docs/design/cli/README.md, 7.3 and 10).
use super::{AGENT, Command, LIST, PROGRAM_CHILD, RELAY, RUN, configure};

#[test]
fn every_row_parses_and_the_rest_is_refused() {
    for (run_as, words, expected) in [
        ("temper-agent", vec![], Err((2, "temper-agent: no command; temper-agent --help lists them\n".into()))),
        ("temper-agent", vec!["--help"], Ok(Command::Print(AGENT.help()))),
        ("temper-agent", vec!["--version"], Ok(Command::Print(AGENT.version_line()))),
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
        ("temper-agent", vec!["--list", "--help"], Ok(Command::Print(AGENT.command_help(&LIST)))),
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
        ("temper-agent", vec!["--mcp-relay", "--help"], Ok(Command::Print(AGENT.command_help(&RELAY)))),
        ("temper-agent", vec!["--mcp-relay", "127.0.0.1:8080"], Ok(Command::Relay)),
        ("temper-agent", vec!["--mcp-relay"], Ok(Command::Relay)),
        ("temper-agent", vec!["--program-child", "--help"], Ok(Command::Print(AGENT.command_help(&PROGRAM_CHILD)))),
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
        (
            "/launchers/claude",
            vec!["--help"],
            Ok(Command::Claude { arguments: Box::new([Box::from(b"--help".as_slice())]) }),
        ),
    ] {
        let arguments: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
        assert_eq!(configure(AGENT.reach(run_as.as_bytes(), &arguments)), expected, "{run_as}: {words:?}");
    }
    let arguments = [Box::from(b"--frob".as_slice())];
    assert_eq!(configure(skein_shell::Reached::Command(&RUN, &arguments)), Err((2, AGENT.unknown(&arguments))));
    assert_eq!(configure(skein_shell::Reached::Unknown("no match\n".into())), Err((2, "no match\n".into())));
}

#[test]
fn the_version_line_names_the_binary_its_version_and_commit() {
    use temper_version::{COMMIT, VERSION};

    assert_eq!(AGENT.version_line(), format!("temper-agent {VERSION} ({COMMIT})\n"));
    let head = COMMIT.strip_suffix("-dirty").unwrap_or(COMMIT);
    assert!(COMMIT == "unknown" || (head.len() == 12 && head.bytes().all(|byte| byte.is_ascii_hexdigit())), "{COMMIT}");
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
