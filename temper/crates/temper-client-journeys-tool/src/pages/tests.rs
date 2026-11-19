use super::*;
use skein_scratch::Scratch;
use std::path::PathBuf;
use temper_client_journeys_kit::{fixture_recording, fixture_with_secret, tree_bytes, tui_recording};
use temper_client_shell_tui::journey as tj;
use temper_client_shell_web::journey;

fn roots(scratch: &Scratch) -> (PathBuf, PathBuf) {
    let web = scratch.path().join("web");
    let tui = scratch.path().join("tui");
    for (name, recording) in [
        ("browser-link", fixture_with_secret(Some("A".repeat(43).into_bytes().into()))),
        ("browser-welcome", fixture_recording()),
        ("a-terminal", tui_recording()),
    ] {
        crate::add_to(&web, &tui, name, &recording, &[]).expect("add current journey");
    }
    for root in [&web, &tui] {
        fs::write(root.join("README.md"), "Journeys.\n").expect("root description");
    }
    fs::write(web.join("browser-link/setup"), "setup 1\nsee Welcome\n").expect("browser source");
    fs::write(tui.join("a-terminal/walk"), "walk 1\nmark\n").expect("TUI source");
    (web, tui)
}

fn unbounded(
    web: &Path,
    tui: &Path,
    journeys: &[Journey],
    walk: impl FnMut(&Journey, Option<Duration>) -> Result<(), Stopped>,
    mut show: impl FnMut(&str),
) -> Result<bool, String> {
    write(
        web,
        tui,
        journeys,
        None,
        || Duration::ZERO,
        walk,
        |destination, line| {
            if destination == Destination::Stdout {
                show(line);
            }
        },
    )
}

fn fresh_lines() -> [&'static str; 3] {
    ["fresh browser-link", "fresh browser-welcome", "fresh a-terminal"]
}

