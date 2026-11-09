use super::{Glue, Inputs, Refusal, STYLESHEET, bundle};
use skein_scratch::Scratch;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

mod worker;

fn inputs(scratch: &Scratch) -> Inputs {
    let path = scratch.path();
    fs::create_dir_all(path.join("pictures")).expect("pictures");
    fs::write(path.join("pictures/icon.svg"), b"<svg/>").expect("picture");
    fs::create_dir_all(path.join("files")).expect("files");
    fs::write(path.join("files/about.txt"), b"About").expect("file");
    fs::write(path.join("a.css"), b"body { color: navy; }").expect("stylesheet");
    Inputs {
        gzip: true,
        title: None,
        placeholder: None,
        name: "notes".into(),
        prefix: "/static/".into(),
        stylesheets: vec![path.join("a.css")],
        pictures: vec![("notes".into(), path.join("pictures"))],
        page: None,
        files: Some(path.join("files")),
        noscript: "Notes needs WebAssembly and scripts.".into(),
    }
}

fn glue() -> Glue {
    Glue {
        js: b"export default function init() {}".to_vec(),
        wasm: b"\0asm".to_vec(),
        snippets: vec![("notes/grow.js".into(), b"export function grow() {}".to_vec())],
    }
}

#[test]
fn the_digest_covers_paths_and_bytes() {
    let scratch = Scratch::new("bundle-digest");
    let mut inputs = inputs(&scratch);
    let out = scratch.path().join("out");
    let original = bundle(&inputs, &glue(), &out).expect("bundle");
    fs::write(scratch.path().join("files/about.txt"), b"about").expect("change one byte");
    let changed = bundle(&inputs, &glue(), &out).expect("bundle");
    assert_ne!(original, changed, "bytes affect the digest");
    inputs.name = "other".into();
    let renamed = bundle(&inputs, &glue(), &out).expect("bundle");
    assert_ne!(changed, renamed, "paths affect the digest");
}

