//! The pass commands cross main, the scripted build and the archive process (qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::faces::RealFiles;
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
    let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
    let started = home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.record_landing(COMMIT, 10, None, "Work", 0);
            let triage = ledger.start_session(
                1,
                "low",
                [("web".into(), vec!["work".into()])].into(),
                vec!["fresh".into()],
                Vec::new(),
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
            Ok(ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("pass"))
        })
        .expect("ledger operations");
    let place = PlacePaths::new(&home.directory.join("places/1"), temper_qa_tool::Kind::Box);
    place.ensure(&mut RealFiles).expect("place");
    let welcome = home.directory.join("passes/1/setups/welcome");
    fs::create_dir_all(&welcome).expect("setups");
    fs::write(welcome.join("recipe"), "recipe 1\nweb see \"Welcome\"\n").expect("local recipe");
    fs::create_dir_all(home.directory.join("test-bin")).expect("commands");
    script(&home.directory.join("test-bin/git"), "#!/bin/sh\nexit 0\n");
    script(
        &place.bin.join("temper-client-journeys"),
        &format!(
            r#"#!/bin/sh
set -eu
case "$2" in
list) printf 'welcome: Welcome.\n' ;;
hash) printf '{HASH}\n' ;;
walk)
    if [ "$3" = welcome ]; then printf 'stopped at welcome:2: missing Work\n'; exit 4; fi
    mkdir -p box/config box/data
    printf 'walked %s {HASH}\n' "$3" ;;
*) exit 1 ;;
esac
"#
        ),
    );
    let failed = run(&home, &["pass", "setup", "welcome"], Some(started.session), None);
    assert_eq!(failed.status.code(), Some(4), "{failed:?}");
    assert!(failed.stderr.is_empty(), "{failed:?}");
    assert!(String::from_utf8_lossy(&failed.stdout).contains("the box failed walking welcome"));

    let added = run(&home, &["pass", "recipe", "added", "-"], Some(started.session), Some(RECIPE));
    assert_eq!(added.status.code(), Some(0), "{added:?}");
    assert!(String::from_utf8_lossy(&added.stdout).contains("recipe added walked from fresh: 111111111111"));
    let archive = place.copies.join("added").join(format!("{HASH}-{COMMIT}.tar"));
    assert!(archive.is_file());
}
