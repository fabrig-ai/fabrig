//! One-shot commands against a script's drive session, no box or browser (qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::{CaseInput, Exit, Home};

const FAKE: &str = r#"#!/bin/sh
printf '%s\n' "$@" > arguments
[ "$CAPTURED" = "drive-environment" ] || exit 1
scratch=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = --scratch ]; then shift; scratch="$1"; fi
  shift
done
mkdir -p "$scratch/session"
: > "$scratch/session/lines"
if [ -f start-status ]; then
  code=$(cat start-status)
  printf 'box failed: fake start %s\n| last start line\n' "$code"
  exit "$code"
fi
printf 'ready\n'
while IFS= read -r line; do
  if [ "$line" = end ]; then
    if [ -f stop-status ]; then printf 'box failed: fake stop\n'; exit 4; fi
    printf 'ended\n'
    exit 0
  fi
  if [ "$line" != 'web look' ]; then printf '%s\n' "$line" >> "$scratch/session/lines"; fi
  if [ "$line" = crash ]; then printf 'box failed: fake crash\n| last crash line\n'; exit 4; fi
  printf '| %s\n' "$line"
  if [ "$line" = fail ]; then printf 'failed: fake line\n'; else printf 'ok\n'; fi
done
exit 4
"#;

fn home() -> (Scratch, Home, PathBuf) {
    let scratch = Scratch::new("qa-holder");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#).expect("config");
    let home = Home::open(scratch.path()).expect("home");
    home.transact(|ledger| {
        ledger.record_landing("build", 10, None, "Start", 0);
        let pages = [("web".into(), vec!["work".into()])].into();
        let triage = ledger.start_session(1, "low", pages, vec!["fresh".into()], 0).expect("triage");
        ledger.add_case(
            triage,
            &CaseInput::decode(
                br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"press Start","expect":"started"}]}"#,
                "case",
            )?,
            1,
        )?;
        ledger.end_session(triage, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build("build", 3)?;
        ledger.set_build_holds("build", vec![1], 1)?;
        assert_eq!(ledger.start_pass("places/1", &["web".into()], "high", 4).expect("pass").session, 2);
        Ok(())
    })
    .expect("pass records");
    let place = home.directory.join("places/1");
    for directory in ["bin", "bundle", "box"] {
        fs::create_dir_all(place.join(directory)).expect("place");
    }
    let fake = place.join("bin/temper-client-journeys");
    fs::write(&fake, FAKE).expect("fake");
    fs::set_permissions(fake, fs::Permissions::from_mode(0o700)).expect("executable");
    (scratch, home, place)
}
fn run(home: &Home, arguments: &[&str]) -> Output {
    Command::new(skein_scratch::paths::binary("temper-qa"))
        .args(arguments)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("CAPTURED", "drive-environment")
        .env("TEMPER_INTERNAL_QA_HOME", &home.directory)
        .env("TEMPER_INTERNAL_QA_SESSION", "2")
        .current_dir(&home.directory)
        .stdin(Stdio::null())
        .output()
        .expect("command")
}
fn answer(output: &Output, code: i32, text: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, text.as_bytes());
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}
fn refusal(output: &Output, text: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, format!("temper-qa: {text}\n").as_bytes());
}
fn free(place: &std::path::Path) {
    assert!(!place.join("drive.sock").exists());
    let lock = fs::File::options().read(true).write(true).open(place.join("drive.lock")).expect("lock");
    lock.try_lock().expect("holder released lock");
}
fn recipe(home: &Home, name: &str, parent: Option<&str>, scripts: bool) {
    let directory = home.directory.join("passes/1/setups").join(name);
    fs::create_dir_all(&directory).expect("recipe directory");
    fs::write(
        directory.join("recipe"),
        format!("recipe 1\n{}web see Start\n", parent.map_or_else(String::new, |name| format!("after {name}\n"))),
    )
    .expect("recipe");
    if scripts {
        fs::write(directory.join("scripts.json"), "{}").expect("scripts");
    }
}
fn arguments(home: &Home, place: &std::path::Path, scripts: &[&str]) {
    let mut expected = vec![
        "drive".to_owned(),
        "--bin".into(),
        place.join("bin").display().to_string(),
        "--bundle".into(),
        place.join("bundle").display().to_string(),
        "--scratch".into(),
        place.join("box").display().to_string(),
    ];
    for name in scripts {
        expected.push("--scripts".into());
        expected.push(home.directory.join("passes/1/setups").join(name).join("scripts.json").display().to_string());
    }
    expected.push("--strict".into());
    assert_eq!(
        fs::read_to_string(home.directory.join("arguments")).expect("arguments"),
        format!("{}\n", expected.join("\n"))
    );
}

