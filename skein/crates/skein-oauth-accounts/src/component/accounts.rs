//! Account admission and removal (oauth/accounts.md, sections 2, 3 and 6).

use super::{AccountEntry, Component, Lending, Lifecycle, Name, Rejection, State, Unusable, failed, held, set_held};
use crate::accounts::Source;
use crate::keeper::{Operation, Private, Store};
use crate::{Account, Asked, Event, Failure, Keeper, Limits, Lower, Refusal, Transport};
use skein_lib::{Env, List, Queue, usize_of};
use skein_oauth as oauth;

pub(super) fn admit(account: u32, configuration: Account, limits: &Limits) -> Result<(Source, State, Store), Unusable> {
    Ok(match configuration {
        Account::HandedIn => (Source::HandedIn, State::Empty, Store::Owner),
        Account::SignIn { registration, endpoint, keeper, shared, origins } => {
            match endpoint.transport {
                Transport::Plaintext if !endpoint.address.ip().is_loopback() => {
                    return Err(Unusable::Plaintext { account });
                }
                Transport::Plaintext | Transport::Tls { .. } => {}
            }
            if registration.client_secret.is_none()
                && !crate::redirect::varying(&registration)
                && oauth::redirect_address(&registration.redirect_uri).is_err()
            {
                return Err(Unusable::Redirect { account });
            }
            match oauth::validate_registration(&registration, &limits.client, true) {
                Ok(()) => {}
                Err(failure) => return Err(Unusable::Registration { account, failure }),
            }
            if origins.len() > crate::limits::MAX_ORIGINS || !terms_origin(&registration, &origins) {
                return Err(Unusable::Origin { account });
            }
            let mut bounded = List::with_capacity(crate::limits::MAX_ORIGINS);
            for origin in origins.into_boxed() {
                if origin.uri.len() > usize_of(limits.client.uri_bytes)
                    || uri_origin(&origin.uri) != Some(origin.uri.as_ref())
                {
                    return Err(Unusable::Origin { account });
                }
                if let Transport::Plaintext = origin.endpoint.transport
                    && !origin.endpoint.address.ip().is_loopback()
                {
                    return Err(Unusable::Plaintext { account });
                }
                bounded.push(origin).expect("admitted origins");
            }
            let (state, store) = match keeper {
                Keeper::Owner { kept } => (
                    match kept {
                        Some(record) => {
                            match oauth::encode_record(&record, &limits.client.document) {
                                Ok(encoded) => drop(encoded),
                                Err(why) => return Err(Unusable::Record { account, why }),
                            }
                            if super::names::missing_id(&registration, &record) {
                                State::Empty
                            } else {
                                State::Loaded { record }
                            }
                        }
                        None => State::Empty,
                    },
                    Store::Owner,
                ),
                Keeper::Private { root, directory, file } => {
                    if directory.len() >= 4096 || file.len() >= 4096 {
                        return Err(Unusable::Path { account });
                    }
                    (State::Empty, Store::Private(Private::new(root, directory, file)))
                }
            };
            (Source::SignIn { registration, endpoint, shared, origins: bounded }, state, store)
        }
    })
}

impl Component {
    pub(super) fn want(&mut self, account: u32, wanted: bool, up: &mut Queue<Event>) {
        if !self.refused(account, Asked::Want, up) {
            let entry = self.entries.get_mut(account).expect("account entry");
            match &mut entry.name {
                Name::Live { wanted: flag, .. } => *flag = wanted,
                Name::Removing { .. } | Name::Free => unreachable!("live name"),
            }
            if !wanted {
                match &mut entry.state {
                    State::Record { lending: Lending::Free, refresh, .. } => *refresh = refresh.forgo(),
                    State::Record { .. } | State::Empty | State::Loaded { .. } => {}
                }
            }
        }
    }

    pub(super) fn release(&mut self, account: u32) {
        let entry = self.entries.get_mut(account).expect("account entry");
        let wanted = entry.wanted();
        match &mut entry.state {
            State::Record { lending, refresh, .. } => {
                *lending = Lending::Free;
                if !wanted {
                    *refresh = refresh.forgo();
                }
            }
            State::Empty | State::Loaded { .. } => {}
        }
    }

