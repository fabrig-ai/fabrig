use super::*;
use crate::backend::fixtures::COMPLETION;
use fixtures::records as active;
use fixtures::{env, records_with as running, request};
use smith_domain::run;

#[test]
fn an_unframed_tool_message_completes_without_any_partial() {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    backend.line(&env(0),br#"{"type":"assistant","message":{"id":"unframed","content":[{"type":"tool_use","id":"t","name":"mcp__smith__finish","input":{}}],"usage":{"input_tokens":17,"output_tokens":3}}}"#,&mut up,&mut below);
    let Some(ToDomain::Completed { owner, completion }) = up.pop() else {
        panic!("unframed completion");
    };
    assert_eq!(owner, COMPLETION);
    assert_eq!(completion.usage.input_tokens, Some(17));
    assert!(backend.turns.message.is_none());
}

#[test]
fn an_open_next_message_does_not_receive_a_duplicate_nudge() {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    backend.line(
        &env(0),
        br#"{"type":"assistant","message":{"id":"one","content":[{"type":"text","text":"thinking"}]}}"#,
        &mut up,
        &mut below,
    );
    backend.line(
        &env(0),
        br#"{"type":"stream_event","event":{"type":"message_start","message":{"id":"next"}}}"#,
        &mut up,
        &mut below,
    );
    assert!(backend.current.owner().is_none(), "the first message was delivered");
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, owner, .. } = &mut next else {
        panic!("next completion");
    };
    *owner = Token::new(81);
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::Text { text: b"finish now".as_slice().into(), replay: None }]),
    }]);
    backend.resume(&env(0), Complete::from_request(next), &mut up, &mut below);
    assert!(backend.outgoing.is_none());
    assert_eq!(backend.current.owner(), Some(Token::new(81)));
    assert!(below.is_empty());
}

