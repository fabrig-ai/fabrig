//! Browser-level downloads belong to a person after their page closes.
use super::{Browser, Download, PendingKind, field_text};
use crate::boundary::{Event, Trouble};
use crate::limits::Limits;
use crate::params::Params;
use crate::wire::decode::Value;
use alloc::vec::Vec;
use skein_lib::{Env, Token};

impl Browser {
    fn cancel_download(&mut self, env: &Env<Limits>, person: Token, guid: &[u8], kind: PendingKind) -> bool {
        let context = match self.person(person) {
            Some(state) => state.context.clone(),
            None => None,
        };
        if let Some(context) = context {
            self.queue_command(
                env,
                b"Browser.cancelDownload",
                None,
                &Params::CancelDownload { context: &context, guid },
                kind,
            )
        } else {
            false
        }
    }

    pub(super) fn close_browser_downloads(&mut self, env: &Env<Limits>) {
        if let Some(download) = self.downloads.pop() {
            if !self.cancel_download(env, download.person, &download.guid, PendingKind::CancelBrowser) {
                self.crash();
            }
        } else if !self.queue_command(env, b"Browser.close", None, &Params::Empty, PendingKind::CloseBrowser) {
            self.crash();
        }
    }

    pub(super) fn close_person_downloads(&mut self, env: &Env<Limits>, person: Token) {
        let mut index = None;
        for (at, download) in self.downloads.iter().enumerate() {
            if download.person == person {
                index = Some(at);
                break;
            }
        }
        if let Some(index) = index {
            let download = self.downloads.remove(index);
            if !self.cancel_download(env, person, &download.guid, PendingKind::CancelPerson(person)) {
                self.crash();
            }
            return;
        }
        let context = match self.person(person) {
            Some(state) => state.context.clone(),
            None => None,
        };
        if let Some(context) = context {
            if !self.queue_command(
                env,
                b"Target.disposeBrowserContext",
                None,
                &Params::Context(&context),
                PendingKind::ClosePerson(person),
            ) {
                self.crash();
            }
        } else {
            self.close_person_local(person);
        }
    }

    pub(super) fn download_begin(&mut self, env: &Env<Limits>, params: Value<'_>) {
        let (Some(frame), Some(guid), Some(name)) =
            (field_text(params, b"frameId"), field_text(params, b"guid"), field_text(params, b"suggestedFilename"))
        else {
            return;
        };
        let mut owner = None;
        for page in self.pages.iter().flatten() {
            if !page.closed && page.target.as_deref() == Some(frame.as_ref()) {
                owner = Some((page.token, page.person));
                break;
            }
        }
        let Some((page, person)) = owner else { return };
        let Some(state) = self.person(person) else { return };
        if state.closing {
            return;
        }
        for download in &self.downloads {
            if download.guid == guid {
                return;
            }
        }
        let limit = usize::try_from(env.limits.text).expect("u32 fits usize");
        if state.downloads.is_none()
            || name.len() > limit
            || guid.len() > limit
            || self.downloads.len() >= usize::try_from(env.limits.downloads).expect("u32 fits usize")
        {
            self.trouble(page, Trouble::Download, name, env.limits.text);
            if !self.cancel_download(env, person, &guid, PendingKind::Release) {
                self.crash();
            }
            return;
        }
        self.downloads.push(Download { guid, person, page, name });
    }

    pub(super) fn download_progress(&mut self, env: &Env<Limits>, params: Value<'_>) {
        let (Some(guid), Some(state)) = (field_text(params, b"guid"), field_text(params, b"state")) else { return };
        if state.as_ref() != b"completed" && state.as_ref() != b"canceled" {
            return;
        }
        let mut index = None;
        for (at, download) in self.downloads.iter().enumerate() {
            if download.guid == guid {
                index = Some(at);
                break;
            }
        }
        let Some(index) = index else { return };
        let download = self.downloads.remove(index);
        let Some(person) = self.person(download.person) else { return };
        if person.closing {
            return;
        }
        let Some(directory) = &person.downloads else { return };
        if state.as_ref() == b"canceled" {
            self.trouble(download.page, Trouble::Download, download.name, env.limits.text);
            return;
        }
        let separator = usize::from(!directory.ends_with(b"/"));
        let length = match directory.len().checked_add(separator) {
            Some(length) => length.checked_add(download.guid.len()),
            None => None,
        };
        let too_long = match length {
            Some(length) => length > usize::try_from(env.limits.text).expect("u32 fits usize"),
            None => true,
        };
        if too_long {
            self.trouble(download.page, Trouble::Download, download.name, env.limits.text);
            return;
        }
        let mut path = Vec::with_capacity(length.expect("bounded path"));
        path.extend_from_slice(directory);
        if separator != 0 {
            path.push(b'/');
        }
        path.extend_from_slice(&download.guid);
        self.events.push_back(Event::Downloaded {
            person: download.person,
            name: download.name,
            path: path.into_boxed_slice(),
        });
    }
}
