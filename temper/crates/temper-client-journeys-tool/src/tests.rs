use crate::{
    JOURNEYS,
    print::{Mark, SCRIPT, beside, pages},
};
use skein_lib::{Time, Wall};
use skein_scratch::Scratch;
use skein_ui::NodeId;
use skein_ui_web::mapping::Primary;
use std::{fs, path::Path};
use temper_client_shell_web::journey;

#[test]
fn the_committed_journeys_are_their_print() {
    let scratch = Scratch::new("temper-client-journeys-committed");
    let committed = Path::new(JOURNEYS);
    let stale = crate::reprint(committed, scratch.path())
        .expect("reprint committed journeys; run cargo run -p temper-client-journeys");
    assert!(stale.is_empty(), "stale journeys: {stale:?}; run cargo run -p temper-client-journeys");
    let committed_pages = printed_files(committed);
    let reprinted_pages = printed_files(scratch.path());
    for path in committed_pages.union(&reprinted_pages) {
        let expected = fs::read(committed.join(path)).ok();
        let actual = fs::read(scratch.path().join(path)).ok();
        assert!(
            expected.is_some() && actual.is_some(),
            "{} is missing or unreadable; run cargo run -p temper-client-journeys",
            path.display()
        );
        assert_eq!(expected, actual, "{} differs; run cargo run -p temper-client-journeys", path.display());
    }
    for file in ["skein-ui.css", "journey.js"] {
        let reprinted = fs::read(scratch.path().join(file)).expect("reprinted file beside the journeys");
        assert_eq!(
            fs::read(committed.join(file)).ok().as_deref(),
            Some(reprinted.as_slice()),
            "{file} differs; run cargo run -p temper-client-journeys"
        );
    }
    let tui = Path::new(crate::TUI_JOURNEYS);
    tui_sources(tui).expect("committed TUI sources");
    let out = scratch.path().join("tui");
    let stale = crate::tui::reprint(tui, &out).expect("reprint TUI; run cargo run -p temper-client-journeys");
    assert!(stale.is_empty(), "stale TUI journeys: {stale:?}; run cargo run -p temper-client-journeys");
    let expected = printed_files(tui);
    let reprinted = printed_files(&out);
    for path in expected.union(&reprinted) {
        assert_eq!(
            fs::read(tui.join(path)).ok(),
            fs::read(out.join(path)).ok(),
            "TUI {} differs; run cargo run -p temper-client-journeys",
            path.display()
        );
    }
    for file in ["theme.css", "tui-journey.js", "overlay.css"] {
        assert!(
            fs::read(tui.join(file)).expect("committed beside") == fs::read(out.join(file)).expect("reprinted beside"),
            "TUI {file} differs; run cargo run -p temper-client-journeys"
        );
    }
}

fn printed_files(root: &Path) -> std::collections::BTreeSet<std::path::PathBuf> {
    let mut files = std::collections::BTreeSet::new();
    for entry in fs::read_dir(root).expect("journeys directory") {
        let entry = entry.expect("journey entry");
        if !entry.file_type().expect("journey type").is_dir() {
            continue;
        }
        for page in fs::read_dir(entry.path()).expect("journey directory") {
            let page = page.expect("page entry");
            if crate::printed_page(&page.path()) || crate::tui::printed_file(&page.path()) {
                files.insert(Path::new(&entry.file_name()).join(page.file_name()));
            }
        }
    }
    files
}

#[test]
fn a_name_outside_the_pattern_is_refused() {
    let scratch = Scratch::new("temper-client-journeys-name");
    for name in ["Welcome", "1-welcome", "welcome_set", ""] {
        assert!(!crate::named(name), "invalid name: {name:?}");
        assert!(
            matches!(crate::add(scratch.path(), name, b"", &[]), Err(crate::Refusal::Name(refused)) if refused == name)
        );
    }
    assert!(crate::named("welcome-set-up"));
    assert!(matches!(crate::add(scratch.path(), "welcome-set-up", b"", &[]), Err(crate::Refusal::Proof { .. })));
}

#[test]
fn an_unreadable_recording_is_refused_by_add() {
    let scratch = Scratch::new("temper-client-journeys-unreadable");
    assert!(matches!(
        crate::add(scratch.path(), "welcome", b"not a recording\n", &[]),
        Err(crate::Refusal::Proof { name, refusal: journey::Refusal::Unreadable(_) }) if name == "welcome"
    ));
    assert!(!scratch.path().join("welcome").exists());
}

