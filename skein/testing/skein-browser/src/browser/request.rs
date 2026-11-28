//! Opt-in main-frame requests and their first answers (skein/docs/design/testing/browser-vocabulary.md, 1, and skein/docs/design/testing/browser-operations.md, 1, and skein/docs/design/testing/browser.md, 7–8).
use super::{Browser, cut_text, field_text, frame_id};
use crate::boundary::{Event, Trouble};
use crate::limits::Limits;
use crate::wire::decode::Value;
use alloc::{boxed::Box, vec::Vec};
use skein_lib::{Env, Token};

#[derive(Debug)]
pub(super) struct Waiting {
    pub(super) page: Token,
    id: Box<[u8]>,
}

impl Browser {
    pub(super) fn page_requests(&self, page: Token) -> bool {
        match self.page(page) {
            Some(page) => page.requests,
            None => false,
        }
    }

    pub(super) fn request_frame(&mut self, env: &Env<Limits>, page: Token, result: Value<'_>) {
        let frame = match result.get(b"frameTree") {
            Some(tree) => tree.get(b"frame"),
            None => None,
        };
        let id = match frame {
            Some(frame) => frame_id(frame, b"id", env.limits.text),
            None => None,
        };
        if let Some(id) = id {
            if let Some(state) = self.page_mut(page) {
                state.frame = Some(id);
            }
            self.enable(env, page, Self::ENABLE_FILES);
        } else {
            self.close_page_local(page);
        }
    }

    pub(super) fn request_sent(&mut self, env: &Env<Limits>, page: Token, params: Value<'_>) {
        if !self.page_requests(page) || params.get(b"redirectResponse").is_some() {
            return;
        }
        let Some(state) = self.page(page) else { return };
        let frame = frame_id(params, b"frameId", env.limits.text);
        if frame.is_none() || state.frame != frame {
            return;
        }
        let (Some(id), Some(load), Some(kind), Some(request)) = (
            field_text(params, b"requestId"),
            field_text(params, b"loaderId"),
            field_text(params, b"type"),
            params.get(b"request"),
        ) else {
            return;
        };
        let (Some(method), Some(url)) = (field_text(request, b"method"), field_text(request, b"url")) else { return };
        let id = cut_text(id, env.limits.text);
        let url = cut_text(url, env.limits.text);
        let kind = cut_text(kind, env.limits.text);
        self.events.push_back(Event::Sent {
            page,
            id: id.clone(),
            load: cut_text(load, env.limits.text),
            document: kind.as_ref() == b"Document",
            method: cut_text(method, env.limits.text),
            url: url.clone(),
        });
        let limit = usize::try_from(env.limits.requests).expect("u32 fits usize");
        if self.requests.len() >= limit {
            self.trouble(
                page,
                Trouble::Request,
                Box::from(b"requests past the limit".as_slice()),
                url,
                env.limits.text,
            );
            return;
        }
        if self.requests.capacity() == 0 {
            self.requests = Vec::with_capacity(limit);
        }
        self.requests.push(Waiting { page, id });
    }

    pub(super) fn request_answered(&mut self, env: &Env<Limits>, page: Token, params: Value<'_>) {
        if !self.page_requests(page) {
            return;
        }
        let Some(id) = field_text(params, b"requestId") else { return };
        let id = cut_text(id, env.limits.text);
        let mut found = None;
        for (at, request) in self.requests.iter().enumerate() {
            if request.page == page && request.id == id {
                found = Some(at);
                break;
            }
        }
        if let Some(at) = found {
            let request = self.requests.swap_remove(at);
            self.events.push_back(Event::Answered { page, id: request.id });
        }
    }
}