fn contents(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(directory).expect("directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).expect("relative").to_str().expect("utf8").to_owned(),
                    fs::read(&path).expect("file"),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(directory, directory, &mut files);
    files
}

#[test]
fn the_same_inputs_give_the_same_bundle() {
    let scratch = Scratch::new("bundle-same");
    let inputs = inputs(&scratch);
    let a = scratch.path().join("a");
    let b = scratch.path().join("b");
    fs::create_dir(&a).expect("out");
    fs::write(a.join("stale.txt"), b"stale").expect("old output");
    let first = bundle(&inputs, &glue(), &a).expect("first");
    let second = bundle(&inputs, &glue(), &b).expect("second");
    assert_eq!(first, second);
    let files = contents(&a);
    assert_eq!(files, contents(&b));
    assert_eq!(files.len(), 16);
    assert_eq!(files.get(&format!("{first}/files/about.txt")), Some(&b"About".to_vec()));
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads SKEIN_UPDATE_GOLDENS, the bundle golden rewrite switch"
)]
fn no_placeholder_keeps_the_page_and_the_starter() {
    let scratch = Scratch::new("bundle-golden");
    let inputs = inputs(&scratch);
    let out = scratch.path().join("out");
    let digest = bundle(&inputs, &glue(), &out).expect("bundle");
    assert_eq!(
        fs::read(out.join(digest.to_string()).join("boot.js")).expect("starter"),
        b"import init from \"./notes.js\";\ninit();\n"
    );
    // Regenerate with SKEIN_UPDATE_GOLDENS=1 and this test alone.
    if std::env::var_os("SKEIN_UPDATE_GOLDENS").is_some() {
        fs::copy(out.join("index.html"), Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/golden/index.html"))
            .expect("regenerate page golden");
        return;
    }
    assert_eq!(fs::read(out.join("index.html")).expect("page"), include_bytes!("tests/golden/index.html"));
}

#[test]
fn an_own_page_gets_the_directory_at_each_marker() {
    let scratch = Scratch::new("bundle-own");
    let mut inputs = inputs(&scratch);
    let page = scratch.path().join("own.html");
    fs::write(&page, b"\xff{bundle}boot.js {bundle}a.css").expect("own page bytes");
    inputs.page = Some(page);
    let out = scratch.path().join("out");
    let digest = bundle(&inputs, &glue(), &out).expect("bundle");
    let mut expected = vec![255];
    expected.extend_from_slice(format!("/static/{digest}/boot.js /static/{digest}/a.css").as_bytes());
    assert_eq!(fs::read(out.join("index.html")).expect("page"), expected);
}

#[test]
fn an_own_page_without_the_marker_is_refused() {
    let scratch = Scratch::new("bundle-marker");
    let mut inputs = inputs(&scratch);
    let page = scratch.path().join("own.html");
    fs::write(&page, b"<html></html>").expect("own page");
    inputs.page = Some(page.clone());
    assert_eq!(bundle(&inputs, &glue(), &scratch.path().join("out")), Err(Refusal::NoMarker { page }));
}

#[test]
fn references_go_under_the_prefix() {
    let scratch = Scratch::new("bundle-prefix");
    let mut inputs = inputs(&scratch);
    inputs.prefix = "/assets/ui/".into();
    inputs.noscript = "Use <scripts> & WebAssembly".into();
    let out = scratch.path().join("out");
    let digest = bundle(&inputs, &glue(), &out).expect("bundle");
    let page = fs::read_to_string(out.join("index.html")).expect("page");
    for reference in ["skein-ui.css", "a.css", "boot.js"] {
        assert!(page.contains(&format!("\"/assets/ui/{digest}/{reference}\"")), "{reference}");
    }
    assert!(page.contains("Use &lt;scripts&gt; &amp; WebAssembly"));
    assert_eq!(
        fs::read(out.join(digest.to_string()).join("boot.js")).expect("starter"),
        b"import init from \"./notes.js\";\ninit();\n"
    );
}

#[test]
fn the_stylesheet_names_only_the_kits_flow_classes() {
    let stylesheet = include_str!("../skein-ui.css");
    let mut remaining = stylesheet;
    let mut without_comments = String::new();
    while let Some(start) = remaining.find("/*") {
        without_comments.push_str(&remaining[..start]);
        let comment = &remaining[start + 2..];
        remaining = &comment[comment.find("*/").expect("comment closes") + 2..];
    }
    without_comments.push_str(remaining);
    for block in without_comments.split('{') {
        let selectors = block.rsplit('}').next().expect("selector before its declarations");
        for selector in selectors.split(',') {
            for class in selector.split('.').skip(1) {
                let name = class.split(|c: char| !c.is_ascii_alphanumeric() && c != '-').next().expect("class name");
                assert!(
                    [
                        "skein-ui-flow-box",
                        "skein-ui-flow-name",
                        "skein-ui-flow-caption",
                        "skein-ui-flow-arrow",
                        "skein-ui-flow-strong"
                    ]
                    .contains(&name),
                    "the stylesheet names no owner's class: {name}"
                );
                assert!(
                    selector.trim().starts_with("[data-drawn] > svg "),
                    "flow tokens outweigh the marks: {selector}"
                );
            }
        }
    }
}

#[test]
fn programmatic_focus_targets_lose_the_outline_after_the_general_rule() {
    let stylesheet = include_str!("../skein-ui.css");
    let general = stylesheet.find(":focus-visible {").expect("general focus outline");
    let target = stylesheet
        .find("[tabindex=\"-1\"]:not([role=menuitem], [role=treeitem]):focus-visible {\n  outline: none;\n}")
        .expect("programmatic targets lose the outline; arrow-key controls keep it");
    assert!(general < target, "the target rule follows the general outline");
}

#[test]
fn skein_ui_css_comes_first_in_the_page() {
    let scratch = Scratch::new("bundle-css");
    let inputs = inputs(&scratch);
    let out = scratch.path().join("out");
    let digest = bundle(&inputs, &glue(), &out).expect("bundle");
    let page = fs::read_to_string(out.join("index.html")).expect("page");
    assert!(page.find("/skein-ui.css").expect("kit stylesheet") < page.find("/a.css").expect("owner stylesheet"));
    assert_eq!(fs::read(out.join(digest.to_string()).join("skein-ui.css")).expect("stylesheet"), STYLESHEET);
}

#[test]
fn each_refusal_names_its_file() {
    let scratch = Scratch::new("bundle-refusals");
    let base = inputs(&scratch);
    let paths =
        [scratch.path().join("bad.xml"), scratch.path().join("skein-ui.css"), scratch.path().join("other/a.css")];
    fs::create_dir(scratch.path().join("other")).expect("other directory");
    for path in &paths {
        fs::write(path, b"input").expect("input file");
    }
    let expected = [
        Refusal::Type { file: paths[0].clone() },
        Refusal::ReservedName { file: paths[1].clone() },
        Refusal::SameName { file: paths[2].clone() },
    ];
    for (path, expected) in paths.iter().zip(expected) {
        let mut inputs = base.clone();
        inputs.stylesheets.push(path.clone());
        assert_eq!(bundle(&inputs, &glue(), &scratch.path().join("out")), Err(expected), "{}", path.display());
    }
}

#[test]
fn each_extension_has_its_type() {
    const CASES: &[(&str, Option<&str>)] = &[
        ("index.html", Some("text/html; charset=utf-8")),
        ("boot.js", Some("text/javascript")),
        ("notes.wasm", Some("application/wasm")),
        ("a.css", Some("text/css")),
        ("a.svg", Some("image/svg+xml")),
        ("a.png", Some("image/png")),
        ("a.webp", Some("image/webp")),
        ("a.ico", Some("image/x-icon")),
        ("a.woff2", Some("font/woff2")),
        ("about.txt", Some("text/plain; charset=utf-8")),
        ("app.webmanifest", Some("application/manifest+json")),
        ("a.xml", None),
        ("no-extension", None),
    ];
    for (name, expected) in CASES {
        assert_eq!(super::types::of(Path::new(name)), *expected, "{name}");
    }
}

#[test]
fn the_glue_loads_its_sibling_module_when_init_has_no_argument() {
    let scratch = Scratch::new("glue-module-path");
    let module = scratch.path().join("notes.wasm");
    fs::write(&module, b"\0asm\x01\0\0\0").expect("empty module");
    let glue = super::glue(&module, "notes").expect("web glue");
    let js = std::str::from_utf8(&glue.js).expect("generated JavaScript");
    assert!(js.contains("new URL('notes_bg.wasm', import.meta.url)"));
}

#[test]
fn the_stylesheet_sets_a_progress_for_every_whole_percent() {
    let stylesheet = std::str::from_utf8(STYLESHEET).expect("stylesheet is UTF-8");
    for percent in 0..=100 {
        let rule = format!("[data-progress=\"{percent}\"] {{ --skein-ui-progress: {percent}%; }}");
        assert!(stylesheet.lines().any(|line| line == rule), "missing rule: {rule}");
    }
    assert_eq!(stylesheet.matches("[data-progress=").count(), 101);
}

#[test]
fn the_bundler_writes_a_compressed_form_beside_each_text() {
    use std::io::Read;
    let scratch = Scratch::new("bundle-gzip-types");
    let inputs = inputs(&scratch);
    let out = scratch.path().join("out");
    bundle(&inputs, &glue(), &out).expect("bundle");
    let files = contents(&out);
    for (name, plain) in &files {
        if ["wasm", "js", "css", "svg"]
            .contains(&Path::new(name).extension().and_then(|ext| ext.to_str()).unwrap_or(""))
        {
            let compressed = files.get(&format!("{name}.gz")).expect("compressed form");
            let mut decoded = Vec::new();
            flate2::read::GzDecoder::new(compressed.as_slice()).read_to_end(&mut decoded).expect("gzip member");
            assert_eq!(&decoded, plain, "{name}");
        } else {
            assert!(!files.contains_key(&format!("{name}.gz")), "no form for {name}");
        }
    }
}

#[test]
fn a_compressed_form_is_the_same_bytes_each_time() {
    let scratch = Scratch::new("bundle-gzip-stable");
    let inputs = inputs(&scratch);
    let a = scratch.path().join("a");
    let b = scratch.path().join("b");
    let digest = bundle(&inputs, &glue(), &a).expect("first");
    bundle(&inputs, &glue(), &b).expect("second");
    assert_eq!(contents(&a), contents(&b));
    let compressed = fs::read(a.join(digest.to_string()).join("notes.js.gz")).expect("compressed form");
    assert_eq!(&compressed[4..8], &[0; 4], "no time");
    assert_eq!(compressed[3], 0, "no filename or optional fields");
}

#[test]
fn the_digest_covers_the_compressed_forms() {
    let scratch = Scratch::new("bundle-gzip-digest");
    let mut inputs = inputs(&scratch);
    let out = scratch.path().join("out");
    let compressed = bundle(&inputs, &glue(), &out).expect("gzip bundle");
    inputs.gzip = false;
    assert_ne!(compressed, bundle(&inputs, &glue(), &out).expect("plain bundle"));
}

#[test]
fn a_gz_input_refuses_the_bundle() {
    for source in ["stylesheet", "files", "pictures"] {
        let scratch = Scratch::new("bundle-gzip-input");
        let mut inputs = inputs(&scratch);
        let path = match source {
            "stylesheet" => {
                let path = scratch.path().join("extra.css.gz");
                inputs.stylesheets.push(path.clone());
                path
            }
            "files" => scratch.path().join("files/extra.js.gz"),
            "pictures" => scratch.path().join("pictures/extra.svg.gz"),
            _ => unreachable!(),
        };
        fs::write(&path, b"already compressed").expect("input");
        assert_eq!(bundle(&inputs, &glue(), &scratch.path().join("out")), Err(Refusal::Type { file: path }));
    }
}

#[test]
fn gzip_off_writes_the_plain_files_alone() {
    let scratch = Scratch::new("bundle-gzip-off");
    let mut inputs = inputs(&scratch);
    inputs.gzip = false;
    let out = scratch.path().join("out");
    bundle(&inputs, &glue(), &out).expect("plain bundle");
    let files = contents(&out);
    assert_eq!(files.len(), 9);
    assert!(files.keys().all(|name| Path::new(name).extension() != Some(std::ffi::OsStr::new("gz"))));
}

fn placeholder(scratch: &Scratch) -> super::Placeholder {
    let markup = scratch.path().join("loading.html");
    let stylesheet = scratch.path().join("loading.css");
    fs::write(&markup, b"<div id=\"boot\">Notes {noscript}</div>").expect("markup");
    fs::write(&stylesheet, b"#boot { color: navy; }").expect("placeholder stylesheet");
    super::Placeholder { markup, stylesheet }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads SKEIN_UPDATE_GOLDENS, the bundle golden rewrite switch"
)]
fn a_placeholder_makes_the_page() {
    let scratch = Scratch::new("bundle-placeholder");
    let mut inputs = inputs(&scratch);
    inputs.placeholder = Some(placeholder(&scratch));
    let out = scratch.path().join("out");
    bundle(&inputs, &glue(), &out).expect("bundle");
    // Regenerate with SKEIN_UPDATE_GOLDENS=1 and this test alone.
    if std::env::var_os("SKEIN_UPDATE_GOLDENS").is_some() {
        fs::copy(
            out.join("index.html"),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/golden/placeholder.html"),
        )
        .expect("regenerate placeholder golden");
        return;
    }
    assert_eq!(fs::read(out.join("index.html")).expect("page"), include_bytes!("tests/golden/placeholder.html"));
}

