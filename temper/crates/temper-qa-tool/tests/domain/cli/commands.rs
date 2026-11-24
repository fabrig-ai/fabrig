//! Triage, wake and read-only commands over the runner's home (temper/docs/design/testing/qa/README.md, 6).
use super::{put, run, success};
use std::io::Read;
use temper_qa_tool::{
    CaseInput, Exit, FindingInput, Ledger,
    faces::{FilesFace, PathKind, RemoveMode},
    testing::World,
};

const CASE: &[u8] = br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#;
pub(super) fn home() -> World {
    let world = World::new();
    world
        .home
        .transact(&mut world.files(), |ledger, _| {
            ledger.record_landing("0123456789abcdef0123456789abcdef01234567", 10, Some(20), "Work drafts", 0);
            let pages =
                [("web".into(), vec!["work".into(), "home".into()]), ("tui".into(), vec!["chat".into()])].into();
            ledger.start_session(1, "low", pages, vec!["fresh".into()], Vec::new(), 0)?;
            Ok(())
        })
        .expect("session");
    world
}
#[test]
fn missing_session_refuses_before_reading_or_writing_and_bad_config_names_its_key() {
    let world = home();
    let home = &world.home;
    let mut files = world.files();
    let before = files.read(&home.ledger).expect("before");
    let output = run(&world, &["triage", "case", "missing.json"], None, None);
    assert_eq!(output.code, 1);
    assert_eq!(
        output.stderr,
        b"temper-qa: TEMPER_INTERNAL_QA_SESSION is not set: triage commands run in a triage session\n"
    );
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(files.read(&home.ledger).expect("after"), before);
    put(&mut files, &home.directory.join("qa.json"), br#"{"unknown_key":0}"#);
    let output = run(&world, &["show"], None, None);
    assert_eq!(output.code, 1);
    assert_eq!(output.stderr, b"temper-qa: qa.json: unknown field unknown_key\n");
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(files.read(&home.ledger).expect("after"), before);
}
#[test]
fn a_case_from_file_and_stdin_records_at_once_and_refusals_leave_it_unchanged() {
    let world = home();
    let home = &world.home;
    let mut files = world.files();
    put(&mut files, &home.directory.join("case.json"), CASE);
    success(&run(&world, &["triage", "case", "case.json"], Some("1"), None), b"case 1 queued\n");
    let ledger = Ledger::decode(&files.read(&home.ledger).expect("ledger")).expect("decode");
    assert_eq!(ledger.cases.len(), 1);
    assert_eq!(ledger.cases[0].asks[0].steps[0].expect, "Drafts lists the draft");
    let before = files.read(&home.ledger).expect("before");
    for (arguments, stdin, words) in [
        (vec!["triage", "none", "nothing"], None, "landing 1 has cases; no QA is refused"),
        (vec!["triage", "case", "-"], Some(br#"{"page":"work"}"#.as_slice()), "case refused: no surface"),
    ] {
        let output = run(&world, &arguments, Some("1"), stdin);
        assert_eq!(output.code, 1);
        assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes());
        assert_eq!(output.stdout.len(), 0);
        assert_eq!(files.read(&home.ledger).expect("unchanged"), before);
    }
}
#[test]
fn none_show_and_lines_print_the_persons_forms() {
    let world = home();
    let home = &world.home;
    let mut files = world.files();
    success(&run(&world, &["triage", "none", "no client change"], Some("1"), None), b"landing 1: no QA\n");
    home.transact(&mut files, |ledger, _| {
        ledger.end_session(1, Exit::Code(0), Some(20), 60, "qa")?;
        ledger.keep_build("0123456789abcdef0123456789abcdef01234567", 60)?;
        ledger.set_build_holds("0123456789abcdef0123456789abcdef01234567", vec![1], 1)?;
        ledger.add_line("qa", "watcher needs attention", 1_794_477_840);
        ledger.record_landing("1123456789abcdef0123456789abcdef01234567", 11, None, "More work", 120);
        let pages = [("web".into(), vec!["work".into()])].into();
        let session = ledger.start_session(2, "low", pages, vec!["fresh".into()], Vec::new(), 120)?;
        let input = temper_qa_tool::CaseInput::decode(CASE, "-")?;
        ledger.add_case(session, &input, 120)?;
        Ok(())
    })
    .expect("person's records");
    success(&run(&world, &["show"], None, None), b"landings:\n  1 0123456789ab goal 10 Work drafts: triaged 1970-01-01T00:01Z; no QA: no client change\n  2 1123456789ab goal 11 More work: untriaged; cases\n\ncases:\n  1 web work queued; pass none; landings 2\n\nbuilds:\n  0123456789ab kept; landings 1; held none\n\nsessions:\n  1 triage landing 1 low 1970-01-01T00:00Z - 1970-01-01T00:01Z; code 0\n  2 triage landing 2 low 1970-01-01T00:02Z - running; running\n\npasses:\n\nfindings:\n\nnotes:\n\ngaps: 0 open; 0 filed; 0 closed\n");
    success(&run(&world, &["lines"], None, None), b"1 2026-11-12T10:04Z qa: watcher needs attention\n");
}

#[test]
fn q3_configuration_show_and_report_cross_the_binary_boundary() {
    use serde_json::json;
    let world = home();
    let home = &world.home;
    let mut files = world.files();
    let full = json!({"owner":"qa","git":{"directory":"repo","branch":"main","fetch":"origin"},
        "agent":{"command":["agent"],"triage":"low","pass":"high"},
        "bounds":{"poll":1,"triage":2,"build_wait":3,"pass":4},"builds":"kept","places":["places/1"],"surfaces":["web","tui"],
        "areas":{"web":"frontend","tui":"terminal"},"request":{"command":["file"],"state":["state"],"area":"fixes"},"notify":["notify"]});
    put(&mut files, &home.directory.join("qa.json"), serde_json::to_vec(&full).expect("config JSON"));
    home.transact(&mut files, |ledger, _| {
        ledger.add_case(1, &CaseInput::decode(CASE, "-")?, 1)?;
        ledger.end_session(1, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build("build", 3)?;
        ledger.set_build_holds("build", vec![1], 5)?;
        let pass = ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("pass");
        let input = FindingInput {
            case: 1,
            title: "Start does nothing".into(),
            severity: "medium".into(),
            area: None,
            setup: "fresh".into(),
            lines: vec!["web press button Start".into()],
            expected: "a goal starts".into(),
            seen: "no goal".into(),
            evidence: vec!["look.png".into()],
        };
        ledger.record_finding(pass.session, &input, &[], "frontend", 5)?;
        ledger.record_finding(pass.session, &input, &[], "frontend", 5)?;
        ledger.record_outcome(pass.session, 1, "failed", "no goal", &[1], 6)?;
        ledger.record_note(pass.session, "off-path", 6)?;
        ledger.end_pass_session(pass.session, Exit::Code(0), None, 7, "qa", &[])
    })
    .expect("pass records");
    let output = run(&world, &["show"], None, None);
    assert_eq!(output.code, 0, "{:?}", output.stderr);
    let shown = String::from_utf8(output.stdout).expect("text");
    assert!(shown.contains("\npasses:\n  1 places/1 build waiting; cases 1; goal none\n"));
    assert!(shown.contains("\nfindings:\n  1 pass 1 case 1 medium open Start does nothing\n"));
    assert!(shown.contains("\nnotes:\n  1 pass 1 off-path\n"));
    let expected = json!({"pass":1,"place":"places/1","build":"build","state":"waiting",
        "cases":[{"case":1,"surface":"web","page":"work","landings":[1],"expected":["Drafts lists the draft"],"outcome":"failed","notes":"no goal","findings":[1]}],
        "findings":[{"finding":1,"title":"Start does nothing","severity":"medium","area":"frontend","landings":[1],"case":1,"setup":"fresh","reproduction":["web press button Start"],"expected":"a goal starts","seen":"no goal","evidence":["look.png"],"state":"open","goals":[]},
            {"finding":2,"title":"Start does nothing","severity":"medium","area":"frontend","landings":[1],"case":1,"setup":"fresh","reproduction":["web press button Start"],"expected":"a goal starts","seen":"no goal","evidence":["look.png"],"state":"linked","goals":[],"linked":1}],"notes":["off-path"],"gaps":[]});
    let report = run(&world, &["report", "1"], None, None);
    assert_eq!(report.code, 0, "{:?}", report.stderr);
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&report.stdout).expect("report"), expected);
    home.write_pass_report(1, &mut files).expect("loop report writer");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &files.read(&home.directory.join("passes/1/report.json")).expect("report file")
        )
        .expect("JSON"),
        expected
    );
    let before = files.read(&home.ledger).expect("ledger");
    for (pointer, key, words) in [
        ("/request", "unknown", "qa.json: request: unknown field unknown"),
        ("/agent", "pass", "qa.json: agent.pass: missing pass"),
    ] {
        let mut value = full.clone();
        let object = value.pointer_mut(pointer).expect("object").as_object_mut().expect("object");
        if key == "pass" {
            object.remove(key);
        } else {
            object.insert(key.into(), json!(true));
        }
        put(&mut files, &home.directory.join("qa.json"), serde_json::to_vec(&value).expect("JSON"));
        let output = run(&world, &["show"], None, None);
        assert_eq!(output.code, 1);
        assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes());
        assert_eq!(output.stdout.len(), 0);
        assert_eq!(files.read(&home.ledger).expect("unchanged"), before);
    }
}

