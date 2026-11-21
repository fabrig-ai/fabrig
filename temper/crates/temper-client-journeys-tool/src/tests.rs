pub(super) use fixtures::{TuiWorld, tui_approval_reply};
use fixtures::{
    absent, fixture_recording, fixture_with_marks_and_secret, fixture_with_secret, stale_recording, tree_bytes,
    tui_approval_recording, tui_recording,
};
use temper_client_journeys_kit as fixtures;

use crate::JOURNEYS;
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
fn normalise_rewrites_scratch_copies_and_keeps_stale_recordings() {
    use crate as tool;
    use fixtures::{assert_normalised, tui_written};
    use skein_lib::Time;
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-normalise");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    for root in [&web, &tui] {
        fs::create_dir_all(root).expect("scratch directory");
    }
    let browser = journey::prove(&fixture_recording()).expect("browser proof");
    let terminal = tj::prove(&tui_recording()).expect("TUI proof");
    for (root, good, stale) in [
        (&web, browser.clone(), stale_recording(&browser)),
        (&tui, terminal.clone(), {
            let mut lines = tj::lines(&terminal).expect("TUI lines");
            let at = lines.iter().position(|line| matches!(line, tj::Line::Deadline(_))).expect("deadline");
            lines[at] = tj::Line::Deadline(Some(Time::ZERO));
            tui_written(&lines).into()
        }),
    ] {
        for (name, recording) in [("a-stale", stale), ("z-good", good)] {
            fs::create_dir(root.join(name)).expect("fixture directory");
            fs::write(root.join(name).join("recording"), recording).expect("fixture recording");
            fs::write(root.join(name).join("01.html"), b"old print").expect("previous print");
            fs::write(root.join(name).join("README.md"), b"Description").expect("description");
        }
    }
    let before_web = tree_bytes(&web.join("a-stale"));
    let before_tui = tree_bytes(&tui.join("a-stale"));
    let (written, stale) = tool::normalise(&web, &tui).expect("normalise both scratch directories");
    assert_eq!(written.iter().filter(|(name, _)| name == "z-good").count(), 2, "continue past both stale recordings");
    assert_eq!(stale.len(), 2);
    assert!(matches!(&stale[0], tool::Refusal::Proof { name, refusal: journey::Refusal::Stale(why) }
        if name == "a-stale" && why.turn == 1 && why.recorded == "deadline" && why.asked == "deadline"));
    assert!(matches!(&stale[1], tool::Refusal::TuiProof { name, refusal: tj::Refusal::Stale(why) }
        if name == "a-stale" && why.turn == 1 && why.recorded.as_ref() == b"deadline" && why.asked.as_ref() == b"deadline"));
    assert_eq!(before_web, tree_bytes(&web.join("a-stale")));
    assert_eq!(before_tui, tree_bytes(&tui.join("a-stale")));
    assert_eq!(tool::reprint(&web, &web).expect("reprint browser").len(), 1);
    assert_eq!(tool::tui::reprint(&tui, &tui).expect("reprint TUI").len(), 1);
    assert_eq!(before_web, tree_bytes(&web.join("a-stale")));
    assert_eq!(before_tui, tree_bytes(&tui.join("a-stale")));
    for root in [&web, &tui] {
        assert_ne!(fs::read(root.join("z-good/01.html")).expect("new print"), b"old print");
        fs::remove_dir_all(root.join("a-stale")).expect("remove stale fixture");
        assert_normalised(root, root == &tui);
    }
    let before = tree_bytes(scratch.path());
    let (written, stale) = tool::normalise(&web, &tui).expect("normalise again");
    assert!(written.is_empty() && stale.is_empty(), "already normalised recordings aren't written again");
    assert_eq!(before, tree_bytes(scratch.path()));
}

#[test]
fn moved_names_refuse_add_for_either_client() {
    let scratch = Scratch::new("moved-add");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    for (root, name, recording) in [(&web, "browser", fixture_recording()), (&tui, "terminal", tui_recording())] {
        crate::add_to(&web, &tui, name, &recording, &[]).expect("fixture journey");
        fs::write(root.join(name).join("walk"), "walk 2\nmark\n").expect("moved source");
        let before = tree_bytes(scratch.path());
        assert_eq!(
            crate::run::refusal(crate::add_to(&web, &tui, name, &recording, &[]).expect_err("add moved name")),
            format!("{name} has a walk of version 2; use temper-client-walk")
        );
        assert_eq!(before, tree_bytes(scratch.path()), "refusal changes no byte");
    }
}
