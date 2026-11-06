//! A clipboard write in the act's event (skein/docs/design/ui-web.md, 14.3).
use skein_lib::{Queue, Token};
use skein_ui::{Act, Binding, Builder, NodeId, Owner, Role, Run, View};
use skein_ui_web::{
    Look, clipboard,
    clipboard::web::Clipboard,
    inbox::Inbox,
    mapping::Primary,
    page::{Page, web::Dom},
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};

enum Report {
    Start,
    Act(NodeId, Act),
    Copied(clipboard::Report),
}

pub(super) fn start(mount: web_sys::Element, primary: Primary) {
    let window = web_sys::window().expect("window");
    let query = window.location().search().expect("page query");
    if query == "?no-clipboard" {
        let descriptor = js_sys::Object::new();
        js_sys::Reflect::set(&descriptor, &JsValue::from_str("value"), &JsValue::UNDEFINED)
            .expect("clipboard descriptor");
        let _navigator = js_sys::Object::define_property(
            window.navigator().unchecked_ref::<js_sys::Object>(),
            &JsValue::from_str("clipboard"),
            &descriptor,
        );
    }
    let limits = skein_ui_example_view::limits::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"copy",
        primary,
    )));
    let inbox = Inbox::new();
    let clipboard = Clipboard::new(Rc::clone(&inbox), Report::Copied);
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut view = View::new(&limits);
    let mut reports = 0_u32;
    let mut active = false;
    let mut status = String::from("Ready");
    inbox.start(Box::new(move |inbox| {
        while let Some(report) = inbox.next() {
            let write = match report {
                Report::Start => query == "?at-start",
                Report::Act(node, act) => view.act(node, act).is_some(),
                Report::Copied(report) => {
                    assert_eq!(report.token, 1, "the write's token");
                    reports = reports.checked_add(1).expect("fixture report count fits");
                    let outcome = match report.outcome {
                        clipboard::Outcome::Done => "Done",
                        clipboard::Outcome::Refused => "Refused",
                        clipboard::Outcome::Failed => "Failed",
                    };
                    status = format!("{outcome} {}, {reports}", if active { "active" } else { "idle" });
                    false
                }
            };
            if write {
                active = activation();
                clipboard.write(1, b"Copied by a person's act.");
            }
        }
        let mut builder = view.builder();
        build(&mut builder, status.as_bytes());
        let mut patches = Queue::with_capacity(limits.patches);
        view.finish(builder, u64::from(reports), &mut patches);
        let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
        let mut ops = Vec::new();
        page.borrow_mut().apply(&patches, &mut ops);
        dom.apply(&ops);
    }));
    inbox.raise(Report::Start);
}

fn activation() -> bool {
    let navigator = web_sys::window().expect("window").navigator();
    js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("userActivation"))
        .ok()
        .and_then(|activation| js_sys::Reflect::get(&activation, &JsValue::from_str("isActive")).ok())
        .and_then(|active| active.as_bool())
        .expect("browser user activation")
}

fn build(builder: &mut Builder, status: &[u8]) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    builder.open(Role::Button);
    builder.name(b"Copy");
    builder.bind(Binding { owner: Owner { number: 1 }, token: Token::new(1) });
    builder.close();
    builder.open(Role::Status);
    builder.text(Run::Plain, status);
    builder.close();
    builder.close();
    builder.close();
}
