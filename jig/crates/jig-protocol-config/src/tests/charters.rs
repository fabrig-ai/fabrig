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
    assert!(engine.charters[0].parts.is_empty());
    assert!(engine.charters[0].rest);
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""model": "small", "alternatives": []"#,
        br#""models": ["small"], "rest": false"#,
    );
    let named = fixtures::engine(&document).expect("named first choice");
    assert!(!named.charters[0].rest);
    assert_eq!(named.charters[0].policy.model, engine.charters[0].policy.model);
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""model": "small", "alternatives": [],"#, b"");
    let all = fixtures::engine(&document).expect("rest without named choices");
    assert!(all.charters[0].rest);
    assert_eq!(all.charters[0].policy.model, engine.charters[0].policy.model);
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
        {"connector":1,"name":["project","resource"],"path":"docs/rules.md"},
        {"connector":1,"name":["other"],"path":"notes.md","heading":"Rules"}
    ]"#,
    );
    let engine = fixtures::engine(&bytes).expect("two charter documents");
    assert_eq!(
        engine.charters[0].documents.as_ref(),
        &[
            jig_domain_core_vocabulary::Document {
                connector: 1,
                name: jig_domain_authority::Name {
                    segments: Box::new([Box::from(b"project".as_slice()), Box::from(b"resource".as_slice())])
                },
                path: Box::from(b"docs/rules.md".as_slice()),
                heading: None,
            },
            jig_domain_core_vocabulary::Document {
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
    malformed_parts_are_refused_at_their_paths();
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

fn with_parts(value: &[u8]) -> Box<[u8]> {
    let member = skein_lib::bytes::concat(&[br#""parts":"#, value, br#","instructions": "charters/chat.txt""#]);
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""instructions": "charters/chat.txt""#, &member);
    fixtures::with(&document, br#""resume": true"#, br#""resume": false"#)
}

fn part_path() -> crate::Path {
    fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"parts")
}

fn refuses_parts(value: &[u8], path: crate::Path, what: What) {
    let problems = fixtures::engine(&with_parts(value)).expect_err("refused parts");
    assert_eq!(problems.problems.get(0), Some(&crate::Problem { path, what }));
}

fn a_part_refuses_an_unknown_reason() {
    refuses_parts(
        br#"[{"on":["unknown"],"instructions":"a.md"}]"#,
        part_path().child(crate::Segment::Index(0)).member(b"on").child(crate::Segment::Index(0)),
        What::Unknown,
    );
}

fn a_part_refuses_always_offered_reasons() {
    for name in jig_domain_tasks::ALWAYS_OFFERED_NAMES {
        let value = skein_lib::bytes::concat(&[br#"[{"on":[""#, name, br#""],"instructions":"a.md"}]"#]);
        refuses_parts(
            &value,
            part_path().child(crate::Segment::Index(0)).member(b"on").child(crate::Segment::Index(0)),
            What::Unknown,
        );
    }
}

fn a_part_refuses_a_repeated_reason() {
    refuses_parts(
        br#"[{"on":["first","first"],"instructions":"a.md"}]"#,
        part_path().child(crate::Segment::Index(0)).member(b"on").child(crate::Segment::Index(1)),
        What::Unique,
    );
}

fn a_part_refuses_no_reasons() {
    refuses_parts(
        br#"[{"on":[],"instructions":"a.md"}]"#,
        part_path().child(crate::Segment::Index(0)).member(b"on"),
        What::Bound,
    );
}

#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "test fixture lists are ordinary Rust (programming-model.md, 10.2)"
)]
fn a_charter_refuses_parts_past_its_limit() {
    let count = crate::profile(crate::Profile::Small).core.tasks.charter_parts;
    let parts =
        vec![r#"{"on":["first"],"instructions":"a.md"}"#; usize::try_from(count.checked_add(1).unwrap()).unwrap()];
    let value = format!("[{}]", parts.join(","));
    refuses_parts(value.as_bytes(), part_path().child(crate::Segment::Index(count)), What::Bound);
}

#[test]
fn a_resuming_charter_admits_parts() {
    let document = with_parts(br#"[{"on":["first"],"instructions":"a.md"}]"#);
    let document = fixtures::with(&document, br#""resume": false"#, br#""resume": true"#);
    let engine = fixtures::engine(&document).expect("resuming charter with parts");
    assert!(engine.charters[0].policy.resume);
    assert_eq!(engine.charters[0].parts.len(), 1);
}

fn a_charter_refuses_policy_and_parts_past_run_bytes() {
    let document = with_parts(br#"[{"on":["first"],"instructions":"a.md"}]"#);
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    let max = jig_domain_core_vocabulary::policy_owned_bytes(&engine.charters[0].policy).unwrap();
    let max = skein_lib::Decimal::of(max);
    let limits =
        skein_lib::bytes::concat(&[br#""profile": "small", "overrides": {"core.run_bytes":"#, max.as_bytes(), b"}"]);
    let document = fixtures::with(&document, br#""profile": "small""#, &limits);
    let problems = fixtures::engine(&document).expect_err("parts exceed remaining run bytes");
    assert_eq!(
        problems.problems.get(0),
        Some(&crate::Problem {
            path: fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)),
            what: What::Bound
        })
    );
}

fn a_charter_reads_two_parts_and_their_files_in_order() {
    let document = with_parts(
        br#"[{"on":["first","question"],"instructions":"a.md"},
        {"on":["news","words"],"instructions":"b.md"}]"#,
    );
    let engine = fixtures::engine(&document).expect("two charter parts");
    let charter = &engine.charters[0];
    assert_eq!(
        charter.parts.as_ref(),
        &[
            jig_domain_core_vocabulary::CharterPart {
                on: jig_domain_tasks::Reasons::FIRST.union(jig_domain_tasks::Reasons::QUESTION),
                instructions: b"a.md".as_slice().into()
            },
            jig_domain_core_vocabulary::CharterPart {
                on: jig_domain_tasks::Reasons::NEWS.union(jig_domain_tasks::Reasons::WORDS),
                instructions: b"b.md".as_slice().into()
            },
        ]
    );
    let files = crate::files(&engine);
    assert_eq!(
        files.len(),
        crate::files(&fixtures::engine(fixtures::ENGINE_SMALL).unwrap()).len().checked_add(2).unwrap()
    );
    let owned = jig_domain_core_vocabulary::policy_owned_bytes(&charter.policy)
        .unwrap()
        .checked_add(jig_domain_core_vocabulary::parts_owned_bytes(&charter.parts).unwrap())
        .unwrap();
    for (file, (path, at)) in files.iter().zip([
        (
            charter.policy.instructions.as_ref(),
            fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"instructions"),
        ),
        (b"a.md".as_slice(), part_path().child(crate::Segment::Index(0)).member(b"instructions")),
        (b"b.md".as_slice(), part_path().child(crate::Segment::Index(1)).member(b"instructions")),
    ]) {
        assert_eq!(file.path.as_ref(), path);
        assert_eq!(file.at, at);
        assert!(!file.private);
        assert_eq!(
            u64::from(file.max),
            u64::from(engine.limits.core.run_bytes)
                .checked_sub(owned)
                .unwrap()
                .checked_add(u64::try_from(path.len()).unwrap())
                .unwrap()
        );
    }
}

fn malformed_parts_are_refused_at_their_paths() {
    let parts = part_path();
    let item = parts.child(crate::Segment::Index(0));
    for (value, path, what) in [
        (b"{}".as_slice(), parts, What::Kind),
        (b"[false]", item.clone(), What::Kind),
        (br#"[{"instructions":"a.md"}]"#, item.member(b"on"), What::Missing),
        (br#"[{"on":"first","instructions":"a.md"}]"#, item.member(b"on"), What::Kind),
        (br#"[{"on":[false],"instructions":"a.md"}]"#, item.member(b"on").child(crate::Segment::Index(0)), What::Kind),
        (br#"[{"on":["first"]}]"#, item.member(b"instructions"), What::Missing),
        (br#"[{"on":["first"],"instructions":false}]"#, item.member(b"instructions"), What::Kind),
        (br#"[{"on":["first"],"instructions":"../a.md"}]"#, item.member(b"instructions"), What::Relative),
        (br#"[{"on":["first"],"instructions":"a.md","extra":true}]"#, item.member(b"extra"), What::Unknown),
        (br#"[{"on":["first"],"on":["news"],"instructions":"a.md"}]"#, item.member(b"on"), What::Duplicate),
    ] {
        refuses_parts(value, path, what);
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
    let endpoint = br#""endpoints": [{"name":"second", "number":2, "dialect":3,
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
        ("a_part_refuses_an_unknown_reason", a_part_refuses_an_unknown_reason),
        ("a_part_refuses_always_offered_reasons", a_part_refuses_always_offered_reasons),
        ("a_part_refuses_a_repeated_reason", a_part_refuses_a_repeated_reason),
        ("a_part_refuses_no_reasons", a_part_refuses_no_reasons),
        ("a_charter_refuses_parts_past_its_limit", a_charter_refuses_parts_past_its_limit),
        ("a_charter_refuses_policy_and_parts_past_run_bytes", a_charter_refuses_policy_and_parts_past_run_bytes),
        ("a_charter_reads_two_parts_and_their_files_in_order", a_charter_reads_two_parts_and_their_files_in_order),
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
    skein_rows::check(cases, |case| case());
}

const COORDINATOR_WAKE: &[u8] = br#""wake": {"words":"immediate", "notices":"immediate",
    "news":{"paced":{"span_ms":1800000}}, "results":"each",
    "questions":true, "answers":true, "timers":true}, "alternatives": []"#;

#[test]
fn a_charter_names_its_default_wake_policy() {
    use jig_domain_tasks::{ResultsWake, WakePolicy, WakeRule};
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""alternatives": []"#, COORDINATOR_WAKE);
    let engine = fixtures::engine(&document).expect("coordinator wake policy");
    assert_eq!(
        engine.charters[0].wake,
        WakePolicy {
            news: WakeRule::Paced { span: skein_lib::Duration::from_millis(1_800_000) },
            results: ResultsWake::Each,
            ..WakePolicy::DEFAULT
        }
    );
}

#[test]
fn a_charter_without_a_wake_member_takes_the_default() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).expect("charter without wake");
    assert_eq!(engine.charters[0].wake, jig_domain_tasks::WakePolicy::DEFAULT);
}

#[test]
fn a_charter_wake_with_a_zero_span_is_refused_at_its_path() {
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""alternatives": []"#, COORDINATOR_WAKE);
    let document = fixtures::with(&document, b"1800000", b"0");
    let problems = fixtures::engine(&document).expect_err("zero span");
    assert_eq!(
        problems.problems.get(0),
        Some(&crate::Problem {
            path: fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"wake").member(b"news"),
            what: What::Bound,
        })
    );
}

#[test]
fn malformed_charter_wake_members_are_refused() {
    let document = fixtures::with(fixtures::ENGINE_SMALL, br#""alternatives": []"#, COORDINATOR_WAKE);
    for (from, to, member) in [
        (
            br#""news":{"paced":{"span_ms":1800000}}"#.as_slice(),
            br#""news":{"batch":{"count":0,"age_ms":1}}"#.as_slice(),
            b"news".as_slice(),
        ),
        (
            br#""news":{"paced":{"span_ms":1800000}}"#.as_slice(),
            br#""news":{"batch":{"count":1,"age_ms":0}}"#.as_slice(),
            b"news".as_slice(),
        ),
        (br#""news":{"paced":{"span_ms":1800000}}"#.as_slice(), br#""news":"unknown""#.as_slice(), b"news".as_slice()),
        (br#""results":"each""#.as_slice(), br#""results":"unknown""#.as_slice(), b"results".as_slice()),
        (br#""timers":true"#.as_slice(), br#""timers":true,"unknown":true"#.as_slice(), b"unknown".as_slice()),
    ] {
        let changed = fixtures::with(&document, from, to);
        let problems = fixtures::engine(&changed).expect_err("malformed wake policy");
        assert_eq!(
            problems.problems.get(0).expect("wake refusal").path,
            fixtures::path(&[b"charters"]).child(crate::Segment::Index(0)).member(b"wake").member(member)
        );
    }
}
