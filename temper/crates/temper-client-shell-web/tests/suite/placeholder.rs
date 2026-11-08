//! The first paint's inputs (client/browser.md, 2.5).
use std::{fs, path::Path};

fn input(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).expect("placeholder input exists")
}

#[test]
fn placeholder_exposes_the_loaders_nodes_and_words() {
    let markup = input("loading.html");
    for required in [
        "{noscript}",
        "id=\"boot\"",
        "<g fill=\"#c2410c\" transform=\"scale(11.4)\">",
        "id=\"boot-line\"></p>",
        "id=\"boot-note\" aria-live=\"polite\"></p>",
        "<progress max=\"1\" value=\"0\" aria-label=\"Loading temper\"></progress>",
        "<button type=\"button\" hidden>Try again</button>",
        "data-figure=\"{got} of {total} MB\"",
        "data-left=\" · about {left} left\"",
        "data-seconds=\"{n} s\"",
        "data-minutes=\"{n} min\"",
        "data-keeps=\"Your browser keeps this, so next time it opens at once.\"",
        "data-waiting=\"Waiting for the server…\"",
        "data-starting=\"Starting…\"",
        "data-failed=\"temper didn’t load\"",
        "data-dropped=\"The connection dropped at {got} of {total} MB.\"",
        "data-answered=\"The server answered {status}.\"",
        "data-silent=\"The server didn’t answer.\"",
        "data-no-wasm=\"This browser has no WebAssembly.\"",
        "data-no-start=\"It didn’t start.\"",
    ] {
        assert!(markup.contains(required), "missing loader contract: {required}");
    }
    assert!(!markup.contains("<circle"), "placeholder mark has a dot");
    assert!(!markup.contains("#b8664e"), "placeholder mark uses the old color");
}

#[test]
fn first_paint_is_self_contained_and_at_most_six_kib() {
    let markup = input("loading.html");
    let css = input("loading.css");
    for forbidden in ["style=", "<style", "<script", "<link", "href="] {
        assert!(!markup.to_ascii_lowercase().contains(forbidden), "placeholder fetch or inline policy: {forbidden}");
    }
    for forbidden in ["url(", "@import", "@font-face"] {
        assert!(!css.to_ascii_lowercase().contains(forbidden), "placeholder stylesheet fetch: {forbidden}");
    }
    assert!(markup.len() + css.len() <= 6 * 1024, "first paint exceeds 6 KiB");
}
