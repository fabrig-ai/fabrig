use crate::{What, tests::fixtures};

fn refuses(from: &[u8], to: &[u8], what: What) {
    let document = fixtures::with(fixtures::ENGINE_SMALL, from, to);
    let problems = fixtures::engine(&document).expect_err("refused charter");
    assert_eq!(problems.problems.get(0).expect("one problem").what, what);
}

fn a_charter_whose_budget_cannot_hold_its_reserve_is_refused() {
    // 8192 input plus 1024 output at 1 per 1000 rounds to 10; one reserve turn needs 20.
    refuses(br#""spend": 20"#, br#""spend": 19"#, What::Budget { charter: 1 });
}

#[test]
fn a_charter_whose_budget_holds_its_reserve_is_read() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).expect("valid charter");
    assert_eq!(engine.charters.len(), 1);
    assert_eq!(engine.charters[0].spend, 20);
    assert!(engine.charters[0].documents.is_empty());
}

fn with_documents(value: &[u8]) -> Box<[u8]> {
    let start = b"\"documents\":";
    let end = b",\"instructions\": \"charters/chat.txt\"";
    let mut member =
        skein_lib::Writer::new(start.len().checked_add(value.len()).unwrap().checked_add(end.len()).unwrap());
    member.put(start).unwrap();
    member.put(value).unwrap();
    member.put(end).unwrap();
    let member = member.finish();
    fixtures::with(fixtures::ENGINE_SMALL, br#""instructions": "charters/chat.txt""#, &member)
}

#[test]
fn a_charters_documents_are_read_whole_in_order() {
    let bytes = with_documents(
        br#"[
        {"connector":1,"name":["project","branch"],"path":"docs/rules.md"},
        {"connector":1,"name":["other"],"path":"notes.md","heading":"Rules"}
    ]"#,
    );
    let engine = fixtures::engine(&bytes).expect("two charter documents");
    assert_eq!(
        engine.charters[0].documents.as_ref(),
        &[
            jig_domain_core::Document {
                connector: 1,
                name: jig_domain_authority::Name {
                    segments: Box::new([Box::from(b"project".as_slice()), Box::from(b"branch".as_slice())])
                },
                path: Box::from(b"docs/rules.md".as_slice()),
                heading: None,
            },
            jig_domain_core::Document {
                connector: 1,
                name: jig_domain_authority::Name { segments: Box::new([Box::from(b"other".as_slice())]) },
                path: Box::from(b"notes.md".as_slice()),
                heading: Some(Box::from(b"Rules".as_slice())),
            },
        ]
    );
}

#[test]
fn malformed_charter_documents_are_refused_at_their_paths() {
    let documents = fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"documents");
    let item = documents.child(crate::Segment::Index(0));
    for (value, path, what) in [
        (b"{}".as_slice(), documents, What::Kind),
        (b"[false]", item.clone(), What::Kind),
        (br#"[{"connector":"1","name":[],"path":"a"}]"#, item.member(b"connector"), What::Kind),
        (br#"[{"connector":1,"name":"a","path":"a"}]"#, item.member(b"name"), What::Kind),
        (
            br#"[{"connector":1,"name":[1],"path":"a"}]"#,
            item.member(b"name").child(crate::Segment::Index(0)),
            What::Kind,
        ),
        (br#"[{"connector":1,"name":[],"path":false}]"#, item.member(b"path"), What::Kind),
        (br#"[{"connector":1,"name":[],"path":"a","heading":null}]"#, item.member(b"heading"), What::Kind),
        (br#"[{"connector":1,"name":[],"path":"a","extra":true}]"#, item.member(b"extra"), What::Unknown),
        (br#"[{"name":[],"path":"a"}]"#, item.member(b"connector"), What::Missing),
        (br#"[{"connector":1,"path":"a"}]"#, item.member(b"name"), What::Missing),
        (br#"[{"connector":1,"name":[]}]"#, item.member(b"path"), What::Missing),
        (br#"[{"connector":1,"connector":1,"name":[],"path":"a"}]"#, item.member(b"connector"), What::Duplicate),
    ] {
        let problems = fixtures::engine(&with_documents(value)).expect_err("malformed document");
        let problem = problems.problems.get(0).unwrap();
        assert_eq!((&problem.path, problem.what), (&path, what), "{value:?}");
    }
}

fn a_charter_turns_not_above_its_reserve_are_refused() {
    refuses(br#""turns": 4"#, br#""turns": 1"#, What::Reserve { charter: 1 });
}

fn a_charter_time_not_above_its_reserve_is_refused() {
    refuses(br#""time_ms": 30000"#, br#""time_ms": 5000"#, What::Reserve { charter: 1 });
}

fn a_charter_naming_an_unknown_model_is_refused() {
    refuses(br#""model": "small""#, br#""model": "absent""#, What::Refers);
}

#[test]
fn a_charter_may_name_models_at_two_endpoints() {
    let alternative = br#""models": [{"name":"other", "endpoint":"second", "window":8192,
        "output":1024, "prices":{"input":1,"cached":1,"output":1,"unit":1000}},"#;
    let endpoint = br#""endpoints": [{"name":"second", "number":2, "dialect":0,
        "provider":"fake-provider", "address":{"socket":"127.0.0.1:7102"},"transport":"plaintext",
        "headers":[],"connect_ms":10000,"handshake_ms":10000},"#;
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""models": ["#, alternative);
    let document = fixtures::with(&document, br#""endpoints": ["#, endpoint);
    let document = fixtures::with(&document, br#""models":["small"]"#, br#""models":["small","other"]"#);
    let document = fixtures::with(&document, br#""alternatives": []"#, br#""alternatives": ["other"]"#);
    let engine = fixtures::engine(&document).expect("runtime accounts are selected per provider");
    assert_eq!(engine.charters[0].policy.alternatives.len(), 1);
}

#[test]
fn charters_configuration_problems_name_the_invalid_fields() {
    table([
        (
            "a_charter_whose_budget_cannot_hold_its_reserve_is_refused",
            a_charter_whose_budget_cannot_hold_its_reserve_is_refused,
        ),
        ("a_charter_turns_not_above_its_reserve_are_refused", a_charter_turns_not_above_its_reserve_are_refused),
        ("a_charter_time_not_above_its_reserve_is_refused", a_charter_time_not_above_its_reserve_is_refused),
        ("a_charter_naming_an_unknown_model_is_refused", a_charter_naming_an_unknown_model_is_refused),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}
