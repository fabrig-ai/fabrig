//! The page's worker and browser timer (ui-web.md, 18; playground.md, 5 to 7).
use super::{Heard, Page, shown};
use crate::{bridge, fetch, inbox::Inbox, log::Log, platform::Platform, store, transport::StreamDuty};
use skein_lib::{Duration, Time, Wall};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// The page's platform for a world in a worker (ui-web.md, 18).
pub struct Worker<R> {
    shared: Rc<Shared<R>>,
}

struct Shared<R> {
    page: RefCell<Page>,
    log: RefCell<Log>,
    worker: RefCell<Option<web_sys::Worker>>,
    handlers: RefCell<Option<Handlers>>,
    ready: Cell<bool>,
    ended: Cell<bool>,
    deadline: Cell<Option<Time>>,
    timeout: Cell<Option<i32>>,
    inbox: Rc<Inbox<R>>,
    fetch: fn(fetch::Report) -> R,
    heard: fn(Heard) -> R,
    due: fn() -> R,
}

struct Handlers {
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
    timeout: Closure<dyn FnMut()>,
}

impl<R: 'static> Worker<R> {
    /// Starts the worker; fetch reports, other messages and timer expiry enter the inbox.
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "the worker's start contract supplies its inbox and wrappers")]
    pub fn start(
        address: &str,
        start: bridge::Start,
        build: Option<&[u8]>,
        log_bound: u32,
        inbox: Rc<Inbox<R>>,
        fetch: fn(fetch::Report) -> R,
        heard: fn(Heard) -> R,
        due: fn() -> R,
    ) -> Worker<R> {
        let log = Log::new(build, &start, log_bound);
        let shared = Rc::new(Shared {
            page: RefCell::new(Page::new(start, crate::platform::clocks().0)),
            log: RefCell::new(log),
            worker: RefCell::new(None),
            handlers: RefCell::new(None),
            ready: Cell::new(false),
            ended: Cell::new(false),
            deadline: Cell::new(None),
            timeout: Cell::new(None),
            inbox,
            fetch,
            heard,
            due,
        });
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        match web_sys::Worker::new_with_options(address, &options) {
            Ok(worker) => shared.install(worker),
            Err(error) => shared.fail(format!("Could not start worker: {error:?}").into_bytes().into()),
        }
        Self { shared }
    }

    pub fn pace(&self, speed: bridge::Speed) {
        if self.shared.ended.get() {
            return;
        }
        let now = crate::platform::clocks().0;
        self.shared.page.borrow_mut().pace(speed, now);
        self.shared.post();
        self.shared.set_timer(now);
    }

    pub fn visibility(&self, hidden: bool) {
        if self.shared.ended.get() {
            return;
        }
        let now = crate::platform::clocks().0;
        self.shared.page.borrow_mut().visibility(hidden, now);
        self.shared.post();
        self.shared.set_timer(now);
    }

    /// Keeps the application's deadline in the world's time, or clears it.
    pub fn timer(&self, deadline: Option<Time>) {
        if self.shared.ended.get() {
            return;
        }
        self.shared.deadline.set(deadline);
        self.shared.set_timer(crate::platform::clocks().0);
    }

    #[must_use]
    pub fn clocks(&self) -> (Time, Wall) {
        self.shared.page.borrow_mut().clocks(crate::platform::clocks().0)
    }

    #[must_use]
    pub fn high_water(&self) -> (u64, Duration) {
        self.shared.page.borrow().high_water()
    }

    /// The session log's text, offered as a file by the frame.
    #[must_use]
    pub fn log(&self) -> Box<[u8]> {
        self.shared.log.borrow().text()
    }

    /// Ends the worker, its handlers and its pending timer.
    pub fn end(&self) {
        self.shared.end();
    }
}

