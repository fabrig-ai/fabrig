//! The plaintext push peer (testing/fakes/push.md, 4), over the shared HTTP connection core.
use crate::{
    Error, Limits, Transport,
    face::{Face, LISTENER},
    transport::Wire,
};
use skein_fake_forge_http::connection::{self as core, Connection as Http, State, Turn};
use skein_fake_push::{self as domain, Received, Script};
use skein_http::server as http;
use skein_io::{self as io, kernel};
use skein_lib::{
    Env, Queue, Rng, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use skein_shell::Host;
use std::net::SocketAddr;

const HTTP: http::Limits = http::Limits { head: 4096, headers: 32, body: 8192, read: 1024, response: 512, send: 1024 };
struct Connection {
    owner: Token,
    wire: Wire,
    http: Http,
    plain: Queue<Up>,
    below: Queue<Down>,
    request: Option<http::Call>,
    body: Vec<u8>,
}
/// Per-mille connection faults, drawn from the world's seed as the forge peer's are.
#[derive(Clone, Copy, Debug)]
pub struct Faults {
    pub reset: u32,
    pub drop: u32,
}

/// One bounded listener. Its origin is the address reported by the successful listen.
pub struct Peer {
    face: Face,
    key: [u8; 65],
    service: Option<domain::Service>,
    scripts: Vec<(Box<[u8]>, Script)>,
    connections: Vec<Connection>,
    next: u64,
    rng: Rng,
    faults: Faults,
}
impl Peer {
    pub fn new(address: SocketAddr, limits: Limits, key: [u8; 65], seed: u64) -> Result<Self, Error> {
        if http::largest_read(&HTTP) > limits.io.intake || http::largest_room(&HTTP) > limits.io.output {
            return Err(Error::Limits);
        }
        Ok(Self {
            face: Face::new(address, limits)?,
            key,
            service: None,
            scripts: Vec::new(),
            connections: Vec::new(),
            next: 1,
            rng: Rng::new(seed),
            faults: Faults { reset: 0, drop: 0 },
        })
    }
    pub fn adopt_listener(&mut self, fd: kernel::Fd, address: SocketAddr) -> Result<(), kernel::Fd> {
        self.face.io.adopt_listener(fd, address).map(|_| ())
    }
    #[must_use]
    pub fn address(&self) -> Option<SocketAddr> {
        self.face.address
    }
    #[must_use]
    pub fn received(&self) -> &[Received] {
        self.service.as_ref().map_or(&[], domain::Service::received)
    }
    pub fn script(&mut self, target: Box<[u8]>, script: Script) {
        if let Some(service) = &mut self.service {
            service.script(target, script);
        } else {
            self.scripts.push((target, script));
        }
    }
    /// Selects connection faults; each chance is in thousandths, at most 1,000.
    pub fn faults(&mut self, faults: Faults) -> Result<(), Error> {
        if faults.reset > 1000 || faults.drop > 1000 {
            return Err(Error::Limits);
        }
        self.faults = faults;
        Ok(())
    }
    pub fn shutdown(&mut self) {
        self.face.shutdown();
    }
    fn event(&mut self, event: io::Event, env: &Env<http::Limits>) {
        match event {
            io::Event::Listening { owner, listener, addr } => {
                assert_eq!(owner, LISTENER);
                self.face.listener = Some(listener);
                let address = addr.socket().expect("loopback address");
                self.face.address = Some(address);
                let mut service =
                    domain::Service::new(format!("http://{address}").into_bytes().into_boxed_slice(), self.key);
                for (target, script) in self.scripts.drain(..) {
                    service.script(target, script);
                }
                self.service = Some(service);
            }
            io::Event::Accepted { owner, socket, .. } => {
                assert_eq!(owner, LISTENER);
                if self.face.stopping || self.connections.len() == skein_lib::usize_of(self.face.limits.connections) {
                    self.face.requests.push(io::Request::Reject { socket });
                    return;
                }
                let owner = Token::new(self.next);
                self.next =
                    self.next.checked_add(1).filter(|next| *next < u64::MAX).expect("finite connection identities");
                self.face.requests.push(io::Request::Bind { socket, owner });
                let mut connection = Connection {
                    owner,
                    wire: Wire::new(socket, Transport::Plaintext, self.face.limits),
                    http: Http::new(&HTTP),
                    plain: Queue::with_capacity(self.face.limits.queue),
                    below: Queue::uncapped(),
                    request: None,
                    body: Vec::new(),
                };
                if self.rng.chance(self.faults.reset) {
                    core::close(&mut connection.http, env, &mut connection.below);
                } else {
                    core::read_next(&mut connection.http);
                }
                self.connections.push(connection);
            }
            io::Event::Stream { owner, up } => {
                if let Some(c) = self.connections.iter_mut().find(|c| c.owner == owner) {
                    c.wire.up(up, &mut c.plain, &mut self.face.requests);
                }
            }
            io::Event::Output { owner, up } => {
                if let Some(c) = self.connections.iter_mut().find(|c| c.owner == owner) {
                    c.wire.output(up, &mut c.plain, &mut self.face.requests);
                }
            }
            io::Event::Closed { owner } => {
                if let Some(index) = self.connections.iter().position(|c| c.owner == owner) {
                    let mut c = self.connections.remove(index);
                    core::close(&mut c.http, env, &mut c.below);
                }
            }
            io::Event::Failed { owner, .. } => {
                assert_ne!(owner, LISTENER, "the push listener binds");
                if let Some(c) = self.connections.iter_mut().find(|c| c.owner == owner) {
                    core::close(&mut c.http, env, &mut c.below);
                }
            }
            io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => unreachable!("the push peer listens and serves streams"),
        }
    }
    fn progress(&mut self, env: &Env<http::Limits>) {
        for c in &mut self.connections {
            if let Some(up) = c.plain.pop() {
                core::up(&mut c.http, env, up, &mut c.below);
            }
            if let Some(turn) = core::turn(&mut c.http, &HTTP) {
                match turn {
                    Turn::Call(call) => {
                        c.http.take_head();
                        c.request = Some(call);
                        c.body.clear();
                        demand(c);
                    }
                    Turn::Body(Up::Bytes(bytes)) => {
                        assert!(c.body.len().saturating_add(bytes.len()) <= 8192, "HTTP admits the body");
                        c.body.extend_from_slice(&bytes);
                        demand(c);
                    }
                    Turn::Body(Up::End) => {
                        let call = c.request.take().expect("request head before body");
                        let request = domain::Request {
                            method: Box::from(call.method.as_bytes()),
                            target: call.target,
                            headers: call.headers,
                            body: std::mem::take(&mut c.body).into_boxed_slice(),
                        };
                        let taken = self.service.as_mut().expect("listening before requests").take(request, env.wall);
                        if self.rng.chance(self.faults.drop) {
                            core::close(&mut c.http, env, &mut c.below);
                        } else {
                            core::respond(
                                &mut c.http,
                                &HTTP,
                                core::Reply {
                                    status: taken.status,
                                    headers: taken.headers,
                                    body: Box::new([]),
                                    bodyless: false,
                                },
                            );
                        }
                    }
                    Turn::Done(http::Reuse::Keep) => {
                        if self.rng.chance(self.faults.reset) {
                            core::close(&mut c.http, env, &mut c.below);
                        } else {
                            core::read_next(&mut c.http);
                        }
                    }
                    Turn::Done(http::Reuse::Close) | Turn::Failed | Turn::Body(Up::Failed(_) | Up::Room) => {
                        core::close(&mut c.http, env, &mut c.below);
                    }
                    Turn::Handled => {}
                }
            }
            core::request(&mut c.http, env, &mut c.below);
            if let Some(down) = c.below.pop() {
                c.wire.down(down, &mut c.plain, &mut self.face.requests);
            }
            c.wire.pump(&mut c.plain, &mut self.face.requests);
            if self.face.stopping
                && c.http.is_settled()
                && !c.http.has_work()
                && c.below.is_empty()
                && c.plain.is_empty()
            {
                core::close(&mut c.http, env, &mut c.below);
            }
            if c.http.state == State::Closing && c.below.is_empty() {
                c.wire.close(&mut self.face.requests);
            }
        }
    }
}
fn demand(c: &mut Connection) {
    c.http.requests.push(http::Request::Body(Down::Demand { read: Read::Arrived { max: HTTP.read }, room: 0 }));
}
impl Host for Peer {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: HTTP };
        self.face.up(now, wall);
        for _ in 0..self.face.limits.queue {
            let Some(event) = self.face.events.pop() else { break };
            self.event(event, &env);
        }
        self.face.close_listener();
        self.progress(&env);
        self.face.down(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.face.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.face.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        self.face.work_pending()
            || self.connections.iter().any(|c| {
                c.http.has_work()
                    || !c.plain.is_empty()
                    || !c.below.is_empty()
                    || c.wire.work_pending()
                    || (self.face.stopping && c.http.is_settled())
            })
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.face.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.face.is_empty() && self.connections.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.face.limits.io).expect("validated io operations")
    }
}
impl crate::Listening for Peer {
    fn address(&self) -> SocketAddr {
        self.address().expect("the push peer processed listening")
    }
    fn listening_address(&self) -> Option<SocketAddr> {
        self.address()
    }
    fn shutdown(&mut self) {
        self.shutdown();
    }
}
