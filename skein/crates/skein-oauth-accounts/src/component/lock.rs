//! Shared writes read once under the owner's lock (oauth.md, section 6.4).

use super::{Component, Lending, Lifecycle, Refresh, State, grant, held, recorded};
use crate::accounts::{Source, copy_registration};
use crate::exchange::{Exchange, OwnerLock, Stage};
use crate::keeper::{Operation, Store};
use crate::{Event, Failure, Limits, Locked, Lower};
use skein_lib::{Env, Id, Queue};
use skein_oauth as oauth;

impl Component {
    pub(super) fn lock_write(&mut self, env: &Env<Limits>, id: Id<Exchange>, up: &mut Queue<Event>) -> bool {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        let entry = self.entries.get(exchange.account).expect("account entry");
        let shared = match &entry.source {
            Source::SignIn { shared, .. } => *shared,
            Source::HandedIn => false,
        };
        if !shared || exchange.lock == OwnerLock::Held {
            return false;
        }
        exchange.lock = OwnerLock::Waiting { until: env.now.saturating_add(env.limits.lock_wait) };
        exchange.stage = Stage::Locking;
        up.push(Event::Lock { account: entry.name.number() });
        true
    }

    pub(super) fn locked(
        &mut self,
        env: &Env<Limits>,
        account: u32,
        answer: Locked,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        if self.lifecycle == Lifecycle::Closed {
            return;
        }
        let entry = self.entries.get_mut(account).expect("account entry");
        let pending = match entry.binding {
            Some(id) => match self.exchanges.get(id).expect("bound exchange").lock {
                OwnerLock::Waiting { .. } => Some(id),
                OwnerLock::Released | OwnerLock::Held => None,
            },
            None => None,
        };
        let Some(id) = pending else {
            if let Locked::Held { .. } = answer {
                up.push(Event::Unlock { account: entry.name.number() });
            }
            return;
        };
        match answer {
            Locked::Busy => {
                self.exchanges.get_mut(id).expect("bound exchange").lock = OwnerLock::Released;
                self.fail_exchange(env, id, Failure::Busy, up, io);
                self.collect(env, id, up, io);
            }
            Locked::Held { record } => {
                let exchange = self.exchanges.get_mut(id).expect("bound exchange");
                exchange.lock = OwnerLock::Held;
                if let Stage::Finished = exchange.stage {
                    self.collect(env, id, up, io);
                    return;
                }
                match &mut entry.store {
                    Store::Owner => self.read_locked(env, id, record, up, io),
                    Store::Private(keeper) => {
                        exchange.stage = Stage::Reading;
                        exchange.file_active = true;
                        keeper.reload();
                    }
                }
            }
        }
    }

    pub(super) fn lock_loaded(
        &mut self,
        env: &Env<Limits>,
        account: u32,
        record: &mut Option<oauth::SavedToken>,
        failure: Option<Failure>,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) -> bool {
        if let Some(id) = self.entries.get(account).expect("account entry").binding {
            let exchange = self.exchanges.get_mut(id).expect("bound exchange");
            if exchange.lock == OwnerLock::Held && exchange.file_active {
                exchange.file_active = false;
                match exchange.stage {
                    Stage::Reading => {
                        if let Some(failure) = failure {
                            self.fail_exchange(env, id, failure, up, io);
                            self.collect(env, id, up, io);
                        } else {
                            self.read_locked(env, id, record.take(), up, io);
                        }
                        return true;
                    }
                    Stage::Finished => {
                        self.collect(env, id, up, io);
                        return true;
                    }
                    Stage::Loading
                    | Stage::Locking
                    | Stage::Running
                    | Stage::Keeping { .. }
                    | Stage::Forgetting { .. } => unreachable!("shared file read"),
                }
            }
        }
        false
    }

