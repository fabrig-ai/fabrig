//! Converter stability through the story's provider and transcript codecs
//! (temper/docs/design/domain/testing.md, 4.1).
use super::{
    CHAT, CODER, CUES, REVIEWER, TASKS, WORK, call, completion, first, fixture, records, save, save_first, text,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::{self as fake, api};
use skein_fake_llm_protocol::documents;
use skein_lib::{Env, Queue, Reader, ReplyTo, Time, Token, Wall, Writer};
use skein_llm::openai::{self, Output, Part};
use smith_transcript::{self as smith, Block, Turn, TurnParts};
use temper_twin::{Input, Replace, twin};

// The checked fixtures contain six chat, five work, four coding and one review turn.
const TURNS: [(u64, u32); 4] = [(CHAT, 6), (WORK, 5), (CODER, 4), (REVIEWER, 1)];
const NO_DROPS: [(u64, u32); 4] = [(CHAT, 0), (WORK, 0), (CODER, 0), (REVIEWER, 0)];
const CALL: Token = Token::new(1);
// The base model's output ceiling also admits the fixtures' individual payloads.
const MODEL_OUTPUT: u32 = 16_384;

fn wire_limits() -> documents::Limits {
    // The base document's model; recorded calls and reasoning fit a 16 KiB admission.
    let declared = skein_llm::Declared {
        window: 65_536,
        output: MODEL_OUTPUT,
        reasoning_item: MODEL_OUTPUT,
        tool_payload: MODEL_OUTPUT,
        calls_per_response: temper_fake::ENTRIES,
        conversations: 1,
        tools: temper_fake::ENTRIES,
        history_items: 256,
    };
    let limits = skein_llm::Limits::derive(&declared).expect("fixture model limits");
    documents::Limits {
        openai: limits.openai.native(),
        anthropic: limits.anthropic.native(),
        model_ceiling: declared.output,
    }
}

pub(super) fn decoded(answer: &api::Answer) -> (Vec<Block>, smith::Usage) {
    let limits = wire_limits();
    let mut decoder = openai::StreamDecoder::new(skein_llm::Provider::OpenAi, &limits.openai);
    let mut out = Queue::uncapped();
    let mut sequence = 0;
    while let Some(event) =
        documents::event(documents::Provider::OpenAi, answer, sequence, CALL.raw(), &limits).expect("provider event")
    {
        let json = openai::Json::from_bytes(&event.data, &limits.openai.document()).expect("event JSON");
        let event = openai::decode_event(&json, &limits.openai).expect("Responses event");
        decoder.event(event, &limits.openai, Wall::EPOCH, &mut out);
        while decoder.has_ready() {
            decoder.ready(&mut out);
        }
        sequence += 1;
    }
    decoder.end(&mut out);
    let mut blocks = Vec::new();
    let mut terminal = None;
    while let Some(output) = out.pop() {
        match output {
            Output::Part(Part::Text { text, .. }) => {
                blocks.push(super::text(std::str::from_utf8(&text).expect("text")));
            }
            Output::Part(Part::ToolCall { call_id, name, input, too_large, cut, .. }) => {
                assert!(!too_large && !cut, "whole scripted call");
                blocks.push(Block::Call(
                    smith::Call::new(&smith::CEILINGS, smith::CallParts { id: call_id, name, input, replay: None })
                        .expect("replayed call"),
                ));
            }
            Output::Completed { stop, usage } => {
                let expected = match answer.finish {
                    api::Finish::Stop => openai::Stop::EndTurn,
                    api::Finish::ToolCalls => openai::Stop::ToolUse,
                    api::Finish::Length | api::Finish::ContentFilter => panic!("whole converted completion"),
                };
                assert_eq!(stop, expected, "finish mode survives Responses");
                assert_eq!(usage.output, answer.usage.output, "output tokens survive Responses");
                assert!(terminal.replace(usage).is_none(), "one terminal per turn");
            }
            Output::Progress | Output::TextDelta { .. } | Output::ToolStart { .. } | Output::ArgumentsDelta { .. } => {}
            Output::Part(Part::Dropped { .. } | Part::Opaque { .. })
            | Output::ReasoningDelta { .. }
            | Output::Failed { .. } => panic!("unexpected scripted output: {output:?}"),
        }
    }
    let usage = terminal.expect("completion terminal");
    let usage = smith::Usage::new(
        &smith::CEILINGS,
        smith::UsageParts {
            input: usage.input,
            output: usage.output,
            cache_read: usage.cache_read,
            cache_write: usage.cache_write,
            reasoning: usage.reasoning,
        },
    )
    .expect("replayed usage");
    (blocks, usage)
}

fn replay(input: &Input, document: &[u8]) {
    let document = temper_fake::llm::Document::decode(document).expect("checked twin document");
    let mut domain = document.domain(17);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: temper_fake::llm::LIMITS };
    for take in &input.takes {
        let mut history = vec![api::Message {
            role: api::Role::User,
            parts: Box::new([api::Part::Text { text: b"replay".as_slice().into() }]),
        }];
        let mut turns = records(input, take.task);
        for record in &mut turns {
            let query = api::Query {
                credential: None,
                caching: api::Caching::Unscoped,
                model: b"fake".as_slice().into(),
                system: take.cue.clone(),
                tools: Box::new([]),
                choice: api::ToolChoice::Auto,
                messages: history.clone().into_boxed_slice(),
                max_tokens: MODEL_OUTPUT,
            };
            let mut out = Queue::uncapped();
            fake::step(
                &mut domain,
                &env,
                fake::Event::Call { reply_to: ReplyTo::new(CALL), call: CALL, query },
                &mut out,
            );
            fake::fire(&mut domain, &env, &mut out);
            let Some(fake::Request::Reply { to, result }) = out.pop() else { panic!("one whole scripted reply") };
            assert_eq!(to.into_token(), CALL);
            assert!(out.is_empty(), "one reply per turn");
            let answer = result.expect("scripted answer");
            let (blocks, usage) = decoded(&answer);
            let mut parts = Turn::decode(&smith::CEILINGS, &mut Reader::new(&record.transcript))
                .expect("recorded turn")
                .into_parts();
            completion(&mut parts, blocks);
            parts.usage = usage;
            let turn = Turn::new(&smith::CEILINGS, parts).expect("replayed turn");
            let mut writer = Writer::new(turn.measure() as usize);
            turn.encode(&mut writer).expect("Smith turn encoding");
            record.transcript = writer.finish();
            let mut results = Vec::new();
            for part in &answer.parts {
                if let api::Part::ToolCall { id, .. } = part {
                    results.push(api::Part::ToolOutput { id: id.clone(), output: Box::new([]), is_error: false });
                }
            }
            history.push(api::Message { role: api::Role::Assistant, parts: answer.parts });
            history.push(api::Message { role: api::Role::User, parts: results.into_boxed_slice() });
            domain.reclaim();
        }
        save(input, take.task, &turns);
    }
}

