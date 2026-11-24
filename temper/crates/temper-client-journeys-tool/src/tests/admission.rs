//! Saved recordings are proved and printed without placeholders (temper/docs/design/client/web/journeys.md, 6).
use super::{Memory, OUT, build, print_at, print_fixture, snapshot};
use crate as tool;
use std::path::{Component, Path, PathBuf};
use temper_client_journeys_kit::{
    self as fixtures, TuiWorld, absent, fixture_with_marks_and_secret, tui_approval_recording, tui_recording,
    tui_written,
};
use temper_client_shell_web::journey;
use temper_client_walk_lines::{
    layout::{TUI, WEB},
    print as web_print,
};

#[test]
fn a_refused_tui_print_writes_nothing_even_over_an_existing_print() {
    use temper_client_shell_tui::journey as tj;
    let mut output = Memory::default();
    let out = Path::new(OUT);
    let recording = tui_recording();
    print_fixture(&mut output, "kept", &recording, &[]).expect("previous good print");
    let mut lines = tj::lines(&recording).expect("fixture");
    let mut changed = 0;
    for line in &mut lines {
        if let tj::Line::End(print) = line {
            *print = Some(b"different live print".as_slice().into());
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "one end proof changed");
    for (recording, given, place) in [
        (tui_written(&lines), Vec::new(), "proof"),
        (recording.into_vec(), vec![Box::<[u8]>::from(b"tui-dark".as_slice())], "printed"),
        (tui_frame_original(), vec![Box::<[u8]>::from(b"private-frame-marker".as_slice())], "frame"),
    ] {
        for name in ["new", "kept"] {
            let before = snapshot(&output);
            let refused = print_fixture(&mut output, name, &recording, &given);
            let refused_place = match refused {
                Err(tool::Refusal::TuiProof { refusal: tj::Refusal::Differs { .. }, .. }) => "proof",
                Err(tool::Refusal::TuiFound {
                    found: tj::Found { place: tj::Place::Printed, what: tj::What::Original, .. },
                    file: Some(_),
                    ..
                }) => "printed",
                Err(tool::Refusal::TuiFound {
                    found: tj::Found { place: tj::Place::Frame { .. }, what: tj::What::Original, .. },
                    ..
                }) => "frame",
                other => panic!("unexpected refusal: {other:?}"),
            };
            assert_eq!(refused_place, place);
            assert!(snapshot(&output) == before, "{place} refusal leaves every file untouched");
            assert!(!output.nodes.contains_key(&out.join(TUI).join("new")), "refusal creates no directory");
        }
    }
}

fn tui_frame_original() -> Vec<u8> {
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::with_box_log("pri\u{200b}vate-frame-marker&".as_bytes());
    world.report(Report::Resize(skein_ui_screen::Size { rows: 0, columns: 0 }));
    world.save().into_vec()
}

#[test]
fn browser_pages_are_the_redacted_replays_print_with_resolving_assets() {
    let mut output = Memory::default();
    let out = Path::new(OUT);
    let secret = ["private", "-link"].concat().into_bytes();
    let given = ["private", "-report"].concat().into_bytes();
    let mut lines = journey::lines(&fixture_with_marks_and_secret(Some(secret.clone().into()))).expect("fixture");
    let at = lines.iter().position(|line| matches!(line, journey::Line::Deadline(_))).expect("deadline");
    let mut quoted = Vec::new();
    journey::Line::Secret(given.clone().into()).write(&mut quoted);
    let answer = [b"heard answered 999 200 ".as_slice(), quoted.strip_prefix(b"secret ").expect("quoted")].concat();
    lines.insert(at, journey::lines(&answer).expect("answer").remove(0));
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    let printed =
        print_fixture(&mut output, "journey.recording", &recording, &[given.clone().into()]).expect("print browser");
    assert_eq!(printed.name, "journey");
    assert_eq!((printed.link, printed.given), (1, 1));
    let root = out.join(WEB);
    let directory = root.join("journey");
    assert!(!output.nodes.contains_key(&directory.join("recording")), "only the pages are written");
    let redacted = journey::redact(&recording, &[given.clone().into()]).expect("redact");
    let mut replay = journey::Replay::new(&redacted.recording).expect("replay");
    let mut marks = Vec::new();
    while let Some(mark) = replay.next().expect("replay holds") {
        marks.push(web_print::Mark {
            html: mark.page.html(),
            heading: web_print::heading(mark.tree),
            acted: mark.acted,
            lead: mark.lead,
        });
    }
    let pages = web_print::pages(&marks);
    assert!(pages.len() >= 2, "fixture has marks");
    for page in pages {
        assert_eq!(output.read(&directory.join(page.name)), page.document.as_bytes());
    }
    for file in ["skein-ui.css", "journey.js", "overlay.css"] {
        assert!(output.is_file(&root.join(file)));
    }
    assert_eq!(output.read(&root.join("overlay.css")).len(), 0);
    for stylesheet in web_print::STYLESHEETS {
        let sheet = relative_file(&directory, stylesheet);
        assert!(sheet.starts_with(out), "stylesheet resolves within OUT");
        let mut rest = std::str::from_utf8(output.read(&sheet)).expect("linked stylesheet copied");
        while let Some((_, after)) = rest.split_once("url(") {
            let (url, after) = after.split_once(')').expect("CSS url closes");
            rest = after;
            let url = url.trim().trim_matches(['\'', '"']);
            if url.starts_with(['/', '#']) || url.contains(':') {
                continue;
            }
            let path = relative_file(sheet.parent().expect("sheet directory"), url);
            assert!(path.starts_with(out) && output.is_file(&path), "relative CSS url resolves within OUT");
        }
    }
    for bytes in output.nodes.values().flatten() {
        absent(bytes, &secret, "output");
        absent(bytes, &given, "output");
    }
    output.put(&directory.join("99.html"), b"old mark");
    output.put(&directory.join("99.txt"), "person’s text");
    output.directory(&directory.join("98.html"));
    print_fixture(&mut output, "journey.recording", &recording, &[given.into()]).expect("replace pages");
    assert!(!output.nodes.contains_key(&directory.join("99.html")), "pages replaced whole");
    assert_eq!(output.read(&directory.join("99.txt")), "person’s text".as_bytes(), "web output keeps non-HTML files");
    assert_eq!(output.nodes.get(&directory.join("98.html")), Some(&None), "only regular numbered pages are removed");
}

fn relative_file(directory: &Path, reference: &str) -> PathBuf {
    let mut path = directory.to_owned();
    for component in Path::new(reference).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::ParentDir => assert!(path.pop(), "relative link has a parent"),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => panic!("expected a relative link"),
        }
    }
    path
}

