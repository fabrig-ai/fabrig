//! Debug-only disclosure fixture for the real browser story (skein/docs/design/ui-web.md, 7).
use skein_lib::{Queue, Token};
use skein_ui::{Binding, Builder, Owner, Role, Run, Shortcut, State, View};
use skein_ui_web::{
    Look,
    inbox::Inbox,
    mapping::Primary,
    page::{Page, web::Dom},
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

const OWNER: Owner = Owner { number: 1 };

pub(super) fn start(mount: web_sys::Element, primary: Primary) {
    let limits = skein_ui_example_view::limits::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"folds",
        primary,
    )));
    let document = mount.owner_document().expect("document");
    let component = document.create_element("button").expect("button");
    component.set_attribute("data-component", "fixture").expect("component marker");
    component.set_text_content(Some("Component outside"));
    let clicked = component.clone();
    let listener = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
        clicked.set_attribute("data-pressed", "yes").expect("component press");
    });
    component.add_event_listener_with_callback("click", listener.as_ref().unchecked_ref()).expect("component listener");
    let inbox = Inbox::new();
    let dom = Dom::new(mount, &page, &inbox, |node, act| (node, act), &[]);
    let mut view = View::new(&limits);
    let mut builder = view.builder();
    build(&mut builder, true);
    let mut patches = Queue::with_capacity(limits.patches);
    view.finish(builder, 0, &mut patches);
    let initial: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
    let mut ops = Vec::new();
    page.borrow_mut().apply(&initial, &mut ops);
    dom.apply(&ops);
    document.get_element_by_id("app").expect("mount").append_child(&component).expect("component mounted");
    // Escape removes the editor's form; other acts leave the disclosures in place.
    inbox.start(Box::new(move |inbox| {
        let _keep_dom_alive = (&dom, &listener);
        while let Some((node, act)) = inbox.next() {
            if let Some(bound) = view.act(node, act) {
                let document = web_sys::window().expect("window").document().expect("document");
                let status = document.query_selector("[role=status]").expect("status selector").expect("status");
                status.set_text_content(Some(&format!("Pressed {}", bound.binding.token.raw())));
                if bound.binding.token == Token::new(5) {
                    let mut builder = view.builder();
                    build(&mut builder, false);
                    view.finish(builder, 1, &mut patches);
                    let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
                    let mut ops = Vec::new();
                    page.borrow_mut().apply(&patches, &mut ops);
                    dom.apply(&ops);
                }
            }
        }
    }));
}

fn button(builder: &mut Builder, name: &[u8], token: u64) {
    builder.open(Role::Button);
    builder.name(name);
    builder.bind(Binding { owner: OWNER, token: Token::new(token) });
}
fn build(builder: &mut Builder, editing: bool) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    builder.open(Role::Form);
    builder.name(b"Choices");
    button(builder, b"Form escape", 1);
    builder.shortcut(Shortcut::ESCAPE);
    builder.close();
    for (name, inside, folds) in [
        (b"Panel".as_slice(), b"Inside panel".as_slice(), true),
        (b"Plain".as_slice(), b"Inside plain".as_slice(), false),
        (b"Call".as_slice(), b"Inside call".as_slice(), false),
    ] {
        builder.open(Role::Disclosure);
        builder.name(name);
        if folds {
            builder.state(State::FoldsAway);
        }
        button(builder, inside, 2);
        builder.close();
        if folds {
            builder.open(Role::Menu);
            builder.name(b"Actions");
            builder.open(Role::MenuItem);
            builder.name(b"Menu item");
            builder.bind(Binding { owner: OWNER, token: Token::new(3) });
            builder.close();
            builder.close();
        }
        builder.close();
    }
    button(builder, b"Outside", 4);
    builder.close();
    builder.close();
    builder.open(Role::Disclosure);
    builder.name(b"Editor");
    builder.focus(1);
    if editing {
        builder.open(Role::Form);
        builder.name(b"Edit");
        builder.open(Role::TextField);
        builder.name(b"Name");
        builder.bind(Binding { owner: OWNER, token: Token::new(6) });
        builder.close();
        button(builder, b"Close editor", 5);
        builder.shortcut(Shortcut::ESCAPE);
        builder.close();
        builder.close();
    }
    builder.close();
    builder.open(Role::Status);
    builder.text(Run::Plain, b"No presses");
    builder.close();
    builder.close();
    builder.close();
}
