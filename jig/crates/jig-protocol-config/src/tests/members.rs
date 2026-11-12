//! Rows and decoder requests agree for every configuration shape.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "tests are ordinary Rust (programming-model.md, 10.2)"
)]
use super::fixtures;
use crate::{Defaults, Directories, DocumentLimits, Path, Problems, Segment, What, members, read};
use alloc::collections::BTreeSet;
use skein_json::{Document, Token, document::Kind};

const LIMITS: DocumentLimits = DocumentLimits { bytes: 65536, tokens: 16384, depth: 32 };

fn document(bytes: &[u8]) -> Document {
    crate::document(bytes, &LIMITS).expect("member fixture JSON")
}

// Edit retained tokens, so examples use the existing fixtures without depending on their spacing.
fn tokens(document: &Document) -> Vec<Token> {
    let mut tokens = Vec::new();
    for index in 0..document.len() {
        let token = document.token(index).unwrap();
        let text = || skein_lib::bytes::copy_of(document.text(token).unwrap());
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
            Kind::Long => panic!("member examples retain their text"),
        });
    }
    tokens
}

fn at(document: &Document, path: &str) -> u32 {
    let mut at = 0;
    if path.is_empty() {
        return at;
    }
    for part in path.split('.') {
        at = if part == "0" {
            assert_eq!(document.token(at).unwrap().kind, Kind::ArrayStart);
            at.checked_add(1).unwrap()
        } else {
            document.member(at, part.as_bytes()).expect("fixture parent member")
        };
    }
    at
}

fn path(path: &str) -> Path {
    let mut result = Path::root();
    if !path.is_empty() {
        for part in path.split('.') {
            result = if part == "0" { result.child(Segment::Index(0)) } else { result.member(part.as_bytes()) };
        }
    }
    result
}

fn put(base: &Document, parent: &str, name: &str, value: &[u8]) -> Document {
    let parent = at(base, parent);
    let mut all = tokens(base);
    let value = tokens(&document(value));
    if let Some(start) = base.member(parent, name.as_bytes()) {
        let end = read::after(base, start).unwrap();
        all.splice(usize::try_from(start).unwrap()..usize::try_from(end).unwrap(), value);
    } else {
        let end = read::after(base, parent).unwrap().checked_sub(1).unwrap();
        let mut pair = vec![Token::Key(skein_lib::bytes::copy_of(name.as_bytes()))];
        pair.extend(value);
        all.splice(usize::try_from(end).unwrap()..usize::try_from(end).unwrap(), pair);
    }
    Document::from_tokens(&all, &skein_json::document::Limits { tokens: LIMITS.tokens, text: LIMITS.bytes }).unwrap()
}

fn engine(document: &Document, defaults: Defaults) -> Problems {
    match crate::engine(document, 0, &Directories::working(), defaults) {
        Ok(_) => Problems::empty(),
        Err(problems) => problems,
    }
}

fn worker(document: &Document) -> Problems {
    match crate::worker(document, 0, &Directories::working()) {
        Ok(_) => Problems::empty(),
        Err(problems) => problems,
    }
}

