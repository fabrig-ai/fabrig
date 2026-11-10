//! Verify one installation before retaining its id (forge/setup/github.md, 1.4).
use super::{
    app,
    given::{self, Given, Refused},
    reply,
};
use crate::github::Reply;
use crate::{
    item::{self, Item},
    plan::{self, Event, Need, Outcome, Stage, Why},
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge::github::{document::Document, request::Request};
use skein_lib::{Decimal, List, Queue};

#[derive(Debug)]
enum Phase {
    Begin,
    Id,
    Key(NonZeroU64),
    Accepted(app::Kept),
    Installation(Box<[u64]>),
    Written,
    Ended,
}
/// The host admits and takes the state before this machine begins (5.7).
#[derive(Debug)]
pub struct Accept {
    forge: u16,
    installation: NonZeroU64,
    key_bytes: u32,
    installations: u32,
    phase: Phase,
}
impl Accept {
    pub fn new(given: &Given, installation: NonZeroU64) -> Result<Self, Refused> {
        given::check(given)?;
        Ok(Self {
            forge: given.forge,
            installation,
            key_bytes: given.limits.key_bytes,
            installations: given.limits.installations,
            phase: Phase::Begin,
        })
    }
    fn stop(&self, why: Why) -> Need {
        plan::stop(self.forge, Stage::Installation, Decimal::of(self.installation.get()).as_bytes(), why)
    }
    fn verified(&mut self, ids: &[u64]) -> Need {
        let id = self.installation.get();
        if ids.contains(&id) {
            return Need::Done(Outcome::Nothing);
        }
        if u32::try_from(ids.len()).expect("bounded accepted set") == self.installations {
            return self.stop(Why::Unexpected);
        }
        let mut updated = List::with_capacity(self.installations);
        let mut inserted = false;
        for kept in ids {
            if !inserted && id < *kept {
                updated.push(id).expect("room for new installation");
                inserted = true;
            }
            updated.push(*kept).expect("bounded accepted set");
        }
        if !inserted {
            updated.push(id).expect("room for new installation");
        }
        self.phase = Phase::Written;
        Need::Write { item: Item::Accepted, bytes: item::accepted_item(updated.as_slice()) }
    }
    fn advance(&mut self, phase: Phase, event: Event) -> Need {
        match phase {
            Phase::Begin => match event.begin() {
                Ok(()) => {
                    self.phase = Phase::Id;
                    Need::Read(Item::AppId)
                }
                Err(why) => plan::stop(self.forge, Stage::App, b"id", why),
            },
            Phase::Id => match event.read() {
                Ok(read) => match app::id_of(read) {
                    Some(id) => {
                        self.phase = Phase::Key(id);
                        Need::Read(Item::AppKey)
                    }
                    None => plan::stop(self.forge, Stage::App, b"id", Why::Missing),
                },
                Err(why) => plan::stop(self.forge, Stage::App, b"id", why),
            },
            Phase::Key(id) => match event.read() {
                Ok(Some(key)) if item::app_key(key.expose(), self.key_bytes).is_some() => {
                    self.phase = Phase::Accepted(app::Kept::new(id, key));
                    Need::Read(Item::Accepted)
                }
                Ok(_) => plan::stop(self.forge, Stage::App, b"key", Why::Missing),
                Err(why) => plan::stop(self.forge, Stage::App, b"key", why),
            },
            Phase::Accepted(app) => match event.read() {
                Ok(read) => {
                    let ids = match read {
                        Some(value) => item::accepted(value.expose(), self.installations),
                        None => Some(Box::<[u64]>::default()),
                    };
                    let Some(ids) = ids else {
                        return self.stop(Why::Unexpected);
                    };
                    self.phase = Phase::Installation(ids);
                    Need::Send {
                        request: Request::Installation { id: self.installation.get() }.into(),
                        auth: app.auth(),
                    }
                }
                Err(why) => self.stop(why),
            },
            Phase::Installation(ids) => match reply(event) {
                Ok(Reply::Document(Document::Installation(installation)))
                    if installation.id == self.installation.get() =>
                {
                    self.verified(&ids)
                }
                Ok(Reply::Status(404)) => self.stop(Why::Missing),
                Ok(Reply::Status(status)) => self.stop(Why::Refused(status)),
                Ok(Reply::Unsigned) => plan::stop(self.forge, Stage::App, b"key", Why::KeyForm),
                Ok(Reply::Down) => self.stop(Why::Down),
                Ok(Reply::Empty | Reply::Document(_) | Reply::Rows(_)) => self.stop(Why::Unexpected),
                Err(why) => self.stop(why),
            },
            Phase::Written => match event.written() {
                Ok(()) => Need::Done(Outcome::Nothing),
                Err(why) => self.stop(why),
            },
            Phase::Ended => unreachable!("ended acceptance is not advanced"),
        }
    }
}
/// Consumes one event and emits its next need; ended machines are silent.
pub fn step(machine: &mut Accept, event: Event, out: &mut Queue<Need>) {
    let phase = core::mem::replace(&mut machine.phase, Phase::Ended);
    match phase {
        Phase::Ended => (),
        Phase::Begin | Phase::Id | Phase::Key(_) | Phase::Accepted(_) | Phase::Installation(_) | Phase::Written => {
            let need = machine.advance(phase, event);
            out.push(need);
        }
    }
}
