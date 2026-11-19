//! Walks of the tests' own, never the checkout's journeys (journey-walks.md, 7).
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use temper_client_walk_tool::{Answer, TUI, WEB};

const W: &str = "walk 2\nweb see \"Try the playground\"\nweb mark\nweb press button \"Try the playground\"\nweb see \"New chat\"\nweb mark\n";
struct Fixture {
    scratch: skein_scratch::Scratch,
}
impl Fixture {
    fn new() -> Self {
        let fixture = Self { scratch: skein_scratch::Scratch::new("walk-tool") };
        fs::create_dir_all(fixture.root().join(WEB)).expect("web root");
        fs::create_dir_all(fixture.root().join(TUI)).expect("tui root");
        fixture.journey("w", W);
        fixture
    }
    fn root(&self) -> &Path {
        self.scratch.path()
    }
    fn journey(&self, name: &str, walk: &str) {
        let path = self.root().join(WEB).join(name);
        fs::create_dir_all(&path).expect("journey directory");
        fs::write(path.join("walk"), walk).expect("walk");
    }
    fn run(&self, arguments: &[&str]) -> Answer {
        let arguments = arguments.iter().map(|arg| arg.as_bytes().into()).collect::<Vec<_>>();
        temper_client_walk_tool::answer(self.root(), &arguments, "test", "unknown")
    }
    fn print(&self, out: &Path, names: &[&str]) -> Answer {
        let mut arguments = vec!["print", "--out", out.to_str().expect("UTF-8 path")];
        arguments.extend_from_slice(names);
        self.run(&arguments)
    }
}
fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root).expect("read directory") {
        let entry = entry.expect("entry");
        if entry.file_type().expect("type").is_dir() {
            for (path, bytes) in bytes(&entry.path()) {
                result.insert(PathBuf::from(entry.file_name()).join(path), bytes);
            }
        } else {
            result.insert(PathBuf::from(entry.file_name()), fs::read(entry.path()).expect("file"));
        }
    }
    result
}
#[expect(clippy::disallowed_types, reason = "filesystem modification times are opaque observations, not a test clock")]
fn times(root: &Path) -> BTreeMap<PathBuf, SystemTime> {
    bytes(root)
        .keys()
        .map(|path| (path.clone(), fs::metadata(root.join(path)).expect("metadata").modified().expect("mtime")))
        .collect()
}
fn ok(answer: &Answer) {
    assert_eq!(answer.code, 0, "{answer:?}");
    assert!(answer.stderr.is_empty(), "{answer:?}");
}

#[test]
fn walked_and_printed_marks_link_the_pressed_control() {
    let f = Fixture::new();
    let out = f.root().join("out");
    let answer = f.print(&out, &["w"]);
    ok(&answer);
    assert_eq!(answer.stdout, format!("printed w\nprinted into {}\n", out.display()));
    let pages = bytes(&out.join(WEB).join("w"));
    assert_eq!(pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(), ["01.html", "02.html"]);
    let first = String::from_utf8(pages[Path::new("01.html")].clone()).expect("HTML");
    let linked = first
        .split('<')
        .find(|tag| tag.starts_with("button ") && tag.contains("data-journey-next=\"02.html\""))
        .expect("linked button");
    assert!(linked.contains("Try the playground"), "{linked}");
    assert!(!first.contains("data-journey-act="));
    assert!(first.contains("Try the playground"));
    assert!(String::from_utf8_lossy(&pages[Path::new("02.html")]).contains("New chat"));
}

#[test]
fn two_prints_have_identical_bytes() {
    let f = Fixture::new();
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &[]));
    ok(&f.print(&b, &[]));
    let first = bytes(&a);
    let second = bytes(&b);
    assert_eq!(first.keys().collect::<Vec<_>>(), second.keys().collect::<Vec<_>>());
    for (path, bytes) in first {
        assert!(second[&path].eq(&bytes), "bytes differ at {}", path.display());
    }
}

#[test]
fn owned_setup_marks_a_live_step_and_the_end_with_repeatable_pages() {
    let f = Fixture::new();
    f.journey(
        "owned",
        r#"walk 2
web press link "Set up temper"
web see "Forgejo"
web press link "Run on this machine"
web press button "Run on this machine"
web see "Fetching Forgejo \xc2\xb7 25%"
web mark
web see "work/hello made"
web mark
"#,
    );
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["owned"]));
    ok(&f.print(&b, &["owned"]));
    let pages = bytes(&a.join(WEB).join("owned"));
    assert_eq!(pages, bytes(&b.join(WEB).join("owned")), "both prints have the same page bytes");
    assert_eq!(pages.len(), 2);
    let first = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(first.contains("Fetching Forgejo · 25%"));
    assert!(first.contains("under way"));
    let last = String::from_utf8_lossy(&pages[Path::new("02.html")]);
    assert!(last.contains("work/hello made"));
    assert!(last.contains("connected as temper"));
}

