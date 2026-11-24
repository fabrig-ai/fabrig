//! Pass recording over the runner's tree and ledger (temper/docs/design/testing/qa/ledger.md, 4.9–4.11 and 5).
use super::{Reply, commands::home, put, run, success};
use serde_json::{Value, json};
use std::io;
use temper_qa_tool::{
    CaseInput, Exit, FindingInput, OutcomeResult, PassState,
    faces::{DirectoryMode, FilesFace},
    testing::{FileOp, World},
};

const BUILD: &str = "0123456789abcdef0123456789abcdef01234567";
const FIX: &str = "1123456789abcdef0123456789abcdef01234567";

fn pass_home() -> World {
    let world = home();
    world
        .home
        .transact(&mut world.files(), |ledger, _| {
            for (surface, page) in [("web", "work"), ("web", "home"), ("tui", "chat")] {
                let case =
                    json!({"surface":surface,"page":page,"setup":"fresh","steps":[{"do":"open","expect":"ready"}]});
                ledger.add_case(1, &CaseInput::decode(&serde_json::to_vec(&case).expect("JSON"), "-")?, 1)?;
            }
            ledger.end_session(1, Exit::Code(0), None, 2, "qa")?;
            ledger.keep_build(BUILD, 3)?;
            ledger.set_build_holds(BUILD, vec![1], 1)?;
            let pass =
                ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("pass");
            assert_eq!((pass.pass, pass.session), (1, 2));
            Ok(())
        })
        .expect("start pass");
    world
}
fn finding(case: u64) -> Value {
    json!({"case":case,"title":"Start does nothing","severity":"medium","setup":"fresh",
        "lines":["web press button Start"],"expected":"a goal starts","seen":"no goal"})
}
fn record(world: &World, value: &Value) -> Reply {
    run(world, &["pass", "finding", "-"], Some("2"), Some(&serde_json::to_vec(value).expect("JSON")))
}
fn refused(world: &World, arguments: &[&str], session: Option<&str>, stdin: Option<&[u8]>, words: &str) {
    let mut files = world.files();
    let home = &world.home;
    let before = files.read(&home.ledger).expect("before");
    let output = run(world, arguments, session, stdin);
    assert_eq!(output.code, 1, "{arguments:?}: {:?}", output.stderr);
    assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes(), "{arguments:?}");
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(files.read(&home.ledger).expect("after"), before);
}

#[test]
fn outcomes_append_notes_and_repeated_finding_options_and_refuse_without_writes() {
    let world = pass_home();
    success(&record(&world, &finding(1)), b"finding 1 recorded\n");
    success(&record(&world, &finding(1)), b"finding 2 linked to open finding 1 of pass 1\n");
    success(&record(&world, &finding(2)), b"finding 3 linked to open finding 1 of pass 1\n");
    for (arguments, answer) in [
        (vec!["pass", "outcome", "1", "passed", ""], "case 1 passed\n"),
        (vec!["pass", "outcome", "1", "blocked", "no connection"], "case 1 blocked\n"),
        (
            vec!["pass", "outcome", "1", "failed", "", "--finding", "1", "--finding", "2", "--finding", "1"],
            "case 1 failed\n",
        ),
    ] {
        success(&run(&world, &arguments, Some("2"), None), answer.as_bytes());
    }
    let stored = world.ledger();
    let outcomes = &stored.cases[0].outcomes;
    assert_eq!(outcomes.len(), 3);
    assert_eq!(
        outcomes.iter().map(|outcome| outcome.result).collect::<Vec<_>>(),
        [OutcomeResult::Passed, OutcomeResult::Blocked, OutcomeResult::Failed]
    );
    assert_eq!(outcomes[0].notes, "");
    assert_eq!(outcomes[1].notes, "no connection");
    assert_eq!(outcomes[2].findings, [1, 2, 1]);
    assert!(outcomes.iter().all(|outcome| outcome.session == 2 && outcome.time > 0));
    for (arguments, words) in [
        (vec!["pass", "outcome", "1", "maybe", ""], "outcome refused: maybe is none of passed, failed, blocked"),
        (vec!["pass", "outcome", "1", "blocked", "  "], "outcome refused: a blocked case says why"),
        (vec!["pass", "outcome", "1", "failed", ""], "outcome refused: a failed case names a finding of its own"),
        (vec!["pass", "outcome", "1", "failed", "", "--finding", "3"], "outcome refused: finding 3 is not case 1's"),
        (vec!["pass", "outcome", "3", "passed", ""], "case 3 is not assigned to pass 1"),
    ] {
        refused(&world, &arguments, Some("2"), None, words);
    }
}

