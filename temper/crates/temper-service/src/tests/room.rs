//! Defaults are checked before a service starts
//! (temper/docs/design/domain/agent/tools.md).
use super::fixtures;
use crate::{startup, stock};
use alloc::boxed::Box;
use jig_protocol_config as config;

const CHARTER: u32 = 11;
const SHORT: u64 = 2_047;
const ENOUGH: u64 = 2_048;

#[expect(clippy::disallowed_macros, reason = "the test reads the embedded stock agent configuration")]
fn agent() -> crate::settings::Agent {
    let bytes = include_str!("../../../temper-init/template/agent.json")
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", r#"{"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}}"#)
        .replace("api.anthropic.com:443", "127.0.0.1:8373")
        .replace("\"server_name\": \"api.anthropic.com\",", "")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"server_name\": \"api.openai.com\",", "")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"");
    let parsed = smith_agent_shell::config::parse(bytes.as_bytes()).unwrap();
    let mut agent = fixtures::startup_agent();
    agent.run_limits = parsed.service.limits.domain.run;
    agent
}

fn start(room: u64) -> Result<crate::Service, startup::Problems> {
    let mut agent = agent();
    // Exercise the brief floor within the fixture's small charter carriers.
    agent.run_limits.host_part = 65_536;
    // The long instructions need the charter carriers, independently of the host-part bound.
    let document = super::stock::limited(&fixtures::document(), b"core.run_bytes", 65_536);
    let mut document: serde_json::Value = serde_json::from_slice(&document).expect("room fixture");
    document["application"]["limits"]["overrides"]["root.journal.run_bytes"] = 65_536_u32.into();
    let document = serde_json::to_vec(&document).expect("room fixture JSON");
    let engine = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
    let host = temper_engine_smith::room::host_part(&agent.run_limits);
    let mut defaults = stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    for charter in &mut defaults.charters {
        if charter.number == CHARTER {
            charter.policy.instructions = Box::new([]);
            // Smith adds two line-ending bytes after these non-newline instructions.
            let instructions = host.room(&charter.policy).checked_sub(room).unwrap().checked_sub(2).unwrap();
            let mut bytes = skein_lib::List::with_capacity(u32::try_from(instructions).unwrap());
            for _ in 0..instructions {
                bytes.push(b'x').unwrap();
            }
            charter.policy.instructions = bytes.into_boxed();
        }
    }
    startup::startup_with(
        &document,
        &config::Directories::working(),
        &fixtures::files(),
        &agent,
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
        &stock::Given::Own { jig: defaults, forge: Box::default(), ways: stock::ways() },
    )
}

#[test]
fn defaults_leaving_2047_bytes_refuse_the_start() {
    let Err(found) = start(SHORT) else { panic!("a short charter refuses the start") };
    assert_eq!(
        startup::lines(&found).as_ref(),
        b"charter 11: charter 11 leaves its brief 2,047 bytes of its host part's 65,536; it needs at least 2,048\n",
    );
}

#[test]
fn defaults_leaving_2048_bytes_start() {
    start(ENOUGH).expect("the least brief room starts");
}

#[test]
#[expect(
    clippy::print_stderr,
    clippy::disallowed_macros,
    reason = "the check reports each stock charter's room for the implementation report"
)]
fn stock_charters_leave_at_least_2048_bytes() {
    let engine = fixtures::parse_engine(&fixtures::value(&fixtures::document(), b"engine"));
    let host = temper_engine_smith::room::host_part(&agent().run_limits);
    for charter in &engine.charters {
        let room = host.room(&charter.policy);
        std::eprintln!("stock charter {}: room {room}, bound {}", charter.number, host.bound);
        assert!(room >= ENOUGH);
    }
}
