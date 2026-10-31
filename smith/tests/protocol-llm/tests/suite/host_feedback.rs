//! Original root Start, opaque host effect and cancellation over observed byte peers.
//! Observations stay outside the domains: native continued arguments have explicit
//! handwritten expectations; incoming host input remains byte exact.
//! Contract: domain/run.md,
//! sections 5.2, 10 and 13; testing-strategy.md, sections 2.3 and 6.

use skein_fake_llm_domain::api::{Line, Part, Query, Script};
use skein_lib::{Time, Wall};
use skein_world::domain::Span;
use smith_domain::{run, session::llm};
use smith_protocol_llm_world::codec::{self as codec};
use smith_protocol_llm_world::{
    Job, Settings, Terminal, World,
    client_peer::{self, Configuration},
};

const BODY: &[u8] = br#"{ "opaque" : {"future":[1,true,null]}, "extra":"unchanged" }"#;
const ID: &[u8] = b"call_0000000000000001";
const FEEDBACK: &[u8] = b"opaque host answer: first decision";

fn scripts(tool_name: &[u8]) -> Box<[Script]> {
    Box::new([Script {
        cue: b"@hosttools".as_slice().into(),
        turns: Box::new([
            smith_protocol_llm_world::fixture::calls(
                Box::new([Line::Call { name: tool_name.into(), arguments: BODY.into() }]),
                8,
            ),
            smith_protocol_llm_world::fixture::says(
                Box::new([Line::Text { text: b"observed continued result".as_slice().into() }]),
                4,
            ),
        ]),
    }])
}

fn feedback(query: &Query, arguments: &[u8], tool_name: &[u8]) -> bool {
    let [_, assistant, user] = query.messages.as_ref() else { return false };
    let [Part::ToolCall { id, name, arguments: observed }] = assistant.parts.as_ref() else { return false };
    let [Part::ToolOutput { id: returned, output, is_error }] = user.parts.as_ref() else { return false };
    id.as_ref() == ID
        && returned.as_ref() == ID
        && name.as_ref() == tool_name
        && observed.as_ref() == arguments
        && output.as_ref() == FEEDBACK
        && !is_error
}

fn host_boundary(world: &World, expected_arguments: &[u8], tool_name: &[u8]) {
    let first = &world.prompts()[0];
    assert!(skein_lib::bytes::find(&first.system, b"@hosttools").is_some());
    assert!(
        skein_lib::bytes::find(&first.system, b"The answer is in src/lib.rs; the checks want it to be 43.\n").is_some(),
        "the discovered guide reaches the first observed system text: {first:?}"
    );
    let read = world
        .trace()
        .iter()
        .position(|line| line.contains("agent -> Lower(Read {"))
        .expect("observed discovery Read request");
    let returned = world
        .trace()
        .iter()
        .position(|line| line.contains("agent <- Below(Read {"))
        .expect("observed discovery Read terminal");
    let complete = world
        .trace()
        .iter()
        .position(|line| line.contains("agent -> Lower(Complete {"))
        .expect("observed root Complete request");
    assert!(
        read < returned && returned < complete,
        "original discovery request and terminal precede observed Complete: {:?}",
        world.trace()
    );
    let host = first.tools.iter().find(|tool| tool.name.as_ref() == tool_name).expect("declared opaque host tool");
    assert_eq!(host.description.as_ref(), b"Opaque host write");
    assert_eq!(host.parameters.as_ref(), br#"{"type":"object"}"#);
    let [submission] = world.host_submissions() else { panic!("one observed host submission") };
    assert_eq!(submission.tool.as_ref(), tool_name);
    assert_eq!(submission.effect, run::HostEffect::Write);
    assert_eq!(submission.input.bytes(), BODY);
    let [(_, _, run::HostReply::Answered(answer))] = world.host_terminals() else { panic!("one observed host answer") };
    assert_eq!(answer.text(), FEEDBACK);
    assert!(!answer.error());
    assert!(feedback(&world.prompts()[1], expected_arguments, tool_name));
    let mut missing = world.prompts()[1].clone();
    missing.messages[2].parts = Box::new([]);
    assert!(!feedback(&missing, expected_arguments, tool_name));
    let mut rewritten = world.prompts()[1].clone();
    rewritten.messages[2].parts =
        Box::new([Part::ToolOutput { id: ID.into(), output: b"rewritten".as_slice().into(), is_error: false }]);
    assert!(!feedback(&rewritten, expected_arguments, tool_name));
}

fn retired_boundary(world: &World, tool_name: &[u8]) {
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::Failure::Cancelled, .. }));
    let [first_binding, second_binding] = world.protocol_calls() else { panic!("settled calls") };
    let owner = first_binding.owner;
    assert_eq!(first_binding.terminal.map(|(_, _, event)| event), Some(Terminal::Completed(owner)));
    assert_eq!(second_binding.terminal.map(|(_, _, event)| event), Some(Terminal::Cancelled(owner)));
    assert!(first_binding.terminal.is_some());
    assert_eq!(second_binding.terminal.expect("component terminal").1, Wall::from_nanos(3000));
    assert!(second_binding.terminal.iter().all(|(_, wall, _)| *wall == Wall::from_nanos(3000)));
    assert!(world.turns().iter().flat_map(|turn| &turn.messages).flat_map(|message| &message.content).any(|block| {
        matches!(block, llm::Block::ToolCall { id, name, input, .. } if id.as_ref() == ID && name.as_ref() == tool_name && input.as_ref() == BODY)
    }), "root-owned concrete Turn retains the incoming delta bytes");
}

