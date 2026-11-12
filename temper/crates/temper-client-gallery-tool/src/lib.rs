//! Static browser pages from real client states (client/web/gallery.md).
use skein_lib::Queue;
use skein_ui::{DrawingKind, Role, scene::Scene};
use skein_ui_web::{Look, mapping::Primary, page::Page};
use std::fmt::Write;
use std::{
    fs,
    path::{Path, PathBuf},
};
mod cli;
pub use cli::{TABLE, main};
mod driver;
mod needs;
pub mod samples;
#[cfg(test)]
mod tests;
mod worlds;
pub use samples::Sample;

// The box at 420 pixels, with a phone's 16 pixels each side (client/pages/README.md).
const NARROW: u16 = 388;

pub const STYLESHEETS: [&str; 2] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/../temper-client-shell-web/temper.css"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../temper-client-shell-web/task-pages.css"),
];
#[derive(Debug)]
pub enum Refusal {
    NotGallery { out: PathBuf },
    Bundle(skein_ui_bundle::Refusal),
    Io { path: PathBuf },
}
/// Prints every state's page, the bundle and the index into out (client/web/gallery.md, 1).
/// # Errors
/// Refuses an existing directory without index.html, or a bundle or filesystem failure.
pub fn print(out: &Path) -> Result<(), Refusal> {
    replace_gallery(out)?;
    let mut stylesheets = Vec::new();
    for path in STYLESHEETS {
        stylesheets.push(PathBuf::from(path));
    }
    let inputs = skein_ui_bundle::Inputs {
        // Prints are opened as files, so no compressed form is read.
        gzip: false,
        title: None,
        placeholder: None,
        name: "temper_client_shell_web".to_owned(),
        prefix: "/bundle/".to_owned(),
        stylesheets,
        pictures: vec![],
        page: None,
        files: Some(PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../temper-client-shell-web/files"))),
        noscript: "temper needs WebAssembly and scripts.".to_owned(),
    };
    let digest = match skein_ui_bundle::bundle(
        &inputs,
        &skein_ui_bundle::Glue { js: vec![], wasm: vec![], snippets: vec![] },
        &out.join("bundle"),
    ) {
        Ok(digest) => digest,
        Err(why) => return Err(Refusal::Bundle(why)),
    };
    let samples = samples::samples();
    let inputs = ViewInputs::new();
    for sample in &samples {
        write(&out.join(format!("{}.html", sample.name)), &page_with(sample, &digest, inputs.clone()))?;
    }
    write(&out.join("index.html"), &index(&samples))
}
// Replace only an owned print, whole (client/web/gallery.md, 1).
fn replace_gallery(out: &Path) -> Result<(), Refusal> {
    if out.exists() {
        if !out.join("index.html").is_file() {
            return Err(Refusal::NotGallery { out: out.to_owned() });
        }
        if fs::remove_dir_all(out).is_err() {
            return Err(Refusal::Io { path: out.to_owned() });
        }
    }
    if fs::create_dir_all(out).is_err() {
        return Err(Refusal::Io { path: out.to_owned() });
    }
    Ok(())
}
fn write(path: &Path, bytes: &str) -> Result<(), Refusal> {
    match fs::write(path, bytes) {
        Ok(()) => Ok(()),
        Err(_) => Err(Refusal::Io { path: path.to_owned() }),
    }
}
fn draw(kind: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut Scene) -> bool {
    temper_client_view::paths::draw(kind, data, width, height, scene)
}

/// The same browser view construction used by pages and semantic tests.
#[must_use]
pub fn view(sample: &Sample) -> temper_client_view::View {
    let (view, _, _) = built(sample);
    view
}
// Every state uses the same shipped words and bounds (client/web/gallery.md, 2).
#[derive(Clone)]
struct ViewInputs {
    limits: temper_client_domain::Limits,
    words: temper_client_view::words::Words,
    bounds: temper_client_view::Limits,
}
impl ViewInputs {
    fn new() -> Self {
        let limits = driver::limits();
        let words = temper_client_view::words::Words::english(
            temper_client_protocol_wire::FORGE,
            limits.client.domain.text,
            temper_client_view::words::Shell::Browser,
        )
        .expect("shipped browser words");
        let bounds = temper_client_view::Limits::of(&limits, &words).expect("shipped view bounds");
        Self { limits, words, bounds }
    }
}
fn built(sample: &Sample) -> (temper_client_view::View, temper_client_view::Limits, Vec<skein_ui::Patch>) {
    built_with(sample, ViewInputs::new())
}
fn built_with(
    sample: &Sample,
    inputs: ViewInputs,
) -> (temper_client_view::View, temper_client_view::Limits, Vec<skein_ui::Patch>) {
    built_root(sample, inputs, &samples::domain(sample))
}
fn built_root(
    sample: &Sample,
    inputs: ViewInputs,
    root: &temper_client_domain::Domain,
) -> (temper_client_view::View, temper_client_view::Limits, Vec<skein_ui::Patch>) {
    let ViewInputs { limits, words, bounds } = inputs;
    let mut view = temper_client_view::View::with_sign_in(
        &bounds,
        &limits,
        words,
        temper_client_protocol_wire::FORGE,
        temper_client_view::sign_in::Ways {
            google: sample.then == samples::Then::GoogleSignIn,
            origin: Some(Box::from(b"https://temper.example.com".as_slice())),
        },
    );
    let mut forge_sign_in = temper_client_view::forge_sign_in::Forges::new();
    forge_sign_in.read("3 forgejo http://127.0.0.1:8368 owned");
    view.forge_sign_in(forge_sign_in);
    let mut patches = Queue::with_capacity(bounds.tree.patches);
    temper_client_view::build(&mut view, root, worlds::sample_now(sample.world, sample.name), &mut patches);
    let mut collected = Vec::new();
    for _ in 0..patches.len() {
        collected.push(patches.pop().expect("queued patch"));
    }
    (view, bounds, collected)
}
/// One state's page, a whole HTML document.
#[must_use]
pub fn page(sample: &Sample, digest: &skein_ui_bundle::Digest) -> String {
    page_with(sample, digest, ViewInputs::new())
}
#[expect(clippy::too_many_lines, reason = "This print keeps the page and its narrow scenes together.")]
fn page_with(sample: &Sample, digest: &skein_ui_bundle::Digest, inputs: ViewInputs) -> String {
    let root = samples::domain(sample);
    let night = if root.pages().look().night { " data-night" } else { "" };
    let (mut view, bounds, collected) = built_root(sample, inputs, &root);
    let mut page = Page::new(
        Look {
            names: &[temper_client_view::look::NAMES],
            pictures: &[],
            components: &[],
            elements: temper_client_view::look::ELEMENTS,
            draw,
        },
        &bounds.tree,
        b"temper",
        Primary::Control,
    );
    let mut ops = Vec::new();
    page.apply(&collected, &mut ops);
    for node in view.tree().nodes() {
        if node.drawing.as_ref().is_some_and(|drawing| drawing.kind == temper_client_view::paths::KIND) {
            page.resized(node.id, 720, 0, &mut ops);
        }
    }
    let links = page.links().to_vec();
    for node in links {
        if let Some(href) = temper_client_view::href(&mut view, node) {
            page.set_href(node, &href, &mut ops);
        }
    }
    if let Some(name) = sample.open_menu()
        && let Some(node) = view.tree().nodes().iter().find(|n| n.role == Role::Menu && n.name.as_deref() == Some(name))
    {
        let _act = page.act(skein_ui_web::page::Event::Click { node: node.id, modifier: false }, &mut ops);
    }
    if sample.name == "chat-work-reading" || sample.name == "agent-work" {
        let log = view.tree().nodes().iter().find(|node| node.role == Role::Log).expect("conversation log");
        let _act = page
            .act(skein_ui_web::page::Event::Scrolled { node: log.id, top: 100, height: 1000, viewport: 200 }, &mut ops);
    }
    let title = escape(&skein_ui_web::title::text(&heading(view.tree()), b"temper"));
    let mut html = page.html().replacen("<div", "<div id=\"app\"", 1).replace("<dialog ", "<dialog open ");
    let mut has_narrow = false;
    for node in view.tree().nodes() {
        let Some(drawing) = node.drawing.as_ref().filter(|drawing| drawing.kind == temper_client_view::paths::KIND)
        else {
            continue;
        };
        let mut scene = Scene::new(&bounds.tree);
        if !draw(drawing.kind, &drawing.data, NARROW, 0, &mut scene) || scene.refused() {
            continue;
        }
        let attribute = format!("data-node=\"{}\"", node.id.0);
        if let Some(start) = html.find(&attribute)
            && let Some(end) = html[start..].find("</svg>")
        {
            let mut svg = skein_ui_web::svg::draw(
                &scene,
                &[temper_client_view::look::NAMES],
                NARROW,
                0,
                &format!("temper-gallery-narrow-{}", node.id.0),
            );
            svg.set("data-gallery-narrow", "");
            html.insert_str(start + end + "</svg>".len(), &skein_ui_web::dom::print(&svg));
            has_narrow = true;
        }
    }
    if let Some(name) = sample.open_disclosure() {
        for node in view.tree().nodes() {
            if node.role == Role::Disclosure && node.name.as_deref() == Some(name) {
                let attribute = format!("data-node=\"{}\"", node.id.0);
                html = html.replacen(&attribute, &format!("{attribute} open"), 1);
                break;
            }
        }
    }
    for op in &ops {
        if let skein_ui_web::page::Op::Focus { node } = op {
            let attribute = format!("data-node=\"{}\"", node.0);
            html = html.replacen(&attribute, &format!("{attribute} autofocus"), 1);
        }
    }
    if sample.name == "needs-saw-open" {
        html = html.replace("class=\"temper-ask-saw\"", "open class=\"temper-ask-saw\"");
    }
    if sample.name == "goal-landed-open" {
        html = html.replace("class=\"temper-goal-landed-fold\"", "open class=\"temper-goal-landed-fold\"");
    }
    let mut styles = String::new();
    for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
        write!(styles, "<link rel=\"stylesheet\" href=\"bundle/{digest}/{file}\">").expect("write string");
    }
    if has_narrow {
        styles.push_str(concat!(
            "<style>\n",
            "@media (max-width: 760px) { #app .temper-ways-path[data-drawn] > svg:not([data-gallery-narrow]) { display: none; } }\n",
            "@media (min-width: 761px) { #app .temper-ways-path[data-drawn] > svg[data-gallery-narrow] { display: none; } }\n",
            "</style>",
        ));
    }
    format!(
        "<!doctype html>\n<html lang=\"en\"{night}><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\"><title>{title}</title>{styles}</head><body>{html}</body></html>\n"
    )
}
fn heading(tree: &skein_ui::Tree) -> Vec<u8> {
    let nodes = tree.nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            if node.styles.as_slice().contains(&temper_client_view::look::WELCOME) {
                return "Welcome · first run".as_bytes().to_vec();
            }
            let end = index + usize::try_from(node.size).expect("main subtree");
            for (offset, heading) in nodes[index..end].iter().enumerate() {
                if heading.role == (Role::Heading { level: 1 }) {
                    let start = index + offset;
                    let end = start + usize::try_from(heading.size).expect("heading subtree");
                    let mut result = Vec::new();
                    for node in &nodes[start..end] {
                        for run in &node.text {
                            result.extend_from_slice(&run.text);
                        }
                    }
                    return result;
                }
            }
        }
    }
    Vec::new()
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}
#[must_use]
pub fn index(samples: &[Sample]) -> String {
    let mut html = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\"><title>temper gallery</title></head><body><h1>temper gallery</h1><ul>\n",
    );
    for sample in samples {
        write!(html, "<li><a href=\"{0}.html\">{0}</a>", sample.name).expect("write string");
        if let Some(mockup) = samples::mockup(sample) {
            write!(html, " <span>{}</span>", escape(&mockup)).expect("write string");
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul></body></html>\n");
    html
}