// Optional objects and nonempty lists from the charters, providers, templates, domain and policy
// fixtures. Shared shapes are exercised at every path; alternate forms are exercised below.
const EXTRAS: &[(&str, &str, &[u8])] = &[
    ("charters.0", "resume", b"false"),
    ("charters.0", "parts", br#"[{"on":["first","question"],"instructions":"part.md"}]"#),
    ("charters.0", "models", br#"["small"]"#),
    ("charters.0", "rest", b"true"),
    ("", "acts", br#"[{"connector":1,"may":["close","add_key","remove_account","adopt","make_project","remove_project"]}]"#),
    ("limits", "overrides", b"{}"),
    ("llm.endpoints.0", "transport", br#""tls""#),
    ("llm.endpoints.0", "server_name", br#""issuer.example""#),
    ("llm.endpoints.0", "trust", br#""roots.pem""#),
    ("llm.endpoints.0", "program", br#""claude_code""#),
    ("llm.endpoints.0", "headers", br#"[{"name":"x-version","value":"1"}]"#),
    ("llm.models.0", "efforts", br#"["medium"]"#),
    ("llm.models.0", "effort", br#""medium""#),
    ("llm.models.0", "description", br#""For later work""#),
    ("llm.models.0", "shown", br#""Small""#),
    ("llm.models.0", "picker", b"false"),
    ("llm.models.0", "cache_life_ms", b"300000"),
    ("llm.models.0.prices", "cache_write", b"1"),
    ("llm.providers.0", "endpoint", br#""fake""#),
    ("llm.providers.0", "direct", br#""anthropic-api""#),
    ("llm.providers.0", "keep", br#"["*"]"#),
    ("llm.providers.0", "drop", br#"["*audio*"]"#),
    ("llm.providers.0", "signed_in_plan", b"true"),
    ("llm.providers.0", "manage_usage", br#""https://usage.example/""#),
    ("llm.providers.0", "sources", br#"{"sign_in":{"issuer":"https://issuer.example","client":"client-a","scopes":["chat","offline_access"],"name_hint":"jig","authorization":"https://issuer.example/authorize","token":"https://issuer.example/token","revocation":"https://issuer.example/revoke","keys":"https://issuer.example/keys","transport":"tls","server_name":"issuer.example","trust":"roots.pem","must_grant":["chat"],"resource":"https://resource.example","redirect_path":"callback","redirect_port":8080},"keys":true,"programs":["claude_code"]}"#),
    ("clients", "approval", b"true"),
    ("clients", "provider", br#"{"key":"secrets/provider.pem","clients":[{"id":"app","secret":"secrets/app","redirect_uri":"https://app.example.com/callback"}],"addresses":[]}"#),
    ("charters.0", "documents", br#"[{"connector":1,"name":["service"],"path":"guide.md","heading":"How"}]"#),
    ("charters.0", "wake", br#"{"words":{"batch":{"count":2,"age_ms":5000}},"notices":{"paced":{"span_ms":1000}},"news":"immediate","results":"each","questions":true,"answers":true,"timers":true}"#),
    ("templates.0", "ended_by", br#""requester""#),
    ("templates.0", "resources", br#"[{"connector":1,"symbol":1}]"#),
    ("templates.0", "deadline", br#"{"span_ms":1000}"#),
    ("templates.0", "llm", br#"{"main":{"model":"small","effort":"medium"},"sub_agents":[{"model":"small","effort":"medium"}]}"#),
    ("templates.0.llm", "later", br#"{"main":{"model":"small","effort":"medium"},"sub_agents":[{"model":"small","effort":"medium"}]}"#),
    ("projects.0", "name", br#""Project One""#),
    ("projects.0.policy", "templates", br#"[{"template":1,"executor":{"charter":1},"budget":{"amount":10},"deadline":{"span_ms":1000},"priority":5}]"#),
    ("", "policies", b"[]"),
];

const GRANTS: &[u8] = br#"[{"connector":1,"kind":2,"pattern":{"segments":["service"],"last":"open","value":""}}]"#;
const SCOPES: &[u8] = br#"[{"connector":1,"pattern":{"segments":["service"],"last":"open","value":""}}]"#;
const REQUIREMENTS: &[u8] = br#"[{"connector":1,"kind":2,"pattern":{"segments":[],"last":"open","value":""},"judge":{"connector":1,"requirement":3,"parameters":4},"guard":"observed","freshness_ms":5000,"must_be_guarded":false}]"#;

fn expanded_engine() -> Document {
    let mut base = document(fixtures::ENGINE_SMALL);
    let direct = document(
        br#"{
        "name":"anthropic-api","number":4,"dialect":3,"provider":"fake-provider",
        "address":{"socket":"127.0.0.1:7101"},"transport":"plaintext",
        "headers":[],"connect_ms":10000,"handshake_ms":10000
    }"#,
    );
    let mut all = tokens(&base);
    let end = usize::try_from(read::after(&base, at(&base, "llm.endpoints")).unwrap().checked_sub(1).unwrap()).unwrap();
    all.splice(end..end, tokens(&direct));
    base = Document::from_tokens(&all, &skein_json::document::Limits { tokens: LIMITS.tokens, text: LIMITS.bytes })
        .unwrap();
    for (parent, name, value) in EXTRAS {
        base = put(&base, parent, name, value);
    }
    for parent in ["rules.ceiling", "projects.0.policy.ceiling", "projects.0.policy.roles.0.authority"] {
        base = put(&base, parent, "models", br#"["small"]"#);
        base = put(&base, parent, "grants", GRANTS);
        base = put(&base, parent, "note_resources", SCOPES);
    }
    base = put(&base, "projects.0.policy", "projections", GRANTS);
    base = put(&base, "rules", "requirements", REQUIREMENTS);
    base = put(&base, "projects.0.policy", "requirements", REQUIREMENTS);
    base = put(&base, "rules", "implies", br#"[{"connector":1,"kind":1,"implies":2}]"#);
    // Copy the existing seed policy's tokens into a numbered default policy, as policies.rs does.
    let project = at(&base, "projects.0");
    let end = read::after(&base, project).unwrap();
    let all = tokens(&base);
    let mut policy = vec![Token::ArrayStart];
    policy.extend_from_slice(&all[usize::try_from(project).unwrap()..usize::try_from(end).unwrap()]);
    policy.push(Token::ArrayEnd);
    let policy =
        Document::from_tokens(&policy, &skein_json::document::Limits { tokens: LIMITS.tokens, text: LIMITS.bytes })
            .unwrap();
    // A configured policy has no project name.
    let mut all = tokens(&policy);
    let name = policy.member(1, b"name").unwrap();
    all.drain(
        usize::try_from(name.checked_sub(1).unwrap()).unwrap()
            ..usize::try_from(read::after(&policy, name).unwrap()).unwrap(),
    );
    let mut base_tokens = tokens(&base);
    let start = base.member(0, b"policies").unwrap();
    base_tokens
        .splice(usize::try_from(start).unwrap()..usize::try_from(read::after(&base, start).unwrap()).unwrap(), all);
    Document::from_tokens(&base_tokens, &skein_json::document::Limits { tokens: LIMITS.tokens, text: LIMITS.bytes })
        .unwrap()
}

struct Example {
    document: Document,
    worker: bool,
    confidential: bool,
}

fn examples() -> Vec<Example> {
    let base = expanded_engine();
    assert!(engine(&base, Defaults::Document).is_empty(), "expanded fixture: {:?}", engine(&base, Defaults::Document));
    let mut examples = vec![Example { document: base.clone(), worker: false, confidential: false }];
    for (parent, name, value) in [
        ("templates.0", "contract", br#"{"change":{"connector":1,"kind":1,"words":512}}"#.as_slice()),
        (
            "templates.0",
            "contract",
            br#"{"verdict":{"choices":[{"code":1,"words":128,"followups":1,"fields":[{"name":"reason","max":64}]}]}}"#,
        ),
        ("templates.0", "budget", br#"{"amount":10}"#),
        ("templates.0", "tracked", br#"{"priority":1}"#),
        ("llm.endpoints.0", "address", br#"{"host":"issuer.example","port":443}"#),
        ("llm.providers.0.sources.sign_in", "secret", br#""refused""#),
    ] {
        examples.push(Example {
            document: put(&base, parent, name, value),
            worker: false,
            confidential: name == "secret",
        });
    }
    let closer = put(&base, "templates.0", "conversation", b"false");
    let closer = put(&closer, "templates.0", "ended_by", br#"{"closer":{"connector":1,"code":6}}"#);
    examples.push(Example { document: closer, worker: false, confidential: false });
    examples.push(Example { document: document(fixtures::WORKER_SMALL), worker: true, confidential: false });
    examples
}

fn members_at(document: &Document, at: u32) -> Vec<&[u8]> {
    let end = read::after(document, at).unwrap().checked_sub(1).unwrap();
    let mut next = at.checked_add(1).unwrap();
    let mut names = Vec::new();
    for _ in at..end {
        if next == end {
            break;
        }
        names.push(document.text(document.token(next).unwrap()).unwrap());
        next = read::after(document, next.checked_add(1).unwrap()).unwrap();
    }
    names
}

fn relative(shape_path: &str) -> (&str, bool) {
    if let Some(path) = shape_path.strip_prefix("engine") {
        (path.strip_prefix('.').unwrap_or(path), false)
    } else {
        let path = shape_path.strip_prefix("workers[]").expect("engine or worker shape path");
        (path.strip_prefix('.').unwrap_or(path), true)
    }
}

fn concrete(path: &str) -> String {
    path.replace("[]", ".0")
}

#[test]
fn every_row_is_a_member_its_decoder_takes() {
    let examples = examples();
    let mut seen = BTreeSet::new();
    for example in &examples {
        let problems =
            if example.worker { worker(&example.document) } else { engine(&example.document, Defaults::Document) };
        assert_eq!(problems.past, 0, "all member diagnostics are retained");
        if example.confidential {
            assert_eq!(problems.problems.len(), 2, "{problems:?}");
            assert_eq!(problems.problems.get(0).unwrap().what, What::Confidential);
            // Refusing the registration also removes the endpoint's offered program provider.
            assert_eq!(problems.problems.get(1).unwrap().what, What::Refers);
        } else {
            assert!(problems.is_empty(), "member example: {problems:?}");
        }
        for shape in &members::SHAPES {
            for shape_path in shape.paths {
                let (relative, is_worker) = relative(shape_path);
                if is_worker != example.worker {
                    continue;
                }
                let relative = concrete(relative);
                // Some examples choose another variant of an object, so this path need not exist.
                let mut object_at = Some(0_u32);
                for part in relative.split('.').filter(|part| !part.is_empty()) {
                    object_at = object_at.and_then(|at| {
                        if part == "0" {
                            let next = at.checked_add(1).unwrap();
                            (example.document.token(next)?.kind != Kind::ArrayEnd).then_some(next)
                        } else {
                            example.document.member(at, part.as_bytes())
                        }
                    });
                }
                let Some(object_at) = object_at else { continue };
                if example.document.token(object_at).unwrap().kind != Kind::ObjectStart {
                    continue;
                }
                for name in members_at(&example.document, object_at) {
                    let member_path = path(&relative).member(name);
                    assert!(
                        !problems
                            .problems
                            .iter()
                            .any(|problem| problem.path == member_path && problem.what == What::Unknown),
                        "{shape_path}: {name:?}: {problems:?}"
                    );
                    let row = members::MEMBERS
                        .iter()
                        .find(|row| row.shape == shape.name && row.name.as_bytes() == name)
                        .expect("fixture member has a row");
                    seen.insert((row.shape, row.name));
                }
            }
        }
    }
    for row in &members::MEMBERS {
        assert!(seen.contains(&(row.shape, row.name)), "row missing a decoder example: {}.{}", row.shape, row.name);
    }
}

#[test]
fn a_member_a_decoder_asks_for_is_a_row() {
    // Each decoder request, including absent optional members, goes through the debug lookup.
    assert!(engine(&document(fixtures::ENGINE_SMALL), Defaults::Document).is_empty());
    assert!(worker(&document(fixtures::WORKER_SMALL)).is_empty());
    for example in examples() {
        if example.worker {
            let _problems = worker(&example.document);
        } else {
            let _problems = engine(&example.document, Defaults::Document);
        }
    }
    let problems = engine(&expanded_engine(), Defaults::Passed);
    for name in ["charters", "templates", "policies", "rules"] {
        assert!(
            problems
                .problems
                .iter()
                .any(|problem| problem.path == Path::root().member(name.as_bytes()) && problem.what == What::Stored)
        );
        let row = members::MEMBERS.iter().find(|row| row.shape == "engine" && row.name == name).unwrap();
        assert_eq!(row.gone, Some("settings live in the store"));
    }
}

#[test]
fn each_line_is_within_its_bound() {
    for line in members::MEMBERS.iter().map(|row| row.line).chain(members::SHAPES.iter().map(|shape| shape.line)) {
        assert!(!line.is_empty() && line.len() <= 160, "{line}");
        assert!(line.ends_with('.'), "{line}");
        assert_eq!(line.bytes().filter(|byte| *byte == b'.').count(), 1, "{line}");
        assert!(!line.contains(['\n', '\r']), "{line}");
    }
    let mut seen = BTreeSet::new();
    for shape in &members::SHAPES {
        assert!(!shape.paths.is_empty(), "{}", shape.name);
        assert!(seen.insert(shape.name), "duplicate shape: {}", shape.name);
        let names: Vec<_> = members::names(shape.name).collect();
        assert!(!names.is_empty());
        let unique: BTreeSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "duplicate member in {}", shape.name);
    }
    for row in &members::MEMBERS {
        assert!(seen.contains(row.shape), "member of an unknown shape: {}", row.shape);
        assert!(!row.kind.is_empty(), "{}.{}", row.shape, row.name);
    }
    assert_eq!(members::names("absent").count(), 0);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "configuration member missing from table")]
fn a_decoder_request_without_a_row_panics_even_when_the_member_is_absent() {
    let document = document(b"{}");
    let mut object = read::Object::shaped(&document, 0, "engine").unwrap();
    let _value = object.member(&document, b"unlisted");
}
