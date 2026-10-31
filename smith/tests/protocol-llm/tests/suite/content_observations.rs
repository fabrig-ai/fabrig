//! Observed content observations through Smith's component and the native HTTP/SSE client.
//! Contracts: protocol/events.md, sections 3.5 and 3.11.

use skein_lib::Token;
use smith_events::Block;
use smith_protocol_llm::Measure;
use smith_protocol_llm_world::pool_peer::Peer;

#[test]
fn observed_codex_text_deltas_join_to_the_observed_completion() {
    let body = [
        r#"{"type":"response.output_item.added","output_index":0,"item":{"type":"message","id":"item","role":"assistant","content":[]}}"#,
        r#"{"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"one "}"#,
        r#"{"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"two"}"#,
        r#"{"type":"response.output_item.done","output_index":0,"item":{"type":"message","id":"item","role":"assistant","content":[{"type":"output_text","text":"one two"}]}}"#,
        r#"{"type":"response.completed","response":{"status":"completed","output":[],"usage":{"input_tokens":1,"output_tokens":2}}}"#,
    ].iter().flat_map(|event| format!("data: {event}\n\n").into_bytes()).collect::<Vec<_>>();
    let head = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n", body.len());
    let mut peer = Peer::configured(1, 1);
    peer.start_observed(7, 60, 60, 300);
    peer.bytes(100, head.as_bytes());
    for bytes in body.chunks(17) {
        peer.bytes(100, bytes);
    }
    assert_eq!(peer.terminals.len(), 1);
    let mut text = Vec::new();
    let mut prompt_count = 0;
    let mut completed = 0;
    let mut terminals = 0;
    for measure in peer.measures {
        match measure {
            Measure::Prompt { at: _, owner, prompt } => {
                assert_eq!(owner, Token::new(7));
                assert_eq!(prompt.system.as_deref(), Some(b"timed reasoning".as_slice()));
                assert_eq!(prompt.messages.len(), 1);
                prompt_count += 1;
            }
            Measure::Delta { at: _, owner, block, text: delta } => {
                assert_eq!(owner, Token::new(7));
                assert_eq!(block, 0);
                text.extend_from_slice(&delta);
            }
            Measure::Completion { at: _, owner, completion } => {
                assert_eq!(owner, Token::new(7));
                assert_eq!(text, b"one two");
                assert_eq!(completion.blocks.as_slice(), &[Block::Text { text: b"one two".to_vec().into() }]);
                completed += 1;
            }
            Measure::Call { .. } | Measure::Arguments { .. } => unreachable!("input not selected"),
            Measure::Retried { .. } => unreachable!("direct components do not retry internally"),
            Measure::Terminal { owner, measures } => {
                assert_eq!(owner, Token::new(7));
                assert_eq!(measures.request_bytes, Some(peer.request_bytes));
                assert!(peer.request_bytes > 0);
                assert_eq!(measures.first_byte, Some(skein_lib::Duration::ZERO));
                assert_eq!(measures.largest_gap, Some(skein_lib::Duration::ZERO));
                terminals += 1;
            }
        }
    }
    assert_eq!((prompt_count, completed), (1, 1));
    assert_eq!(terminals, 1);
}

#[test]
fn observed_transport_latency_and_event_gap_use_the_peer_clock_without_content_capture() {
    // An opened message item is only progress; its first delta is the first output event.
    let first = concat!(
        "data: {\"type\":\"response.output_item.added\",\"output_index\":0,\"item\":{\"type\":\"message\",\"id\":\"item\",\"role\":\"assistant\",\"content\":[]}}\n\n",
        "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"done\"}\n\n"
    );
    let last = concat!(
        "data: {\"type\":\"response.output_item.done\",\"output_index\":0,\"item\":{\"type\":\"message\",\"id\":\"item\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"done\"}]}}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n"
    );
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n",
        first.len() + last.len()
    );
    let mut peer = Peer::configured(1, 1);
    peer.start(7, 60, 60, 300);
    peer.advance(5);
    peer.bytes(100, &[head.as_bytes(), first.as_bytes()].concat());
    peer.advance(8);
    peer.bytes(100, last.as_bytes());
    assert_eq!(peer.terminals.len(), 1);
    assert_eq!(peer.measures.len(), 1, "disabled capture retains only the observed terminal measures");
    let Measure::Terminal { owner, measures } = peer.measures[0] else { panic!("observed terminal measures") };
    assert_eq!(owner, Token::new(7));
    assert_eq!(measures.request_bytes, Some(peer.request_bytes));
    assert_eq!(measures.first_byte, Some(skein_lib::Duration::from_secs(5)));
    assert_eq!(measures.largest_gap, Some(skein_lib::Duration::from_secs(3)));
}

