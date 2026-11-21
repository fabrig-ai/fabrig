//! Saved recordings are proved and printed without placeholders (draft/archive/journey-walks.md, 4.3).
use fixtures::{
    TuiWorld, absent, fixture_with_marks_and_secret, tree_bytes, tui_approval_recording, tui_recording, tui_written,
};
use skein_scratch::Scratch;
use std::{ffi::OsStr, fs, path::Path};
use temper_client_journeys_kit as fixtures;
use temper_client_journeys_tool as tool;
use temper_client_shell_web::journey;
use temper_client_walk_lines::{
    layout::{TUI, WEB},
    print as web_print,
};

fn build() -> tool::Build {
    tool::Build { wire: jig_client_protocol_wire::VERSION, temper: "1.2.3 (abcdef123456-dirty)".into() }
}
fn print(out: &Path, file: &str, recording: &[u8], given: &[Box<[u8]>]) -> Result<tool::Printed, tool::Refusal> {
    tool::print(OsStr::new(file), recording, given, out, &build())
}

#[test]
fn a_refused_tui_print_writes_nothing_even_over_an_existing_print() {
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-tui-refused");
    let out = scratch.path().join("out");
    let recording = tui_recording();
    print(&out, "kept", &recording, &[]).expect("previous good print");
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
            let before = tree_bytes(scratch.path());
            let refused = print(&out, name, &recording, &given);
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
            assert!(tree_bytes(scratch.path()) == before, "{place} refusal leaves every file untouched");
            assert!(!out.join(TUI).join("new").exists(), "refusal creates no directory");
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
    let scratch = Scratch::new("journeys-browser-print");
    let out = scratch.path().join("out");
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
    let printed = print(&out, "journey.recording", &recording, &[given.clone().into()]).expect("print browser");
    assert_eq!(printed.name, "journey");
    assert_eq!((printed.link, printed.given), (1, 1));
    let root = out.join(WEB);
    let directory = root.join("journey");
    assert!(!directory.join("recording").exists(), "only the pages are written");
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
        assert_eq!(fs::read(directory.join(page.name)).expect("page"), page.document.as_bytes());
    }
    for file in ["skein-ui.css", "journey.js", "overlay.css"] {
        assert!(root.join(file).is_file());
    }
    assert_eq!(fs::read(root.join("overlay.css")).expect("overlay").len(), 0);
    for stylesheet in web_print::STYLESHEETS {
        let sheet = fs::canonicalize(directory.join(stylesheet)).expect("linked stylesheet copied");
        assert!(sheet.starts_with(&out), "stylesheet resolves within OUT");
        let css = fs::read_to_string(&sheet).expect("CSS");
        let mut rest = css.as_str();
        while let Some((_, after)) = rest.split_once("url(") {
            let (url, after) = after.split_once(')').expect("CSS url closes");
            rest = after;
            let url = url.trim().trim_matches(['\'', '"']);
            if url.starts_with(['/', '#']) || url.contains(':') {
                continue;
            }
            let path = sheet.parent().expect("sheet directory").join(url);
            assert!(fs::canonicalize(path).expect("url file copied").starts_with(&out));
        }
    }
    for bytes in tree_bytes(&out).values().flatten() {
        absent(bytes, &secret, "output");
        absent(bytes, &given, "output");
    }
    fs::write(directory.join("99.html"), b"old mark").expect("old page");
    print(&out, "journey.recording", &recording, &[given.into()]).expect("replace pages");
    assert!(!directory.join("99.html").exists(), "pages replaced whole");
}

