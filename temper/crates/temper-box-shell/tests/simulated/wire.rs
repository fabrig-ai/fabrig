//! Scripted HTTP clients beside one fake host. The world turns the host and drains its writers.
use crate::{
    CHECKPOINT, SEED,
    disk::{Capture, File, Tree},
};
use serde_json::Value;
use skein_io::kernel::{Complete, Done, Exit, Family, Fd, Op, ServiceSignal, Submit};
use skein_lib::{Queue, Time, Token};
use skein_shell::Host as _;
use skein_sim::{Config, Faults, Pid};
use skein_sim_world::{Turns, World};
use std::net::SocketAddr;
use temper_box_shell::fake::{self, Arguments, Host};

const TURNS: usize = 512;
const CLIENT_OP: Token = Token::new(1);
type Fake = Host<File>;

pub struct Running {
    world: World<Fake>,
    arguments: Arguments,
    stdout: Capture,
    stderr: Capture,
    pub address: SocketAddr,
    pub control: Option<SocketAddr>,
    before_reply: BeforeReply,
}
#[derive(Default)]
struct BeforeReply(Option<Box<dyn FnOnce()>>);
impl Turns<Fake> for BeforeReply {
    fn faults(&mut self, _at: usize, _now: Time, batch: &Queue<Submit>) -> Option<Faults> {
        if batch.iter().any(|submit| matches!(submit.kind, Op::Send { .. }))
            && let Some(check) = self.0.take()
        {
            check();
        }
        None
    }
}
impl Running {
    pub fn start(arguments: Arguments, tree: &mut Tree) -> Self {
        let mut world = World::new(SEED, Config::calm());
        let (stdout, stderr) = (Capture::default(), Capture::default());
        let host = world.spawn_signals(|signal| {
            fake::start_host(&arguments, tree, Box::new([]), signal, Box::new(stdout.clone()), Box::new(stderr.clone()))
                .expect("admit fake host")
        });
        assert_eq!(host, 0);
        for _ in 0..TURNS {
            world.step();
            let ready = stdout.bytes();
            if ready.ends_with(b"\n") {
                let ready = temper_box_kit::fake_readiness::read_with(&mut ready.as_slice(), &arguments, |_| {
                    panic!("fake ended before ready")
                })
                .expect("fake ready");
                if let Some(path) = &arguments.observations {
                    assert_eq!(tree.bytes(path), CHECKPOINT, "no seed observations");
                }
                return Self {
                    world,
                    arguments,
                    stdout,
                    stderr,
                    address: ready.address,
                    control: ready.control,
                    before_reply: BeforeReply::default(),
                };
            }
            if !world.busy()
                && let Some(next) = world.next_due()
            {
                world.advance(next);
            }
        }
        panic!("the fake did not become ready\n{}\n{}", String::from_utf8_lossy(&stderr.bytes()), world.tail());
    }
    pub fn now(&self) -> Time {
        self.world.now()
    }
    pub fn advance_to(&mut self, at: Time) {
        for _ in 0..TURNS {
            self.world.step_with(&mut self.before_reply);
            if !self.world.busy() {
                if self.now() == at {
                    return;
                }
                self.world.advance(self.world.next_due().map_or(at, |next| next.min(at)));
            }
        }
        panic!("the fake did not settle at {at:?}\n{}", self.world.tail());
    }
    pub fn before_reply(&mut self, check: impl FnOnce() + 'static) {
        assert!(self.before_reply.0.is_none());
        self.before_reply.0 = Some(Box::new(check));
    }
    fn submit(&mut self, client: Pid, kind: Op) {
        let mut batch = Queue::with_capacity(1);
        batch.push(Submit { op: CLIENT_OP, kind });
        self.world.parts().0.submit(client, &mut batch);
    }
    fn poll(&mut self, client: Pid) -> Option<Complete> {
        let mut done = Queue::with_capacity(1);
        self.world.parts().0.reap(client, &mut done);
        done.pop()
    }
    fn complete(&mut self, client: Pid) -> Complete {
        for _ in 0..TURNS {
            if let Some(done) = self.poll(client) {
                return done;
            }
            self.world.step_with(&mut self.before_reply);
            if let Some(done) = self.poll(client) {
                return done;
            }
            if !self.world.busy() {
                let next = self
                    .world
                    .next_due()
                    .unwrap_or_else(|| panic!("client is waiting, nothing due\n{}", self.world.tail()));
                self.world.advance(next);
            }
        }
        panic!("client operation did not finish\n{}", self.world.tail());
    }
    fn call(&mut self, client: Pid, kind: Op) -> Done {
        self.submit(client, kind);
        self.complete(client).result.expect("client operation")
    }
    pub fn open(&mut self, address: SocketAddr, request: &[u8]) -> Socket {
        let client = self.world.parts().0.spawn_process();
        let Done::Fd(fd) = self.call(client, Op::Socket { family: Family::Ipv4 }) else { panic!("socket fd") };
        assert_eq!(self.call(client, Op::Connect { fd, addr: address }), Done::Nothing);
        let mut from = 0;
        while usize::try_from(from).expect("request offset") < request.len() {
            let Done::Count(sent) = self.call(client, Op::Send { fd, bytes: request.into(), from }) else {
                panic!("send count");
            };
            assert_ne!(sent, 0);
            from += sent;
        }
        Socket { client, fd, reading: false }
    }
    pub fn request(&mut self, method: &str, target: &str, auth: &str, body: &[u8]) -> Reply {
        self.request_at(self.address, method, target, auth, body)
    }
    pub fn request_at(&mut self, address: SocketAddr, method: &str, target: &str, auth: &str, body: &[u8]) -> Reply {
        let mut socket = self.open(address, &http(method, target, auth, body));
        let reply = socket.read_to_end(self);
        socket.close(self);
        assert!(self.before_reply.0.is_none(), "the reply reached the wire");
        Reply(reply)
    }
    pub fn stop(mut self, tree: &mut Tree) {
        let ready = self.control.map_or_else(
            || format!("ready {}\n", self.address),
            |control| format!("ready {} {control}\n", self.address),
        );
        self.world.signal(0, ServiceSignal::Terminate);
        for _ in 0..TURNS {
            self.world.step();
            if self.world.settled() {
                let host = self.world.finish().procs.pop().expect("fake host");
                let exit = host.exit().expect("settled fake exit");
                assert_eq!(exit, Exit::Code(0));
                assert_eq!(host.finish(&self.arguments, tree, exit, || 1).expect("finish fake"), 0);
                assert_eq!(self.stdout.bytes(), ready.as_bytes(), "stdout holds only the ready line");
                assert_eq!(self.stderr.bytes(), b"", "empty log after the orderly stop");
                if let Some(path) = self.arguments.observations {
                    assert!(tree.bytes(&path).ends_with(CHECKPOINT), "final zero-loss checkpoint");
                }
                return;
            }
            if !self.world.busy() {
                let next =
                    self.world.next_due().unwrap_or_else(|| panic!("fake idle during stop\n{}", self.world.tail()));
                self.world.advance(next);
            }
        }
        panic!("fake did not stop\n{}", self.world.tail());
    }
}

