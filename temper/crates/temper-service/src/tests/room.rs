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

fn start(room: u64, part: Option<usize>) -> Result<crate::Service, startup::Problems> {
    let mut agent = agent();
    // This fixture host bound leaves the other stock charters at least 2,048 bytes.
    agent.run_limits.host_part = 65_714;
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
            let instructions = host.room(charter).checked_sub(room).unwrap().checked_sub(2).unwrap();
            let mut bytes = skein_lib::List::with_capacity(u32::try_from(instructions).unwrap());
            for _ in 0..instructions {
                bytes.push(b'x').unwrap();
            }
            charter.policy.instructions = bytes.into_boxed();
            if let Some(bytes) = part {
                let mut instructions = skein_lib::List::with_capacity(u32::try_from(bytes).unwrap());
                for _ in 0..bytes {
                    instructions.push(b'x').unwrap();
                }
                charter.parts = Box::new([jig_domain_core::CharterPart {
                    on: jig_domain_tasks::Reasons::FIRST,
                    instructions: instructions.into_boxed(),
                }]);
            }
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
    let Err(found) = start(SHORT, None) else { panic!("a short charter refuses the start") };
    assert_eq!(
        startup::lines(&found).as_ref(),
        b"charter 11: charter 11 leaves its brief 2,047 bytes of its host part's 65,714; it needs at least 2,048\n",
    );
}

#[test]
fn defaults_leaving_2048_bytes_start() {
    start(ENOUGH, None).expect("the least brief room starts");
}

#[test]
fn default_parts_are_counted_before_start() {
    let Err(found) = start(4_000, Some(3_000)) else { panic!("the default's part takes its brief room") };
    assert_eq!(
        startup::lines(&found).as_ref(),
        b"charter 11: charter 11 leaves its brief 998 bytes of its host part's 65,714; it needs at least 2,048\n"
    );
    start(4_000, None).expect("without the part the default fits");
}

#[test]
#[expect(
    clippy::print_stderr,
    clippy::disallowed_macros,
    reason = "the check reports each stock charter's room for the implementation report"
)]
fn stock_charters_leave_at_least_2048_bytes() {
    let host = temper_engine_smith::room::host_part(&agent().run_limits);
    let generated = super::stock::generated();
    for (name, profile) in [("small", b"\"small\"".as_slice()), ("standard", b"\"standard\"")] {
        let document = fixtures::with(&generated, b"engine.limits.profile", profile);
        let engine = super::stock::engine(&document);
        let mut problems = startup::Problems::empty();
        startup::checks::room(&engine, host, &mut problems);
        assert!(problems.is_empty(), "{:?}", startup::lines(&problems));
        for charter in &engine.charters {
            let resolved =
                jig_domain_core::settings::charters::resolved(charter, &engine.llm.models, &engine.limits.core)
                    .unwrap();
            let room = host.room(&resolved);
            std::eprintln!("stock charter {}: profile {name}, room {room}, bound {}", charter.number, host.bound);
            assert!(room >= ENOUGH);
            if charter.number == 5 {
                let mut without = resolved.clone();
                without.parts = Box::new([]);
                std::eprintln!("charter 5 before parts: {}, after: {room}", host.room(&without));
                assert!(room < host.room(&without));
            }
        }
    }
}