fn mark(html: &str, heading: &str, acted: Option<u32>) -> Mark {
    Mark {
        html: html.to_owned(),
        heading: heading.as_bytes().to_vec(),
        acted: acted.into_iter().map(NodeId).collect(),
        lead: journey::Lead::Press,
    }
}

#[test]
fn a_page_breaks_inside_each_tag_before_its_close() {
    let printed = pages(&[mark("<div data-node=\"1\"><p data-node=\"2\">Hi</p></div>", "", None)]);
    let document = &printed[0].document;
    let body = document.split_once("<body\n>").expect("body starts").1.split_once("</body\n>").expect("body ends").0;
    assert_eq!(body, "<div id=\"app\"\n><p\n>Hi</p\n></div\n>");
    assert!(document.starts_with("<!doctype html>\n<html lang=\"en\"\n>"));
    assert!(document.ends_with("</html\n>\n"));
}

#[test]
fn the_acted_control_leads_to_the_next_page() {
    let mut first = mark(
        "<div data-node=\"1\"><h1 data-node=\"2\">Welcome</h1><button data-node=\"3\">Set up temper</button><a data-node=\"5\">Begin</a></div>",
        "Welcome",
        Some(3),
    );
    first.acted.push(NodeId(5));
    let printed = pages(&[first, mark("<div data-node=\"4\">Forge</div>", "Forge", None)]);
    assert!(printed[0].document.contains("<button data-journey-next=\"02.html\"\n>Set up temper</button\n>"));
    assert!(printed[0].document.contains("<a data-journey-next=\"02.html\"\n>Begin</a\n>"));
    for page in printed {
        assert!(!page.document.contains("data-node"));
    }
}

#[test]
fn a_page_names_its_neighbours() {
    let printed = pages(&[mark("<div></div>", "", None), mark("<div></div>", "", None), mark("<div></div>", "", None)]);
    assert_eq!(printed.iter().map(|page| page.name.as_str()).collect::<Vec<_>>(), ["01.html", "02.html", "03.html"]);
    assert!(!printed[0].document.contains("rel=\"prev\""));
    assert!(printed[0].document.contains("<link rel=\"next\" href=\"02.html\"\n>"));
    assert!(printed[1].document.contains("<link rel=\"prev\" href=\"01.html\"\n>"));
    assert!(printed[1].document.contains("<link rel=\"next\" href=\"03.html\"\n>"));
    assert!(printed[2].document.contains("<link rel=\"prev\" href=\"02.html\"\n>"));
    assert!(!printed[2].document.contains("rel=\"next\""));
}

#[test]
fn a_dialog_is_printed_open() {
    let printed = pages(&[mark(
        "<div data-node=\"1\"><dialog data-node=\"2\" aria-label=\"Choice\">Choose</dialog></div>",
        "",
        None,
    )]);
    assert!(printed[0].document.contains("<dialog open aria-label=\"Choice\"\n>Choose</dialog\n>"));
}

#[test]
fn the_title_is_the_main_heading_then_temper() {
    let printed = pages(&[mark("<div></div>", "Welcome & more", None), mark("<div></div>", "", None)]);
    assert!(printed[0].document.contains("<title\n>Welcome &amp; more · temper</title\n>"));
    assert!(printed[1].document.contains("<title\n>temper</title\n>"));
}

#[test]
fn the_stylesheets_resolve_in_the_checkout() {
    let printed = pages(&[mark("<div></div>", "", None)]);
    let stylesheets: Vec<&str> = printed[0]
        .document
        .split("<link rel=\"stylesheet\" href=\"")
        .skip(1)
        .map(|rest| rest.split_once('"').expect("stylesheet href closes").0)
        .collect();
    assert_eq!(
        stylesheets,
        [
            "../skein-ui.css",
            "../../../../../../crates/temper-client-shell-web/temper.css",
            "../../../../../../crates/temper-client-shell-web/task-pages.css",
            "../overlay.css",
        ]
    );
    for stylesheet in &stylesheets[1..3] {
        // Neither the journeys root nor a journey need exist before the first print.
        let mut path = Path::new(JOURNEYS).join("example");
        for component in Path::new(stylesheet).components() {
            if component == std::path::Component::ParentDir {
                assert!(path.pop(), "stylesheet stays in the checkout");
            } else {
                path.push(component);
            }
        }
        assert!(path.is_file(), "stylesheet exists: {}", path.display());
    }
}

