//! Debug-only log owner and measurements for Chromium (skein/docs/design/ui-web.md, 7).
use skein_lib::{Queue, Token};
use skein_ui::{Act, Binding, Builder, Key, LogEdge, NodeId, Owner, Role, Run, State, View};
use skein_ui_web::{
    Look,
    inbox::Inbox,
    mapping::Primary,
    page::{Page, web::Dom},
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

const OWNER: Owner = Owner { number: 1 };

#[expect(
    clippy::struct_excessive_bools,
    reason = "the fixture controls independent reading states and records the busy observation"
)]
struct Reading {
    first: u64,
    end: u64,
    remaining: u64,
    page: u64,
    reads: u64,
    heads: u64,
    feet: u64,
    automatic: bool,
    busy: bool,
    empty_page: bool,
    empty_before: i32,
    empty_after: i32,
    empty_was_busy: bool,
    continues: bool,
    request: u64,
    dialog: bool,
    remembered: Option<u64>,
}

impl Reading {
    fn act(&mut self, act: &Act, token: u64, document: &web_sys::Document) -> (bool, bool, bool) {
        let mut changed = false;
        let mut move_back = false;
        let mut to_foot = false;
        match act {
            Act::Near { edge: LogEdge::Head } => {
                self.heads += 1;
                if self.automatic && !self.busy {
                    self.reads += 1;
                    if self.empty_page || self.remaining > 0 {
                        self.busy = true;
                        changed = true;
                    }
                }
            }
            Act::Near { edge: LogEdge::Foot } => self.feet += 1,
            Act::Press => match token {
                1 => {
                    self.automatic = false;
                    move_back = true;
                }
                2 => {
                    self.first -= 8;
                    changed = true;
                }
                3 => {
                    self.first += 8;
                    changed = true;
                }
                4 => {
                    self.end += 1;
                    changed = true;
                }
                5 => {
                    self.continues = true;
                    changed = true;
                    to_foot = true;
                }
                6 => {
                    self.continues = false;
                    self.request += 1;
                    changed = true;
                }
                7 => self.remembered = first_visible(document),
                8 => {}
                9 => {
                    document
                        .active_element()
                        .expect("active button")
                        .unchecked_ref::<web_sys::HtmlElement>()
                        .blur()
                        .expect("body takes focus");
                }
                10 => {
                    self.dialog = true;
                    changed = true;
                }
                11 => {
                    self.dialog = false;
                    changed = true;
                }
                token => panic!("unknown fixture button {token}"),
            },
            Act::Input { .. } => {}
            Act::Submit | Act::Toggle | Act::Drawing { .. } => unreachable!("fixture acts"),
        }
        (changed, move_back, to_foot)
    }
}

enum Report {
    Start,
    Read,
    Act(NodeId, Act),
}

pub(super) fn start(mount: web_sys::Element, path: &str, primary: Primary) {
    let document = mount.owner_document().expect("document");
    let sheet = document
        .style_sheets()
        .item(0)
        .expect("example stylesheet")
        .dyn_into::<web_sys::CssStyleSheet>()
        .expect("CSS stylesheet");
    let anchor = if path.ends_with("/anchored") { "auto" } else { "none" };
    let short = path.ends_with("/short");
    let height = if short { "auto" } else { "200px" };
    sheet.insert_rule(&format!("#app [role=log] {{ height:{height}; max-height:200px; width:400px; overflow:auto; overflow-anchor:{anchor}; line-height:20px; font-size:16px; }}"))
        .expect("fixture log rule");
    sheet.insert_rule("#app [role=log] ul { margin:0; padding:0; list-style:none; }").expect("fixture list rule");
    let mut reading = Reading {
        first: 100,
        end: if short { 102 } else { 108 },
        remaining: if short { 2 } else { 20 },
        page: if short { 2 } else { 8 },
        reads: 0,
        heads: 0,
        feet: 0,
        automatic: true,
        busy: false,
        empty_page: short,
        empty_before: 0,
        empty_after: 0,
        empty_was_busy: false,
        continues: false,
        request: 0,
        dialog: false,
        remembered: None,
    };
    let limits = skein_ui_example_view::limits::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"reading",
        primary,
    )));
    let inbox = Inbox::new();
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut view = View::new(&limits);
    let mut revision = 0_u64;
    inbox.start(Box::new(move |inbox| {
        while let Some(report) = inbox.next() {
            let (changed, move_back, to_foot) = match report {
                Report::Start => (true, false, false),
                Report::Read => {
                    assert!(reading.busy, "read is outstanding");
                    if reading.empty_page {
                        let log = log(&document);
                        reading.empty_before = log.scroll_height();
                        reading.empty_was_busy = log.get_attribute("aria-busy").as_deref() == Some("true");
                        reading.empty_page = false;
                    } else {
                        reading.remaining -= 1;
                        reading.first -= reading.page;
                    }
                    reading.busy = false;
                    (true, false, false)
                }
                Report::Act(node, act) => view.act(node, act).map_or((false, false, false), |bound| {
                    reading.act(&bound.act, bound.binding.token.raw(), &document)
                }),
            };
            if changed {
                let mut builder = view.builder();
                build(&mut builder, &reading);
                let mut patches = Queue::with_capacity(limits.patches);
                view.finish(builder, revision, &mut patches);
                revision += 1;
                let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
                let mut ops = Vec::new();
                page.borrow_mut().apply(&patches, &mut ops);
                dom.apply(&ops);
                if reading.busy {
                    inbox.raise(Report::Read);
                } else if reading.empty_before > 0 && reading.empty_after == 0 {
                    reading.empty_after = log(&document).scroll_height();
                }
            }
            let log = log(&document);
            if move_back || to_foot {
                log.set_scroll_top(if to_foot { log.scroll_height() } else { 240 });
                log.dispatch_event(&web_sys::Event::new("scroll").expect("scroll event")).expect("measure scroll");
            }
            facts(&document, &reading);
        }
    }));
    inbox.raise(Report::Start);
}

