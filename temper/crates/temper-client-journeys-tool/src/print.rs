//! Static pages and the files beside them (client/web/journeys.md, 5.2 and 5.3),
//! with page-local generated ids (client/journey-placeholders.md, 5).

use skein_ui::NodeId;
use std::{collections::BTreeMap, fmt::Write, fs, io, path::Path};
pub use temper_client_shell_web::journey::Lead;

/// A page as the person saw it, at one mark (client/web/journeys.md, 3.4).
pub struct Mark {
    pub html: String,
    pub heading: Vec<u8>,
    pub acted: Vec<NodeId>,
    pub lead: Lead,
}

/// One printed page: its file name in the journey's directory and its document.
pub struct Printed {
    pub name: String,
    pub document: String,
}

/// A journey's pages, in the marks' order.
#[must_use]
pub fn pages(marks: &[Mark]) -> Vec<Printed> {
    let mut printed = Vec::new();
    for (index, mark) in marks.iter().enumerate() {
        let prev = (index > 0).then(|| format!("{index:02}.html"));
        let next = (index + 1 < marks.len()).then(|| format!("{:02}.html", index + 2));
        let mut html = mark.html.replacen("<div", "<div id=\"app\"", 1).replace("<dialog ", "<dialog open ");
        for NodeId(node) in &mark.acted {
            let next = next.as_deref().expect("an acted mark has a next page");
            let attribute = format!(" data-node=\"{node}\"");
            assert!(html.contains(&attribute), "the acted control is on the marked page");
            let act = match mark.lead {
                Lead::Press => "",
                Lead::Submit => " data-journey-act=\"submit\"",
                Lead::Toggle => " data-journey-act=\"toggle\"",
            };
            html = html.replacen(&attribute, &format!(" data-journey-next=\"{next}\"{act}"), 1);
        }
        while let Some(start) = html.find(" data-node=\"") {
            let value = start + " data-node=\"".len();
            let end = value + html[value..].find('"').expect("a node attribute has a closing quote") + 1;
            html.replace_range(start..end, "");
        }
        let html = numbered_ids(&html);
        let heading = if mark.html.contains("temper-main temper-home") {
            b"Home".as_slice()
        } else if mark.html.contains("temper-main temper-welcome") {
            "Welcome · first run".as_bytes()
        } else {
            &mark.heading
        };
        let title = escape(&skein_ui_web::title::text(heading, b"temper"));
        let mut document = format!(
            "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\"><title>{title}</title>"
        );
        for stylesheet in [
            "../skein-ui.css",
            "../../../../../../crates/temper-client-shell-web/temper.css",
            "../../../../../../crates/temper-client-shell-web/task-pages.css",
            "../overlay.css",
        ] {
            write!(document, "<link rel=\"stylesheet\" href=\"{stylesheet}\">").expect("write string");
        }
        if let Some(prev) = prev {
            write!(document, "<link rel=\"prev\" href=\"{prev}\">").expect("write string");
        }
        if let Some(next) = next {
            write!(document, "<link rel=\"next\" href=\"{next}\">").expect("write string");
        }
        write!(document, "<script src=\"../journey.js\" defer></script></head><body>{html}</body></html>")
            .expect("write string");
        printed.push(Printed { name: format!("{:02}.html", index + 1), document: lines(&document) });
    }
    printed
}