#[test]
fn one_holder_keeps_chain_scripts_lines_and_answers_until_close() {
    let (_scratch, home, place) = home();
    recipe(&home, "welcome", None, true);
    recipe(&home, "project-landing", Some("welcome"), false);
    recipe(&home, "chat-draft", Some("project-landing"), true);
    fs::write(place.join("setup"), format!("chat-draft {}\n", "a".repeat(64))).expect("setup");
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    arguments(&home, &place, &["welcome", "chat-draft"]);
    assert_eq!(fs::metadata(place.join("drive.sock")).expect("socket").permissions().mode() & 0o777, 0o600);
    refusal(
        &run(&home, &["pass", "open"]),
        &format!("the drive session is open at {}; close it first: temper-qa pass close", place.display()),
    );
    refusal(
        &run(&home, &["pass", "hold"]),
        &format!("the drive session is open at {}; close it first: temper-qa pass close", place.display()),
    );
    answer(&run(&home, &["pass", "line", "web press button \"Start\""]), 0, "| web press button \"Start\"\nok\n");
    answer(&run(&home, &["pass", "line", "fail"]), 5, "| fail\nfailed: fake line\n");
    answer(&run(&home, &["pass", "line", "web look"]), 0, "| web look\nok\n");
    answer(&run(&home, &["pass", "close"]), 0, "ended\nkept lines:\nweb press button \"Start\"\nfail\n");
    free(&place);
    refusal(
        &run(&home, &["pass", "line", "web look"]),
        &format!("no drive session is open at {}: temper-qa pass open first", place.display()),
    );
}
#[test]
fn fresh_starts_without_scripts_and_preserves_start_failure_statuses() {
    let (_scratch, home, place) = home();
    fs::write(place.join("setup"), "fresh\n").expect("fresh");
    // An old socket file is removed only after the holder has taken the lock.
    fs::write(place.join("drive.sock"), "stale").expect("stale socket");
    for status in [3, 4] {
        fs::write(home.directory.join("start-status"), status.to_string()).expect("failure mode");
        answer(
            &run(&home, &["pass", "open"]),
            status,
            &format!("box failed: fake start {status}\n| last start line\n"),
        );
        arguments(&home, &place, &[]);
        free(&place);
    }
    fs::remove_file(home.directory.join("start-status")).expect("normal start");
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    answer(&run(&home, &["pass", "close"]), 0, "ended\nkept lines:\n");
    free(&place);
}
#[test]
fn open_refuses_missing_setup_unreadable_chains_and_ended_sessions() {
    let (_scratch, home, place) = home();
    refusal(
        &run(&home, &["pass", "open"]),
        &format!("no setup is restored at {}: temper-qa pass setup NAME first", place.display()),
    );
    fs::write(place.join("setup"), format!("leaf {}\n", "a".repeat(64))).expect("setup");
    recipe(&home, "leaf", Some("missing"), false);
    let output = run(&home, &["pass", "open"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("temper-qa: recipe missing: "));
    recipe(&home, "missing", Some("leaf"), false);
    refusal(&run(&home, &["pass", "open"]), "recipe missing: cycle in after chain");
    assert!(!home.directory.join("arguments").exists(), "refusals must not start the drive");
    recipe(&home, "leaf", None, false);
    fs::write(home.directory.join("passes/1/setups/leaf/recipe"), "recipe 1\nafter\nweb see Start\n")
        .expect("bad after");
    refusal(&run(&home, &["pass", "open"]), "recipe leaf: invalid after record");
    for depth in 0..=9 {
        let parent = (depth > 0).then(|| format!("n{}", depth - 1));
        recipe(&home, &format!("n{depth}"), parent.as_deref(), false);
    }
    fs::write(place.join("setup"), format!("n9 {}\n", "a".repeat(64))).expect("ninth parent");
    refusal(&run(&home, &["pass", "open"]), "recipe n1: more than eight parents");
    fs::write(place.join("setup"), format!("n8 {}\n", "a".repeat(64))).expect("eighth parent");
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    answer(&run(&home, &["pass", "close"]), 0, "ended\nkept lines:\n");
    free(&place);
    home.transact(|ledger| ledger.end_pass_session(2, Exit::Code(0), None, 5, "qa", &[])).expect("ended session");
    refusal(&run(&home, &["pass", "open"]), "session 2 is not running");
    refusal(&run(&home, &["pass", "line", "web look"]), "session 2 is not running");
    refusal(&run(&home, &["pass", "close"]), "session 2 is not running");
}
#[test]
fn box_failures_cross_line_and_close_with_status_four() {
    let (_scratch, home, place) = home();
    fs::write(place.join("setup"), "fresh\n").expect("fresh");
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    answer(&run(&home, &["pass", "line", "crash"]), 4, "box failed: fake crash\n| last crash line\n");
    // Wait on the holder's lock, not a guessed sleep, before checking cleanup.
    fs::File::open(place.join("drive.lock")).expect("lock").lock().expect("wait holder");
    free(&place);
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    fs::write(home.directory.join("stop-status"), "4").expect("failed stop");
    answer(&run(&home, &["pass", "close"]), 4, "box failed: fake stop\nkept lines:\n");
    free(&place);
}
