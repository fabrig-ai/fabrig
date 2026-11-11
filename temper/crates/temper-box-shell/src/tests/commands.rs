use crate::{Command, TABLE, TREE, configure_with, fake};
use skein_shell::{Reach, Set};

fn bytes(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| Box::from(word.as_bytes())).collect()
}
fn configured(words: &[&str]) -> Result<Command, String> {
    configure_with(&bytes(words), &[(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))], b"/")
}
fn command_words(command: &Command) -> Vec<&str> {
    match command {
        Command::Version => vec!["--version"],
        Command::Versions => vec!["versions"],
        Command::Usage(row) => match row.reach {
            Reach::Words(words) => words.to_vec(),
            Reach::RunAs(name) => vec![name],
        },
        Command::Init { .. } => vec!["init"],
        Command::Serve { boxed: false, .. } => vec!["serve"],
        Command::Serve { boxed: true, .. } => vec!["internal", "serve"],
        Command::Box { .. } | Command::BoxFakeBackends { .. } => vec![],
        Command::ProgramLogin { .. } => vec!["program-login"],
        Command::Agent { .. } => vec!["internal", "agent"],
        Command::AgentList { .. } => vec!["internal", "agent-list"],
        Command::TuiFront { .. } => vec!["internal", "tui"],
        Command::Open { .. } => vec!["open"],
        Command::Ctl { .. } => vec!["ctl"],
        Command::Overview | Command::Help { .. } => vec!["help"],
        Command::Fake { arguments } => vec![
            "fake",
            match arguments.kind {
                fake::Kind::Llm => "llm",
                fake::Kind::Forge => "forge",
                fake::Kind::Oauth => "oauth",
            },
        ],
        Command::FakeCheck { .. } => vec!["fake", "forge", "check"],
        Command::FakeGit { .. } => vec!["fake", "git"],
        Command::ForgeChild { .. } => vec!["internal", "forge-child"],
        Command::Forge { arguments, .. } => vec!["forge", arguments[0].to_str().expect("forge verb")],
        Command::Claude { .. } => vec!["claude"],
        Command::Relay => vec!["--mcp-relay"],
        Command::ProgramChild => vec!["--program-child"],
    }
}

#[test]
fn every_row_parses_from_an_example() {
    let examples: &[(&[&str], &[&str])] = &[
        (&["help"], &["help", "pages"]),
        (&["versions"], &["versions"]),
        (&["init"], &["init"]),
        (&["serve"], &["serve"]),
        (&[], &[]),
        (&[], &["--echo", "steps", "--no-tui"]),
        (&[], &["--fake-backends", "--bundle", "/bundle", "--echo", "steps", "--no-tui"]),
        (&["program-login"], &["program-login", "source"]),
        (&["forge", "remove"], &["forge", "remove", "1"]),
        (&["forge", "fetch"], &["forge", "fetch", "1"]),
        (&["forge", "prepare"], &["forge", "prepare", "1"]),
        (&["forge", "run"], &["forge", "run", "1", "forgejo"]),
        (&["forge", "sign-in"], &["forge", "sign-in", "1"]),
        (&["forge", "source"], &["forge", "source", "1"]),
        (&["ctl"], &["ctl", "DIR", "task", "show", "1"]),
        (&["ctl"], &["ctl", "DIR", "me"]),
        (&["ctl"], &["ctl", "DIR", "--call", "FILE"]),
        (&["ctl"], &["ctl", "help", "task", "show"]),
        (&["open"], &["open", "/client", "http://127.0.0.1:7000"]),
        (&["internal", "agent"], &["internal", "agent", "agent.json"]),
        (&["internal", "agent-list"], &["internal", "agent-list", "agent.json"]),
        (&["internal", "agent-list"], &["--list", "internal", "agent-list", "agent.json"]),
        (&["internal", "serve"], &["internal", "serve", "--box"]),
        (&["internal", "forge-child"], &["internal", "forge-child", "--parent", "1", "--signal", "kill", "--", "p"]),
        (&["internal", "tui"], &["internal", "tui", "--box", "http://127.0.0.1:8367"]),
        (&["fake", "llm"], &["fake", "llm"]),
        (&["fake", "llm"], &["fake", "llm", "--document", "llm.json"]),
        (&["fake", "forge"], &["fake", "forge", "1"]),
        (&["fake", "forge"], &["fake", "forge", "--document", "forge.json"]),
        (&["fake", "forge", "check"], &["fake", "forge", "check", "1", "observations", "store"]),
        (&["fake", "oauth"], &["fake", "oauth", "--document", "oauth.json"]),
        (&["fake", "git"], &["fake", "git", "status"]),
        (&["--mcp-relay"], &["--mcp-relay"]),
        (&["--program-child"], &["--program-child"]),
    ];
    for &(words, example) in examples {
        let command = configured(example).expect("example parses");
        assert_eq!(command_words(&command), words, "{example:?}: dispatched command");
    }
    for (run_as, expected) in [("/demo/bin/git", vec!["fake", "git"]), ("/bin/claude", vec!["claude"])] {
        let arguments = bytes(&["--help", "status", "--version"]);
        let command = crate::configure_as(run_as.as_bytes(), &arguments, &[], b"/").expect("delegated grammar");
        assert_eq!(command_words(&command), expected);
        let (Command::FakeGit { arguments: passed } | Command::Claude { arguments: passed, .. }) = command else {
            panic!("delegated command")
        };
        assert_eq!(passed.as_ref(), arguments, "all delegated arguments are preserved");
    }
    assert!(matches!(configured(&["init", "--box"]).expect("boxed init"), Command::Init { boxed: true, .. }));
    assert!(matches!(configured(&["--no-tui"]).expect("no TUI"), Command::Box { no_tui: true, .. }));
    assert!(matches!(configured(&[]).expect("bare box"), Command::Box { no_tui: false, .. }));
    for words in [
        vec!["--no-tui", "--no-tui"],
        vec!["--bundle", "/bundle"],
        vec!["internal", "tui"],
        vec!["internal", "tui", "--box"],
    ] {
        assert!(configured(&words).is_err(), "refused {words:?}");
    }
}