fn numbered_ids(mut html: &str) -> String {
    let mut words: BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();
    let mut numbered = String::with_capacity(html.len());
    while let Some((before, after)) = html.split_once("skein-ui-") {
        numbered.push_str(before);
        numbered.push_str("skein-ui-");
        let word_end = after.find(|c: char| !c.is_ascii_lowercase()).unwrap_or(after.len());
        let (word, rest) = after.split_at(word_end);
        numbered.push_str(word);
        html = rest;
        if word.is_empty() {
            continue;
        }
        let Some(after_dash) = rest.strip_prefix('-') else {
            continue;
        };
        let number_end = after_dash.find(|c: char| !c.is_ascii_digit()).unwrap_or(after_dash.len());
        let (number, rest) = after_dash.split_at(number_end);
        if number.is_empty() {
            continue;
        }
        let numbers = words.entry(word).or_default();
        let next = numbers.len() + 1;
        let fresh = numbers.entry(number).or_insert(next);
        write!(numbered, "-{fresh}").expect("write string");
        html = rest;
    }
    numbered.push_str(html);
    numbered
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn lines(html: &str) -> String {
    let html = html.strip_prefix("<!doctype html>\n").expect("a printed document starts with the doctype line");
    format!("<!doctype html>\n{}\n", html.replace('>', "\n>"))
}

/// The files beside the journeys, written at each print (client/web/journeys.md, 5.2):
/// `skein-ui.css`, `journey.js`, and `overlay.css` made empty when missing and never written otherwise.
/// # Errors
/// Returns a filesystem error when the stylesheet, script or a missing overlay cannot be written.
pub fn beside(journeys: &Path) -> io::Result<()> {
    fs::write(journeys.join("skein-ui.css"), skein_ui_bundle::STYLESHEET)?;
    fs::write(journeys.join("journey.js"), SCRIPT)?;
    match fs::OpenOptions::new().write(true).create_new(true).open(journeys.join("overlay.css")) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

/// The pages' script, written beside the journeys.
pub const SCRIPT: &str = include_str!("../journey.js");

#[cfg(test)]
mod id_tests {
    fn mark(html: &str) -> super::Mark {
        super::Mark { html: html.to_owned(), heading: Vec::new(), acted: Vec::new(), lead: super::Lead::Press }
    }

    fn body(page: &super::Printed) -> &[u8] {
        page.document
            .split_once("<body\n>")
            .expect("body starts")
            .1
            .split_once("</body\n>")
            .expect("body ends")
            .0
            .as_bytes()
    }

    #[test]
    fn each_words_ids_and_references_follow_first_appearance() {
        let printed = super::pages(&[mark(concat!(
            "<div data-node=\"999\">",
            "<button id=\"skein-ui-menu-41\" aria-controls=\"skein-ui-options-41\"></button>",
            "<input aria-activedescendant=\"skein-ui-option-57\" aria-labelledby=\"skein-ui-label-80\" aria-describedby=\"skein-ui-description-90 skein-ui-state-100\">",
            "<div id=\"skein-ui-options-41\"><span id=\"skein-ui-option-8\"></span><span id=\"skein-ui-option-57\"></span></div>",
            "<label id=\"skein-ui-label-80\"></label><span id=\"skein-ui-description-90\"></span><span id=\"skein-ui-state-100\"></span>",
            "<button id=\"skein-ui-menu-1\" aria-controls=\"skein-ui-menu-41\"></button></div>",
        ))]);
        assert_eq!(body(&printed[0]), concat!(
            "<div id=\"app\"\n>",
            "<button id=\"skein-ui-menu-1\" aria-controls=\"skein-ui-options-1\"\n></button\n>",
            "<input aria-activedescendant=\"skein-ui-option-1\" aria-labelledby=\"skein-ui-label-1\" aria-describedby=\"skein-ui-description-1 skein-ui-state-1\"\n>",
            "<div id=\"skein-ui-options-1\"\n><span id=\"skein-ui-option-2\"\n></span\n><span id=\"skein-ui-option-1\"\n></span\n></div\n>",
            "<label id=\"skein-ui-label-1\"\n></label\n><span id=\"skein-ui-description-1\"\n></span\n><span id=\"skein-ui-state-1\"\n></span\n>",
            "<button id=\"skein-ui-menu-2\" aria-controls=\"skein-ui-menu-1\"\n></button\n></div\n>",
        ).as_bytes());
    }

    #[test]
    fn gradients_keep_their_indices_under_a_fresh_node_number() {
        let printed = super::pages(&[mark(concat!(
            "<svg><linearGradient id=\"skein-ui-fade-88-0\"></linearGradient>",
            "<path fill=\"url(#skein-ui-fade-88-1)\" stroke=\"url(#skein-ui-fade-4-0)\"></path>",
            "<linearGradient id=\"skein-ui-fade-4-0\"></linearGradient></svg>",
        ))]);
        assert_eq!(
            body(&printed[0]),
            concat!(
                "<svg\n><linearGradient id=\"skein-ui-fade-1-0\"\n></linearGradient\n>",
                "<path fill=\"url(#skein-ui-fade-1-1)\" stroke=\"url(#skein-ui-fade-2-0)\"\n></path\n>",
                "<linearGradient id=\"skein-ui-fade-2-0\"\n></linearGradient\n></svg\n>",
            )
            .as_bytes()
        );
    }

    #[test]
    fn each_page_numbers_its_ids_independently() {
        let printed = super::pages(&[
            mark("<section id=\"skein-ui-menu-12\" aria-controls=\"skein-ui-menu-12\"></section>"),
            mark("<section id=\"skein-ui-menu-30\" aria-controls=\"skein-ui-menu-30\"></section>"),
        ]);
        assert_eq!(body(&printed[0]), body(&printed[1]));
        assert_eq!(
            body(&printed[0]),
            b"<section id=\"skein-ui-menu-1\" aria-controls=\"skein-ui-menu-1\"\n></section\n>"
        );
    }

    #[test]
    fn non_ids_and_surrounding_bytes_are_unchanged() {
        let printed = super::pages(&[mark(concat!(
            "<p title=\"skein-uix-3\">é &amp; skein-ui-menu- skein-ui-menu-x ",
            "skein-ui-9 skein-ui-Menu-7 skein-ui-state-72</p>",
        ))]);
        assert_eq!(
            body(&printed[0]),
            concat!(
                "<p title=\"skein-uix-3\"\n>é &amp; skein-ui-menu- skein-ui-menu-x ",
                "skein-ui-9 skein-ui-Menu-7 skein-ui-state-1</p\n>",
            )
            .as_bytes()
        );
    }
}

#[cfg(test)]
mod home_tests {
    #[test]
    fn home_title_keeps_the_page_name_with_a_ready_headline() {
        for heading in ["Hello is ready.", "Nothing needs you."] {
            let mark = super::Mark {
                html: "<div><main class=\"temper-main temper-home\"><h1>Headline</h1></main></div>".to_owned(),
                heading: heading.as_bytes().to_vec(),
                acted: Vec::new(),
                lead: super::Lead::Press,
            };
            let printed = super::pages(&[mark]);
            assert!(printed[0].document.contains("<title\n>Home · temper</title"));
        }
    }
}

#[cfg(test)]
mod welcome_tests {
    #[test]
    fn welcome_title_keeps_first_run_on_every_step() {
        for heading in ["Welcome to temper", "Connect a forge", "Connect a model", "Ready", "Almost ready", ""] {
            let mark = super::Mark {
                html: "<div><main class=\"temper-main temper-welcome\"><h1>Heading</h1></main></div>".to_owned(),
                heading: heading.as_bytes().to_vec(),
                acted: Vec::new(),
                lead: super::Lead::Press,
            };
            let printed = super::pages(&[mark]);
            assert!(printed[0].document.contains("<title\n>Welcome · first run · temper</title"));
        }
    }
}
