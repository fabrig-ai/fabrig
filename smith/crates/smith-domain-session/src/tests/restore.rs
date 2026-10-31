//! Concrete history and resumed session contracts.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::*;

#[test]
fn a_resumed_ordinary_window_carries_the_guard() {
    let mut original = Harness::new(&LIMITS);
    original.yielded();
    let mut resumed = Harness::new(&LIMITS);
    let mut opening = opening(spec());
    opening.transcript = Some(crate::record::Transcript {
        version: crate::record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 1,
        turns: original.turn_records.to_boxed(),
    });
    step(&mut resumed.domain, &resumed.env, Event::Open { opener: Token::new(2), opening }, &mut resumed.out);
    let Some(Request::Opened { opener, .. }) = resumed.next() else { panic!("restored session opens") };
    assert_eq!(opener, Token::new(2));
    let (_, prompt) = calling(resumed.one());
    assert_eq!(
        prompt.purpose,
        crate::llm::Purpose::Ordinary { guard: Some(crate::llm::Guard { line: 64_512, last_prompt: 15, from: 1 }) }
    );
}