#[test]
fn each_row_answers_its_help() {
    for command in TABLE.commands {
        let Reach::Words(words) = command.reach else { continue };
        if words.is_empty() {
            continue;
        }
        let mut arguments = words.to_vec();
        arguments.push("--help");
        assert_eq!(configure_with(&bytes(&arguments), &[], b"/unused"), Ok(Command::Usage(command)));
        assert_eq!(TABLE.command_help(command).starts_with("internal:"), command.set == Set::Internal);
    }
}

#[test]
fn temper_help_lists_only_public_commands() {
    let text = TABLE.help();
    let (_, commands) = text.split_once("\ncommands:\n").expect("commands");
    let commands = commands.split("\n\n").next().expect("command lines");
    let names: Vec<_> = commands
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(names, ["init", "serve", "program-login", "help", "versions"]);
    assert!(text.contains("temper-box [--config DIR]"));
}

#[test]
fn spawned_commands_answer_only_under_internal() {
    for (words, error) in [
        (vec!["agent", "X"], "temper-box: no command agent; temper-box --help lists them\n"),
        (vec!["agent-list", "X"], "temper-box: no command agent-list; temper-box --help lists them\n"),
        (vec!["serve", "--box"], "temper-box serve: invalid arguments\nusage: temper-box serve [DIRECTORIES]\n"),
        (vec!["forge", "child"], "temper-box: no command forge child; temper-box --help lists them\n"),
    ] {
        assert_eq!(configured(&words), Err(error.into()));
    }
    assert_eq!(configured(&["internal", "agent", "X"]), Ok(Command::Agent { configuration: "X".into() }));
    let expected = Ok(Command::AgentList { configuration: "X".into() });
    assert_eq!(configured(&["internal", "agent-list", "X"]), expected);
    assert_eq!(configured(&["--list", "internal", "agent-list", "X"]), expected);
    for words in [vec!["internal", "agent-list"], vec!["internal", "agent-list", "X", "Y"], vec!["internal", "serve"]] {
        assert!(configured(&words).is_err(), "refused {words:?}");
    }
    assert!(matches!(
        configured(&["internal", "serve", "--box"]).expect("box step"),
        Command::Serve { boxed: true, .. }
    ));
}

