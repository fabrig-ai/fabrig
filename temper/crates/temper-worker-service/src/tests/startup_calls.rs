//! The worker's composed call limits (temper/docs/design/protocol/services.md, 5).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary tests edit configuration fixtures")]
#![expect(clippy::disallowed_macros, reason = "ordinary tests format explicit scalar overrides")]
use super::fixtures;
use crate::settings;
use crate::startup::worker::check_worker_part;
use crate::startup::*;
use jig_protocol_config as config;
use skein_lib::List;
#[test]
fn worker_only_runtime_and_checks_use_the_same_composed_limits() {
    let directories = config::Directories::working();
    for (bound, expected) in [(None, 69_632), (Some(40_960), 40_960)] {
        let input = worker_document(bound);
        let mut agent = fixtures::startup_agent();
        agent.tool_payload = 40_960;
        let part = worker_part(&input, &directories).unwrap();
        let service = worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7).unwrap();
        check(&input, &directories, &fixtures::files(), &agent).unwrap();
        fixtures::assert_worker_only(&service);
        let mut part = worker_part(&input, &directories).unwrap();
        check_worker_part(&mut part, &fixtures::files(), &agent).unwrap();
        assert_eq!(part.worker.jig.limits.core.call_answer_bytes, expected);
        let payload = crate::worker_settings::payload_limits(&part.limits).unwrap();
        let wire = crate::worker_settings::link_limits(&part.worker.jig, &payload, part.limits.peer.forges).unwrap();
        assert_eq!((wire.sizes.call_input, wire.sizes.answer), (expected, expected));
        assert_eq!(service.worker.env.limits, crate::worker_settings::root_limits(&part.worker, &agent, &part.limits));
    }
}

#[test]
fn explicit_short_call_inputs_refuse_every_worker_only_entrance() {
    let directories = config::Directories::working();
    for (bound, payload) in [(4095, 4096), (32_768, 40_960)] {
        let input = worker_document(Some(bound));
        let mut agent = fixtures::startup_agent();
        agent.tool_payload = payload;
        let expected =
            call_input_lines("workers[0].limits.overrides.core.call_answer_bytes", bound, payload, 4096, 65_536);
        let found = [
            worker_alone(
                worker_part(&input, &directories).unwrap(),
                &fixtures::files(),
                &agent,
                fixtures::worker_runtime(),
                7,
            )
            .err()
            .unwrap(),
            check(&input, &directories, &fixtures::files(), &agent).unwrap_err(),
        ];
        for problems in found {
            assert_eq!(core::str::from_utf8(&lines(&problems)).unwrap(), expected);
            assert_eq!(problems.problems.len(), 1);
        }
    }
}

#[test]
fn per_kind_terms_reject_call_and_reply_mismatches_even_with_larger_other_frames() {
    use jig_protocol_worker_wire as wire;
    use skein_channel::{Role, Term};
    let worker = fixtures::worker_settings();
    let own = fixtures::worker_limits();
    let payload = crate::worker_settings::payload_limits(&own).unwrap();
    let mut link =
        crate::worker_settings::link_limits(&worker.jig, &payload, fixtures::worker_limits().peer.forges).unwrap();
    link.sizes.call_input = 40_960;
    link.sizes.answer = 40_960;
    let larger = wire::schema(&wire::sizes::limits(&link.sizes).unwrap()).unwrap();
    link.sizes.call_input = 32_768;
    link.sizes.answer = 32_768;
    let smaller = wire::schema(&wire::sizes::limits(&link.sizes).unwrap()).unwrap();
    for (role, kind) in [(Role::Initiator, 0x0102), (Role::Responder, 0x0182)] {
        for short in [true, false] {
            let mut terms = List::with_capacity(u32::try_from(wire::KINDS.len()).unwrap());
            for entry in &larger.version(wire::VERSION).unwrap().kinds {
                let largest = if short && entry.kind == kind {
                    smaller.version(wire::VERSION).unwrap().kind(kind).unwrap().largest
                } else {
                    entry.largest
                };
                terms.push(Term { kind: entry.kind, largest }).unwrap();
            }
            assert_eq!(wire::terms_gap(&larger, wire::VERSION, role, &terms), if short { Some(kind) } else { None });
        }
    }
}

fn worker_document(bound: Option<u32>) -> Box<[u8]> {
    let input = fixtures::worker_alone_document();
    let replacement = bound.map_or_else(String::new, |bytes| format!(",\"core.call_answer_bytes\":{bytes}"));
    core::str::from_utf8(&input)
        .unwrap()
        .replace(",\"core.call_answer_bytes\":2048", &replacement)
        .into_bytes()
        .into_boxed_slice()
}

fn worker_refusals(input: &[u8], agent: &settings::Agent) -> [Problems; 2] {
    let directories = config::Directories::working();
    [
        worker_alone(
            worker_part(input, &directories).unwrap(),
            &fixtures::files(),
            agent,
            fixtures::worker_runtime(),
            7,
        )
        .err()
        .unwrap(),
        check(input, &directories, &fixtures::files(), agent).unwrap_err(),
    ]
}

fn call_input_lines(path: &str, bytes: u32, input: u32, ask: u32, reply: u32) -> String {
    format!(
        "{path}: {bytes} bytes, short of the agent's call input (profile.declared.tool_payload), {input} bytes; raise it to at least {input} bytes, or omit it to follow the agent's call input\n{path}: call input {ask} bytes; wire answer {bytes} bytes; smith host_reply_bytes {reply} bytes\n"
    )
}

fn answer_floor_line(path: &str, bytes: u32) -> String {
    format!("{path}: {bytes} bytes, short of temper's documentation answer (read_docs), 2048 bytes\n")
}

#[test]
fn a_workers_call_answer_bound_must_hold_the_documentation_answer() {
    let agent = fixtures::startup_agent();
    let expected = call_input_lines("workers[0].limits.overrides.core.call_answer_bytes", 2047, 2048, 4096, 65_536)
        + &answer_floor_line("workers[0].limits.overrides.core.call_answer_bytes", 2047);
    for found in worker_refusals(&worker_document(Some(2047)), &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    let service = worker_alone(
        worker_part(&worker_document(Some(2048)), &config::Directories::working()).unwrap(),
        &fixtures::files(),
        &agent,
        fixtures::worker_runtime(),
        7,
    )
    .unwrap();
    fixtures::assert_worker_only(&service);
}

#[test]
fn a_local_agents_tool_payload_must_hold_the_documentation_answer() {
    let mut agent = fixtures::startup_agent();
    agent.tool_payload = 2047;
    let expected = answer_floor_line("workers[0].agent.profile.declared.tool_payload", 2047);
    for found in worker_refusals(&worker_document(Some(2048)), &agent) {
        assert_eq!(core::str::from_utf8(&lines(&found)).unwrap(), expected);
    }
    agent.tool_payload = 2048;
    let directories = config::Directories::working();
    worker_alone(
        worker_part(&worker_document(Some(2048)), &directories).unwrap(),
        &fixtures::files(),
        &agent,
        fixtures::worker_runtime(),
        7,
    )
    .unwrap();
}