#[test]
fn a_line_outside_the_stream_grammar_is_a_program_failure() {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    backend.line(&env(0), br#"{"other":true}"#, &mut up, &mut below);
    let Some(ToDomain::Failed { failure, .. }) = up.pop() else {
        panic!("program failure");
    };
    assert_eq!(failure, llm::Failure::Program { exit: None });
}
#[test]
fn a_result_without_an_assistant_hands_the_nudge_to_the_domain() {
    let fixtures::Harness { mut backend, mut up, mut below } = fixtures::Harness::new(active());
    backend.line(&env(0), br#"{"type":"result","subtype":"success","is_error":false}"#, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else {
        panic!("no-tool completion");
    };
    assert!(completion.content.is_empty());
    assert_eq!(completion.stop, llm::Stop::EndTurn);
    assert_eq!(completion.usage, llm::Usage::ZERO);
}

fn sub_agent_call(input: &[u8], served: bool) -> llm::Decoded {
    let mut backend = active();
    let Some(Complete { prompt, .. }) = backend.current.request_mut() else {
        panic!("completion");
    };
    prompt.served = if served { Box::new([llm::Served::SubAgent]) } else { Box::default() };
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    let line = skein_lib::bytes::concat(&[
        br#"{"type":"assistant","message":{"id":"unframed","content":[{"type":"tool_use","id":"t","name":"mcp__smith__sub_agent","input":"#,
        input,
        b"}]}}",
    ]);
    backend.line(&env(0), &line, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else {
        panic!("completion");
    };
    let mut content = completion.content.into_iter();
    let llm::Said::ToolCall { call, .. } = content.next().expect("one tool call") else {
        panic!("tool call");
    };
    assert!(content.next().is_none());
    call
}

#[test]
fn a_served_sub_agent_call_decodes_inspect_only() {
    assert_eq!(
        sub_agent_call(br#"{"brief":"Read lib.rs","tools":["inspect"]}"#, true),
        llm::Decoded::Served {
            ask: run::Ask::SubAgent {
                brief: b"Read lib.rs".as_slice().into(),
                families: run::charter::Families {
                    tools: run::charter::Tools { inspect: true, modify: false, shell: false },
                    agents: false,
                },
                llm: None,
                share: run::Share { turns: None, time: None },
            }
        }
    );
}

#[test]
fn a_sub_agent_asking_to_write_is_invalid_naming_tools() {
    for input in [
        br#"{"brief":"Read lib.rs","tools":["inspect","modify"]}"#.as_slice(),
        br#"{"brief":"Read lib.rs","tools":["shell"]}"#,
    ] {
        assert_eq!(
            sub_agent_call(input, true),
            llm::Decoded::Invalid { problem: llm::Problem::BadValue { field: b"tools".as_slice().into() } }
        );
    }
}

#[test]
fn a_sub_agent_call_without_a_brief_is_missing_it() {
    assert_eq!(
        sub_agent_call(br#"{"tools":["inspect"]}"#, true),
        llm::Decoded::Invalid { problem: llm::Problem::Missing { field: b"brief".as_slice().into() } }
    );
}

#[test]
fn the_unoffered_sub_agent_tool_is_not_executed() {
    assert_eq!(
        sub_agent_call(br#"{"brief":"Read lib.rs","tools":["inspect"]}"#, false),
        llm::Decoded::Invalid { problem: llm::Problem::UnknownTool }
    );
}

#[test]
fn arrays_keep_both_boundary_tokens() {
    // Reader records count both opening and closing container brackets.
    for (bytes, count) in [
        (b"[]".as_slice(), 2),
        (b"[[]]".as_slice(), 4),
        (b"[[[]]]".as_slice(), 6),
        (b"[0]".as_slice(), 3),
        (b"[0,0]".as_slice(), 4),
    ] {
        let length = u32::try_from(bytes.len()).unwrap();
        let document = stream::collect(bytes, length, Settings::stream_tokens(length)).expect("bounded JSON");
        assert_eq!(document.len(), count);
    }
}

// The review's observed service ceiling: six escapes per 65,536 answer bytes,
// plus the existing 65,536-byte line framing allowance. This is never enlarged
// to accommodate a token-derived fixture.
const REVIEW_LINE_BYTES: u32 = 458_752;

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary JSON fixture builder, programming-model.md, section 10.2"
)]
fn dense_result(element: &[u8], length: u32) -> alloc::vec::Vec<u8> {
    const PREFIX: &[u8] = br#"{"type":"result","padding":["#;
    const SUFFIX: &[u8] = b"]}";
    let length = usize::try_from(length).unwrap();
    let room = length.checked_sub(PREFIX.len()).unwrap().checked_sub(SUFFIX.len()).unwrap();
    let width = element.len().checked_add(1).unwrap();
    let count = room.checked_add(1).unwrap().checked_div(width).unwrap();
    let mut bytes = PREFIX.to_vec();
    for index in 0..count {
        if index != 0 {
            bytes.push(b',');
        }
        bytes.extend_from_slice(element);
    }
    bytes.extend_from_slice(SUFFIX);
    bytes.resize(length, b' ');
    bytes
}

fn result_in_stdout(element: &[u8], length: u32) -> Queue<ToDomain> {
    // Backend::new owns one 4096-byte deferred stdout intake.
    const READ_BYTES: usize = 4096;
    let mut backend = running(fixtures::backend_with_line_bytes(REVIEW_LINE_BYTES));
    let mut bytes = dense_result(element, length);
    bytes.push(b'\n');
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    // The real stdout entrance retains at most one 4096-byte read at a time.
    for chunk in bytes.chunks(READ_BYTES) {
        for byte in chunk {
            backend.intake.push(*byte).unwrap();
        }
        backend.consume_stdout(&env(0), &mut up, &mut below);
    }
    up
}

#[test]
fn a_dense_result_at_the_line_bound_decodes() {
    // Empty array elements retain more tokens per byte than one-digit numbers.
    for element in [b"0".as_slice(), b"[]".as_slice()] {
        let mut up = result_in_stdout(element, REVIEW_LINE_BYTES);
        let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("result at line bound") };
        assert_eq!(completion.stop, llm::Stop::EndTurn);
    }
}

#[test]
fn a_result_past_the_line_bound_is_refused() {
    let mut up = result_in_stdout(b"0", REVIEW_LINE_BYTES.checked_add(1).unwrap());
    let Some(ToDomain::Failed { failure, detail, .. }) = up.pop() else { panic!("line overflow") };
    assert_eq!(failure, llm::Failure::Program { exit: None });
    assert_eq!(detail.as_ref(), b"smith: the program stream exceeds its line bound");
}

#[test]
fn a_result_with_the_reviews_ignored_array_decodes() {
    // The review's 131,610 zeros plus seven container/key/value records:
    // 131,617 tokens, encoded in 263,249 bytes, below REVIEW_LINE_BYTES.
    const REVIEW_RESULT_BYTES: u32 = 263_249;
    let mut up = result_in_stdout(b"0", REVIEW_RESULT_BYTES);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else { panic!("review result") };
    assert_eq!(completion.stop, llm::Stop::EndTurn);
}

// Queue capacities used by the existing conversation step harness.
const UP_EVENTS: u32 = 32;
const BELOW_EFFECTS: u32 = 64;

const STATUS: &[u8] = br#"{"type":"system","subtype":"status","status":null}"#;
const REPLY: &[u8] = br#"{"type":"assistant","message":{"id":"reply","content":[{"type":"text","text":"Hello."}]}}"#;
const RESULT: &[u8] = br#"{"type":"result","is_error":false,"terminal_reason":"completed","stop_reason":"end_turn"}"#;

fn active_with_queues() -> (Backend, Queue<ToDomain>, Queue<Below>) {
    (active(), Queue::with_capacity(UP_EVENTS), Queue::with_capacity(BELOW_EFFECTS))
}

fn expected_reply() -> Box<[llm::Said]> {
    Box::new([llm::Said::Text { text: b"Hello.".as_slice().into(), replay: None }])
}

fn finished_reply(up: &Queue<ToDomain>) -> &llm::Completion {
    for event in up {
        match event {
            ToDomain::Completed { completion, .. } => return completion,
            ToDomain::BuiltinResult { .. }
            | ToDomain::ReadVersion { .. }
            | ToDomain::Failed { .. }
            | ToDomain::Cancelled { .. }
            | ToDomain::Text { .. } => {}
        }
    }
    panic!("completed reply")
}

fn reply_after_notice(notice: &[u8]) -> (Backend, Queue<ToDomain>) {
    let (mut backend, mut up, mut below) = active_with_queues();
    backend.line(&env(0), notice, &mut up, &mut below);
    backend.line(&env(0), REPLY, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    (backend, up)
}

#[test]
fn status_before_a_completion_preserves_the_reply() {
    let (backend, up) = reply_after_notice(STATUS);
    assert_eq!(finished_reply(&up).content, expected_reply());
    assert_eq!(backend.startup.phase(), State::Running);
}

#[test]
fn status_after_a_tool_result_preserves_the_next_completion() {
    let (mut backend, mut up, mut below) = active_with_queues();
    backend.line(&env(0), br#"{"type":"assistant","message":{"id":"call","content":[{"type":"tool_use","id":"read","name":"Read","input":{}}]}}"#, &mut up, &mut below);
    backend.line(
        &env(0),
        br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"read","content":"file text"}]}}"#,
        &mut up,
        &mut below,
    );
    while backend.pop_measure().is_some() {}
    backend.resume(&env(0), Complete::from_request(request(&backend)), &mut up, &mut below);
    up = Queue::with_capacity(UP_EVENTS);
    backend.line(&env(0), STATUS, &mut up, &mut below);
    backend.line(&env(0), REPLY, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    assert_eq!(finished_reply(&up).content, expected_reply());
    assert_eq!(backend.startup.phase(), State::Running);
}

fn second_turn() -> (Backend, Queue<ToDomain>, Queue<Below>) {
    let (mut backend, mut up, mut below) = active_with_queues();
    let Some(Complete { prompt, .. }) = backend.current.request_mut() else { panic!("complete") };
    prompt.served = Box::new([llm::Served::Finish]);
    backend.line(&env(0), REPLY, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    while backend.pop_measure().is_some() {}
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, .. } = &mut next else { panic!("complete") };
    prompt.served = Box::new([llm::Served::Finish]);
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::Text { text: b"second message".as_slice().into(), replay: None }]),
    }]);
    backend.resume(&env(0), Complete::from_request(next), &mut up, &mut below);
    up = Queue::with_capacity(UP_EVENTS);
    (backend, up, below)
}

