#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "unit test harnesses use ordinary Rust per programming-model.md, section 10.2"
)]

extern crate std;

use super::fixtures;
use super::{files, mcp, start::Names, stream};
use alloc::boxed::Box;
use skein_lib::Secret;

#[test]
fn entropy_names_have_separate_domains() {
    let names = Names::new(&Secret::new(Box::new([7; 32])));
    let repeated = Names::new(&Secret::new(Box::new([7; 32])));
    assert!(names.mcp.expose() == repeated.mcp.expose());
    assert_eq!(names.run, repeated.run);
    assert!(names.mcp.expose() != names.run);
}

#[test]
fn settings_require_each_builtin_to_ask() {
    let value: serde_json::Value = serde_json::from_slice(files::SETTINGS).expect("settings JSON");
    assert_eq!(
        value,
        serde_json::json!({"permissions":{"allow":["mcp__smith"],
        "ask":["Read","Glob","Grep","Write","Edit","Bash"]}})
    );
}

fn tool() -> mcp::Tool {
    mcp::Tool {
        name: b"finish".as_slice().into(),
        description: b"Finish".as_slice().into(),
        input_schema: br#"{"type":"object"}"#.as_slice().into(),
    }
}

#[test]
fn init_requires_the_connected_smith_server() {
    let bytes = br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":["mcp__smith__finish","mcp__smith__permission_prompt"]}"#;
    assert!(stream::init(bytes, &[tool()], 32, 256));
    let missing = br#"{"type":"system","subtype":"init","mcp_servers":[],"tools":["mcp__smith__finish","mcp__smith__permission_prompt"]}"#;
    assert!(!stream::init(missing, &[tool()], 32, 256));
}

#[test]
fn init_refuses_an_ungranted_builtin() {
    let bytes = br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":["mcp__smith__finish","mcp__smith__permission_prompt","Read"]}"#;
    assert!(!stream::init(bytes, &[tool()], 32, 256));
}

#[test]
fn init_refuses_a_repeated_tool() {
    let bytes = br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":["mcp__smith__finish","mcp__smith__finish"]}"#;
    assert!(!stream::init(bytes, &[tool()], 32, 256));
}

#[test]
fn diagnostics_stay_within_the_utf8_byte_cap() {
    assert_eq!(super::diagnostic(b"tail\xff", 5).as_ref(), b"tail?");
    assert_eq!(super::diagnostic("tailé".as_bytes(), 5).as_ref(), b"tail");
}