fn stale_browser(web: &Path) {
    let path = web.join("browser-link/recording");
    let mut lines = journey::lines(&fs::read(&path).expect("recording")).expect("records");
    let mut changed = 0;
    for line in &mut lines {
        if let journey::Line::Exchange { path, body, .. } = line
            && path.as_ref() == b"/jig/v1/sign-in/link"
        {
            *body = b"another-fake-ask".as_slice().into();
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "the sign-in ask");
    let mut recording = Vec::new();
    for line in lines {
        line.write(&mut recording);
    }
    fs::write(path, recording).expect("stale ask");
}

#[test]
fn current_journeys_walk_nothing_and_change_no_file_byte() {
    let scratch = Scratch::new("pages-current");
    let (web, tui) = roots(&scratch);
    let before = tree_bytes(scratch.path());
    let stale = stale(&web, &tui).expect("replay both roots");
    assert!(stale.iter().all(|journey| !journey.stale));
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = unbounded(
        &web,
        &tui,
        &stale,
        |journey, rest| {
            assert!(rest.is_none(), "no bound");
            walks.push(journey.name.clone());
            Err("no walk belongs here".into())
        },
        |line| shown.push(line.to_owned()),
    )
    .expect("print current roots");
    assert!(!unprinted, "exit 0");
    assert!(walks.is_empty(), "no box, browser or terminal requested");
    assert_eq!(shown, fresh_lines());
    assert!(before == tree_bytes(scratch.path()), "all files, including recordings and files beside pages");
}

#[test]
fn one_stale_browser_alone_is_walked_and_keeps_its_pages_when_stopped() {
    let scratch = Scratch::new("pages-stopped");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    let before = tree_bytes(scratch.path());
    // A current journey of each client must still be printed after the failed walk.
    fs::write(web.join("browser-welcome/01.html"), "old browser print").expect("old page");
    fs::write(tui.join("a-terminal/01.html"), "old TUI print").expect("old page");
    fs::write(tui.join("a-terminal/01.txt"), "old TUI text").expect("old text");
    let stale = stale(&web, &tui).expect("replay both roots");
    let expected = vec![Journey { client: Client::Web, name: "browser-link".into(), stale: true }];
    assert_eq!(stale.iter().filter(|journey| journey.stale).collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = unbounded(
        &web,
        &tui,
        &stale,
        |journey, rest| {
            assert!(rest.is_none(), "no bound");
            walks.push(Journey { client: journey.client, name: journey.name.clone(), stale: journey.stale });
            Err("at step 1: fake control is absent\nfake diagnostic".into())
        },
        |line| shown.push(line.to_owned()),
    )
    .expect("print the other journeys");
    assert_eq!(walks, expected);
    assert_eq!(
        shown,
        [
            "stopped browser-link: at step 1: fake control is absent; fake diagnostic",
            "fresh browser-welcome",
            "fresh a-terminal"
        ]
    );
    assert!(unprinted, "exit 1 once the others print");
    assert!(before == tree_bytes(scratch.path()), "stale files kept; both current clients reprinted");
}

#[test]
fn orphan_numbered_pages_go_and_sources_and_other_files_stay() {
    let scratch = Scratch::new("pages-orphans");
    let (web, tui) = roots(&scratch);
    for root in [&web, &tui] {
        let orphan = root.join("unrecorded");
        fs::create_dir(&orphan).expect("unrecorded journey");
        fs::create_dir(orphan.join("02.html")).expect("a directory is not a page");
        for file in ["README.md", "01.html", "100.html", "1.html", "notes.html", "walk", "setup"] {
            fs::write(orphan.join(file), format!("fake {file}\n")).expect("orphan files");
        }
    }
    fs::write(web.join("unrecorded/01.txt"), "not a browser page").expect("other browser file");
    fs::write(tui.join("unrecorded/01.txt"), "TUI page").expect("TUI text page");
    let mut expected = tree_bytes(scratch.path());
    for file in [
        "web/unrecorded/01.html",
        "web/unrecorded/100.html",
        "tui/unrecorded/01.html",
        "tui/unrecorded/100.html",
        "tui/unrecorded/01.txt",
    ] {
        assert!(expected.remove(Path::new(file)).is_some());
    }
    let stale = stale(&web, &tui).expect("orphan is not a journey to walk");
    assert!(stale.iter().all(|journey| !journey.stale));
    assert!(!unbounded(&web, &tui, &stale, |_, _| Err("no walk".into()), |_| {}).expect("remove orphan pages"));
    assert!(expected == tree_bytes(scratch.path()), "only numbered pages removed");
}

#[test]
fn both_clients_stale_walks_are_in_name_order_and_successes_reprint() {
    let scratch = Scratch::new("pages-walked");
    let (web, tui) = roots(&scratch);
    let browser = fs::read(web.join("browser-link/recording")).expect("current browser recording");
    let terminal = fs::read(tui.join("a-terminal/recording")).expect("current TUI recording");
    stale_browser(&web);
    stale_terminal(&tui);
    let stale = stale(&web, &tui).expect("stale clients");
    let expected = vec![
        Journey { client: Client::Web, name: "browser-link".into(), stale: true },
        Journey { client: Client::Tui, name: "a-terminal".into(), stale: true },
    ];
    assert_eq!(stale.iter().filter(|journey| journey.stale).collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = unbounded(
        &web,
        &tui,
        &stale,
        |journey, rest| {
            assert!(rest.is_none(), "no bound");
            walks.push(Journey { client: journey.client, name: journey.name.clone(), stale: journey.stale });
            let (root, recording) = match journey.client {
                Client::Web => (&web, &browser),
                Client::Tui => (&tui, &terminal),
            };
            fs::write(root.join(&journey.name).join("recording"), recording).expect("fake successful walk");
            Ok(())
        },
        |line| shown.push(line.to_owned()),
    )
    .expect("print successful walks");
    assert_eq!(walks, expected);
    assert_eq!(shown, ["walked again browser-link", "fresh browser-welcome", "walked again a-terminal"]);
    assert!(!unprinted, "exit 0 after both current recordings print");
    assert!(super::stale(&web, &tui).expect("replay walked recordings").iter().all(|journey| !journey.stale));
}

#[test]
fn printing_and_orphan_removal_refuse_filesystem_errors() {
    let scratch = Scratch::new("pages-errors");
    let (web, tui) = roots(&scratch);
    let blocked = scratch.path().join("blocked");
    fs::write(&blocked, "not a root directory").expect("blocked root");
    assert!(reprint(&blocked, &tui, |_, _| {}).is_err(), "print failure reaches the command's exit 3");
    let orphan = web.join("unrecorded");
    fs::create_dir(&orphan).expect("orphan directory");
    let recording = orphan.join("recording");
    std::os::unix::fs::symlink("recording", &recording).expect("unreadable recording path");
    assert!(matches!(remove_orphans(&web, Client::Web), Err(Refusal::Io { path }) if path == recording));
}

fn stale_terminal(tui: &Path) {
    let path = tui.join("a-terminal/recording");
    let mut lines = tj::lines(&fs::read(&path).expect("TUI recording")).expect("TUI records");
    let deadline = lines
        .iter_mut()
        .find_map(|line| if let tj::Line::Deadline(deadline) = line { Some(deadline) } else { None })
        .expect("TUI deadline");
    *deadline = Some(skein_lib::Time::ZERO);
    fs::write(path, temper_client_journeys_kit::tui_written(&lines)).expect("stale TUI");
}

#[test]
fn one_successful_walk_reprints_before_its_line_and_routes_diagnostics_to_stderr() {
    let scratch = Scratch::new("pages-one-walk");
    let (web, tui) = roots(&scratch);
    let current = fs::read(web.join("browser-link/recording")).expect("current recording");
    stale_browser(&web);
    let journeys = stale(&web, &tui).expect("journeys");
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    fs::write(web.join("browser-link/01.html"), "old page").expect("old page before walk");
    assert!(
        !write(
            &web,
            &tui,
            &journeys,
            None,
            || Duration::ZERO,
            |journey, _| {
                walks.push(journey.name.clone());
                fs::write(web.join("browser-link/recording"), &current).expect("current walk");
                Ok(())
            },
            |destination, line| {
                if destination == Destination::Stdout {
                    assert!(
                        fs::read_to_string(web.join("browser-link/01.html"))
                            .expect("printed page")
                            .contains("<!doctype html>"),
                        "print before status"
                    );
                }
                shown.push((destination, line.to_owned()));
            }
        )
        .expect("reprint")
    );
    assert_eq!(walks, ["browser-link"]);
    assert_eq!(
        shown,
        [
            (Destination::Stderr, "a-terminal".into()),
            (Destination::Stderr, "browser-link".into()),
            (Destination::Stderr, "browser-welcome".into()),
            (Destination::Stdout, "walked again browser-link".into()),
            (Destination::Stdout, "fresh browser-welcome".into()),
            (Destination::Stdout, "fresh a-terminal".into()),
        ]
    );
}

#[test]
fn a_successful_walk_that_stays_stale_is_stopped_without_scratch() {
    let scratch = Scratch::new("pages-still-stale");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    let before = tree_bytes(scratch.path());
    let journeys = stale(&web, &tui).expect("journeys");
    let mut shown = Vec::new();
    assert!(unbounded(&web, &tui, &journeys, |_, _| Ok(()), |line| shown.push(line.to_owned())).expect("reprint"));
    assert_eq!(
        shown,
        ["stopped browser-link: still stale after its walk", "fresh browser-welcome", "fresh a-terminal"]
    );
    assert!(before == tree_bytes(scratch.path()), "stale pages kept");
}

#[test]
fn a_stopped_walk_keeps_its_own_scratch_and_every_journey_byte() {
    let scratch = Scratch::new("pages-kept-scratch");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    let before_web = tree_bytes(&web);
    let before_tui = tree_bytes(&tui);
    let own = scratch.path().join("walk-scratch");
    let journeys = stale(&web, &tui).expect("journeys");
    let mut shown = Vec::new();
    assert!(
        unbounded(
            &web,
            &tui,
            &journeys,
            |_, _| {
                fs::create_dir(&own).expect("walk's own scratch");
                fs::write(own.join("diagnostic"), "kept").expect("walk diagnostic");
                Err(Stopped {
                    why: "browser-link at step 1: no \"Send\" button\nsecond line".into(),
                    scratch: Some(own.clone()),
                })
            },
            |line| shown.push(line.to_owned())
        )
        .expect("reprint others")
    );
    assert_eq!(
        shown,
        [
            format!(
                "stopped browser-link: browser-link at step 1: no \"Send\" button; second line; scratch kept at {}",
                own.display()
            ),
            "fresh browser-welcome".into(),
            "fresh a-terminal".into()
        ]
    );
    assert_eq!(fs::read_to_string(own.join("diagnostic")).expect("scratch remains"), "kept");
    assert!(before_web == tree_bytes(&web), "browser files unchanged");
    assert!(before_tui == tree_bytes(&tui), "TUI files unchanged");
}

#[test]
fn a_past_bound_stops_each_stale_journey_without_a_walk() {
    let scratch = Scratch::new("pages-past-bound");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    stale_terminal(&tui);
    let before = tree_bytes(scratch.path());
    let journeys = stale(&web, &tui).expect("journeys");
    let bound = Bound { word: "0s".into(), duration: Duration::ZERO };
    let mut shown = Vec::new();
    assert!(
        write(
            &web,
            &tui,
            &journeys,
            Some(&bound),
            || Duration::ZERO,
            |_, _| { panic!("past-bound walks must not start") },
            |destination, line| if destination == Destination::Stdout {
                shown.push(line.to_owned());
            }
        )
        .expect("print others")
    );
    assert_eq!(
        shown,
        [
            "stopped browser-link: not walked, past --within 0s",
            "fresh browser-welcome",
            "stopped a-terminal: not walked, past --within 0s"
        ]
    );
    assert!(before == tree_bytes(scratch.path()), "no byte changed");
}

#[test]
fn a_walk_cut_at_the_bound_keeps_its_scratch_and_the_given_duration_word() {
    let scratch = Scratch::new("pages-cut-bound");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    let journeys = stale(&web, &tui).expect("journeys");
    let bound = Bound { word: "90m".into(), duration: Duration::from_mins(90) };
    // Scanning and building have already consumed 80 minutes.
    let elapsed = std::cell::Cell::new(Duration::from_mins(80));
    let own = scratch.path().join("cut-walk");
    let mut shown = Vec::new();
    assert!(
        write(
            &web,
            &tui,
            &journeys,
            Some(&bound),
            || elapsed.get(),
            |_, rest| {
                assert_eq!(rest, Some(Duration::from_mins(10)));
                fs::create_dir(&own).expect("walk scratch");
                elapsed.set(Duration::from_secs(5401));
                Err(Stopped { why: "harness failed".into(), scratch: Some(own.clone()) })
            },
            |destination, line| if destination == Destination::Stdout {
                shown.push(line.to_owned());
            }
        )
        .expect("print others")
    );
    assert_eq!(
        shown,
        [
            format!("stopped browser-link: cut at --within 90m; scratch kept at {}", own.display()),
            "fresh browser-welcome".into(),
            "fresh a-terminal".into()
        ]
    );
    assert!(own.is_dir());
}

#[test]
fn a_rest_shorter_than_the_close_window_is_not_walked() {
    let scratch = Scratch::new("pages-close-window");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    let journeys = stale(&web, &tui).expect("journeys");
    let bound = Bound { word: "45s".into(), duration: Duration::from_secs(45) };
    let mut shown = Vec::new();
    assert!(
        write(
            &web,
            &tui,
            &journeys,
            Some(&bound),
            || Duration::from_secs(42),
            |_, _| { panic!("a short harness horizon would panic") },
            |destination, line| if destination == Destination::Stdout {
                shown.push(line.to_owned());
            }
        )
        .expect("print others")
    );
    assert_eq!(
        shown,
        ["stopped browser-link: not walked, past --within 45s", "fresh browser-welcome", "fresh a-terminal"]
    );
}

#[test]
fn check_names_fresh_and_stale_journeys_in_client_order_without_writing() {
    let scratch = Scratch::new("pages-check");
    let (web, tui) = roots(&scratch);
    let mut shown = Vec::new();
    let before = tree_bytes(scratch.path());
    assert!(!check(&web, &tui, |line| shown.push(line.to_owned())).expect("check current"));
    assert_eq!(shown, fresh_lines());
    assert!(before == tree_bytes(scratch.path()), "no byte changed");
    stale_browser(&web);
    let before = tree_bytes(scratch.path());
    shown.clear();
    assert!(check(&web, &tui, |line| shown.push(line.to_owned())).expect("check stale"));
    assert_eq!(shown, ["stale browser-link", "fresh browser-welcome", "fresh a-terminal"]);
    assert!(before == tree_bytes(scratch.path()), "no byte changed");
    let blocked = scratch.path().join("blocked");
    fs::write(&blocked, "not a directory").expect("blocked root");
    for unreadable in [&blocked, &scratch.path().join("missing")] {
        assert!(check(unreadable, &tui, |_| panic!("unreadable root cannot print")).is_err(), "exit 3");
    }
}

#[test]
fn bare_reprint_keeps_names_on_stdout_and_stale_diagnostics_on_stderr() {
    let scratch = Scratch::new("pages-reprint-destination");
    let (web, tui) = roots(&scratch);
    stale_browser(&web);
    for names_to in [Destination::Stdout, Destination::Stderr] {
        let mut shown = Vec::new();
        assert_eq!(
            run::reprint_roots(&web, &tui, names_to, |destination, line| shown.push((destination, line.to_owned())))
                .expect("reprint"),
            ["browser-link"]
        );
        let mut lines = shown.iter();
        for name in ["a-terminal", "browser-link", "browser-welcome"] {
            assert_eq!(lines.next(), Some(&(names_to, name.into())));
        }
        let (destination, message) = lines.next().expect("stale diagnostic");
        assert_eq!(*destination, Destination::Stderr);
        assert!(message.starts_with("temper-client-journeys: browser-link is stale at turn "));
        assert!(lines.next().is_none());
    }
}