#[test]
fn stopped_walks_and_unreadable_chains_do_not_stop_other_journeys() {
    let f = Fixture::new();
    f.journey("x", "walk 2\nweb press button \"No such control\"\n");
    f.journey("y", "walk 2\nafter missing\nweb mark\n");
    let out = f.root().join("out");
    let answer = f.print(&out, &["x", "w", "y"]);
    assert_eq!(answer.code, 1, "{answer:?}");
    assert!(answer.stdout.starts_with("stopped x: x line 2: button \"No such control\" not found within 60 s\nprinted w\nstopped y: y line 2: after \"missing\": missing journey\n"), "{answer:?}");
    assert!(out.join(WEB).join("w/02.html").is_file());
    assert!(!out.join(WEB).join("x").exists() && !out.join(WEB).join("y").exists());
}

#[test]
fn ancestors_marks_are_not_printed_and_old_numbered_pages_are_removed() {
    let f = Fixture::new();
    f.journey("p", "walk 2\nweb see \"Try the playground\"\nweb mark\nweb press button \"Try the playground\"\nweb see \"New chat\"\nweb mark\n");
    f.journey("c", "walk 2\nafter p\nweb mark\n");
    let out = f.root().join("out");
    let directory = out.join(WEB).join("c");
    fs::create_dir_all(&directory).expect("old pages");
    for name in ["01.html", "02.html", "03.html", "notes.html"] {
        fs::write(directory.join(name), "old").expect("old file");
    }
    ok(&f.print(&out, &["c"]));
    let printed = bytes(&directory);
    assert_eq!(printed.len(), 2);
    assert!(String::from_utf8_lossy(&printed[Path::new("01.html")]).contains("New chat"));
    assert_eq!(printed[Path::new("notes.html")], b"old");
    assert!(!out.join(WEB).join("p").exists());
    // The two-mark print replaces its own stale third page, too.
    let directory = out.join(WEB).join("w");
    fs::create_dir_all(&directory).expect("old w");
    fs::write(directory.join("03.html"), "old").expect("third page");
    ok(&f.print(&out, &["w"]));
    assert_eq!(bytes(&directory).len(), 2);
}

#[test]
fn printed_pages_resolve_styles_scripts_and_relative_css_urls() {
    let f = Fixture::new();
    let shell = f.root().join("temper/crates/temper-client-shell-web");
    fs::create_dir_all(shell.join("files")).expect("stylesheet tree");
    fs::write(shell.join("temper.css"), "body { background: url('files/paper.svg'); }\n").expect("css");
    fs::write(shell.join("task-pages.css"), "p {}\n").expect("css");
    fs::write(shell.join("files/paper.svg"), "<svg/>\n").expect("picture");
    fs::write(f.root().join(WEB).join("overlay.css"), "body {color: red}\n").expect("overlay");
    let out = f.root().join("out");
    ok(&f.print(&out, &["w"]));
    for name in ["01.html", "02.html"] {
        let page = out.join(WEB).join("w").join(name);
        let document = fs::read_to_string(&page).expect("page");
        for tag in document
            .split('<')
            .filter(|tag| tag.starts_with("link rel=\"stylesheet\"") || tag.starts_with("script src="))
        {
            let attribute = if tag.starts_with("link") { "href=\"" } else { "src=\"" };
            let url = tag.split_once(attribute).expect("attribute").1.split('"').next().expect("url");
            let asset = page.parent().expect("page parent").join(url);
            assert!(asset.is_file(), "{} missing", asset.display());
            if asset.extension().is_some_and(|ext| ext == "css") {
                let css = fs::read_to_string(&asset).expect("stylesheet");
                for url in css.split("url(").skip(1) {
                    let url = url.split(')').next().expect("url end").trim_matches(['\'', '"']);
                    assert!(asset.parent().expect("css parent").join(url).is_file(), "missing CSS url {url}");
                }
            }
        }
    }
    assert_eq!(fs::read(out.join(WEB).join("overlay.css")).expect("copied overlay"), b"body {color: red}\n");
}

