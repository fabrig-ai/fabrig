//! Positive-first adapter boundary controls over observed shared Client terminals.
//! Caller schemas, restored results and a literal Wait attestation are fixture
//! data; no application decoder or Finish policy lives in this test.
//! Contract: testing-strategy.md, sections 2.5 and 6.

use skein_fake_llm_domain::api;
use skein_lib::Token;
use skein_llm::{self as shared, client};
use skein_llm_world::fake::Exchange;
use smith_domain::{Event, llm, run, tools};
use smith_protocol_llm_world::client_peer::{self, Configuration};
use smith_protocol_llm_world::codec::{self as codec, Context, Input, Limits, ToolKind, ToolSchema};

const OWNER: Token = Token::new(71);
const WAIT_SCHEMA: &[u8] = br#"{"type":"object","properties":{},"additionalProperties":false}"#;

fn limits() -> Limits {
    Limits {
        rendered_result: 2048,
        ..smith_protocol_llm::fixture::small(skein_llm_world::limits(), smith_protocol_llm_world::codec::stated())
    }
}

fn prompt() -> llm::Prompt {
    llm::Prompt {
        effort: None,
        purpose: llm::Purpose::Ordinary { guard: None },
        choice: smith_domain::llm::ToolChoice::Auto,
        namespace: smith_domain::tools::Namespace::One,
        affinity: smith_domain::llm::Affinity { key: [0; 16], thread: 0 },
        asker: llm::Asker::Main,
        endpoint: llm::Endpoint(1),
        model: b"fixture-model".as_slice().into(),
        system: b"@adapter-boundary".as_slice().into(),
        tools: tools::Grants { inspect: false, modify: false, shell: false },
        served: Box::new([llm::Served::Wait]),
        messages: Box::new([llm::Message {
            role: llm::Role::User,
            content: Box::new([llm::Block::Text { text: b"Start.".as_slice().into(), replay: None }]),
        }]),
        max_tokens: 128,
    }
}

fn schema(kind: ToolKind, name: &[u8]) -> ToolSchema {
    ToolSchema {
        kind,
        name: name.into(),
        description: b"Boundary caller declaration.".as_slice().into(),
        schema: WAIT_SCHEMA.into(),
    }
}

fn input(configuration: &Configuration) -> Input {
    let bounds = limits();
    let decoded_call_bytes = 4096;
    Input {
        owner: OWNER,
        prompt: prompt(),
        endpoint_name: llm::Endpoint(1),
        endpoint: configuration.endpoint.clone(),
        credential: match configuration.credential.scheme {
            skein_llm::Scheme::Bearer => shared::Credential::bearer(configuration.credential.value.clone()),
            skein_llm::Scheme::Key => shared::Credential::key(configuration.credential.value.clone()),
        },
        receiving: smith_protocol_llm::fixture::receiving(&bounds.client, decoded_call_bytes)
            .expect("fixture receiving bound"),
    }
}

fn scripts(calls: bool) -> Box<[api::Script]> {
    let lines: Box<[api::Line]> = if calls {
        Box::new([
            api::Line::Text { text: b"before".as_slice().into() },
            api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            api::Line::Text { text: b"after".as_slice().into() },
        ])
    } else {
        Box::new([api::Line::Text { text: b"admitted exact".as_slice().into() }])
    };
    Box::new([api::Script {
        cue: b"@adapter-boundary".as_slice().into(),
        turns: Box::new([api::Turn {
            lines,
            finish: if calls { api::Finish::ToolCalls } else { api::Finish::Stop },
            tokens: 7,
        }]),
    }])
}

fn adopt(input: Input, calls: bool) -> (Context, Exchange) {
    let endpoint = input.endpoint.clone();
    let credential = match input.credential.scheme {
        shared::Scheme::Bearer => shared::Credential::bearer(input.credential.value.clone()),
        shared::Scheme::Key => shared::Credential::key(input.credential.value.clone()),
    };
    let codec::Prepared { client, context } = codec::prepare(input, &limits()).expect("positive admission");
    let peer = Exchange::prepared(client, endpoint, Some(credential), limits().client, scripts(calls));
    assert!(peer.queries.is_empty(), "preparation starts no wire request");
    assert!(peer.requests.is_empty());
    (context, peer)
}

fn take_terminal(peer: &mut Exchange) -> client::Event {
    let positions: Vec<_> = peer
        .seen
        .iter()
        .enumerate()
        .filter_map(|(index, event)| match event {
            client::Event::Completed { .. } | client::Event::Failed { .. } | client::Event::Cancelled { .. } => {
                Some(index)
            }
            client::Event::Delta { .. }
            | client::Event::Block { .. }
            | client::Event::Reusable
            | client::Event::Close
            | client::Event::Closed => None,
        })
        .collect();
    let [position] = positions.as_slice() else {
        panic!("one observed native terminal: {:?}", peer.seen);
    };
    peer.seen.remove(*position)
}