#[test]
fn every_recording_command_checks_the_pass_session_before_reading_inputs() {
    let world = pass_home();
    let commands = [
        vec!["pass", "outcome", "1", "passed", ""],
        vec!["pass", "finding", "missing.json"],
        vec!["pass", "check", "1", "holds", ""],
        vec!["pass", "note", "noticed"],
    ];
    for arguments in &commands {
        for (session, words) in [
            (None, "TEMPER_INTERNAL_QA_SESSION is not set: pass commands run in a pass session"),
            (Some("999"), "session 999 is not running"),
            (Some("1"), "session 1 is not running"),
        ] {
            refused(&world, arguments, session, None, words);
        }
    }
    world
        .home
        .transact(&mut world.files(), |ledger, _| {
            ledger.record_landing(FIX, 11, None, "More work", 5);
            assert_eq!(
                ledger.start_session(
                    2,
                    "low",
                    [("web".into(), vec!["work".into()])].into(),
                    vec!["fresh".into()],
                    Vec::new(),
                    5
                )?,
                3
            );
            Ok(())
        })
        .expect("running triage");
    for arguments in &commands {
        refused(&world, arguments, Some("3"), None, "session 3 is a triage session");
    }
    world
        .home
        .transact(&mut world.files(), |ledger, _| {
            // A stale running session must not record against a non-running pass.
            ledger.passes[0].state = PassState::Waiting;
            Ok(())
        })
        .expect("stale pass");
    for arguments in &commands {
        refused(&world, arguments, Some("2"), None, "pass 1 is waiting");
    }
}

#[test]
fn findings_from_file_and_stdin_copy_evidence_inherit_the_case_and_choose_area() {
    let world = pass_home();
    let home = &world.home;
    let mut files = world.files();
    let configuration = home.directory.join("qa.json");
    let mut config: Value = serde_json::from_slice(&files.read(&configuration).expect("configuration")).expect("JSON");
    config["areas"] = json!({"web":"frontend"});
    put(&mut files, &configuration, serde_json::to_vec(&config).expect("JSON"));
    files.directory(&home.directory.join("passes/1/setups/project"), DirectoryMode::Parents).expect("recipe");
    put(&mut files, &home.directory.join("look.png"), b"pixels");
    let mut value = finding(1);
    value["setup"] = json!("project");
    value["evidence"] = json!(["look.png"]);
    let input = FindingInput::decode(&serde_json::to_vec(&value).expect("JSON"), "fixture").expect("finding");
    home.transact(&mut files, |ledger, _| ledger.record_finding(2, &input, &["project".into()], "frontend", 5))
        .expect("open finding for the stdin row to match");

    success(&record(&world, &value), b"finding 2 linked to open finding 1 of pass 1\n");
    assert_eq!(files.read(&home.directory.join("evidence/2/look.png")).expect("linked kept"), b"pixels");
    let stored = world.ledger();
    let linked = &stored.findings[1];
    assert_eq!(linked.to, Some(1));
    assert_eq!((linked.pass, linked.case, linked.session), (1, 1, 2));
    assert_eq!((linked.surface.as_str(), linked.page.as_str(), linked.landings.as_slice()), ("web", "work", &[1][..]));
    assert_eq!(
        (linked.area.as_str(), linked.setup.as_str(), linked.evidence.as_slice()),
        ("frontend", "project", &["look.png".to_owned()][..])
    );
    assert_eq!(
        (linked.title.as_str(), linked.severity.as_str(), linked.expected.as_str(), linked.seen.as_str()),
        ("Start does nothing", "medium", "a goal starts", "no goal")
    );
    assert_eq!(linked.lines, ["web press button Start"]);

    config["areas"] = json!({});
    put(&mut files, &configuration, serde_json::to_vec(&config).expect("JSON"));
    success(&record(&world, &finding(1)), b"finding 3 linked to open finding 1 of pass 1\n");
    let mut explicit = finding(1);
    explicit["area"] = json!("other");
    success(&record(&world, &explicit), b"finding 4 linked to open finding 1 of pass 1\n");
    let stored = world.ledger();
    assert_eq!(stored.findings[2].area, "qa");
    assert_eq!(stored.findings[3].area, "other");
}

