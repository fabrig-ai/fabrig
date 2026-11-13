//! One-shot push exchanges (temper/docs/design/domain/forge/attention.md, 7.2–7.3).
use super::fetch::{self, Transport, Web};
use alloc::boxed::Box;
use skein_io::{Event, Request};
use skein_lib::{Duration, Env, List, Map, Queue, Time, Token, Wall, bytes};
use temper_engine_domain as root;
use temper_engine_domain_forge::{self as forge, PushAnswer};

pub const PUSHES_IN_FLIGHT: u32 = 16;
pub(super) const PUSH: u64 = 1_u64 << 58;
const DEADLINE: Duration = Duration::from_secs(10);
const LIMITS: fetch::Limits = fetch::Limits {
    body: 0,
    http: skein_http::client::Limits { request: 8192, head: 8192, headers: 32, read: 8192, send: 8192 },
    tls: skein_tls::client::Limits { read: 16_384, send: 16_384, records: 131_072 },
};
struct Sending {
    push: Option<root::Push>,
    sealed: Option<skein_push::Sealed>,
    host: Box<[u8]>,
    port: u16,
    tls: bool,
    web: Option<Web>,
}
pub(crate) struct Runner {
    settings: Option<crate::settings::Push>,
    pub(crate) roots: Option<skein_tls::Config>,
    sending: Map<Token, Sending>,
    next: u64,
    answers: Queue<root::Event>,
}
impl Runner {
    pub(crate) fn new() -> Self {
        Self {
            settings: None,
            roots: None,
            sending: Map::with_capacity(PUSHES_IN_FLIGHT),
            next: 1,
            answers: Queue::uncapped(),
        }
    }
    pub(crate) fn configure(&mut self, settings: Option<crate::settings::Push>) {
        self.settings = settings;
    }
    pub(crate) fn send(&mut self, push: root::Push, now: Time, wall: Wall, io: &mut Queue<Request>) {
        let Some(settings) = &self.settings else { return };
        if self.sending.len() >= PUSHES_IN_FLIGHT {
            return;
        }
        let Some(endpoint) = skein_push::Endpoint::parse(&push.endpoint) else { return };
        let mut allowed = false;
        for origin in &settings.services {
            allowed |= origin.as_ref() == endpoint.origin();
        }
        if !allowed {
            return;
        }
        let Some((_, host, port, tls)) = crate::startup::sign_in_url(&push.endpoint) else { return };
        let host = bytes::copy_of(host);
        let subscription =
            skein_push::Subscription { endpoint: push.endpoint.clone(), p256dh: push.p256dh, auth: push.auth };
        let message = skein_push::Push {
            payload: &push.payload,
            ttl: 86_400,
            urgency: skein_push::Urgency::High,
            topic: Some(&push.topic),
        };
        let Ok(sealed) = skein_push::seal(&settings.key, &settings.subject, &subscription, &message, wall) else {
            self.answers.push(pushed(push, PushAnswer::Failed));
            return;
        };
        let token = Token::new(crate::tokens::SERVER | PUSH | self.next);
        self.next = self.next.checked_add(1).expect("push sequence");
        let ip = ip(&host);
        let sending = Sending { push: Some(push), sealed: Some(sealed), host, port, tls, web: None };
        assert!(self.sending.insert(token, sending).is_ok(), "admitted push");
        if let Some(ip) = ip {
            self.resolved(token, Some(core::net::SocketAddr::new(ip, port)), now, io);
        }
    }
    #[expect(clippy::type_complexity, reason = "the shell resolves a token's host and port")]
    pub(crate) fn unresolved(&self) -> Box<[(Token, Box<[u8]>, u16)]> {
        let mut names = List::with_capacity(PUSHES_IN_FLIGHT);
        for (token, sending) in &self.sending {
            if sending.web.is_none() {
                names.push((*token, sending.host.clone(), sending.port)).expect("admitted pushes");
            }
        }
        names.into_boxed()
    }
    #[expect(clippy::disallowed_methods, reason = "the transport validates a bounded configured TLS name")]
    pub(crate) fn resolved(
        &mut self,
        token: Token,
        address: Option<core::net::SocketAddr>,
        now: Time,
        io: &mut Queue<Request>,
    ) {
        let Some(mut sending) = self.sending.remove(&token) else { return };
        if sending.web.is_some() {
            assert!(self.sending.insert(token, sending).is_ok(), "removed push");
            return;
        }
        let transport = if sending.tls {
            match (&self.roots, core::str::from_utf8(&sending.host).ok().and_then(skein_tls::Name::new)) {
                (Some(trust), Some(server_name)) => Some(Transport::Tls { server_name, trust: trust.clone() }),
                (Some(_), None) | (None, _) => None,
            }
        } else {
            Some(Transport::Plaintext)
        };
        if let (Some(address), Some(transport)) = (address, transport)
            && address.port() == sending.port
            && (sending.tls || address.ip().is_loopback())
        {
            let sealed = sending.sealed.take().expect("one seal per push");
            sending.web = Some(Web::push(sealed, &transport, now.saturating_add(DEADLINE), &LIMITS));
            io.push(Request::Connect { owner: token, addr: skein_io::Addr::Ip(address) });
            assert!(self.sending.insert(token, sending).is_ok(), "removed push");
        } else {
            self.answers.push(pushed(sending.push.take().expect("unresolved push"), PushAnswer::Failed));
        }
    }
    pub(crate) fn up(&mut self, event: Event, now: Time, wall: Wall, io: &mut Queue<Request>) {
        if let Some(owner) = super::event_owner(&event)
            && let Some(sending) = self.sending.get_mut(&owner)
            && let Some(web) = &mut sending.web
        {
            web.up(&Env { now, wall, limits: LIMITS }, event, io);
        }
    }
    pub(crate) fn advance(
        &mut self,
        now: Time,
        wall: Wall,
        stopping: bool,
        events: &mut Queue<root::Event>,
        io: &mut Queue<Request>,
    ) {
        let env = Env { now, wall, limits: LIMITS };
        let mut keys = List::with_capacity(PUSHES_IN_FLIGHT);
        for (token, _) in &self.sending {
            keys.push(*token).expect("admitted pushes");
        }
        for token in &keys {
            let mut sending = self.sending.remove(token).expect("live push");
            if let Some(web) = &mut sending.web {
                if stopping {
                    web.close(&env, true, io);
                } else if now >= web.deadline {
                    web.transport_failed(&env);
                } else {
                    web.progress(&env, io);
                }
                let answer = match web.answer.take() {
                    Some(answer) => Some(classify(
                        answer.status,
                        u32::try_from(
                            answer.retry_after.as_nanos().checked_div(1_000_000_000).expect("second divisor"),
                        )
                        .ok(),
                    )),
                    None if web.failure.take().is_some() => Some(PushAnswer::Failed),
                    None => None,
                };
                if let Some(answer) = answer {
                    if let Some(push) = sending.push.take() {
                        self.answers.push(pushed(push, answer));
                    }
                    web.close(&env, true, io);
                }
                web.close_progress(&env, io);
                if !web.closed() {
                    assert!(self.sending.insert(*token, sending).is_ok(), "removed push");
                }
            } else if !stopping {
                assert!(self.sending.insert(*token, sending).is_ok(), "removed push");
            }
        }
        while let Some(answer) = self.answers.pop() {
            events.push(answer);
        }
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, sending) in &self.sending {
            if let Some(web) = &sending.web
                && !web.closing()
                && !web.closed()
            {
                next = skein_lib::earliest(next, Some(web.deadline));
            }
        }
        next
    }
    pub(crate) fn work_pending(&self) -> bool {
        if !self.answers.is_empty() {
            return true;
        }
        for (_, sending) in &self.sending {
            if let Some(web) = &sending.web
                && (web.has_work() || web.closed())
            {
                return true;
            }
        }
        false
    }
    pub(crate) fn closed(&self) -> bool {
        self.sending.is_empty() && self.answers.is_empty()
    }
}
#[expect(clippy::disallowed_methods, reason = "numeric destinations need no shell name resolution")]
fn ip(host: &[u8]) -> Option<core::net::IpAddr> {
    core::str::from_utf8(host).ok()?.parse().ok()
}
fn classify(status: u16, retry_after: Option<u32>) -> PushAnswer {
    match skein_push::answer(status, retry_after) {
        skein_push::Answer::Made => PushAnswer::Made,
        skein_push::Answer::Gone => PushAnswer::Gone,
        skein_push::Answer::TooLarge
        | skein_push::Answer::Later(_)
        | skein_push::Answer::Refused(_)
        | skein_push::Answer::Failed(_) => PushAnswer::Failed,
    }
}
fn pushed(push: root::Push, answer: PushAnswer) -> root::Event {
    root::Event::Forge(Box::new(forge::Event::Pushed {
        party: push.party,
        device: push.device,
        endpoint: push.endpoint,
        answer,
    }))
}
#[cfg(test)]
mod tests;
