//! Print arguments and outcomes (temper/docs/design/client/web/journeys.md, 6).
use super::{Memory, OUT, build, snapshot, welcome_lines, words, write_lines};
use crate::run::{Outcome, print_arguments, print_command};
use std::{cell::Cell, io, path::Path};
use temper_client_shell_web::journey;
use temper_client_walk_lines::layout::{TUI, WEB};

#[test]
fn a_stale_recording_prints_no_line_of_its_own() {
    let mut lines = welcome_lines();
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
    let recording = write_lines(&lines);
    let name = "a-refused-print";
    let answer = print_recording(&format!("{name}.recording"), &recording, &build());
    for text in [&answer.stdout, &answer.stderr] {
        assert!(!text.contains(&marker), "a stale refusal exposes no exchange body");
        assert!(!text.contains("fake-link"), "a stale refusal exposes no link original");
        assert!(!text.contains("\"/jig"), "a stale refusal exposes no exchange path");
    }
    let expected = format!(
        "temper-client-journeys: {name} is stale at turn 1, line {exchange_line}: recorded exchange, asked exchange; record it again (client/web/journeys.md, 8); nothing was written\n"
    );
    assert!(answer.stderr == expected, "the stale refusal names the turn, line and record kinds alone");
}

#[test]
fn another_layout_names_both_builds_and_writes_nothing() {
    use temper_client_journeys_kit as kit;
    use temper_client_shell_tui::journey as tj;
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
    let browser = write_lines(&lines).into_vec();
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
    let build = build();
    for (name, recording, words) in [
        (
            "browser",
            browser,
            format!("at jig-client-wire {old}; this build reads jig-client-wire {}", jig_client_protocol_wire::VERSION),
        ),
        (
            "terminal",
            terminal,
            format!("by temper 0.0.0 (aaaaaaaaaaaa); this is temper {} ({})", "1.2.3", "abcdef123456-dirty"),
        ),
    ] {
        let answer = print_recording(name, &recording, &build);
        assert_eq!(
            answer.stderr,
            format!(
                "temper-client-journeys: {name} was recorded {words}: print it with the build that recorded it; nothing was written\n"
            )
        );
    }
}

fn print_recording(file: &str, recording: &[u8], build: &crate::Build) -> Outcome {
    let mut output = Memory::default();
    let before = snapshot(&output);
    let reads = Cell::new(0);
    let answer = print_command(
        &words(&[file, OUT]),
        build,
        |out| Ok(out.to_owned()),
        |read| {
            assert_eq!(read, Path::new(file).as_os_str());
            reads.set(reads.get() + 1);
            Ok(recording.to_vec())
        },
        &mut output,
    );
    assert_eq!(reads.get(), 1);
    assert_eq!(answer.exit, 1);
    assert_eq!(answer.stdout, "");
    assert_eq!(snapshot(&output), before, "refused print creates no journey or other output");
    answer
}

#[test]
fn parse_keeps_operands_and_redactions_in_order() {
    let arguments = words(&["--redact", "one", "FILE", "--redact", "two", "OUT"]);
    let parsed = print_arguments(&arguments).expect("two operands and redactions");
    assert_eq!(parsed.file, "FILE");
    assert_eq!(parsed.out, Path::new("OUT"));
    assert_eq!(parsed.given, words(&["one", "two"]));
}

#[test]
fn usage_echoes_nothing_given() {
    let private = ["private", "-usage-text"].concat();
    let flag = format!("--{private}");
    for (arguments, why) in [
        (&["--redact", "FILE", OUT][..], "expected FILE and OUT"),
        (&["--bogus", &private, "FILE", OUT][..], "unknown flag"),
        (&["--redact", &private][..], "expected FILE and OUT"),
        (&["--redact"][..], "--redact TEXT is not given"),
        (&["--redact", "", "FILE", OUT][..], "--redact TEXT is empty or starts with --"),
        (&["--redact", &flag, "FILE", OUT][..], "--redact TEXT is empty or starts with --"),
    ] {
        let mut output = Memory::default();
        let answer = print_command(
            &words(arguments),
            &build(),
            |_| panic!("usage does not resolve OUT"),
            |_| panic!("usage does not read FILE"),
            &mut output,
        );
        assert_eq!(answer.exit, 2);
        assert_eq!(answer.stdout, "");
        assert_eq!(answer.stderr, usage(why));
        for text in [&answer.stdout, &answer.stderr] {
            assert!(!text.contains(&private), "usage echoes no given text");
        }
        assert_eq!(snapshot(&output), snapshot(&Memory::default()));
    }
}

fn usage(why: &str) -> String {
    format!("temper-client-journeys print: {why}\nusage:\n  temper-client-journeys print [--redact TEXT]... FILE OUT\n")
}

#[test]
fn output_in_either_writer_root_is_usage_even_through_links_and_parents() {
    const REPO: &str = "/checkout";
    let cwd = Path::new("/scratch");
    let repo = Path::new(REPO);
    let roots = [repo.join(WEB), repo.join(TUI)];
    for (client, root) in ["web", "tui"].into_iter().zip(&roots) {
        let link = cwd.join(client);
        let mut output = Memory::default();
        for root in &roots {
            output.directory(root);
        }
        let before = snapshot(&output);
        for out in [
            root.clone(),
            root.join("refused-print"),
            Path::new(client).join("refused-print"),
            Path::new(client).join("../journeys/refused-print"),
        ] {
            let answer = print_command(
                &words(&["missing.recording", out.to_str().expect("fixture path")]),
                &build(),
                |out| {
                    crate::destination(
                        out,
                        repo,
                        |out| Ok(cwd.join(out)),
                        |prefix| {
                            if prefix == link {
                                Ok(root.clone())
                            } else if roots.iter().any(|root| root.starts_with(prefix)) || cwd.starts_with(prefix) {
                                Ok(prefix.to_owned())
                            } else {
                                Err(io::ErrorKind::NotFound.into())
                            }
                        },
                    )
                },
                |_| panic!("check OUT before reading FILE"),
                &mut output,
            );
            assert_eq!(answer.exit, 2);
            assert_eq!(answer.stdout, "");
            assert_eq!(answer.stderr, usage("OUT is inside a journeys root"));
            assert_eq!(snapshot(&output), before, "usage writes nothing and leaves both roots untouched");
        }
    }
}
