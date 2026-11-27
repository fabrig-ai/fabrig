//! Walks of the tests' own, never the checkout's journeys (client/journey-walks.md, 7).
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use temper_client_walk_tool::{Answer, TUI, WEB};

const W: &str = "walk 2\nweb see \"Try the playground\"\nweb mark\nweb press button \"Try the playground\"\nweb see \"New chat\"\nweb mark\n";
const T: &str = "walk 2\ntui see \"then press Approve.\"\ntui mark\ntui approve\ntui see \"Setting temper up makes your first project\"\ntui mark\n";
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
    fn scripted_chain() -> Self {
        let fixture = Self::new();
        fixture.journey("a", W);
        fixture.journey(
            "b",
            r#"walk 2
after a
web input textbox "What do you want done?" "first question"
web press button "Send"
web see "Answer from a."
web input textbox "Message" "second question"
web press button "Send"
web see "Answer from b."
web mark
"#,
        );
        fs::write(
            fixture.root().join(WEB).join("a/scripts.json"),
            r#"{"asked":[{"cue":"first question","turns":["Answer from a."]}],"otherwise":["Otherwise."]}"#,
        )
        .expect("a scripts");
        fs::write(
            fixture.root().join(WEB).join("b/scripts.json"),
            r#"{"asked":[{"cue":"second question","turns":["Answer from b."]}]}"#,
        )
        .expect("b scripts");
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
    fn tui_journey(&self, name: &str, walk: &str) {
        let path = self.root().join(TUI).join(name);
        fs::create_dir_all(&path).expect("TUI journey directory");
        fs::write(path.join("walk"), walk).expect("TUI walk");
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
fn tui_walks_print_frames_and_assets_under_the_tui_root() {
    let f = Fixture::new();
    f.tui_journey("t", T);
    fs::write(f.root().join(TUI).join("overlay.css"), "pre {font-size: 14px}\n").expect("overlay");
    let out = f.root().join("out");
    let answer = f.print(&out, &["t"]);
    ok(&answer);
    assert_eq!(answer.stdout, format!("printed t\nprinted into {}\n", out.display()));
    let pages = bytes(&out.join(TUI).join("t"));
    assert_eq!(
        pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(),
        ["01.html", "01.txt", "02.html", "02.txt"]
    );
    let first = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(first.contains("<pre class=\"tui tui-dark\""), "{first}");
    assert!(!first.contains("data-journey-key"), "approval, not a key, reaches the next mark");
    assert!(String::from_utf8_lossy(&pages[Path::new("01.txt")]).contains("then press Approve."));
    assert!(
        String::from_utf8_lossy(&pages[Path::new("02.txt")]).contains("Setting temper up makes your first project")
    );
    assert!(!String::from_utf8_lossy(&pages[Path::new("02.html")]).contains("data-journey-key"));
    for name in ["theme.css", "tui-journey.js", "overlay.css"] {
        assert!(out.join(TUI).join(name).is_file(), "{name}");
    }
    assert_eq!(fs::read(out.join(TUI).join("overlay.css")).expect("copied overlay"), b"pre {font-size: 14px}\n");
    assert!(!out.join(WEB).exists(), "only the printed root's assets go to scratch");
}

#[test]
fn an_owned_forge_link_prints_the_same_bytes_twice() {
    let f = Fixture::new();
    f.journey(
        "owned-link",
        r#"walk 2
web press link "Set up temper"
web press link "Run on this machine"
web press button "Run on this machine"
web press button "Continue with ChatGPT"
web press button "Create your project"
web see "work/hello is in Hello"
web reload "/projects/1"
web see "work/hello \xe2\x86\x97"
web mark
"#,
    );
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["owned-link"]));
    ok(&f.print(&b, &["owned-link"]));
    let first = bytes(&a);
    let second = bytes(&b);
    assert_eq!(first.keys().collect::<Vec<_>>(), second.keys().collect::<Vec<_>>());
    let page = Path::new(WEB).join("owned-link/01.html");
    let html = String::from_utf8_lossy(&first[&page]);
    assert!(html.contains("http://127.0.0.1:10002/user/oauth2/temper?redirect_to=%2Fwork%2Fhello"));
    for (path, bytes) in first {
        assert!(second[&path].eq(&bytes), "bytes differ at {}", path.display());
    }
}

