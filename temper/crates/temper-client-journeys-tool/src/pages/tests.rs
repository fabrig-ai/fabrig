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
        ("terminal", tui_recording()),
    ] {
        crate::add_to(&web, &tui, name, &recording, &[]).expect("add current journey");
    }
    for root in [&web, &tui] {
        fs::write(root.join("README.md"), "Journeys.\n").expect("root description");
    }
    fs::write(web.join("browser-link/setup"), "setup 1\nsee Welcome\n").expect("browser source");
    fs::write(tui.join("terminal/walk"), "walk 1\nmark\n").expect("TUI source");
    (web, tui)
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
    assert_eq!(stale, vec![]);
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = write(
        &web,
        &tui,
        &stale,
        |journey| {
            walks.push(journey.name.clone());
            Err("no walk belongs here".into())
        },
        |line| shown.push(line.to_owned()),
    )
    .expect("print current roots");
    assert!(!unprinted, "exit 0");
    assert!(walks.is_empty(), "no box, browser or terminal requested");
    assert_eq!(shown, Vec::<String>::new());
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
    fs::write(tui.join("terminal/01.html"), "old TUI print").expect("old page");
    fs::write(tui.join("terminal/01.txt"), "old TUI text").expect("old text");
    let stale = stale(&web, &tui).expect("replay both roots");
    let expected = vec![Journey { client: Client::Web, name: "browser-link".into() }];
    assert_eq!(stale, expected);
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = write(
        &web,
        &tui,
        &stale,
        |journey| {
            walks.push(Journey { client: journey.client, name: journey.name.clone() });
            Err("at step 1: fake control is absent\nfake diagnostic".into())
        },
        |line| shown.push(line.to_owned()),
    )
    .expect("print the other journeys");
    assert_eq!(walks, expected);
    assert_eq!(shown, ["stopped browser-link: at step 1: fake control is absent; fake diagnostic"]);
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
    assert_eq!(stale, vec![]);
    assert!(!write(&web, &tui, &stale, |_| Err("no walk".into()), |_| {}).expect("remove orphan pages"));
    assert!(expected == tree_bytes(scratch.path()), "only numbered pages removed");
}

#[test]
fn both_clients_stale_walks_are_in_name_order_and_successes_reprint() {
    let scratch = Scratch::new("pages-walked");
    let (web, tui) = roots(&scratch);
    fs::rename(tui.join("terminal"), tui.join("a-terminal")).expect("TUI journey sorts before the browser");
    let browser = fs::read(web.join("browser-link/recording")).expect("current browser recording");
    let terminal = fs::read(tui.join("a-terminal/recording")).expect("current TUI recording");
    stale_browser(&web);
    let mut lines = tj::lines(&terminal).expect("TUI records");
    let deadline = lines
        .iter_mut()
        .find_map(|line| if let tj::Line::Deadline(deadline) = line { Some(deadline) } else { None })
        .expect("a TUI deadline");
    *deadline = Some(skein_lib::Time::ZERO);
    fs::write(tui.join("a-terminal/recording"), temper_client_journeys_kit::tui_written(&lines)).expect("stale TUI");
    let stale = stale(&web, &tui).expect("stale clients");
    let expected = vec![
        Journey { client: Client::Tui, name: "a-terminal".into() },
        Journey { client: Client::Web, name: "browser-link".into() },
    ];
    assert_eq!(stale, expected);
    let mut walks = Vec::new();
    let mut shown = Vec::new();
    let unprinted = write(
        &web,
        &tui,
        &stale,
        |journey| {
            walks.push(Journey { client: journey.client, name: journey.name.clone() });
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
    assert_eq!(shown, ["walked again a-terminal", "walked again browser-link"]);
    assert!(!unprinted, "exit 0 after both current recordings print");
    assert_eq!(super::stale(&web, &tui).expect("replay walked recordings"), vec![]);
}

#[test]
fn printing_and_orphan_removal_refuse_filesystem_errors() {
    let scratch = Scratch::new("pages-errors");
    let (web, tui) = roots(&scratch);
    let blocked = scratch.path().join("blocked");
    fs::write(&blocked, "not a root directory").expect("blocked root");
    assert!(reprint(&blocked, &tui).is_err(), "print failure reaches the command's exit 3");
    let orphan = web.join("unrecorded");
    fs::create_dir(&orphan).expect("orphan directory");
    let recording = orphan.join("recording");
    std::os::unix::fs::symlink("recording", &recording).expect("unreadable recording path");
    assert!(matches!(remove_orphans(&web, Client::Web), Err(Refusal::Io { path }) if path == recording));
}
