//! Original Start/discovery, observed native replay, parking and root restoration.
//! Caller fixtures supply opaque native data and handwritten outside expectations;
//! Skein alone owns their grammar. Every historical body comes from an observed
//! root Turn, including the independently saved concrete post-transcript result.
//! Contract: domain/run.md, section 6 and domain/budget.md and domain/run.md, sections 9 and 12
//! ; domain/session.md, section 3; testing-strategy.md, sections 2.3 and 6.

use skein_fake_llm_domain::api::{Line, Message, Part, Query, Role, Script, Turn};
use skein_lib::{Duration, Time, Token};
use skein_world::domain::Span;
use smith_domain::{Answered, AnsweredCall, Transcript, run, session::llm};
use smith_protocol_llm_world::codec::{self as codec, Limits};
use smith_protocol_llm_world::{
    Job, Settings, World,
    client_peer::{self, Configuration},
};

const BEGIN: &[u8] = b"Begin the work your brief describes.";
const ID: &[u8] = b"call_0000000000000001";
const FIRST: &[u8] = b"First activation parked.";
const RESUMED: &[u8] = b"Observed resumed activation.";
const LAST: &[u8] = b"Second activation parked.";
const PERSON: &[u8] = b"host: native wake";
const WAITED: &[u8] = b"Messages arrived. Read them before continuing.";
const OPENAI_INPUT: &[u8] = br#"{"type":"reasoning","id":"opaque_root","encrypted_content":"encrypted-root","summary":[],"future":{"proof":[1,true,null],"nonce":"root-proof"}}"#;
// OpenAI retains its known reasoning fields; the scripted future field is skipped.
const OPENAI_OPAQUE: &[u8] =
    br#"{"type":"reasoning","id":"opaque_root","encrypted_content":"encrypted-root","summary":[]}"#;
const ANTHROPIC_OPAQUE: &[u8] = br#"{"type":"thinking","future":{"proof":[1,true,null],"nonce":"root-proof"},"thinking":"Plan for parking","signature":"signed-root"}"#;
const OPENAI_ENVELOPE: &[u8] = b"\x00\x01\x03\x00\x00\x00\x59{\"type\":\"reasoning\",\"id\":\"opaque_root\",\"encrypted_content\":\"encrypted-root\",\"summary\":[]}";
const ANTHROPIC_ENVELOPE: &[u8] = b"\x00\x01\x02\x00\x00\x00\x81{\"type\":\"thinking\",\"future\":{\"proof\":[1,true,null],\"nonce\":\"root-proof\"},\"thinking\":\"Plan for parking\",\"signature\":\"signed-root\"}";
const CALL_REPLAY: &[u8] = b"\x00\x01\x03\x00\x00\x00\x16{\"item_id\":\"item_1_1\"}";
const SECOND_TEXT_REPLAY: &[u8] = b"\x00\x01\x03\x00\x00\x00\x28{\"id\":\"item_2_0\",\"phase\":\"final_answer\"}";
const TEXT_REPLAY: &[u8] = b"\x00\x01\x03\x00\x00\x00\x28{\"id\":\"item_1_0\",\"phase\":\"final_answer\"}";

struct Fixture {
    input: &'static [u8],
    opaque: &'static [u8],
    envelope: &'static [u8],
    call_replay: Option<&'static [u8]>,
    text_replay: Option<&'static [u8]>,
    markers: &'static [&'static [u8]],
    cache_write: bool,
    reasoning: Option<u64>,
}

fn fixtures() -> [Fixture; 2] {
    [
        Fixture {
            input: OPENAI_INPUT,
            opaque: OPENAI_OPAQUE,
            envelope: OPENAI_ENVELOPE,
            call_replay: Some(CALL_REPLAY),
            text_replay: Some(TEXT_REPLAY),
            markers: &[b"opaque_root", b"summary", b"type", b"encrypted-root"],
            cache_write: true,
            reasoning: Some(0),
        },
        Fixture {
            input: ANTHROPIC_OPAQUE,
            opaque: ANTHROPIC_OPAQUE,
            envelope: ANTHROPIC_ENVELOPE,
            call_replay: None,
            text_replay: None,
            markers: &[b"root-proof", b"proof", b"type", b"signed-root"],
            cache_write: true,
            reasoning: None,
        },
    ]
}

