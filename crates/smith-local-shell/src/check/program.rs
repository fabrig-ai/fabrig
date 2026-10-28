//! The spawned program chat's complete startup admission (shell.md, sections 4.4 and 9).
use super::*;

// Ceilings on each invocation's startup formula: a quarter above
// 10-28's figures, moved by hand when a real change passes one.
const EXEC_CEILING: u64 = 1_100_000_000;
const JSON_CEILING: u64 = 1_200_000_000;
const CHAT_CEILING: u64 = 1_200_000_000;

#[test]
fn program_invocations_fit_the_standard_memory() {
    // Agent program paths are bounded at 4096 bytes; /programs consumes nine.
    const PROGRAM_PATH_BYTES: usize = 4096;
    const PROGRAMS_SUFFIX_BYTES: usize = 9;
    const MEMORY: u64 = 2_281_701_376;
    let credentials = format!(
        "/{}",
        "c".repeat(
            PROGRAM_PATH_BYTES
                .checked_sub(PROGRAMS_SUFFIX_BYTES)
                .and_then(|bytes| bytes.checked_sub(1))
                .expect("programs path prefix")
        )
    );
    let trust = skein_scratch::paths::workspace().join("tests/protocol-llm/fixtures/root.der");
    let settings = settings::merge(
        &command::Shared::default(),
        "/workspace".into(),
        &[settings::Source {
            layer: settings::Layer::User,
            path: "program-chat.toml".into(),
            text: format!(
                "credentials='{credentials}'\nmodel='anthropic/claude-opus-5-5'\n\
                 [accounts.anthropic]\nsource='program'\n[endpoints.anthropic]\nprogram='claude_code'\ntrust.system=false\ntrust.der=['{}']\n[programs]\nclaude_code='/bin/true'\n",
                trust.display()
            ),
        }],
    )
    .unwrap_or_else(|why| panic!("{}", why.line()));
    for (name, interactive, json, ceiling) in
        [("exec", false, false, EXEC_CEILING), ("json", false, true, JSON_CEILING), ("chat", true, false, CHAT_CEILING)]
    {
        let mut resolved =
            settings::resolve::resolve_startup(&settings).unwrap_or_else(|why| panic!("{}", why.line())).0;
        startup_agent(&mut resolved).expect("product placement");
        let invocation = command::Exec {
            shared: command::Shared::default(),
            prompt: command::Prompt::Stdin,
            output: None,
            ephemeral: false,
            deliver: false,
            json,
        };
        let (limits, _, worst) = price_invocation(&mut resolved, &invocation, skein_lib::Wall::EPOCH, interactive)
            .expect("program invocation startup admission");
        assert!(matches!(limits.local.agent, smith_local_domain::AgentLimits::Spawned(_)), "program placement");
        assert!(limits.inline.is_none(), "spawned composition has no inline effects");
        assert_eq!(resolved.agent.memory, MEMORY);
        assert_eq!(resolved.agent.service.limits.events.is_some(), interactive || json);
        let producer = crate::Local::producer_memory(&resolved.agent.service).expect("priced producer");
        if interactive || json {
            assert!(producer > 0, "a consumed product stream has a producer");
        } else {
            assert_eq!(producer, 0, "plain spawned exec composes no producer");
        }
        assert!(worst <= ceiling, "{name}: worst_case={worst} passed its ceiling {ceiling}");
        let margin = MEMORY.checked_sub(worst).expect("program invocation fits shipped cap");
        println!("PROGRAM_{name} worst_case={worst} memory={MEMORY} margin={margin} ceiling={ceiling}");
    }
}