fn close_won(peer: &mut Exchange) {
    peer.request(client::Request::Close);
    assert_eq!(peer.seen.iter().filter(|event| matches!(event, client::Event::Close)).count(), 1);
    assert_eq!(peer.seen.iter().filter(|event| matches!(event, client::Event::Closed)).count(), 0);
    peer.settle();
    assert_eq!(peer.seen.iter().filter(|event| matches!(event, client::Event::Closed)).count(), 1);
    assert!(peer.seen.iter().all(|event| !matches!(
        event,
        client::Event::Completed { .. } | client::Event::Failed { .. } | client::Event::Cancelled { .. }
    )));
    let events = peer.seen.len();
    peer.settle();
    assert_eq!(peer.seen.len(), events, "repeated observed settlement emits nothing");
    assert_eq!(peer.machine.waiting(), client::Waiting::Nothing);
}

fn result_input(configuration: &Configuration) -> Input {
    let mut input = input(configuration);
    input.prompt.tools.shell = true;
    input.prompt.messages = Box::new([
        llm::Message {
            role: llm::Role::Assistant,
            content: Box::new([llm::Block::ToolCall {
                id: b"prior-shell".as_slice().into(),
                name: b"shell".as_slice().into(),
                input: br#"{"command":"echo prior"}"#.as_slice().into(),
                replay: None,
            }]),
        },
        llm::Message {
            role: llm::Role::User,
            content: Box::new([llm::Block::ToolResult {
                id: b"prior-shell".as_slice().into(),
                result: llm::Returned::Owned {
                    outcome: tools::Outcome::Exited {
                        exit: tools::Exit::Code { code: 0 },
                        head: b"prior".as_slice().into(),
                        tail: Box::new([]),
                        dropped: 0,
                        timeout: tools::TimeoutClamp::Within,
                        output: tools::BytesClamp::Within,
                    },
                },
            }]),
        },
    ]);
    input
}

#[test]
fn owned_results_use_the_production_renderer_before_the_request_goes() {
    for configuration in &client_peer::configurations() {
        let (context, mut peer) = adopt(result_input(configuration), false);
        peer.start();
        peer.run();
        let [query] = peer.queries.as_slice() else { panic!("one query") };
        let [_, user] = query.messages.as_ref() else { panic!("ordered history") };
        let [api::Part::ToolOutput { id, output, is_error }] = user.parts.as_ref() else { panic!("result") };
        assert_eq!(id.as_ref(), b"prior-shell");
        assert!(output.starts_with(b"exit code 0\nprior"));
        assert!(!*is_error);
        let client::Event::Completed { owner: _, completion } = take_terminal(&mut peer) else { panic!("completion") };
        assert!(matches!(
            codec::completion(context, completion),
            Ok(Event::Below(smith_domain::Below::Completed { .. }))
        ));
        close_won(&mut peer);
    }
}

#[test]
fn a_result_past_its_bound_is_cut_with_its_marker_and_the_request_goes() {
    for configuration in &client_peer::configurations() {
        let mut input = result_input(configuration);
        let byte_count = usize::try_from(limits().rendered_result).expect("u32 fits usize") * 2;
        input.prompt.messages[1].content[0] = llm::Block::ToolResult {
            id: b"prior-shell".as_slice().into(),
            result: llm::Returned::Owned {
                outcome: tools::Outcome::Exited {
                    exit: tools::Exit::Code { code: 0 },
                    head: vec![b'x'; byte_count].into(),
                    tail: Box::new([]),
                    dropped: 0,
                    timeout: tools::TimeoutClamp::Within,
                    output: tools::BytesClamp::Within,
                },
            },
        };
        let (context, mut peer) = adopt(input, false);
        peer.start();
        peer.run();
        let [query] = peer.queries.as_slice() else { panic!("one request reached the peer") };
        let [_, user] = query.messages.as_ref() else { panic!("ordered history") };
        let [api::Part::ToolOutput { id, output, is_error }] = user.parts.as_ref() else { panic!("one result") };
        assert_eq!(id.as_ref(), b"prior-shell");
        assert!(!*is_error);
        assert!(output.ends_with(b" bytes omitted]"));
        assert!(output.len() <= usize::try_from(limits().rendered_result).expect("u32 fits usize"));
        let marker = output.windows(2).position(|bytes| bytes == b"\n[").expect("cut marker");
        let omitted = std::str::from_utf8(&output[marker + 2..])
            .expect("ASCII marker")
            .split(' ')
            .next()
            .expect("byte count")
            .parse::<usize>()
            .expect("decimal omitted count");
        assert_eq!(omitted + marker, byte_count + b"exit code 0\n".len());
        let client::Event::Completed { owner: _, completion } = take_terminal(&mut peer) else { panic!("completion") };
        assert!(matches!(
            codec::completion(context, completion),
            Ok(Event::Below(smith_domain::Below::Completed { .. }))
        ));
        close_won(&mut peer);
    }
}

