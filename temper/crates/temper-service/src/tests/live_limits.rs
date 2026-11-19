//! Source settings precede derivation; same-set overrides follow it (configuration/derived.md, 4).
use super::{fixtures, stock};
use crate::startup;
use jig_protocol_config as config;
use temper_service_core::derived::{self, STOCK};

fn resolve(input: &fixtures::Start) -> (config::Engine, startup::application::Members) {
    let bytes = &input.document;
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 }).unwrap();
    let jig = temper_engine_service::startup::engine_over(
        &document,
        document.member(0, b"engine").unwrap(),
        &config::Directories::working(),
        &crate::stock::Given::Stock,
        &mut input.jig.clone(),
    )
    .unwrap();
    let mut problems = startup::Problems::empty();
    let own = startup::application::parse_with_limits(
        &document,
        document.member(0, b"application").unwrap(),
        &jig,
        &config::Directories::working(),
        &input.own,
        &mut problems,
    )
    .unwrap();
    assert!(problems.is_empty(), "{problems:?}");
    (jig, own)
}
fn document(goals: u32) -> fixtures::Start {
    let mut input = stock::generated();
    (input.jig, input.own) = crate::limits::base(goals, STOCK.payload, STOCK).unwrap();
    input
}

// Test step 5's reader with a limit fragment, not a deployment document. The
// resolved figures, never this fragment, cross the start boundary.
fn jig_over(input: &mut fixtures::Start, values: serde_json::Value) {
    let bytes = serde_json::to_vec(&serde_json::json!({"limits":{"overrides":values}})).unwrap();
    let fragment = config::document(&bytes, &config::DocumentLimits { bytes: 4096, tokens: 512, depth: 8 }).unwrap();
    derived::follow_tasks(&fragment, 0, &mut input.jig);
    for (name, value) in values.as_object().unwrap() {
        config::overrides::apply(&mut input.jig, name.as_bytes(), value.as_u64().unwrap()).unwrap();
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "test compares the store refusal with both measured value figures")]
fn scaled_ties_and_caps_pass_todays_start_checks() {
    for goals in [1, 40, 80, 400] {
        let input = document(goals);
        if goals >= 80 {
            let found = fixtures::named(&input, &config::Directories::working()).unwrap_err();
            let needed = if goals == 80 { 199_008_107_u32 } else { 993_957_867 };
            let expected = format!(
                "engine.limits.store: the store's map value, {needed} bytes, past its bound of 103387876 bytes\n"
            );
            assert_eq!(startup::lines(&found).as_ref(), expected.as_bytes());
        } else {
            let result = fixtures::named(&input, &config::Directories::working());
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
        let mut changes = serde_json::json!({"tasks.tasks":1000_u32});
        if let Some(inbox) = inbox {
            changes["people.inbox_tasks"] = inbox.into();
        }
        jig_over(&mut input, changes);
        input.own.forge.entries = 12;
        let (jig, own) = resolve(&input);
        assert_eq!(jig.limits.core.tasks.tasks, 1000);
        assert_eq!(jig.limits.core.people.inbox_tasks, inbox.unwrap_or(2000));
        assert_eq!(jig.limits.core.tasks.exemptions, 1000);
        assert_eq!(own.limits.forge.entries, 12);
    }
    let mut input = document(1);
    jig_over(&mut input, serde_json::json!({"tasks.delegates":0_u32}));
    let (jig, _) = resolve(&input);
    assert_eq!(jig.limits.core.tasks.delegates, 0);
    assert_eq!(jig.limits.core.people.operations, 2);
    let mut input = document(20);
    input.own.checkout.workspaces = 4;
    input.own.git_component.operations = 4;
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    let checked = startup::checked_start_with_limits(
        &input,
        &config::Directories::working(),
        &stock::files(),
        Some(&agent),
        &fixtures::seeds(),
        skein_lib::List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
        None,
        &input.jig,
        &input.own,
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
        (input.jig, input.own) = crate::limits::base(40, payload, derived::Sources { payload, ..STOCK }).unwrap();
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
        let sources = derived::Sources { payload: agent, ..STOCK };
        (input.jig, input.own) = crate::limits::base(40, setting, sources).unwrap();
        let (jig, own) = resolve(&input);
        assert_eq!(jig.limits.core.tasks.message_bytes, expected - 2);
        assert_eq!(own.limits.checkout.message_bytes, expected);
        let mut worker = config::stock();
        derived::worker(&mut worker, sources.with_payload(setting)).unwrap();
        assert_eq!(worker.hub.event_bytes, u64::from(expected - 2));
    }
}

#[test]
fn a_document_cannot_override_the_transcript_budget_by_its_future_name() {
    let mut input: serde_json::Value = serde_json::from_slice(&stock::generated()).unwrap();
    input["application"]["limits"] = serde_json::json!({"overrides":{"transcript_memory":2_147_483_648_u64}});
    let bytes = serde_json::to_vec(&input).unwrap();
    let found = fixtures::named(&bytes, &config::Directories::working()).unwrap_err();
    assert_eq!(startup::lines(&found).as_ref(), b"application.limits: moved: temper's limits are the setting temper-limits; ask temper, or use temper ctl settings apply\n");
}
