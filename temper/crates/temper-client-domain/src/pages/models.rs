//! Model switches and names ask the newest catalogue (temper/docs/design/client/README.md, 3.5, The `Models` tab).
use super::{Pages, Request, accounts, choice};
use crate::{Address, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c};
use skein_lib::Queue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Picker { number: u32, on: bool },
    Name(Box<[u8]>),
    Provider(u32),
    Add,
    Read(u32),
}
#[derive(Debug)]
struct Pending {
    key: Key,
    number: Option<u32>,
    provider: Option<u32>,
}
#[derive(Debug)]
struct Reading {
    key: Key,
    provider: u32,
    answered: bool,
    changed: bool,
}
#[derive(Debug)]
pub struct Models {
    reading: Option<Reading>,
    read_refused: Option<(u32, Outcome)>,
    pending: Option<Pending>,
    refused: Option<(Option<u32>, Outcome)>,
    name: Box<[u8]>,
    provider: Option<u32>,
    added: Option<u32>,
}
impl Models {
    pub(super) fn new() -> Self {
        Self {
            reading: None,
            read_refused: None,
            pending: None,
            refused: None,
            name: Box::default(),
            provider: None,
            added: None,
        }
    }
    #[must_use]
    pub fn reading(&self, provider: u32) -> bool {
        match &self.reading {
            Some(read) => read.provider == provider,
            None => false,
        }
    }
    #[must_use]
    pub fn read_refused(&self, provider: u32) -> Option<&Outcome> {
        let (at, outcome) = self.read_refused.as_ref()?;
        if *at == provider { Some(outcome) } else { None }
    }
    fn read(&mut self, client: &Client, provider: u32, serial: &mut u64, out: &mut Queue<Request>) -> bool {
        if self.reading.is_some() || !readable(client, provider) {
            return false;
        }
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.reading = Some(Reading { key, provider, answered: false, changed: false });
        self.read_refused = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::ReadLists { provider })),
        }));
        true
    }
    fn read_told(&mut self, client: &Client, told: &Told) -> bool {
        let Some(read) = &mut self.reading else { return false };
        if let Told::Changed { handle } = told {
            if *handle == accounts::handle() {
                read.changed = true;
                if read.answered {
                    self.reading = None;
                }
            }
            return false;
        }
        let outcome = match told {
            Told::Answered { key, outcome } if *key == read.key => outcome,
            Told::Pending { key } if *key == read.key => {
                if let Some(request) = client.pending(*key)
                    && request.progress == jig_client::Progress::Unknown
                {
                    &Outcome::Unknown
                } else {
                    return false;
                }
            }
            Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => return false,
        };
        if super::seen_answered(outcome) {
            read.answered = true;
            if read.changed {
                self.reading = None;
            }
        } else {
            self.read_refused = Some((read.provider, outcome.clone()));
            self.reading = None;
        }
        true
    }
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    #[must_use]
    pub const fn added(&self) -> Option<u32> {
        self.added
    }
    #[must_use]
    pub fn refused_at(&self, number: u32) -> Option<&Outcome> {
        let (row, outcome) = self.refused.as_ref()?;
        if *row == Some(number) { Some(outcome) } else { None }
    }
    #[must_use]
    pub fn pending(&self, number: u32) -> bool {
        match &self.pending {
            Some(pending) => pending.number == Some(number),
            None => false,
        }
    }
    #[must_use]
    pub fn add_pending(&self) -> bool {
        match &self.pending {
            Some(pending) => pending.number.is_none(),
            None => false,
        }
    }
    #[must_use]
    pub fn add_refused(&self) -> Option<&Outcome> {
        let (number, outcome) = self.refused.as_ref()?;
        if number.is_none() { Some(outcome) } else { None }
    }
    #[must_use]
    pub fn provider(&self, me: &c::Me, playground: Option<u32>) -> Option<u32> {
        let mut first = None;
        for provider in &me.providers {
            if Some(provider.provider) != playground {
                if self.provider == Some(provider.provider) {
                    return self.provider;
                }
                if first.is_none() {
                    first = Some(provider.provider);
                }
            }
        }
        first
    }
    #[must_use]
    pub fn duplicate(&self, me: &c::Me, project: u32, playground: Option<u32>) -> bool {
        let Some(provider) = self.provider(me, playground) else { return false };
        if let Some(models) = choice::offer(me, project) {
            for model in models {
                if model.provider == Some(provider) && model.name.bytes() == self.name.as_ref() {
                    return true;
                }
            }
        }
        false
    }
    #[must_use]
    pub fn can_add(&self, me: &c::Me, project: u32, playground: Option<u32>) -> bool {
        me.deployment_role == Some(c::HeldRole::Owner)
            && self.pending.is_none()
            && self.provider(me, playground).is_some()
            && name_shape(&self.name)
            && !self.duplicate(me, project, playground)
    }
    pub(super) fn act(
        &mut self,
        client: &Client,
        address: &Address,
        act: Act,
        playground: Option<u32>,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) -> bool {
        let Address::Models { project } = address else { return false };
        if let Act::Read(provider) = act {
            return self.read(client, provider, serial, out);
        }
        if self.pending.is_some() {
            return false;
        }
        let Some(me) = accounts::current_me(client) else { return false };
        if me.deployment_role != Some(c::HeldRole::Owner) {
            return false;
        }
        let Some(models) = choice::offer(me, *project) else { return false };
        let (act, number, provider) = match act {
            Act::Read(..) => return false,
            Act::Name(name) => {
                self.name = name;
                return false;
            }
            Act::Provider(provider) => {
                for offered in &me.providers {
                    if offered.provider == provider && Some(provider) != playground {
                        self.provider = Some(provider);
                    }
                }
                return false;
            }
            Act::Add => {
                if !self.can_add(me, *project, playground) {
                    return false;
                }
                let Some(provider) = self.provider(me, playground) else { return false };
                let name = c::Text::new(self.name.clone(), 128).expect("name shape checks the text bound");
                (c::ModelAct::Add { provider, name }, None, Some(provider))
            }
            Act::Picker { number, on } => {
                let mut offered = false;
                for model in models {
                    offered |= model.number == number && model.picker != on;
                }
                if !offered {
                    return false;
                }
                (c::ModelAct::Picker { number, on }, Some(number), None)
            }
        };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending = Some(Pending { key, number, provider });
        self.refused = None;
        self.added = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::ChangeModel(c::ChangeModel { act }))),
        }));
        true
    }
    pub(super) fn told(&mut self, client: &Client, told: &Told) -> bool {
        if self.read_told(client, told) {
            return true;
        }
        let Some(pending) = &self.pending else { return false };
        let outcome = match told {
            Told::Answered { key, outcome } if *key == pending.key => outcome,
            Told::Pending { key } if *key == pending.key => {
                if let Some(request) = client.pending(*key)
                    && request.progress == jig_client::Progress::Unknown
                {
                    &Outcome::Unknown
                } else {
                    return false;
                }
            }
            Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => return false,
        };
        if !super::seen_answered(outcome) {
            self.refused = Some((pending.number, outcome.clone()));
        } else if let Some(provider) = pending.provider {
            self.name = Box::default();
            self.added = Some(provider);
        }
        self.pending = None;
        true
    }
}
impl Pages {
    #[must_use]
    pub const fn models(&self) -> &Models {
        &self.models
    }
}

/// The engine's model-name shape (jig/docs/design/domain/engine.md, 17.11).
#[must_use]
pub fn name_shape(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    for &byte in name {
        if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.' && byte != b'_' && byte != b':' {
            return false;
        }
    }
    true
}

/// Only the person's usable list-bearing accounts offer a read (client/README.md, 3.5).
#[must_use]
pub fn readable(client: &Client, provider: u32) -> bool {
    let Some(me) = accounts::current_me(client) else { return false };
    let Some(document) = accounts::document(client) else { return false };
    for account in &document.accounts {
        let mut usable = false;
        for source in &account.sources {
            usable |= source.usable;
        }
        if account.provider == Some(provider)
            && account.holder == c::AccountHolder::Party(me.party.number)
            && account.list.is_some()
            && usable
        {
            return true;
        }
    }
    false
}
