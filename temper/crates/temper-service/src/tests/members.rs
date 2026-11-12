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
"machine":{
 "owned":{"sign_in":null,"listen":{"socket":"127.0.0.1:8368"},"remote":"http://127.0.0.1:8368","forgejo":"fetched",
 "runner":{"binary":"fetched","labels":["temper-host:host"],"capacity":1,"path":"/usr/bin:/bin"},
 "person":{"login":"person","email":"person@localhost"},"public":"https://forge.example.test",
 "mirror":{"forgejo":"https://downloads.example.test","runner":"https://runner.example.test"}},
 "fake":{"listen":{"socket":"127.0.0.1:8369"},"remote":"http://127.0.0.1:8369","git":"demo/bin/git"},
 "github":{"api":{"host":"api.example.test","port":443},"transport":{"tls":{"server_name":"api.example.test","trust":null}},
 "remote":"https://git.example.test","web":"https://web.example.test","git":"demo/bin/github-git"}},
"clients":{"listener":{"socket":"127.0.0.1:8367"},"bundle":"/bundle","public":"https://temper.example.test"},
"welcome":true,
"webhooks":{"listener":{"socket":"127.0.0.1:8371"}},
"sign_in":{"google":{"provider":3,"client":"client-a","secret":"sign-in.secret",
 "endpoints":{"issuer":"https://issuer.example.test","authorization":"https://issuer.example.test/authorize",
 "token":"https://issuer.example.test/token","keys":"https://issuer.example.test/keys",
 "transport":{"tls":{"server_name":"issuer.example.test","trust":"roots.pem"}}}},
 "members":[{"address":"person@example.test","login":"person"}],"forges":[{"forge":1,"client":"forge-client"}]},
"demo":{"llm":{"listen":{"socket":"127.0.0.1:8370"},"provider":1,"key":"demo/llm.key",
 "asked":[{"cue":"Hello","turns":[{"lines":[{"text":"Hello"}],"finish":"stop","tokens":1}]}],
 "cued":[{"cue":"Call","turns":[{"lines":[{"call":"inspect","arguments":{"name":"anything"}}],"finish":"tool_calls","tokens":1}]}],
 "otherwise":["Fallback"]}},
"limits":{"profile":"small","overrides":{}},"workers":[{"root":"workspaces","git":"/usr/bin/git"}],
"landing":{"projects":[{"project":1,"by":"turn","check_timeout_ms":900000,"merge_job":".temper/merge","close_job":".temper/close","gate_retries":2,"gate_timeout_ms":3600000,"tier_retries":2,"verification_bound_ms":7200000,
 "gates":[{"forge":1,"repository":1,"gates":[{"name":"gate","workflow":"gate.yml","ignored":["docs"],"blocking":true,"base":true}],
 "tiers":[{"name":"tier","workflow":"tier.yml","ignored":["docs"],"artifact":"failing","format":"lines","branch":"goal","span_ms":1200000,"description":"the slow suites"}]}]}]}
}"#;

fn expanded() -> Box<[u8]> {
    let input = fixtures::with(&fixtures::document(), b"application", APPLICATION);
    let input = fixtures::with(
        &input,
        b"engine.providers",
        br#"[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"},{"number":3,"kind":"application"}]"#,
    );
    let input = fixtures::with(
        &input,
        b"engine.owners",
        br#"[{"provider":1,"subject":"owner-a"},{"provider":3,"subject":"person@example.test"}]"#,
    );
    fixtures::with(&input, b"engine.clients", br#"{"heartbeat_ms":10000,"link":true,"provider":{"key":"provider.pem","clients":[{"id":"forge-client","secret":"forge.secret","redirect_uri":"https://forge.example.test/user/oauth2/temper/callback"}],"addresses":[]}}"#)
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
    let signing_in = fixtures::with(&input, b"application.machine.owned.sign_in", br#""forge-client""#);
    let signing_in = fixtures::with(&signing_in, b"application.sign_in", b"null");
    let mut signing_in: serde_json::Value = serde_json::from_slice(&signing_in).unwrap();
    let owned = signing_in.pointer_mut("/application/machine/owned").unwrap().as_object_mut().unwrap();
    assert!(owned.remove("public").is_some());
    let signing_in = serde_json::to_vec(&signing_in).unwrap();
    startup::load(&signing_in, &config::Directories::working(), false).expect("box sign-in decoder example");
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
        ("application", What::CharterStored, "application", "charter"),
        ("application.landing", What::LandingStored, "landing", "deployment"),
        ("application.landing.projects[]", What::ProjectLandingStored, "landing-project", "rules"),
    ] {
        let changed = gone(&document, parent, name);
        let problems =
            startup::load_document(&changed, &config::Directories::working(), false, &crate::stock::Given::Stock)
                .expect_err("gone member");
        assert_eq!(problems.problems.len(), 1, "{parent}.{name}: {problems:?}");
        assert_eq!(problems.problems.get(0).unwrap().what, what);
        let row = MEMBERS.iter().find(|row| row.shape == shape && row.name == name).unwrap();
        let lines = String::from_utf8(startup::lines(&problems).into_vec()).unwrap();
        assert!(lines.contains(row.gone.unwrap()), "{lines}");
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
        assert!(!shape.paths.is_empty());
        assert!(!shape.line.is_empty() && shape.line.len() <= 160);
        let names: Vec<_> = super::names(shape.name).collect();
        assert!(!names.is_empty());
        assert_eq!(names.len(), names.iter().collect::<BTreeSet<_>>().len());
    }
    for row in &temper_forge_setup::machine::MEMBERS {
        assert!(
            MEMBERS.iter().any(|member| member.shape == row.shape && member.name == row.name),
            "missing machine row: {row:?}"
        );
    }
    for row in &MEMBERS {
        assert!(shapes.contains(row.shape));
        assert!(!row.kind.is_empty());
        if row.gone.is_none() {
            assert!(!row.line.is_empty() && row.line.len() <= 160 && row.line.ends_with('.'), "{row:?}");
        }
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "configuration member missing from table")]
fn an_absent_member_without_a_row_fails_its_decoder() {
    let document = config::document(b"{}", &LIMITS).unwrap();
    let _member = super::object(&document, 0, "application").unwrap().member(&document, b"unlisted");
}
