//! Pass recording through the binary, with ledger-created sessions (qa/ledger.md, 4.9–4.11 and 5).
use super::commands::{home, run, success};
use serde_json::{Value, json};
use skein_scratch::Scratch;
use std::{fs, process::Output};
use temper_qa_tool::{CaseInput, Exit, Home, Ledger, OutcomeResult, PassState};

const BUILD: &str = "0123456789abcdef0123456789abcdef01234567";
const FIX: &str = "1123456789abcdef0123456789abcdef01234567";

fn pass_home() -> (Scratch, Home) {
    let (scratch, home) = home();
    home.transact(|ledger| {
        for (surface, page) in [("web", "work"), ("web", "home"), ("tui", "chat")] {
            let case = json!({"surface":surface,"page":page,"setup":"fresh","steps":[{"do":"open","expect":"ready"}]});
            ledger.add_case(1, &CaseInput::decode(&serde_json::to_vec(&case).expect("JSON"), "-")?, 1)?;
        }
        ledger.end_session(1, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build(BUILD, 3)?;
        ledger.set_build_holds(BUILD, vec![1], 1)?;
        let pass = ledger.start_pass("places/1", &["web".into()], "high", 4).expect("pass");
        assert_eq!((pass.pass, pass.session), (1, 2));
        Ok(())
    })
    .expect("start pass");
    (scratch, home)
}
fn finding(case: u64) -> Value {
    json!({"case":case,"title":"Start does nothing","severity":"medium","setup":"fresh",
        "lines":["web press button Start"],"expected":"a goal starts","seen":"no goal"})
}
fn record(home: &Home, value: &Value) -> Output {
    run(home, &["pass", "finding", "-"], Some("2"), Some(&serde_json::to_vec(value).expect("JSON")))
}
fn ledger(home: &Home) -> Ledger {
    Ledger::decode(&fs::read(&home.ledger).expect("ledger")).expect("decode")
}
fn refused(home: &Home, arguments: &[&str], session: Option<&str>, stdin: Option<&[u8]>, words: &str) {
    let before = fs::read(&home.ledger).expect("before");
    let output = run(home, arguments, session, stdin);
    assert_eq!(output.status.code(), Some(1), "{arguments:?}: {:?}", output.stderr);
    assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes(), "{arguments:?}");
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(fs::read(&home.ledger).expect("after"), before);
}

#[test]
fn outcomes_append_notes_and_repeated_finding_options_and_refuse_without_writes() {
    let (_scratch, home) = pass_home();
    success(&record(&home, &finding(1)), b"finding 1 recorded\n");
    success(&record(&home, &finding(1)), b"finding 2 linked to open finding 1 of pass 1\n");
    success(&record(&home, &finding(2)), b"finding 3 linked to open finding 1 of pass 1\n");
    for (arguments, answer) in [
        (vec!["pass", "outcome", "1", "passed", ""], "case 1 passed\n"),
        (vec!["pass", "outcome", "1", "blocked", "no connection"], "case 1 blocked\n"),
        (
            vec!["pass", "outcome", "1", "failed", "", "--finding", "1", "--finding", "2", "--finding", "1"],
            "case 1 failed\n",
        ),
    ] {
        success(&run(&home, &arguments, Some("2"), None), answer.as_bytes());
    }
    let stored = ledger(&home);
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
        refused(&home, &arguments, Some("2"), None, words);
    }
}

#[test]
fn every_recording_command_checks_the_pass_session_before_reading_inputs() {
    let (_scratch, home) = pass_home();
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
            refused(&home, arguments, session, None, words);
        }
    }
    home.transact(|ledger| {
        ledger.record_landing(FIX, 11, None, "More work", 5);
        assert_eq!(
            ledger.start_session(2, "low", [("web".into(), vec!["work".into()])].into(), vec!["fresh".into()], 5)?,
            3
        );
        Ok(())
    })
    .expect("running triage");
    for arguments in &commands {
        refused(&home, arguments, Some("3"), None, "session 3 is a triage session");
    }
    home.transact(|ledger| {
        // A stale running session must not record against a non-running pass.
        ledger.passes[0].state = PassState::Waiting;
        Ok(())
    })
    .expect("stale pass");
    for arguments in &commands {
        refused(&home, arguments, Some("2"), None, "pass 1 is waiting");
    }
}

