#![expect(clippy::disallowed_macros, reason = "ordinary tests format diagnostic text")]
use super::fixtures;
use crate::startup::{Problems, lines, worker::worker as check_worker};
#[test]
fn the_checkouts_repositories_past_the_agents_directories_name_both_limits() {
    let mut agent = fixtures::startup_agent();
    agent.host_limits.directories = 1;
    let worker = fixtures::worker_settings();
    let own = crate::worker_settings::Limits::shared(temper_service_core::limits::profile(
        temper_service_core::limits::Profile::Small,
    ));
    let mut found = Problems::empty();
    check_worker(&crate::worker_settings::root_limits(&worker, &agent, &own), &mut found);
    assert_eq!(
        &*lines(&found),
        b"application.limits.overrides.checkout.repositories: must agree with the agent's directories\n"
    );
}

/// The same typed worker limits that startup checks before allocating a service.
fn worker_limits() -> temper_worker_domain::Limits {
    crate::worker_settings::root_limits(
        &fixtures::worker_settings(),
        &fixtures::startup_agent(),
        &crate::worker_settings::Limits::shared(temper_service_core::limits::profile(
            temper_service_core::limits::Profile::Small,
        )),
    )
}

#[test]
fn payload_event_refusals_name_the_configured_and_required_bytes() {
    for (configured, expected) in [
        (4096, "4096 bytes, short of agent.json profile.declared.tool_payload less the 2-byte separator, 8192 bytes"),
        (8193, "8193 bytes, past agent.json profile.declared.tool_payload less the 2-byte separator of 8192"),
    ] {
        let mut limits = worker_limits();
        limits.agent.message_bytes = 8194;
        limits.bounds.message_bytes = 8192;
        limits.host.event_bytes = configured;
        let mut found = Problems::empty();
        check_worker(&limits, &mut found);
        let expected = format!("workers[0].limits.overrides.hub.event_bytes: {expected}\n");
        assert_eq!(lines(&found).as_ref(), expected.as_bytes());
    }
}

#[test]
fn payload_checkout_refusals_name_the_configured_and_required_bytes() {
    let mut limits = worker_limits();
    limits.agent.call_bytes = 8194;
    limits.checkout.message_bytes = 4098;
    let mut found = Problems::empty();
    check_worker(&limits, &mut found);
    assert_eq!(lines(&found).as_ref(), b"application.limits.overrides.checkout.message_bytes: 4098 bytes, short of the agent's call_bytes (agent.json profile.declared.tool_payload), 8194 bytes\n");
}

#[test]
fn payload_turn_queue_refusals_name_the_configured_and_required_bytes() {
    let mut limits = worker_limits();
    limits.agent.turns = 2;
    limits.host.turns = 2;
    limits.host.turn_bytes = 100;
    limits.host.turn_queue_bytes = 199;
    let mut found = Problems::empty();
    check_worker(&limits, &mut found);
    assert_eq!(lines(&found).as_ref(), b"workers[0].limits.overrides.hub.turn_queue_bytes: 199 bytes, short of the agent's turns * workers[0].limits.overrides.hub.turn_bytes, 200 bytes\n");
}
