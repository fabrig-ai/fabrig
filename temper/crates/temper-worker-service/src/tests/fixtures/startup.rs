#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]
use crate::settings;
use jig_domain_link::{Bounds, EndpointBound};
use skein_lib::{List, Secret};
pub fn worker_alone_document() -> Box<[u8]> {
    let document = br#"{"workers":[{
"engine":{"socket":"127.0.0.1:7100"},"name":"worker-a","secret":"worker-secret","slots":1,
"graces":{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000},
"programs":{"check_interval_ms":60000},
"agent":{"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["internal", "agent","/opt/temper/agent.json"]},
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}},
"memory_bytes":1152921504606846976}],"application":{"workers":[{"root":"worker-root","git":"/usr/bin/git"}],"limits":{"profile":"small","overrides":{}}}}"#;
    with(document, b"workers[0].limits.overrides", br#"{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}"#)
}
pub fn with(input: &[u8], path: &[u8], json: &[u8]) -> Box<[u8]> {
    let json = turn_limits(path, json);
    let span = member_span(input, path);
    let mut result = input[..span.start].to_vec();
    result.extend_from_slice(&json);
    result.extend_from_slice(&input[span.end..]);
    result.into_boxed_slice()
}

fn member_span(input: &[u8], path: &[u8]) -> core::ops::Range<usize> {
    let document = jig_protocol_config::document(
        input,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    member_span_in(input, &document, path)
}
fn member_span_in(input: &[u8], document: &skein_json::Document, path: &[u8]) -> core::ops::Range<usize> {
    use jig_protocol_config::read;
    let mut at = 0;
    for member in core::str::from_utf8(path).unwrap().split('.') {
        let (member, index) = match member.split_once('[') {
            Some((member, index)) => (member, Some(index.trim_end_matches(']').parse::<usize>().unwrap())),
            None => (member, None),
        };
        at = read::Object::new(document, at).unwrap().member(document, member.as_bytes()).unwrap();
        if let Some(index) = index {
            at = *read::list(document, at, 16_384).unwrap().get(u32::try_from(index).unwrap()).unwrap();
        }
    }
    let after = read::after(document, at).unwrap();
    let mut token = 0;
    let mut start = None;
    let mut end = None;
    let mut offset = 0_usize;
    // Compact tokens hold decoded text offsets, so recover their raw spans.
    while offset < input.len() {
        let byte = input[offset];
        if byte.is_ascii_whitespace() || byte == b',' || byte == b':' {
            offset = offset.checked_add(1).unwrap();
            continue;
        }
        if token == at {
            start = Some(offset);
        }
        if token == after {
            end = Some(offset);
            break;
        }
        if byte == b'"' {
            offset = offset.checked_add(1).unwrap();
            while input[offset] != b'"' {
                if input[offset] == b'\\' {
                    offset = offset.checked_add(1).unwrap();
                }
                offset = offset.checked_add(1).unwrap();
            }
            offset = offset.checked_add(1).unwrap();
        } else if b"{}[]".contains(&byte) {
            offset = offset.checked_add(1).unwrap();
        } else {
            while offset < input.len() && !b",:}] \n\r\t".contains(&input[offset]) {
                offset = offset.checked_add(1).unwrap();
            }
        }
        token = token.checked_add(1).unwrap();
        if token == after {
            end = Some(offset);
            break;
        }
    }
    start.unwrap()..end.unwrap_or(input.len())
}

fn turn_limits(path: &[u8], json: &[u8]) -> Box<[u8]> {
    if path != b"workers[0].limits.overrides" {
        return json.into();
    }
    let turn = super::agent().limits.bodies.turn_body;
    let keys = &[
        ("hub.turn_bytes", u64::from(turn)),
        ("hub.transcript_bytes", u64::from(turn.checked_add(4).unwrap()) * 4),
        ("hub.turn_queue_bytes", u64::from(turn) * 8),
    ];
    let mut text = core::str::from_utf8(json).unwrap().trim().trim_end_matches('}').to_string();
    for (key, value) in keys {
        let mut quoted = String::from("\"");
        quoted.push_str(key);
        quoted.push('"');
        if !text.contains(&quoted) {
            if text.trim() != "{" {
                text.push(',');
            }
            text.push_str(&quoted);
            text.push(':');
            text.push_str(&value.to_string());
        }
    }
    text.push('}');
    text.into_bytes().into_boxed_slice()
}
/// The owned example's worker and application limits, in the worker-only shape.
#[expect(clippy::disallowed_macros, reason = "the regression reads the shipped example as fixture data")]
pub fn owned_worker_alone_document() -> Box<[u8]> {
    let owned = include_bytes!("../../../../../docs/examples/owned.json");
    let mut input = Vec::from(b"{\"workers\":".as_slice());
    input.extend_from_slice(&owned[member_span(owned, b"workers")]);
    input.extend_from_slice(b",\"application\":{\"workers\":");
    input.extend_from_slice(&owned[member_span(owned, b"application.workers")]);
    input.extend_from_slice(b",\"limits\":");
    input.extend_from_slice(&owned[member_span(owned, b"application.limits")]);
    input.extend_from_slice(b"}}");
    let input = with(&input, b"workers[0].engine", br#"{"socket":"127.0.0.1:7100"}"#);
    let input = with(&input, b"workers[0].name", br#""w1""#);
    with(&input, b"workers[0].secret", br#""secrets/w1.secret""#)
}

/// The host/channel metadata from owned/agent.json's standard profile, with
/// calm-world run bounds. The sans-io service has no shell/profile reader;
/// the process regression reads the example's agent document.
#[expect(clippy::too_many_lines, reason = "one typed snapshot of the example's agent metadata")]
pub fn owned_worker_agent() -> settings::Agent {
    let mut endpoints = List::with_capacity(3);
    endpoints
        .push(smith_protocol_channel::Endpoint { name: b"openai".as_slice().into(), number: 1, dialect: 3 })
        .unwrap();
    settings::Agent {
        run_limits: smith_agent_world::LIMITS.run,
        tool_payload: 4_098,
        programs: Box::new([settings::Program {
            adapter: settings::Adapter::ClaudeCode,
            path: b"/usr/local/bin/claude".as_slice().into(),
            directory: b"/home/me/temper/programs".as_slice().into(),
        }]),
        endpoints: smith_protocol_channel::Endpoints::new(endpoints),
        limits: smith_host_protocol::Limits {
            bodies: smith_channel::Limits {
                directory_name: 256,
                workspace_directories: 4,
                host_reply_text: 4_098,
                message_text: 4_098,
                start_charter: 131_072,
                start_transcript: 70_266_672,
                start_answered: 16,
                start_grants: 8,
                start_messages: 8,
                host_ask_input: 4_098,
                field_text: 4_096,
                turn_body: 1_589_621,
                accepted_result: 8_192,
                ..smith_channel::CEILINGS
            },
            charter: smith_charter::CEILINGS,
            transcript: smith_transcript::v7::Limits {
                replay_bytes: 93,
                text_text: 400_421,
                opaque_bytes: 262_151,
                call_input: 26_148,
                entry_name: 256,
                hit_path: 256,
                hit_text: 16_384,
                read_content: 4_098,
                found_hits: 17_584,
                command_end_head: 16_384,
                command_end_last: 49_152,
                command_running_head: 16_384,
                command_running_last: 49_152,
                job_read_head: 16_384,
                job_read_last: 49_152,
                ambiguous_lines: 8,
                working_path_path: 256,
                other_path_directory: 256,
                other_path_path: 256,
                outside_path_path: 256,
                said_text: 459_008,
                allowed_calls_names: 132,
                message_blocks: 4,
                turn_messages: 4,
                ..smith_transcript::CEILINGS
            },
            endpoints: 3,
            channel: skein_channel::Limits {
                chunk: 4096,
                credential: 0,
                skip: 72_105_650,
                output_bytes: 72_105_658,
                output_frames: 26,
                kinds: 20,
            },
            start_bytes: 72_105_650,
            calls: 1,
        },
        bounds: Bounds {
            program_endpoints: Box::new([]),
            endpoints: Box::new([EndpointBound { endpoint: 1, window: 1_048_576, output: 8192 }]),
            turns: 16,
            spend: 1_000_000_000,
            time: skein_lib::Duration::from_secs(86_400),
            waiting: skein_lib::Duration::from_secs(86_400),
            checks: skein_lib::Duration::from_secs(60),
            message_bytes: 4096,
            inbox: 8,
            unacknowledged: 8,
            reply_bytes: None,
        },
        host_limits: smith_host_domain::Limits {
            agents: 1,
            directories: 4,
            conflicts: 1024,
            path_bytes: 4096,
            name_bytes: 256,
            accounts: 4,
            charter_bytes: 65_536,
            transcript_bytes: 34_989_934,
            answered_bytes: 549_888,
            message_bytes: 4098,
            messages: 8,
            calls: 1,
            call_bytes: 4098,
            answer_bytes: 34_304,
            turns: 8,
            turn_bytes: 34_989_934,
            unacknowledged_bytes: 34_989_934,
            fact_bytes: 17,
            outcome_bytes: 4096,
            detail_bytes: 1024,
            spawn_timeout: skein_lib::Duration::from_secs(20),
            no_progress: skein_lib::Duration::from_secs(35),
            silence: skein_lib::Duration::from_secs(30),
            long_span: skein_lib::Duration::from_secs(86_400),
            long_open: 3,
            wall_time: skein_lib::Duration::from_secs(86_400),
            cancel_grace: skein_lib::Duration::from_secs(10),
            exit_grace: skein_lib::Duration::from_secs(15),
            teardown: skein_lib::Duration::from_secs(10),
            kill_after: skein_lib::Duration::from_secs(2),
            facts: 1024,
        },
    }
}

pub fn startup_agent() -> settings::Agent {
    let mut agent = super::agent();
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    agent.host_limits.directories = 4;
    agent.host_limits.conflicts = 1024;
    agent.host_limits.name_bytes = 600;
    agent.host_limits.accounts = hub.accounts;
    agent.host_limits.messages = hub.held.max(hub.inbox);
    agent.host_limits.call_bytes = 256;
    agent.host_limits.message_bytes = hub.event_bytes.checked_add(temper_worker_domain::SEPARATOR).unwrap();
    agent.bounds.message_bytes = 512;
    agent.bounds.spend = 43_584;
    agent.bounds.endpoints = Box::new([EndpointBound { endpoint: 1, window: 1_048_576, output: 8192 }]);
    agent
}

pub fn files() -> settings::Files {
    let mut entries = List::with_capacity(8);
    for (path, value) in [
        (b"worker-secret".as_slice(), super::worker::WORKER_SECRET),
        (
            b"secrets/worker-a.digest".as_slice(),
            b"8f7be71db3941de11249a5e3fba7750a41c7d9958083ef522cb79e9fd7822878".as_slice(),
        ),
        (super::AGENT_JSON, b"{}".as_slice()),
    ] {
        entries.push((path.into(), Secret::new(value.into()))).unwrap();
    }
    settings::Files { entries }
}