#[test]
fn beside_writes_the_kits_and_the_journeys_files_and_keeps_an_overlay() {
    let scratch = Scratch::new("temper-client-journeys-beside");
    beside(scratch.path()).expect("write beside the journeys");
    let overlay = scratch.path().join("overlay.css");
    let script = scratch.path().join("journey.js");
    let stylesheet = scratch.path().join("skein-ui.css");
    assert_eq!(fs::read(&overlay).expect("empty overlay"), b"");
    assert_eq!(fs::read_to_string(&script).expect("script"), SCRIPT);
    assert_eq!(fs::read(&stylesheet).expect("stylesheet"), skein_ui_bundle::STYLESHEET);
    let rule = b"#app { color: red; }\n";
    fs::write(&overlay, rule).expect("person's overlay");
    fs::write(&script, "old script").expect("old script");
    fs::write(&stylesheet, "old stylesheet").expect("old stylesheet");
    beside(scratch.path()).expect("rewrite beside the journeys");
    assert_eq!(fs::read(&overlay).expect("kept overlay"), rule);
    assert_eq!(fs::read_to_string(&script).expect("rewritten script"), SCRIPT);
    assert_eq!(fs::read(&stylesheet).expect("rewritten stylesheet"), skein_ui_bundle::STYLESHEET);
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
    for name in crate::names(Path::new(JOURNEYS)).expect("committed journeys") {
        let path = Path::new(JOURNEYS).join(&name).join("README.md");
        assert!(
            fs::read_to_string(path).is_ok_and(|text| !text.trim().is_empty()),
            "{name} needs a README.md holding its description"
        );
    }
}
#[test]
fn a_reprint_keeps_a_journeys_readme() {
    let scratch = Scratch::new("temper-client-journeys-readme");
    let source = scratch.path().join("source");
    let out = scratch.path().join("out");
    crate::add(&source, "welcome", &fixture_recording(), &[]).expect("add fixture journey");
    let directory = out.join("welcome");
    fs::create_dir_all(&directory).expect("journey directory");
    let readme = directory.join("README.md");
    let description = b"The journey's description.\n";
    fs::write(&readme, description).expect("write description");
    fs::write(directory.join("99.html"), "old page").expect("old page");
    assert!(crate::reprint(&source, &out).expect("reprint").is_empty());
    assert_eq!(fs::read(readme).expect("description kept"), description);
    assert!(!directory.join("99.html").exists());
}

#[test]
fn a_submits_field_leads_on_by_its_act() {
    let mut first = mark("<div><form><input data-node=\"3\"></form></div>", "", Some(3));
    first.lead = journey::Lead::Submit;
    let printed = pages(&[first, mark("<div></div>", "", None)]);
    assert!(printed[0].document.contains("<input data-journey-next=\"02.html\" data-journey-act=\"submit\"\n>"));
}
#[test]
fn a_toggles_box_leads_on_by_its_act() {
    let mut first = mark("<div><input type=\"checkbox\" data-node=\"3\"></div>", "", Some(3));
    first.lead = journey::Lead::Toggle;
    let printed = pages(&[first, mark("<div></div>", "", None)]);
    assert!(
        printed[0]
            .document
            .contains("<input type=\"checkbox\" data-journey-next=\"02.html\" data-journey-act=\"toggle\"\n>")
    );
}

fn fixture_recording() -> Box<[u8]> {
    fixture_with_secret(None)
}

fn fixture_with_secret(secret: Option<Box<[u8]>>) -> Box<[u8]> {
    let facts = journey::Facts {
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: None,
        forges: Vec::new(),
        path: b"/welcome".as_slice().into(),
        secret,
        stored: Vec::new(),
    };
    let (mut host, mut page) = facts.start();
    let mut recorder = journey::Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, temper_client_shell_web::host::Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    recorder.save(&page)
}

