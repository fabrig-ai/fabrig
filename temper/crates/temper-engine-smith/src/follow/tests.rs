//! Follow calls, binding line forms and bounded reads beside the finding tests.
use super::{Problem, answer, decided, engine};
use engine::{FollowCall as Call, FollowToolAnswer as Answer};
use temper_engine_domain_forge::{self as f, follows::FollowAct, settings as s};

const A: &str = "a1b2a1b2a1b2a1b2a1b2a1b2a1b2a1b2a1b2a1b2";
const B: &str = "c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4c3d4";
fn commit(text: &str) -> skein_forge::api::Commit {
    crate::forge::commit(&crate::json::Value::String(text.as_bytes().into()), b"commit").unwrap()
}
#[test]
fn follow_calls_validate_members_before_deciding() {
    let a = commit(A);
    let b = commit(B);
    for (input, expected) in [
        (
            format!(r#"{{"act":"start","follow":1,"commit":"{A}"}}"#),
            Call::Act { follow: 1, act: FollowAct::Start { after: a } },
        ),
        (
            format!(r#"{{"act":"hold","follow":1,"commit":"{A}","reason":"wait"}}"#),
            Call::Act { follow: 1, act: FollowAct::Hold { after: a, reason: b"wait".as_slice().into() } },
        ),
        (
            format!(r#"{{"act":"skip","follow":1,"commit":"{A}","reason":"ported"}}"#),
            Call::Act { follow: 1, act: FollowAct::Skip { after: a, reason: b"ported".as_slice().into() } },
        ),
        (
            format!(r#"{{"act":"merge","follow":1,"commits":["{A}","{B}"]}}"#),
            Call::Act { follow: 1, act: FollowAct::Merge { afters: Box::new([a, b]) } },
        ),
        (r#"{"act":"list"}"#.into(), Call::List),
        (r#"{"act":"show","follow":1}"#.into(), Call::Show { follow: 1 }),
    ] {
        assert_eq!(decided(b"follow", input.as_bytes()).unwrap().tool, engine::Tool::Follow(expected));
    }
    for (input, expected) in [
        (r"{}".into(), Problem::Missing(b"act")),
        (r#"{"act":0}"#.into(), Problem::Type(b"act")),
        (r#"{"act":"unknown"}"#.into(), Problem::Range(b"act: start, hold, skip, merge, list or show")),
        (r#"{"act":"start"}"#.into(), Problem::Missing(b"follow")),
        (r#"{"act":"start","follow":1}"#.into(), Problem::Missing(b"commit")),
        (r#"{"act":"show","follow":0}"#.into(), Problem::Range(b"follow")),
        (r#"{"act":"show","follow":4294967296}"#.into(), Problem::Range(b"follow")),
        (r#"{"act":"show","follow":"1"}"#.into(), Problem::Type(b"follow")),
        (r#"{"act":"list","follow":1}"#.into(), Problem::Range(b"follow: member does not belong to this act")),
        (r#"{"act":"list","project":2}"#.into(), Problem::Range(b"follow: member does not belong to this act")),
        (r#"{"act":"merge","follow":1}"#.into(), Problem::Missing(b"commits")),
        (format!(r#"{{"act":"merge","follow":1,"commits":["{A}"]}}"#), Problem::Range(b"commits: 2 to 32")),
        (r#"{"act":"merge","follow":1,"commits":false}"#.into(), Problem::Type(b"commits")),
        (format!(r#"{{"act":"hold","follow":1,"commit":"{A}"}}"#), Problem::Missing(b"reason")),
        (
            format!(r#"{{"act":"skip","follow":1,"commit":"{A}","reason":""}}"#),
            Problem::Range(b"reason: 1 to 8192 UTF-8 bytes"),
        ),
        (format!(r#"{{"act":"skip","follow":1,"commit":"{A}","reason":false}}"#), Problem::Type(b"reason")),
    ] {
        assert_eq!(decided(b"follow", input.as_bytes()), Err(expected), "{input}");
    }
    for text in ["a".repeat(39), "a".repeat(41), "a".repeat(63), "a".repeat(65), "A".repeat(40), "g".repeat(64)] {
        let input = format!(r#"{{"act":"start","follow":1,"commit":"{text}"}}"#);
        assert_eq!(decided(b"follow", input.as_bytes()), Err(Problem::Range(b"full commit identifier")));
    }
    for (count, accepted) in [(2, true), (32, true), (33, false)] {
        let commits = format!("\"{A}\",").repeat(count);
        let commits = commits.strip_suffix(',').expect("trailing separator");
        let input = format!(r#"{{"act":"merge","follow":1,"commits":[{commits}]}}"#);
        assert_eq!(decided(b"follow", input.as_bytes()).is_ok(), accepted);
    }
    for (size, accepted) in [(8192, true), (8193, false)] {
        let reason = "x".repeat(size);
        let input = format!(r#"{{"act":"hold","follow":1,"commit":"{A}","reason":"{reason}"}}"#);
        assert_eq!(decided(b"follow", input.as_bytes()).is_ok(), accepted);
    }
    let schema = crate::json::parse(crate::follow::SPEC.schema).unwrap();
    schema_members(&schema);
    let properties = schema.required(b"properties").unwrap();
    assert_eq!(properties.required(b"commits").unwrap().required(b"minItems").unwrap().number(b"minItems").unwrap(), 2);
    assert!(schema.field(b"oneOf").unwrap().is_none());
    assert_eq!(
        properties.required(b"reason").unwrap().required(b"minLength").unwrap().number(b"minLength").unwrap(),
        1
    );
    for field in [b"commit".as_slice(), b"commits"] {
        let property = properties.required(field).unwrap();
        let commit = if field == b"commits" { property.required(b"items").unwrap() } else { property };
        assert_eq!(
            commit.required(b"pattern").unwrap().text(b"pattern").unwrap().as_ref(),
            b"^([0-9a-f]{40}|[0-9a-f]{64})$"
        );
    }
    let delegate = jig_domain_core::ToolFamilies::JIG.delegate.0;
    for (charter, authority, offered) in [(delegate, delegate, true), (delegate, 0, true), (0, delegate, false)] {
        let tools = crate::calls::tools_for(
            skein_lib::Duration::from_secs(1),
            jig_domain_tasks::Tools(charter),
            jig_domain_tasks::Tools(authority),
        );
        let mut found = false;
        for tool in tools {
            if tool.name.as_ref() == b"follow" {
                found = true;
            }
        }
        assert_eq!(found, offered);
    }
}
fn schema_members(schema: &crate::json::Value) {
    use crate::json::Value;
    assert_eq!(schema.required(b"additionalProperties").unwrap(), &Value::Boolean(false));
    let acts = schema
        .required(b"properties")
        .unwrap()
        .required(b"act")
        .unwrap()
        .required(b"enum")
        .unwrap()
        .array(b"enum")
        .unwrap();
    assert_eq!(acts.len(), 6);
    for (act, expected) in [
        (b"start".as_slice(), [b"act".as_slice(), b"follow", b"commit"].as_slice()),
        (b"hold", [b"act".as_slice(), b"follow", b"commit", b"reason"].as_slice()),
        (b"skip", [b"act".as_slice(), b"follow", b"commit", b"reason"].as_slice()),
        (b"merge", [b"act".as_slice(), b"follow", b"commits"].as_slice()),
        (b"list", [b"act".as_slice()].as_slice()),
        (b"show", [b"act".as_slice(), b"follow"].as_slice()),
    ] {
        assert!(acts.contains(&Value::String(act.into())));
        let mut required = skein_lib::List::uncapped();
        schema_required(schema, act, &mut required);
        assert_eq!(skein_lib::usize_of(required.len()), expected.len());
        for name in expected {
            assert!(required.as_slice().contains(&Box::<[u8]>::from(*name)));
        }
    }
}
fn schema_required(schema: &crate::json::Value, act: &[u8], output: &mut skein_lib::List<Box<[u8]>>) {
    if let Some(required) = schema.field(b"required").unwrap() {
        for name in required.array(b"required").unwrap() {
            output.push(name.text(b"required").unwrap()).unwrap();
        }
    }
    if let Some(condition) = schema.field(b"if").unwrap() {
        let acts = condition
            .required(b"properties")
            .unwrap()
            .required(b"act")
            .unwrap()
            .required(b"enum")
            .unwrap()
            .array(b"enum")
            .unwrap();
        let branch: &[u8] = if acts.contains(&crate::json::Value::String(act.into())) { b"then" } else { b"else" };
        if let Some(branch) = schema.field(branch).unwrap() {
            schema_required(branch, act, output);
        }
    }
}
fn row(number: u32) -> f::FollowView {
    f::FollowView {
        number,
        setting: s::Follow {
            source: s::FollowSource {
                repository: skein_forge::api::Repository { forge: 1, repository: 12 },
                branch: b"main".as_slice().into(),
                paths: Box::new([b"src".as_slice().into()]),
            },
            target: s::FollowTarget {
                repository: skein_forge::api::Repository { forge: 1, repository: 13 },
                branch: b"port".as_slice().into(),
            },
            owner: s::FollowOwner::Task(52),
            start: s::FollowStart::Owner,
            producer: None,
            check: s::FollowCheck {
                workflow: b"test.yml".as_slice().into(),
                artifact: b"results".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Lines,
            },
        },
        source: b"org/vsr".as_slice().into(),
        source_branch: b"main".as_slice().into(),
        target: b"org/vsr-rs".as_slice().into(),
        target_branch: b"port".as_slice().into(),
        task: Some(53),
        stopped: None,
        lag: 1,
        entries: Box::new([f::FollowLagEntry {
            after: commit(A),
            change: Some(54),
            state: f::FollowState::HeldByOwner(b"waits for the handoff fix".as_slice().into()),
        }]),
        skips: Box::new([f::FollowSkip { after: commit(B), reason: b"build only".as_slice().into() }]),
        parity: None,
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test checks that bounded replies preserve UTF-8")]
fn follow_reads_match_ctl_and_fit_whole_lines() {
    let empty = engine::CallAnswer::Follow(Box::new(Answer::List(f::FollowsView {
        project: 3,
        follows: Box::new([]),
        dependencies: Box::new([]),
    })));
    assert_eq!(answer(&empty).text(), b"no follows");
    let expected = "follow 1: org/vsr main → org/vsr-rs port · 1 behind · no check yet";
    let listed = engine::CallAnswer::Follow(Box::new(Answer::List(f::FollowsView {
        project: 3,
        follows: Box::new([row(1), row(2), row(3)]),
        dependencies: Box::new([]),
    })));
    let cut = format!("{expected}\nand 2 more");
    assert_eq!(crate::answer_bounded(&listed, u32::try_from(cut.len()).unwrap()).text(), cut.as_bytes());
    let shown = engine::CallAnswer::Follow(Box::new(Answer::Show { follow: 1, view: Some(Box::new(row(1))) }));
    assert_eq!(answer(&shown).text(), format!("{expected}\nsource: org/vsr (1/12)\nbranch: main\npaths: src\ntarget: org/vsr-rs (1/13)\nbranch: port\nowner: task 52\nstart: owner\nproducer: default\ncheck: test.yml · artifact results · lines\ntask: 53\nlag: 1\n  a1b2a1b2a1b2 · change 54 · held by its owner: waits for the handoff fix\nskips:\n  c3d4c3d4c3d4 · build only\nparity: no check yet\n").as_bytes());
    for bound in 0..=answer(&shown).text().len() {
        let rendered = crate::answer_bounded(&shown, u32::try_from(bound).unwrap());
        assert!(rendered.text().len() <= bound);
        core::str::from_utf8(rendered.text()).expect("bounded UTF-8");
        assert!(!rendered.error());
    }
    let mut caught = row(1);
    caught.lag = 0;
    caught.parity = Some(f::FollowParity {
        source: commit(A),
        head: commit(B),
        verdict: f::FollowVerdict::Passed,
        run: b"https://forge.example/run".as_slice().into(),
    });
    let caught = engine::CallAnswer::Follow(Box::new(Answer::List(f::FollowsView {
        project: 3,
        follows: Box::new([caught]),
        dependencies: Box::new([]),
    })));
    assert_eq!(
        answer(&caught).text(),
        "follow 1: org/vsr main → org/vsr-rs port · caught up · check passes at a1b2a1b2a1b2".as_bytes()
    );
}
#[test]
fn follow_writes_render_kept_and_error_words() {
    let kept = answer(&engine::CallAnswer::Follow(Box::new(Answer::Write { follow: 1, refusal: None })));
    assert_eq!(kept.text(), b"Kept: follow 1's next step applies it.");
    assert!(!kept.error());
    let why = b"follow 1 is not yours: its owner is task 52";
    let refusal = answer(&engine::CallAnswer::Follow(Box::new(Answer::Write {
        follow: 1,
        refusal: Some(why.as_slice().into()),
    })));
    assert_eq!(refusal.text(), why);
    assert!(refusal.error());
    let unknown = answer(&engine::CallAnswer::Follow(Box::new(Answer::Show { follow: 1, view: None })));
    assert_eq!(unknown.text(), b"no such follow");
    assert!(unknown.error());
}
