//! Program account checks before runs and source environment isolation
//! (smith/docs/design/protocol/programs.md, sections 3 and 4.2).
use skein_io::kernel::Exit;
use skein_lib::Duration;
use smith_fake_claude::{Observation, Seen, Status, Step, Story, Turn, observations, seen};
use smith_local_process_world::{Files, Placement, World, referee::Ending, terminal::Action};

const ANSWER: &str = "Program answer";
const CREDENTIALS: [(&str, &str); 5] = [
    ("ANTHROPIC_API_KEY", "key-for-work"),
    ("ANTHROPIC_AUTH_TOKEN", "token-for-work"),
    ("CLAUDE_CODE_OAUTH_TOKEN", "token-for-work"),
    ("CLAUDE_CODE_USE_BEDROCK", "1"),
    ("AWS_BEARER_TOKEN_BEDROCK", "token-for-work"),
];
const OTHER_DIRECTORY: &str = "/elsewhere";

fn program(files: &Files, status: Status, mut steps: Vec<Step>) -> World {
    let mut world = World::new(19, files, Placement::Spawned, ANSWER.as_bytes());
    steps.push(Step::Outcome { input: serde_json::json!({"outcome":{"form":"report","text":ANSWER,"fields":{}}}) });
    world.on_program(Story { status, turns: vec![Turn::new(steps)], ..Story::default() });
    world
}

fn chat(files: &Files, status: Status) -> World {
    let mut world = program(files, status, Vec::new());
    world.interactive();
    world.scenario.commands = vec![
        Action::Send(b"Ask first\n".as_slice().into()),
        Action::Wait(ANSWER.as_bytes().into()),
        Action::Send(b"Ask second\n".as_slice().into()),
        Action::Eof,
    ];
    world
}

fn call_order(calls: &[Observation]) -> Vec<&'static str> {
    calls
        .iter()
        .map(|call| {
            if call.arguments == ["auth", "status"] {
                "status"
            } else if call.arguments.first().is_some_and(|arg| arg == "-p") {
                "print"
            } else {
                panic!("unexpected program invocation: {:?}", call.arguments);
            }
        })
        .collect()
}

fn option<'a>(call: &'a Observation, name: &str) -> Option<&'a str> {
    call.arguments.windows(2).find(|pair| pair[0] == name).map(|pair| pair[1].as_str())
}

#[test]
fn program_exec_checks_its_source_without_lending_host_credentials() {
    let files = Files::new();
    let mut world = program(&files, Status::Truthful, vec![Step::Text("Told program text".into())]);
    world.exec(b"Ask first", ANSWER.as_bytes());
    world.scenario.launch.program.as_mut().expect("program launch").environment = CREDENTIALS
        .into_iter()
        .chain([("CLAUDE_CONFIG_DIR", OTHER_DIRECTORY)])
        .map(|(name, value)| (name.into(), value.into()))
        .collect();
    world.settle();
    assert_eq!(world.exit(), Some(Exit::Code(0)));
    assert_eq!(world.shown(), ANSWER.as_bytes());

    // The fake opens this log beneath CLAUDE_CONFIG_DIR. It contains all
    // three calls only if the host, the agent, and the run use the source.
    let log = world.program_log();
    let calls = observations(&log);
    assert_eq!(call_order(&calls), ["status", "status", "print"], "host check, agent check, then run");
    for call in &calls {
        let names = &call.environment;
        for (name, _) in CREDENTIALS {
            assert!(!names.iter().any(|kept| kept == name), "program kept {name}");
        }
        assert_eq!(names.iter().filter(|name| *name == "CLAUDE_CONFIG_DIR").count(), 1);
        assert_eq!(
            names.iter().filter(|name| name.starts_with("ANTHROPIC_")).map(String::as_str).collect::<Vec<_>>(),
            ["ANTHROPIC_CONFIG_DIR"]
        );
    }
    for output in [world.shown(), world.errors()] {
        let output = std::str::from_utf8(output).expect("UTF-8 terminal output");
        for value in [CREDENTIALS[0].1, CREDENTIALS[1].1, OTHER_DIRECTORY] {
            assert!(!output.contains(value), "person's credential or directory leaked to the terminal");
        }
    }
    assert!(
        !seen(&log).iter().any(|record| matches!(record, Seen::Called { tool, .. } if tool == "finish")),
        "a structured outcome does not call finish over MCP"
    );
}

#[test]
fn program_chat_refuses_a_signed_out_second_run() {
    let files = Files::new();
    let mut world = chat(&files, Status::Sequence(vec![Status::Truthful, Status::Truthful, Status::NotSignedIn]));
    world.scenario.ending = Ending::ProgramRecheck { text: ANSWER.as_bytes().to_vec(), interrupted: false };
    world.settle();
    assert_eq!(world.exit(), Some(Exit::Code(0)));
    assert!(std::str::from_utf8(world.shown()).expect("UTF-8 stdout").contains(ANSWER));
    assert_eq!(
        std::str::from_utf8(world.errors()).expect("UTF-8 stderr").lines().last(),
        Some("smith: account work: Claude Code is not signed in: run `smith login work`")
    );
    assert_eq!(call_order(&observations(&world.program_log())), ["status", "status", "print", "status"]);
}

#[test]
fn program_chat_interrupts_a_silent_recheck() {
    let files = Files::new();
    let mut world = chat(&files, Status::Sequence(vec![Status::Truthful, Status::Truthful, Status::Silent]));
    world.scenario.ending = Ending::ProgramRecheck { text: ANSWER.as_bytes().to_vec(), interrupted: true };
    world.interrupt_on_program_status(3);
    world.settle();
    assert_eq!(world.exit(), Some(Exit::Code(130)));
    assert_eq!(call_order(&observations(&world.program_log())), ["status", "status", "print", "status"]);
    let elapsed = world.program_interrupt_settlement();
    assert!(elapsed < Duration::from_secs(5), "the world settled {elapsed:?} after the interrupt");
}

#[test]
fn program_chat_runs_both_lines_after_input_ends() {
    let files = Files::new();
    let mut world = chat(&files, Status::Truthful);
    world.settle();
    assert_eq!(world.exit(), Some(Exit::Code(0)));
    let output = std::str::from_utf8(world.shown()).expect("UTF-8 stdout");
    assert_eq!(output.matches(ANSWER).count(), 2, "both reports: {output}");
    let calls = observations(&world.program_log());
    assert_eq!(call_order(&calls), ["status", "status", "print", "status", "status", "print"]);
    let (first, next) = (&calls[2], &calls[5]);
    for run in [first, next] {
        assert!(option(run, "--mcp-config").is_some(), "each ordinary print has MCP");
    }
    assert_eq!(option(next, "--resume"), Some(option(first, "--session-id").expect("new session")));
}
