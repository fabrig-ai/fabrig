//! Whole-tool admission, normalisation and filesystem results (testing/tools.md, 6).
use fixtures::{
    TuiWorld, absent, assert_normalised, fixture_recording, stale_recording, tree_bytes, tui_approval_recording,
    tui_recording, tui_written,
};
use skein_lib::Time;
use skein_scratch::Scratch;
use std::fs;
use temper_client_journeys_kit as fixtures;
use temper_client_journeys_tool as tool;
use temper_client_shell_web::journey;

fn vary_fields(fields: Vec<&mut Box<[u8]>>) -> bool {
    let mut changed = false;
    for field in fields {
        for (from, to) in [
            (b"1.2.3 (abcdef123456-dirty)".as_slice(), b"9.8.7 (fedcba654321-dirty)".as_slice()),
            (b"34073".as_slice(), b"45182".as_slice()),
            (b"51, 52, 48, 55, 51".as_slice(), b"52, 53, 49, 56, 50".as_slice()),
        ] {
            let mut start = 0;
            let mut replaced = Vec::new();
            while let Some(at) = skein_lib::bytes::find(&field[start..], from) {
                replaced.extend_from_slice(&field[start..start + at]);
                replaced.extend_from_slice(to);
                start += at + from.len();
                changed = true;
            }
            replaced.extend_from_slice(&field[start..]);
            *field = replaced.into();
        }
    }
    changed
}

fn varying_recording(recording: &[u8], is_tui: bool, expected_records: usize) -> Vec<u8> {
    let mut changed = 0;
    let mut out = Vec::new();
    if is_tui {
        use temper_client_shell_tui::journey as tj;
        for mut line in tj::lines(recording).expect("TUI records") {
            changed +=
                usize::from(vary_fields(tj::fields_mut(&mut line).into_iter().map(|(field, _)| field).collect()));
            line.write(&mut out);
        }
    } else {
        for mut line in journey::lines(recording).expect("browser records") {
            changed +=
                usize::from(vary_fields(journey::fields_mut(&mut line).into_iter().map(|(field, _)| field).collect()));
            line.write(&mut out);
        }
    }
    assert_eq!(changed, expected_records, "records whose build or port varies");
    out
}

fn contains(bytes: &[u8], text: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, text).is_some()
}

#[test]
fn add_gives_both_clients_placeholders_after_proving() {
    // Each placeholder says whether it appears in the drawn marks as well as fields.
    const WEB_PLACEHOLDERS: &[(&[u8], bool)] = &[(b"10001", true)];
    const TUI_PLACEHOLDERS: &[(&[u8], bool)] = &[(b"10001", true), (b"0.0.0 (000000000000)", true)];
    const APPROVAL_PLACEHOLDERS: &[(&[u8], bool)] =
        &[(b"10001", true), (b"0.0.0 (000000000000)", true), (b"BBBBBBBB", false), (b"BBBB-BBBB", true)];
    // Browser: ways and end. TUI: version, address and marks, plus paste/event
    // for a link or the approval's browse address.
    const WEB_VARIED_RECORDS: usize = 2;
    const TUI_VARIED_RECORDS: usize = 6;
    let scratch = Scratch::new("journeys-add-placeholders");
    for (name, recording, is_tui, counts, placeholders, redacted_link, varied_records) in [
        ("browser", fixture_recording(), false, (0, 1, 0, 0), WEB_PLACEHOLDERS, false, WEB_VARIED_RECORDS),
        ("terminal", tui_recording(), true, (1, 1, 0, 0), TUI_PLACEHOLDERS, true, TUI_VARIED_RECORDS),
        ("approval", tui_approval_recording(), true, (1, 1, 1, 0), APPROVAL_PLACEHOLDERS, false, TUI_VARIED_RECORDS),
    ] {
        let variant = varying_recording(&recording, is_tui, varied_records);
        assert_ne!(recording.as_ref(), variant, "the fixture varies");
        let mut first = None;
        for (build, recording) in [("first", recording.as_ref()), ("second", variant.as_slice())] {
            let root = scratch.path().join(name).join(build);
            let web = root.join("web");
            let tui = root.join("tui");
            let added = tool::add_to(&web, &tui, name, recording, &[]).expect("prove then normalise and replay");
            assert_eq!((added.build, added.port, added.code, added.stood), counts);
            assert!(added.placeholder_places > 0);
            let directory = if is_tui { &tui } else { &web };
            let saved = fs::read(directory.join(name).join("recording")).expect("written recording");
            let files = tree_bytes(&directory.join(name));
            let prints: Vec<_> = files
                .iter()
                .filter(|(path, _)| path.extension().is_some_and(|extension| extension == "html" || extension == "txt"))
                .filter_map(|(_, bytes)| bytes.as_deref())
                .collect();
            let live = if is_tui {
                temper_client_shell_tui::journey::normalise(recording).expect("normalise live marks")
            } else {
                journey::normalise(recording).expect("normalise live marks")
            };
            let marks: Vec<_> = if is_tui {
                use temper_client_shell_tui::journey as tj;
                tj::lines(&live.recording)
                    .expect("normalised live TUI")
                    .into_iter()
                    .filter_map(|line| match line {
                        tj::Line::Mark(print) | tj::Line::End(print) => print,
                        _ => None,
                    })
                    .collect()
            } else {
                journey::lines(&live.recording)
                    .expect("normalised live browser")
                    .into_iter()
                    .filter_map(|line| match line {
                        journey::Line::Mark(html) | journey::Line::End(html) => html,
                        _ => None,
                    })
                    .collect()
            };
            for &(placeholder, drawn) in placeholders {
                assert!(contains(&saved, placeholder), "{} in written fields", String::from_utf8_lossy(placeholder));
                if drawn {
                    assert!(
                        prints.iter().any(|bytes| contains(bytes, placeholder)),
                        "{name}: {} in replay's print",
                        String::from_utf8_lossy(placeholder)
                    );
                    assert!(marks.iter().any(|bytes| contains(bytes, placeholder)), "{name}: placeholder in live mark");
                }
            }
            if redacted_link {
                let lines = temper_client_shell_tui::journey::lines(&saved).expect("written TUI lines");
                assert!(
                    lines.iter().any(|line| matches!(line,
                    temper_client_shell_tui::journey::Line::Send {
                        message: temper_client_shell_tui::journey::Sent::SignInLink(bytes), .. }
                    if contains(bytes, "x".repeat(32).as_bytes()))),
                    "link remains redacted"
                );
                absent(&saved, "A".repeat(43).as_bytes(), "normalised recording");
            }
            let snapshot = tree_bytes(directory);
            if let Some(first) = &first {
                assert!(snapshot == *first, "builds and ports add to identical recording and printed files");
            } else {
                first = Some(snapshot);
            }
        }
    }
}

