pub(super) use fixtures::{TuiWorld, tui_approval_reply};
use fixtures::{
    absent, fixture_recording, fixture_with_marks_and_secret, fixture_with_secret, stale_recording, tree_bytes,
    tui_approval_recording, tui_recording, tui_written,
};
use temper_client_journeys_kit as fixtures;

use crate::JOURNEYS;
use skein_lib::Time;
use skein_scratch::Scratch;
use std::{fs, path::Path};
use temper_client_shell_web::journey;

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
fn the_tui_overlay_is_empty() {
    let overlay = fs::read(Path::new(crate::TUI_JOURNEYS).join("overlay.css")).expect("committed TUI overlay.css");
    assert!(overlay.is_empty(), "TUI overlay.css holds a request; move its rules into the theme and empty it");
}

#[test]
fn every_tui_journey_has_its_description() {
    for name in crate::names(Path::new(crate::TUI_JOURNEYS)).expect("committed TUI journeys") {
        let path = Path::new(crate::TUI_JOURNEYS).join(&name).join("README.md");
        let description = fs::read_to_string(path).expect("TUI journey README.md");
        assert!(!description.trim().is_empty(), "{name} needs a README.md holding its description");
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
    assert_eq!(crate::reprint(&source, &out).expect("reprint").len(), 0);
    assert_eq!(fs::read(readme).expect("description kept"), description);
    assert!(!directory.join("99.html").exists());
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
fn a_marked_walk_redacted_prints_the_same_pages() {
    let secret = ["walk", "-secret"].concat().into_bytes();
    let recording = fixture_with_marks_and_secret(Some(secret.clone().into()));
    let redacted = journey::redact(&recording, &[]).expect("redact host walk");
    assert_eq!(redacted.originals.count(journey::Kind::Link), 1);
    absent(&redacted.recording, &secret, "redacted host walk");
    let expected = crate::printed("marked", &recording).expect("read host walk").expect("replay original");
    let pages = crate::printed("marked", &redacted.recording).expect("read redacted walk").expect("replay redacted");
    assert!(pages.len() >= 2, "several marked pages compared");
    assert_eq!(pages.len(), expected.len(), "every page retained");
    for (page, expected) in pages.iter().zip(&expected) {
        assert_eq!(page.name, expected.name);
        assert_eq!(page.document, expected.document, "redaction preserves the printed page");
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
fn normalised_stale_replays_use_the_proofs_safe_refusal() {
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

#[test]
fn placeholder_counts_follow_each_clients_redaction() {
    let added = crate::Added {
        link: 1,
        stored: 2,
        typed: 3,
        given: 4,
        places: 5,
        approval: 6,
        credential: 7,
        bearer: 8,
        build: 9,
        port: 10,
        code: 11,
        placeholder_places: 12,
        stood: 13,
    };
    for (tui, redaction) in [
        (false, "link: 1; stored: 2; typed: 3; given: 4; places replaced: 5"),
        (true, "link: 1; stored: 2; typed: 3; given: 4; approval: 6; credential: 7; bearer: 8; places replaced: 5"),
    ] {
        assert_eq!(
            crate::run::addition(&added, tui),
            format!(
                "{redaction}\nplaceholders: build: 9; port: 10; code: 11; places replaced: 12; stood: 13\n{}",
                crate::table::CAUTION
            )
        );
    }
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
    assert_eq!(crate::tui::reprint(&tui, &out).expect("reprint").len(), 0);
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