#[test]
fn refusals_name_the_command_and_its_forms() {
    for (words, error) in [
        (vec!["bogus"], "temper-box: no command bogus; temper-box --help lists them\n"),
        (vec!["box"], "temper-box: no command box; temper-box --help lists them\n"),
        (vec!["http://127.0.0.1:8367"], "temper-box: no command http://127.0.0.1:8367; temper-box --help lists them\n"),
        (vec!["forge", "bogus", "1"], "temper-box: no command forge bogus; temper-box --help lists them\n"),
        (
            vec!["forge", "run", "1"],
            "temper-box forge run: invalid arguments\nusage: temper-box forge run [DIRECTORIES] FORGE forgejo|runner\n",
        ),
        (vec!["init", "extra"], "temper-box init: invalid arguments\nusage: temper-box init [DIRECTORIES]\n"),
    ] {
        assert_eq!(configured(&words), Err(error.into()));
    }
}

#[test]
fn rows_are_within_bounds() {
    for (index, command) in TABLE.commands.iter().enumerate() {
        assert!(command.line.len() <= 120, "{:?}: line", command.reach);
        assert!(command.notes.len() <= 1024, "{:?}: notes", command.reach);
        for argument in command.arguments {
            assert!(argument.line.len() <= 200, "{:?}: {}", command.reach, argument.shown);
        }
        for other in &TABLE.commands[index + 1..] {
            assert_ne!(command.reach, other.reach, "unique reach");
        }
    }
}

#[test]
fn the_page_commands_is_what_the_table_renders() {
    let rendered = temper_shell::table::commands_reference(&[
        &temper_engine_shell::ENGINE,
        &temper_worker_shell::WORKER,
        &temper_agent_shell::AGENT,
        &TABLE,
    ]);
    let write = temper_test_env::rewrite();
    if write {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/help/reference/commands.md");
        std::fs::write(path, &rendered).expect("write commands reference");
        return;
    }
    assert_eq!(
        include_str!("../../../../docs/help/reference/commands.md"),
        rendered,
        "the page should be:\n{rendered}"
    );
}

#[test]
fn the_page_host_tools_is_what_the_declarations_render() {
    let mut measured = skein_lib::Writer::new(temper_docs::PAGE_BYTES);
    temper_engine_smith::reference(&mut measured).expect("host-tools fits PAGE_BYTES");
    let mut out = skein_lib::Writer::new(measured.written());
    temper_engine_smith::reference(&mut out).expect("host-tools fits measured length");
    let rendered = String::from_utf8(out.finish().into_vec()).expect("reference UTF-8");
    if temper_test_env::rewrite() {
        let path = skein_scratch::paths::tree().join("docs/help/reference/host-tools.md");
        std::fs::write(path, &rendered).expect("write host-tools reference");
        return;
    }
    assert_eq!(
        include_str!("../../../../docs/help/reference/host-tools.md"),
        rendered,
        "the page should be:\n{rendered}"
    );
}

#[test]
fn the_page_see_is_what_the_addresses_render() {
    let mut measured = skein_lib::Writer::new(temper_docs::PAGE_BYTES);
    temper_engine_smith::see_reference(&mut measured).expect("see fits PAGE_BYTES");
    let mut out = skein_lib::Writer::new(measured.written());
    temper_engine_smith::see_reference(&mut out).expect("see fits measured length");
    let rendered = String::from_utf8(out.finish().into_vec()).expect("reference UTF-8");
    if temper_test_env::rewrite() {
        let path = skein_scratch::paths::tree().join("docs/help/reference/see.md");
        std::fs::write(path, &rendered).expect("write see reference");
        return;
    }
    assert_eq!(include_str!("../../../../docs/help/reference/see.md"), rendered, "the page should be:\n{rendered}");
}

