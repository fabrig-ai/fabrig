//! Navigation's heading, including one arriving after its main (skein/docs/design/ui-web.md, 7).
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};

pub(crate) struct HeadingFocus {
    pending: Rc<RefCell<Option<web_sys::Element>>>,
    moved: Closure<dyn FnMut(web_sys::Event)>,
}

impl HeadingFocus {
    pub(crate) fn new() -> Self {
        let pending = Rc::new(RefCell::new(None));
        let waiting = Rc::clone(&pending);
        let moved = Closure::new(move |_: web_sys::Event| {
            // A focus placed after navigation owns its place, even if it returns to main.
            waiting.borrow_mut().take();
        });
        web_sys::window()
            .expect("window")
            .document()
            .expect("document")
            .add_event_listener_with_callback("focusin", moved.as_ref().unchecked_ref())
            .expect("navigation focus listener");
        Self { pending, moved }
    }

    pub(crate) fn finish(
        &mut self,
        document: &web_sys::Document,
        heading: Option<web_sys::Element>,
        new_page: bool,
        requested: bool,
    ) {
        let pending = self.pending.borrow_mut().take();
        if requested {
            return;
        }
        if new_page {
            let waiting = heading.is_none();
            if let Some(target) = heading.or_else(|| document.query_selector("main").expect("main selector")) {
                focus(&target);
                if waiting {
                    *self.pending.borrow_mut() = Some(target);
                }
            }
        } else if let Some(main) = pending
            && document.active_element().as_ref() == Some(&main)
        {
            if let Some(heading) = heading {
                focus(&heading);
            } else {
                *self.pending.borrow_mut() = Some(main);
            }
        }
    }
}

impl Drop for HeadingFocus {
    fn drop(&mut self) {
        web_sys::window()
            .expect("window")
            .document()
            .expect("document")
            .remove_event_listener_with_callback("focusin", self.moved.as_ref().unchecked_ref())
            .expect("remove navigation focus listener");
    }
}

fn focus(target: &web_sys::Element) {
    target.set_attribute("tabindex", "-1").expect("focus target");
    target.unchecked_ref::<web_sys::HtmlElement>().focus().expect("focus mounted target");
}