#[test]
fn findings_from_file_and_stdin_copy_evidence_inherit_the_case_and_choose_area() {
    let (_scratch, home) = pass_home();
    let configuration = home.directory.join("qa.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&configuration).expect("configuration")).expect("JSON");
    config["areas"] = json!({"web":"frontend"});
    fs::write(&configuration, serde_json::to_vec(&config).expect("JSON")).expect("areas");
    fs::create_dir_all(home.directory.join("passes/1/setups/project")).expect("recipe");
    fs::write(home.directory.join("look.png"), b"pixels").expect("look");
    let mut value = finding(1);
    value["setup"] = json!("project");
    value["evidence"] = json!(["look.png"]);
    fs::write(home.directory.join("finding.json"), serde_json::to_vec(&value).expect("JSON")).expect("finding");
    success(&run(&home, &["pass", "finding", "finding.json"], Some("2"), None), b"finding 1 recorded\n");
    success(&record(&home, &value), b"finding 2 linked to open finding 1 of pass 1\n");
    assert_eq!(fs::read(home.directory.join("evidence/1/look.png")).expect("kept"), b"pixels");
    assert_eq!(fs::read(home.directory.join("evidence/2/look.png")).expect("linked kept"), b"pixels");
    let stored = ledger(&home);
    let first = &stored.findings[0];
    assert_eq!((first.pass, first.case, first.session), (1, 1, 2));
    assert_eq!(
        (&first.surface, &first.page, first.landings.as_slice()),
        (&"web".to_owned(), &"work".to_owned(), &[1][..])
    );
    assert_eq!(
        (&first.area, &first.setup, first.evidence.as_slice()),
        (&"frontend".to_owned(), &"project".to_owned(), &["look.png".to_owned()][..])
    );
    assert_eq!(
        (&first.title, &first.severity, &first.expected, &first.seen),
        (&"Start does nothing".to_owned(), &"medium".to_owned(), &"a goal starts".to_owned(), &"no goal".to_owned())
    );
    assert_eq!(first.lines, ["web press button Start"]);
    assert_eq!(stored.findings[1].to, Some(1));
    config["areas"] = json!({});
    fs::write(&configuration, serde_json::to_vec(&config).expect("JSON")).expect("owner fallback");
    success(&record(&home, &finding(1)), b"finding 3 linked to open finding 1 of pass 1\n");
    let mut explicit = finding(1);
    explicit["area"] = json!("other");
    success(&record(&home, &explicit), b"finding 4 linked to open finding 1 of pass 1\n");
    let stored = ledger(&home);
    assert_eq!(stored.findings[2].area, "qa");
    assert_eq!(stored.findings[3].area, "other");
}

#[test]
fn finding_field_and_file_refusals_leave_no_record_or_evidence() {
    let (_scratch, home) = pass_home();
    fs::write(home.directory.join("look.png"), b"pixels").expect("look");
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
                &home,
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
            &home,
            &["pass", "finding", "-"],
            Some("2"),
            Some(&serde_json::to_vec(&value_object).expect("JSON")),
            words,
        );
        assert!(!home.directory.join("evidence/1").exists());
    }
    let output = run(&home, &["pass", "finding", "missing.json"], Some("2"), None);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: missing.json: "));
    let output = run(&home, &["pass", "finding", "-"], Some("2"), Some(b"{"));
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: -: "));
    assert_eq!(ledger(&home).findings.len(), 0);
    assert!(!home.directory.join("evidence/1").exists());
}

#[test]
fn evidence_limits_refuse_and_failed_ledger_writes_remove_the_copied_directory() {
    let (_scratch, home) = pass_home();
    fs::create_dir(home.directory.join("other")).expect("other directory");
    fs::write(home.directory.join("look.png"), b"pixels").expect("look");
    fs::write(home.directory.join("other/look.png"), b"other").expect("same name");
    let large = fs::File::create(home.directory.join("large")).expect("large");
    large.set_len(16 * 1024 * 1024 + 1).expect("oversize");
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
        refused(&home, &["pass", "finding", "-"], Some("2"), Some(&serde_json::to_vec(&value).expect("JSON")), words);
        assert!(!home.directory.join("evidence/1").exists());
    }
    let before = fs::read(&home.ledger).expect("before");
    value["evidence"] = json!(["look.png", "missing"]);
    let output = record(&home, &value);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: finding refused: evidence missing: "));
    assert_eq!(fs::read(&home.ledger).expect("unchanged"), before);
    assert!(!home.directory.join("evidence/1").exists());
    let mut paths = Vec::new();
    for number in 0..17 {
        let name = format!("evidence-{number}");
        fs::write(home.directory.join(&name), []).expect("file");
        paths.push(name);
    }
    value["evidence"] = json!(paths);
    refused(
        &home,
        &["pass", "finding", "-"],
        Some("2"),
        Some(&serde_json::to_vec(&value).expect("JSON")),
        "finding refused: evidence evidence-16: more than 16 files",
    );
    assert!(!home.directory.join("evidence/1").exists());
    paths.pop();
    value["evidence"] = json!(paths);
    success(&record(&home, &value), b"finding 1 recorded\n");
    assert_eq!(fs::read_dir(home.directory.join("evidence/1")).expect("16 files").count(), 16);
    large.set_len(16 * 1024 * 1024).expect("at limit");
    value["evidence"] = json!(["large"]);
    success(&record(&home, &value), b"finding 2 linked to open finding 1 of pass 1\n");
    assert_eq!(fs::metadata(home.directory.join("evidence/2/large")).expect("kept").len(), 16 * 1024 * 1024);
    // Force the transaction's disk write to fail after copying the evidence.
    fs::create_dir(home.directory.join("ledger.tmp")).expect("block ledger write");
    let before = fs::read(&home.ledger).expect("before failed write");
    value["evidence"] = json!(["look.png"]);
    let output = record(&home, &value);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: ledger.json: "));
    assert_eq!(fs::read(&home.ledger).expect("unchanged"), before);
    assert!(!home.directory.join("evidence/3").exists());
}