#[test]
fn command_keeps_the_inline_schema_in_one_argument() {
    let arguments = super::start::arguments(
        b"chosen-model",
        Some(b"high"),
        b"/run/instructions.md",
        b"/run/mcp.json",
        br#"{"type":"object","description":"spaces stay inline"}"#,
        9,
        b"9213b8f4-a680-416e-a34c-6718a9575807",
        b"",
        false,
        false,
    )
    .expect("bounded command");
    let flag = arguments.iter().position(|argument| argument.as_ref() == b"--json-schema").expect("schema flag");
    assert_eq!(arguments[flag + 1].as_ref(), br#"{"type":"object","description":"spaces stay inline"}"#);
    let flag = arguments.iter().position(|argument| argument.as_ref() == b"--effort").expect("effort flag");
    assert_eq!(arguments[flag + 1].as_ref(), b"high");
    let flag = arguments.iter().position(|argument| argument.as_ref() == b"--tools").expect("tools flag");
    assert!(arguments[flag + 1].is_empty());
}

#[test]
fn mcp_file_keeps_the_secret_in_the_relay_environment() {
    let bytes = files::mcp_configuration(b"/relay/quoted\"name", 1234, &[b'a'; 64]);
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("MCP file JSON");
    let server = &value["mcpServers"]["smith"];
    assert_eq!(server["type"], "stdio");
    assert_eq!(server["command"], "/relay/quoted\"name");
    assert_eq!(server["args"], serde_json::json!(["--mcp-relay", "127.0.0.1:1234"]));
    assert_eq!(server["env"]["SMITH_MCP_SECRET"].as_str().expect("token").len(), 64);
}

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[test]
#[expect(clippy::print_stdout, reason = "record the measured peak beside its bound")]
fn compaction_rendering_stays_under_its_scenario_ceiling() {
    use super::fixtures::{backend, request};
    use smith_domain::llm;
    use smith_protocol_llm::{FromDomain, Observation};

    const INPUT: u32 = 4096;
    // Room for the nearly input-sized message payload and its neutral cells.
    const PROMPT: u64 = 8192;
    // Leave the 53-byte envelope, quotes and escaped newlines within INPUT.
    const TEXT_SLACK: usize = 64;
    const FRAMING: usize = b"user:\n".len() + b"\nresult of id: withdrawn\n".len();
    // Peak of compaction rendering with and without prompt observations.
    // Measured 12_159 B on 2026-10-28; a quarter of headroom, rounded up.
    const COMPACTION_CEILING: u64 = 16_000;

    for observe in [false, true] {
        let mut backend = backend();
        backend.settings.input_bytes = INPUT;
        backend.settings.prompt_bytes = PROMPT;
        backend.settings.translation.rendered_result = 128;
        backend.settings.translation.client.history_items = 2;
        backend.settings.translation.client.tools = 0;
        let mut next = request(&backend);
        let FromDomain::Complete { prompt, observation, .. } = &mut next else { panic!("complete") };
        prompt.purpose = llm::Purpose::Compaction { room: 1000 };
        *observation = observe.then_some(Observation { first: true, messages: 0 });
        // Include the consumed message ownership in the peak: Span otherwise
        // subtracts its frees from render/quote/join's simultaneous live bytes.
        let span = skein_heap::Span::start();
        prompt.messages = Box::new([llm::Message {
            role: llm::Role::User,
            content: Box::new([
                llm::Block::Text {
                    text: alloc::vec![b'x'; usize::try_from(INPUT).unwrap() - TEXT_SLACK - FRAMING].into(),
                    replay: None,
                },
                llm::Block::ToolResult { id: b"id".as_slice().into(), result: llm::Returned::Withdrawn },
            ]),
        }]);
        let messages = span.end().net;
        fixtures::begin_prompt(&next, &mut backend);
        fixtures::retain_request(next, &mut backend);

        let span = skein_heap::Span::start();
        let rendered = backend.compaction_input();
        let measured = span.end();
        let peak = u64::try_from(measured.peak + messages).unwrap();
        std::println!(
            "COMPACTION observe={observe} growth={} messages={messages} peak={peak} ceiling={COMPACTION_CEILING}",
            measured.peak
        );
        assert_eq!(rendered, Some(()));
        assert!(peak > 2 * u64::from(INPUT), "peak={peak}");
        assert!(peak <= COMPACTION_CEILING, "peak={peak} ceiling={COMPACTION_CEILING}");
    }
}

#[test]
fn invalid_limits_are_refused() {
    let mut settings = fixtures::backend().settings;
    assert!(super::backend_valid(&settings));
    let path = settings.path_bytes;
    settings.path_bytes = u32::MAX;
    assert!(!super::backend_valid(&settings));
    settings.path_bytes = path;
    let blocks = settings.receiving.max_completion_blocks;
    for count in [u32::MAX / 5 + 1, u32::MAX - 15] {
        settings.receiving.max_completion_blocks = count;
        assert!(!super::backend_valid(&settings));
    }
    settings.receiving.max_completion_blocks = blocks;
    let tools = settings.translation.client.tools;
    settings.translation.client.tools = u32::MAX - 6;
    assert!(!super::backend_valid(&settings));
    settings.translation.client.tools = tools;
    let input = settings.translation.stated.call_input_bytes;
    for bound in [0, u32::MAX] {
        settings.translation.stated.call_input_bytes = bound;
        assert!(!super::backend_valid(&settings));
    }
    settings.translation.stated.call_input_bytes = input;
    let line = settings.line_bytes;
    for bound in [0, u32::MAX] {
        settings.line_bytes = bound;
        assert!(!super::backend_valid(&settings));
    }
    settings.line_bytes = line;
    let message = settings.mcp.message_bytes;
    for bound in [0, u32::MAX - 1, u32::MAX] {
        settings.mcp.message_bytes = bound;
        assert!(!super::backend_valid(&settings));
    }
    settings.mcp.message_bytes = message;
    let schema = settings.mcp.schema_bytes;
    for bound in [0, u32::MAX - 1, u32::MAX] {
        settings.mcp.schema_bytes = bound;
        assert!(!super::backend_valid(&settings));
    }
    settings.mcp.schema_bytes = schema;
    // These extremes only enlarge heap totals; they do not invalidate an edge.
    settings.prompt_bytes = u64::MAX;
    settings.receiving.max_completion_bytes = u64::MAX;
    assert!(super::backend_valid(&settings));
}

#[test]
fn launch_uses_the_workspace_policy() {
    use super::Root;
    use skein_io::kernel;
    use skein_lib::Token;
    use smith_domain::llm;
    use smith_protocol_llm::FromDomain;

    const EMPTY_INIT: &[u8] =
        br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":[]}"#;
    const INSPECT_INIT: &[u8] = br#"{"type":"system","subtype":"init","mcp_servers":[{"name":"smith","status":"connected"}],"tools":["Read","Glob","Grep"]}"#;
    const SUMMARY_INIT: &[u8] = br#"{"type":"system","subtype":"init","mcp_servers":[],"tools":[]}"#;
    for (label, workspace, summary, resume, tools, disabled, directory, init) in [
        ("no workspace", None, false, false, b"".as_slice(), true, b"work".as_slice(), EMPTY_INIT),
        (
            "a workspace",
            Some(Root { file: Token::new(50), descriptor: kernel::Fd::new(51) }),
            false,
            false,
            b"Read,Glob,Grep",
            false,
            b"",
            INSPECT_INIT,
        ),
        ("no workspace summary", None, true, false, b"", true, b"work", SUMMARY_INIT),
        (
            "a workspace summary",
            Some(Root { file: Token::new(50), descriptor: kernel::Fd::new(51) }),
            true,
            false,
            b"",
            false,
            b"",
            SUMMARY_INIT,
        ),
        ("no workspace resume", None, false, true, b"", true, b"work", EMPTY_INIT),
        (
            "a workspace resume",
            Some(Root { file: Token::new(50), descriptor: kernel::Fd::new(51) }),
            false,
            true,
            b"Read,Glob,Grep",
            false,
            b"",
            INSPECT_INIT,
        ),
    ] {
        for keep_program_prompt in [false, true] {
            let mut backend = fixtures::resumable();
            backend.settings.workspace = workspace;
            backend.settings.keep_program_prompt = keep_program_prompt;
            let mut request = if resume {
                fixtures::resume_request(&backend)
            } else {
                fixtures::new_window([0; 16], &mut backend);
                fixtures::request(&backend)
            };
            let FromDomain::Complete { prompt, .. } = &mut request else { panic!("complete") };
            prompt.tools.inspect = true;
            if summary {
                prompt.purpose = llm::Purpose::Compaction { room: 1000 };
            }
            fixtures::claim_completion(fixtures::COMPLETION, &mut backend);
            fixtures::retain_request(request, &mut backend);
            let spawn = backend.launch_spec().expect("launch");
            let value = |flag: &[u8]| {
                let index = spawn.args.iter().position(|argument| argument.as_ref() == flag).expect("flag");
                spawn.args[index + 1].as_ref()
            };
            let (prompt_flag, absent_flag) = if keep_program_prompt {
                (b"--append-system-prompt-file".as_slice(), b"--system-prompt-file".as_slice())
            } else {
                (b"--system-prompt-file".as_slice(), b"--append-system-prompt-file".as_slice())
            };
            assert_eq!(value(prompt_flag), backend.run_path(b"instructions.md").as_ref(), "{label}: prompt flag");
            assert!(!spawn.args.iter().any(|arg| arg.as_ref() == absent_flag), "{label}: one prompt flag");
            assert_eq!(value(b"--tools"), tools, "{label}: built-ins");
            assert!(spawn.args.iter().any(|arg| arg.as_ref() == b"--strict-mcp-config"), "{label}: strict MCP");
            if !summary {
                assert_eq!(value(b"--mcp-config"), backend.run_path(b"mcp.json").as_ref(), "{label}: MCP file");
            }
            assert_eq!(
                spawn.args.iter().any(|arg| arg.as_ref() == b"--disable-slash-commands"),
                disabled,
                "{label}: slash commands"
            );
            assert_eq!(spawn.args.iter().any(|arg| arg.as_ref() == b"--resume"), resume, "{label}: resume");
            let expected = if directory.is_empty() {
                Box::default()
            } else {
                skein_lib::bytes::concat(&[&backend.run_relative(), b"/", directory])
            };
            assert_eq!(spawn.dir, expected, "{label}: working directory");
            assert_eq!(spawn.root, workspace.or(backend.settings.directory).expect("root").descriptor);
            let mut up = skein_lib::Queue::uncapped();
            let mut below = skein_lib::Queue::uncapped();
            backend.later_init(&fixtures::env(0), init, &mut up, &mut below);
            assert!(up.is_empty(), "{label}: the init agrees with the launch: {up:?}");
        }
    }
}