fn waiting_turn(opaque: &[u8]) -> Turn {
    smith_protocol_llm_world::fixture::calls(
        Box::new([
            Line::Opaque { bytes: opaque.into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        17,
    )
}

fn resumed_turn() -> Turn {
    smith_protocol_llm_world::fixture::calls(
        Box::new([
            Line::Text { text: RESUMED.into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        11,
    )
}

fn text_turn(text: &[u8], tokens: u64) -> Turn {
    smith_protocol_llm_world::fixture::calls(
        Box::new([
            Line::Text { text: text.into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        tokens,
    )
}

fn scripts(fixture: &Fixture, tail: bool) -> Box<[Script]> {
    let mut turns = vec![waiting_turn(fixture.input)];
    if !tail {
        turns.push(text_turn(FIRST, 3));
    }
    turns.extend([resumed_turn(), text_turn(LAST, 5)]);
    Box::new([Script { cue: b"@waiting".as_slice().into(), turns: turns.into() }])
}

fn bounds() -> Limits {
    let mut bounds =
        smith_protocol_llm_fixtures::small(skein_llm_kit::limits(), smith_protocol_llm_world::codec::stated());
    bounds.client.http.request = 16384;
    bounds.client.request = 16384;
    bounds
}

fn settings(seed: u64, resume: bool, bounds: &Limits) -> Settings {
    let mut settings = Settings {
        job: Job::Waiting,
        resume,
        writable: false,
        waiting: Duration::from_secs(1),
        network: Span::millis(1, 1),
        ..smith_protocol_llm_world::fixture::settings(seed)
    };
    settings.limits.session.completion_bytes =
        codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes)
            .expect("complete observed translated reservation before Start");
    settings.limits.session.completion_blocks = bounds.client.output_items;
    settings.limits.session.failure_bytes = bounds.client.detail_bytes;
    settings
}

fn message(role: Role, parts: Vec<Part>) -> Message {
    Message { role, parts: parts.into() }
}

fn text(text: &[u8]) -> Part {
    Part::Text { text: text.into() }
}

fn call() -> Part {
    Part::ToolCall { id: ID.into(), name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }
}

fn feedback() -> Part {
    Part::ToolOutput { id: ID.into(), output: WAITED.into(), is_error: false }
}

fn settled_prefix(fixture: &Fixture) -> Vec<Message> {
    vec![
        message(Role::User, vec![text(BEGIN)]),
        message(Role::Assistant, vec![Part::Opaque { bytes: fixture.opaque.into() }, call()]),
        message(Role::User, vec![feedback()]),
    ]
}

fn prefix(fixture: &Fixture) -> Vec<Message> {
    let mut messages = settled_prefix(fixture);
    if fixture.call_replay.is_some() {
        messages[2].parts = vec![feedback(), text(PERSON)].into();
    } else {
        messages.push(message(Role::User, vec![text(PERSON)]));
    }
    messages
}

fn resumed_prefix(fixture: &Fixture, tail: bool) -> Vec<Message> {
    let mut messages = if tail { settled_prefix(fixture) } else { prefix(fixture) };
    if tail {
        if fixture.call_replay.is_some() {
            messages[2].parts = vec![feedback(), text(&waking_prompt())].into();
        } else {
            messages.push(message(Role::User, vec![text(&waking_prompt())]));
        }
    } else {
        messages.push(message(
            Role::Assistant,
            vec![
                text(FIRST),
                Part::ToolCall {
                    id: b"call_0000000000000002".as_slice().into(),
                    name: b"wait".as_slice().into(),
                    arguments: b"{}".as_slice().into(),
                },
            ],
        ));
        messages.push(message(
            Role::User,
            vec![Part::ToolOutput {
                id: b"call_0000000000000002".as_slice().into(),
                output: smith_agent_world::fixture::RESUME.into(),
                is_error: false,
            }],
        ));
    }
    messages
}

fn waking_prompt() -> Vec<u8> {
    let mut text = b"Earlier host answers absent from the saved transcript:\ncall activation=1 completion=2 position=1 tool=wait result: Messages arrived. Read them before continuing.\n\n".to_vec();
    text.extend_from_slice(smith_agent_world::fixture::RESUME);
    text
}

fn exact_prefix(query: &Query, expected: &[Message]) -> bool {
    query.model.as_ref() == b"fake-1" && query.messages.as_ref() == expected
}

fn prefix_corruptions(query: &Query, fixture: &Fixture, expected: &[Message]) {
    assert!(exact_prefix(query, expected), "whole positive observed native prefix: {query:?}");
    for &marker in fixture.markers {
        let mut changed = query.clone();
        let Part::Opaque { bytes } = &mut changed.messages[1].parts[0] else { panic!("observed opaque") };
        let at = bytes.windows(marker.len()).position(|part| part == marker).expect("positive opaque metadata");
        bytes[at] = b'X';
        assert!(!exact_prefix(&changed, expected), "rewritten native metadata {marker:?}");
    }
    let mut rewritten = query.clone();
    rewritten.messages[1].parts[0] = Part::Opaque { bytes: b"{}".as_slice().into() };
    assert!(!exact_prefix(&rewritten, expected));
    let mut missing = query.clone();
    missing.messages[1].parts = vec![call()].into();
    assert!(!exact_prefix(&missing, expected));
    let mut duplicate = query.clone();
    duplicate.messages[1].parts = vec![Part::Opaque { bytes: fixture.opaque.into() }, call(), call()].into();
    assert!(!exact_prefix(&duplicate, expected));
    for field in 0..6 {
        let mut changed = query.clone();
        match field {
            0 => changed.messages[1].role = Role::User,
            1 => changed.messages[2].role = Role::Assistant,
            2 => {
                let Part::ToolCall { id, .. } = &mut changed.messages[1].parts[1] else { panic!("observed call") };
                *id = b"rewritten-id".as_slice().into();
            }
            3 => {
                let Part::ToolOutput { id, .. } = &mut changed.messages[2].parts[0] else {
                    panic!("observed feedback")
                };
                *id = b"rewritten-id".as_slice().into();
            }
            4 => {
                let Part::ToolOutput { output, .. } = &mut changed.messages[2].parts[0] else {
                    panic!("observed feedback")
                };
                *output = b"rewritten feedback".as_slice().into();
            }
            5 => changed.messages[2].parts = Box::new([]),
            _ => panic!("bounded independent corruption"),
        }
        assert!(!exact_prefix(&changed, expected), "native prefix corruption {field}");
    }
}

fn replay(observed: Option<&llm::Replay>, expected: Option<&[u8]>) -> bool {
    observed.map(|replay| replay.bytes.as_ref()) == expected
}

fn recorded_wait(turn: &smith_domain::Turn, fixture: &Fixture, opaque: bool) -> bool {
    let (wake, assistant) = match turn.messages.as_ref() {
        [wake, assistant, result] if opaque => {
            if result.role != llm::Role::User
                || !matches!(result.content.as_ref(), [llm::Block::ToolResult { id, result: llm::Returned::Text { text, error: false, replay: None } }] if id.as_ref() == ID && text.as_ref() == WAITED)
            {
                return false;
            }
            (wake, assistant)
        }
        [wake, assistant] if !opaque => (wake, assistant),
        _ => return false,
    };
    let wake_matches = if opaque {
        matches!(wake.content.as_ref(), [llm::Block::Text { text, replay: None }] if text.as_ref() == BEGIN)
    } else {
        matches!(wake.content.as_ref(), [llm::Block::ToolResult { id, result: llm::Returned::Text { text, error: false, replay: None } }] if id.as_ref() == b"call_0000000000000002" && text.as_ref() == smith_agent_world::fixture::RESUME)
    };
    if wake.role != llm::Role::User || assistant.role != llm::Role::Assistant || !wake_matches {
        return false;
    }
    let [
        first,
        llm::Block::ToolCall(llm::ToolCall { id, name, input, call: llm::Decoded::Historical, replay: metadata }),
    ] = assistant.content.as_ref()
    else {
        return false;
    };
    let first_matches = if opaque {
        matches!(first, llm::Block::Opaque { bytes } if bytes.as_ref() == fixture.envelope)
    } else {
        matches!(first, llm::Block::Text { text, replay: metadata } if text.as_ref() == RESUMED && replay(metadata.as_ref(), fixture.text_replay))
    };
    first_matches
        && id.as_ref() == ID
        && name.as_ref() == b"wait"
        && input.as_ref() == b"{}"
        && replay(metadata.as_ref(), fixture.call_replay)
}

fn recorded_corruptions(turn: &smith_domain::Turn, fixture: &Fixture) {
    assert!(recorded_wait(turn, fixture, true), "positive full root-owned replay and concrete feedback");
    for at in [0, 1, 2, 6, 7] {
        let mut changed = turn.clone();
        let llm::Block::Opaque { bytes } = &mut changed.messages[1].content[0] else { panic!("observed envelope") };
        bytes[at] ^= 1;
        assert!(!recorded_wait(&changed, fixture, true), "version/tag/length/body corruption {at}");
    }
    let mut missing = turn.clone();
    missing.messages[1].content = Box::new([missing.messages[1].content[1].clone()]);
    assert!(!recorded_wait(&missing, fixture, true));
    let mut unresolved = turn.clone();
    let llm::Block::ToolCall(llm::ToolCall { call, .. }) = &mut unresolved.messages[1].content[1] else {
        panic!("observed historical")
    };
    *call = llm::Decoded::Delegated {
        descriptor: skein_lib::Token::new(99),
        source: smith_domain::session::ToolSource::Run,
        ticket: skein_lib::Token::new(99),
        effect: smith_domain::tools::Effect::Write,
    };
    assert!(!recorded_wait(&unresolved, fixture, true));
    if fixture.call_replay.is_some() {
        let mut missing = turn.clone();
        let llm::Block::ToolCall(llm::ToolCall { replay, .. }) = &mut missing.messages[1].content[1] else {
            panic!("observed call")
        };
        *replay = None;
        assert!(!recorded_wait(&missing, fixture, true));
    }
}

fn history(world: &World) -> Transcript {
    let first = world.turns().first().expect("observed root Turn");
    Transcript { version: first.version, endpoint: first.endpoint, dialect: first.dialect, turns: world.turns().into() }
}

// The parent retains an observed prefix and its concrete result separately;
// no completion, application classification, feedback or usage is manufactured.
fn post_tail(world: &World) -> (Transcript, AnsweredCall) {
    let mut saved = history(world);
    let first = saved.turns[0].clone();
    let result = first.messages.last().expect("observed saved Wait terminal").clone();
    saved.turns = Box::new([first]);
    let llm::Block::ToolResult { result: llm::Returned::Text { text, error, .. }, .. } = &result.content[0] else {
        panic!("saved host answer")
    };
    let answered = AnsweredCall {
        tool: b"wait".as_slice().into(),
        name: run::CallName { activation: 1, completion: saved.turns[0].sequence + 1, position: 1 },
        answer: Answered::Host(run::HostAnswer::new(text.clone(), *error).expect("saved host answer fits")),
    };
    (saved, answered)
}

fn part_bytes(part: &Part) -> usize {
    match part {
        Part::Text { text } | Part::Opaque { bytes: text } => text.len(),
        Part::ToolCall { id, name, arguments } => id.len() + name.len() + arguments.len(),
        Part::ToolOutput { id, output, .. } => id.len() + output.len(),
    }
}

fn expected_usage(
    query: &Query,
    expected: &[Message],
    previous: Option<&[Message]>,
    output: u64,
    fixture: &Fixture,
) -> [u64; 4] {
    let fixed = query.system.len()
        + query
            .tools
            .iter()
            .map(|tool| tool.name.len() + tool.description.len() + tool.parameters.len())
            .sum::<usize>();
    let tokens =
        u64::try_from((fixed + expected.iter().flat_map(|message| &message.parts).map(part_bytes).sum::<usize>()) / 4)
            .expect("bounded caller fixture");
    let read = previous.map_or(0, |messages| {
        u64::try_from((fixed + messages.iter().flat_map(|message| &message.parts).map(part_bytes).sum::<usize>()) / 4)
            .expect("previously written prompt")
    });
    let fresh = tokens - read;
    [if fixture.cache_write { 0 } else { fresh }, output, read, if fixture.cache_write { fresh } else { 0 }]
}

fn observe_usage(world: &World, fixture: &Fixture, expected_prompts: &[Vec<Message>], outputs: &[u64]) {
    assert_eq!(world.prompts().len(), expected_prompts.len());
    assert_eq!(world.turns().len(), expected_prompts.len());
    for (index, ((query, turn), expected_prompt)) in
        world.prompts().iter().zip(world.turns()).zip(expected_prompts).enumerate()
    {
        assert!(exact_prefix(query, expected_prompt), "observed query {index}: {query:?}");
        let wanted = expected_usage(
            query,
            expected_prompt,
            index.checked_sub(1).map(|previous| expected_prompts[previous].as_slice()),
            outputs[index],
            fixture,
        );
        let observed = [
            turn.usage.input_tokens.expect("SDK fixture reports usage"),
            turn.usage.output_tokens.expect("SDK fixture reports usage"),
            turn.usage.cache_read_tokens.expect("SDK fixture reports usage"),
            turn.usage.cache_write_tokens.expect("SDK fixture reports usage"),
        ];
        assert_eq!(observed, wanted, "all four observed SDK usage fields at completion {index}");
        for field in 0..4 {
            let mut changed = observed;
            changed[field] += 1;
            assert_ne!(changed, wanted, "usage field {field} participates in the oracle");
        }
        assert_eq!(turn.usage.reasoning_tokens, fixture.reasoning, "literal peer reasoning reporting");
    }
}

fn configuration(index: usize) -> Configuration {
    client_peer::configurations().into_iter().nth(index).expect("two caller native fixtures")
}

fn resume(first: &World, fixture: &Fixture, index: usize, tail: bool) {
    let (saved, answered) = if tail {
        let (saved, answered) = post_tail(first);
        (saved, Box::new([answered]) as Box<[_]>)
    } else {
        (history(first), Box::default())
    };
    let bounds = bounds();
    let settings = settings(920, true, &bounds);
    let mut world = World::with_protocol_answers(
        settings,
        Some(saved),
        answered,
        configuration(index),
        bounds,
        scripts(fixture, tail),
    );
    world.run(100_000);
    let prefix = resumed_prefix(fixture, tail);
    prefix_corruptions(&world.prompts()[0], fixture, &prefix);
    observe_usage(&world, fixture, &[prefix], &[11]);
    if !tail {
        assert!(recorded_wait(&world.turns()[0], fixture, false));
    }

    assert_eq!(first.turn_metadata()[1].2.output, Some(20), "saved activation is independently charged");
    assert_eq!(world.turn_metadata()[0].2.output, Some(11), "history is not recharged in the new activation");
}

#[test]
fn native_root_parks_then_restores_whole_opaque_prefix_and_real_post_tail() {
    for (index, fixture) in fixtures().into_iter().enumerate() {
        let bounds = bounds();
        let settings = settings(919, false, &bounds);
        let mut first = World::with_protocol(settings, None, configuration(index), bounds, scripts(&fixture, false));
        first.message_at(Time::from_nanos(500_000_000), Token::new(0), b"native wake".as_slice().into());
        first.run(100_000);
        observe_usage(&first, &fixture, &[vec![message(Role::User, vec![text(BEGIN)])], prefix(&fixture)], &[17, 3]);
        recorded_corruptions(&first.turns()[0], &fixture);
        assert!(
            matches!(first.turns()[1].messages[1].content.as_ref(), [llm::Block::Text { text, replay: metadata }, llm::Block::ToolCall { .. }]
            if text.as_ref() == FIRST && replay(metadata.as_ref(), fixture.text_replay.map(|_| SECOND_TEXT_REPLAY)))
        );

        resume(&first, &fixture, index, false);
        resume(&first, &fixture, index, true);
    }
}
