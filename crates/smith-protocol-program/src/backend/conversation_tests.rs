use super::lifecycle_tests::{backend, env, request};
use super::*;

fn active() -> Backend {
    running(backend())
}

fn running(mut backend: Backend) -> Backend {
    backend.phase = Phase::Running;
    backend.child = Some(Token::new(30));
    backend.pipes[0] = Some(Token::new(31));
    backend.request = Some(request(&backend));
    backend.owner = Some(Token::new(80));
    backend
}

#[test]
fn an_unframed_tool_message_completes_without_any_partial() {
    let mut backend = active();
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0),br#"{"type":"assistant","message":{"id":"unframed","content":[{"type":"tool_use","id":"t","name":"mcp__smith__finish","input":{}}],"usage":{"input_tokens":17,"output_tokens":3}}}"#,&mut up,&mut below);
    let Some(ToDomain::Completed { owner, completion }) = up.pop() else {
        panic!("unframed completion");
    };
    assert_eq!(owner, Token::new(80));
    assert_eq!(completion.usage.input_tokens, Some(17));
    assert!(backend.turns.message.is_none());
}

#[test]
fn an_open_next_message_does_not_receive_a_duplicate_nudge() {
    let mut backend = active();
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
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
    assert!(backend.owner.is_none(), "the first message was delivered");
    let mut next = request(&backend);
    let FromDomain::Complete { prompt, owner, .. } = &mut next else {
        panic!("next completion");
    };
    *owner = Token::new(81);
    prompt.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::Text { text: b"finish now".as_slice().into(), replay: None }]),
    }]);
    backend.resume(&env(0), next, &mut up, &mut below);
    assert!(backend.outgoing.is_none());
    assert_eq!(backend.owner, Some(Token::new(81)));
    assert!(below.is_empty());
}

#[test]
fn a_line_outside_the_stream_grammar_is_a_program_failure() {
    let mut backend = active();
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), br#"{"other":true}"#, &mut up, &mut below);
    let Some(ToDomain::Failed { failure, .. }) = up.pop() else {
        panic!("program failure");
    };
    assert_eq!(failure, llm::Failure::Program { exit: None });
}
#[test]
fn a_result_without_an_assistant_hands_the_nudge_to_the_domain() {
    let mut backend = active();
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0), br#"{"type":"result","subtype":"success","is_error":false}"#, &mut up, &mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else {
        panic!("no-tool completion");
    };
    assert!(completion.content.is_empty());
    assert_eq!(completion.stop, llm::Stop::EndTurn);
    assert_eq!(completion.usage, llm::Usage::ZERO);
}
#[test]
fn the_unoffered_sub_agent_tool_is_not_executed() {
    let mut backend = active();
    let Some(FromDomain::Complete { prompt, .. }) = &mut backend.request else {
        panic!("completion");
    };
    prompt.served = Box::new([llm::Served::SubAgent]);
    let mut up = Queue::with_capacity(32);
    let mut below = Queue::with_capacity(64);
    backend.line(&env(0),br#"{"type":"assistant","message":{"id":"unframed","content":[{"type":"tool_use","id":"t","name":"mcp__smith__sub_agent","input":{}}]}}"#,&mut up,&mut below);
    let Some(ToDomain::Completed { completion, .. }) = up.pop() else {
        panic!("completion");
    };
    let llm::Said::ToolCall { call, .. } = &completion.content[0] else {
        panic!("tool call");
    };
    assert_eq!(*call, llm::Decoded::Invalid { problem: llm::Problem::UnknownTool });
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

// The review's actual service ceiling: six escapes per 65,536 answer bytes,
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
    let mut backend = running(lifecycle_tests::backend_with_line_bytes(REVIEW_LINE_BYTES));
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
