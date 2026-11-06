//! Public admission and refunds (jig/docs/design/protocol/client.md, 19).
use super::{
    Box, Component, Env, Kept, Limits, Map, Queue, ReplyTo, Request, SecretOwner, SecretsRequest, Settings, Time,
    Token, Unusable, core, sign_in, w,
};
use crate::{
    PublicRead, PublicReply,
    public::{Names, Of, PublicKind},
};
use jig_client_protocol_wire::{self as wire, public_v1};
use skein_lib::{List, Secret, Wall, Writer, length::len_u64};
#[path = "records.rs"]
mod records;

#[derive(Debug)]
enum Key {
    Loading,
    Keeping(Secret),
    Held(Names),
}
#[derive(Debug)]
struct Reservation {
    exchange: Option<Token>,
    kind: PublicKind,
}
#[derive(Debug)]
pub(super) struct State {
    key: Key,
    wire: Option<public_v1::Limits>,
    pending: Map<Token, Reservation>,
    reads: Bucket,
    bytes: Bucket,
}
#[derive(Debug)]
struct Bucket {
    available: u128,
    updated: Time,
}
const SECOND: u128 = 1_000_000_000;
impl Bucket {
    fn new(burst: u64, now: Time) -> Self {
        Self { available: u128::from(burst).saturating_mul(SECOND), updated: now }
    }
    fn refill(&mut self, now: Time, rate: u64, burst: u64) {
        let elapsed = now.saturating_since(self.updated).as_nanos();
        self.updated = self.updated.max(now);
        self.available = self
            .available
            .saturating_add(u128::from(elapsed).saturating_mul(u128::from(rate)))
            .min(u128::from(burst).saturating_mul(SECOND));
    }
    fn has(&self, amount: u64) -> bool {
        self.available >= u128::from(amount).saturating_mul(SECOND)
    }
    fn take(&mut self, amount: u64) {
        self.available =
            self.available.checked_sub(u128::from(amount).saturating_mul(SECOND)).expect("admitted bucket");
    }
    fn refund(&mut self, amount: u64, burst: u64) {
        self.available = self
            .available
            .saturating_add(u128::from(amount).saturating_mul(SECOND))
            .min(u128::from(burst).saturating_mul(SECOND));
    }
}
impl State {
    pub(super) fn new(settings: &Settings, env: &Env<Limits>) -> Result<Self, Unusable> {
        let limits = env.limits.public;
        let wire = wire::public_limits(&settings.public_bounds);
        if limits.max_age > 60 {
            return Err(Unusable::Limits);
        }
        if limits.on() {
            if wire.is_none()
                || limits.reads == 0
                || limits.rate_burst == 0
                || limits.bytes == 0
                || limits.answer_bytes == 0
                || u64::from(limits.answer_bytes) > limits.bytes_burst
            {
                return Err(Unusable::Limits);
            }
            for kind in &settings.public {
                if kind.bound > limits.answer_bytes {
                    return Err(Unusable::Limits);
                }
            }
        }
        for kind in &settings.public {
            if kind.first > kind.last || not_served(*kind, &env.limits.wire).is_none() {
                return Err(Unusable::Limits);
            }
        }
        Ok(Self {
            key: Key::Loading,
            wire,
            pending: Map::with_capacity(limits.reads),
            reads: Bucket::new(u64::from(limits.rate_burst), env.now),
            bytes: Bucket::new(limits.bytes_burst, env.now),
        })
    }
    #[cfg(test)]
    pub(super) fn available_bytes(&self) -> u64 {
        u64::try_from(self.bytes.available.div_euclid(SECOND)).expect("bucket burst fits u64")
    }
    pub(super) fn settled(&self) -> bool {
        let held = match self.key {
            Key::Loading | Key::Keeping(_) => false,
            Key::Held(_) => true,
        };
        held && self.pending.is_empty()
    }
    pub(super) fn abort(&mut self, burst: u64) {
        for (_, reservation) in &self.pending {
            self.bytes.refund(u64::from(reservation.kind.bound), burst);
        }
        self.pending = Map::with_capacity(self.pending.capacity());
    }
    pub(super) fn gone(&mut self, exchange: Token) {
        let mut found = None;
        for (name, reservation) in &self.pending {
            if reservation.exchange == Some(exchange) {
                found = Some(*name);
                break;
            }
        }
        if let Some(name) = found {
            self.pending.get_mut(&name).expect("public exchange").exchange = None;
        }
    }
    pub(super) fn names(&self) -> Option<&Names> {
        match &self.key {
            Key::Held(names) => Some(names),
            Key::Loading | Key::Keeping(_) => None,
        }
    }
}
impl Component {
    pub(super) fn public_key(&mut self, value: Option<Kept>, out: &mut Queue<Request>) {
        match &self.public.key {
            Key::Loading => (),
            Key::Keeping(_) | Key::Held(_) => return,
        }
        match value {
            Some(value) => {
                if value.digest.len() != 32 {
                    self.abort();
                    return;
                }
                self.public.key = Key::Held(Names::new(&value.digest));
            }
            None => {
                let key = sign_in::draw(&mut self.rng);
                self.public.key = Key::Keeping(key.clone());
                out.push(Request::Secrets(SecretsRequest::Keep {
                    owner: SecretOwner::PublicNames,
                    value: Kept { digest: key, expires: Wall::from_nanos(u64::MAX) },
                }));
            }
        }
    }
    pub(super) fn public_kept(&mut self) {
        match &self.public.key {
            Key::Keeping(key) => self.public.key = Key::Held(Names::new(key)),
            Key::Loading | Key::Held(_) => (),
        }
    }
    pub(super) fn public_read(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        read: PublicRead,
        out: &mut Queue<Request>,
    ) {
        let mut declared = None;
        for kind in &self.settings.public {
            if kind.owner == read.owner && kind.kind == read.kind {
                declared = Some(*kind);
                break;
            }
        }
        let Some(kind) = declared else {
            public_reply(exchange, PublicReply::Unknown, out);
            return;
        };
        let ids = if kind.owner == 0 && kind.kind == wire::PUBLIC_PROJECTS {
            0
        } else if kind.task {
            2
        } else {
            1
        };
        if read.ids.len() != ids {
            public_reply(exchange, PublicReply::Unknown, out);
            return;
        }
        if read.version < kind.first || read.version > kind.last {
            let record = not_served(kind, &env.limits.wire).expect("public versions checked at start");
            public_reply(exchange, PublicReply::NotServed { record }, out);
            return;
        }
        let limits = env.limits.public;
        self.public.reads.refill(env.now, u64::from(limits.rate), u64::from(limits.rate_burst));
        self.public.bytes.refill(env.now, limits.bytes, limits.bytes_burst);
        if !limits.on()
            || self.stopping()
            || self.public.names().is_none()
            || self.public.pending.len() >= limits.reads
            || !self.public.reads.has(1)
            || !self.public.bytes.has(u64::from(kind.bound))
        {
            public_reply(exchange, PublicReply::Busy, out);
            return;
        }
        self.public.reads.take(1);
        self.public.bytes.take(u64::from(kind.bound));
        let names = self.public.names().expect("key held at admission");
        let project = match read.ids.first() {
            Some(id) => Some(names.number(*id, Of::Project)),
            None => None,
        };
        let task = match read.ids.get(1) {
            Some(id) => Some(names.number(*id, Of::Task)),
            None => None,
        };
        let read = core::PublicRead { owner: read.owner, version: read.version, kind: read.kind, project, task };
        let name = self.name();
        self.public.pending.insert(name, Reservation { exchange: Some(exchange), kind }).expect("public room admitted");
        out.push(Request::Core(Box::new(core::Event::PublicRead { reply: ReplyTo::new(name), read })));
    }
    pub(super) fn public_answer(
        &mut self,
        env: &Env<Limits>,
        name: Token,
        answer: core::PublicAnswer,
        out: &mut Queue<Request>,
    ) {
        let Some(reservation) = self.public.pending.remove(&name) else {
            return;
        };
        let mut sent = 0;
        let reply = match answer {
            core::PublicAnswer::Unknown => PublicReply::Unknown,
            core::PublicAnswer::Busy => PublicReply::Busy,
            core::PublicAnswer::Document { decided, document } => {
                let bytes = match (self.public.names(), self.public.wire.as_ref()) {
                    (Some(names), Some(limits)) => records::encode(
                        &document,
                        names,
                        limits,
                        env.limits.public.max_age,
                        reservation.kind,
                        env.limits.public.answer_bytes,
                    ),
                    (Some(_), None) | (None, Some(_) | None) => None,
                };
                match bytes {
                    Some(bytes) => {
                        if reservation.exchange.is_some() {
                            sent = len_u64(bytes.len());
                        }
                        PublicReply::Document { bytes, decided }
                    }
                    None => PublicReply::Unknown,
                }
            }
        };
        self.public.bytes.refill(env.now, env.limits.public.bytes, env.limits.public.bytes_burst);
        self.public.bytes.refund(u64::from(reservation.kind.bound).saturating_sub(sent), env.limits.public.bytes_burst);
        if let Some(exchange) = reservation.exchange {
            public_reply(exchange, reply, out);
        }
    }
}
fn public_reply(exchange: Token, reply: PublicReply, out: &mut Queue<Request>) {
    out.push(Request::Transport(crate::ToTransport::Public { exchange, reply }));
}
fn not_served(kind: PublicKind, limits: &w::Limits) -> Option<Box<[u8]>> {
    let family = kind.owner.checked_sub(1);
    let owner = match family {
        None => w::Owner::Jig,
        Some(number) => w::Owner::Family(w::Family::new(limits, w::FamilyParts { number }).ok()?),
    };
    // The refusal is the ordinary served envelope, narrowed to this public kind's versions.
    let value = w::NotServed::new(
        limits,
        w::NotServedParts {
            owner: owner.clone(),
            served: {
                let mut ranges = List::with_capacity(1);
                ranges
                    .push(
                        w::Served::new(
                            limits,
                            w::ServedParts { owner, first: u32::from(kind.first), last: u32::from(kind.last) },
                        )
                        .ok()?,
                    )
                    .expect("one range");
                ranges
            },
        },
    )
    .ok()?;
    let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut out).expect("measured not served");
    Some(out.finish())
}
