//! Source settings precede derivation; same-set overrides follow it (configuration/derived.md, 4).
use super::{fixtures, stock};
use crate::startup;
use jig_protocol_config as config;
use temper_service_core::derived::{self, STOCK};

fn resolve(input: &serde_json::Value) -> (config::Engine, startup::application::Members) {
    let bytes = serde_json::to_vec(input).unwrap();
    let document =
        config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 }).unwrap();
    let jig =
        startup::engine(&document, document.member(0, b"engine").unwrap(), &config::Directories::working()).unwrap();
    let mut problems = startup::Problems::empty();
    let own = startup::application::parse(
        &document,
        document.member(0, b"application").unwrap(),
        &jig,
        &config::Directories::working(),
        &mut problems,
    )
    .unwrap();
    assert!(problems.is_empty(), "{problems:?}");
    (jig, own)
}
fn document(goals: u32) -> serde_json::Value {
    let mut input: serde_json::Value = serde_json::from_slice(&stock::generated()).unwrap();
    input["application"]["limits"]["overrides"]["live_goals"] = goals.into();
    input
}

#[test]
#[expect(clippy::disallowed_macros, reason = "test compares the store refusal with both measured value figures")]
fn scaled_ties_and_caps_pass_todays_start_checks() {
    for goals in [1, 40, 80, 400] {
        let input = document(goals);
        let bytes = serde_json::to_vec(&input).unwrap();
        if goals >= 80 {
            let found = crate::named(&bytes, &config::Directories::working()).unwrap_err();
            let needed = if goals == 80 { 199_008_107_u32 } else { 993_957_867 };
            let expected = format!(
                "engine.limits.store: the store's map value, {needed} bytes, past its bound of 100881496 bytes\n"
            );
            assert_eq!(startup::lines(&found).as_ref(), expected.as_bytes());
        } else {
            let result = crate::named(&bytes, &config::Directories::working());
            assert!(result.is_ok(), "live_goals {goals}: {result:?}");
        }
        let (mut engine, own) = resolve(&input);
        let root =
            temper_engine_service::startup::engine_limits(&mut engine.limits, &own.limits, engine.memory).unwrap();
        let tasks = root.tasks;
        let forge = root.forge;
        assert_eq!(root.people.inbox_tasks, tasks.tasks * 2);
        assert!(tasks.exemptions <= tasks.tasks && tasks.exemptions <= 4096);
        assert!(tasks.delegates < tasks.tasks);
        assert!(root.people.operations > tasks.delegates);
        assert!(root.people.operations >= tasks.project_goals && root.people.operations <= tasks.tasks);
        assert!(forge.landings > 0 && forge.landings <= forge.holds.checked_div(forge.landing_members).unwrap());
        assert!(forge.holds <= 256 && forge.tasks <= 1024);
        assert_eq!(own.limits.live_goals, goals);
    }
}

#[test]
fn scaled_count_overrides_win_and_their_dependents_follow() {
    for inbox in [None, Some(2500)] {
        let mut input = document(80);
        input["engine"]["limits"]["overrides"]["tasks.tasks"] = 1000_u32.into();
        if let Some(inbox) = inbox {
            input["engine"]["limits"]["overrides"]["people.inbox_tasks"] = inbox.into();
        }
        input["application"]["limits"]["overrides"]["forge.entries"] = 12_u32.into();
        let (jig, own) = resolve(&input);
        assert_eq!(jig.limits.core.tasks.tasks, 1000);
        assert_eq!(jig.limits.core.people.inbox_tasks, inbox.unwrap_or(2000));
        assert_eq!(jig.limits.core.tasks.exemptions, 1000);
        assert_eq!(own.limits.forge.entries, 12);
    }
    let mut input = document(1);
    input["engine"]["limits"]["overrides"]["tasks.delegates"] = 0_u32.into();
    let (jig, _) = resolve(&input);
    assert_eq!(jig.limits.core.tasks.delegates, 0);
    assert_eq!(jig.limits.core.people.operations, 2);
    let bytes = serde_json::to_vec(&document(20)).unwrap();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).unwrap());
    let checked = startup::checked_start_with(
        &bytes,
        &config::Directories::working(),
        &stock::files(),
        Some(&agent),
        &fixtures::seeds(),
        skein_lib::List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
        None,
    )
    .unwrap();
    assert_eq!(checked.engine.jig.limits.core.tasks.tasks, 384);
    assert_eq!(checked.engine.limits.live_goals, 20);
    assert_eq!(checked.engine.limits.forge.entries, 4);
}