#[test]
fn deltas_are_handed_over_only_when_asked() {
    const OWNER: u64 = 7;
    const SOCKET: u64 = 100;
    const DEADLINE: u64 = 60;
    let body = concat!(
        "data: {\"type\":\"response.output_item.added\",\"output_index\":0,\"item\":{\"type\":\"message\",\"id\":\"item\",\"role\":\"assistant\",\"content\":[]}}\n\n",
        "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"reply\"}\n\n",
        "data: {\"type\":\"response.output_item.done\",\"output_index\":0,\"item\":{\"type\":\"message\",\"id\":\"item\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"reply\"}]}}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n"
    );
    let head = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n", body.len());
    for (observed, deltas, expected) in
        [(false, false, b"".as_slice()), (false, true, b"reply"), (true, false, b""), (true, true, b"reply")]
    {
        let mut peer = Peer::configured(1, 1);
        peer.deltas(deltas);
        if observed {
            peer.start_observed(OWNER, DEADLINE, DEADLINE, DEADLINE);
        } else {
            peer.start(OWNER, DEADLINE, DEADLINE, DEADLINE);
        }
        peer.bytes(SOCKET, head.as_bytes());
        peer.bytes(SOCKET, body.as_bytes());
        let mut text = Vec::new();
        for measure in &peer.measures {
            if let Measure::Delta { text: delta, .. } = measure {
                text.extend_from_slice(delta);
            }
        }
        assert_eq!(text, expected, "observed={observed}, deltas={deltas}");
    }
    for provider in [skein_llm::Provider::OpenAiCodex, skein_llm::Provider::Anthropic] {
        let events: &[&str] = match provider {
            skein_llm::Provider::OpenAiCodex => &[
                r#"{"type":"response.output_item.added","output_index":0,"item":{"type":"function_call","id":"item","call_id":"call","name":"read","arguments":""}}"#,
                r#"{"type":"response.function_call_arguments.delta","output_index":0,"delta":"{}"}"#,
                r#"{"type":"response.output_item.done","output_index":0,"item":{"type":"function_call","id":"item","call_id":"call","name":"read","arguments":"{}"}}"#,
                r#"{"type":"response.completed","response":{"status":"completed","output":[],"usage":{"input_tokens":1,"output_tokens":1}}}"#,
            ],
            skein_llm::Provider::Anthropic => &[
                r#"{"type":"message_start","message":{"id":"message","type":"message","role":"assistant","content":[],"usage":{"input_tokens":1,"output_tokens":0}}}"#,
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"call","name":"read","input":{}}}"#,
                r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{}"}}"#,
                r#"{"type":"content_block_stop","index":0}"#,
                r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":1}}"#,
                r#"{"type":"message_stop"}"#,
            ],
            skein_llm::Provider::OpenAi => unreachable!("current dialects"),
        };
        let mut body = String::new();
        for event in events {
            body.push_str("data: ");
            body.push_str(event);
            body.push_str("\n\n");
        }
        let head =
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n", body.len());
        for observed in [false, true] {
            for text in [false, true] {
                for input in [false, true] {
                    let mut peer = Peer::for_provider(1, 1, provider);
                    peer.deltas(text);
                    peer.input(input);
                    if observed {
                        peer.start_observed(OWNER, DEADLINE, DEADLINE, DEADLINE);
                    } else {
                        peer.start(OWNER, DEADLINE, DEADLINE, DEADLINE);
                    }
                    peer.bytes(SOCKET, head.as_bytes());
                    peer.bytes(SOCKET, body.as_bytes());
                    let mut calls = 0;
                    let mut arguments = Vec::new();
                    for measure in &peer.measures {
                        match measure {
                            Measure::Call { owner, index, id, name, .. } => {
                                assert_eq!((*owner, *index), (Token::new(OWNER), 0));
                                assert_eq!((id.as_ref(), name.as_ref()), (b"call".as_slice(), b"read".as_slice()));
                                calls += 1;
                            }
                            Measure::Arguments { owner, index, bytes, .. } => {
                                assert_eq!((*owner, *index), (Token::new(OWNER), 0));
                                arguments.extend_from_slice(bytes);
                            }
                            Measure::Prompt { .. }
                            | Measure::Completion { .. }
                            | Measure::Delta { .. }
                            | Measure::Retried { .. }
                            | Measure::Terminal { .. } => {}
                        }
                    }
                    assert_eq!(
                        calls,
                        usize::from(input),
                        "{provider:?}, input={input}, text={text}, observed={observed}"
                    );
                    assert_eq!(arguments, if input { b"{}".as_slice() } else { b"" }, "{provider:?}");
                }
            }
        }
    }
}
