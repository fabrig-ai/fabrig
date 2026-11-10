//! A tab opened, pointed with its opener cut, and let go (skein/docs/design/ui/web/README.md, section 8).

#[derive(Default)]
pub struct Tab {
    held: Option<web_sys::Window>,
}

impl Tab {
    pub fn open(&mut self, words: &[u8]) -> bool {
        self.close();
        let Some(window) = web_sys::window() else { return false };
        // No noopener: the page needs the handle until it cuts the opener before pointing.
        // An empty address keeps the first document synchronously reachable on Firefox too.
        let Ok(Some(tab)) = window.open_with_url_and_target("", "_blank") else { return false };
        if let (Ok(words), Some(document)) = (std::str::from_utf8(words), tab.document()) {
            document.set_title(words);
            if let Some(body) = document.body() {
                body.set_text_content(Some(words));
            }
        }
        self.held = Some(tab);
        true
    }

    pub fn point(&mut self, to: &[u8], loopback: bool) {
        let Some(tab) = self.held.take() else { return };
        if !crate::navigation::allowed(to, loopback) {
            let _closed = tab.close();
            return;
        }
        let Ok(to) = std::str::from_utf8(to) else {
            let _closed = tab.close();
            return;
        };
        if tab.set_opener(&wasm_bindgen::JsValue::NULL).is_err() {
            let _closed = tab.close();
            return;
        }
        if tab.location().replace(to).is_err() {
            let _closed = tab.close();
        }
    }

    pub fn close(&mut self) {
        if let Some(tab) = self.held.take() {
            let _closed = tab.close();
        }
    }
}
