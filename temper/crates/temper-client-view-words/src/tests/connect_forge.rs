#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "test assertions inspect the semantic tree")]
use super::fixtures as f;
use temper_client_domain as d;

// The mockup's failed frame, and each other step of the same five-step flow.

#[test]
fn setup_failure_words_keep_the_full_bounded_reason() {
    let words = f::words();
    let bound = usize::try_from(temper_client_protocol_wire::SET_UP_WHY_BYTES).unwrap();
    for (bytes, reason) in [(b"x".repeat(bound), "x".repeat(bound)), ([0xff].repeat(bound), "�".repeat(bound))] {
        let shown = d::forge::words::lossy(&bytes);
        let text = words.say(crate::words::Word::ForgePrepareFailed, &[skein_ui::phrases::Hole::Name(shown)]);
        let mut expected = String::from("Preparing failed: ");
        expected.push_str(&reason);
        assert_eq!(text.as_ref(), expected.as_bytes());
    }
}