#[test]
fn a_valid_later_init_allows_the_second_turn() {
    let (mut backend, mut up, mut below) = second_turn();
    let line = stream::tests::init_line(&["mcp__smith__finish"]);
    backend.line(&env(0), &line, &mut up, &mut below);
    backend.line(&env(0), STATUS, &mut up, &mut below);
    backend.line(&env(0), REPLY, &mut up, &mut below);
    backend.line(&env(0), RESULT, &mut up, &mut below);
    assert_eq!(finished_reply(&up).stop, llm::Stop::EndTurn);
    assert_eq!(backend.startup.phase(), State::Running);
}

fn init_failure(mut up: Queue<ToDomain>) -> (llm::Failure, Box<[u8]>) {
    let Some(ToDomain::Failed { failure, detail, .. }) = up.pop() else { panic!("init failure") };
    (failure, detail)
}

#[test]
fn a_later_init_fails_like_startup_when_tools_are_missing() {
    let line = stream::tests::init_line(&[]);
    let (mut startup, mut startup_up, mut startup_below) = second_turn();
    startup.admit_init(&env(0), &line, &mut startup_up, &mut startup_below);
    let (mut later, mut later_up, mut later_below) = second_turn();
    later.line(&env(0), &line, &mut later_up, &mut later_below);
    let expected =
        (llm::Failure::Program { exit: None }, b"smith: the program started without smith's tools".as_slice().into());
    assert_eq!(init_failure(startup_up), expected);
    assert_eq!(init_failure(later_up), expected);
}

