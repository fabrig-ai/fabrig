//! Empty assistant history is kept verbatim and marked only on the direct wire.
//! Contract: smith/docs/design/protocol/llm.md, section 2.

use skein_fake_llm_domain::api::{Line, Part, Script, Turn};
use skein_lib::Token;
use skein_llm::{Provider, client};
use skein_llm_kit::fake::Exchange;
use smith_domain::llm;
use smith_domain_session::{self as session, llm as saved};
use smith_session_world::recorded::{Scripted, opening, transcript};

#[test]
fn a_same_endpoint_restore_sends_an_empty_assistant_turn_as_text() {
    let mut first = Scripted::new(9222);
    first.open(opening(None, 100));
    first.complete(Box::new([]), saved::Stop::EndTurn, saved::Usage::ZERO);
    first.close();
    let history = transcript(&first);
    assert!(history.turns.first().expect("recorded turn").messages.last().expect("assistant").content.is_empty());

    let mut restored = Scripted::new(9223);
    restored.open(opening(Some(history.clone()), 100));
    let source = restored.prompts.last().expect("same-endpoint ordinary completion");
    match source.purpose {
        saved::Purpose::Ordinary { .. } => {}
        saved::Purpose::Compaction { .. } => unreachable!("same endpoint does not carry history over"),
    }
    assert_eq!(source.messages.get(1).expect("restored assistant").role, saved::Role::Assistant);
    assert!(source.messages.get(1).expect("restored assistant").content.is_empty());
    let mut control = resolved(source.clone());
    control.messages.get_mut(1).expect("assistant position").content =
        Box::new([llm::Block::Text { text: b"[an empty turn]".as_slice().into(), replay: None }]);

    // An unmarked empty message is invalid at local admission on every attempt.
    let attempts: Vec<_> = (0..3).map(|_| exchange(resolved(source.clone()))).collect();
    let errors: Vec<_> = attempts.iter().map(|attempt| attempt.as_ref().err()).collect();
    assert!(attempts.iter().all(Result::is_ok), "next completion must be admitted, not locally refused: {errors:?}");
    let mut expected = exchange(control).expect("explicit marker control is valid");
    expected.start();
    expected.run();
    for attempt in attempts {
        let mut peer = attempt.expect("admitted above");
        peer.start();
        peer.run();
        assert!(!peer.requests.is_empty(), "the completion goes to the provider");
        assert_eq!(peer.requests, expected.requests, "only the empty message's content changes");
        let query = peer.queries.first().expect("provider received the request");
        assert_eq!(
            query.messages.get(1).expect("assistant on wire").parts.as_ref(),
            &[Part::Text { text: b"[an empty turn]".as_slice().into() }]
        );
    }
    assert!(restored.prompts.last().expect("unchanged history").messages.get(1).expect("assistant").content.is_empty());
    assert!(history.turns.first().expect("stored turn").messages.last().expect("assistant").content.is_empty());
    restored.close();
}

fn exchange(input: llm::Prompt) -> Result<Exchange, skein_llm::Error> {
    let client_limits = skein_llm_kit::limits_for(Provider::Anthropic);
    let stated =
        smith_protocol_llm::Stated::from_tools(&smith_domain_tools_fixtures::SMALL, 4096).expect("stated bounds");
    let limits = smith_protocol_llm_fixtures::small(client_limits, stated);
    let prompt = smith_protocol_llm::prompt(input, &[], &limits).expect("text-only translation");
    let endpoint = skein_llm::Endpoint::anthropic();
    let credential = || skein_llm::Credential::key(b"synthetic-key".as_slice().into());
    let machine = client::Client::prepare(
        skein_llm::Call { owner: Token::new(1), endpoint: endpoint.clone(), credential: Some(credential()), prompt },
        &client_limits,
    )?;
    Ok(Exchange::prepared(
        machine,
        endpoint,
        Some(credential()),
        client_limits,
        Box::new([Script {
            cue: b"instructions".as_slice().into(),
            turns: Box::new([Turn {
                lines: Box::new([Line::Text { text: b"next answer".as_slice().into() }]),
                finish: skein_fake_llm_domain::api::Finish::Stop,
                tokens: 1,
            }]),
        }]),
    ))
}

fn resolved(prompt: session::llm::Prompt) -> llm::Prompt {
    llm::Prompt {
        purpose: prompt.purpose,
        asker: llm::Asker::Main,
        endpoint: prompt.endpoint,
        affinity: prompt.affinity,
        model: prompt.model,
        effort: prompt.effort,
        system: prompt.system,
        tools: prompt.tools,
        namespace: prompt.namespace,
        served: Box::new([]),
        choice: llm::ToolChoice::Auto,
        messages: prompt
            .messages
            .into_iter()
            .map(|message| llm::Message {
                role: message.role,
                content: message
                    .content
                    .into_iter()
                    .map(|block| match block {
                        saved::Block::Text { text, replay } => llm::Block::Text { text, replay },
                        saved::Block::Refusal { .. }
                        | saved::Block::Opaque { .. }
                        | saved::Block::ToolCall(_)
                        | saved::Block::ToolResult { .. } => unreachable!("text-only fixture"),
                    })
                    .collect(),
            })
            .collect(),
        max_tokens: prompt.max_tokens,
    }
}