#[test]
fn tui_print_keeps_the_recorded_build_ports_and_approval_code() {
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-tui-print");
    let out = scratch.path().join("out");
    let recording = tui_approval_recording();
    let printed = print(&out, "approval.recording", &recording, &[]).expect("print TUI");
    assert_eq!((printed.approval, printed.credential, printed.bearer), (1, 1, 1));
    let root = out.join(TUI);
    let directory = root.join("approval");
    assert!(!out.join(WEB).exists(), "only the TUI root is written");
    assert!(!directory.join("recording").exists());
    for file in ["theme.css", "tui-journey.js", "overlay.css"] {
        assert!(root.join(file).is_file());
    }
    assert_eq!(fs::read(root.join("overlay.css")).expect("overlay").len(), 0);
    let redacted = tj::redact(&recording, &[]).expect("redact");
    let mut replay = tj::Replay::new(&tj::prove(&redacted.recording).expect("proof")).expect("replay");
    let mut marks = Vec::new();
    while let Some(mark) = replay.next().expect("replay holds") {
        marks.push(mark);
    }
    let expected = tool::tui::pages("approval", temper_client_tui::Scheme::Dark, &marks);
    for page in expected {
        assert_eq!(fs::read(directory.join(page.name)).expect("printed frame"), page.document.as_bytes());
    }
    let text: Vec<_> = tree_bytes(&directory).values().flatten().flatten().copied().collect();
    for original in [b"1.2.3 (abcdef123456-dirty)".as_slice(), b"34073", b"WDJB-MJHT"] {
        assert!(skein_lib::bytes::find(&text, original).is_some(), "recorded build, port and shown code remain");
    }
}

#[test]
fn layouts_are_refused_before_proof_and_leave_output_untouched() {
    let scratch = Scratch::new("journeys-layout");
    let out = scratch.path().join("out");
    let recording = fixtures::fixture_recording();
    let mut build = build();
    build.wire -= 1;
    let refusal =
        tool::print(OsStr::new("browser.recording"), &recording, &[], &out, &build).expect_err("wire mismatch");
    assert!(matches!(refusal, tool::Refusal::Layout { file, recorded, read }
        if file == "browser.recording" && recorded == format!("at jig-client-wire {}", build.wire + 1)
        && read == format!("this build reads jig-client-wire {}", build.wire)));
    assert!(!out.exists());
    fs::create_dir(&out).expect("existing output");
    fs::write(out.join("kept"), b"person's file").expect("existing file");
    let before = tree_bytes(&out);
    build.temper = "0.0.0 (aaaaaaaaaaaa)".into();
    let refusal = tool::print(OsStr::new("terminal"), &tui_recording(), &[], &out, &build).expect_err("build mismatch");
    assert!(matches!(refusal, tool::Refusal::Layout { file, recorded, read }
        if file == "terminal" && recorded == "by temper 1.2.3 (abcdef123456-dirty)"
        && read == "this is temper 0.0.0 (aaaaaaaaaaaa)"));
    assert_eq!(tree_bytes(&out), before);
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
    let scratch = Scratch::new("journeys-capture-print");
    print(scratch.path(), "capture", &recording, &[]).expect("redact, prove and scan");
    let header = b"journey 2\nseed 11\nclocks 0 0\nprimary control\n";
    assert!(recording.starts_with(header), "capture keeps the start facts");
    let redacted = journey::redact(&recording, &[]).expect("redact capture");
    let proved = journey::prove(&redacted.recording).expect("prove capture");
    assert!(proved.starts_with(header), "print's redaction and proof keep the start facts");
    let directory = scratch.path().join(WEB).join("capture");
    assert_eq!(fs::read_dir(directory).expect("printed pages").count(), 2, "the press mark and end print");
}

#[test]
fn two_captures_and_their_printed_pages_are_identical() {
    let first = capture_recording();
    let second = capture_recording();
    assert!(first == second, "fresh captures of the same scripted walk have identical bytes");
    let scratch = Scratch::new("journeys-capture-twice");
    for (name, recording) in [("first", first), ("second", second)] {
        print(&scratch.path().join(name), "capture", &recording, &[]).expect("print capture");
    }
    assert_eq!(
        tree_bytes(&scratch.path().join("first")),
        tree_bytes(&scratch.path().join("second")),
        "identical printed pages"
    );
}