#[test]
fn every_ctl_group_page_is_what_the_table_renders() {
    use temper_client_tools::{help, table::GROUPS};
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/help/reference");
    let expected = GROUPS
        .iter()
        .map(|group| format!("ctl-{}", core::str::from_utf8(group.name).expect("group name")))
        .collect::<Vec<_>>();
    let mut files = std::fs::read_dir(&directory)
        .expect("reference directory")
        .map(|entry| entry.expect("reference file").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .map(|path| path.file_stem().expect("page name").to_str().expect("ASCII name").to_owned())
        .filter(|name| name.starts_with("ctl-"))
        .collect::<Vec<_>>();
    files.sort();
    let mut sorted = expected.clone();
    sorted.sort();
    assert_eq!(files, sorted, "the group pages and GROUPS are the same set");
    let compiled = temper_docs::PAGES
        .iter()
        .filter(|page| page.name.starts_with("ctl-"))
        .map(|page| page.name)
        .collect::<Vec<_>>();
    assert_eq!(compiled, expected, "compiled group pages follow GROUPS' order");
    let commands = temper_docs::PAGES.iter().position(|page| page.name == "commands").expect("commands page");
    for (index, (group, name)) in GROUPS.iter().zip(&expected).enumerate() {
        let page = &temper_docs::PAGES[commands + 1 + index];
        assert_eq!(page.name, name, "group pages immediately follow commands");
        assert_eq!(page.kind, temper_docs::Kind::Reference, "{name}");
        let mut measured =
            skein_lib::Writer::new(usize::try_from(temper_client_tools::limits::TEXT).expect("text bound"));
        help::reference(group, &mut measured).expect("reference within renderer's text bound");
        let mut out = skein_lib::Writer::new(measured.written());
        help::reference(group, &mut out).expect("reference within renderer's text bound");
        let rendered = String::from_utf8(out.finish().into_vec()).expect("reference UTF-8");
        assert!(rendered.len() <= temper_docs::PAGE_BYTES, "{name}: {} bytes", rendered.len());
        let path = directory.join(name).with_extension("md");
        if temper_test_env::rewrite() {
            std::fs::write(path, &rendered).expect("write group reference");
        } else {
            assert_eq!(
                std::fs::read_to_string(path).expect("group page"),
                rendered,
                "{name} should be:\n{rendered}; regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
            );
            assert_eq!(page.text, rendered, "compiled {name} should be:\n{rendered}");
        }
    }
}

#[test]
fn bundle_build_variable_is_internal_in_both_tables() {
    for table in [TABLE, temper_engine_shell::ENGINE] {
        let bundle = table.variables.iter().find(|variable| variable.name == "TEMPER_INTERNAL_BUNDLE");
        let bundle = bundle.unwrap_or_else(|| panic!("{} must declare the bundle's build variable", table.binary));
        assert_eq!(bundle.set, Set::Internal, "{}: build variable is internal", table.binary);
        assert_eq!(bundle.way, skein_shell::Way::Reads, "{}: build variable is read", table.binary);
    }
}

#[test]
fn arguments_and_environment_documents_are_the_tables() {
    let documents = [
        ("temper-box-args.md", TABLE.arguments_document()),
        ("temper-box-env.md", TABLE.environment_document(&TREE)),
        ("temper-engine-env.md", temper_engine_shell::ENGINE.environment_document(&TREE)),
        ("temper-worker-args.md", temper_worker_shell::WORKER.arguments_document()),
        ("temper-worker-env.md", temper_worker_shell::WORKER.environment_document(&TREE)),
        ("temper-agent-args.md", temper_agent_shell::AGENT.arguments_document()),
        ("temper-agent-env.md", temper_agent_shell::AGENT.environment_document(&TREE)),
        ("common-env.md", skein_shell::Table::common_document("temper", &TREE)),
    ];
    let write = temper_test_env::rewrite();
    for (name, rendered) in documents {
        let path = skein_scratch::paths::tree().join("docs/design/cli").join(name);
        if write {
            std::fs::write(path, rendered).expect("write table document");
        } else {
            assert_eq!(
                std::fs::read_to_string(path).expect("table document"),
                rendered,
                "{name}: regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
            );
        }
    }
}

#[test]
fn shared_rows_are_the_deployment_binaries_constants() {
    for command in [
        temper_engine_shell::commands::VERSIONS,
        temper_agent_shell::RELAY,
        temper_agent_shell::PROGRAM_CHILD,
        temper_agent_shell::CLAUDE,
    ] {
        assert!(TABLE.commands.contains(&command), "shared row: {:?}", command.reach);
    }
}