    pub(super) fn grant_before_refresh(&mut self, env: &Env<Limits>, account: u32, up: &mut Queue<Event>) {
        let entry = self.entries.get_mut(account).expect("account entry");
        match &mut entry.state {
            State::Record { expiry, rejection, .. } if env.now < *expiry && *rejection == Rejection::Clear => {
                super::grant(&mut entry.state, &entry.source, env.now, entry.name.number(), up);
            }
            State::Record { lending, refresh, .. } => {
                *lending = Lending::Held;
                *refresh = refresh.request();
            }
            State::Empty | State::Loaded { .. } => unreachable!("refreshable record"),
        }
    }

    /// Admits a named account, answering only a refusal into the uncapped owner queue.
    pub fn add(&mut self, env: &Env<Limits>, account: u32, configuration: Account, up: &mut Queue<Event>) {
        let why = if self.lifecycle != Lifecycle::Live {
            Some(Refusal::Closed)
        } else if self.terminal_place(account).is_some() {
            Some(Refusal::Taken)
        } else {
            None
        };
        if let Some(why) = why {
            up.push(Event::Refused { account, asked: Asked::Add, why });
            return;
        }
        let mut free = None;
        for place in 0..self.entries.len() {
            match &self.entries.get(place).expect("account entry").name {
                Name::Free => {
                    free = Some(place);
                    break;
                }
                Name::Live { .. } | Name::Removing { .. } => {}
            }
        }
        let place = match free {
            Some(place) => place,
            None if self.entries.room() > 0 => self.entries.len(),
            None => {
                up.push(Event::Refused {
                    account,
                    asked: Asked::Add,
                    why: Refusal::Full { bound: env.limits.accounts },
                });
                return;
            }
        };
        let (source, state, store) = match admit(account, configuration, &env.limits) {
            Ok(admitted) => admitted,
            Err(why) => {
                up.push(Event::Refused { account, asked: Asked::Add, why: Refusal::Unusable(why) });
                return;
            }
        };
        let entry =
            AccountEntry { name: Name::Live { number: account, wanted: false }, state, source, binding: None, store };
        if place == self.entries.len() {
            match self.entries.push(entry) {
                Ok(()) => {}
                Err(AccountEntry { .. }) => unreachable!("free account slot"),
            }
        } else {
            *self.entries.get_mut(place).expect("free account slot") = entry;
        }
    }

    pub(super) fn terminal_place(&self, number: u32) -> Option<u32> {
        for place in 0..self.entries.len() {
            match &self.entries.get(place).expect("account entry").name {
                Name::Live { number: held, .. } | Name::Removing { number: held } if number == *held => {
                    return Some(place);
                }
                Name::Live { .. } | Name::Removing { .. } | Name::Free => {}
            }
        }
        None
    }

    pub(super) fn removing(&self) -> bool {
        for entry in &self.entries {
            if entry.is_removing() {
                return true;
            }
        }
        false
    }

    pub(super) fn remove(&mut self, env: &Env<Limits>, place: u32, up: &mut Queue<Event>, io: &mut Queue<Lower>) {
        let entry = self.entries.get_mut(place).expect("account entry");
        entry.name = Name::Removing { number: entry.name.number() };
        self.removal_step(env, place, up, io);
    }

    pub(super) fn removal_has_work(&self) -> bool {
        for place in 0..self.entries.len() {
            let entry = self.entries.get(place).expect("account entry");
            if !entry.is_removing() {
                continue;
            }
            if held(&entry.state) {
                return true;
            }
            if let Some(id) = entry.binding {
                let exchange = self.exchanges.get(id).expect("bound exchange");
                if exchange.pending_grant || unsent(exchange) {
                    return true;
                }
            } else if self.account_has_exchanges(place) {
                if let Store::Private(keeper) = &entry.store
                    && keeper.waiting_grant
                {
                    return true;
                }
            } else {
                match &entry.store {
                    Store::Owner => return true,
                    Store::Private(keeper) => {
                        if keeper.waiting_grant || keeper.closed || keeper.pending.is_none() {
                            return true;
                        }
                        match &keeper.pending {
                            Some(pending) if pending.operation != Operation::Close && !pending.cancelled => {
                                return true;
                            }
                            Some(_) | None => {}
                        }
                    }
                }
            }
        }
        false
    }

