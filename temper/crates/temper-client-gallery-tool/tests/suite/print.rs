use skein_scratch::Scratch;
use std::{collections::BTreeMap, fs, path::Path};
use temper_client_gallery_tool::{STYLESHEETS, print_with, samples};

#[test]
#[expect(clippy::too_many_lines, reason = "the print walk checks every sample and its shared bundle")]
fn print_writes_every_page_and_the_bundle() {
    let scratch = Scratch::new("temper-client-gallery-print");
    let out = scratch.path().join("gallery");
    fs::create_dir_all(&out).expect("previous gallery");
    fs::write(out.join("index.html"), b"previous index").expect("previous index");
    let stale = out.join("stale.html");
    fs::write(&stale, b"obsolete").expect("stale page");
    let mut paths = BTreeMap::new();
    print_with(&out, |sample, tree| {
        let count = tree
            .nodes()
            .iter()
            .filter(|node| node.drawing.as_ref().is_some_and(|drawing| drawing.kind == temper_client_view::paths::KIND))
            .count();
        paths.insert(sample.name, count);
    })
    .expect("replace gallery");
    assert!(!stale.exists(), "old output removed");
    let index = fs::read_to_string(out.join("index.html")).expect("index");
    let bundle = out.join("bundle");
    let mut digest = None;
    for entry in fs::read_dir(&bundle).expect("bundle") {
        let entry = entry.expect("bundle entry");
        if entry.file_type().expect("file type").is_dir() {
            assert!(digest.is_none(), "one digest");
            digest = Some(entry.file_name().into_string().expect("digest ASCII"));
        }
    }
    let digest = digest.expect("digest directory");
    for file in ["menu.svg", "send.svg"] {
        assert!(bundle.join(&digest).join("files").join(file).is_file(), "bundled decorative image {file}");
    }
    for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
        assert!(bundle.join(&digest).join(file).is_file(), "bundled {file}");
    }
    let mut position = 0;
    for sample in samples::samples() {
        let link = format!("href=\"{}.html\"", sample.name);
        assert_eq!(index.matches(&link).count(), 1, "one link for {}", sample.name);
        let at = index.find(&link).expect("index link");
        assert!(at >= position, "table order");
        position = at + link.len();
        let page = fs::read_to_string(out.join(format!("{}.html", sample.name))).expect("sample page");
        let paths = paths[&sample.name];
        assert_eq!(page.matches("data-gallery-narrow=\"\"").count(), paths, "{}: narrow paths", sample.name);
        for svg in page.split("<svg").skip(1) {
            let (attributes, _) = svg.split_once('>').expect("SVG attributes");
            if attributes.contains("data-gallery-narrow=\"\"") {
                assert!(attributes.contains("viewBox=\"0 0 388 "), "{}: narrow viewBox", sample.name);
            }
        }
        assert_eq!(
            page.matches("@media (max-width: 760px)").count(),
            usize::from(paths > 0),
            "{}: narrow media rule",
            sample.name
        );
        let night = sample.name.starts_with("night-");
        assert_eq!(page.contains("<html lang=\"en\" data-night>"), night, "{}: night attribute", sample.name);
        assert_eq!(page.contains("<html lang=\"en\">"), !night, "{}: day root", sample.name);
        if matches!(sample.address, temper_client_domain::Address::Welcome { .. }) {
            assert!(page.contains("class=\"temper-top\""), "welcome has a top bar");
            assert!(!page.contains("temper-menu"), "welcome has no menu");
            assert!(page.contains("<title>Welcome · first run · temper</title>"), "{}: welcome title", sample.name);
        } else if sample.name.starts_with("chat-terminal") {
            let (_, main) = page.split_once("<main ").expect("terminal main element");
            let (main, _) = main.split_once('>').expect("terminal main attributes");
            assert!(main.contains("class=\"temper-terminal "), "{}: terminal main", sample.name);
            for class in ["temper-rail", "temper-tabbar", "temper-menu"] {
                assert!(!page.contains(&format!("class=\"{class}\"")), "{}: no {class}", sample.name);
            }
        } else if page.contains("class=\"temper-rail\"") {
            assert!(page.contains("class=\"temper-tabbar"), "{}: project sections", sample.name);
        } else {
            assert!(page.contains("class=\"temper-top\""), "{}: top bar", sample.name);
            assert!(!page.contains("temper-menu"), "{}: no menu", sample.name);
        }
        let mut position = 0;
        for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
            let link = format!("href=\"bundle/{digest}/{file}\"");
            let at = page.find(&link).expect("stylesheet linked");
            assert!(at >= position, "stylesheet order");
            position = at + link.len();
        }
    }
    for name in ["goal-landed", "goal-landed-open"] {
        let page = fs::read_to_string(out.join(format!("{name}.html"))).expect("landed goal print");
        assert_eq!(page.contains("open class=\"temper-goal-landed-fold\""), name == "goal-landed-open");
    }
    for (name, parts) in [
        ("needs-list", ["class=\"temper-needs-bar\"", "1 of 3"]),
        ("ask-chat", ["class=\"temper-tabbar temper-tabbar-ask\"", ">Ask</span"]),
        ("rail-new-change", ["aria-label=\"Needs\"", ">Needs</span"]),
    ] {
        let page = fs::read_to_string(out.join(format!("{name}.html"))).expect("page print");
        for part in parts {
            assert!(page.contains(part), "{name}: {part}");
        }
    }
    let signin = fs::read_to_string(out.join("connect-model-signing-in.html")).expect("sign-in page");
    assert!(signin.contains("href=\"https://issuer.example/visit/gallery-sign-in\""), "external sign-in href");
    let closing = fs::read_to_string(out.join("chat-closing.html")).expect("confirmation page");
    assert!(closing.contains("<dialog open "), "static confirmation opens");
    let chats = fs::read_to_string(out.join("chats-solo.html")).expect("chats page");
    assert!(chats.contains("<title>Talk to an agent about robin/tally. · temper</title>"), "shell heading title");
    let signed_out = fs::read_to_string(out.join("signed-out.html")).expect("signed-out page");
    assert!(signed_out.contains("<title>Signed out · temper</title>"), "shell heading title");
    let mut expected = Vec::new();
    for path in STYLESHEETS {
        expected.push(Path::new(path).file_name().expect("stylesheet basename").to_str().expect("ASCII"));
    }
    let mut actual = Vec::new();
    for line in include_str!("../../../../scripts/bundle.sh").lines() {
        let line = line.trim();
        if line == "else" {
            break;
        }
        if line.starts_with("--stylesheet ") {
            let (_, path) = line.split_once("$shell/").expect("shell stylesheet");
            actual.push(path.split('"').next().expect("basename"));
        }
    }
    assert_eq!(expected, actual);
}