#[test]
fn normalise_rewrites_scratch_copies_and_keeps_stale_recordings() {
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
fn a_refused_tui_add_writes_nothing_even_over_an_existing_print() {
    use temper_client_shell_tui::journey as tj;
    let scratch = Scratch::new("journeys-tui-refused");
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    let recording = tui_recording();
    tool::add_to(&web, &tui, "kept", &recording, &[]).expect("previous good print");
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
            let refused = tool::add_to(&web, &tui, name, &recording, &given);
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
            assert!(!tui.join("new").exists(), "refusal creates no directory");
        }
    }
}

fn tui_frame_original() -> Vec<u8> {
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::with_box_log("pri\u{200b}vate-frame-marker&".as_bytes());
    world.report(Report::Resize(skein_ui_screen::Size { rows: 0, columns: 0 }));
    world.save().into_vec()
}

fn fixed_capture() -> Box<[u8]> {
    use fixtures::web::{Capture, managed_capture};
    use skein_ui::{Act, Role};
    let mut capture = managed_capture(Capture::fixed());
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    capture.save()
}

#[test]
fn a_fixed_capture_keeps_its_start_through_add() {
    let recording = fixed_capture();
    let scratch = skein_scratch::Scratch::new("journeys-fixed-add");
    tool::add(scratch.path(), "fixed", &recording, &[]).expect("redact, prove, normalise and scan");
    let added = fs::read(scratch.path().join("fixed/recording")).expect("added recording");
    let header = b"journey 2\nseed 7\nclocks 3600000000000 1767690000000000000\nfixed\n";
    assert!(recording.starts_with(header), "capture keeps the fixed facts");
    assert!(added.starts_with(header), "add keeps the fixed facts");
    let mut replay = journey::Replay::new(&added).expect("added fixed recording");
    let mut marks = 0;
    while replay.next().expect("added fixed recording replays").is_some() {
        marks += 1;
    }
    assert_eq!(marks, 2, "the press mark and the end are replayed");
}

#[test]
fn two_fixed_walks_and_their_added_recordings_are_identical() {
    let first = fixed_capture();
    let second = fixed_capture();
    assert!(first == second, "fresh captures of the same scripted walk have identical bytes");
    let scratch = skein_scratch::Scratch::new("journeys-fixed-twice");
    for (name, recording) in [("first", first), ("second", second)] {
        tool::add(scratch.path(), name, &recording, &[]).expect("add fixed walk");
    }
    let first = fs::read(scratch.path().join("first/recording")).expect("first added recording");
    let second = fs::read(scratch.path().join("second/recording")).expect("second added recording");
    assert!(first == second, "add writes identical recording bytes");
}