    pub(super) fn read_locked(
        &mut self,
        env: &Env<Limits>,
        id: Id<Exchange>,
        record: Option<oauth::SavedToken>,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        exchange.file_active = false;
        let account = exchange.account;
        let entry = self.entries.get_mut(account).expect("account entry");
        let different = match &entry.state {
            State::Record { record: prior, .. } => match &record {
                Some(record) => record.subject != prior.subject,
                None => false,
            },
            State::Empty | State::Loaded { .. } => false,
        };
        if different && !exchange.signing_in() {
            self.fail_exchange(env, id, Failure::AnotherSubject, up, io);
            let entry = self.entries.get_mut(account).expect("account entry");
            entry.state = State::Empty;
            self.collect(env, id, up, io);
            return;
        }
        if let Some((candidate, _)) = &mut exchange.lock_candidate {
            if record.is_none() {
                entry.state = State::Empty;
            }
            if let Some(record) = record {
                let Some(next) = record.generation.checked_add(1) else {
                    self.fail_exchange(env, id, Failure::Exchange(oauth::Failure::Limit), up, io);
                    self.collect(env, id, up, io);
                    return;
                };
                candidate.generation = candidate.generation.max(next);
            }
            self.keep_candidate(env, id, up, io);
            return;
        }
        match record {
            Some(record) => {
                let newer = match &mut entry.state {
                    State::Record { record: prior, .. } => {
                        record.generation > prior.generation && record.subject == prior.subject
                    }
                    State::Empty | State::Loaded { .. } => false,
                };
                if newer {
                    let holding = if held(&entry.state) { Lending::Held } else { Lending::Free };
                    entry.state = recorded(record, env, holding);
                    let ready = match &mut entry.state {
                        State::Record { lead, .. } => env.now < *lead,
                        State::Empty | State::Loaded { .. } => unreachable!("record just read"),
                    };
                    if ready && !exchange.signing_out() {
                        if holding == Lending::Held || exchange.pending_grant {
                            grant(&mut entry.state, &entry.source, env.now, entry.name.number(), up);
                        }
                        exchange.stage = Stage::Finished;
                        self.collect(env, id, up, io);
                        return;
                    }
                }
            }
            None => {
                if exchange.signing_out() {
                    self.finish_sign_out(env, id, false, true, up, io);
                } else {
                    self.fail_exchange(env, id, Failure::Expired, up, io);
                }
                let entry = self.entries.get_mut(account).expect("account entry");
                entry.state = State::Empty;
                self.collect(env, id, up, io);
                return;
            }
        }
        if exchange.signing_out() {
            self.begin_sign_out(env, account, up, io);
        } else {
            self.begin_refresh(env, id);
        }
    }

    pub(super) fn keep_candidate(
        &mut self,
        env: &Env<Limits>,
        id: Id<Exchange>,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        let (record, missing) = exchange.lock_candidate.take().expect("candidate waiting to keep");
        if let Some(subject) = &exchange.subject
            && record.subject.as_ref() != Some(subject)
        {
            self.fail_exchange(env, id, Failure::AnotherSubject, up, io);
            self.collect(env, id, up, io);
            return;
        }
        let generation = record.generation;
        let entry = self.entries.get_mut(exchange.account).expect("account entry");
        match &mut entry.store {
            Store::Owner => {
                up.push(Event::Keep { account: entry.name.number(), record: record.clone() });
            }
            Store::Private(keeper) => {
                keeper.next = Some(Operation::Store { generation });
                exchange.file_active = true;
            }
        }
        exchange.stage = Stage::Keeping { candidate: record, missing };
    }

    pub(super) fn start_refresh(
        &mut self,
        env: &Env<Limits>,
        account: u32,
        pending_grant: bool,
        up: &mut Queue<Event>,
    ) {
        let entry = self.entries.get_mut(account).expect("account entry");
        if let State::Record { refresh, .. } = &mut entry.state {
            *refresh = Refresh::Spent;
        }
        let client = oauth::Client::new(env.limits.client).expect("validated client limits");
        let exchange = Exchange::new(account, client, pending_grant);
        let id = self.insert_exchange(exchange);
        let entry = self.entries.get_mut(account).expect("account entry");
        entry.binding = Some(id);
        if !self.lock_write(env, id, up) {
            self.begin_refresh(env, id);
        }
    }

    fn begin_refresh(&mut self, env: &Env<Limits>, id: Id<Exchange>) {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        let entry = self.entries.get_mut(exchange.account).expect("account entry");
        let prior = match &mut entry.state {
            State::Record { record, refresh, .. } => {
                *refresh = Refresh::Spent;
                record.refresh_state().expect("refreshable account")
            }
            State::Empty | State::Loaded { .. } => unreachable!("refreshable account"),
        };
        let registration = match &entry.source {
            Source::SignIn { registration, .. } => copy_registration(registration),
            Source::HandedIn => unreachable!("sign-in source"),
        };
        exchange.stage = Stage::Running;
        exchange.client.step(oauth::Event::Refresh { registration, prior, now: env.now }, &mut exchange.above);
    }
}