#[test]
fn tui_prints_twice_have_identical_bytes() {
    let f = Fixture::new();
    f.tui_journey("t", T);
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["t"]));
    ok(&f.print(&b, &["t"]));
    let first = bytes(&a);
    let second = bytes(&b);
    assert_eq!(first.keys().collect::<Vec<_>>(), second.keys().collect::<Vec<_>>());
    for (path, bytes) in first {
        assert!(second[&path].eq(&bytes), "bytes differ at {}", path.display());
    }
}

#[test]
fn tui_keys_link_the_marked_frame_to_the_next_page() {
    let f = Fixture::new();
    f.tui_journey("t", "walk 2\ntui approve\ntui see \"Setting temper up makes your first project\"\ntui key paste \"/theme\"\ntui mark\ntui key enter\ntui mark\n");
    let out = f.root().join("out");
    ok(&f.print(&out, &["t"]));
    let first = fs::read_to_string(out.join(TUI).join("t/01.html")).expect("first");
    assert!(first.contains("data-journey-key=\"enter\" data-journey-next=\"02.html\""), "{first}");
    let last = fs::read_to_string(out.join(TUI).join("t/02.html")).expect("last");
    assert!(!last.contains("data-journey-key"));
    assert!(fs::read_to_string(out.join(TUI).join("t/02.txt")).expect("theme frame").contains("Theme"));
}

#[test]
fn a_tui_drafts_pane_cycles_and_key_links_print_repeatably() {
    let f = Fixture::new();
    f.tui_journey(
        "pane",
        r#"walk 2
after w
tui approve
tui see "/draft    a design session"
tui key paste "/draft Design a small checklist."
tui key enter
tui see "Tell it something else"
tui see "checklist.md"
tui mark
tui key control "o"
tui see "The last checkpoint is visible here."
tui mark
tui key control "o"
tui see "Which approach should we take?"
tui mark
tui key control "o"
tui see "checklist.md"
tui mark
"#,
    );
    // The ancestor reaches the playground: its first chat is T1 in project 1.
    fs::write(
        f.root().join(TUI).join("pane/scripts.json"),
        r##"{
  "cued": [{
    "cue": "Design a small checklist.",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 1},
        "description": "Checklist",
        "body": "# Checklist\n\n## Steps\n\nRead the request carefully before choosing the smallest useful change.\n\nKeep the person's existing authority and budget throughout the work.\n\nMake each decision clear enough that another person can follow it.\n\nCheck the result against the request before offering the finished work.\n\nLeave the surrounding code intact when a small local edit will suffice.\n\nName each check so another person can repeat the same observation.\n\nExplain what remains without claiming that unobserved work has passed.\n\nThe last checkpoint is visible here.\n"
      }}], "finish": "tool_calls", "tokens": 100
    }, "Which approach should we take?\n\n1. Use it — keep the checklist\n2. Change it — revise the checklist\n3. Tell me something else"]
  }],
  "otherwise": ["No scripted request matched."]
}"##,
    )
    .expect("pane scripts");
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["pane"]));
    ok(&f.print(&b, &["pane"]));
    assert_eq!(bytes(&a), bytes(&b), "both prints include identical frames, HTML and assets");
    let pages = bytes(&a.join(TUI).join("pane"));
    assert_eq!(pages.len(), 8, "four marks each print one frame and one HTML page");
    let half = String::from_utf8_lossy(&pages[Path::new("01.txt")]);
    assert!(half.contains("checklist.md"), "{half}");
    assert!(!half.contains("The last checkpoint is visible here."), "{half}");
    let all = String::from_utf8_lossy(&pages[Path::new("02.txt")]);
    assert!(all.contains("The last checkpoint is visible here."), "{all}");
    let folded = String::from_utf8_lossy(&pages[Path::new("03.txt")]);
    for document_words in [
        "checklist.md",
        "Steps",
        "Read the request",
        "Keep the person's",
        "Make each decision",
        "Check the result",
        "Leave the surrounding",
        "Name each check",
        "Explain what remains",
        "The last checkpoint",
    ] {
        assert!(!folded.contains(document_words), "folded document: {folded}");
    }
    let below_box = folded.rsplit('─').next().expect("choices below the composer's rule");
    assert!(below_box.contains("1. Use it"), "{folded}");
    assert!(below_box.contains("2. Change it"), "{folded}");
    assert!(below_box.contains("3. Tell it something else"), "{folded}");
    let half_again = String::from_utf8_lossy(&pages[Path::new("04.txt")]);
    assert_eq!(half, half_again, "the pane returns to the same half frame");
    for (page, next) in [("01.html", "02.html"), ("02.html", "03.html"), ("03.html", "04.html")] {
        let html = String::from_utf8_lossy(&pages[Path::new(page)]);
        assert!(html.contains(&format!("data-journey-key=\"control o\" data-journey-next=\"{next}\"")), "{html}");
    }
    assert!(!String::from_utf8_lossy(&pages[Path::new("04.html")]).contains("data-journey-key"));
}