#[test]
fn a_later_init_rechecks_the_runs_builtin_grants() {
    let (mut backend, mut up, mut below) = second_turn();
    let line = stream::tests::init_line(&["mcp__smith__finish", "Read"]);
    backend.line(&env(0), &line, &mut up, &mut below);
    assert_eq!(
        init_failure(up),
        (
            llm::Failure::Program { exit: None },
            b"smith: the program offers a built-in tool the run did not grant: Read".as_slice().into()
        )
    );
}

#[test]
fn an_unknown_system_subtype_preserves_the_reply() {
    let (backend, up) = reply_after_notice(br#"{"type":"system","subtype":"something_new"}"#);
    assert_eq!(finished_reply(&up).content, expected_reply());
    assert_eq!(backend.startup.phase(), State::Running);
}

#[test]
fn thinking_tokens_is_a_notice_without_state_or_measure() {
    let (mut backend, mut up, mut below) = active_with_queues();
    backend.line(
        &env(0),
        br#"{"type":"system","subtype":"thinking_tokens","estimated_tokens":50,"estimated_tokens_delta":50}"#,
        &mut up,
        &mut below,
    );
    assert_eq!(backend.startup.phase(), State::Running);
    assert!(backend.turns.message.is_none());
    assert_eq!(backend.turns.closed, 0);
    assert!(backend.pop_measure().is_none());
    assert!(up.is_empty());
    assert!(below.is_empty());
}

fn advisor_request(backend: &Backend, owner: u64) -> Complete {
    let mut complete = Complete::from_request(request(backend));
    complete.owner = Token::new(owner);
    complete.prompt.served = Box::new([llm::Served::Finish]);
    complete.prompt.tools.inspect = true;
    complete.outcome.report = Some(run::outcome::TextSpec { max: 128, fields: Box::default() });
    complete
}

fn advisor_tools(harness: &mut fixtures::Harness, assistant: &[u8], results: &[u8], owner: u64, calls: usize) {
    harness.backend.line(&env(0), assistant, &mut harness.up, &mut harness.below);
    let Some(ToDomain::Completed { owner: got, completion }) = harness.up.pop() else { panic!("tool completion") };
    assert_eq!(got, Token::new(owner));
    assert_eq!(completion.content.len(), calls);
    harness.backend.line(&env(0), results, &mut harness.up, &mut harness.below);
    for _ in 0..calls {
        let Some(ToDomain::BuiltinResult { owner: got, .. }) = harness.up.pop() else { panic!("built-in result") };
        assert_eq!(got, Token::new(owner));
    }
    assert!(harness.up.is_empty());
    while harness.backend.pop_measure().is_some() {}
    let next = advisor_request(&harness.backend, owner.checked_add(1).unwrap());
    harness.backend.resume(&env(0), next, &mut harness.up, &mut harness.below);
}

#[test]
fn advisor_two_reads_reaches_next_completion() {
    for framed in [false, true] {
        advisor_two_reads(framed);
    }
}

fn advisor_two_reads(framed: bool) {
    // The tool order is retained evidence. The post-tool StructuredOutput boundary
    // is a reachable design-admitted transition, not walk-79's proven rejected record.
    let mut harness = fixtures::Harness::new(active());
    let first = advisor_request(&harness.backend, 80);
    harness.backend.current.retain(first, &mut harness.up);
    advisor_tools(&mut harness,
        br#"{"type":"assistant","message":{"id":"one","content":[{"type":"tool_use","id":"a","name":"Read","input":{"file_path":"src/main.rs"}},{"type":"tool_use","id":"b","name":"Glob","input":{"pattern":"*"}},{"type":"tool_use","id":"c","name":"Read","input":{"file_path":".forgejo/workflows/ci.yml"}}]}}"#,
        br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"a","content":"source"},{"type":"tool_result","tool_use_id":"b","content":"paths"},{"type":"tool_result","tool_use_id":"c","content":"workflow"}]}}"#, 80, 3);
    advisor_tools(&mut harness,
        br#"{"type":"assistant","message":{"id":"two","content":[{"type":"tool_use","id":"d","name":"Read","input":{"file_path":"CONTRIBUTING.md"}},{"type":"tool_use","id":"e","name":"Read","input":{"file_path":"README.md"}}]}}"#,
        br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"d","content":"contributing"},{"type":"tool_result","tool_use_id":"e","content":"readme"}]}}"#, 81, 2);
    for record in [
        br#"{"type":"assistant","message":{"id":"outcome","content":[{"type":"text","text":"before"},{"type":"tool_use","id":"s","name":"StructuredOutput","input":{}},{"type":"text","text":"after"}],"usage":{"input_tokens":17,"output_tokens":3}}}"#.as_slice(),
        br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"s","content":"accepted"}]}}"#,
    ] {
        harness.backend.line(&env(0), record, &mut harness.up, &mut harness.below);
        assert!(harness.up.is_empty(), "held outcome, got {:?}", harness.up.pop());
    }
    advisor_tail(&mut harness, framed);
    harness.backend.line(
        &env(0),
        br#"{"type":"result","structured_output":{"outcome":{"form":"report","text":"done","fields":{}}}}"#,
        &mut harness.up,
        &mut harness.below,
    );
    advisor_finish(&harness.up);
    assert_eq!(harness.backend.turns.closed, 4);
    assert_eq!(harness.backend.turns.ready.len(), 1);
    assert_eq!(harness.backend.current.owner(), None);
    while harness.backend.pop_measure().is_some() {}
    harness.up = Queue::with_capacity(UP_EVENTS);
    let next = advisor_request(&harness.backend, 83);
    harness.backend.resume(&env(0), next, &mut harness.up, &mut harness.below);
    let tail = finished_reply(&harness.up);
    assert_eq!(tail.usage.input_tokens, Some(19));
    assert_eq!(tail.usage.output_tokens, Some(2));
    assert!(harness.backend.turns.ready.is_empty());
}

