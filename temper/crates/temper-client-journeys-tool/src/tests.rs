pub(super) use fixtures::{TuiWorld, tui_approval_reply};
use fixtures::{absent, fixture_recording, fixture_with_marks_and_secret, fixture_with_secret, stale_recording};
use temper_client_journeys_kit as fixtures;

use crate::JOURNEYS;
use skein_scratch::Scratch;
use std::{fs, path::Path};
use temper_client_shell_web::journey;

fn print_fixture(
    out: &Path,
    name: &str,
    recording: &[u8],
    given: &[Box<[u8]>],
) -> Result<crate::Printed, crate::Refusal> {
    crate::print(
        std::ffi::OsStr::new(name),
        recording,
        given,
        out,
        &crate::Build { wire: jig_client_protocol_wire::VERSION, temper: "1.2.3 (abcdef123456-dirty)".into() },
    )
}

fn replay_pages(recording: &[u8]) -> Vec<crate::print::Printed> {
    crate::printed_replay(journey::Replay::new(recording).expect("fixture replay")).expect("fixture replays")
}

#[test]
fn an_unreadable_recording_is_refused_by_print() {
    let scratch = Scratch::new("temper-client-journeys-unreadable");
    assert!(matches!(
        print_fixture(scratch.path(), "welcome", b"not a recording\n", &[]),
        Err(crate::Refusal::Proof { name, refusal: journey::Refusal::Unreadable(_) }) if name == "welcome"
    ));
    assert!(!scratch.path().join(temper_client_walk_lines::layout::WEB).join("welcome").exists());
}

#[test]
fn the_overlay_is_empty() {
    let overlay = fs::read(Path::new(JOURNEYS).join("overlay.css")).expect("committed overlay.css");
    assert!(
        overlay.is_empty(),
        "overlay.css holds a request: move its rules into temper.css or task-pages.css, and empty it"
    );
}
#[test]
fn every_journey_has_its_description() {
    descriptions(Path::new(JOURNEYS)).expect("committed journey descriptions");
}
#[test]
fn the_tui_overlay_is_empty() {
    let overlay = fs::read(Path::new(crate::TUI_JOURNEYS).join("overlay.css")).expect("committed TUI overlay.css");
    assert!(overlay.is_empty(), "TUI overlay.css holds a request; move its rules into the theme and empty it");
}

#[test]
fn every_tui_journey_has_its_description() {
    descriptions(Path::new(crate::TUI_JOURNEYS)).expect("committed TUI journey descriptions");
}

fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
    fn visit(
        root: &Path,
        directory: &Path,
        files: &mut std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>>,
    ) {
        for entry in fs::read_dir(directory).expect("snapshot directory") {
            let entry = entry.expect("snapshot entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("entry beneath root").to_owned();
            if entry.file_type().expect("snapshot type").is_dir() {
                files.insert(relative, None);
                visit(root, &path, files);
            } else {
                files.insert(relative, Some(fs::read(path).expect("snapshot file")));
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn a_refused_print_leaves_its_root_as_it_was() {
    let scratch = Scratch::new("temper-client-journeys-refused");
    let recording = fixture_recording();
    let unproved = journey::prove(&recording).expect("strip live prints");
    for recording in [unproved, stale_recording(&recording)] {
        let before = snapshot(scratch.path());
        assert!(print_fixture(scratch.path(), "welcome", &recording, &[]).is_err(), "recording refused");
        assert!(snapshot(scratch.path()) == before, "a refused print leaves the root empty");
    }
    let missing = scratch.path().join("missing");
    assert!(print_fixture(&missing, "welcome", &stale_recording(&recording), &[]).is_err(), "stale recording refused");
    assert!(!missing.exists(), "a refused print makes no root");
}

#[test]
fn a_refused_print_keeps_the_journey_of_its_name() {
    let scratch = Scratch::new("temper-client-journeys-kept");
    let recording = fixture_recording();
    print_fixture(scratch.path(), "welcome", &recording, &[]).expect("print fixture journey");
    fs::write(
        scratch.path().join(temper_client_walk_lines::layout::WEB).join("welcome/README.md"),
        "The person's description.\n",
    )
    .expect("description");
    fs::write(scratch.path().join(temper_client_walk_lines::layout::WEB).join("welcome/99.html"), "An older page.\n")
        .expect("older page");
    fs::write(scratch.path().join("overlay.css"), "#app { color: red; }\n").expect("overlay");
    fs::write(scratch.path().join("unrelated"), "Another file.\n").expect("unrelated file");
    let before = snapshot(scratch.path());
    assert!(
        print_fixture(scratch.path(), "welcome", &stale_recording(&recording), &[]).is_err(),
        "stale recording refused"
    );
    assert!(snapshot(scratch.path()) == before, "a refused print keeps every directory and file byte for byte");
}

fn write_lines(lines: &[journey::Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes.into()
}

fn heard_answer(value: &[u8]) -> journey::Line {
    let mut quoted = Vec::new();
    journey::Line::Secret(value.into()).write(&mut quoted);
    let quoted = quoted.strip_prefix(b"secret ").expect("quoted bytes");
    journey::lines(&[b"heard answered 999 200 ".as_slice(), quoted].concat()).expect("fake answer").remove(0)
}

fn with_answer(value: &[u8]) -> (Box<[u8]>, u32) {
    let mut lines = journey::lines(&fixture_recording()).expect("fixture lines");
    let at = lines.iter().position(|line| matches!(line, journey::Line::Deadline(_))).expect("deadline");
    lines.insert(at, heard_answer(value));
    (write_lines(&lines), u32::try_from(at + 1).expect("fixture line"))
}

#[test]
fn print_takes_the_link_out_and_the_replay_holds() {
    let scratch = Scratch::new("journeys-link");
    let fake = ["fake", "-link-private"].concat().into_bytes();
    let recording = fixture_with_secret(Some(fake.clone().into()));
    let printed = print_fixture(scratch.path(), "welcome", &recording, &[]).expect("print redacted link");
    assert_eq!(printed.link, 1, "one link original");
    assert_eq!((printed.stored, printed.typed, printed.given), (0, 0, 0));
    assert!(printed.places >= 2, "secret and exchange replaced");
    let redacted = journey::redact(&recording, &[]).expect("redacted fixture");
    let pages = replay_pages(&redacted.recording);
    for page in pages {
        let bytes =
            fs::read(scratch.path().join(temper_client_walk_lines::layout::WEB).join("welcome").join(page.name))
                .expect("written page");
        absent(&bytes, &fake, "page");
    }
}

#[test]
fn print_takes_a_given_text_out() {
    let scratch = Scratch::new("journeys-given");
    let fake = ["private", "-heard-answer"].concat().into_bytes();
    let (recording, _) = with_answer(&fake);
    let printed =
        print_fixture(scratch.path(), "welcome", &recording, &[fake.clone().into()]).expect("print given text");
    assert_eq!((printed.given, printed.places), (1, 1));
    for bytes in snapshot(scratch.path()).values().flatten() {
        absent(bytes, &fake, "written file");
    }
}

#[test]
fn a_shape_in_a_heard_answer_refuses() {
    let scratch = Scratch::new("journeys-shape");
    let fake = ["s", "k", "-", "Ab0123456789_-cdefGHI"].concat();
    let (recording, line) = with_answer(fake.as_bytes());
    let before = snapshot(scratch.path());
    assert!(
        matches!(print_fixture(scratch.path(), "welcome", &recording, &[]),
        Err(crate::Refusal::Found { line: found, record: "heard", place: journey::Place::Field, what: journey::What::Shape(journey::Shape::Key), .. }) if found == line),
        "heard field shape refused"
    );
    assert!(snapshot(scratch.path()) == before, "refusal writes nothing");
}

#[test]
fn a_given_text_the_page_draws_refuses() {
    let scratch = Scratch::new("journeys-drawn");
    let before = snapshot(scratch.path());
    let result = print_fixture(scratch.path(), "welcome", &fixture_recording(), &[b"temper".as_slice().into()]);
    assert!(
        matches!(result, Err(crate::Refusal::FoundDrawn { turn: 1, what: journey::What::Original, .. })),
        "drawn hit precedes differing live print"
    );
    assert!(snapshot(scratch.path()) == before, "drawn refusal writes nothing");
}

#[test]
fn a_given_text_only_the_print_holds_refuses() {
    let scratch = Scratch::new("journeys-printed");
    let before = snapshot(scratch.path());
    {
        let recording = fixture_recording();
        let result = print_fixture(scratch.path(), "welcome", &recording, &[b"skein-ui.css".as_slice().into()]);
        assert!(
            matches!(result, Err(crate::Refusal::FoundPrinted { page, what: journey::What::Original, .. }) if page == "01.html"),
            "printed document is scanned"
        );
        assert!(snapshot(scratch.path()) == before, "printed refusal writes nothing");
    }
}

#[test]
fn a_refusal_keeps_the_journey_of_its_name() {
    let scratch = Scratch::new("journeys-existing-guard");
    print_fixture(scratch.path(), "welcome", &fixture_recording(), &[]).expect("initial journey");
    fs::write(scratch.path().join(temper_client_walk_lines::layout::WEB).join("welcome/README.md"), "Description")
        .expect("description");
    let before = snapshot(scratch.path());
    assert!(
        print_fixture(scratch.path(), "welcome", &fixture_recording(), &[b"skein-ui.css".as_slice().into()]).is_err(),
        "guard refuses replacement"
    );
    assert!(snapshot(scratch.path()) == before, "existing journey and neighbours kept");
}

#[test]
fn a_marked_walk_redacted_prints_the_same_pages() {
    let secret = ["walk", "-secret"].concat().into_bytes();
    let recording = fixture_with_marks_and_secret(Some(secret.clone().into()));
    let redacted = journey::redact(&recording, &[]).expect("redact host walk");
    assert_eq!(redacted.originals.count(journey::Kind::Link), 1);
    absent(&redacted.recording, &secret, "redacted host walk");
    let expected = replay_pages(&recording);
    let pages = replay_pages(&redacted.recording);
    assert!(pages.len() >= 2, "several marked pages compared");
    assert_eq!(pages.len(), expected.len(), "every page retained");
    for (page, expected) in pages.iter().zip(&expected) {
        assert_eq!(page.name, expected.name);
        assert_eq!(page.document, expected.document, "redaction preserves the printed page");
    }
}

#[test]
fn stale_replays_use_the_proofs_safe_refusal() {
    use temper_client_shell_tui::journey as tj;
    let web = crate::proof_refusal(
        "browser",
        journey::Refusal::Stale(journey::Stale {
            turn: 3,
            line: 17,
            recorded: "exchange private-recorded-bytes".into(),
            asked: "open private-asked-bytes".into(),
        }),
    );
    let tui = crate::tui::proof_refusal(
        "terminal",
        tj::Refusal::Stale(tj::Stale {
            turn: 5,
            line: 23,
            recorded: b"send private-recorded-bytes".as_slice().into(),
            asked: b"stream private-asked-bytes".as_slice().into(),
        }),
    );
    assert!(matches!(&web, crate::Refusal::Proof { refusal: journey::Refusal::Stale(why), .. }
        if why.recorded == "exchange" && why.asked == "open"));
    assert!(matches!(&tui, crate::Refusal::TuiProof { refusal: tj::Refusal::Stale(why), .. }
        if why.recorded.as_ref() == b"send" && why.asked.as_ref() == b"stream"));
    assert_eq!(
        crate::run::refusal(web),
        "browser is stale at turn 3, line 17: recorded exchange, asked open; record it again (client/web/journeys.md, 8)"
    );
    assert_eq!(
        crate::run::refusal(tui),
        "terminal is stale at turn 5, line 23: recorded send, asked stream; record it again (client/tui/journeys.md, 8)"
    );
}

fn descriptions(root: &Path) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|why| why.to_string())? {
        let entry = entry.map_err(|why| why.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|why| why.to_string())?.is_dir()
            && path.join("walk").is_file()
            && !fs::read_to_string(path.join("README.md")).is_ok_and(|text| !text.trim().is_empty())
        {
            return Err(format!("{} needs a README.md holding its description", path.display()));
        }
    }
    Ok(())
}

fn tui_sources(root: &Path) -> Result<(), String> {
    if !fs::read(root.join("overlay.css")).is_ok_and(|bytes| bytes.is_empty()) {
        return Err("TUI overlay.css holds a request; move its rules into the theme and empty it".into());
    }
    descriptions(root)?;
    Ok(())
}

#[test]
fn tui_sources_refuse_an_overlay_and_missing_or_empty_descriptions() {
    let scratch = Scratch::new("journeys-tui-sources");
    fs::create_dir(scratch.path().join("described")).expect("journey");
    fs::write(scratch.path().join("described/walk"), b"walk 2\n").expect("walk");
    fs::write(scratch.path().join("overlay.css"), b"").expect("empty overlay");
    for description in [None, Some(""), Some(" \n")] {
        if let Some(description) = description {
            fs::write(scratch.path().join("described/README.md"), description).expect("description");
        }
        assert_eq!(
            tui_sources(scratch.path()).expect_err("description refused"),
            format!("{} needs a README.md holding its description", scratch.path().join("described").display())
        );
    }
    fs::write(scratch.path().join("described/README.md"), "A journey.").expect("description");
    assert_eq!(tui_sources(scratch.path()), Ok(()));
    fs::write(scratch.path().join("overlay.css"), b"x").expect("overlay request");
    assert!(tui_sources(scratch.path()).expect_err("overlay refused").contains("overlay.css"));
}

#[test]
fn a_files_first_component_names_its_print_or_falls_back() {
    use std::ffi::OsStr;
    for (file, name) in [
        ("journey.recording", "journey"),
        ("recording-1767690000", "recording-1767690000"),
        ("My File.txt", "recorded"),
        (".recording", "recorded"),
        ("1-walk.txt", "recorded"),
        ("a_b.txt", "recorded"),
        ("/scratch/walk.extra.recording", "walk"),
    ] {
        assert_eq!(crate::recording_name(OsStr::new(file)), name);
    }
}