fn stale_recording(recording: &[u8]) -> Box<[u8]> {
    let mut lines = journey::lines(recording).expect("fixture records");
    let mut changed = 0;
    for line in &mut lines {
        if let journey::Line::Deadline(deadline) = line {
            *deadline = Some(Time::ZERO);
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "the start turn's deadline");
    let mut out = Vec::new();
    for line in lines {
        line.write(&mut out);
    }
    out.into()
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
fn a_refused_add_leaves_its_root_as_it_was() {
    let scratch = Scratch::new("temper-client-journeys-refused");
    let recording = fixture_recording();
    let unproved = journey::prove(&recording).expect("strip live prints");
    for recording in [unproved, stale_recording(&recording)] {
        let before = snapshot(scratch.path());
        assert!(crate::add(scratch.path(), "welcome", &recording, &[]).is_err(), "recording refused");
        assert!(snapshot(scratch.path()) == before, "a refused add leaves the root empty");
    }
    let missing = scratch.path().join("missing");
    assert!(crate::add(&missing, "welcome", &stale_recording(&recording), &[]).is_err(), "stale recording refused");
    assert!(!missing.exists(), "a refused add makes no root");
}

#[test]
fn a_refused_add_keeps_the_journey_of_its_name() {
    let scratch = Scratch::new("temper-client-journeys-kept");
    let recording = fixture_recording();
    crate::add(scratch.path(), "welcome", &recording, &[]).expect("add fixture journey");
    fs::write(scratch.path().join("welcome/README.md"), "The person's description.\n").expect("description");
    fs::write(scratch.path().join("welcome/99.html"), "An older page.\n").expect("older page");
    fs::write(scratch.path().join("overlay.css"), "#app { color: red; }\n").expect("overlay");
    fs::write(scratch.path().join("unrelated"), "Another file.\n").expect("unrelated file");
    let before = snapshot(scratch.path());
    assert!(
        crate::add(scratch.path(), "welcome", &stale_recording(&recording), &[]).is_err(),
        "stale recording refused"
    );
    assert!(snapshot(scratch.path()) == before, "a refused add keeps every directory and file byte for byte");
}

#[test]
fn a_stale_reprint_leaves_its_pages_as_they_were() {
    let scratch = Scratch::new("temper-client-journeys-stale-reprint");
    let recording = fixture_recording();
    crate::add(scratch.path(), "welcome", &recording, &[]).expect("add fixture journey");
    fs::write(scratch.path().join("welcome/recording"), stale_recording(&recording)).expect("stale recording");
    let before = snapshot(scratch.path());
    let stale = crate::reprint(scratch.path(), scratch.path()).expect("reprint");
    assert!(matches!(stale.as_slice(), [(name, _)] if name == "welcome"), "reprint reports the stale journey");
    assert!(snapshot(scratch.path()) == before, "a stale reprint keeps every page byte for byte");
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

fn absent(bytes: &[u8], private: &[u8], place: &str) {
    assert!(skein_lib::bytes::find(bytes, private).is_none(), "original absent from {place}");
}

#[test]
fn add_takes_the_link_out_and_the_replay_holds() {
    let scratch = Scratch::new("journeys-link");
    let fake = ["fake", "-link-private"].concat().into_bytes();
    let recording = fixture_with_secret(Some(fake.clone().into()));
    let added = crate::add(scratch.path(), "welcome", &recording, &[]).expect("add redacted link");
    assert_eq!(added.link, 1, "one link original");
    assert_eq!((added.stored, added.typed, added.given), (0, 0, 0));
    assert!(added.places >= 2, "secret and exchange replaced");
    let saved = fs::read(scratch.path().join("welcome/recording")).expect("saved recording");
    absent(&saved, &fake, "recording");
    let lines = journey::lines(&saved).expect("saved lines");
    assert!(
        lines
            .iter()
            .any(|line| matches!(line, journey::Line::Secret(value) if value.as_ref() == vec![b'x'; fake.len()]))
    );
    let pages = crate::printed("welcome", &saved).expect("read replay").expect("replay holds");
    for page in pages {
        let bytes = fs::read(scratch.path().join("welcome").join(page.name)).expect("written page");
        absent(&bytes, &fake, "page");
    }
}

#[test]
fn add_takes_a_given_text_out() {
    let scratch = Scratch::new("journeys-given");
    let fake = ["private", "-heard-answer"].concat().into_bytes();
    let (recording, _) = with_answer(&fake);
    let added = crate::add(scratch.path(), "welcome", &recording, &[fake.clone().into()]).expect("add given text");
    assert_eq!((added.given, added.places), (1, 1));
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
        matches!(crate::add(scratch.path(), "welcome", &recording, &[]),
        Err(crate::Refusal::Found { line: found, record: "heard", place: journey::Place::Field, what: journey::What::Shape(journey::Shape::Key), .. }) if found == line),
        "heard field shape refused"
    );
    assert!(snapshot(scratch.path()) == before, "refusal writes nothing");
}

#[test]
fn a_given_text_the_page_draws_refuses() {
    let scratch = Scratch::new("journeys-drawn");
    let before = snapshot(scratch.path());
    let result = crate::add(scratch.path(), "welcome", &fixture_recording(), &[b"temper".as_slice().into()]);
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
    let mut differing = journey::lines(&fixture_recording()).expect("fixture lines");
    let mut count = 0;
    for line in &mut differing {
        if let journey::Line::End(html) = line {
            *html = Some(b"different live print".as_slice().into());
            count += 1;
        }
    }
    assert_eq!(count, 1, "changed end mark");
    for recording in [fixture_recording(), write_lines(&differing)] {
        let result = crate::add(scratch.path(), "welcome", &recording, &[b"skein-ui.css".as_slice().into()]);
        assert!(
            matches!(result, Err(crate::Refusal::FoundPrinted { page, what: journey::What::Original, .. }) if page == "01.html"),
            "printed document hit precedes proof refusal"
        );
        assert!(snapshot(scratch.path()) == before, "printed refusal writes nothing");
    }
}

#[test]
fn a_refusal_keeps_the_journey_of_its_name() {
    let scratch = Scratch::new("journeys-existing-guard");
    crate::add(scratch.path(), "welcome", &fixture_recording(), &[]).expect("initial journey");
    fs::write(scratch.path().join("welcome/README.md"), "Description").expect("description");
    let before = snapshot(scratch.path());
    assert!(
        crate::add(scratch.path(), "welcome", &fixture_recording(), &[b"skein-ui.css".as_slice().into()]).is_err(),
        "guard refuses replacement"
    );
    assert!(snapshot(scratch.path()) == before, "existing journey and neighbours kept");
}

#[test]
fn the_committed_journey_redacted_prints_the_same_pages() {
    let root = Path::new(JOURNEYS).join("welcome-playground");
    let recording = fs::read(root.join("recording")).expect("committed recording");
    let redacted = journey::redact(&recording, &[]).expect("redact committed journey");
    let pages = crate::printed("welcome-playground", &redacted.recording).expect("read journey").expect("replay");
    assert!(!pages.is_empty(), "committed pages checked");
    for page in pages {
        let expected = fs::read(root.join(page.name)).expect("committed page");
        assert!(expected == page.document.as_bytes(), "redacted page equals committed page");
    }
}

#[test]
fn a_setup_joins_only_the_named_steps() {
    let scratch = Scratch::new("journeys-setup-join");
    for name in ["base", "child"] {
        fs::create_dir(scratch.path().join(name)).expect("setup directory");
    }
    fs::write(
        scratch.path().join("base/setup"),
        b"setup 1\npress button \"First\"\nreload \"/base\"\nmark 1 \"Base mark\"\n",
    )
    .expect("base setup");
    fs::write(
        scratch.path().join("child/setup"),
        b"setup 1\nafter \"base\"\nsee \"Ready\"\nreload \"/child\"\nmark 2 \"Child mark\"\n",
    )
    .expect("child setup");
    let joined = crate::again::setup(scratch.path(), "child").expect("join").expect("setup");
    assert!(
        matches!(&joined.steps[..], [journey::SetupStep::Act { name, .. }, journey::SetupStep::See { text }] if name.as_ref() == b"First" && text.as_ref() == b"Ready")
    );
    assert_eq!(joined.reload.as_ref(), b"/child");
    assert_eq!(joined.marks, [journey::MarkWait { mark: 2, text: b"Child mark".as_slice().into() }]);
    assert!(crate::again::setup(scratch.path(), "missing").expect("optional setup").is_none());
}

#[test]
fn a_setups_after_refuses_chains_missing_and_self() {
    let scratch = Scratch::new("journeys-setup-after");
    for name in ["base", "parent", "child"] {
        fs::create_dir(scratch.path().join(name)).expect("setup directory");
    }
    fs::write(scratch.path().join("base/setup"), b"setup 1\nreload \"/\"\n").expect("base");
    fs::write(scratch.path().join("parent/setup"), b"setup 1\nafter \"base\"\nreload \"/\"\n").expect("parent");
    for (after, why) in [
        ("parent", "a chain of after is refused"),
        ("missing", "has no setup"),
        ("child", "names the journey itself"),
        ("../base", "invalid setup name"),
    ] {
        fs::write(scratch.path().join("child/setup"), format!("setup 1\nafter \"{after}\"\nreload \"/\"\n"))
            .expect("child");
        assert!(
            matches!(crate::again::setup(scratch.path(), "child"), Err(crate::Refusal::SetupAfter { name, after: found, why: reason }) if name == "child" && found == after && reason == why),
            "after {after}: {why}"
        );
    }
}

#[test]
fn every_committed_setup_reads_and_joins() {
    let root = Path::new(JOURNEYS);
    let mut count = 0;
    for name in crate::names(root).expect("committed journeys") {
        if root.join(&name).join("setup").is_file() {
            let bytes = fs::read(root.join(&name).join("setup")).expect("committed setup");
            journey::setup(&bytes).expect("read setup natively");
            let joined = crate::again::setup(root, &name).expect("join setup").expect("setup exists");
            assert!(!joined.steps.is_empty(), "{name} has setup steps");
            count += 1;
        }
    }
    assert!(count > 0, "committed setups exercised");
}

#[test]
fn project_landings_start_is_welcome_set_ups_recorded_acts() {
    use skein_fake_person::{Face, Find, Found, TreeFace};
    let root = Path::new(JOURNEYS);
    let setup =
        journey::setup(&fs::read(root.join("project-landing/setup")).expect("project setup")).expect("read setup");
    let recording = fs::read(root.join("welcome-set-up/recording")).expect("welcome recording");
    let lines = journey::lines(&recording).expect("welcome records");
    let acts: Vec<_> = lines
        .iter()
        .filter_map(|line| match line {
            journey::Line::Act { act, .. } => Some(act),
            _ => None,
        })
        .collect();
    let mut replay = journey::Replay::new(&recording).expect("replay welcome");
    let mut at = 0;
    while let Some(mark) = replay.next().expect("welcome replay holds") {
        if mark.acted.is_empty() {
            continue;
        }
        let journey::SetupStep::Act { role, name, order, act } = &setup.steps[at] else {
            panic!("setup step {} must act", at + 1)
        };
        assert_eq!(act, acts[at], "setup step {} gives the recorded act", at + 1);
        let find = Find::exactly_named(*role, name).nth(*order);
        let Found::Node(node) = TreeFace::new(mark.tree).find(&find) else {
            panic!("setup step {}: no {find}", at + 1)
        };
        assert!(mark.acted.contains(&node), "setup step {}: {find} must give the recorded event", at + 1);
        at += 1;
    }
    assert_eq!(at, acts.len(), "every welcome act checked");
    assert!(at > 0);
}

pub(super) struct TuiWorld {
    host: temper_client_shell_tui::host::Host,
    recorder: temper_client_shell_tui::journey::Recorder,
    now: u64,
}
impl TuiWorld {
    pub(super) fn new(address: &[u8]) -> Self {
        Self::sized(address, skein_ui_screen::Size { rows: 16, columns: 100 })
    }
    pub(super) fn sized(address: &[u8], size: skein_ui_screen::Size) -> Self {
        use temper_client_shell_tui::journey::{Facts, Recorder};
        let facts = Facts {
            seed: 7,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            size,
            scheme: Some(temper_client_tui::Scheme::Dark),
            background: None,
            boxed: false,
            address: address.into(),
            front: false,
            local: false,
            note: None,
            kept: Vec::new(),
            credential: None,
        };
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: Recorder::new(&facts), now: 1 };
        world.report(start);
        world
    }
    pub(super) fn report(
        &mut self,
        report: temper_client_shell_tui::host::Report,
    ) -> temper_client_shell_tui::host::Duties {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = self.recorder.report(&mut self.host, report);
        self.host.frame();
        self.recorder.deadline(self.host.deadline());
        duties
    }
    pub(super) fn mark(&mut self) {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        self.recorder.mark(&mut self.host);
        self.recorder.deadline(self.host.deadline());
    }
    pub(super) fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
    pub(super) fn exchange(&mut self, approval: bool) -> skein_lib::Token {
        use temper_client_shell_tui::journey::{Line, Sent, lines};
        let file = self.save();
        let records = lines(&file).expect("records");
        records
            .iter()
            .rev()
            .find_map(|line| match line {
                Line::Send { exchange, message: Sent::SignInApproval(_), .. } if approval => {
                    Some(skein_lib::Token::new(*exchange))
                }
                Line::Send { exchange, message: Sent::ApprovalAsk(_), .. } if !approval => {
                    Some(skein_lib::Token::new(*exchange))
                }
                _ => None,
            })
            .expect("sign-in exchange")
    }
}

fn tui_recording() -> Box<[u8]> {
    use skein_ui_screen_terminal::Key;
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::new(b"127.0.0.1:8367");
    world.report(Report::Key(Key::Paste(
        format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43)).into_bytes().into(),
    )));
    world.report(Report::Key(Key::Enter));
    world.save()
}

