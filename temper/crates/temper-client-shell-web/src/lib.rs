//! The client host on every target and the browser's shell on wasm32.
#![forbid(unsafe_code)]
#[cfg(target_arch = "wasm32")]
use jig_client_shell_web::{adapter, sign_in};
#[cfg(target_arch = "wasm32")]
use skein_ui::{Act, NodeId};
#[cfg(target_arch = "wasm32")]
use skein_ui_web::{
    clipboard, fetch, history,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform, screen,
    store::Store,
    timer, title, transport,
};
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
mod focus;
#[cfg(target_arch = "wasm32")]
mod push;
pub mod host;
#[cfg(any(not(target_arch = "wasm32"), feature = "record"))]
pub mod journey;
pub mod limits;
pub mod playground;
pub mod public;
pub mod records;
#[cfg(target_arch = "wasm32")]
enum Report {
    Push(temper_client_domain::pages::told::Report),
    Start,
    #[cfg(feature = "record")]
    Mark,
    #[cfg(feature = "record")]
    Save,
    Due,
    TabRefused,
    Act(NodeId, Act),
    Fetch(fetch::Report),
    Carried(Vec<transport::Carried>, u32),
    Copied(clipboard::Report),
    Screen(screen::Report),
    Place(Box<[u8]>),
    Frame(playground::Control),
    World(platform::worker::Heard),
}
#[cfg(target_arch = "wasm32")]
/// The shared host loop (temper/docs/design/playground.md, 6).
struct Shell<P: ShellPlatform> {
    host: host::Host,
    push_inbox: Rc<Inbox<Report>>,
    #[cfg(feature = "record")]
    recorder: Option<journey::Recorder>,
    page: Rc<RefCell<Page<'static>>>,
    dom: Dom,
    platform: P,
    store: Store,
    kept: Store,
    screen: screen::web::Screen<Report>,
    clipboard: clipboard::web::Clipboard<Report>,
    history: Rc<history::web::History>,
    timer: P::Timer,
    title: title::web::Title,
    heading_focus: focus::HeadingFocus,
}
#[cfg(target_arch = "wasm32")]
trait ShellPlatform: platform::Platform {
    type Timer;
    fn clocks(&self) -> (skein_lib::Time, skein_lib::Wall);
    fn fired(timer: &mut Self::Timer);
    fn end(&self, timer: &mut Self::Timer);
    fn timer(&self, timer: &mut Self::Timer, deadline: Option<skein_lib::Time>, now: skein_lib::Time);
}
#[cfg(target_arch = "wasm32")]
impl ShellPlatform for platform::Browser<Report> {
    type Timer = timer::web::Timer;
    fn clocks(&self) -> (skein_lib::Time, skein_lib::Wall) {
        platform::clocks()
    }
    fn fired(timer: &mut Self::Timer) {
        timer.fired();
    }
    fn end(&self, timer: &mut Self::Timer) {
        timer.set(None, platform::clocks().0);
    }
    fn timer(&self, timer: &mut Self::Timer, deadline: Option<skein_lib::Time>, now: skein_lib::Time) {
        timer.set(deadline, now);
    }
}
#[cfg(target_arch = "wasm32")]
#[cfg_attr(not(feature = "playground"), wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    push::manifest();
    let window = web_sys::window().expect("window");
    let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
    if matches!(temper_client_domain::address::parse(path.as_bytes()), temper_client_domain::Place::Public(_)) {
        public::web::start(path.as_bytes());
        return;
    }
    start_signed_in();
}
#[cfg(target_arch = "wasm32")]
fn start_signed_in() {
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
        .map(|content| {
            let origin = window.location().origin().expect("origin");
            (content, origin)
        });
    let parsed_ways = ways.as_ref().map_or(temper_client_view::sign_in::Ways::NONE, |(content, origin)| {
        temper_client_view::sign_in::Ways::parse(content, origin.as_bytes())
    });
    let mount = document.get_element_by_id("app").expect("bundle mount");
    let facts = platform::facts();
    let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
    #[cfg(debug_assertions)]
    assert_ne!(path.split('?').next(), Some("/test/panic"), "the test address panics at start");
    let inbox = Inbox::new();
    #[cfg(feature = "record")]
    record_keys(&document, &inbox);
    let platform =
        platform::Browser::new(fetch::web::Fetch::new(limits::EXCHANGES + 1, Rc::clone(&inbox), Report::Fetch));
    let store = Store::new(b"temper.", None);
    let mut host = host::Host::new(facts.seed, facts.now, facts.wall, facts.offset, parsed_ways);
    #[cfg(feature = "record")]
    let mut forges = Vec::new();
    host.forge_sign_in(forge_sign_in(
        &document,
        #[cfg(feature = "record")]
        &mut forges,
    ));
    let kept = Store::kept(b"temper.", None);
    let mut stored = store.read_all(&platform);
    stored.extend(kept.read_all(&platform).into_iter().filter(|record| record.key.as_ref() == records::LOOK_KEY));
    #[cfg(feature = "record")]
    let recorder = Some(journey::Recorder::new(&journey::Facts {
        seed: facts.seed,
        now: facts.now,
        wall: facts.wall,
        primary: facts.primary,
        zone_minutes: facts.offset,
        ways: ways.map(|(content, origin)| (content.into_bytes().into(), origin.into_bytes().into())),
        forges,
        path: path.as_bytes().into(),
        secret: secret.clone(),
        stored: stored.clone(),
    }));
    host.start(path.as_bytes(), stored, secret);
    night(&document, host.night());
    let page = Rc::new(RefCell::new(host::page(&host.bounds.tree, facts.primary)));
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let history = Rc::new(history::web::History::new(Rc::clone(&inbox), Report::Place));
    let timer = timer::web::Timer::new(Rc::clone(&inbox), || Report::Due);
    let mut shell = Shell {
        host,
        push_inbox: Rc::clone(&inbox),
        #[cfg(feature = "record")]
        recorder,
        page,
        dom,
        platform,
        store,
        kept,
        screen: screen::web::Screen::new(Rc::clone(&inbox), Report::Screen),
        clipboard: clipboard::web::Clipboard::new(Rc::clone(&inbox), Report::Copied),
        history,
        timer,
        title: title::web::Title::default(),
        heading_focus: focus::HeadingFocus::new(),
    };
    push::state(Rc::clone(&inbox));
    inbox.start(Box::new(move |inbox| shell.turn(inbox)));
    inbox.raise(Report::Start);
}
/// The page follows the domain's look even when keeping it is refused (client/pages/README.md).
#[cfg(target_arch = "wasm32")]
fn night(document: &web_sys::Document, on: bool) {
    let html = document.document_element().expect("document html");
    if html.has_attribute("data-night") == on {
        return;
    }
    if on {
        html.set_attribute("data-night", "").expect("set night look");
    } else {
        html.remove_attribute("data-night").expect("remove night look");
    }
}
#[cfg(target_arch = "wasm32")]
impl<P: ShellPlatform> Shell<P> {
    /// One inbox turn: each report given to the host, its duties carried, then the page drawn.
    fn turn(&mut self, inbox: &Inbox<Report>) {
        let now = self.begin();
        let mut new_page = false;
        while let Some(report) = inbox.next() {
            if let Some(shown) = self.report(inbox, report) {
                new_page = shown;
            }
        }
        self.draw(new_page, now);
    }
    fn begin(&mut self) -> skein_lib::Time {
        let (now, wall) = self.platform.clocks();
        #[cfg(feature = "record")]
        if let Some(recorder) = &mut self.recorder {
            recorder.turn(now, wall);
        }
        self.host.clocks(now, wall);
        now
    }
    /// Carries one host report in order; navigation says whether a new page shows.
    fn report(&mut self, inbox: &Inbox<Report>, report: Report) -> Option<bool> {
        let read_on = matches!(
            &report,
            Report::Fetch(
                fetch::Report::StreamOpened | fetch::Report::StreamRefused { .. } | fetch::Report::StreamPiece { .. }
            )
        );
        let report = self.host_report(report)?;
        #[cfg(feature = "record")]
        let duties = if let Some(recorder) = &mut self.recorder {
            recorder.report(&mut self.host, &self.page.borrow(), report)
        } else {
            self.host.report(report)
        };
        #[cfg(not(feature = "record"))]
        let duties = self.host.report(report);
        let document = web_sys::window().expect("window").document().expect("document");
        night(&document, self.host.night());
        let navigates = duties.show.is_some();
        let shown = self.carry(inbox, duties, read_on);
        navigates.then_some(shown)
    }
    fn draw(&mut self, new_page: bool, now: skein_lib::Time) {
        let mut ops = Vec::new();
        self.host.draw(&mut self.page.borrow_mut(), &mut ops);
        self.dom.apply(&ops);
        let document = web_sys::window().expect("window").document().expect("document");
        forge_links(&document);
        self.finish(&ops, new_page, now);
    }
    /// The host's report, or none for the shell's recording marks and saves; the timer told it fired.
    fn host_report(&mut self, report: Report) -> Option<host::Report> {
        match report {
            Report::Push(report) => host::Report::Push(report),
            Report::Start => host::Report::Start,
            Report::Frame(_) | Report::World(_) => return None,
            #[cfg(feature = "record")]
            Report::Mark => {
                if let Some(recorder) = &mut self.recorder {
                    recorder.mark(&self.page.borrow());
                }
                return None;
            }
            #[cfg(feature = "record")]
            Report::Save => {
                self.save_recording(false);
                return None;
            }
            Report::Due => {
                P::fired(&mut self.timer);
                host::Report::Due
            }
            Report::TabRefused => host::Report::TabRefused,
            Report::Screen(report) => host::Report::Screen(report),
            Report::Act(node, act) => host::Report::Act { node, act },
            Report::Place(path) => host::Report::Place { path },
            Report::Fetch(report) => host::Report::Heard(adapter::fetched(report)),
            Report::Copied(report) => host::Report::Copied {
                token: report.token,
                written: match report.outcome {
                    clipboard::Outcome::Done => temper_client_domain::copy::Written::Done,
                    clipboard::Outcome::Refused => temper_client_domain::copy::Written::Refused,
                    clipboard::Outcome::Failed => temper_client_domain::copy::Written::Failed,
                },
            },
            Report::Carried(reports, writes) => host::Report::Carried { reports, writes },
        }
        .into()
    }
    /// Carries one report's requests, navigation, store writes and stream read; returns whether a new page shows.
    fn carry(&mut self, inbox: &Inbox<Report>, duties: host::Duties, read_on: bool) -> bool {
        // Push permission is asked inside the press event, before any asynchronous work.
        for duty in duties.push { push::carry(Rc::clone(&self.push_inbox), duty); }
        // Keep the person's gesture: tabs precede transport in this turn
        // (skein/docs/design/ui/web/README.md, section 9).
        for tab in &duties.tab {
            match tab {
                host::TabAct::Open { words } => {
                    if !platform::Platform::open_tab(&mut self.platform, words) {
                        inbox.raise(Report::TabRefused);
                    }
                }
                host::TabAct::Point { to } => platform::Platform::point_tab(&mut self.platform, to),
                host::TabAct::Close => platform::Platform::close_tab(&mut self.platform),
            }
        }
        if let Some(on) = duties.screen {
            if on {
                self.screen.enter();
            } else {
                self.screen.leave();
            }
        }
        // Keep the person's gesture (skein/docs/design/ui/web/README.md, section 9).
        if let Some((token, words)) = &duties.copy {
            self.clipboard.write(*token, words);
        }
        #[cfg(feature = "record")]
        if duties.departs() {
            self.save_recording(true);
        }
        if let Some(look) = duties.keep_look {
            self.kept.write(records::LOOK_KEY, &look, &mut self.platform).ok();
        }
        let mut requests = adapter::requests(duties.asked);
        requests.navigate = duties.navigate;
        if requests.navigate.is_some() {
            // This acknowledged departure erases its finished request before
            // navigating. The previous turn's close warning is already stale.
            web_sys::window().expect("window").set_onbeforeunload(None);
        }
        if let Some(to) = duties.replace {
            let window = web_sys::window().expect("window");
            window.set_onbeforeunload(None);
            window
                .location()
                .replace(std::str::from_utf8(&to).expect("jig address is UTF-8"))
                .expect("replace authorization address");
        }
        let mut carried = Vec::new();
        transport::carry(requests, &mut self.store, &mut self.platform, &mut carried);
        if duties.writes != 0 || !carried.is_empty() {
            inbox.raise(Report::Carried(carried, duties.writes));
        }
        if read_on {
            platform::Platform::read_on(&mut self.platform);
        }
        if let Some((path, replace)) = duties.show {
            self.history.show(&path, replace);
            return !replace;
        }
        false
    }
    fn finish(&mut self, ops: &[skein_ui_web::page::Op], new_page: bool, now: skein_lib::Time) {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let heading = document.query_selector("main h1").expect("heading selector");
        let words = if document.query_selector("main.temper-home").expect("Home selector").is_some() {
            "Home".to_owned()
        } else if document.query_selector("main.temper-welcome").expect("Welcome selector").is_some() {
            "Welcome · first run".to_owned()
        } else {
            heading.as_ref().and_then(|element| element.text_content()).unwrap_or_default()
        };
        // Session storage survives reload, but closing the tab loses its requests.
        let closing_loses = self.store.read_all(&self.platform).iter().any(|record| record.key.len() == 16);
        self.title.set(words.as_bytes(), b"temper", b"en", closing_loses);
        if new_page {
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
        let requested = ops.iter().any(|op| match op {
            skein_ui_web::page::Op::Focus { node } => {
                self.host.tree().nodes().iter().any(|made| made.id == *node && made.focus.is_some())
            }
            skein_ui_web::page::Op::Insert { .. }
            | skein_ui_web::page::Op::Remove { .. }
            | skein_ui_web::page::Op::Move { .. }
            | skein_ui_web::page::Op::Replace { .. }
            | skein_ui_web::page::Op::Attributes { .. }
            | skein_ui_web::page::Op::Text { .. }
            | skein_ui_web::page::Op::Value { .. }
            | skein_ui_web::page::Op::Choice { .. }
            | skein_ui_web::page::Op::Options { .. }
            | skein_ui_web::page::Op::Href { .. }
            | skein_ui_web::page::Op::Reveal { .. }
            | skein_ui_web::page::Op::ScrollEnd { .. }
            | skein_ui_web::page::Op::HoldView { .. }
            | skein_ui_web::page::Op::PutView { .. }
            | skein_ui_web::page::Op::Measure { .. }
            | skein_ui_web::page::Op::ScrollScreen { .. }
            | skein_ui_web::page::Op::Component { .. } => false,
        });
        self.heading_focus.finish(&document, heading, new_page, requested);
        self.history.restore();
        let deadline = self.host.deadline();
        #[cfg(feature = "record")]
        if let Some(recorder) = &mut self.recorder {
            recorder.deadline(deadline);
        }
        self.platform.timer(&mut self.timer, deadline, now);
    }
    #[cfg(feature = "record")]
    fn save_recording(&mut self, departing: bool) {
        if departing {
            if let Some(recorder) = self.recorder.take() {
                save_recording(&recorder.end(&self.page.borrow()));
            }
        } else if let Some(recorder) = &self.recorder {
            save_recording(&recorder.save(&self.page.borrow()));
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// Re-applies the tab policy to the page's forge sign-in links.
fn forge_links(document: &web_sys::Document) {
    // Nodes can be reused by a later page: remove this shell's previous
    // tab policy before applying it to the current forge links.
    for _ in 0..temper_client_view::forge_sign_in::FORGES {
        let Some(link) = document.query_selector("a[data-temper-forge-sign-in]").expect("previous forge link selector")
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
}

#[cfg(target_arch = "wasm32")]
fn forge_sign_in(
    document: &web_sys::Document,
    #[cfg(feature = "record")] raw: &mut Vec<Box<[u8]>>,
) -> temper_client_view::forge_sign_in::Forges {
    let mut facts = temper_client_view::forge_sign_in::Forges::new();
    let mut selector = String::from("meta[name='temper-forge-sign-in']");
    for _ in 0..temper_client_view::forge_sign_in::FORGES {
        let Some(meta) = document.query_selector(&selector).expect("forge meta selector") else { break };
        if let Some(content) = meta.get_attribute("content") {
            facts.read(&content);
            #[cfg(feature = "record")]
            raw.push(content.into_bytes().into());
        }
        selector.push_str(" ~ meta[name='temper-forge-sign-in']");
    }
    let mut selector = String::from("meta[name='temper-forge-sign-in-off']");
    for _ in 0..temper_client_view::forge_sign_in::FORGES {
        let Some(meta) = document.query_selector(&selector).expect("forge off meta selector") else { break };
        if let Some(content) = meta.get_attribute("content") {
            facts.read_off(&content);
            #[cfg(feature = "record")]
            raw.push([b"off ".as_slice(), content.as_bytes()].concat().into());
        }
        selector.push_str(" ~ meta[name='temper-forge-sign-in-off']");
    }
    facts
}

#[cfg(all(target_arch = "wasm32", feature = "record"))]
fn record_keys(document: &web_sys::Document, inbox: &Rc<Inbox<Report>>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let inbox = Rc::clone(inbox);
    let listener = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
        if !event.alt_key() || !event.shift_key() {
            return;
        }
        let report = match event.code().as_str() {
            "KeyM" => Report::Mark,
            "KeyS" => Report::Save,
            _ => return,
        };
        event.prevent_default();
        event.stop_immediate_propagation();
        if !event.repeat() {
            inbox.raise(report);
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    document
        .add_event_listener_with_callback_and_add_event_listener_options(
            "keydown",
            listener.as_ref().unchecked_ref(),
            &options,
        )
        .expect("recording keys listener");
    listener.forget();
}

#[cfg(all(target_arch = "wasm32", feature = "record"))]
fn save_recording(bytes: &[u8]) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/octet-stream");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).expect("recording blob");
    let url = web_sys::Url::create_object_url_with_blob(&blob).expect("recording URL");
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let anchor = document.create_element("a").expect("recording anchor").unchecked_into::<web_sys::HtmlAnchorElement>();
    anchor.set_href(&url);
    anchor.set_download("journey.recording");
    anchor.click();
    // Give the browser a later task to start the download before releasing its URL.
    let revoke = Closure::once_into_js(move || web_sys::Url::revoke_object_url(&url).expect("revoke recording URL"));
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 1_000)
        .expect("recording URL cleanup timer");
}