#[test]
fn finding_field_and_file_refusals_leave_no_record_or_evidence() {
    let world = pass_home();
    let home = &world.home;
    let mut files = world.files();
    put(&mut files, &home.directory.join("look.png"), b"pixels");
    let mut good = finding(1);
    good["evidence"] = json!(["look.png"]);
    for field in ["case", "title", "severity", "setup", "lines", "expected", "seen"] {
        for remove in [true, false] {
            let mut value = good.clone();
            if remove {
                value.as_object_mut().expect("object").remove(field);
            } else {
                value[field] = if field == "case" {
                    json!(0)
                } else if field == "lines" {
                    json!([])
                } else {
                    json!("")
                };
            }
            refused(
                &world,
                &["pass", "finding", "-"],
                Some("2"),
                Some(&serde_json::to_vec(&value).expect("JSON")),
                &format!("finding refused: no {field}"),
            );
        }
    }
    for (field, value, words) in [
        ("surprise", json!(true), "finding refused: unknown field surprise"),
        ("severity", json!("urgent"), "finding refused: severity urgent is none of low, medium, high"),
        ("setup", json!("missing"), "finding refused: setup missing is not in pass 1's setups; setups: "),
        ("lines", json!([""]), "finding refused: line 1 is empty or spans lines"),
        ("lines", json!(["open", "web press\nStart"]), "finding refused: line 2 is empty or spans lines"),
        ("lines", json!(["web press\rStart"]), "finding refused: line 1 is empty or spans lines"),
        ("case", json!(3), "case 3 is not assigned to pass 1"),
    ] {
        let mut value_object = good.clone();
        value_object[field] = value;
        refused(
            &world,
            &["pass", "finding", "-"],
            Some("2"),
            Some(&serde_json::to_vec(&value_object).expect("JSON")),
            words,
        );
        assert!(!files.exists(home.directory.join("evidence/1")));
    }
    let before = files.read(&home.ledger).expect("ledger");
    let output = run(&world, &["pass", "finding", "missing.json"], Some("2"), None);
    assert_eq!(output.code, 1);
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: missing.json: "));
    let output = run(&world, &["pass", "finding", "-"], Some("2"), Some(b"{"));
    assert_eq!(output.code, 1);
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: -: "));
    assert_eq!(files.read(&home.ledger).expect("unchanged"), before);
    assert_eq!(world.ledger().findings.len(), 0);
    assert!(!files.exists(home.directory.join("evidence/1")));
}

#[test]
fn evidence_limits_refuse_and_failed_ledger_writes_remove_the_copied_directory() {
    const LIMIT: usize = 16 * 1024 * 1024;
    let world = pass_home();
    let home = &world.home;
    let mut files = world.files();
    files.directory(&home.directory.join("other"), DirectoryMode::One).expect("other directory");
    put(&mut files, &home.directory.join("look.png"), b"pixels");
    put(&mut files, &home.directory.join("other/look.png"), b"other");
    let large = home.directory.join("large");
    let bytes = vec![0; LIMIT + 1];
    put(&mut files, &large, &bytes);
    let mut value = finding(1);
    for (paths, words) in [
        (
            json!(["look.png", "other/look.png"]),
            "finding refused: evidence other/look.png: duplicate file name look.png",
        ),
        (json!(["large"]), "finding refused: evidence large: file exceeds 16 MiB"),
        (json!(["other"]), "finding refused: evidence other: not a regular file"),
    ] {
        value["evidence"] = paths;
        refused(&world, &["pass", "finding", "-"], Some("2"), Some(&serde_json::to_vec(&value).expect("JSON")), words);
        assert!(!files.exists(home.directory.join("evidence/1")));
    }
    let before = files.read(&home.ledger).expect("before");
    value["evidence"] = json!(["look.png", "missing"]);
    let output = record(&world, &value);
    assert_eq!(output.code, 1);
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: evidence missing: "));
    assert_eq!(files.read(&home.ledger).expect("unchanged"), before);
    assert!(!files.exists(home.directory.join("evidence/1")));
    let mut paths = Vec::new();
    for number in 0..17 {
        let name = format!("evidence-{number}");
        put(&mut files, &home.directory.join(&name), []);
        paths.push(name);
    }
    value["evidence"] = json!(paths);
    refused(
        &world,
        &["pass", "finding", "-"],
        Some("2"),
        Some(&serde_json::to_vec(&value).expect("JSON")),
        "finding refused: evidence evidence-16: more than 16 files",
    );
    assert!(!files.exists(home.directory.join("evidence/1")));
    paths.pop();
    value["evidence"] = json!(paths);
    success(&record(&world, &value), b"finding 1 recorded\n");
    assert_eq!(files.list(&home.directory.join("evidence/1")).expect("16 files").len(), 16);
    put(&mut files, &large, &bytes[..LIMIT]);
    value["evidence"] = json!(["large"]);
    success(&record(&world, &value), b"finding 2 linked to open finding 1 of pass 1\n");
    assert_eq!(files.facts(&home.directory.join("evidence/2/large"), true).expect("kept").size, LIMIT as u64);
    // Fail only the transaction's save, after it has copied the evidence.
    files.inject(home.directory.join("ledger.tmp"), FileOp::Write, io::Error::other("save refused"));
    let before = files.read(&home.ledger).expect("before failed write");
    value["evidence"] = json!(["look.png"]);
    let output = record(&world, &value);
    assert_eq!(output.code, 1);
    assert_eq!(output.stderr, b"temper-qa: ledger.json: save refused\n");
    assert_eq!(files.read(&home.ledger).expect("unchanged"), before);
    assert!(!files.exists(home.directory.join("evidence/3")));
}

