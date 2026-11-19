//! Members tables agree with the decoders (temper/docs/design/protocol/documentation.md, 6 and 8).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "step tests inspect configuration fixtures with ordinary Rust"
)]
use super::{MEMBERS, SHAPES};
use crate::{
    startup::{self, What},
    tests::fixtures,
};
use alloc::collections::BTreeSet;
use jig_protocol_config::{self as config, Path, read};

const LIMITS: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
const APPLICATION: &[u8] = br#"{
 "test":{"github":{"api":{"host":"api.example.test","port":443},"transport":{"tls":{"server_name":"api.example.test","trust":null}},
 "remote":"https://git.example.test","web":"https://web.example.test","git":"demo/bin/github-git"}},
"clients":{"listener":{"socket":"127.0.0.1:8367"},"bundle":"/bundle","public":"https://temper.example.test"},
"push":{"key":"push.key","subject":"mailto:operator@example.test","services":null},
"webhooks":{"listener":{"socket":"127.0.0.1:8371"}},
"sign_in":{"google":{"provider":3,"client":"client-a","secret":"sign-in.secret",
 "endpoints":{"issuer":"https://issuer.example.test","authorization":"https://issuer.example.test/authorize",
 "token":"https://issuer.example.test/token","keys":"https://issuer.example.test/keys",
 "transport":{"tls":{"server_name":"issuer.example.test","trust":"roots.pem"}}}},
 "members":[{"address":"person@example.test","login":"person"}],"forges":[{"forge":1,"client":"forge-client"}]},
"environment":[{"name":"CARGO_TARGET_DIR","value":"/srv/target"}]
}"#;