#[test]
fn pages_writes_nothing_when_fresh_and_repairs_only_the_changed_page() {
    let f = Fixture::new();
    f.journey("w", "walk 2\nweb see \"Try the playground\"\nweb mark\nweb mark\n");
    let first = f.run(&["pages"]);
    ok(&first);
    assert_eq!(first.stdout, "printed w\n");
    let before = bytes(f.root());
    let before_times = times(f.root());
    let fresh = f.run(&["pages"]);
    ok(&fresh);
    assert_eq!(fresh.stdout, "fresh w\n");
    assert_eq!(before, bytes(f.root()));
    assert_eq!(before_times, times(f.root()));
    let page = f.root().join(WEB).join("w/02.html");
    let original = fs::read_to_string(&page).expect("page");
    fs::write(&page, original.replacen("Try the playground", "Changed playground", 1)).expect("hand edit");
    let repaired = f.run(&["pages"]);
    ok(&repaired);
    assert_eq!(repaired.stdout, "printed w\n");
    assert_eq!(fs::read_to_string(&page).expect("repaired"), original);
    let mut expected_times = before_times;
    expected_times.remove(Path::new(&format!("{WEB}/w/02.html")));
    let mut after_times = times(f.root());
    after_times.remove(Path::new(&format!("{WEB}/w/02.html")));
    assert_eq!(expected_times, after_times, "only the changed page is written");
}

#[test]
fn pages_removes_recording_and_preserves_unmoved_and_stopped_journeys() {
    let f = Fixture::new();
    fs::write(f.root().join(WEB).join("w/recording"), "old recording").expect("recording");
    let unmoved = f.root().join(WEB).join("unmoved");
    fs::create_dir(&unmoved).expect("unmoved");
    fs::write(unmoved.join("recording"), "keep").expect("recording");
    fs::write(unmoved.join("01.html"), "keep page").expect("page");
    f.journey("x", "walk 2\nweb press button \"No such control\"\n");
    let stopped = f.root().join(WEB).join("x");
    fs::write(stopped.join("01.html"), "keep stopped page").expect("page");
    fs::write(stopped.join("recording"), "keep stopped recording").expect("recording");
    let kept = bytes(&unmoved);
    let stopped_before = bytes(&stopped);
    let stopped_times = times(&stopped);
    let answer = f.run(&["pages"]);
    assert_eq!(answer.code, 1, "{answer:?}");
    assert!(!f.root().join(WEB).join("w/recording").exists());
    assert_eq!(kept, bytes(&unmoved));
    assert_eq!(stopped_before, bytes(&stopped));
    assert_eq!(stopped_times, times(&stopped));
}

#[test]
fn usage_refuses_unmoved_names_and_output_inside_either_root_even_through_links() {
    let f = Fixture::new();
    let link = f.root().join("alias");
    std::os::unix::fs::symlink(f.root().join(TUI), &link).expect("root link");
    for arguments in [vec!["print", "not-moved"], vec!["unknown-command"]] {
        let answer = f.run(&arguments);
        assert_eq!(answer.code, 2, "{answer:?}");
        assert_eq!(answer.stdout, "");
    }
    for out in [
        f.root().join(WEB).join("new"),
        f.root().join(TUI).join("new/../other"),
        link.join("new"),
        link.join("../journeys/new"),
    ] {
        let answer = f.print(&out, &["w"]);
        assert_eq!(answer.code, 2, "{}: {answer:?}", out.display());
        assert_eq!(answer.stdout, "");
    }
}

#[test]
fn submit_on_a_textbox_links_by_its_act() {
    let f = Fixture::new();
    f.journey("s", "walk 2\nweb see \"Try the playground\"\nweb press button \"Try the playground\"\nweb see \"New chat\"\nweb input textbox \"What do you want done?\" \"Add CSV export to widget\"\nweb mark\nweb submit textbox \"What do you want done?\"\nweb mark\n");
    let out = f.root().join("out");
    ok(&f.print(&out, &["s"]));
    let first = fs::read_to_string(out.join(WEB).join("s/01.html")).expect("first");
    let linked = first
        .split('<')
        .find(|tag| {
            tag.starts_with("textarea ") && tag.contains("data-journey-next=\"02.html\" data-journey-act=\"submit\"")
        })
        .expect("submit links the textarea");
    assert!(linked.contains("aria-label=\"What do you want done?\""), "{linked}");
}