#[test]
fn the_loader_heads_with_each_files_sizes() {
    let scratch = Scratch::new("bundle-sizes");
    let mut inputs = inputs(&scratch);
    inputs.placeholder = Some(placeholder(&scratch));
    // Quoting is independent of HTML escaping.
    let sheet = scratch.path().join("a\\\".css");
    fs::write(&sheet, b"body {}").expect("quoted sheet");
    inputs.stylesheets.push(sheet);
    for gzip in [true, false] {
        inputs.gzip = gzip;
        let out = scratch.path().join("out");
        let digest = bundle(&inputs, &glue(), &out).expect("bundle");
        let directory = out.join(digest.to_string());
        let row = |name: &str| {
            let plain = fs::metadata(directory.join(name)).expect("plain size").len();
            let compressed =
                if gzip { fs::metadata(directory.join(format!("{name}.gz"))).expect("gzip size").len() } else { plain };
            format!("[\"{}\", {plain}, {compressed}]", name.replace('\\', "\\\\").replace('\"', "\\\""))
        };
        let head = format!(
            "const GLUE = {};\nconst WASM = {};\nconst SHEETS = [{}, {}, {}];\n",
            row("notes.js"),
            row("notes_bg.wasm"),
            row("skein-ui.css"),
            row("a.css"),
            row("a\\\".css")
        );
        let boot = fs::read_to_string(directory.join("boot.js")).expect("loader");
        assert!(boot.starts_with(&head), "{boot}");
        assert!(!boot.contains("loading.css"));
    }
}