#[test]
fn an_engines_payload_setting_precedes_its_carriers() {
    for payload in [32_000, 100_000] {
        let mut input = document(40);
        input["workers"].as_array_mut().unwrap().clear();
        input["application"]["limits"]["overrides"]["tool_payload"] = payload.into();
        input["engine"]["limits"]["overrides"] = serde_json::json!({});
        input["application"]["limits"]["overrides"] = serde_json::json!({"tool_payload":payload});
        let (jig, own) = resolve(&input);
        assert_eq!(jig.limits.core.tasks.message_bytes, payload - 2);
        assert_eq!(jig.limits.core.tasks.inbox_bytes, STOCK.inbox.checked_mul(payload - 2).unwrap());
        assert_eq!(own.limits.checkout.message_bytes, payload);
        assert_eq!(own.limits.forge.client.op_bytes, payload - 2);
        assert_eq!(own.limits.tool_payload, payload);
    }
}

#[test]
fn box_and_worker_carriers_take_the_larger_payload() {
    for (setting, agent, expected) in [(100_000_u32, 63_740, 100_000), (32_000, 63_740, 63_740)] {
        let mut input = document(40);
        input["application"]["limits"]["overrides"] = serde_json::json!({"tool_payload":setting});
        input["engine"]["limits"]["overrides"] = serde_json::json!({});
        input["workers"][0]["limits"]["overrides"] = serde_json::json!({});
        let bytes = serde_json::to_vec(&input).unwrap();
        let doc =
            config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 }).unwrap();
        let sources = derived::Sources { payload: agent, ..STOCK };
        let jig = temper_engine_service::startup::engine_with_sources(
            &doc,
            doc.member(0, b"engine").unwrap(),
            &config::Directories::working(),
            &crate::stock::Given::Stock,
            sources,
        )
        .unwrap();
        let mut problems = startup::Problems::empty();
        let own = startup::application::parse_over(
            &doc,
            doc.member(0, b"application").unwrap(),
            &jig,
            &config::Directories::working(),
            sources,
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(jig.limits.core.tasks.message_bytes, expected - 2);
        assert_eq!(own.limits.checkout.message_bytes, expected);
        let workers = config::read::list(&doc, doc.member(0, b"workers").unwrap(), doc.len()).unwrap();
        let worker =
            derived::worker_document(&doc, *workers.get(0).unwrap(), &config::Directories::working(), sources).unwrap();
        assert_eq!(worker.limits.hub.event_bytes, u64::from(expected - 2));
        let application = doc.member(0, b"application").unwrap();
        let at = doc.member(application, b"limits").unwrap();
        let own = temper_service_core::startup::configured_limits_over(
            &doc,
            at,
            &config::Path::root().member(b"application").member(b"limits"),
            sources,
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(own.checkout.message_bytes, expected);
    }
}

#[test]
fn a_document_cannot_override_the_transcript_budget_by_its_future_name() {
    let mut input = document(40);
    input["application"]["limits"]["overrides"]["transcript_memory"] = 2_147_483_648_u64.into();
    let bytes = serde_json::to_vec(&input).unwrap();
    let result = crate::named(&bytes, &config::Directories::working());
    assert!(result.is_err(), "the transcript budget override must be refused");
    let found = result.err().unwrap();
    let expected = b"application.limits.overrides.transcript_memory: must agree with engine.memory_bytes, which sets transcript_memory for now\n";
    assert_eq!(startup::lines(&found).as_ref(), expected);
    let document = config::document(
        br#"{"overrides":{"transcript_memory":2147483648}}"#,
        &config::DocumentLimits { bytes: 128, tokens: 16, depth: 8 },
    )
    .unwrap();
    let mut found = startup::Problems::empty();
    temper_service_core::startup::configured_limits(
        &document,
        0,
        &config::Path::root().member(b"application").member(b"limits"),
        &mut found,
    )
    .unwrap();
    assert_eq!(startup::lines(&found).as_ref(), expected);
}