impl<R: 'static> Shared<R> {
    fn install(self: &Rc<Self>, worker: web_sys::Worker) {
        let weak = Rc::downgrade(self);
        let message = Closure::new(move |event: web_sys::MessageEvent| {
            if let Some(shared) = weak.upgrade() {
                shared.message(&event.data());
            }
        });
        let weak = Rc::downgrade(self);
        let error = Closure::new(move |event: web_sys::ErrorEvent| {
            if let Some(shared) = weak.upgrade() {
                shared.fail(event.message().into_bytes().into());
            }
        });
        let weak = Rc::downgrade(self);
        let timeout = Closure::new(move || {
            if let Some(shared) = weak.upgrade()
                && !shared.ended.get()
            {
                shared.timeout.set(None);
                shared.deadline.set(None);
                shared.inbox.raise((shared.due)());
            }
        });
        worker.set_onmessage(Some(message.as_ref().unchecked_ref()));
        worker.set_onerror(Some(error.as_ref().unchecked_ref()));
        self.handlers.replace(Some(Handlers { _message: message, _error: error, timeout }));
        self.worker.replace(Some(worker));
    }

    fn message(&self, data: &JsValue) {
        if self.ended.get() {
            return;
        }
        let bytes = if let Some(buffer) = data.dyn_ref::<js_sys::ArrayBuffer>() {
            js_sys::Uint8Array::new(buffer).to_vec()
        } else if let Some(bytes) = data.dyn_ref::<js_sys::Uint8Array>() {
            bytes.to_vec()
        } else {
            self.malformed();
            return;
        };
        if !self.ready.get() {
            if bytes.as_slice() == bridge::READY {
                self.ready.set(true);
                self.page.borrow_mut().ready();
                self.post();
            } else {
                self.malformed();
            }
            return;
        }
        let Ok(message) = bridge::ToPage::decode(&bytes) else {
            self.malformed();
            return;
        };
        let now = crate::platform::clocks().0;
        self.page.borrow_mut().heard(&message, now);
        self.log.borrow_mut().heard(&message);
        match message {
            bridge::ToPage::Report(report) => self.inbox.raise((self.fetch)(report)),
            bridge::ToPage::Printed(printed) => self.inbox.raise((self.heard)(Heard::Printed(printed))),
            bridge::ToPage::Stopped => self.inbox.raise((self.heard)(Heard::Stopped)),
            bridge::ToPage::Failed { words } => self.fail(words),
            bridge::ToPage::Clock(_) => {
                self.set_timer(now);
                self.inbox.raise((self.heard)(Heard::Clock));
            }
            bridge::ToPage::Stamp(_) => {}
        }
    }

    fn malformed(&self) {
        self.fail(b"Broken worker bridge contract: malformed message or missing ready message".as_slice().into());
    }

    fn fail(&self, words: Box<[u8]>) {
        if self.ended.get() {
            return;
        }
        self.end();
        self.inbox.raise((self.heard)(Heard::Failed { words }));
    }

    fn send(&self, input: bridge::Input) {
        if self.ended.get() {
            return;
        }
        self.log.borrow_mut().sent(&input);
        self.page.borrow_mut().send(input);
        self.post();
    }

    fn post(&self) {
        if self.ended.get() {
            return;
        }
        let mut outgoing = Vec::new();
        self.page.borrow_mut().outgoing(&mut outgoing);
        for message in outgoing {
            let bytes = message.encode();
            // Copy into a JavaScript allocation so transferring cannot detach wasm memory.
            let buffer = js_sys::Uint8Array::from(bytes.as_ref()).buffer();
            let transfer = js_sys::Array::new();
            transfer.push(&buffer);
            let result = self
                .worker
                .borrow()
                .as_ref()
                .expect("a live worker has its browser worker")
                .post_message_with_transfer(&buffer, &transfer);
            if let Err(error) = result {
                self.fail(format!("Could not post worker message: {error:?}").into_bytes().into());
                return;
            }
        }
    }

    fn set_timer(&self, now: Time) {
        self.clear_timer();
        if self.ended.get() {
            return;
        }
        let Some(deadline) = self.deadline.get() else { return };
        let Some(at) = self.page.borrow().timer(deadline, now) else { return };
        let millis = at.saturating_since(now).as_nanos().div_ceil(1_000_000);
        let handlers = self.handlers.borrow();
        let callback = &handlers.as_ref().expect("a live worker has its handlers").timeout;
        let handle = web_sys::window()
            .expect("browser window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                i32::try_from(millis).unwrap_or(i32::MAX),
            )
            .expect("worker platform timeout");
        self.timeout.set(Some(handle));
    }
}

impl<R> Shared<R> {
    fn clear_timer(&self) {
        if let Some(handle) = self.timeout.take() {
            web_sys::window().expect("browser window").clear_timeout_with_handle(handle);
        }
    }

    fn end(&self) {
        if self.ended.replace(true) {
            return;
        }
        self.clear_timer();
        self.deadline.set(None);
        if let Some(worker) = self.worker.borrow_mut().take() {
            worker.set_onmessage(None);
            worker.set_onerror(None);
            worker.terminate();
        }
        self.handlers.borrow_mut().take();
    }
}

impl<R> Drop for Worker<R> {
    fn drop(&mut self) {
        self.shared.end();
    }
}

impl<R: 'static> Platform for Worker<R> {
    fn read_all(&self) -> Vec<store::Record> {
        self.shared.page.borrow().read_all()
    }

    fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        self.shared.page.borrow_mut().write(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        self.shared.page.borrow_mut().remove(key);
    }

    fn send(&mut self, exchange: &fetch::Exchange) {
        self.shared.send(bridge::Input::Exchange(exchange.clone()));
    }

    fn navigate(&mut self, to: &[u8]) {
        if !shown(to) || self.shared.ended.get() {
            return;
        }
        let to = std::str::from_utf8(to).expect("visible ASCII navigation");
        web_sys::window()
            .expect("browser window")
            .history()
            .expect("browser history")
            .replace_state_with_url(&JsValue::NULL, "", Some(to))
            .expect("navigation path on the page's origin");
    }

    fn stream(&mut self, duty: StreamDuty) {
        self.shared.send(bridge::Input::Stream(duty));
    }

    fn abandon(&mut self, exchange: u32) {
        self.shared.send(bridge::Input::Abandon { exchange });
    }

    fn read_on(&mut self) {
        self.shared.send(bridge::Input::ReadOn);
    }
}

/// The worker starter beside the page's module starter (playground.md, 7).
#[must_use]
pub fn worker_address() -> Option<String> {
    let document = web_sys::window()?.document()?;
    let scripts = document.scripts();
    (0..scripts.length()).find_map(|index| {
        let script = scripts.item(index)?.dyn_into::<web_sys::HtmlScriptElement>().ok()?;
        if script.type_() != "module" {
            return None;
        }
        let src = script.src();
        let directory = src.strip_suffix("/boot.js")?;
        Some(format!("{directory}/boot-worker.js"))
    })
}
