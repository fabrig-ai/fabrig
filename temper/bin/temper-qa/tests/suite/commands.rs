//! Commands in a scratch home, no inherited environment (qa/README.md, 6).
use skein_scratch::Scratch;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::{Exit, Home, Ledger};

const CASE: &[u8] = br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#;
fn home() -> (Scratch, Home) {
    let scratch = Scratch::new("qa-commands");
    std::fs::write(
        scratch.path().join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low"}}"#,
    )
    .expect("config");
    let home = Home::open(scratch.path()).expect("home");
    home.transact(|ledger| {
        ledger.record_landing("0123456789abcdef0123456789abcdef01234567", 10, Some(20), "Work drafts", 0);
        let pages = [("web".into(), vec!["work".into(), "home".into()]), ("tui".into(), vec!["chat".into()])].into();
        ledger.start_session(1, "low", pages, vec!["fresh".into()], 0)?;
        Ok(())
    })
    .expect("session");
    (scratch, home)
}
fn run(home: &Home, arguments: &[&str], session: Option<&str>, stdin: Option<&[u8]>) -> Output {
    let mut command = Command::new(skein_scratch::paths::binary("temper-qa"));
    command
        .args(arguments)
        .env_clear()
        .env("TEMPER_INTERNAL_QA_HOME", &home.directory)
        .current_dir(&home.directory)
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(session) = session {
        command.env("TEMPER_INTERNAL_QA_SESSION", session);
    }
    let mut child = command.spawn().expect("run tool");
    if let Some(bytes) = stdin {
        child.stdin.take().expect("stdin pipe").write_all(bytes).expect("stdin");
    }
    child.wait_with_output().expect("tool output")
}
fn success(output: &Output, stdout: &[u8]) {
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, stdout);
    assert!(output.stderr.is_empty());
}
#[test]
fn missing_session_refuses_before_reading_or_writing_and_bad_config_names_its_key() {
    let (_scratch, home) = home();
    let before = std::fs::read(&home.ledger).expect("before");
    let output = run(&home, &["triage", "case", "missing.json"], None, None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stderr,
        b"temper-qa: TEMPER_INTERNAL_QA_SESSION is not set: triage commands run in a triage session\n"
    );
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read(&home.ledger).expect("after"), before);
    std::fs::write(home.directory.join("qa.json"), br#"{"unknown_key":0}"#).expect("unknown key");
    let output = run(&home, &["show"], None, None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"temper-qa: qa.json: unknown field unknown_key\n");
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read(&home.ledger).expect("after"), before);
}
#[test]
fn a_case_from_file_and_stdin_records_at_once_and_refusals_leave_it_unchanged() {
    let (_scratch, home) = home();
    std::fs::write(home.directory.join("case.json"), CASE).expect("case");
    success(&run(&home, &["triage", "case", "case.json"], Some("1"), None), b"case 1 queued\n");
    let ledger = Ledger::decode(&std::fs::read(&home.ledger).expect("ledger")).expect("decode");
    assert_eq!(ledger.cases.len(), 1);
    assert_eq!(ledger.cases[0].asks[0].steps[0].expect, "Drafts lists the draft");
    success(&run(&home, &["triage", "case", "-"], Some("1"), Some(CASE)), b"case 1 joined: landings 1\n");
    let before = std::fs::read(&home.ledger).expect("before");
    for (arguments, stdin, words) in [
        (vec!["triage", "none", "nothing"], None, "landing 1 has cases; no QA is refused"),
        (vec!["triage", "case", "-"], Some(br#"{"page":"work"}"#.as_slice()), "case refused: no surface"),
    ] {
        let output = run(&home, &arguments, Some("1"), stdin);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes());
        assert!(output.stdout.is_empty());
        assert_eq!(std::fs::read(&home.ledger).expect("unchanged"), before);
    }
}
#[test]
fn none_show_and_lines_print_the_persons_forms() {
    let (_scratch, home) = home();
    success(&run(&home, &["triage", "none", "no client change"], Some("1"), None), b"landing 1: no QA\n");
    home.transact(|ledger| {
        ledger.end_session(1, Exit::Code(0), Some(20), 60, "qa")?;
        ledger.keep_build("0123456789abcdef0123456789abcdef01234567", 60)?;
        ledger.set_build_holds("0123456789abcdef0123456789abcdef01234567", vec![1], 1)?;
        ledger.add_line("qa", "watcher needs attention", 1_794_477_840);
        ledger.record_landing("1123456789abcdef0123456789abcdef01234567", 11, None, "More work", 120);
        let pages = [("web".into(), vec!["work".into()])].into();
        let session = ledger.start_session(2, "low", pages, vec!["fresh".into()], 120)?;
        let input = temper_qa_tool::CaseInput::decode(CASE, "-")?;
        ledger.add_case(session, &input, 120)?;
        Ok(())
    })
    .expect("person's records");
    success(&run(&home, &["show"], None, None), b"landings:\n  1 0123456789ab goal 10 Work drafts: triaged 1970-01-01T00:01Z; no QA: no client change\n  2 1123456789ab goal 11 More work: untriaged; cases\n\ncases:\n  1 web work queued; landings 2\n\nbuilds:\n  0123456789ab kept; landings 1\n\nsessions:\n  1 triage landing 1 low 1970-01-01T00:00Z - 1970-01-01T00:01Z; code 0\n  2 triage landing 2 low 1970-01-01T00:02Z - running; running\n");
    success(&run(&home, &["lines"], None, None), b"1 2026-11-12T10:04Z qa: watcher needs attention\n");
}
