#![expect(
    clippy::disallowed_methods,
    reason = "programming-model.md section10.2: deterministic input-join fixtures use ordinary Rust"
)]

use super::*;

fn limits() -> session::Limits {
    smith_agent_world::LIMITS.session
}

fn completion(input: &[u8]) -> llm::Completion {
    llm::Completion {
        reasoning_dropped: Box::new([]),
        content: Box::new([llm::Said::ToolCall {
            id: b"same-provider-id".to_vec().into(),
            name: b"wait".to_vec().into(),
            input: input.into(),
            call: llm::Decoded::Served { ask: smith_domain::run::Ask::Wait },
            replay: None,
        }]),
        stop: llm::Stop::ToolUse,
        usage: llm::Usage::ZERO,
    }
}

fn metadata(input: &[u8]) -> session::ToolCall {
    session::ToolCall {
        owner: Token::new(17),
        id: b"same-provider-id".to_vec().into(),
        name: b"wait".to_vec().into(),
        source: session::ToolSource::Run,
        effect: smith_domain::tools::Effect::Read,
        deadline: skein_lib::Time::ZERO,
        input_bytes: u64::try_from(input.len()).unwrap(),
    }
}

#[test]
fn only_selected_capture_moves_exact_arguments_into_the_matching_conversation_start() {
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        let bounds = limits();
        let mut book = Book::new(&bounds, capture);
        let owner = Token::new(11);
        let input = b" { } \n";
        book.received(owner, &completion(input));
        assert!(book.take(Token::new(12), &metadata(input)).is_none());
        let mut wrong = metadata(input);
        wrong.name = b"different-tool".to_vec().into();
        assert!(book.take(owner, &wrong).is_none());
        let captured = book.take(owner, &metadata(input));
        match capture {
            Capture::None => {
                assert!(captured.is_none());
                assert_eq!(book.conversations.capacity(), 0);
            }
            Capture::Calls | Capture::Everything => assert_eq!(captured.as_deref(), Some(input.as_slice())),
        }
    }
}

#[test]
fn a_later_completion_reusing_an_id_and_an_unstarted_session_end_release_the_prior_batch() {
    let mut book = Book::new(&limits(), Capture::Calls);
    let owner = Token::new(11);
    let first = b"{}";
    book.received(owner, &completion(first));
    assert_eq!(book.take(owner, &metadata(first)).as_deref(), Some(first.as_slice()));
    let later = b" { } ";
    book.received(owner, &completion(later));
    assert_eq!(book.take(owner, &metadata(later)).as_deref(), Some(later.as_slice()));
    book.received(owner, &completion(first));
    book.ended(owner);
    assert_eq!(book.conversations.len(), 0);
    assert!(book.take(owner, &metadata(first)).is_none());
}