#[test]
fn a_title_names_the_page() {
    let scratch = Scratch::new("bundle-title");
    let mut inputs = inputs(&scratch);
    inputs.title = Some("Notes <&> \"'".into());
    for placeholder in [None, Some(placeholder(&scratch))] {
        inputs.placeholder = placeholder;
        let out = scratch.path().join("out");
        bundle(&inputs, &glue(), &out).expect("bundle");
        assert!(
            fs::read_to_string(out.join("index.html"))
                .expect("page")
                .contains("<title>Notes &lt;&amp;&gt; &quot;&#39;</title>")
        );
    }
}

#[test]
fn a_placeholder_is_refused_where_it_cant_hold() {
    let scratch = Scratch::new("bundle-placeholder-refusal");
    let mut inputs = inputs(&scratch);
    let placeholder = placeholder(&scratch);
    inputs.placeholder = Some(placeholder.clone());
    let out = scratch.path().join("out");
    fs::write(&placeholder.markup, b"No marker").expect("markup");
    assert_eq!(bundle(&inputs, &glue(), &out), Err(Refusal::NoNoscriptMarker { markup: placeholder.markup.clone() }));
    fs::write(&placeholder.markup, b"{noscript}").expect("marker");
    let page = scratch.path().join("own.html");
    inputs.page = Some(page.clone());
    assert_eq!(bundle(&inputs, &glue(), &out), Err(Refusal::PlaceholderWithPage { page }));
    inputs.page = None;
    for (name, expected) in [
        ("skein-ui.css", Refusal::ReservedName { file: scratch.path().join("skein-ui.css") }),
        ("a.css", Refusal::SameName { file: scratch.path().join("a.css") }),
        ("bad\n.css", Refusal::Type { file: scratch.path().join("bad\n.css") }),
    ] {
        let file = scratch.path().join(name);
        fs::write(&file, b"body {}").expect("sheet");
        inputs.placeholder.as_mut().expect("placeholder").stylesheet = file;
        assert_eq!(bundle(&inputs, &glue(), &out), Err(expected));
    }
}

#[test]
fn the_page_and_the_loader_hold_nothing_inline() {
    let scratch = Scratch::new("bundle-policy");
    let mut inputs = inputs(&scratch);
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/shell-web");
    inputs.placeholder =
        Some(super::Placeholder { markup: example.join("loading.html"), stylesheet: example.join("loading.css") });
    let out = scratch.path().join("out");
    let digest = bundle(&inputs, &glue(), &out).expect("bundle");
    let page = fs::read_to_string(out.join("index.html")).expect("page");
    assert!(!page.contains("<style"));
    for tag in page.split('<').skip(1) {
        for attribute in tag.split('>').next().expect("tag").split_ascii_whitespace().skip(1) {
            let name = attribute.split('=').next().expect("attribute");
            assert!(name != "style" && !name.starts_with("on"), "inline attribute: {name}");
        }
    }
    assert_eq!(page.matches("<script").count(), 1);
    assert!(page.contains(&format!("<script type=\"module\" src=\"/static/{digest}/boot.js\"></script>")));
    let loader = include_str!("../loader.js");
    assert!(!loader.contains("eval("));
    assert!(!loader.contains("new Function"));
}