fn log(document: &web_sys::Document) -> web_sys::Element {
    document.query_selector("[role=log]").expect("log selector").expect("fixture log")
}

fn first_visible(document: &web_sys::Document) -> Option<u64> {
    let log = log(document);
    let top = log.get_bounding_client_rect().top();
    let children = log.query_selector("ul").expect("list selector").expect("log list").children();
    (0..children.length())
        .filter_map(|index| children.item(index))
        .find(|entry| entry.get_bounding_client_rect().bottom() > top)
        .and_then(|entry| entry.text_content()?.strip_prefix("Line ")?.parse().ok())
}

fn facts(document: &web_sys::Document, reading: &Reading) {
    let log = log(document);
    for (name, value) in [
        ("top", log.scroll_top().to_string()),
        ("height", log.scroll_height().to_string()),
        ("viewport", log.client_height().to_string()),
        ("reads", reading.reads.to_string()),
        ("empty-before", reading.empty_before.to_string()),
        ("empty-after", reading.empty_after.to_string()),
        ("empty-was-busy", reading.empty_was_busy.to_string()),
        ("feet", reading.feet.to_string()),
        ("heads", reading.heads.to_string()),
        ("focus", document.active_element().expect("active element").tag_name()),
        ("away", log.has_attribute("data-log-away").to_string()),
    ] {
        log.set_attribute(&format!("data-{name}"), &value).expect("fixture fact");
    }
    if let Some(line) = reading.remembered {
        let children = log.query_selector("ul").expect("list selector").expect("log list").children();
        let words = format!("Line {line}");
        let entry = (0..children.length())
            .filter_map(|index| children.item(index))
            .find(|entry| entry.text_content().as_deref() == Some(&words))
            .expect("remembered line survives");
        log.set_attribute("data-position", &entry.get_bounding_client_rect().top().to_string()).expect("position fact");
    }
}

fn build(builder: &mut Builder, reading: &Reading) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    builder.open(Role::Log);
    builder.name(b"Reading");
    builder.bind(Binding { owner: OWNER, token: Token::new(100) });
    builder.end_request(reading.request);
    if reading.busy {
        builder.state(State::Busy);
    }
    if reading.continues {
        builder.state(State::Continues);
    }
    builder.open(Role::List);
    for line in reading.first..reading.end {
        builder.open(Role::Item);
        builder.key(Key { owner: OWNER, kind: 1, a: line, b: 0 });
        builder.text(Run::Plain, format!("Line {line}").as_bytes());
        builder.close();
    }
    builder.close();
    builder.close();
    builder.open(Role::TextArea);
    builder.name(b"Message");
    builder.bind(Binding { owner: OWNER, token: Token::new(101) });
    builder.close();
    for (token, name) in [
        (1, "Read back"),
        (2, "Prepend"),
        (3, "Remove head"),
        (4, "Append"),
        (5, "Continues"),
        (6, "End"),
        (7, "Remember line"),
        (8, "Inspect"),
        (9, "Body focus"),
        (10, "Open dialog"),
    ] {
        button(builder, token, name);
    }
    if reading.dialog {
        builder.open(Role::Dialog);
        button(builder, 8, "Inspect dialog");
        button(builder, 11, "Close dialog");
        builder.close();
    }
    builder.close();
    builder.close();
}

fn button(builder: &mut Builder, token: u64, name: &str) {
    builder.open(Role::Button);
    builder.name(name.as_bytes());
    builder.bind(Binding { owner: OWNER, token: Token::new(token) });
    builder.close();
}
