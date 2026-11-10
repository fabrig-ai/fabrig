//! The short path through main on fakes (temper/docs/design/testing/process-tier.md).
use crate::support;

#[test]
fn a_scratch_binary_answers_one_chat_and_drains() {
    let scratch = support::Scratch::new("main-wiring");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let peer = support::fake_llm(Box::new([support::chat_script(support::ANSWER)]));
    let seen = support::run(
        scratch.path(),
        support::person::answered_chat(b"Hello", support::ANSWER),
        Some(peer),
        support::WITHIN,
    );
    assert_eq!((seen.signed_in, seen.finished), (1, true));
    scratch.remove();
}
