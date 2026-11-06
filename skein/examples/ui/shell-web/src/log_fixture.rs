//! Debug-only log fixture for the browser story (skein/docs/design/ui-web.md, 7).
use skein_lib::{Queue, Token};
use skein_ui::{Binding, Builder, Owner, Role, Run, State, View};
use skein_ui_web::{
    Look,
    inbox::Inbox,
    mapping::Primary,
    page::{Page, web::Dom},
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

const OWNER: Owner = Owner { number: 1 };
// Thirty paragraphs overflow the fixture's 120px log.
const LINES: u32 = 30;

pub(super) fn start(mount: &web_sys::Element, primary: Primary) {
    let limits = skein_ui_example_view::limits::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"log",
        primary,
    )));
    let document = mount.owner_document().expect("document");
    let position = document.create_element("p").expect("position");
    let added = document.create_element("p").expect("addition result");
    let inbox = Inbox::new();
    let dom = Dom::new(mount.clone(), &page, &inbox, |node, act| (node, act), &[]);
    let mut view = View::new(&limits);
    let mut lines = LINES;
    apply(&mut view, &page, &dom, lines);
    let log = mount.query_selector("[role=log]").expect("log selector").expect("log");
    let sheet = document
        .query_selector("link[rel=stylesheet]")
        .expect("stylesheet selector")
        .expect("bundle stylesheet")
        .unchecked_into::<web_sys::HtmlLinkElement>()
        .sheet()
        .expect("loaded bundle stylesheet")
        .unchecked_into::<web_sys::CssStyleSheet>();
    sheet.insert_rule("#app [role=log]{height:120px;overflow-y:auto;overflow-anchor:none}").expect("fixture log rule");
    log.set_scroll_top(log.scroll_height());
    let (scrolled, status) = (log.clone(), position.clone());
    let listener = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
        status.set_text_content(Some(if at_end(&scrolled) { "At end" } else { "Away" }));
    });
    log.add_event_listener_with_callback("scroll", listener.as_ref().unchecked_ref()).expect("scroll listener");
    mount.append_child(&position).expect("position mounted");
    mount.append_child(&added).expect("addition result mounted");
    inbox.start(Box::new(move |inbox| {
        let _keep_listener_alive = &listener;
        while let Some((node, act)) = inbox.next() {
            if let Some(bound) = view.act(node, act) {
                match bound.binding.token.raw() {
                    1 => log.set_scroll_top(0),
                    2 => {
                        lines = lines.checked_add(1).expect("fixture line count fits");
                        apply(&mut view, &page, &dom, lines);
                        added.set_text_content(Some(if at_end(&log) {
                            "New line followed"
                        } else {
                            "New line missed"
                        }));
                    }
                    _ => unreachable!("fixture binding"),
                }
            }
        }
    }));
}

fn at_end(log: &web_sys::Element) -> bool {
    log.scroll_top().saturating_add(log.client_height()) >= log.scroll_height().saturating_sub(1)
}
fn apply(view: &mut View, page: &RefCell<Page<'_>>, dom: &Dom, lines: u32) {
    let mut builder = view.builder();
    build(&mut builder, lines);
    let mut patches = Queue::with_capacity(skein_ui_example_view::limits::limits().patches);
    view.finish(builder, u64::from(lines), &mut patches);
    let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
    let mut ops = Vec::new();
    page.borrow_mut().apply(&patches, &mut ops);
    dom.apply(&ops);
}
fn build(builder: &mut Builder, lines: u32) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    builder.open(Role::Log);
    builder.name(b"Fixture log");
    for line in 1..=lines {
        builder.open(Role::Paragraph);
        builder.text(Run::Plain, format!("Line {line}").as_bytes());
        builder.close();
    }
    builder.close();
    for (name, token) in [(b"Scroll away".as_slice(), 1), (b"Add line".as_slice(), 2)] {
        builder.open(Role::Button);
        builder.name(name);
        builder.bind(Binding { owner: OWNER, token: Token::new(token) });
        builder.close();
    }
    builder.open(Role::Button);
    builder.name(b"Latest");
    builder.state(State::ToLogEnd);
    builder.close();
    builder.close();
    builder.close();
}
