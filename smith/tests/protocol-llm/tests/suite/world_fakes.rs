//! Finite world adapters held to the protocol (smith/docs/design/testing-tiers.md).
use smith_domain::{llm, tools};
use smith_domain_session::llm as session;
use smith_session_world::translate;

#[test]
fn the_session_fake_decodes_like_the_protocol() {
    let limits = smith_protocol_llm_world::fixture::limits();
    let grants = tools::Grants { inspect: true, modify: true, shell: true };
    let menu = smith_session_world::provider::menu();
    let deliberate = menu.invalid.len();
    let mut inputs = menu.invalid.into_iter().map(|entry| entry.arguments).collect::<Vec<_>>();
    for input in [
        br#"{"path":"a","offset":2,"window":32,"depth":3,"glob":"*.rs","pattern":"x","content":"c","edits":[{"old":"x","new":"y","all":true}],"command":"ls","timeout_seconds":3,"output_bytes":20,"yield_seconds":2,"job":1,"wait_seconds":2,"stop":true}"#.as_slice(),
        br#"{"path":"a","offset":0}"#,
        br#"{"path":"a","window":0}"#,
        br#"{"path":"a","depth":0}"#,
        br#"{"path":"bad\u0000path"}"#,
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
    let names: &[&[u8]] = &[b"read", b"list", b"search", b"write", b"edit", b"remove", b"shell", b"job", b"unknown"];
    let check = |name: &[u8], arguments: &[u8]| {
        let expected = match smith_protocol_llm::decode(name, arguments, grants, &[], llm::Asker::Main, &limits) {
            llm::Decoded::Owned { call } => session::Decoded::Owned { call },
            llm::Decoded::Invalid { problem } => session::Decoded::Invalid { descriptor: None, problem },
            llm::Decoded::Builtin { .. } | llm::Decoded::Served { .. } => panic!("finite owned tools"),
        };
        assert_eq!(translate::decode(name, arguments), expected, "{name:?}, {arguments:?}");
        expected
    };
    for &name in names {
        for input in menu.arguments.iter().chain(&inputs).map(Box::as_ref).chain([oversized.as_slice()]) {
            check(name, input);
            check(name, &input[..input.len().min(3)]);
        }
    }
    let cut = session::Decoded::Invalid { descriptor: None, problem: session::Problem::NotAnObject };
    // Proper cuts cannot close the root object or pass an admission bound.
    // Admission is shared by all the owned tools and occurs before field
    // decoding, even for the longest literal path. Check one admission per
    // distinct cut, rotating the tool so no cut relies on a single schema.
    let all_bodies = menu.arguments.iter().chain(&inputs[..deliberate]);
    for (prefix, name) in cut_prefixes(all_bodies).into_iter().zip(names[..names.len() - 1].iter().cycle()) {
        assert_eq!(translate::decode(name, prefix), cut);
    }
}

fn cut_prefixes<'a>(bodies: impl IntoIterator<Item = &'a Box<[u8]>>) -> std::collections::BTreeSet<&'a [u8]> {
    bodies.into_iter().flat_map(|input| (0..input.len()).map(|end| &input[..end])).collect()
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
