//! Full screen and Escape held by the browser (skein/docs/design/ui-web.md, section 11).
use super::{Report, changed};
use crate::inbox::Inbox;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

pub struct Screen<R> {
    shared: Rc<Shared<R>>,
}
struct Shared<R> {
    inbox: Rc<Inbox<R>>,
    wrap: fn(Report) -> R,
    document: web_sys::Document,
    full: Cell<bool>,
}
impl<R: 'static> Screen<R> {
    /// Listens for the document's `fullscreenchange`, once, for the page's life.
    #[must_use]
    pub fn new(inbox: Rc<Inbox<R>>, wrap: fn(Report) -> R) -> Self {
        let document = web_sys::window().expect("browser window").document().expect("browser document");
        let shared = Rc::new(Shared { inbox, wrap, document, full: Cell::new(false) });
        let target = Rc::clone(&shared);
        let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            let now = target.document.fullscreen_element().is_some();
            if let Some(report) = changed(target.full.replace(now), now) {
                match report {
                    Report::Entered => lock_escape(),
                    Report::Left => unlock(),
                    Report::Refused => {}
                }
                target.inbox.raise((target.wrap)(report));
            }
        });
        shared
            .document
            .add_event_listener_with_callback("fullscreenchange", listener.as_ref().unchecked_ref())
            .expect("full-screen listener");
        // The document holds this listener and its state for the page's life.
        let _listener = listener.into_js_value();
        Self { shared }
    }

    /// Asked in the turn of a person's act: `requestFullscreen`, then, once in, Escape locked.
    pub fn enter(&self) {
        match request(&self.shared.document) {
            Ok(promise) => {
                let shared = Rc::clone(&self.shared);
                let reject = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                    shared.inbox.raise((shared.wrap)(Report::Refused));
                });
                let _next = promise.catch(&reject);
                let _reject = reject.into_js_value();
            }
            Err(()) => self.shared.inbox.raise((self.shared.wrap)(Report::Refused)),
        }
    }

    /// Leaves full screen where the page is in it; nothing otherwise.
    pub fn leave(&self) {
        if self.shared.document.fullscreen_element().is_some() {
            self.shared.document.exit_fullscreen();
        }
    }
}

fn request(document: &web_sys::Document) -> Result<js_sys::Promise, ()> {
    if !document.fullscreen_enabled() {
        return Err(());
    }
    let element = document.document_element().ok_or(())?;
    method(element.as_ref(), "requestFullscreen")
        .ok_or(())?
        .call0(element.as_ref())
        .map_err(|_| ())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| ())
}
fn method(target: &JsValue, name: &str) -> Option<js_sys::Function> {
    js_sys::Reflect::get(target, &JsValue::from_str(name)).ok()?.dyn_into().ok()
}
fn keyboard() -> Option<JsValue> {
    js_sys::Reflect::get(web_sys::window()?.navigator().as_ref(), &JsValue::from_str("keyboard")).ok()
}
fn lock_escape() {
    let Some(keyboard) = keyboard() else { return };
    let Some(lock) = method(&keyboard, "lock") else { return };
    let keys = js_sys::Array::new();
    keys.push(&JsValue::from_str("Escape"));
    if let Ok(result) = lock.call1(&keyboard, keys.as_ref())
        && let Ok(promise) = result.dyn_into::<js_sys::Promise>()
    {
        let ignore = Closure::<dyn FnMut(JsValue)>::new(|_| {});
        let _next = promise.catch(&ignore);
        let _ignore = ignore.into_js_value();
    }
}
fn unlock() {
    if let Some(keyboard) = keyboard()
        && let Some(unlock) = method(&keyboard, "unlock")
    {
        let _result = unlock.call0(&keyboard);
    }
}
