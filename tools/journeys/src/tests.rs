use crate::{
    JOURNEYS,
    print::{Mark, SCRIPT, beside, pages},
};
use skein_scratch::Scratch;
use skein_ui::NodeId;
use std::{fs, path::Path};

fn mark(html: &str, heading: &str, acted: Option<u32>) -> Mark {
    Mark { html: html.to_owned(), heading: heading.as_bytes().to_vec(), acted: acted.map(NodeId) }
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
    let printed = pages(&[
        mark(
            "<div data-node=\"1\"><h1 data-node=\"2\">Welcome</h1><button data-node=\"3\">Set up temper</button></div>",
            "Welcome",
            Some(3),
        ),
        mark("<div data-node=\"4\">Forge</div>", "Forge", None),
    ]);
    assert!(printed[0].document.contains("<button data-journey-next=\"02.html\"\n>Set up temper</button\n>"));
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
            "../../../../../crates/temper-client-shell-web/temper.css",
            "../../../../../crates/temper-client-shell-web/task-pages.css",
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
fn beside_keeps_an_overlay_and_writes_the_script() {
    let scratch = Scratch::new("temper-client-journeys-beside");
    beside(scratch.path()).expect("write beside the journeys");
    let overlay = scratch.path().join("overlay.css");
    let script = scratch.path().join("journey.js");
    assert_eq!(fs::read(&overlay).expect("empty overlay"), b"");
    assert_eq!(fs::read_to_string(&script).expect("script"), SCRIPT);
    let rule = b"#app { color: red; }\n";
    fs::write(&overlay, rule).expect("person's overlay");
    fs::write(&script, "old script").expect("old script");
    beside(scratch.path()).expect("rewrite beside the journeys");
    assert_eq!(fs::read(&overlay).expect("kept overlay"), rule);
    assert_eq!(fs::read_to_string(&script).expect("rewritten script"), SCRIPT);
}