#[test]
fn a_delegated_question_walks_its_choices_and_prints_repeatably() {
    let f = Fixture::new();
    f.journey("project", W);
    f.tui_journey(
        "question",
        r#"walk 2
after project
tui approve
tui see "Ask for a change, or ask temper."
tui key paste "/change Choose a report format"
tui key enter
tui see "A report format question waits for you."
tui see "report-format: text or table?"
tui key paste "/new"
tui key enter
tui see "/needs"
tui see "Choose a report format: report-format"
tui mark
tui key paste "/needs"
tui key enter
tui see "Next"
tui mark
tui key enter
tui see "Nothing else needs you."
tui mark
"#,
    );
    // The playground's first chat is T1; its delegated goal is T2.
    fs::write(
        f.root().join(TUI).join("question/scripts.json"),
        r#"{
  "asked": [{
    "cue": "Choose a report format",
    "turns": [{
      "lines": [{"call": "goal", "arguments": {"words": "Ask which report format."}}],
      "finish": "tool_calls", "tokens": 32
    }, "A report format question waits for you."]
  }],
  "cued": [{
    "cue": "Ask which report format.",
    "turns": [{
      "lines": [{"call": "message", "arguments": {
        "target": 1, "form": "question",
        "words": "report-format: text or table?\nChoice: Text (recommended)\nChoice: Table"
      }}], "finish": "tool_calls", "tokens": 32
    }, {
      "lines": [{"call": "wait", "arguments": {}}],
      "finish": "tool_calls", "tokens": 16
    }, {
      "lines": [{"call": "finish", "arguments": {
        "form": "report", "text": "The report format is agreed.", "fields": {}
      }}], "finish": "tool_calls", "tokens": 16
    }],
    "inputs": [{"cue": "Text", "turn": 2}, {"cue": "Table", "turn": 2},
               {"cue": "Message ", "turn": 1}, {"cue": "", "turn": 0}]
  }],
  "otherwise": ["No scripted request matched."]
}"#,
    )
    .expect("question scripts");
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["question"]));
    ok(&f.print(&b, &["question"]));
    assert_eq!(bytes(&a), bytes(&b), "both prints have the same bytes, including assets");
    let pages = bytes(&a.join(TUI).join("question"));
    assert_eq!(
        pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(),
        ["01.html", "01.txt", "02.html", "02.txt", "03.html", "03.txt"]
    );
    let launchpad = String::from_utf8_lossy(&pages[Path::new("01.txt")]);
    assert!(launchpad.contains("needs you · /needs"), "{launchpad}");
    let launchpad_html = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(launchpad_html.contains("rel=\"next\" href=\"02.html\""), "{launchpad_html}");
    assert!(!launchpad_html.contains("data-journey-key"), "the mark precedes typing /needs");
    let prompt = String::from_utf8_lossy(&pages[Path::new("02.txt")]);
    assert!(prompt.contains("report-format: text or table?"), "{prompt}");
    assert!(prompt.contains("❯ 1. Text"), "the recommended choice is active: {prompt}");
    assert!(prompt.contains("2. Table"), "{prompt}");
    let prompt_html = String::from_utf8_lossy(&pages[Path::new("02.html")]);
    assert!(prompt_html.contains("data-journey-key=\"enter\" data-journey-next=\"03.html\""), "{prompt_html}");
    let answered = String::from_utf8_lossy(&pages[Path::new("03.txt")]);
    assert!(answered.contains("Nothing else needs you."), "{answered}");
    assert!(!answered.contains("Needs you"), "{answered}");
    assert!(!answered.contains("needs you · /needs"), "{answered}");
    assert!(!String::from_utf8_lossy(&pages[Path::new("03.html")]).contains("data-journey-key"));
}

