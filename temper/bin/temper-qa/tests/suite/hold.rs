//! Shipped holder commands against a script's drive session (temper/docs/design/testing/qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{CaseInput, Exit, Home};

const FAKE: &str = r#"#!/bin/sh
[ "$CAPTURED" = "drive-environment" ] || exit 1
scratch=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = --scratch ]; then shift; scratch="$1"; fi
  shift
done
mkdir -p "$scratch/session"
: > "$scratch/session/lines"
printf 'ready\n'
while IFS= read -r line; do
  if [ "$line" = end ]; then
    printf 'ended\n'
    exit 0
  fi
  printf '%s\n' "$line" >> "$scratch/session/lines"
  if [ "$line" = crash ]; then printf 'box failed: fake crash\n| last crash line\n'; exit 4; fi
  printf '| %s\nok\n' "$line"
done
exit 4
"#;

fn home() -> (Scratch, Home, PathBuf) {
    let scratch = Scratch::new("qa-holder");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#).expect("config");
    let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
    home.transact(&mut RealFiles, |ledger, _| {
        ledger.record_landing("build", 10, None, "Start", 0);
        let pages = [("web".into(), vec!["work".into()]), ("ctl".into(), vec!["task".into()])].into();
        let triage = ledger
            .start_session(1, "low", pages, vec!["fresh".into(), "turn-landing".into()], Vec::new(), 0)
            .expect("triage");
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
        assert_eq!(
            ledger
                .start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into(), "ctl".into()], "high", 4)
                .expect("pass")
                .session,
            2
        );
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
#[test]
fn one_holder_keeps_chain_scripts_lines_and_answers_until_close() {
    let (_scratch, home, place) = home();
    recipe(&home, "welcome", None, true);
    recipe(&home, "project-landing", Some("welcome"), false);
    recipe(&home, "chat-draft", Some("project-landing"), true);
    fs::write(place.join("setup"), format!("chat-draft {}\n", "a".repeat(64))).expect("setup");
    answer(&run(&home, &["pass", "open"]), 0, "ready\n");
    assert_eq!(fs::metadata(place.join("drive.sock")).expect("socket").permissions().mode() & 0o777, 0o600);
    answer(&run(&home, &["pass", "line", "web press button \"Start\""]), 0, "| web press button \"Start\"\nok\n");
    answer(&run(&home, &["pass", "close"]), 0, "ended\nkept lines:\nweb press button \"Start\"\n");
    free(&place);
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
}
