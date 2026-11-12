//! The page's life, frame and fail-stop hook (temper/docs/design/playground.md, 6;
//! skein/docs/design/playground.md, 7).
use super::{Control, STORIES, control, memory_words, printed_link, slice_words, speed_words, time_words};
use jig_client_shell_web::sign_in;
use skein_lib::{Time, Wall};
use skein_ui_web::{
    bridge::{self, Speed},
    history,
    inbox::Inbox,
    page::{Page, web::Dom},
    platform::{
        self,
        worker::{FailStop, Heard, Worker, worker_address},
    },
    store::Store,
};
use std::{cell::RefCell, rc::Rc};
use temper_client_shell_web::{Report, Shell, ShellPlatform, host, limits};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

// Four MiB bounds the page's kept log; past this the kit writes a replayable cut.
const LOG_BYTES: u32 = 4 * 1024 * 1024;

thread_local! {
    // Independent of the shell and the inbox: the panic may hold either one's borrow.
    static FAIL_STOP: RefCell<Option<FailStop<Report>>> = const { RefCell::new(None) };
}

/// The page owns the worker adapter; the shell knows only its platform seam.
struct World(Worker<Report>);

impl platform::Platform for World {
    fn read_all(&self) -> Vec<skein_ui_web::store::Record> {
        self.0.read_all()
    }
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        self.0.write(key, value)
    }
    fn remove(&mut self, key: &[u8]) {
        self.0.remove(key);
    }
    fn send(&mut self, exchange: &skein_ui_web::fetch::Exchange) {
        self.0.send(exchange);
    }
    fn navigate(&mut self, to: &[u8]) {
        self.0.navigate(to);
    }
    fn stream(&mut self, duty: skein_ui_web::transport::StreamDuty) {
        self.0.stream(duty);
    }
    fn abandon(&mut self, exchange: u32) {
        self.0.abandon(exchange);
    }
    fn read_on(&mut self) {
        self.0.read_on();
    }
}

impl ShellPlatform for World {
    type Timer = ();
    fn clocks(&self) -> (Time, Wall) {
        self.0.clocks()
    }
    fn fired((): &mut ()) {}
    fn end(&self, (): &mut ()) {
        self.0.end();
    }
    fn timer(&self, (): &mut (), deadline: Option<Time>, _now: Time) {
        self.0.timer(deadline);
    }
}

/// Starts the frame first; the client waits for the world's printed sign-in link.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| {
        let log = FAIL_STOP.with(|slot| {
            let slot = slot.try_borrow().ok()?;
            let stop = slot.as_ref()?;
            stop.stop();
            stop.log()
        });
        failure(&info.to_string(), log.as_deref(), true);
    }));
    let document = document();
    element(super::FRAME).remove_attribute("hidden").expect("show frame");
    element(super::VERSION).set_text_content(Some(concat!("temper ", env!("CARGO_PKG_VERSION"))));
    let select = element(super::STORY).unchecked_into::<web_sys::HtmlSelectElement>();
    for (name, _) in STORIES {
        let option = document.create_element("option").expect("story option");
        option.set_attribute("value", name).expect("story name");
        option.set_text_content(Some(&format!("Story {}", name.to_uppercase())));
        select.append_child(&option).expect("offered story");
    }
    let inbox = Inbox::new();
    controls(&inbox);
    let history = Rc::new(history::web::History::new(Rc::clone(&inbox), Report::Place));
    let mut life = Life {
        worker: None,
        shell: None,
        history,
        inbox: Rc::downgrade(&inbox),
        facts: platform::facts(),
        story: "a",
        speed: Speed::ONE,
        failed: false,
    };
    life.reset(&inbox);
    inbox.start(Box::new(move |inbox| life.turn(inbox)));
}

struct Life {
    worker: Option<Worker<Report>>,
    shell: Option<Shell<World>>,
    // The history listeners and inbox live for the page; each client DOM and worker ends on reset.
    history: Rc<history::web::History>,
    inbox: std::rc::Weak<Inbox<Report>>,
    facts: platform::Facts,
    story: &'static str,
    speed: Speed,
    failed: bool,
}

