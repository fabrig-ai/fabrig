//! Claude Code's terminal sign-in (temper/docs/design/client/pages/welcome.md, 1).
use super::{ConnectModel, Output, accounts, program_default};
use jig_client::{Client, contract as c};
use skein_lib::Queue;

/// The note and its ending, read from the accounts document and the press's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginState {
    Idle,
    Waiting { number: Option<u32>, cancelled: bool },
    Usable { provider: u32 },
    NotFinished,
    Refused { refusal: c::AccountRefusal },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequestState {
    Idle,
    Requested,
    Asked { number: u32 },
    Following,
    NoTerminal,
}
#[derive(Debug)]
pub(super) struct Login {
    state: RequestState,
    previous: Option<u32>,
    cancel: bool,
    cancel_sent: bool,
}
impl Login {
    pub(super) const fn new() -> Self {
        Self { state: RequestState::Idle, previous: None, cancel: false, cancel_sent: false }
    }
    pub(super) fn requested(&mut self, client: &Client) {
        self.state = RequestState::Requested;
        self.previous = match accounts::document(client) {
            Some(doc) => previous_end(doc),
            None => None,
        };
        self.cancel = false;
        self.cancel_sent = false;
    }
    pub(super) fn answered(&mut self, outcome: &jig_client::Outcome) {
        self.state = match outcome {
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountLoginAsked { number },
            )))) => RequestState::Asked { number: *number },
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::NoTerminal, ..
            })) => RequestState::NoTerminal,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::LoginUnderWay, ..
            })) => RequestState::Following,
            jig_client::Outcome::Answered(_)
            | jig_client::Outcome::Lost
            | jig_client::Outcome::Unsent
            | jig_client::Outcome::Full { .. }
            | jig_client::Outcome::NotKept
            | jig_client::Outcome::Held
            | jig_client::Outcome::Unknown
            | jig_client::Outcome::Limit
            | jig_client::Outcome::Unconfirmed => RequestState::Idle,
        };
    }
}
fn previous_end(doc: &c::Accounts) -> Option<u32> {
    Some(doc.lent_end.as_ref()?.number)
}
fn claude(client: &Client) -> Option<u32> {
    for offer in &accounts::current_me(client)?.providers {
        if offer.name.bytes() == b"anthropic" && !offer.programs.is_empty() {
            return Some(offer.provider);
        }
    }
    None
}
impl ConnectModel {
    /// The card has a terminal only while the document offers one and the press has not lost it.
    #[must_use]
    pub fn lendable(&self, client: &Client) -> bool {
        if self.login.state == RequestState::NoTerminal {
            return false;
        }
        match accounts::document(client) {
            Some(doc) => doc.lendable,
            None => false,
        }
    }
    #[must_use]
    pub fn login_state(&self, client: &Client) -> LoginState {
        let Some(provider) = claude(client) else { return LoginState::Idle };
        let doc = accounts::document(client);
        if let Some(doc) = doc
            && let Some(login) = &doc.lending
            && login.provider == provider
        {
            return LoginState::Waiting { number: Some(login.number), cancelled: login.cancelled || self.login.cancel };
        }
        if self.login.state == RequestState::Requested {
            return LoginState::Waiting { number: None, cancelled: self.login.cancel };
        }
        let number = match self.login.state {
            RequestState::Asked { number } => Some(number),
            RequestState::Idle | RequestState::Following | RequestState::NoTerminal => None,
            RequestState::Requested => unreachable!("requested login returned its note"),
        };
        if let Some(doc) = doc
            && let Some(end) = &doc.lent_end
            && Some(end.number) != self.login.previous
            && (number.is_none() || number == Some(end.number))
        {
            return match end.end {
                c::LoginEnd::Cancelled => LoginState::Idle,
                c::LoginEnd::NotFinished { .. } => LoginState::NotFinished,
                c::LoginEnd::Refused { refusal } => LoginState::Refused { refusal },
                c::LoginEnd::SignedIn { source } => signed_in(doc, provider, source, end.number),
            };
        }
        match self.login.state {
            RequestState::Asked { .. } | RequestState::Following => {
                LoginState::Waiting { number, cancelled: self.login.cancel }
            }
            RequestState::Idle | RequestState::NoTerminal => LoginState::Idle,
            RequestState::Requested => unreachable!("requested login returned its note"),
        }
    }
    pub(super) fn start_login(
        &mut self,
        client: &Client,
        name: skein_lib::RequestKey,
        serial: &mut u64,
        out: &mut Queue<Output>,
    ) {
        let Some(provider) = claude(client) else { return };
        if self.holder != c::AskAccountHolder::Own || !self.lendable(client) {
            return;
        }
        match self.login_state(client) {
            LoginState::Waiting { .. } => return,
            LoginState::Idle | LoginState::Usable { .. } | LoginState::NotFinished | LoginState::Refused { .. } => {}
        }
        let Some(me) = accounts::current_me(client) else { return };
        self.login.requested(client);
        self.request.send(
            name,
            c::Ask::AccountChange {
                holder: c::AskAccountHolder::Own,
                change: c::AccountChange::LoginProgram { provider, name: program_default(me.party.name.bytes()) },
            },
            false,
            serial,
            out,
        );
    }
    pub(super) fn cancel_login(&mut self, client: &Client) {
        match self.login_state(client) {
            LoginState::Waiting { .. } => self.login.cancel = true,
            LoginState::Idle | LoginState::Usable { .. } | LoginState::NotFinished | LoginState::Refused { .. } => {}
        }
    }
    pub(crate) fn refresh_login(&mut self, client: &Client, serial: &mut u64, out: &mut Queue<Output>) {
        if !self.login.cancel || self.login.cancel_sent {
            return;
        }
        if let Some(pending) = &self.request.pending
            && pending.outcome.is_none()
        {
            return;
        }
        match self.login_state(client) {
            LoginState::Waiting { number: Some(number), .. } => {
                self.login.cancel_sent = true;
                self.request.send(
                    super::super::account_request_name(*serial),
                    c::Ask::AccountChange {
                        holder: c::AskAccountHolder::Own,
                        change: c::AccountChange::CancelLogin { number },
                    },
                    false,
                    serial,
                    out,
                );
            }
            LoginState::Waiting { number: None, .. }
            | LoginState::Idle
            | LoginState::Usable { .. }
            | LoginState::NotFinished
            | LoginState::Refused { .. } => {}
        }
    }
}
fn signed_in(doc: &c::Accounts, provider: u32, source: u32, number: u32) -> LoginState {
    for row in &doc.accounts {
        if row.provider == Some(provider) {
            for login in &row.sources {
                if login.number == source && login.usable {
                    return LoginState::Usable { provider };
                }
            }
        }
    }
    if !doc.settled || doc.cut || !crate::route::program_settled(doc, source) {
        return LoginState::Waiting { number: Some(number), cancelled: false };
    }
    LoginState::NotFinished
}
