//! Commands in a scratch home, no inherited environment (qa/README.md, 6).
use skein_scratch::Scratch;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{CaseInput, Home};

const CASE: &[u8] = br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]}"#;
pub(super) fn home() -> (Scratch, Home) {
    let scratch = Scratch::new("qa-commands");
    std::fs::write(
        scratch.path().join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#,
    )
    .expect("config");
    let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
    home.transact(&mut RealFiles, |ledger, _| {
        ledger.record_landing("0123456789abcdef0123456789abcdef01234567", 10, Some(20), "Work drafts", 0);
        let pages = [("web".into(), vec!["work".into(), "home".into()]), ("tui".into(), vec!["chat".into()])].into();
        ledger.start_session(1, "low", pages, vec!["fresh".into()], Vec::new(), 0)?;
        Ok(())
    })
    .expect("session");
    (scratch, home)
}
pub(super) fn run(home: &Home, arguments: &[&str], session: Option<&str>, stdin: Option<&[u8]>) -> Output {
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
pub(super) fn success(output: &Output, stdout: &[u8]) {
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, stdout);
    assert_eq!(output.stderr.len(), 0);
}
#[test]
fn a_case_from_file_and_stdin_records_at_once_and_refusals_leave_it_unchanged() {
    let (_scratch, home) = home();
    home.transact(&mut RealFiles, |ledger, _| ledger.add_case(1, &CaseInput::decode(CASE, "fixture")?, 1))
        .expect("case for stdin to join");
    success(&run(&home, &["triage", "case", "-"], Some("1"), Some(CASE)), b"case 1 joined: landings 1\n");
}

#[test]
fn wake_pokes_without_a_session_or_ledger_write_and_saves_poke_too() {
    use std::{fs, io::Read};
    let (_scratch, home) = home();
    let before = fs::read(&home.ledger).expect("ledger");
    let listener = temper_qa_tool::events::bind(&home.directory, &mut RealFiles).expect("fake loop socket");
    success(&run(&home, &["wake"], None, None), b"");
    let (mut stream, _) = listener.accept().expect("wake connection");
    assert_eq!(stream.read(&mut [0]).expect("closed"), 0);
    assert_eq!(fs::read(&home.ledger).expect("unchanged ledger"), before);
}
