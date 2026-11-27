//! Finite world adapters held to the protocol (smith/docs/design/testing-tiers.md).
use smith_domain::{llm, tools};
use smith_domain_session::llm as session;
use smith_session_world::translate;

#[test]
fn the_session_fake_decodes_like_the_protocol() {
    const CUT_PREFIX: usize = 3;
    let limits = smith_protocol_llm_world::fixture::limits();
    let grants = tools::Grants { inspect: true, modify: true, shell: true };
    let menu = smith_session_world::provider::menu();
    let mut inputs = menu.invalid.into_iter().map(|entry| entry.arguments).collect::<Vec<_>>();
    for input in [
        br#"{"path":"a\u002fb","content":"line\nnext","command":"printf \"hi\"","pattern":"x","edits":[{"old":"\u0061","new":"b","all":true}]}"#.as_slice(),
        b"{}", b"{", b"[]", b"null", br#"{"path":"a"}"#,
        br#"{"path":"a","edits":[{}]}"#, br#"{"path":"a","edits":[{"old":"a"}]}"#,
        br#"{"job":1}"#, br#"{"job":"1"}"#, br#"{"job":0}"#,
        br#"{"job":1,"wait_seconds":0,"stop":false}"#,
        br#"{"job":1,"wait_seconds":2,"output_bytes":20,"stop":true}"#,
        br#"{"job":1,"wait_seconds":"2"}"#, br#"{"job":1,"wait_seconds":-1}"#,
        br#"{"job":1,"stop":1}"#, br#"{"job":1,"output_bytes":0}"#,
        br#"{"command":"ls","yield_seconds":0}"#,
        br#"{"command":"ls","yield_seconds":2,"timeout_seconds":3,"output_bytes":20}"#,
        br#"{"command":"ls","yield_seconds":"2"}"#,
        br#"{"command":"ls","yield_seconds":-1}"#,
        br#"{"command":"ls","yield_seconds":4294967296}"#,
    ] { inputs.push(input.into()); }
    let oversized = vec![b' '; usize::try_from(limits.client.input).expect("finite input") + 1];
    for name in [b"read".as_slice(), b"list", b"search", b"write", b"edit", b"shell", b"job", b"unknown"] {
        let check = |arguments: &[u8]| {
            let expected = match smith_protocol_llm::decode(name, arguments, grants, &[], llm::Asker::Main, &limits) {
                llm::Decoded::Owned { call } => session::Decoded::Owned { call },
                llm::Decoded::Invalid { problem } => session::Decoded::Invalid { descriptor: None, problem },
                llm::Decoded::Builtin { .. } | llm::Decoded::Served { .. } => panic!("finite owned tools"),
            };
            assert_eq!(translate::decode(name, arguments), expected, "{name:?}, {arguments:?}");
        };
        for input in menu.arguments.iter().chain(&inputs).map(Box::as_ref).chain([oversized.as_slice()]) {
            check(input);
            check(&input[..input.len().min(CUT_PREFIX)]);
        }
        // Exhaust the cut positions in the randomized bodies for the two job decoders.
        if matches!(name, b"job" | b"shell") {
            for input in &menu.arguments {
                for end in (0..input.len()).filter(|end| *end != CUT_PREFIX) {
                    check(&input[..end]);
                }
            }
        }
    }
}

#[test]
fn the_session_fake_renders_path_failures_like_the_protocol() {
    use tools::{Namespace, Outcome, Resolved};
    let name = || tools::Name::new(b"work".as_slice().into()).expect("name");
    for namespace in [Namespace::NoWorkspace, Namespace::One, Namespace::Several { working: name() }] {
        for path in [b"".as_slice(), b"src/missing.rs"] {
            for path in [
                Resolved::Working { path: path.into() },
                Resolved::Other { directory: name(), path: path.into() },
                Resolved::Outside { path: path.into() },
            ] {
                for outcome in [
                    Outcome::NotFound { path: path.clone() },
                    Outcome::NotDirectory { path: path.clone() },
                    Outcome::Outside { path: path.clone() },
                    Outcome::ReadOnly { path },
                ] {
                    let expected =
                        smith_protocol_llm::render_outcome(&outcome, &namespace, 16_384).expect("finite path");
                    assert_eq!(
                        translate::render_with_namespace(&session::Returned::Owned { outcome }, &namespace),
                        expected
                    );
                }
            }
        }
    }
}