pub struct Socket {
    client: Pid,
    fd: Fd,
    reading: bool,
}
impl Socket {
    fn arm(&mut self, run: &mut Running) {
        if !self.reading {
            run.submit(self.client, Op::Recv { fd: self.fd, buf: Box::new([0; 4096]) });
            self.reading = true;
        }
    }
    fn received(&mut self, done: Complete) -> Vec<u8> {
        self.reading = false;
        let (Op::Recv { buf, .. }, Done::Count(n)) = (done.kind, done.result.expect("receive simulated bytes")) else {
            panic!("receive count");
        };
        buf[..usize::try_from(n).expect("receive bound")].to_vec()
    }
    pub fn read(&mut self, run: &mut Running) -> Vec<u8> {
        self.arm(run);
        self.received(run.complete(self.client))
    }
    /// Reads what can reach the client without advancing time; None means the receive is still in flight.
    pub fn available(&mut self, run: &mut Running) -> Option<Vec<u8>> {
        self.arm(run);
        run.advance_to(run.now());
        run.poll(self.client).map(|done| self.received(done))
    }
    pub fn read_to_end(&mut self, run: &mut Running) -> Vec<u8> {
        let mut output = Vec::new();
        for _ in 0..TURNS {
            let bytes = self.read(run);
            if bytes.is_empty() {
                return output;
            }
            output.extend(bytes);
        }
        panic!("reply did not end: {}", String::from_utf8_lossy(&output));
    }
    pub fn close(self, run: &mut Running) {
        assert!(!self.reading);
        assert_eq!(run.call(self.client, Op::Close { fd: self.fd }), Done::Nothing);
        let sim = run.world.parts().0;
        sim.assert_quiescent(self.client);
        sim.assert_no_open_fds(self.client);
    }
}

pub fn http(method: &str, target: &str, auth: &str, body: &[u8]) -> Vec<u8> {
    let mut request = format!("{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n", body.len()).into_bytes();
    if !auth.is_empty() {
        request.extend_from_slice(format!("Authorization: {auth}\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    request
}
pub struct Reply(pub Vec<u8>);
impl Reply {
    pub fn status(&self, expected: u16) {
        assert!(self.0.starts_with(format!("HTTP/1.1 {expected} ").as_bytes()), "{}", self.text());
    }
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.0).expect("reply text")
    }
    pub fn json(&self) -> Value {
        let (_, body) = self.text().split_once("\r\n\r\n").expect("HTTP head and body");
        serde_json::from_str(body).expect("JSON reply")
    }
}