fn expanded() -> fixtures::Start {
    let input = fixtures::with(&fixtures::turn_document(), b"application", APPLICATION);
    let input = fixtures::with(
        &input,
        b"engine.owners",
        br#"[{"provider":1,"subject":"owner-a"},{"provider":3,"subject":"person@example.test"}]"#,
    );
    fixtures::with(&input, b"engine.clients", br#"{"provider":{"key":"provider.pem","clients":[{"id":"forge-client","secret":"forge.secret","redirect_uri":"https://forge.example.test/user/oauth2/temper/callback"}],"addresses":[]}}"#)
}

fn objects(document: &skein_json::Document, path: &str) -> Vec<(u32, Path)> {
    let mut ats = vec![(0, Path::root())];
    for part in path.split('.').filter(|part| !part.is_empty()) {
        let array = part.ends_with("[]");
        let name = part.trim_end_matches("[]");
        ats = ats
            .into_iter()
            .flat_map(|(at, path)| {
                let path = path.member(name.as_bytes());
                document.member(at, name.as_bytes()).map_or_else(Vec::new, |at| {
                    if array {
                        read::list(document, at, document.len())
                            .unwrap()
                            .into_boxed()
                            .into_iter()
                            .enumerate()
                            .map(|(index, at)| (at, path.child(config::Segment::Index(u32::try_from(index).unwrap()))))
                            .collect()
                    } else {
                        vec![(at, path)]
                    }
                })
            })
            .collect();
    }
    ats
}

fn gone(document: &skein_json::Document, parent: &str, name: &str) -> skein_json::Document {
    use skein_json::{Token, document::Kind};
    let (at, _) = objects(document, parent).into_iter().next().unwrap();
    let end = read::after(document, at).unwrap().checked_sub(1).unwrap();
    let mut tokens = Vec::new();
    for index in 0..document.len() {
        if index == end {
            tokens.extend([Token::Key(name.as_bytes().into()), Token::ObjectStart, Token::ObjectEnd]);
        }
        let token = document.token(index).unwrap();
        let text = || Box::from(document.text(token).unwrap());
        tokens.push(match token.kind {
            Kind::ObjectStart => Token::ObjectStart,
            Kind::ObjectEnd => Token::ObjectEnd,
            Kind::ArrayStart => Token::ArrayStart,
            Kind::ArrayEnd => Token::ArrayEnd,
            Kind::Key => Token::Key(text()),
            Kind::String => Token::String(text()),
            Kind::Number => Token::Number(text()),
            Kind::True => Token::True,
            Kind::False => Token::False,
            Kind::Null => Token::Null,
            Kind::Long => panic!("whole fixture"),
        });
    }
    skein_json::Document::from_tokens(
        &tokens,
        &skein_json::document::Limits { tokens: LIMITS.tokens, text: LIMITS.bytes },
    )
    .unwrap()
}

#[test]
fn every_row_is_a_member_its_decoder_takes() {
    let input = expanded();
    startup::load(&input, &config::Directories::working(), false).expect("expanded decoder example");
    let document = config::document(&input, &LIMITS).unwrap();
    let mut seen = BTreeSet::new();
    for shape in &SHAPES {
        for path in shape.paths {
            for (at, _) in objects(&document, path) {
                if document.token(at).unwrap().kind != skein_json::document::Kind::ObjectStart {
                    continue;
                }
                for row in MEMBERS.iter().filter(|row| row.shape == shape.name) {
                    if document.member(at, row.name.as_bytes()).is_some() {
                        seen.insert((row.shape, row.name));
                    }
                }
            }
        }
    }
    // Gone members are consumed and refused by their owning decoder, with the table's remedy.
    for (parent, what, shape, name) in [
        ("application", What::Half(startup::EngineWhat::CharterStored), "application", "charter"),
        ("application", What::Jig(config::What::Fixed), "application", "landing"),
        ("application", What::Jig(config::What::Fixed), "application", "workers"),
        ("application", What::Jig(config::What::Fixed), "application", "machine"),
        ("application", What::Jig(config::What::Fixed), "application", "welcome"),
        ("application", What::Jig(config::What::Fixed), "application", "limits"),
        ("application", What::Jig(config::What::Fixed), "application", "demo"),
    ] {
        let changed = gone(&document, parent, name);
        let problems =
            startup::load_document(&changed, &config::Directories::working(), false, &crate::stock::Given::Stock)
                .expect_err("gone member");
        assert_eq!(problems.problems.len(), 1, "{parent}.{name}: {problems:?}");
        assert_eq!(problems.problems.get(0).unwrap().what, what);
        let row = MEMBERS.iter().find(|row| row.shape == shape && row.name == name).unwrap();
        let lines = String::from_utf8(startup::lines(&problems).into_vec()).unwrap();
        let words = row.gone.unwrap_or_else(|| temper_service_core::startup::moved::child(parent, name).unwrap().words);
        assert!(lines.contains(words), "{lines}");
        seen.insert((shape, name));
    }
    for row in &MEMBERS {
        assert!(seen.contains(&(row.shape, row.name)), "missing decoder example: {}.{}", row.shape, row.name);
    }
}

#[test]
fn the_rows_have_unique_names_and_bounded_lines() {
    let mut shapes = BTreeSet::new();
    for shape in &SHAPES {
        assert!(shapes.insert(shape.name), "duplicate shape: {}", shape.name);
        assert_ne!(shape.paths.len(), 0);
        assert!(!shape.line.is_empty() && shape.line.len() <= 160);
        let names: Vec<_> = super::names(shape.name).collect();
        assert_ne!(names.len(), 0);
        assert_eq!(names.len(), names.iter().collect::<BTreeSet<_>>().len());
    }
    for row in temper_forge_setup::machine::MEMBERS.iter().filter(|row| row.shape.starts_with("test")) {
        assert!(
            MEMBERS.iter().any(|member| member.shape == row.shape && member.name == row.name),
            "missing machine row: {row:?}"
        );
    }
    for row in &MEMBERS {
        assert!(shapes.contains(row.shape));
        assert_ne!(row.kind, "");
        if row.gone.is_none() {
            assert!(!row.line.is_empty() && row.line.len() <= 160 && row.line.ends_with('.'), "{row:?}");
        }
    }
}

#[test]
fn test_peers_are_optional_and_only_github_is_taken_at_the_new_path() {
    let input =
        serde_json::json!({"application": serde_json::from_slice::<serde_json::Value>(fixtures::APPLICATION).unwrap()});
    let engine = fixtures::engine().jig;
    let parse = |input: &[u8]| {
        let document = config::document(input, &LIMITS).unwrap();
        let mut problems = startup::Problems::empty();
        let application = startup::application::parse(
            &document,
            document.member(0, b"application").unwrap(),
            &engine,
            &config::Directories::working(),
            &mut problems,
        );
        if problems.is_empty() { Ok(application.expect("parsed application")) } else { Err(problems) }
    };
    for test in [None, Some(serde_json::json!({})), Some(serde_json::json!({"github":null}))] {
        let mut input = input.clone();
        input["application"].as_object_mut().unwrap().remove("test");
        if let Some(test) = test {
            input["application"]["test"] = test;
        }
        let input = serde_json::to_vec(&input).unwrap();
        let parsed = parse(&input).expect("optional test");
        assert!(parsed.test_github.is_none());
    }
    for (test, expected) in [
        (serde_json::json!({"other":null}), "application.test.other: unknown member\n"),
        (serde_json::json!(null), "application.test: wrong value kind\n"),
        (serde_json::json!({"github":false}), "application.test.github: wrong value kind\n"),
    ] {
        let mut input = input.clone();
        input["application"]["test"] = test;
        let input = serde_json::to_vec(&input).unwrap();
        let problems = parse(&input).expect_err("invalid test");
        assert_eq!(startup::lines(&problems).as_ref(), expected.as_bytes());
    }
    let mut input = input;
    input["application"]["machine"]["github"] = serde_json::Value::Null;
    let problems = parse(&serde_json::to_vec(&input).unwrap()).expect_err("machine.github is unknown");
    assert_eq!(
        startup::lines(&problems).as_ref(),
        b"application.machine: moved: a forge's place is chosen when it's set up, and kept with the forge\n"
    );
}
