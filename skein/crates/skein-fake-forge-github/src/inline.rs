//! A lent domain and whole requests without io (skein/docs/design/testing/fakes/github.md, 1.6).
use crate::{
    server::{self, Event, Server},
    service::{Limits, Service},
};
use alloc::boxed::Box;
use skein_fake_forge_domain as domain;
use skein_lib::{
    Env, Intake, List, Queue, Time, Token, Wall,
    stream::{Down, Up},
};

// More events than one request's admission and continuations raise.
const EVENTS: u32 = 256;
// More timers than a fixture's queue and CI can have due at once.
const FIRES: u32 = 64;
const OWNER: Token = Token::new(77);

#[derive(Debug)]
pub struct Lent<'a> {
    pub domain: &'a mut domain::Domain,
    pub env: &'a Env<domain::Config>,
    /// `release` runs ahead of each call of the pass.
    pub hooks: bool,
}
/// One call at zero latency, as admitted, followed by reclamation.
pub fn call(
    domain: &mut domain::Domain,
    env: &Env<domain::Config>,
    event: domain::Event,
) -> Result<domain::api::Answer, domain::api::Error> {
    match reply(domain, env, event) {
        domain::Request::Reply { result, .. } => result,
        domain::Request::Hook { .. } => unreachable!("reply returns a call's reply"),
    }
}
fn reply(domain: &mut domain::Domain, env: &Env<domain::Config>, event: domain::Event) -> domain::Request {
    let mut out = Queue::uncapped();
    domain::step_admitted(domain, env, event, &mut out);
    for _ in 0..FIRES {
        if !out.is_empty() {
            break;
        }
        domain::fire(domain, env, &mut out);
    }
    let reply = out.pop().expect("zero-latency reply");
    match &reply {
        domain::Request::Reply { .. } => {}
        domain::Request::Hook { .. } => unreachable!("a hooked world sets Lent::hooks"),
    }
    assert!(out.is_empty(), "one reply per admitted call");
    domain.reclaim();
    reply
}
/// Fires due timers, hands their hooks to the service, then polls it.
pub fn release(service: &mut Service, domain: &mut domain::Domain, env: &Env<domain::Config>) {
    for _ in 0..FIRES {
        match domain.next_deadline() {
            Some(at) if at <= env.now => {}
            Some(_) | None => break,
        }
        let mut out = Queue::uncapped();
        domain::fire(domain, env, &mut out);
        for _ in 0..out.len() {
            let hook = out.pop().expect("queued hook");
            match &hook {
                domain::Request::Hook { .. } => {}
                domain::Request::Reply { .. } => unreachable!("direct calls settle synchronously"),
            }
            let post = service.hook(&hook, env.wall).expect("fixture hook fits");
            assert!(post.is_none(), "App hook has no endpoint");
        }
    }
    service.poll(env.wall);
}
/// Drives a connection until it waits on bytes or holds an untaken head.
pub fn pass(
    server: &mut Server,
    service: &mut Service,
    lent: &mut Lent<'_>,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    for _ in 0..EVENTS {
        if server.head_pending() {
            return;
        }
        if let Some(event) = above.pop() {
            match event {
                Event::Domain(event) => {
                    if lent.hooks {
                        release(service, lent.domain, lent.env);
                    }
                    let reply = reply(lent.domain, lent.env, event);
                    server::down(server, service, env, reply, lent.domain, above, below);
                }
                Event::Rate { key, limit, window } => {
                    let result = domain::rate(lent.domain, lent.env, key, limit, window);
                    server::admitted(server, service, env, result, above, below);
                }
                Event::Close | Event::Closed => {}
            }
        } else if server.has_work() {
            server::resume(server, service, env, above, below);
        } else {
            return;
        }
    }
}
#[derive(Debug)]
pub struct Inline {
    pub service: Service,
    pub domain: domain::Domain,
    pub config: domain::Config,
    pub limits: Limits,
}
impl Inline {
    #[must_use]
    pub fn new(service: Service, domain: domain::Domain, config: domain::Config, limits: Limits) -> Self {
        Self { service, domain, config, limits }
    }
    /// One whole request on a fresh API connection; the whole response's bytes.
    pub fn serve(&mut self, request: &[u8], wall: Wall) -> Box<[u8]> {
        self.exchange(Server::new(OWNER, &self.limits).expect("fixture server limits"), request, wall)
    }
    /// The same on the blob listener.
    pub fn blob(&mut self, request: &[u8], wall: Wall) -> Box<[u8]> {
        self.exchange(Server::blob(OWNER, &self.limits).expect("fixture server limits"), request, wall)
    }
    fn exchange(&mut self, mut server: Server, request: &[u8], wall: Wall) -> Box<[u8]> {
        let env = Env { now: Time::from_nanos(wall.as_nanos()), wall, limits: self.limits };
        let domain_env = env.with(self.config.clone());
        let mut lent = Lent { domain: &mut self.domain, env: &domain_env, hooks: false };
        let mut intake = Intake::with_capacity(
            u32::try_from(request.len())
                .expect("fixture request length")
                .max(skein_http::server::largest_read(&self.limits.http)),
        );
        intake.append(request).expect("whole request fits");
        let capacity = self.limits.http.response.checked_add(self.limits.response).expect("fixture response length");
        let mut output = List::with_capacity(capacity);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        server::start(&mut server, &mut self.service, &env, &mut above, &mut below);
        for _ in 0..EVENTS {
            pass(&mut server, &mut self.service, &mut lent, &env, &mut above, &mut below);
            drop(server.take_head());
            if let Some(down) = below.pop() {
                match down {
                    Down::Demand { read, room } => {
                        let up = if let Some(bytes) = intake.meet(read) {
                            Some(Up::Bytes(bytes))
                        } else if room > 0 {
                            Some(Up::Room)
                        } else {
                            None
                        };
                        if let Some(up) = up {
                            server::up(&mut server, &mut self.service, &env, up, &mut above, &mut below);
                        }
                    }
                    Down::Send(bytes) => {
                        for byte in &bytes {
                            output.push(*byte).expect("fixture response fits");
                        }
                    }
                    Down::Finish => return output.into_boxed(),
                }
            } else if !server.has_work() {
                return output.into_boxed();
            }
        }
        unreachable!("whole request settles within fixture event bound")
    }
}