#[test]
fn checks_require_a_resumed_session_and_an_open_finding_and_print_the_build() {
    let world = pass_home();
    success(&record(&world, &finding(1)), b"finding 1 recorded\n");
    success(&record(&world, &finding(1)), b"finding 2 linked to open finding 1 of pass 1\n");
    refused(
        &world,
        &["pass", "check", "1", "holds", ""],
        Some("2"),
        None,
        "check refused: pass 1's first session has no finding to check",
    );
    world
        .home
        .transact(&mut world.files(), |ledger, _| {
            ledger.record_outcome(2, 1, "failed", "", &[1], 5)?;
            ledger.record_outcome(2, 2, "passed", "", &[], 5)?;
            ledger.end_pass_session(2, Exit::Code(0), None, 6, "qa", &[])?;
            ledger.file_request(1, 1, Some(30), 7)?;
            ledger.record_landing(FIX, 30, None, "Fix", 8);
            let triage = ledger.start_session(
                2,
                "low",
                [("web".into(), vec!["work".into()])].into(),
                vec!["fresh".into()],
                Vec::new(),
                9,
            )?;
            ledger.answer_none(triage, "fix covered by pass")?;
            ledger.end_session(triage, Exit::Code(0), None, 10, "qa")?;
            ledger.keep_build(FIX, 11)?;
            ledger.set_build_holds(FIX, vec![1, 2], 2)?;
            assert_eq!(
                ledger
                    .resume_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 12)
                    .expect("resume")
                    .session,
                4
            );
            Ok(())
        })
        .expect("resume through ledger");
    for (arguments, words) in [
        (vec!["pass", "check", "1", "maybe", ""], "check refused: maybe is neither holds nor gone"),
        (vec!["pass", "check", "2", "gone", ""], "check refused: finding 2 is not open in pass 1"),
        (vec!["pass", "check", "999", "gone", ""], "check refused: finding 999 is not open in pass 1"),
    ] {
        refused(&world, &arguments, Some("4"), None, words);
    }
    success(&run(&world, &["pass", "check", "1", "holds", "still happens"], Some("4"), None), b"finding 1 holds\n");
    success(
        &run(&world, &["pass", "check", "1", "gone", "fixed"], Some("4"), None),
        b"finding 1 gone at 1123456789ab\n",
    );
    let stored = world.ledger();
    let checks = &stored.findings[0].checks;
    assert_eq!(checks.len(), 2);
    assert!(checks[0].holds);
    assert!(!checks[1].holds);
    assert_eq!((&checks[0].notes, &checks[1].notes), (&"still happens".to_owned(), &"fixed".to_owned()));
    assert!(checks.iter().all(|check| check.session == 4 && check.commit == FIX && check.time > 0));
}

#[test]
fn notes_record_words_and_refuse_empty_words() {
    let world = pass_home();
    success(&run(&world, &["pass", "note", "off path"], Some("2"), None), b"note 1\n");
    let stored = world.ledger();
    assert_eq!(stored.notes.len(), 1);
    assert_eq!((stored.notes[0].number, stored.notes[0].pass, stored.notes[0].session), (1, 1, 2));
    assert_eq!(stored.notes[0].words, "off path");
    assert!(stored.notes[0].time > 0);
    refused(&world, &["pass", "note", ""], Some("2"), None, "note refused: no words");
    refused(&world, &["pass", "note", "  "], Some("2"), None, "note refused: no words");
}
