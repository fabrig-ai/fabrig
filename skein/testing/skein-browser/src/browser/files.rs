//! File-only pages intercept requests before dispatch (skein/docs/design/testing/browser.md, 5–6).

use super::{Browser, PendingKind, Phase, field_text, opt_field};
use crate::{Limits, params::Params, wire::decode::Value};
use skein_lib::{Env, Token};

impl Browser {
    pub(super) fn page_files(&self, page: Token) -> bool {
        match self.page(page) {
            Some(page) => page.files,
            None => false,
        }
    }

    pub(super) fn request_paused(&mut self, env: &Env<Limits>, page: Token, params: Value<'_>) {
        if !self.page_files(page) {
            return;
        }
        let Some(id) = field_text(params, b"requestId") else {
            self.interception_failed(page);
            return;
        };
        let allow = match opt_field(params.get(b"request"), b"url") {
            Some(url) => match url.decoded_text() {
                Some(url) => url.bytes().starts_with(b"file://"),
                None => false,
            },
            None => false,
        };
        let method = if allow { b"Fetch.continueRequest".as_slice() } else { b"Fetch.failRequest" };
        if !self.queue_page(env, page, method, &Params::FetchAnswer { id: &id, allow }, PendingKind::Paused(page)) {
            self.interception_failed(page);
        }
    }

    pub(super) fn interception_failed(&mut self, page: Token) {
        if self.phase == Phase::Ready && self.page_files(page) {
            // Never disable interception to release a request we could not answer.
            // End the browser instead so the owner tears down the child.
            self.crash();
        }
    }
}
