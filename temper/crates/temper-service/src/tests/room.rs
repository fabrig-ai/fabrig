//! Defaults are checked before a service starts
//! (temper/docs/design/domain/agent/tools.md).
use super::fixtures;
use crate::{startup, stock};
use alloc::boxed::Box;
use jig_protocol_config as config;

const CHARTER: u32 = 11;
const SHORT: u64 = 2_047;
const ENOUGH: u64 = 2_048;

fn agent() -> crate::settings::Agent {
    let bytes = serde_json::to_string_pretty(&temper_fixtures::agent::stock(true))
        .expect("stock agent JSON")
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
    agent.run_limits.host_part = 64_000;
    // The long instructions need the charter carriers, independently of the host-part bound.
    let document = super::stock::limited(&fixtures::document(), b"core.run_bytes", 65_536);
    let mut document: serde_json::Value = serde_json::from_slice(&document).expect("room fixture");
    document["application"]["limits"]["overrides"]["root.journal.run_bytes"] = 65_536_u32.into();
    let document = serde_json::to_vec(&document).expect("room fixture JSON");
    let engine = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
    let host = temper_engine_smith::room::host_part(
        Some(&agent.run_limits),
        65_536,
        &crate::settings::charter_endpoints(&engine),
    );
    let mut defaults = stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    for charter in &mut defaults.charters {
        if charter.number == 5 {
            // Finish's object wrapper (smith/docs/design/protocol/llm.md, 3) costs 95 bytes; this fixture lowers the host bound, and charter 5 is not under test.
            let shortfall = ENOUGH.saturating_sub(host.room(charter));
            if shortfall > 0 {
                let keep = charter.policy.instructions.len().checked_sub(usize::try_from(shortfall).unwrap()).unwrap();
                charter.policy.instructions = charter.policy.instructions.get(..keep).unwrap().into();
            }
        }
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
        b"charter 11: charter 11 leaves its brief 2,047 bytes of its host part's 64,000; it needs at least 2,048\n",
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
        b"charter 11: charter 11 leaves its brief 998 bytes of its host part's 64,000; it needs at least 2,048\n"
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
    let generated = super::stock::generated();
    {
        let document = generated;
        let engine = super::stock::engine(&document);
        let host = temper_engine_smith::room::host_part(
            Some(&agent().run_limits),
            65_536,
            &crate::settings::charter_endpoints(&engine),
        );
        let mut problems = startup::Problems::empty();
        startup::checks::room(&engine, host, &mut problems);
        assert!(problems.is_empty(), "{:?}", startup::lines(&problems));
        for charter in &engine.charters {
            let resolved =
                jig_domain_core::settings::charters::resolved(charter, &engine.llm.models, &engine.limits.core)
                    .unwrap();
            let room = host.room(&resolved);
            std::eprintln!("stock charter {}: stock, room {room}, bound {}", charter.number, host.bound.unwrap());
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

#[test]
#[expect(
    clippy::print_stderr,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "report the known wake names and each stock charter's margins"
)]
fn stock_charters_pass_the_fixed_check_on_every_wake() {
    use jig_domain_tasks::{REASON_NAMES, Reasons};
    let engine = super::stock::engine(&super::stock::generated());
    let host = temper_engine_smith::room::host_part(
        Some(&agent().run_limits),
        65_536,
        &crate::settings::charter_endpoints(&engine),
    );
    for (name, cost) in [
        ("fixed", host.fixed),
        ("model", host.model),
        ("always", host.always),
        ("asking", host.asking),
        ("asking_settings", host.asking_settings),
    ] {
        std::eprintln!("stock slot {name}: rendered {}, owned {}, encoded {}", cost.rendered, cost.owned, cost.encoded);
    }
    for (index, cost) in host.families.iter().enumerate() {
        std::eprintln!(
            "stock family {index}: rendered {}, owned {}, encoded {}",
            cost.rendered,
            cost.owned,
            cost.encoded
        );
    }
    for charter in &engine.charters {
        let mut owned = (REASON_NAMES[0], 0);
        let mut encoded = owned;
        for name in REASON_NAMES {
            let cost = host.on_wake(charter, Reasons::named(name).unwrap());
            if cost.owned > owned.1 {
                owned = (name, cost.owned);
            }
            if cost.encoded > encoded.1 {
                encoded = (name, cost.encoded);
            }
        }
        let mut optional = 0_u64;
        let mut models = 0_u64;
        for model in core::iter::once(&charter.policy.model).chain(&charter.policy.alternatives) {
            models += 1;
            if model.cache_write_price.is_none() {
                optional += 8;
            }
            if model.effort.is_none() {
                optional += 4;
            }
        }
        std::eprintln!(
            "stock charter {}: largest owned {}={} margin {}; encoded {}={} margin {}; optional overstatement {} with accounts, {} without accounts",
            charter.number,
            core::str::from_utf8(owned.0).unwrap(),
            owned.1,
            host.allowance.saturating_sub(owned.1),
            core::str::from_utf8(encoded.0).unwrap(),
            encoded.1,
            host.allowance.saturating_sub(encoded.1),
            optional,
            optional + models * 4
        );
        assert_eq!(
            host.check_fixed(charter),
            Ok(()),
            "charter {} largest owned {owned:?}, encoded {encoded:?}",
            charter.number
        );
    }
}

fn alone_start(
    document: &[u8],
    defaults: jig_domain_core::settings::Defaults,
) -> Result<startup::CheckedStart, startup::Problems> {
    startup::checked_start_with(
        document,
        &config::Directories::working(),
        &fixtures::files(),
        None,
        &fixtures::seeds(),
        fixtures::forge_runtime(),
        None,
        &stock::Given::Own { jig: defaults, forge: Box::new([]), ways: stock::ways() },
        Some(b"/usr/bin/git"),
    )
}

#[test]
fn a_start_without_an_agent_measures_and_refuses_an_unfit_default() {
    let document = fixtures::engine_alone_document();
    let document = super::stock::limited(&document, b"core.run_bytes", 65_536);
    let document = fixtures::with(&document, b"application.limits.overrides", br#"{"root.journal.run_bytes":65536}"#);
    let engine = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
    let defaults = stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let start =
        alone_start(&document, stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes))
            .expect("fitting defaults start without an agent");
    let measured = start.engine.host_part;
    assert_eq!(measured.bound, None);
    assert_eq!(measured.allowance, crate::assignment::charter_bound(engine.limits.hub.charter_bytes, None));
    assert!(measured.fixed.owned > 0);
    assert!(measured.always.encoded > 0);
    let mut defaults = defaults;
    let mut expected = None;
    for charter in &mut defaults.charters {
        if charter.number == 4 {
            let mut instructions = skein_lib::bytes::zeroed(40_000);
            instructions.fill(b'x');
            charter.policy.instructions = instructions;
            expected = Some(measured.check_fixed(charter).unwrap_err().words());
        }
    }
    let expected = expected.unwrap();
    let mut line = skein_lib::Writer::new(b"charter 4: ".len() + expected.len() + 1);
    for text in [b"charter 4: ".as_slice(), &expected, b"\n"] {
        line.put(text).unwrap();
    }
    let Err(problems) = alone_start(&document, defaults) else { panic!("unfit default must stop startup") };
    assert_eq!(startup::lines(&problems).as_ref(), line.finish().as_ref());
}

#[test]
fn a_configured_hub_allowance_below_a_default_stops_startup() {
    let document = fixtures::engine_alone_document();
    let engine = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
    let defaults = stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let host = temper_engine_smith::room::host_part(None, 65_536, &crate::settings::charter_endpoints(&engine));
    let charter = defaults.charters.first().unwrap();
    let cost = host.on_wake(charter, jig_domain_tasks::Reasons::FIRST).owned;
    let document = super::stock::limited(&document, b"hub.charter_bytes", cost - 1);
    let Err(problems) = alone_start(&document, defaults) else { panic!("hub allowance is checked before startup") };
    let text = startup::lines(&problems);
    assert!(text.starts_with(b"charter 1: charter 1 on a first wake is "), "{text:?}");
    let mut names_allowance = false;
    for part in text.windows(b"past its allowance".len()) {
        names_allowance |= part == b"past its allowance";
    }
    assert!(names_allowance);
}
