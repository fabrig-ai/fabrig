//! Public fetch, visibility and history; no small store (temper/docs/design/client/web/public.md, 2).
use super::{PublicHost, Report};
use jig_client_shell_web::adapter;
use skein_ui::{Act, NodeId};
use skein_ui_web::{
    fetch, history,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform, timer, title,
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};

enum Input {
    Start,
    Due,
    Act(NodeId, Act),
    Fetch(fetch::Report),
    Place(Box<[u8]>),
    Visible(bool),
}
struct Shell {
    host: PublicHost,
    page: Rc<RefCell<Page<'static>>>,
    dom: Dom,
    fetch: fetch::web::Fetch<Input>,
    history: history::web::History,
    timer: timer::web::Timer,
    title: title::web::Title,
}
pub(crate) fn start(path: &[u8]) {
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let origin = window.location().origin().expect("origin");
    let ways = document
        .query_selector("meta[name=\"temper-sign-in\"]")
        .expect("sign-in selector")
        .and_then(|meta| meta.get_attribute("content"))
        .map_or(temper_client_view::sign_in::Ways::NONE, |content| {
            temper_client_view::sign_in::Ways::parse(&content, origin.as_bytes())
        });
    let facts = platform::facts();
    let inbox = Inbox::new();
    let mut host = PublicHost::new(facts.now, facts.wall, ways);
    // Visibility is supplied before Open, so a tab opened hidden sends no GET.
    host.report(Report::Visible { visible: !document.hidden() });
    host.start(path);
    let page = Rc::new(RefCell::new(crate::host::page(&host.bounds, facts.primary)));
    let dom = Dom::new(document.get_element_by_id("app").expect("bundle mount"), &page, &inbox, Input::Act, &[]);
    let fetch = fetch::web::Fetch::new(crate::limits::EXCHANGES, Rc::clone(&inbox), Input::Fetch);
    let history = history::web::History::new(Rc::clone(&inbox), Input::Place);
    let timer = timer::web::Timer::new(Rc::clone(&inbox), || Input::Due);
    let listener_inbox = Rc::clone(&inbox);
    let listener = Closure::<dyn Fn()>::new(move || {
        let document = web_sys::window().expect("window").document().expect("document");
        listener_inbox.raise(Input::Visible(!document.hidden()));
    });
    document
        .add_event_listener_with_callback("visibilitychange", listener.as_ref().unchecked_ref())
        .expect("public visibility listener");
    listener.forget();
    let mut shell = Shell { host, page, dom, fetch, history, timer, title: title::web::Title::default() };
    inbox.start(Box::new(move |inbox| shell.turn(inbox)));
    inbox.raise(Input::Start);
}
impl Shell {
    fn turn(&mut self, inbox: &Inbox<Input>) {
        let (now, wall) = platform::clocks();
        self.host.clocks(now, wall);
        let mut new_page = false;
        while let Some(input) = inbox.next() {
            let report = match input {
                Input::Start => Report::Start,
                Input::Due => {
                    self.timer.fired();
                    Report::Due
                }
                Input::Act(node, act) => Report::Act { node, act },
                Input::Fetch(report) => Report::Heard(adapter::fetched(report)),
                Input::Place(path) => Report::Place { path },
                Input::Visible(visible) => Report::Visible { visible },
            };
            let previous: Vec<_> = self.host.tickets.keys().copied().collect();
            let duties = self.host.report(report);
            if let Some((path, replace)) = duties.show {
                for id in previous {
                    self.fetch.abandon(id);
                }
                self.history.show(&path, replace);
                new_page = !replace;
            }
            // Only exchanges leave the public host; there is no store or stream transport.
            for exchange in adapter::requests(duties.asked).exchanges {
                self.fetch.exchange(&exchange);
            }
        }
        let mut ops = Vec::new();
        self.host.draw(&mut self.page.borrow_mut(), &mut ops);
        self.dom.apply(&ops);
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let heading = document.query_selector("main h1").expect("heading selector");
        let words = heading.as_ref().and_then(|element| element.text_content()).unwrap_or_default();
        self.title.set(words.as_bytes(), b"temper", b"en", false);
        if new_page {
            window.scroll_to_with_x_and_y(0.0, 0.0);
            if let Some(heading) = heading {
                heading.set_attribute("tabindex", "-1").expect("heading focus");
                heading.unchecked_ref::<web_sys::HtmlElement>().focus().expect("focus public heading");
            }
        }
        self.history.restore();
        self.timer.set(self.host.deadline(), now);
    }
}