#[test]
fn wake_pokes_without_a_session_or_ledger_write_and_saves_poke_too() {
    let world = home();
    let home = &world.home;
    let mut files = world.files();
    let socket = home.directory.join("run.sock");
    let before = files.read(&home.ledger).expect("ledger");
    success(&run(&world, &["wake"], None, None), b"");
    assert_eq!(files.read(&home.ledger).expect("unchanged ledger"), before);

    let listener = files.socket(&socket).expect("fake loop socket");
    success(&run(&world, &["triage", "none", "internal change"], Some("1"), None), b"landing 1: no QA\n");
    let mut stream = files.accept(&listener).expect("accept").expect("saved command connection");
    assert_eq!(stream.read(&mut [0]).expect("closed"), 0);
    let saved = files.read(&home.ledger).expect("saved ledger");
    assert_ne!(saved, before);

    drop(listener);
    assert_eq!(files.kind(&socket), Some(PathKind::Socket));
    success(&run(&world, &["wake"], None, None), b"");
    files.remove(&socket, RemoveMode::File).expect("remove stale socket");
    success(&run(&world, &["wake"], None, None), b"");
    put(&mut files, &home.directory.join("qa.json"), b"unread configuration");
    success(&run(&world, &["wake"], None, None), b"");
    assert_eq!(files.read(&home.ledger).expect("unchanged ledger"), saved);
}
