//! Argument refusals never start a browser or a box (client/web/journeys.md, 6).
use skein_lib::{Time, Wall};
use skein_scratch::{Scratch, paths};
use skein_ui_web::mapping::Primary;
use std::{fs, process::Command};
use temper_client_shell_web::journey;

#[test]
fn again_without_a_name_is_usage() {
    let output =
        Command::new(paths::binary("temper-client-journeys")).arg("again").output().expect("run the journeys program");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("usage is text"),
        "usage: temper-client-journeys [add [--redact TEXT]... NAME FILE | again NAME | --help]\n"
    );
}

#[test]
fn a_stale_recording_prints_no_line_of_its_own() {
    let scratch = Scratch::new("temper-client-journeys-stale-message");
    let facts = journey::Facts {
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: None,
        forges: Vec::new(),
        path: b"/welcome".as_slice().into(),
        secret: Some(["fake", "-link"].concat().into_bytes().into()),
        stored: Vec::new(),
    };
    let (mut host, mut page) = facts.start();
    let mut recorder = journey::Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, temper_client_shell_web::host::Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let mut lines = journey::lines(&recorder.save(&page)).expect("fixture records");
    let marker = ["fake", "-bytes", "-in-exchange"].concat();
    let mut changed = 0;
    let mut exchange_line = 0;
    for (index, line) in lines.iter_mut().enumerate() {
        if let journey::Line::Exchange { path, body, .. } = line
            && path.as_ref() == b"/jig/v1/sign-in/link"
        {
            *body = marker.as_bytes().into();
            exchange_line = index + 1;
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "the sign-in exchange body");
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    let file = scratch.path().join("recording");
    fs::write(&file, recording).expect("write stale recording");
    let name = "a-refused-add";
    let directory = paths::tree().join("docs/design/client/web/journeys").join(name);
    assert!(!directory.exists(), "the refused journey name is unused");
    let output = Command::new(paths::binary("temper-client-journeys"))
        .args(["add", name])
        .arg(file)
        .output()
        .expect("run the journeys program");
    assert_eq!(output.status.code(), Some(1));
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(&marker), "a stale refusal exposes no exchange body");
        assert!(!text.contains("\"/jig"), "a stale refusal exposes no exchange path");
    }
    assert!(output.stdout.is_empty());
    let expected = format!(
        "temper-client-journeys: {name} is stale at turn 1, line {exchange_line}: recorded exchange, asked exchange; record it again (client/web/journeys.md, 8); nothing was written\n"
    );
    assert!(output.stderr == expected.as_bytes(), "the stale refusal names the turn, line and record kinds alone");
    assert!(!directory.exists(), "a refused add creates no journey");
}

#[test]
fn help_says_what_is_left_to_the_person() {
    let output = Command::new(paths::binary("temper-client-journeys")).arg("--help").output().expect("help");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = String::from_utf8(output.stdout).expect("help text");
    for sentence in [
        "usage: temper-client-journeys [add [--redact TEXT]... NAME FILE | again NAME | --help]",
        "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.",
        "FILE keeps every original: keep it out of the repository and remove it once added.",
        "link",
        "stored",
        "typed",
        "--redact",
        "key",
        "JWT",
        "bearer",
        "code",
        "refresh token",
    ] {
        assert!(help.contains(sentence), "help includes the promised fact");
    }
}