fn tui_approval_recording() -> Box<[u8]> {
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::new(b"127.0.0.1:8367");
    let exchange = world.exchange(false);
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: tui_approval_reply() }));
    world.report(Report::Opened { opened: false });
    world.mark();
    let exchange = world.exchange(true);
    let mut credential = 7_u64.to_be_bytes().to_vec();
    credential.extend_from_slice(&[8; 32]);
    let mut reply = vec![0, 1];
    reply.extend_from_slice(&u32::try_from(credential.len()).expect("credential length").to_be_bytes());
    reply.extend_from_slice(&credential);
    reply.extend_from_slice(&7_u64.to_be_bytes());
    reply.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: reply.into() }));
    world.report(Report::Kept { written: true });
    world.save()
}

pub(super) fn tui_approval_reply() -> Box<[u8]> {
    let mut reply = vec![0];
    reply.extend_from_slice(&32_u32.to_be_bytes());
    reply.extend_from_slice(&[7; 32]);
    reply.extend_from_slice(b"WDJBMJHT");
    reply.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    reply.into()
}

fn tui_written(lines: &[temper_client_shell_tui::journey::Line]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes
}

fn tree_bytes(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).expect("tree directory") {
            let entry = entry.expect("entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("relative path").to_owned();
            if entry.file_type().expect("file type").is_dir() {
                files.insert(relative, None);
                directories.push(path);
            } else {
                files.insert(relative, Some(fs::read(path).expect("tree file")));
            }
        }
    }
    files
}