    pub(super) fn removal_progress(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<Lower>) -> bool {
        for place in 0..self.entries.len() {
            if self.entries.get(place).expect("account entry").is_removing() && self.removal_step(env, place, up, io) {
                return true;
            }
        }
        false
    }

    fn removal_step(&mut self, env: &Env<Limits>, place: u32, up: &mut Queue<Event>, io: &mut Queue<Lower>) -> bool {
        let entry = self.entries.get_mut(place).expect("account entry");
        let number = entry.name.number();
        if let Some(id) = entry.binding {
            let exchange = self.exchanges.get_mut(id).expect("bound exchange");
            if unsent(exchange) {
                self.fail_exchange(env, id, Failure::Exchange(oauth::Failure::Cancelled), up, io);
                self.collect(env, id, up, io);
                return true;
            }

            if exchange.pending_grant || held(&entry.state) {
                exchange.pending_grant = false;
                set_held(&mut entry.state, Lending::Free);
                failed(number, Failure::Exchange(oauth::Failure::Cancelled), up);
                return true;
            }
            return false;
        }
        if held(&entry.state) {
            set_held(&mut entry.state, Lending::Free);
            failed(number, Failure::Exchange(oauth::Failure::Cancelled), up);
            return true;
        }
        match &mut entry.store {
            Store::Private(keeper) => {
                if keeper.waiting_grant {
                    keeper.waiting_grant = false;
                    failed(number, Failure::Exchange(oauth::Failure::Cancelled), up);
                    return true;
                }
                if !keeper.closed {
                    return false;
                }
            }
            Store::Owner => {}
        }
        if self.account_has_exchanges(place) {
            return false;
        }
        let entry = self.entries.get_mut(place).expect("account entry");
        entry.name = Name::Free;
        entry.state = State::Empty;
        entry.source = Source::HandedIn;
        entry.store = Store::Owner;
        up.push(Event::Removed { account: number });
        self.settled(up);
        true
    }
}

fn unsent(exchange: &crate::exchange::Exchange) -> bool {
    match exchange.stage {
        crate::exchange::Stage::Loading | crate::exchange::Stage::Locking | crate::exchange::Stage::Reading => true,
        crate::exchange::Stage::Running => match &exchange.web {
            Some(web) => !web.sent,
            None => true,
        },
        crate::exchange::Stage::Keeping { .. }
        | crate::exchange::Stage::Forgetting { .. }
        | crate::exchange::Stage::Finished => false,
    }
}

fn uri_origin(uri: &[u8]) -> Option<&[u8]> {
    let start = skein_lib::bytes::find(uri, b"://")?.checked_add(3)?;
    let rest = uri.get(start..)?;
    let end = skein_lib::bytes::find(rest, b"/").unwrap_or(rest.len()).checked_add(start)?;
    uri.get(..end)
}

pub(super) fn endpoint<'a>(
    uri: &[u8],
    registration: &oauth::Registration,
    token: &'a crate::Endpoint,
    origins: &'a List<crate::Origin>,
) -> Option<&'a crate::Endpoint> {
    let origin = uri_origin(uri)?;
    if Some(origin) == uri_origin(&registration.token_endpoint) {
        return Some(token);
    }
    for configured in origins {
        if origin == configured.uri.as_ref() {
            return Some(&configured.endpoint);
        }
    }
    None
}

fn terms_origin(registration: &oauth::Registration, origins: &List<crate::Origin>) -> bool {
    match &registration.terms {
        Some(terms) => {
            if let Some(endpoint) = &terms.revocation_endpoint
                && !given(endpoint, registration, origins)
            {
                return false;
            }
            match &terms.openid {
                Some(openid) => {
                    given(&openid.issuer, registration, origins)
                        && match &openid.key_set {
                            Some(uri) => given(uri, registration, origins),
                            None => true,
                        }
                }
                None => true,
            }
        }
        None => true,
    }
}

fn given(uri: &[u8], registration: &oauth::Registration, origins: &List<crate::Origin>) -> bool {
    let origin = uri_origin(uri);
    if origin == uri_origin(&registration.token_endpoint) {
        return true;
    }
    for configured in origins {
        if Some(configured.uri.as_ref()) == origin {
            return true;
        }
    }
    false
}
