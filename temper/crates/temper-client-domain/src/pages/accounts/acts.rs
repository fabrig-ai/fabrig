//! Named operations and one-shot secret transport; jig owns admission and durability.
use super::{Accounts, Request, Row};
use jig_client::{Client, Key, contract as c};
use skein_lib::{Queue, RequestKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    SignIn { provider: u32 },
    SignOut { source: u32 },
    AddKey { provider: u32 },
    ReplaceKey { source: u32 },
    RemoveKey { source: u32 },
    RemoveAccount,
    Resume { source: u32 },
    MoveAccount { place: u32 },
    MoveSource { source: u32, place: u32 },
    CheckProgram { source: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Finish { name: RequestKey },
    Submit { name: RequestKey, row: Row, operation: Operation },
    KeyFor { row: Row, source: Option<u32> },
    Confirm,
    Cancel,
}
#[derive(Debug)]
pub struct Pending {
    pub key: Key,
    pub name: Option<RequestKey>,
    pub ask: c::Ask,
    pub outcome: Option<jig_client::Outcome>,
    pub signing: bool,
    pub ended: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    Empty,
    TooLong,
    Selection,
    Rank,
    ValueAgain,
    ProgramName,
}

impl Accounts {
    pub(crate) fn act(&mut self, client: &Client, act: Act, serial: &mut u64, out: &mut Queue<Request>) {
        match act {
            Act::Finish { name } => self.request.finish(client, name, serial, out),
            Act::KeyFor { row, source } => {
                if self.request.busy(client) {
                    return;
                }
                if self.key_for != Some((row, source)) {
                    self.request.clear_key();
                }
                self.key_for = Some((row, source));
                self.request.confirm = None;
                self.request.broken = None;
            }
            Act::Cancel => self.request.cancel(),
            Act::Confirm => self.request.confirm(serial, out),
            Act::Submit { name, row, operation } => {
                self.submit(client, name, row, operation, serial, out);
            }
        }
    }
    fn submit(
        &mut self,
        client: &Client,
        name: RequestKey,
        row: Row,
        operation: Operation,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let restart = if let Operation::SignIn { .. } = operation {
            self.request.paste_state(client) == super::PasteState::Ended
                || self.request.paste_state(client) == super::PasteState::Refused { why: c::SignInRedirectWhy::TooMany }
        } else {
            false
        };
        if self.request.busy(client) && !restart {
            return;
        }
        let field = match operation {
            Operation::AddKey { .. } => Some(None),
            Operation::ReplaceKey { source } => Some(Some(source)),
            Operation::SignIn { .. }
            | Operation::SignOut { .. }
            | Operation::RemoveKey { .. }
            | Operation::RemoveAccount
            | Operation::Resume { .. }
            | Operation::MoveAccount { .. }
            | Operation::MoveSource { .. }
            | Operation::CheckProgram { .. } => None,
        };
        if let Some(source) = field
            && self.key_for != Some((row, source))
        {
            self.request.clear_key();
            self.request.broken = Some(Broken::ValueAgain);
            return;
        }
        if operation.carries() && !self.request.has_key() {
            if self.request.broken != Some(Broken::Empty) && self.request.broken != Some(Broken::TooLong) {
                self.request.broken = Some(Broken::ValueAgain);
            }
            return;
        }
        let Some(ask) = self.ask(client, row, operation) else {
            return;
        };
        if let Some(pending) = &self.request.pending
            && let Some(jig_client::Outcome::Lost | jig_client::Outcome::Unsent | jig_client::Outcome::Full { .. }) =
                &pending.outcome
            && (pending.name != Some(name) || pending.ask != ask)
        {
            self.request.clear_key();
            self.request.broken = Some(Broken::ValueAgain);
            return;
        }
        self.request_row = Some(row);
        self.request.confirm = None;
        if operation.confirms() {
            self.request.confirm = Some((name, ask));
        } else {
            self.request.send(name, ask, operation.carries(), serial, out);
        }
    }
    fn ask(&mut self, client: &Client, row: Row, operation: Operation) -> Option<c::Ask> {
        self.request.broken = Some(Broken::Selection);
        let me = super::current_me(client)?;
        if row.holder == c::AccountHolder::Deployment && me.deployment_role != Some(c::HeldRole::Owner) {
            return None;
        }
        // Explicit numbers are forwarded; account/source policy remains the engine's.
        let holder = row.holder;
        let change = match operation {
            Operation::SignIn { provider } => {
                if holder == c::AccountHolder::Deployment {
                    return None;
                }
                self.request.broken = None;
                return Some(c::Ask::SignIn { holder, provider });
            }
            Operation::SignOut { source } => {
                self.request.broken = None;
                return Some(c::Ask::SignOut { holder, source });
            }
            Operation::AddKey { provider } => {
                c::AccountChange::AddKey { provider, join: Some(row.account), digest: self.request.digest()? }
            }
            Operation::ReplaceKey { source } => c::AccountChange::ReplaceKey { source, digest: self.request.digest()? },
            Operation::RemoveKey { source } => c::AccountChange::RemoveKey { source },
            Operation::CheckProgram { source } => c::AccountChange::CheckProgram { source },
            Operation::RemoveAccount => c::AccountChange::Remove { account: row.account },
            Operation::Resume { source } => c::AccountChange::Resume { source },
            Operation::MoveAccount { place } => c::AccountChange::MoveAccount { account: row.account, place },
            Operation::MoveSource { source, place } => {
                c::AccountChange::MoveSource { account: row.account, source, place }
            }
        };
        self.request.broken = None;
        Some(c::Ask::AccountChange { holder, change })
    }
}

impl Operation {
    const fn carries(self) -> bool {
        match self {
            Self::AddKey { .. } | Self::ReplaceKey { .. } => true,
            Self::SignIn { .. }
            | Self::SignOut { .. }
            | Self::RemoveKey { .. }
            | Self::RemoveAccount
            | Self::Resume { .. }
            | Self::MoveAccount { .. }
            | Self::MoveSource { .. }
            | Self::CheckProgram { .. } => false,
        }
    }
    const fn confirms(self) -> bool {
        match self {
            Self::RemoveAccount | Self::RemoveKey { .. } | Self::SignOut { .. } => true,
            Self::SignIn { .. }
            | Self::AddKey { .. }
            | Self::ReplaceKey { .. }
            | Self::Resume { .. }
            | Self::MoveAccount { .. }
            | Self::MoveSource { .. }
            | Self::CheckProgram { .. } => false,
        }
    }
}
