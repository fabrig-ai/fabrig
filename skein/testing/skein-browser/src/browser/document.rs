//! The page's reusable document handle (skein/docs/design/testing/browser.md, 6).

use alloc::boxed::Box;
use skein_lib::{Env, Token};

use super::{Browser, Limits, OpKind, Operation, Params, PendingKind};

#[derive(Debug)]
pub(super) enum Root {
    Empty,
    Current(Box<[u8]>),
    Retired(Box<[u8]>),
}

impl Root {
    pub(super) fn current(&self) -> Option<&[u8]> {
        match self {
            Root::Current(object) => Some(object),
            Root::Empty | Root::Retired(_) => None,
        }
    }
}

impl Browser {
    pub(super) fn found_root(&mut self, env: &Env<Limits>, op: Token, object: Box<[u8]>) {
        let Some(Operation { page, kind: OpKind::Find { query, .. }, .. }) = self.op(op) else { return };
        let page = *page;
        let scoped = query.within.is_some();
        self.query_document(env, op, &object);
        let queued = self.op(op).is_some();
        if !scoped && let Some(state) = self.page_mut(page) {
            match &state.document {
                Root::Empty if queued => {
                    state.document = Root::Current(object);
                    return;
                }
                // Two finds can have asked for the same document before either reply.
                Root::Current(current) if current == &object => return,
                Root::Empty | Root::Current(_) | Root::Retired(_) => {}
            }
        }
        if !self.queue_page(env, page, b"Runtime.releaseObject", &Params::Object(&object), PendingKind::Release) {
            self.close_page_local(page);
        }
    }

    pub(super) fn retire_document(&mut self, page: Token) {
        if let Some(page) = self.page_mut(page) {
            let root = core::mem::replace(&mut page.document, Root::Empty);
            if let Root::Current(object) = root {
                page.document = Root::Retired(object);
                self.releasing_documents = true;
            } else {
                page.document = root;
            }
        }
    }

    pub(super) fn release_retired_documents(&mut self, env: &Env<Limits>) {
        self.releasing_documents = false;
        for index in 0..self.pages.len() {
            let Some(Some(page)) = self.pages.get_mut(index) else { continue };
            if let Root::Retired(_) = page.document {
                if self.pending.len() >= usize::try_from(env.limits.commands).expect("u32 fits usize") {
                    // Release is cleanup: wait for a command slot; never lend out a retired handle.
                    self.releasing_documents = true;
                    break;
                }
                let page_token = page.token;
                if let Root::Retired(object) = core::mem::replace(&mut page.document, Root::Empty)
                    && !self.queue_page(
                        env,
                        page_token,
                        b"Runtime.releaseObject",
                        &Params::Object(&object),
                        PendingKind::Release,
                    )
                {
                    self.close_page_local(page_token);
                }
            }
        }
    }

    pub(super) fn release_late_object(&mut self, env: &Env<Limits>, session: Option<&[u8]>, object: &[u8]) {
        if let Some(page) = self.notification_page(session)
            && let Some(page) = self.page(page)
            && page.document.current() == Some(object)
        {
            return;
        }
        self.queue_command(env, b"Runtime.releaseObject", session, &Params::Object(object), PendingKind::Release);
    }
}