impl Life {
    fn inbox(&self) -> Rc<Inbox<Report>> {
        self.inbox.upgrade().expect("the page's inbox lives with its listeners")
    }

    fn worker(&self) -> Option<&Worker<Report>> {
        self.shell.as_ref().map(|shell| &shell.platform().0).or(self.worker.as_ref())
    }

    fn end(&mut self) {
        if let Some(shell) = &mut self.shell {
            shell.end();
        }
        if let Some(worker) = &self.worker {
            worker.end();
        }
        web_sys::window().expect("window").set_onbeforeunload(None);
    }

    fn reset(&mut self, inbox: &Rc<Inbox<Report>>) {
        self.end();
        self.shell = None;
        self.worker = None;
        // All queued reports belong to the ended life; its worker can raise no more.
        while inbox.next().is_some() {}
        FAIL_STOP.with(|slot| slot.replace(None));
        element(super::APP).set_text_content(None);
        hidden(super::APP, true);
        hidden(super::LOADING, false);
        hidden(super::FAILURE, true);
        clear_log_link();
        disabled(false);
        self.failed = false;
        self.speed = Speed::ONE;
        self.facts = platform::facts();
        element(super::STORY).unchecked_into::<web_sys::HtmlSelectElement>().set_value(self.story);
        let description = STORIES.iter().find(|(name, _)| *name == self.story).expect("offered story").1;
        element(super::DESCRIPTION).set_text_content(Some(description));
        let Some(address) = worker_address() else {
            self.failed = true;
            failure("Could not find the playground worker beside boot.js.", None, false);
            return;
        };
        let directory = address.strip_suffix("/boot-worker.js").expect("worker starter address");
        let build = directory.rsplit('/').next().expect("bundle digest directory");
        let worker = Worker::start(
            &address,
            bridge::Start { story: self.story.as_bytes().into(), seed: self.facts.seed, start: self.facts.wall },
            Some(build.as_bytes()),
            LOG_BYTES,
            Rc::clone(inbox),
            Report::Fetch,
            Report::World,
            || Report::Due,
        );
        FAIL_STOP.with(|slot| slot.replace(Some(worker.fail_stop())));
        worker.visibility(document().hidden());
        self.worker = Some(worker);
        self.figures();
    }

