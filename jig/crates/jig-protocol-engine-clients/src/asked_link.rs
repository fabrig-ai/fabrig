//! Volatile links for the asking person's party (protocol/client/sign-in.md, The asked link).
use jig_client_protocol_wire::Record as _;
use super::{Component, Env, Limits, Queue, ReplyTo, Request, Token, Waiting, core, people, routes, sign_in, w};
use crate::sign_in::{AskedLink, LinkKind, Past, Reservation};
use alloc::boxed::Box;
use skein_lib::{Duration, Map, Reader, Secret, Time, Wall};

#[derive(Debug)]
pub(super) struct State {
    // A reservation keeps its sign-in's place even while it has no live link.
    places: Map<u64, Option<AskedLink>>,
    reserved: Map<Token, AskedLink>,
}
impl State {
    pub(super) fn new(exchanges: u32) -> Self {
        Self { places: Map::with_capacity(super::approval::ASKED_LINKS), reserved: Map::with_capacity(exchanges) }
    }
    pub(super) fn capacity(&self) -> u32 {
        self.reserved.capacity()
    }
    fn release_place(&mut self, sign_in: u64) {
        for (_, link) in &self.reserved {
            if link.sign_in == sign_in {
                return;
            }
        }
        if let Some(None) = self.places.get(&sign_in) {
            self.places.remove(&sign_in);
        }
    }
    pub(super) fn take(&mut self, exchange: Token) -> Option<AskedLink> {
        let link = self.reserved.remove(&exchange)?;
        self.release_place(link.sign_in);
        Some(link)
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, link) in &self.places {
            if let Some(link) = link {
                next = Some(match next {
                    Some(next) => link.until.min(next),
                    None => link.until,
                });
            }
        }
        next
    }
}
impl Component {
    pub(super) fn asked_checked(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        cap: Wall,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        let mut reader = Reader::new(body);
        if w::LinkAsk::decode(&env.limits.wire, &mut reader).is_err() || !reader.is_empty() {
            if let Some(owner) = self.owner(exchange) {
                self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
            }
            return;
        }
        if self.stopping() {
            self.respond(exchange, routes::link_ask_answer(w::LinkAskAnswer::NotReady));
        } else if self.admitted() >= env.limits.exchanges {
            self.respond(exchange, routes::link_ask_answer(w::LinkAskAnswer::Busy));
        } else {
            self.pending.exchanges.insert(exchange, Waiting::LinkAskWhose { sign_in, cap }).expect("admitted ask");
            out.push(Request::Core(Box::new(core::Event::People(people::Event::Whose {
                reply_to: ReplyTo::new(exchange),
                sign_in,
            }))));
        }
    }
    pub(super) fn link_ask_whose(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        cap: Wall,
        reply: people::Reply,
    ) {
        self.expire_asked(env.now);
        let answer = if self.stopping() {
            w::LinkAskAnswer::NotReady
        } else {
            match reply {
                people::Reply::Signer(Some(signer)) => match signer.kind {
                    people::Kind::Service => w::LinkAskAnswer::Refused,
                    people::Kind::Person => {
                        if cap <= env.wall {
                            w::LinkAskAnswer::SignedOut
                        } else if !self.asked.places.contains_key(&sign_in)
                            && self.asked.places.len() == super::approval::ASKED_LINKS
                        {
                            w::LinkAskAnswer::Busy
                        } else {
                            let secret = sign_in::draw(&mut self.rng);
                            let expires =
                                Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.link_span.as_nanos()))
                                    .min(cap);
                            let until = env.now.saturating_add(Duration::from_nanos(
                                expires.as_nanos().saturating_sub(env.wall.as_nanos()),
                            ));
                            let link =
                                AskedLink { digest: sign_in::digest(secret.expose()), sign_in, cap, expires, until };
                            if let Some(Some(old)) =
                                self.asked.places.insert(sign_in, Some(link)).expect("ask has a place")
                            {
                                self.remember_digest(LinkKind::Asked, old.digest, Past::Expired);
                            }
                            w::LinkAskAnswer::Asked(
                                w::LinkAsked { secret, expires: expires.as_nanos() },
                            )
                        }
                    }
                },
                people::Reply::Signer(None) => w::LinkAskAnswer::SignedOut,
                people::Reply::Again(people::Again::NotReady) => w::LinkAskAnswer::NotReady,
                people::Reply::Again(people::Again::Busy | people::Again::Full { .. }) | people::Reply::Full { .. } => {
                    w::LinkAskAnswer::Busy
                }
                people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Outcome(_)
                | people::Reply::Refused(_) => w::LinkAskAnswer::Refused,
            }
        };
        self.respond(exchange, routes::link_ask_answer(answer));
    }
    // Returns whether this digest was handled, live or reserved.
    pub(super) fn asked_sign_in(
        &mut self,
        env: &Env<Limits>,
        connection: &mut crate::exchange::Exchange,
        digest: &Secret,
        out: &mut Queue<Request>,
    ) -> bool {
        for (_, link) in &self.asked.reserved {
            if sign_in::constant_time_eq(link.digest.expose(), digest.expose()) {
                connection.respond(crate::Reply::Record(routes::sign_in_busy()));
                return true;
            }
        }
        let mut found = None;
        for (sign_in, link) in &self.asked.places {
            if let Some(link) = link
                && sign_in::constant_time_eq(link.digest.expose(), digest.expose())
            {
                found = Some(*sign_in);
            }
        }
        let Some(sign_in) = found else { return false };
        if self.admitted() >= env.limits.exchanges {
            connection.respond(crate::Reply::Record(routes::sign_in_busy()));
            return true;
        }
        let link = self.asked.places.get_mut(&sign_in).expect("live place").take().expect("live link");
        let cap = link.cap;
        self.asked.reserved.insert(connection.name, link).expect("admitted reservation");
        self.pending.exchanges.insert(connection.name, Waiting::AskedWhose { cap }).expect("admitted sign-in");
        out.push(Request::Core(Box::new(core::Event::People(people::Event::Whose {
            reply_to: ReplyTo::new(connection.name),
            sign_in,
        }))));
        true
    }
    pub(super) fn asked_whose(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        cap: Wall,
        reply: people::Reply,
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.sign_in_reply(env, exchange, people::Reply::Again(people::Again::NotReady), None, false, out);
            return;
        }
        match reply {
            people::Reply::Signer(Some(signer)) if signer.kind == people::Kind::Person => {
                self.pending.exchanges.insert(exchange, Waiting::Core { held: None }).expect("reserved sign-in");
                out.push(Request::Core(Box::new(core::Event::SignIn {
                    reply_to: ReplyTo::new(exchange),
                    identity: signer.identity,
                    held: None,
                    until: Some(cap),
                })));
            }
            people::Reply::Again(_) | people::Reply::Full { .. } => {
                self.sign_in_reply(env, exchange, reply, None, false, out);
            }
            people::Reply::Signer(_)
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Refused(_) => {
                self.sign_in_reply(env, exchange, people::Reply::Refused(people::Refusal::Unknown), None, false, out);
            }
        }
    }
    pub(super) fn return_asked(&mut self, link: AskedLink) {
        if let Some(Some(_)) = self.asked.places.get(&link.sign_in) {
            self.remember_digest(LinkKind::Asked, link.digest, Past::Expired);
        } else {
            self.asked.places.insert(link.sign_in, Some(link)).expect("reservation kept its place");
        }
    }
    pub(super) fn expire_asked(&mut self, now: Time) -> bool {
        let mut any = false;
        for _ in 0..super::approval::ASKED_LINKS {
            let mut found = None;
            for (sign_in, link) in &self.asked.places {
                if let Some(link) = link
                    && link.until <= now
                {
                    found = Some(*sign_in);
                    break;
                }
            }
            let Some(sign_in) = found else { break };
            let link = self.asked.places.get_mut(&sign_in).expect("expired place").take().expect("expired link");
            self.remember_digest(LinkKind::Asked, link.digest, Past::Expired);
            self.asked.release_place(sign_in);
            any = true;
        }
        any
    }
}

impl Reservation {
    pub(super) fn printed(&self) -> bool {
        match self {
            Self::Startup(which, _) => *which == sign_in::Which::Printed,
            Self::Asked(_) => false,
        }
    }
    pub(super) fn expires(&self, env: &Env<Limits>) -> u64 {
        match self {
            Self::Startup(_, link) => {
                env.wall.as_nanos().saturating_add(link.until.as_nanos().saturating_sub(env.now.as_nanos()))
            }
            Self::Asked(link) => link.expires.as_nanos(),
        }
    }
}
