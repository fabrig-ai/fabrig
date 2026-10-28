//! One page's account request and one-shot secret transport.
use super::{Broken, KEY_BYTES, Paste, Pending, SignInState, document};
use crate::{owner::Owner, pages::Request as Output};
use alloc::boxed::Box;
use jig_client::{Client, Key, Told, contract as c};
use skein_lib::{Queue, RequestKey, Secret};
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the one-shot value is never bound or persisted")]
pub struct Request {
    pub pending: Option<Pending>,
    pub paste: Paste,
    pub confirm: Option<(RequestKey, c::Ask)>,
    pub broken: Option<Broken>,
    value: Option<Secret>,
    written: u64,
    returned: Option<u32>,
}
impl Request {
    #[must_use]
    pub const fn key_written(&self) -> u64 {
        self.written
    }
    pub(crate) fn clear_key(&mut self) {
        self.value = None;
        self.paste.clear();
        self.written = self.written.checked_add(1).expect("account field writes fit");
    }
    #[must_use]
    pub fn busy(&self, client: &Client) -> bool {
        if self.paste.out() {
            return true;
        }
        if let Some(p) = &self.pending
            && p.outcome.is_none()
        {
            return true;
        }
        match self.sign_in_state(client) {
            SignInState::Started { .. } | SignInState::Signing { .. } => true,
            SignInState::Unknown | SignInState::Ended { .. } => false,
        }
    }
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.value.is_some()
    }
    /// Started is an address to visit, never proof of a completed login.
    #[must_use]
    pub fn sign_in_state(&self, client: &Client) -> SignInState {
        if self.pending.is_none()
            && let Some(source) = self.returned
        {
            let Some(accounts) = document(client) else { return SignInState::Signing { source } };
            for signing in &accounts.signing {
                if signing.source == source {
                    return SignInState::Signing { source };
                }
            }
            if let Some(end) = &accounts.last_end
                && end.source == source
            {
                return SignInState::Ended { source };
            }
            return SignInState::Unknown;
        }
        let Some((source, _)) = self.started() else {
            return SignInState::Unknown;
        };
        let pending = self.pending.as_ref().expect("started request");
        if pending.ended {
            return SignInState::Ended { source };
        }
        if let Some(accounts) = document(client) {
            for signing in &accounts.signing {
                if signing.source == source {
                    return SignInState::Signing { source };
                }
            }
        }
        if pending.name.is_some() { SignInState::Started { source } } else { SignInState::Unknown }
    }
    pub(crate) fn refresh(&mut self, client: &Client) {
        let Some((source, _)) = self.started() else {
            return;
        };
        let Some(accounts) = document(client) else {
            return;
        };
        let pending = self.pending.as_mut().expect("started request");
        let mut signing = false;
        for row in &accounts.signing {
            if row.source == source {
                signing = true;
            }
        }
        let ended = match &accounts.last_end {
            Some(end) => end.source == source,
            None => false,
        };
        pending.ended |= ended || (!signing && !accounts.cut && (pending.signing || pending.name.is_none()));
        pending.signing |= signing;
    }
    #[must_use]
    pub fn sign_in_address(&self, client: &Client) -> Option<&[u8]> {
        match self.sign_in_state(client) {
            SignInState::Started { .. } | SignInState::Signing { .. } => Some(self.started()?.1),
            SignInState::Unknown | SignInState::Ended { .. } => None,
        }
    }
    #[must_use]
    pub const fn returned(&self) -> bool {
        self.pending.is_none() && self.returned.is_some()
    }
    pub(crate) fn return_from(&mut self, source: Option<u32>) {
        self.returned = source;
    }
    fn started(&self) -> Option<(u32, &[u8])> {
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
            c::OwnerOutcome::AccountSignInStarted { source, address },
        )))) = self.pending.as_ref()?.outcome.as_ref()?
        {
            Some((*source, address))
        } else {
            None
        }
    }
}