fn child_completed(configuration: &Configuration, arguments: &[u8]) -> (Context, Exchange, Token, shared::Completion) {
    let mut offered = input(configuration);
    offered.prompt.served = Box::new([llm::Served::SubAgent]);
    let schema = codec::schemas(&offered.prompt, &limits().stated)[0].schema.clone();
    let endpoint = offered.endpoint.clone();
    let credential = match offered.credential.scheme {
        shared::Scheme::Bearer => shared::Credential::bearer(offered.credential.value.clone()),
        shared::Scheme::Key => shared::Credential::key(offered.credential.value.clone()),
    };
    let codec::Prepared { client, context } = codec::prepare(offered, &limits()).expect("observed admission");
    let script = Box::new([api::Script {
        cue: b"@adapter-boundary".as_slice().into(),
        turns: Box::new([smith_protocol_llm_world::fixture::calls(
            Box::new([api::Line::Call { name: b"sub_agent".as_slice().into(), arguments: arguments.into() }]),
            7,
        )]),
    }]);
    let mut peer = Exchange::prepared(client, endpoint, Some(credential), limits().client, script);
    peer.start();
    peer.run();
    let [query] = peer.queries.as_slice() else { panic!("one observed provider query") };
    let [tool] = query.tools.as_ref() else { panic!("one offered child tool") };
    assert_eq!(tool.name.as_ref(), b"sub_agent");
    assert_eq!(tool.parameters, schema, "the provider sees the complete production schema");
    for name in [b"max_turns".as_slice(), b"max_seconds"] {
        assert!(skein_lib::bytes::find(&tool.parameters, name).is_some());
    }
    assert!(skein_lib::bytes::find(&tool.parameters, b"maximum").is_none());
    let client::Event::Completed { owner: _, completion } = take_terminal(&mut peer) else {
        panic!("one observed provider completion");
    };
    (context, peer, OWNER, completion)
}

#[test]
fn optional_child_shares_cross_both_provider_schemas_and_observed_completions() {
    use skein_lib::Duration;

    let cases = [
        (br#"{"brief":"inspect","tools":["inspect"]}"#.as_slice(), Some(run::Share::UNASKED), b"".as_slice()),
        (
            br#"{"brief":"inspect","tools":["inspect"],"max_turns":1}"#,
            Some(run::Share { turns: Some(1), time: None }),
            b"",
        ),
        (
            br#"{"brief":"inspect","tools":["inspect"],"max_seconds":2}"#,
            Some(run::Share { turns: None, time: Some(Duration::from_secs(2)) }),
            b"",
        ),
        (
            br#"{"brief":"inspect","tools":["inspect"],"max_turns":4294967295,"max_seconds":4294967295}"#,
            Some(run::Share { turns: Some(u32::MAX), time: Some(Duration::from_secs(u64::from(u32::MAX))) }),
            b"",
        ),
        (br#"{"brief":"inspect","tools":["inspect"],"max_turns":0}"#, None, b"max_turns"),
        (br#"{"brief":"inspect","tools":["inspect"],"max_turns":1.5}"#, None, b"max_turns"),
        (br#"{"brief":"inspect","tools":["inspect"],"max_seconds":0}"#, None, b"max_seconds"),
        (br#"{"brief":"inspect","tools":["inspect"],"max_seconds":1.5}"#, None, b"max_seconds"),
    ];
    for configuration in &client_peer::configurations() {
        for (arguments, expected, field) in cases {
            let (context, mut peer, _owner, completion) = child_completed(configuration, arguments);
            let [shared::Block::ToolCall { arguments: received, .. }] = completion.content.as_ref() else {
                panic!("the provider retains the child call");
            };
            assert_eq!(received.as_ref(), arguments);
            let Event::Below(smith_domain::Below::Completed { completion, .. }) =
                codec::completion(context, completion).expect("observed checked adapter callback")
            else {
                panic!("the observed call has one root completion");
            };
            let [llm::Said::ToolCall { input, call, .. }] = completion.content.as_ref() else {
                panic!("the root retains the original call")
            };
            assert_eq!(input.as_ref(), arguments);
            match expected {
                Some(expected) => {
                    let llm::Decoded::Served { ask: run::Ask::SubAgent { share, .. } } = call else {
                        panic!("valid child quota is admitted: {call:?}")
                    };
                    assert_eq!(*share, expected);
                }
                None => {
                    assert_eq!(
                        *call,
                        llm::Decoded::Invalid { problem: llm::Problem::BadValue { field: field.into() } }
                    );
                }
            }
            close_won(&mut peer);
        }
    }
}

#[test]
fn a_prompt_takes_its_sessions_effort_and_none_for_the_provider_default() {
    let bounds = limits();
    let application = schema(ToolKind::Wait, b"wait");
    for effort in [None, Some(b"high".as_slice())] {
        let mut requested = prompt();
        requested.effort = effort.map(Into::into);
        let translated = smith_protocol_llm::fixture::prompt(requested, core::slice::from_ref(&application), &bounds)
            .expect("admitted prompt");
        assert_eq!(translated.reasoning_effort.as_deref(), effort);
    }
}