#[test]
fn checks_require_a_resumed_session_and_an_open_finding_and_print_the_build() {
    let (_scratch, home) = pass_home();
    success(&record(&home, &finding(1)), b"finding 1 recorded\n");
    success(&record(&home, &finding(1)), b"finding 2 linked to open finding 1 of pass 1\n");
    refused(
        &home,
        &["pass", "check", "1", "holds", ""],
        Some("2"),
        None,
        "check refused: pass 1's first session has no finding to check",
    );
    home.transact(|ledger| {
        ledger.record_outcome(2, 1, "failed", "", &[1], 5)?;
        ledger.record_outcome(2, 2, "passed", "", &[], 5)?;
        ledger.end_pass_session(2, Exit::Code(0), None, 6, "qa", &[])?;
        ledger.file_request(1, 1, Some(30), 7)?;
        ledger.record_landing(FIX, 30, None, "Fix", 8);
        let triage =
            ledger.start_session(2, "low", [("web".into(), vec!["work".into()])].into(), vec!["fresh".into()], 9)?;
        ledger.answer_none(triage, "fix covered by pass")?;
        ledger.end_session(triage, Exit::Code(0), None, 10, "qa")?;
        ledger.keep_build(FIX, 11)?;
        ledger.set_build_holds(FIX, vec![1, 2], 2)?;
        assert_eq!(ledger.resume_pass("places/1", &["web".into()], "high", 12).expect("resume").session, 4);
        Ok(())
    })
    .expect("resume through ledger");
    for (arguments, words) in [
        (vec!["pass", "check", "1", "maybe", ""], "check refused: maybe is neither holds nor gone"),
        (vec!["pass", "check", "2", "gone", ""], "check refused: finding 2 is not open in pass 1"),
        (vec!["pass", "check", "999", "gone", ""], "check refused: finding 999 is not open in pass 1"),
    ] {
        refused(&home, &arguments, Some("4"), None, words);
    }
    success(&run(&home, &["pass", "check", "1", "holds", "still happens"], Some("4"), None), b"finding 1 holds\n");
    success(
        &run(&home, &["pass", "check", "1", "gone", "fixed"], Some("4"), None),
        b"finding 1 gone at 1123456789ab\n",
    );
    let stored = ledger(&home);
    let checks = &stored.findings[0].checks;
    assert_eq!(checks.len(), 2);
    assert!(checks[0].holds);
    assert!(!checks[1].holds);
    assert_eq!((&checks[0].notes, &checks[1].notes), (&"still happens".to_owned(), &"fixed".to_owned()));
    assert!(checks.iter().all(|check| check.session == 4 && check.commit == FIX && check.time > 0));
}

#[test]
fn notes_record_words_and_refuse_empty_words() {
    let (_scratch, home) = pass_home();
    success(&run(&home, &["pass", "note", "off path"], Some("2"), None), b"note 1\n");
    let stored = ledger(&home);
    assert_eq!(stored.notes.len(), 1);
    assert_eq!((stored.notes[0].number, stored.notes[0].pass, stored.notes[0].session), (1, 1, 2));
    assert_eq!(stored.notes[0].words, "off path");
    assert!(stored.notes[0].time > 0);
    refused(&home, &["pass", "note", ""], Some("2"), None, "note refused: no words");
    refused(&home, &["pass", "note", "  "], Some("2"), None, "note refused: no words");
}

#[test]
fn finding_options_with_no_number_or_a_nonnumber_are_usage_refusals() {
    let (_scratch, home) = pass_home();
    let before = fs::read(&home.ledger).expect("before");
    for tail in [vec!["--finding"], vec!["--finding", "no"], vec!["--finding", "1", "--finding"]] {
        let mut arguments = vec!["pass", "outcome", "1", "failed", ""];
        arguments.extend(tail);
        let output = run(&home, &arguments, Some("2"), None);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout.len(), 0);
        let stderr = String::from_utf8(output.stderr).expect("UTF-8");
        assert!(stderr.contains("expected --finding N"));
        assert!(stderr.contains("pass outcome CASE RESULT NOTES [--finding N]..."));
        assert_eq!(fs::read(&home.ledger).expect("after"), before);
    }
}