impl Request {
    pub(crate) fn key(&mut self, value: Secret) {
        self.value = None;
        if let Some(pending) = &self.pending
            && pending.outcome.is_none()
        {
            return;
        }
        self.broken = if value.is_empty() {
            Some(Broken::Empty)
        } else if value.len() > KEY_BYTES {
            Some(Broken::TooLong)
        } else {
            self.value = Some(value);
            None
        };
    }
    pub(crate) fn digest(&mut self) -> Option<[u8; 32]> {
        if self.value.is_none() {
            self.broken = Some(Broken::ValueAgain);
        }
        Some(skein_lib::sha256::digest(self.value.as_ref()?.expose()))
    }
    pub(crate) fn send(
        &mut self,
        name: RequestKey,
        ask: c::Ask,
        carrying: bool,
        serial: &mut u64,
        out: &mut Queue<Output>,
    ) {
        if let Some(pending) = &self.pending
            && pending.outcome.is_none()
        {
            return;
        }
        if self.paste.out() {
            return;
        }
        if let c::Ask::SignIn { .. } = ask {
            self.paste = Paste::default();
        }
        let value = self.value.take();
        self.clear_key();
        if carrying && value.is_none() {
            self.broken = Some(Broken::ValueAgain);
            return;
        }
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending =
            Some(Pending { key, name: Some(name), ask: ask.clone(), outcome: None, signing: false, ended: false });
        self.broken = None;
        let request = jig_client::Ask::Jig(Box::new(ask));
        let event = if carrying {
            jig_client::Event::Carry { key, name, request, value: Some(value.expect("checked value")) }
        } else {
            jig_client::Event::Send { key, name: Some(name), request }
        };
        out.push(Output::Client(event));
    }
    pub(crate) fn told(&mut self, client: &Client, told: &Told) -> bool {
        match told {
            Told::Kept { key, request } => {
                let jig_client::Ask::Jig(ask) = request else {
                    return false;
                };
                if let c::Ask::AccountChange { change: c::AccountChange::FinishSignIn { .. }, .. } = ask.as_ref() {
                    if self.paste.pending.is_none() {
                        self.paste.pending = Some(Pending {
                            key: *key,
                            name: None,
                            ask: *ask.clone(),
                            outcome: None,
                            signing: false,
                            ended: false,
                        });
                    }
                    return true;
                }
                if let c::Ask::SignIn { .. } | c::Ask::SignOut { .. } | c::Ask::AccountChange { .. } = ask.as_ref() {
                    if self.pending.is_none() {
                        self.pending = Some(Pending {
                            key: *key,
                            name: None,
                            ask: *ask.clone(),
                            outcome: None,
                            signing: false,
                            ended: false,
                        });
                    }
                    true
                } else {
                    false
                }
            }
            Told::Answered { key, outcome } => {
                if let Some(pending) = &mut self.paste.pending
                    && pending.key == *key
                {
                    pending.outcome = Some(outcome.clone());
                    return true;
                }
                if let Some(pending) = &mut self.pending
                    && pending.key == *key
                {
                    pending.outcome = Some(outcome.clone());
                    self.refresh(client);
                    true
                } else {
                    false
                }
            }
            Told::Pending { key } if self.paste.has_pending(*key) => true,
            Told::Pending { key } => match &self.pending {
                Some(pending) => pending.key == *key,
                None => false,
            },
            Told::Changed { handle } if *handle == super::handle() => {
                self.refresh(client);
                true
            }
            Told::Link { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
}

impl Request {
    pub(crate) fn cancel(&mut self) {
        self.confirm = None;
        self.clear_key();
        self.broken = None;
        if let Some(pending) = &self.pending
            && let Some(jig_client::Outcome::Lost | jig_client::Outcome::Unsent | jig_client::Outcome::Full { .. }) =
                &pending.outcome
        {
            self.pending = None;
        }
    }
    pub(crate) fn confirm(&mut self, serial: &mut u64, out: &mut Queue<Output>) {
        if let Some((name, ask)) = self.confirm.take() {
            self.send(name, ask, false, serial, out);
        }
    }
}

#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for Request {
    fn default() -> Self {
        Self {
            pending: None,
            paste: Paste::default(),
            confirm: None,
            broken: None,
            value: None,
            written: 0,
            returned: None,
        }
    }
}