    fn client(&mut self, path: &str, inbox: &Rc<Inbox<Report>>) {
        let window = web_sys::window().expect("window");
        window
            .history()
            .expect("history")
            .replace_state_with_url(&JsValue::NULL, "", Some(path))
            .expect("printed link on this page");
        // As in the shipped starter, erase the secret before the host can send.
        let secret = sign_in::web::take_from_page(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/welcome" });
        let path = window.location().pathname().expect("path") + &window.location().search().expect("query");
        let platform = self.worker.take().expect("the waiting world's worker");
        let (now, wall) = platform.clocks();
        let store = Store::new(b"temper.", None);
        let mut host = host::Host::new(
            limits::client(),
            self.facts.seed,
            now,
            wall,
            self.facts.offset,
            temper_client_view::sign_in::Ways::NONE,
        );
        host.start(path.as_bytes(), store.read_all(&platform), secret);
        let page: Rc<RefCell<Page<'static>>> = Rc::new(RefCell::new(host::page(&host.bounds.tree, self.facts.primary)));
        let dom = Dom::new(element(super::APP), &page, inbox, Report::Act, &[]);
        self.shell = Some(Shell::new(host, page, dom, World(platform), store, Rc::clone(&self.history), (), inbox));
        hidden(super::LOADING, true);
        hidden(super::APP, false);
        inbox.raise(Report::Start);
    }

    fn turn(&mut self, inbox: &Inbox<Report>) {
        if let Some(worker) = self.worker() {
            worker.visibility(document().hidden());
        }
        let mut now = self.shell.as_mut().map(Shell::begin);
        let mut new_page = false;
        while let Some(report) = inbox.next() {
            match report {
                Report::Frame(control) => {
                    self.control(control);
                    now = self.shell.as_mut().map(Shell::begin);
                    new_page = false;
                }
                Report::World(Heard::Printed(printed)) => {
                    if !self.failed
                        && self.shell.is_none()
                        && let Some(path) = printed_link(&printed)
                    {
                        self.client(path, &self.inbox());
                        now = self.shell.as_mut().map(Shell::begin);
                    }
                }
                Report::World(Heard::Failed { words }) => {
                    self.end();
                    self.failed = true;
                    let log = self.worker().map(Worker::log);
                    failure(&String::from_utf8_lossy(&words), log.as_deref(), false);
                }
                Report::World(Heard::Stopped) => {
                    self.end();
                    self.failed = true;
                    let log = self.worker().map(Worker::log);
                    failure("The playground world stopped.", log.as_deref(), false);
                }
                Report::World(Heard::Clock) => {}
                report @ (Report::Start
                | Report::Due
                | Report::Act(..)
                | Report::Fetch(_)
                | Report::TabRefused
                | Report::Copied(_)
                | Report::Screen(_)
                | Report::Carried(..)
                | Report::Place(_)) => {
                    if !self.failed
                        && let Some(shell) = &mut self.shell
                        && let Some(shown) = shell.report(inbox, report)
                    {
                        new_page = shown;
                    }
                }
                report @ (Report::Mark | Report::Save) => {
                    if !self.failed
                        && let Some(shell) = &mut self.shell
                    {
                        shell.report(inbox, report);
                    }
                }
            }
        }
        if !self.failed
            && let Some(shell) = &mut self.shell
        {
            let now = now.unwrap_or_else(|| shell.begin());
            shell.draw(new_page, now);
        }
        self.figures();
    }

    fn control(&mut self, control: Control) {
        match control {
            Control::Reset => self.reset(&self.inbox()),
            Control::Story(story) => {
                self.story = story;
                self.reset(&self.inbox());
            }
            Control::Pause => self.pace(Speed::PAUSED),
            Control::Speed(speed) => self.pace(speed),
            Control::Log => {
                if let Some(worker) = self.worker() {
                    download(&worker.log());
                }
            }
        }
    }

    fn pace(&mut self, speed: Speed) {
        if !self.failed
            && let Some(worker) = self.worker()
        {
            worker.pace(speed);
            self.speed = speed;
        }
    }

    fn figures(&self) {
        if let Some(worker) = self.worker() {
            let (now, _) = worker.clocks();
            let (memory, slice) = worker.high_water();
            element(super::TIME).set_text_content(Some(&time_words(now)));
            element(super::MEMORY).set_text_content(Some(&memory_words(memory)));
            element(super::SLICE).set_text_content(Some(&slice_words(slice)));
        }
        element(super::PACE).set_text_content(Some(if document().hidden() {
            "Paused (tab hidden)"
        } else {
            speed_words(self.speed)
        }));
        for (id, speed) in [
            (super::PAUSE, Speed::PAUSED),
            (super::HALF, Speed(500)),
            (super::ONE, Speed::ONE),
            (super::TEN, Speed(10_000)),
        ] {
            element(id)
                .set_attribute("aria-pressed", if self.speed == speed { "true" } else { "false" })
                .expect("selected pace");
        }
    }
}

fn document() -> web_sys::Document {
    web_sys::window().expect("window").document().expect("document")
}

fn element(id: &str) -> web_sys::Element {
    document().get_element_by_id(id).expect("playground page element")
}

fn hidden(id: &str, hide: bool) {
    if hide {
        element(id).set_attribute("hidden", "").expect("hide playground section");
    } else {
        element(id).remove_attribute("hidden").expect("show playground section");
    }
}

fn disabled(disable: bool) {
    for id in [super::PAUSE, super::HALF, super::ONE, super::TEN, super::STORY] {
        if disable {
            element(id).set_attribute("disabled", "").expect("disable world control");
        } else {
            element(id).remove_attribute("disabled").expect("enable world control");
        }
    }
}

fn controls(inbox: &Rc<Inbox<Report>>) {
    for id in
        [super::PAUSE, super::HALF, super::ONE, super::TEN, super::RESET, super::LOG, super::START_OVER, super::STORY]
    {
        let reports = Rc::clone(inbox);
        let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            event.prevent_default();
            let story = element(super::STORY).unchecked_into::<web_sys::HtmlSelectElement>().value();
            if let Some(control) = control(id, &story) {
                reports.raise(Report::Frame(control));
            }
        });
        element(id)
            .add_event_listener_with_callback(
                if id == super::STORY { "change" } else { "click" },
                listener.as_ref().unchecked_ref(),
            )
            .expect("frame listener");
        listener.forget();
    }
    let reports = Rc::clone(inbox);
    let listener = Closure::<dyn FnMut()>::new(move || reports.raise(Report::World(Heard::Clock)));
    document()
        .add_event_listener_with_callback("visibilitychange", listener.as_ref().unchecked_ref())
        .expect("visibility listener");
    listener.forget();
}