#[test]
fn tui_add_redacts_proves_and_reprints_byte_for_byte() {
    let scratch = Scratch::new("journeys-tui-add");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    let out = scratch.path().join("out");
    for (name, file, counts) in
        [("pasted", tui_recording(), (1, 0, 0)), ("approval", tui_approval_recording(), (0, 1, 1))]
    {
        let added = crate::add_to(&web, &tui, name, &file, &[]).expect("add TUI");
        assert_eq!((added.link, added.approval, added.credential), counts);
        assert_eq!(added.bearer, added.credential);
        assert_eq!((added.stored, added.typed, added.given), (0, 0, 0));
        assert!(!web.exists(), "screen after clocks routes to TUI");
        let recording = fs::read(tui.join(name).join("recording")).expect("committed recording");
        let lines = temper_client_shell_tui::journey::lines(&recording).expect("redacted lines");
        assert!(lines.iter().all(|line| !matches!(
            line,
            temper_client_shell_tui::journey::Line::Mark(Some(_))
                | temper_client_shell_tui::journey::Line::End(Some(_))
        )));
    }
    assert!(crate::tui::reprint(&tui, &out).expect("reprint").is_empty());
    let mut expected = tree_bytes(&tui);
    for name in ["pasted", "approval"] {
        expected.remove(&Path::new(name).join("recording"));
    }
    assert!(expected == tree_bytes(&out), "reprinted TUI files agree");
}