#[test]
fn tui_print_keeps_the_recorded_build_ports_and_approval_code() {
    use temper_client_shell_tui::journey as tj;
    let mut output = Memory::default();
    let out = Path::new(OUT);
    let recording = tui_approval_recording();
    let printed = print_fixture(&mut output, "approval.recording", &recording, &[]).expect("print TUI");
    assert_eq!((printed.approval, printed.credential, printed.bearer), (1, 1, 1));
    let root = out.join(TUI);
    let directory = root.join("approval");
    assert!(!output.nodes.contains_key(&out.join(WEB)), "only the TUI root is written");
    assert!(!output.nodes.contains_key(&directory.join("recording")));
    for file in ["theme.css", "tui-journey.js", "overlay.css"] {
        assert!(output.is_file(&root.join(file)));
    }
    assert_eq!(output.read(&root.join("overlay.css")).len(), 0);
    let redacted = tj::redact(&recording, &[]).expect("redact");
    let mut replay = tj::Replay::new(&tj::prove(&redacted.recording).expect("proof")).expect("replay");
    let mut marks = Vec::new();
    while let Some(mark) = replay.next().expect("replay holds") {
        marks.push(mark);
    }
    let expected = tool::tui::pages("approval", temper_client_tui::Scheme::Dark, &marks);
    for page in expected {
        assert_eq!(output.read(&directory.join(page.name)), page.document.as_bytes());
    }
    let text: Vec<_> = output
        .nodes
        .iter()
        .filter(|(path, _)| path.starts_with(&directory))
        .flat_map(|(_, bytes)| bytes.iter().flatten().copied())
        .collect();
    for original in [b"1.2.3 (abcdef123456-dirty)".as_slice(), b"34073", b"WDJB-MJHT"] {
        assert!(skein_lib::bytes::find(&text, original).is_some(), "recorded build, port and shown code remain");
    }
}

#[test]
fn layouts_are_refused_before_proof_and_leave_output_untouched() {
    let mut output = Memory::default();
    let out = Path::new(OUT);
    let recording = fixtures::fixture_recording();
    let mut build = build();
    build.wire -= 1;
    let refusal = print_at(&mut output, out, "browser.recording", &recording, &[], &build).expect_err("wire mismatch");
    assert!(matches!(refusal, tool::Refusal::Layout { file, recorded, read }
        if file == "browser.recording" && recorded == format!("at jig-client-wire {}", build.wire + 1)
        && read == format!("this build reads jig-client-wire {}", build.wire)));
    assert_eq!(snapshot(&output), snapshot(&Memory::default()));
    output.put(&out.join("kept"), b"person's file");
    let before = snapshot(&output);
    build.temper = "0.0.0 (aaaaaaaaaaaa)".into();
    let refusal = print_at(&mut output, out, "terminal", &tui_recording(), &[], &build).expect_err("build mismatch");
    assert!(matches!(refusal, tool::Refusal::Layout { file, recorded, read }
        if file == "terminal" && recorded == "by temper 1.2.3 (abcdef123456-dirty)"
        && read == "this is temper 0.0.0 (aaaaaaaaaaaa)"));
    assert_eq!(snapshot(&output), before);
}

fn capture_recording() -> Box<[u8]> {
    use fixtures::web::{Capture, managed_capture};
    use skein_ui::{Act, Role};
    let mut capture = managed_capture(Capture::new());
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    capture.save()
}

#[test]
fn a_capture_keeps_its_start_through_print() {
    let recording = capture_recording();
    let mut output = Memory::default();
    print_fixture(&mut output, "capture", &recording, &[]).expect("redact, prove and scan");
    let header = b"journey 2\nseed 11\nclocks 0 0\nprimary control\n";
    assert!(recording.starts_with(header), "capture keeps the start facts");
    let redacted = journey::redact(&recording, &[]).expect("redact capture");
    let proved = journey::prove(&redacted.recording).expect("prove capture");
    assert!(proved.starts_with(header), "print's redaction and proof keep the start facts");
    let directory = Path::new(OUT).join(WEB).join("capture");
    assert_eq!(
        output.nodes.keys().filter(|path| path.parent() == Some(&directory)).count(),
        2,
        "the press mark and end print"
    );
}

#[test]
fn two_captures_and_their_printed_pages_are_identical() {
    let first = capture_recording();
    let second = capture_recording();
    assert!(first == second, "fresh captures of the same scripted walk have identical bytes");
    let mut first_output = Memory::default();
    let mut second_output = Memory::default();
    print_fixture(&mut first_output, "capture", &first, &[]).expect("first print");
    print_fixture(&mut second_output, "capture", &second, &[]).expect("second print");
    assert_eq!(snapshot(&first_output), snapshot(&second_output), "identical printed pages");
}