fn log_url(bytes: &[u8]) -> String {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/octet-stream");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).expect("session log blob");
    web_sys::Url::create_object_url_with_blob(&blob).expect("session log URL")
}

fn download(bytes: &[u8]) {
    let url = log_url(bytes);
    let anchor =
        document().create_element("a").expect("session log anchor").unchecked_into::<web_sys::HtmlAnchorElement>();
    anchor.set_href(&url);
    anchor.set_download("temper-playground.log");
    anchor.click();
    // Let the browser start the download before releasing the snapshot's URL.
    let revoke = Closure::once_into_js(move || web_sys::Url::revoke_object_url(&url).expect("revoke log URL"));
    web_sys::window()
        .expect("window")
        .set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 1000)
        .expect("log URL cleanup timer");
}

fn clear_log_link() {
    let target = element(super::FAILURE_LOG);
    if let Some(anchor) = target.first_element_child()
        && let Some(url) = anchor.get_attribute("href")
    {
        web_sys::Url::revoke_object_url(&url).expect("revoke failure log URL");
    }
    target.set_text_content(None);
}

/// Uses only the DOM and the snapshot: safe to display while the loop's cells are borrowed.
fn failure(words: &str, log: Option<&[u8]>, trapped: bool) {
    hidden(super::APP, true);
    hidden(super::LOADING, true);
    hidden(super::FAILURE, false);
    web_sys::window().expect("window").set_onbeforeunload(None);
    element(super::FAILURE_WORDS).set_text_content(Some(words));
    disabled(true);
    clear_log_link();
    if let Some(bytes) = log {
        let anchor = document().create_element("a").expect("failure log anchor");
        anchor.set_attribute("href", &log_url(bytes)).expect("failure log URL");
        anchor.set_attribute("download", "temper-playground.log").expect("failure log file name");
        anchor.set_text_content(Some("Download the session log"));
        element(super::FAILURE_LOG).append_child(&anchor).expect("failure log download");
    } else {
        element(super::FAILURE_LOG).set_text_content(Some("The session log could not be taken."));
    }
    if trapped {
        // No callback into a trapped module: a visitor's ordinary link reloads the page.
        let restart = element(super::START_OVER);
        let anchor = document().create_element("a").expect("start over link");
        // A fragment would make a same-document link; the bare path starts a new page.
        let path = web_sys::window().expect("window").location().pathname().expect("page path");
        anchor.set_attribute("href", &path).expect("start over address");
        anchor.set_text_content(Some("Start over"));
        restart
            .parent_node()
            .expect("start over parent")
            .replace_child(&anchor, &restart)
            .expect("plain start over link");
        hidden(super::FRAME, true);
    }
    element(super::FAILURE).unchecked_into::<web_sys::HtmlElement>().focus().expect("failure focus");
}