#[test]
fn a_name_is_unique_across_both_clients_directories() {
    let scratch = Scratch::new("journeys-both-names");
    let web = scratch.path().join("web/journeys");
    let tui = scratch.path().join("tui/journeys");
    fs::create_dir(web.parent().expect("web surface")).expect("web directory");
    for (held, recording) in [(&web, tui_recording()), (&tui, fixture_recording())] {
        fs::create_dir_all(held.join("held")).expect("other client's name");
        let before = tree_bytes(scratch.path());
        assert!(
            matches!(crate::add(&web, "held", &recording, &[]), Err(crate::Refusal::OtherClient { name }) if name == "held")
        );
        assert_eq!(before, tree_bytes(scratch.path()));
        fs::remove_dir_all(held).expect("remove held name");
    }
}

#[test]
fn a_refused_tui_add_writes_nothing_even_over_an_existing_print() {
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-tui-refused");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    crate::add_to(&web, &tui, "kept", &tui_recording(), &[]).expect("previous good print");
    let mut lines = tj::lines(&tui_recording()).expect("fixture");
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
        (tui_recording().into_vec(), vec![Box::<[u8]>::from(b"tui-dark".as_slice())], "printed"),
        (tui_frame_original(), vec![Box::<[u8]>::from(b"private-frame-marker".as_slice())], "frame"),
    ] {
        for name in ["new", "kept"] {
            let before = tree_bytes(scratch.path());
            let refused = crate::add_to(&web, &tui, name, &recording, &given);
            let refused_place = match refused {
                Err(crate::Refusal::TuiProof { refusal: tj::Refusal::Differs { .. }, .. }) => "proof",
                Err(crate::Refusal::TuiFound {
                    found: tj::Found { place: tj::Place::Printed, what: tj::What::Original, .. },
                    file: Some(_),
                    ..
                }) => "printed",
                Err(crate::Refusal::TuiFound {
                    found: tj::Found { place: tj::Place::Frame { .. }, what: tj::What::Original, .. },
                    ..
                }) => "frame",
                other => panic!("unexpected refusal: {other:?}"),
            };
            assert_eq!(refused_place, place);
            assert!(tree_bytes(scratch.path()) == before, "{place} refusal leaves every file untouched");
            assert!(!tui.join("new").exists(), "refusal creates no directory");
        }
    }
}

