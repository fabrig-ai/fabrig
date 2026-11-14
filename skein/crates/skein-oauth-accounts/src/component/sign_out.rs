//! Sign-out revokes once, then forgets the record (oauth/README.md, section 2.3, and oauth/accounts.md, section 3).

use super::{Component, Lending, State, failed, held, set_held};
use crate::accounts::{Source, copy_registration};
use crate::exchange::{Exchange, Purpose, Stage};
use crate::keeper::{Operation, Store};
use crate::{Asked, Ends, Event, Failure, Limits, Lower, Refusal};
use skein_io::file;
use skein_lib::{Env, Id, Queue};
use skein_oauth as oauth;

impl Component {
    pub(super) fn sign_out(&mut self, env: &Env<Limits>, account: u32, up: &mut Queue<Event>, io: &mut Queue<Lower>) {
        if self.refused(account, Asked::SignOut, up) {
            return;
        }
        let entry = self.entries.get_mut(account).expect("account entry");
        let why = match &entry.source {
            Source::HandedIn => Some(Refusal::Source),
            Source::SignIn { .. } => {
                if entry.binding.is_some() {
                    Some(Refusal::Busy)
                } else if self.exchanges.is_full() {
                    Some(Refusal::Full { bound: env.limits.exchanges })
                } else {
                    None
                }
            }
        };
        if let Some(why) = why {
            up.push(Event::Refused { account: entry.name.number(), asked: Asked::SignOut, why });
            return;
        }
        let empty = match &entry.state {
            State::Empty => true,
            State::Record { .. } | State::Loaded { .. } => false,
        };
        let loading = match &mut entry.store {
            Store::Private(keeper) if empty => {
                keeper.reload();
                true
            }
            Store::Private(_) | Store::Owner => false,
        };
        if empty && !loading {
            up.push(Event::Refused { account: entry.name.number(), asked: Asked::SignOut, why: Refusal::NoRecord });
            return;
        }
        let mut limits = env.limits.client;
        limits.max_attempts = 1;
        let mut exchange = Exchange::new(account, oauth::Client::new(limits).expect("validated client"), false);
        exchange.purpose = Purpose::SignOut;
        exchange.stage = Stage::Loading;
        let id = self.insert_exchange(exchange);
        let entry = self.entries.get_mut(account).expect("account entry");
        entry.binding = Some(id);
        if !loading {
            self.begin_sign_out(env, account, up, io);
        }
    }

    pub(super) fn begin_sign_out(
        &mut self,
        env: &Env<Limits>,
        account: u32,
        up: &mut Queue<Event>,
        _io: &mut Queue<Lower>,
    ) {
        let entry = self.entries.get_mut(account).expect("account entry");
        let id = entry.binding.expect("sign-out exchange");
        if held(&entry.state) {
            set_held(&mut entry.state, Lending::Free);
            failed(entry.name.number(), Failure::Expired, up);
        }
        if self.lock_write(env, id, up) {
            return;
        }
        let entry = self.entries.get(account).expect("account entry");
        let registration = match &entry.source {
            Source::SignIn { registration, .. } => copy_registration(registration),
            Source::HandedIn => unreachable!("sign-out source"),
        };
        let prior = match &entry.state {
            State::Record { record, .. } => record.refresh_state(),
            State::Empty | State::Loaded { .. } => unreachable!("loaded sign-out record"),
        };
        let endpoint = match &registration.terms {
            Some(terms) => terms.revocation_endpoint.is_some(),
            None => false,
        };
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        exchange.stage = Stage::Running;
        match prior {
            Some(prior) if endpoint => {
                exchange.client.step(oauth::Event::Revoke { registration, prior, now: env.now }, &mut exchange.above);
            }
            Some(_) | None => exchange.above.push(oauth::Request::Revoked { confirmed: false }),
        }
    }

    pub(super) fn forget(
        &mut self,
        env: &Env<Limits>,
        id: Id<Exchange>,
        revoked: bool,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        assert!(exchange.signing_out(), "revocation belongs to sign-out");
        exchange.stage = Stage::Forgetting { revoked };
        let entry = self.entries.get_mut(exchange.account).expect("account entry");
        match &mut entry.store {
            Store::Owner => self.finish_sign_out(env, id, revoked, true, up, io),
            Store::Private(keeper) => {
                keeper.next = Some(Operation::Remove);
                exchange.file_active = true;
            }
        }
    }

    pub(super) fn forgotten(
        &mut self,
        env: &Env<Limits>,
        account: u32,
        event: file::Event,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        let entry = self.entries.get_mut(account).expect("account entry");
        let id = entry.binding.expect("removal exchange");
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        exchange.file_active = false;
        let revoked = match exchange.stage {
            Stage::Forgetting { revoked } => revoked,
            Stage::Finished => {
                self.collect(env, id, up, io);
                return;
            }
            Stage::Loading | Stage::Locking | Stage::Reading | Stage::Running | Stage::Keeping { .. } => {
                unreachable!("sign-out removal")
            }
        };
        let removed = match event {
            file::Event::Removed { .. } => true,
            file::Event::Failed { .. }
            | file::Event::Cancelled { .. }
            | file::Event::Made { .. }
            | file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Refused { .. }
            | file::Event::Opened { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stored { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Closed { .. }
            | file::Event::Linked { .. }
            | file::Event::Renamed { .. }
            | file::Event::Listed { .. } => false,
        };
        if removed {
            match &mut entry.store {
                Store::Private(keeper) => keeper.expected = file::Expect::Absent,
                Store::Owner => unreachable!("private removal"),
            }
        }
        self.finish_sign_out(env, id, revoked, removed, up, io);
        self.collect(env, id, up, io);
    }

    pub(super) fn finish_sign_out(
        &mut self,
        env: &Env<Limits>,
        id: Id<Exchange>,
        revoked: bool,
        removed: bool,
        up: &mut Queue<Event>,
        io: &mut Queue<Lower>,
    ) {
        let exchange = self.exchanges.get_mut(id).expect("bound exchange");
        let entry = self.entries.get_mut(exchange.account).expect("account entry");
        entry.state = State::Empty;
        let account = entry.name.number();
        if removed {
            up.push(Event::SignedOut { account, revoked });
        } else {
            up.push(Event::Failed { account, ends: Ends::SignOut, failure: Failure::NotKept });
        }
        if exchange.pending_grant {
            failed(account, Failure::Expired, up);
            exchange.pending_grant = false;
        }
        exchange.stage = Stage::Finished;
        if let Some(web) = &mut exchange.web {
            web.close(env, false, io);
        }
    }
}