#[test]
fn a_tui_stop_names_its_line_and_leaves_other_journeys_printable() {
    let f = Fixture::new();
    f.tui_journey("t", "walk 2\ntui mark\ntui see \"Never drawn\"\ntui mark\n");
    let out = f.root().join("out");
    let directory = out.join(TUI).join("t");
    fs::create_dir_all(&directory).expect("old pages");
    fs::write(directory.join("01.txt"), "keep stopped frame").expect("frame");
    fs::write(directory.join("recording"), "keep stopped recording").expect("recording");
    let before = bytes(&directory);
    let before_times = times(&directory);
    let answer = f.print(&out, &["t", "w"]);
    assert_eq!(answer.code, 1, "{answer:?}");
    assert!(answer.stderr.is_empty(), "{answer:?}");
    assert_eq!(
        answer.stdout,
        format!(
            "stopped t: t line 3: \"Never drawn\" not seen in the TUI within 30 s\nprinted w\nprinted into {}\n",
            out.display()
        )
    );
    assert_eq!(bytes(&directory), before);
    assert_eq!(times(&directory), before_times);
    assert!(out.join(WEB).join("w/02.html").is_file());
    assert!(!out.join(TUI).join("theme.css").exists());
}

#[test]
fn tui_pages_remove_stale_frames_and_recording_then_write_nothing_when_fresh() {
    let f = Fixture::new();
    f.tui_journey("t", T);
    let directory = f.root().join(TUI).join("t");
    for name in ["03.txt", "03.html", "recording"] {
        fs::write(directory.join(name), "stale").expect("stale file");
    }
    let overlay = f.root().join(TUI).join("overlay.css");
    fs::write(&overlay, "pre {color: red}\n").expect("overlay");
    let overlay_time = fs::metadata(&overlay).expect("metadata").modified().expect("mtime");
    let first = f.run(&["pages"]);
    ok(&first);
    assert_eq!(first.stdout, "printed w\nprinted t\n");
    for name in ["03.txt", "03.html", "recording"] {
        assert!(!directory.join(name).exists(), "{name} removed");
    }
    assert!(directory.join("02.txt").is_file());
    assert_eq!(fs::read(&overlay).expect("overlay kept"), b"pre {color: red}\n");
    assert_eq!(fs::metadata(&overlay).expect("metadata").modified().expect("mtime"), overlay_time);
    let before = bytes(f.root());
    let before_times = times(f.root());
    let fresh = f.run(&["pages"]);
    ok(&fresh);
    assert_eq!(fresh.stdout, "fresh w\nfresh t\n");
    assert_eq!(bytes(f.root()), before);
    assert_eq!(times(f.root()), before_times);
}