fn tui_frame_original() -> Vec<u8> {
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::new("pri\u{200b}vate-frame-marker&".as_bytes());
    let exchange = world.exchange(false);
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: tui_approval_reply() }));
    world.report(Report::Opened { opened: true });
    world.report(Report::Resize(skein_ui_screen::Size { rows: 0, columns: 0 }));
    world.save().into_vec()
}

fn tui_sources(root: &Path) -> Result<(), String> {
    if !fs::read(root.join("overlay.css")).is_ok_and(|bytes| bytes.is_empty()) {
        return Err("TUI overlay.css holds a request; move its rules into the theme and empty it".into());
    }
    for name in crate::names(root).map_err(|why| format!("{why:?}"))? {
        if !fs::read_to_string(root.join(&name).join("README.md")).is_ok_and(|text| !text.trim().is_empty()) {
            return Err(format!("{name} needs a README.md holding its description"));
        }
    }
    Ok(())
}

#[test]
fn tui_sources_refuse_an_overlay_and_missing_or_empty_descriptions() {
    let scratch = Scratch::new("journeys-tui-sources");
    fs::create_dir(scratch.path().join("described")).expect("journey");
    fs::write(scratch.path().join("described/recording"), b"recording").expect("recording");
    fs::write(scratch.path().join("overlay.css"), b"").expect("empty overlay");
    for description in [None, Some(""), Some(" \n")] {
        if let Some(description) = description {
            fs::write(scratch.path().join("described/README.md"), description).expect("description");
        }
        assert!(tui_sources(scratch.path()).expect_err("description refused").contains("README.md"));
    }
    fs::write(scratch.path().join("described/README.md"), "A journey.").expect("description");
    assert_eq!(tui_sources(scratch.path()), Ok(()));
    fs::write(scratch.path().join("overlay.css"), b"x").expect("overlay request");
    assert!(tui_sources(scratch.path()).expect_err("overlay refused").contains("overlay.css"));
}

#[test]
fn a_stale_tui_keeps_its_print_while_other_journeys_reprint() {
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-tui-stale");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    for name in ["a-stale", "b-good"] {
        crate::add_to(&web, &tui, name, &tui_recording(), &[]).expect("fixture");
    }
    let mut lines = tj::lines(&fs::read(tui.join("a-stale/recording")).expect("recording")).expect("lines");
    let at = lines.iter().position(|line| matches!(line, tj::Line::Deadline(_))).expect("deadline");
    lines[at] = tj::Line::Deadline(Some(Time::ZERO));
    fs::write(tui.join("a-stale/recording"), tui_written(&lines)).expect("stale recording");
    fs::write(tui.join("a-stale/01.html"), b"kept HTML").expect("kept print");
    fs::write(tui.join("a-stale/01.txt"), b"kept text").expect("kept print");
    fs::write(tui.join("b-good/99.txt"), b"obsolete").expect("extra print");
    fs::write(tui.join("b-good/README.md"), b"Description").expect("description");
    let stale = crate::tui::reprint(&tui, &tui).expect("reprint all");
    assert_eq!(stale.len(), 1);
    assert_eq!((&*stale[0].0, stale[0].1.turn, stale[0].1.line), ("a-stale", 1, u32::try_from(at + 1).expect("line")));
    assert_eq!(fs::read(tui.join("a-stale/01.html")).expect("HTML"), b"kept HTML");
    assert_eq!(fs::read(tui.join("a-stale/01.txt")).expect("text"), b"kept text");
    assert!(!tui.join("b-good/99.txt").exists());
    assert_eq!(fs::read(tui.join("b-good/README.md")).expect("description"), b"Description");
}
