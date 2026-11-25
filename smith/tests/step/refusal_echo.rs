//! The startup refusal's terminal transcript (smith/docs/design/testing-tiers.md, 3).

use crate::support::refusal_shows_nothing;

const EOF_ECHO: &[u8] = b"^D\x08\x08";

#[test]
fn a_linux_refusal_requires_an_empty_transcript() {
    assert!(refusal_shows_nothing(b"", true), "Linux accepts an empty transcript");
    assert!(!refusal_shows_nothing(EOF_ECHO, true), "Linux does not echo canonical EOF");
}

#[test]
fn a_non_linux_refusal_may_echo_eof_once() {
    assert!(refusal_shows_nothing(b"", false), "an empty transcript is still accepted");
    assert!(refusal_shows_nothing(EOF_ECHO, false), "the canonical EOF echo alone is accepted");
}

#[test]
fn a_non_linux_refusal_rejects_any_other_output() {
    for shown in [b"x".as_slice(), b"^D", b"^D\x08\x08x", b"^D\x08\x08^D\x08\x08"] {
        assert!(!refusal_shows_nothing(shown, false), "{shown:?} is neither empty nor one complete EOF echo");
    }
}