#[test]
fn all_prints_visit_both_roots_in_order_and_names_prefer_the_browser() {
    let f = Fixture::new();
    f.journey("a", "walk 2\nweb mark\n");
    f.tui_journey("a", "walk 2\ntui see \"Never drawn\"\n");
    f.tui_journey("z", "walk 2\ntui mark\n");
    f.tui_journey("t", "walk 2\ntui mark\n");
    let out = f.root().join("out");
    let answer = f.print(&out, &[]);
    ok(&answer);
    assert_eq!(answer.stdout, format!("printed a\nprinted w\nprinted t\nprinted z\nprinted into {}\n", out.display()));
    assert!(out.join(WEB).join("a/01.html").is_file());
    assert!(!out.join(TUI).join("a").exists());
    assert!(out.join(TUI).join("t/01.txt").is_file());
    assert!(out.join(TUI).join("z/01.txt").is_file());
}

#[test]
fn tui_ancestors_marks_are_skipped_and_stop_restarts_the_terminal() {
    let f = Fixture::new();
    f.tui_journey("p", T);
    f.tui_journey("t", "walk 2\nafter p\ntui stop\ntui see \"Setting temper up makes your first project\"\ntui mark\n");
    let out = f.root().join("out");
    ok(&f.print(&out, &["t"]));
    let pages = bytes(&out.join(TUI).join("t"));
    assert_eq!(pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(), ["01.html", "01.txt"]);
    assert!(
        String::from_utf8_lossy(&pages[Path::new("01.txt")]).contains("Setting temper up makes your first project")
    );
    assert!(!out.join(TUI).join("p").exists());
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
fn scripts_along_a_chain_answer_both_questions_with_repeatable_pages() {
    let f = Fixture::scripted_chain();
    let first = f.root().join("first");
    let second = f.root().join("second");
    ok(&f.print(&first, &["b"]));
    ok(&f.print(&second, &["b"]));
    assert_eq!(bytes(&first), bytes(&second), "both prints have the same bytes");
    let pages = bytes(&first.join(WEB).join("b"));
    assert_eq!(pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(), ["01.html"]);
    let page = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(page.contains("Answer from a."), "{page}");
    assert!(page.contains("Answer from b."), "{page}");
    assert!(!first.join(WEB).join("a").exists(), "ancestor marks print nothing");
}

#[test]
fn duplicate_script_cues_stop_at_the_child_without_stopping_other_journeys() {
    let f = Fixture::scripted_chain();
    fs::write(
        f.root().join(WEB).join("b/scripts.json"),
        r#"{"asked":[{"cue":"first question","turns":["Answer from b."]}]}"#,
    )
    .expect("duplicate cue");
    let roots = temper_client_walk_lines::walk::Roots { web: f.root().join(WEB), tui: f.root().join(TUI) };
    let chain = temper_client_walk_lines::walk::read(&roots, "b").expect("read chain");
    let stopped = temper_client_walk_world::walk::walk(&chain).err().expect("duplicate scripts stop the walk");
    assert_eq!(stopped.journey, "b");
    assert_eq!(stopped.line, 0);
    assert!(stopped.why.starts_with("scripts.json:"), "{stopped:?}");
    let out = f.root().join("out");
    let answer = f.print(&out, &["b", "w"]);
    assert_eq!(answer.code, 1, "{answer:?}");
    assert!(answer.stderr.is_empty(), "{answer:?}");
    assert_eq!(answer.stdout, format!("stopped b: {stopped}\nprinted w\nprinted into {}\n", out.display()));
    assert!(!out.join(WEB).join("b").exists());
    assert!(out.join(WEB).join("w/02.html").is_file());
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
fn sign_in_reaches_ready_and_prints_the_same_bytes_twice() {
    let f = Fixture::new();
    f.journey(
        "sign-in",
        r#"walk 2
web press link "Set up temper"
web press link "Run on this machine"
web press button "Run on this machine"
web press button "Continue with ChatGPT"
web see "Ready"
web see "Everything is connected."
web mark
"#,
    );
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["sign-in"]));
    ok(&f.print(&b, &["sign-in"]));
    let first = bytes(&a);
    assert_eq!(first, bytes(&b), "both prints have the same page and asset bytes");
    let pages = bytes(&a.join(WEB).join("sign-in"));
    assert_eq!(pages.len(), 1);
    let ready = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(ready.contains("Create your project"));
    assert!(ready.contains("ChatGPT plan · sam@example.com"));
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
#[expect(clippy::too_many_lines, reason = "the owned project's walk and box-form model scripts stay together")]
fn an_owned_projects_chat_and_draft_goal_print_repeatably() {
    let f = Fixture::new();
    f.journey(
        "owned-chat",
        r#"walk 2
web press link "Set up temper"
web press link "Run on this machine"
web press button "Run on this machine"
web press button "Continue with ChatGPT"
web press button "Create your project"
web see "work/hello is in Hello"
web reload "/projects/1"
web see "Hello is ready."
web reload "/chats"
web press link "+ New chat"
web see "Say what you want; the agent works in work/hello."
web mark
web input textbox "What do you want done?" "first question"
web press button "Send"
web see "First scripted reply."
web press button "Split"
web press button "Write it up from this chat"
web see "Away mode keeps decisions for the person's return."
web see "I wrote the Away mode draft."
web press button "Chat"
web input textbox "Message" "second question"
web press button "Send"
web see "Second scripted reply."
web mark
web press button "Split"
web see "A running goal keeps going."
web press button "Start goal\xe2\x80\xa6"
web see "Start a goal from away-mode.md"
web mark
web press button "Start goal"
web see "The owned project's goal wrote its report."
web see "Nothing remains in this demonstration."
web see "landed"
web mark
web press link "Draft"
web see "First scripted reply."
web see "Second scripted reply."
web mark
"#,
    );
    // Hello is project 1: its first chat is T2, its draft is note 3
    // (revised to revision 2), and that draft starts goal T3.
    fs::write(
        f.root().join(WEB).join("owned-chat/scripts.json"),
        r##"{
  "asked": [{
    "cue": "first question",
    "turns": ["First scripted reply."]
  }, {
    "cue": "second question",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 2},
        "name": 3, "recalled": 1,
        "description": "Away mode",
        "body": "# Away mode\n\nAway mode keeps decisions for the person's return.\n\nA running goal keeps going.\n"
      }}], "finish": "tool_calls", "tokens": 64
    }, "Second scripted reply."]
  }],
  "cued": [{
    "cue": "Write this up as a draft beside us.",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 2},
        "description": "Away mode",
        "body": "# Away mode\n\nAway mode keeps decisions for the person's return.\n"
      }}], "finish": "tool_calls", "tokens": 64
    }, "I wrote the Away mode draft."]
  }, {
    "cue": "away-mode\n\nImplement this draft.",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 3},
        "description": "report: The owned project's goal wrote its report.",
        "body": "I wrote this report in Hello.\n\nLeft: Nothing remains in this demonstration.\nRisk: This scripted model does not change a repository."
      }}], "finish": "tool_calls", "tokens": 64
    }, {
      "lines": [{"call": "finish", "arguments": {
        "form": "report", "text": "The owned project's goal wrote its report.", "fields": {}
      }}], "finish": "tool_calls", "tokens": 16
    }]
  }],
  "otherwise": ["No scripted request matched."]
}"##,
    )
    .expect("model scripts");
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["owned-chat"]));
    ok(&f.print(&b, &["owned-chat"]));
    assert_eq!(bytes(&a), bytes(&b), "both prints have the same bytes, including assets");
    let pages = bytes(&a.join(WEB).join("owned-chat"));
    let new_chat = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(new_chat.contains("Say what you want; the agent works in work/hello."), "{new_chat}");
    let chat = String::from_utf8_lossy(&pages[Path::new("02.html")]);
    assert!(chat.contains("First scripted reply."), "{chat}");
    assert!(chat.contains("Second scripted reply."), "{chat}");
    let sheet = String::from_utf8_lossy(&pages[Path::new("03.html")]);
    assert!(sheet.contains("Start a goal from away-mode.md"), "{sheet}");
    assert!(sheet.contains("A running goal keeps going."), "{sheet}");
    let goal = String::from_utf8_lossy(&pages[Path::new("04.html")]);
    assert!(goal.contains("The owned project&#39;s goal wrote its report."), "{goal}");
    assert!(goal.contains("Nothing remains in this demonstration."), "{goal}");
    assert!(goal.contains("data-journey-next=\"05.html\""), "Draft links back to the chat: {goal}");
    let returned = String::from_utf8_lossy(&pages[Path::new("05.html")]);
    assert!(returned.contains("First scripted reply."), "{returned}");
    assert!(returned.contains("Second scripted reply."), "{returned}");
}