fn root_story(configuration: Configuration, tool_name: &'static [u8]) {
    let expected_arguments = configuration.continuation_arguments.clone();
    let mut bounds =
        smith_protocol_llm::fixture::small(skein_llm_world::limits(), smith_protocol_llm_world::codec::stated());
    bounds.client.http.request = 16384;
    bounds.client.request = 16384;
    let mut settings = Settings {
        job: Job::HostTools,
        writable: false,
        network: Span::millis(1, 1),
        ..smith_protocol_llm_world::fixture::settings(71)
    };
    settings.host_tool_name = tool_name;
    settings.limits.session.completion_bytes =
        codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes)
            .expect("complete observed translated reservation");
    settings.limits.session.completion_blocks = bounds.client.output_items;
    settings.limits.session.failure_bytes = bounds.client.detail_bytes;
    let expected_receiving = settings.limits;
    let mut world = World::with_protocol(settings, None, configuration, bounds, scripts(tool_name));
    world.wall_at(Wall::from_nanos(9000));
    for _ in 0..100_000 {
        assert!(!world.drive(1), "continued query precedes parent cancellation");
        if world.prompts().len() == 2 {
            break;
        }
    }
    assert_eq!(world.prompts().len(), 2, "both queries were parsed by observed byte peers");
    assert!(world.now() < Time::from_nanos(1_000_000_000), "queued wire bytes prevent jumping to the run deadline");
    host_boundary(&world, &expected_arguments, tool_name);
    let [first_binding, second_binding] = world.protocol_calls() else { panic!("two observed calls") };
    assert_eq!(first_binding.owner, second_binding.owner, "the root reuses the logical callback");
    assert_eq!(first_binding.receiving.max_completion_bytes, expected_receiving.session.completion_bytes);
    assert_eq!(first_binding.receiving.max_completion_blocks, expected_receiving.session.completion_blocks);
    assert_eq!(first_binding.receiving.max_failure_bytes, expected_receiving.session.failure_bytes);
    assert_eq!(first_binding.receiving.decoded_call_bytes, expected_receiving.decoded_call_bytes);
    assert_eq!(first_binding.started.1, Wall::from_nanos(9000));
    assert_eq!(second_binding.started.1, Wall::from_nanos(9000));
    world.wall_at(Wall::from_nanos(3000));
    world.cancel_run();
    world.run(100_000);
    retired_boundary(&world, tool_name);
    assert!(
        feedback(&world.prompts()[1], &expected_arguments, tool_name),
        "settlement preserves the positive outside observation"
    );
}

#[test]
fn root_wire_host_feedback_and_reused_owner_close_survive_parent_cancel() {
    for configuration in client_peer::configurations() {
        root_story(configuration, b"host_action");
    }
}

#[test]
fn root_wire_message_host_feedback_reaches_continuation() {
    for configuration in client_peer::configurations() {
        root_story(configuration, b"message");
    }
}