fn advisor_finish(up: &Queue<ToDomain>) {
    let completion = finished_reply(up);
    let [
        llm::Said::Text { text: before, .. },
        llm::Said::ToolCall { name, call, .. },
        llm::Said::Text { text: after, .. },
    ] = completion.content.as_ref()
    else {
        panic!("finish in held position")
    };
    assert_eq!(before.as_ref(), b"before");
    assert_eq!(after.as_ref(), b"after");
    assert_eq!(name.as_ref(), b"finish");
    let llm::Decoded::Served { ask: run::Ask::Finish { .. } } = call else { panic!("decoded finish") };
    assert_eq!(completion.usage.input_tokens, Some(17));
    assert_eq!(completion.usage.output_tokens, Some(3));
    let mut completions = 0_u32;
    for event in up {
        match event {
            ToDomain::Completed { owner, .. } => {
                assert_eq!(*owner, Token::new(82));
                completions = completions.checked_add(1).unwrap();
            }
            ToDomain::Text { .. } => {}
            ToDomain::BuiltinResult { .. }
            | ToDomain::ReadVersion { .. }
            | ToDomain::Failed { .. }
            | ToDomain::Cancelled { .. } => panic!("only the held completion"),
        }
    }
    assert_eq!(completions, 1);
}

#[test]
fn rejected_records_name_the_kind_and_condition_without_payloads() {
    let cases: &[(&[u8], &[u8])] = &[
        (br#"{"private":"never repeat this"}"#, b"record: top-level record has no string type"),
        (br#"{"type":"never repeat this"}"#, b"record: unknown top-level record type"),
        (br#"{"type":"system","private":"never repeat this"}"#, b"system: system record has no string subtype"),
        (
            br#"{"type":"stream_event","event":{"type":"never repeat this"}}"#,
            b"stream_event: unknown stream event type",
        ),
        (br#"{"type":"stream_event"}"#, b"stream_event: stream_event has no event"),
        (
            br#"{"type":"stream_event","event":{"type":"message_stop"}}"#,
            b"stream_event: message_stop has no open message",
        ),
        (
            br#"{"type":"assistant","message":{"id":"m","content":[{"type":"never repeat this"}]}}"#,
            b"assistant: unknown assistant block type",
        ),
        (
            br#"{"type":"assistant","message":{"id":"m","content":[{"type":"tool_use","name":"Read","input":{}}]}}"#,
            b"assistant: tool_use has no string id",
        ),
        (
            br#"{"type":"user","message":{"content":[{"type":"tool_result"}]}}"#,
            b"user: tool_result has no string tool_use_id",
        ),
    ];
    for (record, reason) in cases {
        let (mut backend, mut up, mut below) = active_with_queues();
        backend.line(&env(0), record, &mut up, &mut below);
        let Some(ToDomain::Failed { failure, detail, .. }) = up.pop() else { panic!("record rejected") };
        assert_eq!(failure, llm::Failure::Program { exit: None });
        assert_eq!(detail.as_ref(), skein_lib::bytes::concat(&[b"smith: ", reason]).as_ref());
        assert!(detail.len() <= 256);
    }
}

fn advisor_tail(harness: &mut fixtures::Harness, framed: bool) {
    if framed {
        harness.backend.line(
            &env(0),
            br#"{"type":"stream_event","event":{"type":"message_start","message":{"id":"tail"}}}"#,
            &mut harness.up,
            &mut harness.below,
        );
        harness.backend.line(
            &env(0),
            br#"{"type":"stream_event","event":{"type":"content_block_start","index":0}}"#,
            &mut harness.up,
            &mut harness.below,
        );
    }
    harness.backend.line(&env(0), br#"{"type":"assistant","message":{"id":"tail","content":[{"type":"text","text":"Done."}],"usage":{"input_tokens":19,"output_tokens":2}}}"#, &mut harness.up, &mut harness.below);
    if framed {
        harness.backend.line(
            &env(0),
            br#"{"type":"stream_event","event":{"type":"message_stop"}}"#,
            &mut harness.up,
            &mut harness.below,
        );
    }
    assert!(harness.up.is_empty(), "outcome and tail wait for the result");
}

#[test]
fn unresolved_outcomes_release_on_retry_and_interrupt() {
    for interrupt in [false, true] {
        let mut harness = fixtures::Harness::new(active());
        let first = advisor_request(&harness.backend, 80);
        harness.backend.current.retain(first, &mut harness.up);
        harness.backend.line(&env(0), br#"{"type":"assistant","message":{"id":"outcome","content":[{"type":"tool_use","id":"s","name":"StructuredOutput","input":{}}]}}"#, &mut harness.up, &mut harness.below);
        advisor_tail(&mut harness, false);
        assert_eq!(harness.backend.turns.ready.len(), 1);
        if interrupt {
            harness.backend.turns.interrupted();
            assert_eq!(harness.backend.turns.closed, 1);
            let completion = harness.backend.turns.ready.pop().expect("interrupt terminal");
            assert!(completion.content.is_empty());
            assert_eq!(completion.usage, llm::Usage::ZERO);
        } else {
            harness.backend.turns.retry();
            assert_eq!(harness.backend.turns.closed, 0);
        }
        assert!(harness.backend.turns.ready.is_empty());
        assert!(harness.backend.turns.message.is_none());
    }
}

#[test]
fn an_unresolved_outcome_waits_for_stdout_without_polling() {
    let mut harness = fixtures::Harness::new(fixtures::running(fixtures::backend()));
    let first = advisor_request(&harness.backend, 80);
    harness.backend.current.retain(first, &mut harness.up);
    harness.backend.line(&env(0), br#"{"type":"assistant","message":{"id":"outcome","content":[{"type":"tool_use","id":"s","name":"StructuredOutput","input":{}}]}}"#, &mut harness.up, &mut harness.below);
    advisor_tail(&mut harness, false);
    while harness.backend.pop_measure().is_some() {}
    assert_eq!(harness.backend.turns.ready.len(), 1);
    for _ in 0_u32..2 {
        harness.backend.consume_stdout(&env(0), &mut harness.up, &mut harness.below);
        assert_eq!(harness.backend.intake_due, None, "only more stdout can resolve the held outcome");
        assert!(harness.backend.stdout_demanding);
        assert!(harness.up.is_empty());
    }
    for byte in br#"{"type":"result","structured_output":{"outcome":{"form":"report","text":"done","fields":{}}}}
"# {
        harness.backend.intake.push(*byte).unwrap();
    }
    harness.backend.consume_stdout(&env(0), &mut harness.up, &mut harness.below);
    assert_eq!(finished_reply(&harness.up).stop, llm::Stop::ToolUse);
}
