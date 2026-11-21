//! Argument refusals never start a browser or a box (client/web/journeys.md, 6).
use skein_lib::{Time, Wall};
use skein_scratch::{Scratch, paths};
use skein_ui_web::mapping::Primary;
use std::{fs, process::Command};
use temper_client_shell_web::journey;

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
    let file = scratch.path().join("a-refused-print.recording");
    fs::write(&file, recording).expect("write stale recording");
    let name = "a-refused-print";
    let directory = paths::tree().join("docs/design/client/web/journeys").join(name);
    assert!(!directory.exists(), "the refused journey name is unused");
    let output = Command::new(paths::binary("temper-client-journeys"))
        .arg("print")
        .arg(file)
        .arg(scratch.path().join("out"))
        .output()
        .expect("run the journeys program");
    assert_eq!(output.status.code(), Some(1));
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(&marker), "a stale refusal exposes no exchange body");
        assert!(!text.contains("\"/jig"), "a stale refusal exposes no exchange path");
    }
    assert_eq!(output.stdout.len(), 0);
    let expected = format!(
        "temper-client-journeys: {name} is stale at turn 1, line {exchange_line}: recorded exchange, asked exchange; record it again (client/web/journeys.md, 8); nothing was written\n"
    );
    assert!(output.stderr == expected.as_bytes(), "the stale refusal names the turn, line and record kinds alone");
    assert!(!directory.exists(), "a refused print creates no journey");
}

#[test]
fn help_says_what_is_left_to_the_person() {
    let output = Command::new(paths::binary("temper-client-journeys")).arg("--help").output().expect("help");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr.len(), 0);
    let help = String::from_utf8(output.stdout).expect("help text");
    assert!(super::help(&help), "the bare page lists only public entries");
    let output =
        Command::new(paths::binary("temper-client-journeys")).args(["print", "--help"]).output().expect("print help");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr.len(), 0);
    let help =
        String::from_utf8(output.stdout).expect("print help text").split_whitespace().collect::<Vec<_>>().join(" ");
    for sentence in [
        "temper-client-journeys print [--redact TEXT]... FILE OUT",
        "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.",
        "FILE keeps every original: keep it out of the repository and remove it once printed.",
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

#[test]
fn output_in_either_writer_root_is_usage_even_through_links_and_parents() {
    use std::os::unix::fs::symlink;
    let scratch = Scratch::new("journeys-out-usage");
    for client in ["web", "tui"] {
        let root = paths::tree().join("docs/design/client").join(client).join("journeys");
        let link = scratch.path().join(client);
        symlink(&root, &link).expect("root link");
        let before: Vec<_> =
            fs::read_dir(&root).expect("writer root").map(|entry| entry.expect("entry").file_name()).collect();
        for out in [
            root.clone(),
            root.join("refused-print"),
            link.join("refused-print"),
            link.join("../journeys/refused-print"),
        ] {
            let output = Command::new(paths::binary("temper-client-journeys"))
                .arg("print")
                .arg(scratch.path().join("missing.recording"))
                .arg(&out)
                .output()
                .expect("print");
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(output.stdout.len(), 0);
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("OUT is inside a journeys root"),
                "check OUT before reading FILE"
            );
            if out != root {
                assert!(!out.exists(), "usage writes nothing");
            }
        }
        let after: Vec<_> =
            fs::read_dir(&root).expect("writer root").map(|entry| entry.expect("entry").file_name()).collect();
        assert_eq!(after, before);
    }
    assert_eq!(fs::read_dir(scratch.path()).expect("scratch").count(), 2, "only the test's links exist");
}

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

#[test]
fn another_layout_names_both_builds_and_writes_nothing() {
    use temper_client_journeys_kit as kit;
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-print-layout");
    let out = scratch.path().join("out");
    let mut lines = journey::lines(&kit::fixture_recording()).expect("browser");
    let old = jig_client_protocol_wire::VERSION - 1;
    let mut changed = 0;
    for line in &mut lines {
        if let journey::Line::Open { path, .. } = line {
            *path = format!("/jig/v1/stream?version={old}").into_bytes().into();
            changed += 1;
            break;
        }
    }
    assert_eq!(changed, 1);
    let mut browser = Vec::new();
    for line in lines {
        line.write(&mut browser);
    }
    let mut lines = tj::lines(&kit::tui_approval_recording()).expect("TUI");
    let mut changed = 0;
    for line in &mut lines {
        if let tj::Line::Version(version) = line {
            *version = b"0.0.0 (aaaaaaaaaaaa)".as_slice().into();
            changed += 1;
        }
    }
    assert_eq!(changed, 1);
    let terminal = kit::tui_written(&lines);
    for (name, recording, words) in [
        (
            "browser",
            browser,
            format!("at jig-client-wire {old}; this build reads jig-client-wire {}", jig_client_protocol_wire::VERSION),
        ),
        (
            "terminal",
            terminal,
            format!(
                "by temper 0.0.0 (aaaaaaaaaaaa); this is temper {} ({})",
                temper_version::VERSION,
                temper_version::COMMIT
            ),
        ),
    ] {
        let file = scratch.path().join(name);
        fs::write(&file, recording).expect("saved recording");
        let output = Command::new(paths::binary("temper-client-journeys"))
            .arg("print")
            .arg(&file)
            .arg(&out)
            .output()
            .expect("print layout");
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout.len(), 0);
        assert_eq!(
            String::from_utf8(output.stderr).expect("stderr"),
            format!(
                "temper-client-journeys: {} was recorded {words}: print it with the build that recorded it; nothing was written\n",
                file.display()
            )
        );
        assert!(!out.exists());
    }
}
