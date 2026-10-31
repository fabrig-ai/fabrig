//! A child answers its final turn through session boundaries.
//! Contract: smith/docs/design/domain/session.md, section 7.

use smith_session_world::bytes;

use skein_lib::{Duration, Token};
use smith_domain_session::{self as session, llm};
use smith_session_world::recorded::{Scripted, opening};

#[test]
fn a_childs_final_turn_is_answered_with_stop_budget() {
    for stop in [llm::Stop::ToolUse, llm::Stop::EndTurn, llm::Stop::MaxTokens, llm::Stop::Refusal] {
        let mut world = Scripted::new(72);
        let mut start = opening(None, 1000);
        start.spec.authority.grants.inspect = true;
        start.spec.budget.turns = 1;
        start.spec.final_turn =
            Some(session::FinalTurn { note: bytes(b"Answer with what you have."), allowance: Duration::from_secs(1) });
        world.open(start);
        assert_eq!(world.prompts[0].choice, llm::ToolChoice::None);
        let definitions = world.prompts[0].delegated.clone();
        assert!(!definitions.is_empty(), "definitions stay offered");
        world.complete(
            Box::new([
                llm::Block::Text { text: bytes(b"Found it."), replay: None },
                llm::Block::ToolCall {
                    id: bytes(b"outside"),
                    name: bytes(b"opaque_host"),
                    input: bytes(b"{}"),
                    replay: None,
                    call: llm::Decoded::Delegated {
                        source: session::ToolSource::Host,
                        effect: smith_domain_tools::Effect::Read,
                        descriptor: Token::new(19),
                        ticket: Token::new(991),
                    },
                },
            ]),
            stop,
            llm::Usage::ZERO,
        );
        assert!(world.operations.is_empty() && world.delegated.is_empty());
        world.resume();
        assert_eq!(world.yielded.as_slice(), &[(session::Yield::Budget, Box::<[u8]>::from(b"Found it.".as_slice()))]);
        assert_eq!(world.prompts.len(), 1, "the last answer requests no further provider turn");
        let result = &world.turns[0].messages.last().expect("settled concrete result").content[0];
        assert_eq!(
            result,
            &llm::Block::ToolResult {
                id: bytes(b"outside"),
                result: llm::Returned::NotRun { why: llm::NotRun::OutsideChoice { choice: None } },
            }
        );
        world.close();
    }
}
