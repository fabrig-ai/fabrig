//! The pass commands cross main, the scripted build and the archive process (qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::{CaseInput, Exit, Home, place::PlacePaths};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const RECIPE: &[u8] = br#"{"after":"fresh","lines":["web see \"Added\""],"readme":"An added state."}"#;
fn script(path: &Path, text: &str) {
    fs::write(path, text).expect("script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable");
}
fn run(home: &Home, arguments: &[&str], session: Option<u64>, input: Option<&[u8]>) -> Output {
    let mut command = Command::new(skein_scratch::paths::binary("temper-qa"));
    command
        .args(arguments)
        .env_clear()
        .env("TEMPER_INTERNAL_QA_HOME", &home.directory)
        .env("PATH", format!("{}:/usr/bin:/bin", home.directory.join("test-bin").display()))
        .current_dir(&home.directory)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(session) = session {
        command.env("TEMPER_INTERNAL_QA_SESSION", session.to_string());
    }
    let mut child = command.spawn().expect("tool");
    if let Some(input) = input {
        child.stdin.take().expect("stdin").write_all(input).expect("input");
    }
    child.wait_with_output().expect("output")
}

#[test]
fn setup_and_recipe_preserve_walk_exits_and_use_the_pass_place_and_captured_processes() {
    let scratch = Scratch::new("qa-place-commands");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["unused"]}}"#).expect("configuration");
    let home = Home::open(scratch.path()).expect("home");
    let started = home
        .transact(|ledger| {
            ledger.record_landing(COMMIT, 10, None, "Work", 0);
            let triage = ledger.start_session(
                1,
                "low",
                [("web".into(), vec!["work".into()])].into(),
                vec!["fresh".into()],
                0,
            )?;
            let case = CaseInput::decode(
                br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open","expect":"Work"}]}"#,
                "-",
            )?;
            ledger.add_case(triage, &case, 1)?;
            ledger.end_session(triage, Exit::Code(0), None, 2, "qa")?;
            ledger.keep_build(COMMIT, 3)?;
            ledger.set_build_holds(COMMIT, vec![1], 1)?;
            Ok(ledger.start_pass("places/1", &["web".into()], "high", 4).expect("pass"))
        })
        .expect("ledger operations");
    let place = PlacePaths::new(&home.directory.join("places/1"));
    place.ensure().expect("place");
    fs::create_dir_all(home.directory.join("passes/1/setups")).expect("setups");
    fs::create_dir_all(home.directory.join("test-bin")).expect("commands");
    script(&home.directory.join("test-bin/git"), "#!/bin/sh\nexit 0\n");
    script(
        &place.bin.join("temper-client-journeys"),
        &format!(
            r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> arguments
case "$2" in
list)
    printf 'welcome: Welcome.\n'
    if test -f "$4/added/recipe"; then printf 'added: Added.\n'; fi ;;
hash) printf '{HASH}\n' ;;
walk)
    if test -f walk-exit; then printf 'stopped at %s:2: missing Work\n' "$3"; exit "$(cat walk-exit)"; fi
    mkdir -p box/config box/data
    printf 'walked %s {HASH}\n' "$3" ;;
*) exit 1 ;;
esac
"#
        ),
    );
    let missing = run(&home, &["pass", "setup", "fresh"], None, None);
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(
        missing.stderr,
        b"temper-qa: TEMPER_INTERNAL_QA_SESSION is not set: pass commands run in a pass session\n"
    );
    for code in [4, 5] {
        fs::write(place.directory.join("walk-exit"), code.to_string()).expect("exit");
        let output = run(&home, &["pass", "setup", "welcome"], Some(started.session), None);
        assert_eq!(output.status.code(), Some(code), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains(if code == 4 {
            "the box failed walking welcome"
        } else {
            "stopped at welcome:2: missing Work"
        }));
    }
    fs::remove_file(place.directory.join("walk-exit")).expect("walk succeeds");
    let fresh = run(&home, &["pass", "setup", "fresh"], Some(started.session), None);
    assert_eq!(fresh.status.code(), Some(0), "{fresh:?}");
    assert_eq!(fresh.stdout, b"setup fresh: an empty box\n");
    let added = run(&home, &["pass", "recipe", "added", "-"], Some(started.session), Some(RECIPE));
    assert_eq!(added.status.code(), Some(0), "{added:?}");
    assert!(String::from_utf8_lossy(&added.stdout).contains("recipe added walked from fresh: 111111111111"));
    let archive = place.copies.join("added").join(format!("{HASH}-{COMMIT}.tar"));
    assert!(archive.is_file());
    fs::write(place.scratch.join("data/stale"), "not in copy").expect("stale box");
    let restored = run(&home, &["pass", "setup", "added"], Some(started.session), None);
    assert_eq!(restored.status.code(), Some(0), "{restored:?}");
    assert!(
        String::from_utf8_lossy(&restored.stdout).contains("setup added: restored from its copy made at 0123456789ab")
    );
    assert!(!place.scratch.join("data/stale").exists());
    assert_eq!(fs::read_to_string(place.setup).expect("setup"), format!("added {HASH}\n"));
    fs::write(home.directory.join("recipe.json"), RECIPE).expect("recipe file");
    let rewritten = run(&home, &["pass", "recipe", "added", "recipe.json"], Some(started.session), None);
    assert_eq!(rewritten.status.code(), Some(0), "{rewritten:?}");
}
