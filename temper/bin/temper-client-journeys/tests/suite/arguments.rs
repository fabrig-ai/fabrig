//! The shipped print command's report and destination (temper/docs/design/client/web/journeys.md, 6).
use skein_scratch::{Scratch, paths};
use std::{fs, process::Command};
use temper_client_shell_web::journey;

#[test]
fn print_reports_redaction_then_destination_then_caution() {
    let scratch = Scratch::new("journeys-print-output");
    let file = scratch.path().join("journey.recording");
    let out = scratch.path().join("out");
    let secret = ["private", "-link"].concat().into_bytes();
    let recording = temper_client_journeys_kit::fixture_with_marks_and_secret(Some(secret.into()));
    fs::write(&file, &recording).expect("recording");
    let given = "private-text";
    let redacted = journey::redact(&recording, &[given.as_bytes().into()]).expect("redacted fixture");
    let output = Command::new(paths::binary("temper-client-journeys"))
        .args(["print", "--redact", given])
        .arg(&file)
        .arg(&out)
        .output()
        .expect("print");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stderr.len(), 0);
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout"),
        format!(
            "link: 1; stored: 0; typed: 0; given: 1; places replaced: {}\nprinted journey into {}\nThese aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.\n",
            redacted.places,
            out.display()
        )
    );
    assert!(out.join("temper/docs/design/client/web/journeys/journey/01.html").is_file());
    assert_eq!(fs::read(file).expect("original remains"), recording.as_ref());
}
