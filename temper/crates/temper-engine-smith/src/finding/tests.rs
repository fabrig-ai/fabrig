//! The finding vocabulary, its refusal feedback and whole-line answer bound.
use super::{Problem, answer, decided, engine};
use engine::{FindingCall as Call, FindingToolAnswer as Answer};
use skein_lib::Wall;
use temper_engine_domain_forge as f;

#[test]
fn the_finding_declaration_preserves_the_stock_brief_allowance() {
    let spec = crate::finding::SPEC;
    let bytes = spec.name.len() + spec.description.len() + spec.schema.len();
    assert!(bytes <= 678, "finding declaration: {bytes} bytes, at most 678 for the stock brief");
}

#[test]
fn every_finding_act_decodes_its_minimum_shape_and_names_a_wrong_member() {
    for (input, expected) in [
        (
            r#"{"act":"file","kind":"red","severity":"high","title":"boot flake","body":"..."}"#,
            Call::File {
                kind: f::FindingKind::Red,
                severity: f::Severity::High,
                area: None,
                title: b"boot flake".as_slice().into(),
                body: b"...".as_slice().into(),
            },
        ),
        (
            r#"{"act":"hand","number":12,"area":"tests","why":"flake"}"#,
            Call::Hand { number: 12, area: b"tests".as_slice().into(), why: b"flake".as_slice().into() },
        ),
        (r#"{"act":"route","number":12,"goal":40}"#, Call::Triage { number: 12, act: f::Triage::Route(40) }),
        (r#"{"act":"backlog","number":12}"#, Call::Triage { number: 12, act: f::Triage::Backlog }),
        (
            r#"{"act":"drop","number":12,"reason":"duplicate"}"#,
            Call::Triage { number: 12, act: f::Triage::Drop(b"duplicate".as_slice().into()) },
        ),
        (r#"{"act":"list"}"#, Call::List { state: None, mine: false }),
        (r#"{"act":"show","number":12}"#, Call::Show { number: 12 }),
        (r#"{"act":"list","state":"new","mine":true}"#, Call::List { state: Some(f::FindingState::New), mine: true }),
    ] {
        assert_eq!(decided(b"finding", input.as_bytes()).unwrap().tool, engine::Tool::Finding(expected));
    }
    for (input, expected) in [
        (r"{}", Problem::Missing(b"act")),
        (r#"{"act":"file"}"#, Problem::Missing(b"kind")),
        (r#"{"act":"hand","number":12,"area":"tests"}"#, Problem::Missing(b"why")),
        (r#"{"act":"route","number":12}"#, Problem::Missing(b"goal")),
        (r#"{"act":"backlog","number":0}"#, Problem::Range(b"number")),
        (r#"{"act":"drop","number":12,"reason":false}"#, Problem::Type(b"reason")),
        (r#"{"act":"list","mine":1}"#, Problem::Type(b"mine")),
        (r#"{"act":"show","number":4294967296}"#, Problem::Range(b"number")),
        (r#"{"act":"show","number":12,"project":3}"#, Problem::Range(b"finding: member does not belong to this act")),
    ] {
        assert_eq!(decided(b"finding", input.as_bytes()), Err(expected), "{input}");
    }
    for (field, size, bound) in [("title", 200, 200), ("body", 4096, 4096), ("title", 201, 200), ("body", 4097, 4096)] {
        let title = if field == "title" { "x".repeat(size) } else { "title".into() };
        let body = if field == "body" { "x".repeat(size) } else { "body".into() };
        let input = format!(r#"{{"act":"file","kind":"qa","severity":"low","title":"{title}","body":"{body}"}}"#);
        let decoded = decided(b"finding", input.as_bytes());
        if size <= bound {
            decoded.expect("text at its byte bound decodes");
        } else {
            assert_eq!(
                decoded,
                Err(Problem::Range(if field == "title" {
                    b"title: 1 to 200 UTF-8 bytes, no control characters"
                } else {
                    b"body: at most 4,096 UTF-8 bytes"
                }))
            );
        }
    }
    let schema = crate::json::parse(crate::finding::SPEC.schema).unwrap();
    let properties = schema.required(b"properties").unwrap();
    assert_eq!(properties.required(b"act").unwrap().required(b"enum").unwrap().array(b"enum").unwrap().len(), 7);
    for field in [
        b"kind".as_slice(),
        b"severity",
        b"area",
        b"title",
        b"body",
        b"number",
        b"why",
        b"goal",
        b"reason",
        b"state",
        b"mine",
    ] {
        assert!(properties.field(field).unwrap().is_some());
    }
}
fn write(result: Result<f::FindingDone, f::FindingRefusal>) -> engine::CallAnswer {
    engine::CallAnswer::Finding(Box::new(Answer::Write(f::FindingReply {
        project: 3,
        number: 12,
        area: Some(b"tests".as_slice().into()),
        result,
    })))
}
fn row(number: u32) -> f::FindingRow {
    f::FindingRow {
        number,
        kind: f::FindingKind::Red,
        severity: f::Severity::High,
        area: Some(b"tests".as_slice().into()),
        state: f::FindingState::New,
        title: "boot flaké".as_bytes().into(),
        holder: f::Holder::Task(40),
        at: Wall::from_nanos(19),
        why: Some(b"flake".as_slice().into()),
    }
}
#[test]
fn finding_answers_use_the_tools_words_and_mark_refusals_as_errors() {
    for (done, expected) in [
        (f::FindingDone::Filed(f::Holder::Task(40)), "Filed finding 3/12; it is task 40's."),
        (f::FindingDone::Handed(f::Holder::Party(7)), "Handed finding 3/12 to area tests; it is party 7's."),
        (f::FindingDone::Handed(f::Holder::Role(0)), "Handed finding 3/12 to area tests; it is role 0's."),
        (f::FindingDone::Routed(40), "Routed finding 3/12 to task 40."),
        (f::FindingDone::Backlog, "Finding 3/12 is in the backlog."),
        (f::FindingDone::Dropped, "Dropped finding 3/12."),
    ] {
        let rendered = answer(&write(Ok(done)));
        assert!(!rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
    }
    for (why, expected) in [
        (f::FindingRefusal::NotYours(f::Holder::Task(40)), "finding 3/12 is not yours: its owner is task 40"),
        (
            f::FindingRefusal::NoArea(Box::new([b"ui".as_slice().into(), b"docs".as_slice().into()])),
            "no area tests in project 3; its areas: ui, docs",
        ),
        (f::FindingRefusal::Full(256), "the project's findings are full: 256 new"),
    ] {
        let rendered = answer(&write(Err(why)));
        assert!(rendered.error());
        assert_eq!(rendered.text(), expected.as_bytes());
    }
    let shown = answer(&engine::CallAnswer::Finding(Box::new(Answer::Show {
        project: 3,
        number: 12,
        finding: Some(f::FindingAnswer {
            row: row(12),
            body: "reproduce é".as_bytes().into(),
            filer: f::Filer::Task(2),
            goal: Some(99),
            reason: Some(b"duplicate".as_slice().into()),
        }),
    })));
    assert!(!shown.error());
    assert_eq!(shown.text(), "3/12 new red high tests task 40: boot flaké\nfiler: task 2\nat: 19\nwhy: flake\ngoal: task 99\nreason: duplicate\n\nreproduce é".as_bytes());
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test checks that bounded replies preserve UTF-8")]
fn finding_list_fits_whole_lines_and_counts_every_omitted_entry() {
    let doc = f::Findings {
        project: 3,
        rows: Box::new([row(12), row(11), row(10)]),
        areas: Box::new([f::AreaRow {
            name: b"tests".as_slice().into(),
            words: b"Suite".as_slice().into(),
            owner: f::settings::AreaOwner::Charter(13),
            holder: Some(f::Holder::Task(40)),
        }]),
        triager: Some(f::settings::AreaOwner::Task(40)),
        holder: f::Holder::Task(40),
        cut: false,
    };
    let answer = engine::CallAnswer::Finding(Box::new(Answer::List(doc)));
    let first = "tests (charter 13, task 40): Suite\ntriager (task 40, task 40)\n3/12 new red high tests task 40: boot flaké\nand 2 more";
    let rendered = crate::answer_bounded(&answer, u32::try_from(first.len()).unwrap());
    assert!(!rendered.error());
    assert_eq!(rendered.text(), first.as_bytes());
    let whole = crate::answer(&answer);
    assert!(core::str::from_utf8(whole.text()).unwrap().ends_with("3/10 new red high tests task 40: boot flaké"));
    for bound in 0..=whole.text().len() {
        let rendered = crate::answer_bounded(&answer, u32::try_from(bound).unwrap());
        assert!(rendered.text().len() <= bound);
        core::str::from_utf8(rendered.text()).expect("bounded answer is UTF-8");
        assert!(!rendered.error());
    }
}
