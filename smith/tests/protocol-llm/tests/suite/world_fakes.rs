//! Finite world adapters held to the protocol (smith/docs/design/testing.md, section 2).
use smith_domain::{llm, tools};
use smith_domain_session::llm as session;
use smith_session_world::translate;

#[test]
fn the_session_fake_decodes_like_the_protocol() {
    let limits = smith_protocol_llm_world::fixture::limits();
    let grants = tools::Grants { inspect: true, modify: true, shell: true };
    let mut inputs = smith_session_world::provider::menu().arguments.into_vec();
    for input in [
        br#"{"path":"a\u002fb","content":"line\nnext","command":"printf \"hi\"","pattern":"x","edits":[{"old":"\u0061","new":"b","all":true}]}"#.as_slice(),
        b"{}", b"{", b"[]", b"null", br#"{"path":"a"}"#,
        br#"{"path":"a","edits":[{}]}"#, br#"{"path":"a","edits":[{"old":"a"}]}"#,
    ] { inputs.push(input.into()); }
    inputs.push(vec![b' '; usize::try_from(limits.client.input).expect("finite input") + 1].into());
    for name in [b"read".as_slice(), b"list", b"search", b"write", b"edit", b"shell", b"unknown"] {
        for input in &inputs {
            for arguments in [input.as_ref(), &input[..input.len().min(3)]] {
                let expected = match smith_protocol_llm::decode(name, arguments, grants, &[], llm::Asker::Main, &limits)
                {
                    llm::Decoded::Owned { call } => session::Decoded::Owned { call },
                    llm::Decoded::Invalid { problem } => session::Decoded::Invalid { descriptor: None, problem },
                    llm::Decoded::Builtin { .. } | llm::Decoded::Served { .. } => panic!("finite owned tools"),
                };
                assert_eq!(translate::decode(name, arguments), expected, "{name:?}, {} bytes", arguments.len());
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