#[test]
fn a_goal_from_a_draft_prints_its_report_and_returns_to_chat_repeatably() {
    let f = Fixture::new();
    f.journey(
        "draft-goal",
        r#"walk 2
web press button "Try the playground"
web see "New chat"
web input textbox "What do you want done?" "Let us design Away mode."
web submit textbox "What do you want done?"
web see "Let us write down how Away mode works."
web press button "Split"
web press button "Write it up from this chat"
web see "Away mode keeps the person's decisions for their return."
web see "I wrote the Away mode draft."
web press button "Start goal\xe2\x80\xa6"
web see "Start a goal from away-mode.md"
web mark
web press button "Start goal"
web see "The walk goal wrote its report."
web see "Nothing remains in this demonstration."
web see "landed"
web mark
web press link "Draft"
web see "Let us write down how Away mode works."
web mark
"#,
    );
    // The playground's first chat is T1; its draft starts goal T2.
    fs::write(
        f.root().join(WEB).join("draft-goal/scripts.json"),
        r##"{
  "asked": [{
    "cue": "Let us design Away mode.",
    "turns": ["Let us write down how Away mode works."]
  }],
  "cued": [{
    "cue": "Write this up as a draft beside us.",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 1},
        "description": "Away mode",
        "body": "# Away mode\n\nAway mode keeps the person's decisions for their return.\n"
      }}], "finish": "tool_calls", "tokens": 64
    }, "I wrote the Away mode draft."]
  }, {
    "cue": "away-mode\n\nImplement this draft.",
    "turns": [{
      "lines": [{"call": "note", "arguments": {
        "scope": {"kind": "goal", "project": 1, "goal": 2},
        "description": "report: The walk goal wrote its report.",
        "body": "I wrote this report in the walk's world.\n\nLeft: Nothing remains in this demonstration.\nRisk: This scripted model does not change a repository."
      }}], "finish": "tool_calls", "tokens": 64
    }, {
      "lines": [{"call": "finish", "arguments": {
        "form": "report", "text": "The walk goal wrote its report.", "fields": {}
      }}], "finish": "tool_calls", "tokens": 16
    }]
  }],
  "otherwise": ["No scripted request matched."]
}"##,
    )
    .expect("model scripts");
    let a = f.root().join("a");
    let b = f.root().join("b");
    ok(&f.print(&a, &["draft-goal"]));
    ok(&f.print(&b, &["draft-goal"]));
    assert_eq!(bytes(&a), bytes(&b), "both prints have the same bytes, including assets");
    let pages = bytes(&a.join(WEB).join("draft-goal"));
    assert_eq!(pages.keys().map(|p| p.to_str().expect("name")).collect::<Vec<_>>(), ["01.html", "02.html", "03.html"]);
    let sheet = String::from_utf8_lossy(&pages[Path::new("01.html")]);
    assert!(sheet.contains("Start a goal from away-mode.md"), "{sheet}");
    let goal = String::from_utf8_lossy(&pages[Path::new("02.html")]);
    assert!(goal.contains("The walk goal wrote its report."), "{goal}");
    assert!(goal.contains("Nothing remains in this demonstration."), "{goal}");
    assert!(goal.contains("data-journey-next=\"03.html\""), "Draft links back to the chat: {goal}");
    let chat = String::from_utf8_lossy(&pages[Path::new("03.html")]);
    assert!(chat.contains("Let us write down how Away mode works."), "{chat}");
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
