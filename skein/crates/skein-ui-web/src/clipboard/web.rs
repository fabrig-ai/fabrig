//! The browser's clipboard promise (skein/docs/design/ui-web.md, sections 5.1 and 11).
use super::{Outcome, Report, rejected};
use crate::inbox::Inbox;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

pub struct Clipboard<R> {
    inbox: Rc<Inbox<R>>,
    wrap: fn(Report) -> R,
}

impl<R: 'static> Clipboard<R> {
    #[must_use]
    pub fn new(inbox: Rc<Inbox<R>>, wrap: fn(Report) -> R) -> Self {
        Self { inbox, wrap }
    }

    /// Asked in the turn of a person's act; one report through the inbox, at once when refused.
    pub fn write(&self, token: u32, text: &[u8]) {
        match write(text) {
            Ok(promise) => {
                let (inbox, failed) = (Rc::clone(&self.inbox), Rc::clone(&self.inbox));
                let wrap = self.wrap;
                let resolve = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                    inbox.raise(wrap(Report { token, outcome: Outcome::Done }));
                });
                let reject = Closure::<dyn FnMut(JsValue)>::new(move |error| {
                    failed.raise(wrap(Report { token, outcome: rejection(&error) }));
                });
                let _next = promise.then2(&resolve, &reject);
                // The promise holds the functions; browser GC frees both captures after settlement.
                let _resolve = resolve.into_js_value();
                let _reject = reject.into_js_value();
            }
            Err(outcome) => self.inbox.raise((self.wrap)(Report { token, outcome })),
        }
    }
}

fn write(text: &[u8]) -> Result<js_sys::Promise, Outcome> {
    let text = std::str::from_utf8(text).map_err(|_| Outcome::Refused)?;
    let window = web_sys::window().ok_or(Outcome::Refused)?;
    let navigator = window.navigator();
    let clipboard =
        js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("clipboard")).map_err(|error| rejection(&error))?;
    if clipboard.is_null() || clipboard.is_undefined() {
        return Err(Outcome::Refused);
    }
    let method = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText"))
        .map_err(|error| rejection(&error))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| Outcome::Failed)?;
    method
        .call1(&clipboard, &JsValue::from_str(text))
        .map_err(|error| rejection(&error))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| Outcome::Failed)
}

fn rejection(error: &JsValue) -> Outcome {
    let name = js_sys::Reflect::get(error, &JsValue::from_str("name")).ok().and_then(|name| name.as_string());
    rejected(name.as_deref())
}