fn round_trip(mut input: Input, dropped: &[(u64, u32)], replaced: &[u32]) -> Value {
    let first = twin(&input).expect("first conversion");
    assert_eq!(&*first.report.turns, &TURNS);
    assert_eq!(&*first.report.dropped, dropped);
    assert_eq!(&*first.report.replaced, replaced);
    let before: Value = serde_json::from_slice(&first.document).expect("first document");
    for (script, cue) in before["cued"].as_array().expect("cued scripts").iter().zip(CUES) {
        assert_eq!(script["cue"], cue, "task/cue mapping");
    }
    replay(&input, &first.document);
    // Reapply the same replacement rules: their old words are gone from arguments and assistant text.
    input.base = first.document;
    let second = twin(&input).expect("conversion after replay");
    assert_eq!(&*second.report.turns, &TURNS);
    assert_eq!(&*second.report.dropped, &NO_DROPS);
    assert_eq!(&*second.report.replaced, vec![0; replaced.len()]);
    let after: Value = serde_json::from_slice(&second.document).expect("second document");
    // JSON values normalize the fake's compact argument encoding and generated call ids
    // are absent from scripts. Ordered text/calls, finishes, tokens and cues must agree.
    assert_eq!(after, before, "normalized document survives replay and conversion");
    after
}

#[test]
fn recorded_tasks_keep_their_scripts_through_a_round_trip() {
    let (_scratch, input) = fixture(&TASKS);
    round_trip(input, &NO_DROPS, &[]);
}

#[test]
fn dropped_and_replaced_blocks_stay_normalized_through_a_round_trip() {
    let (_scratch, mut input) = fixture(&TASKS);
    let mut parts: TurnParts = first(&input);
    let opaque =
        smith::Opaque::new(&smith::CEILINGS, smith::OpaqueParts { dialect: 1, bytes: b"opaque".as_slice().into() })
            .expect("opaque block");
    completion(
        &mut parts,
        vec![
            Block::Opaque(opaque.clone()),
            text("m24-before"),
            call(br#"{ "z": 2, "a": "m24-before m24-before" }"#),
            Block::Opaque(opaque),
            text("after"),
        ],
    );
    save_first(&input, parts);
    input.replaces = Box::new([Replace { old: b"m24-before".as_slice().into(), new: b"m24-after".as_slice().into() }]);
    let document = round_trip(input, &[(CHAT, 2), (WORK, 0), (CODER, 0), (REVIEWER, 0)], &[3]);
    assert_eq!(
        document["cued"][0]["turns"][0],
        json!({"lines":[
            {"text":"m24-after"},
            {"call":"delegate", "arguments":{"z":2,"a":"m24-after m24-after"}},
            {"text":"after"}
        ], "finish":"tool_calls", "tokens":20})
    );
}

#[test]
fn text_and_missing_tokens_stay_normalized_through_a_round_trip() {
    let (_scratch, input) = fixture(&TASKS);
    let mut parts = first(&input);
    completion(&mut parts, vec![text("CSV reply")]);
    let mut usage = parts.usage.into_parts();
    usage.output = None;
    parts.usage = smith::Usage::new(&smith::CEILINGS, usage).expect("missing output tokens");
    save_first(&input, parts);
    let document = round_trip(input, &NO_DROPS, &[]);
    assert_eq!(document["cued"][0]["turns"][0], json!({"lines":[{"text":"CSV reply"}], "finish":"stop", "tokens":1}));
}
