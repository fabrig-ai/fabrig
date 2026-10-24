//! Temper's browser shell: platform facts enter, the root's requests leave.
#![cfg(target_arch = "wasm32")]
#![forbid(unsafe_code)]
use jig_client_shell_web::{adapter, sign_in};
use skein_ui::{Act, NodeId};
use skein_ui_web::{
    Look, fetch, history,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform,
    store::Store,
    timer, title, transport, wasm_bindgen,
};
use std::{cell::RefCell, rc::Rc};
mod host;
mod limits;
mod records;
enum Report {
    Start,
    Due,
    Act(NodeId, Act),
    Fetch(fetch::Report),
    Carried(Vec<transport::Carried>, u32),
    Place(Box<[u8]>),
}
struct Shell {
    host: host::Host,
    page: Rc<RefCell<Page<'static>>>,
    dom: Dom,
    platform: platform::Browser<Report>,
    store: Store,
    history: history::web::History,
    timer: timer::web::Timer,
    title: title::web::Title,
}
#[wasm_bindgen(start)]
pub fn start() {
    if !skein_ui_web::restart::Restart::new(
        3,
        skein_lib::Duration::from_secs(60),
        "temper-restarts",
        "The application failed.",
    )
    .install("app")
    {
        return;
    }
    // Replace the printed secret's address before starting anything that can send.
    let secret = sign_in::web::take_from_page(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/welcome" });
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let ways = document
        .query_selector("meta[name=\"temper-sign-in\"]")
        .expect("sign-in selector")
        .and_then(|meta| meta.get_attribute("content"))
        .map_or(temper_client_view::sign_in::Ways::NONE, |content| {
            let origin = window.location().origin().expect("origin");
            temper_client_view::sign_in::Ways::parse(&content, origin.as_bytes())
        });
    let mount = document.get_element_by_id("app").expect("bundle mount");
    let facts = platform::facts();
    let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
    #[cfg(debug_assertions)]
    assert_ne!(path.split('?').next(), Some("/test/panic"), "the test address panics at start");
    let inbox = Inbox::new();
    let platform =
        platform::Browser::new(fetch::web::Fetch::new(limits::EXCHANGES + 1, Rc::clone(&inbox), Report::Fetch));
    let store = Store::new(b"temper.", None);
    let mut host = host::Host::new(facts.seed, facts.now, facts.wall, ways);
    host.forge_sign_in(forge_sign_in(&document));
    host.start(path.as_bytes(), store.read_all(&platform), secret);
    let page = Rc::new(RefCell::new(Page::new(
        Look {
            names: &[temper_client_view::look::NAMES],
            pictures: &[],
            draw: |_, _, _, _, _| false,
            components: &[],
            elements: temper_client_view::look::ELEMENTS,
        },
        &host.bounds.tree,
        b"temper",
        facts.primary,
    )));
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let history = history::web::History::new(Rc::clone(&inbox), Report::Place);
    let timer = timer::web::Timer::new(Rc::clone(&inbox), || Report::Due);
    let mut shell = Shell { host, page, dom, platform, store, history, timer, title: title::web::Title::default() };
    inbox.start(Box::new(move |inbox| shell.turn(inbox)));
    inbox.raise(Report::Start);
}
impl Shell {
    fn turn(&mut self, inbox: &Inbox<Report>) {
        let (now, wall) = platform::clocks();
        self.host.clocks(now, wall);
        let mut new_page = false;
        while let Some(report) = inbox.next() {
            let read_on =
                matches!(&report, Report::Fetch(fetch::Report::StreamOpened | fetch::Report::StreamPiece { .. }));
            match report {
                Report::Start => {}
                Report::Due => {
                    self.timer.fired();
                    self.host.due();
                }
                Report::Act(node, act) => self.host.act(node, act),
                Report::Place(path) => self.host.went(&path),
                Report::Fetch(report) => self.host.heard(adapter::fetched(report)),
                Report::Carried(reports, writes) => self.host.carried(reports, writes),
            }
            let (asked, writes) = self.host.duties();
            let mut requests = adapter::requests(asked);
            requests.navigate = self.host.navigate.take();
            if requests.navigate.is_some() {
                // This acknowledged departure erases its finished request before
                // navigating. The previous turn's close warning is already stale.
                web_sys::window().expect("window").set_onbeforeunload(None);
            }
            if let Some(to) = self.host.replace.take() {
                let window = web_sys::window().expect("window");
                window.set_onbeforeunload(None);
                window
                    .location()
                    .replace(std::str::from_utf8(&to).expect("jig address is UTF-8"))
                    .expect("replace authorization address");
            }
            let mut carried = Vec::new();
            transport::carry(requests, &mut self.store, &mut self.platform, &mut carried);
            if writes != 0 || !carried.is_empty() {
                inbox.raise(Report::Carried(carried, writes));
            }
            if read_on {
                self.platform.fetch.read_on();
            }
            if let Some((path, replace)) = self.host.show.take() {
                self.history.show(&path, replace);
                new_page = !replace;
            }
        }
        let patches = self.host.patches();
        let mut ops = Vec::new();
        self.page.borrow_mut().apply(&patches, &mut ops);
        let links = self.page.borrow().links().to_vec();
        for node in links {
            if let Some(path) = self.host.href(node) {
                self.page.borrow_mut().set_href(node, &path, &mut ops);
            }
        }
        self.dom.apply(&ops);
        let document = web_sys::window().expect("window").document().expect("document");
        // Nodes can be reused by a later page: remove this shell's previous
        // tab policy before applying it to the current forge links.
        for _ in 0..temper_client_view::forge_sign_in::FORGES {
            let Some(link) =
                document.query_selector("a[data-temper-forge-sign-in]").expect("previous forge link selector")
            else {
                break;
            };
            for name in ["target", "rel", "data-temper-forge-sign-in"] {
                link.remove_attribute(name).expect("previous forge link policy");
            }
        }
        // These are the page-owned Forgejo sign-in links. The semantic link
        // supplies its checked href; the browser owns opening a new tab.
        for _ in 0..temper_client_view::forge_sign_in::FORGES {
            let Some(link) =
                document.query_selector("a[href$='/user/oauth2/temper']:not([target])").expect("forge link selector")
            else {
                break;
            };
            link.set_attribute("data-temper-forge-sign-in", "").expect("forge link policy marker");
            link.set_attribute("target", "_blank").expect("link target");
            link.set_attribute("rel", "noopener noreferrer").expect("link relation");
        }
        self.finish(&ops, new_page, now);
    }
    fn finish(&mut self, ops: &[skein_ui_web::page::Op], new_page: bool, now: skein_lib::Time) {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let heading = document.query_selector("main h1").expect("heading selector");
        let words = heading.as_ref().and_then(|element| element.text_content()).unwrap_or_default();
        // Session storage survives reload, but closing the tab loses its requests.
        let closing_loses = self.store.read_all(&self.platform).iter().any(|record| record.key.len() == 16);
        self.title.set(words.as_bytes(), b"temper", b"en", closing_loses);
        if new_page {
            window.scroll_to_with_x_and_y(0.0, 0.0);
            if !ops.iter().any(|op| matches!(op, skein_ui_web::page::Op::Focus { .. })) {
                use wasm_bindgen::JsCast;
                if let Some(target) = heading.or_else(|| document.query_selector("main").expect("main selector")) {
                    target.set_attribute("tabindex", "-1").expect("focus target");
                    target.unchecked_ref::<web_sys::HtmlElement>().focus().expect("focus mounted target");
                }
            }
        }
        self.history.restore();
        self.timer.set(self.host.deadline(), now);
    }
}

fn forge_sign_in(document: &web_sys::Document) -> temper_client_view::forge_sign_in::Forges {
    let mut facts = temper_client_view::forge_sign_in::Forges::new();
    let mut selector = String::from("meta[name='temper-forge-sign-in']");
    for _ in 0..temper_client_view::forge_sign_in::FORGES {
        let Some(meta) = document.query_selector(&selector).expect("forge meta selector") else { break };
        if let Some(content) = meta.get_attribute("content") {
            facts.read(&content);
        }
        selector.push_str(" ~ meta[name='temper-forge-sign-in']");
    }
    facts
}
