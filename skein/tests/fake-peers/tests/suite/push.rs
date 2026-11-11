//! A sealed push across the simulator's sockets (testing/fakes/push.md, 4).
use skein_fake_peers::{Limits, Listening, push::Peer};
use skein_fake_peers_tests::call;
use skein_fake_push::Script;
use skein_io::kernel::{Complete, Done, Family, Fd, Op, Submit};
use skein_lib::{Duration, Queue, Token};
use skein_push::{Push, Subscription, Urgency, Vapid};
use skein_shell::Host;
use skein_sim::{Pid, Sim};
const PASSES: u32 = 1000;
// RFC 8291 Appendix A's public test key in OpenSSL PKCS#8 form.
const PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgyfWPiYE+n46HLnH0KqZOF1fJJU3MYrct3AELtAQ+oRyhRANCAAT+M/SrDepxkU21WCP3O1SUj0EwbZIHMtu5pZpTKGSCIA5Zent7wmC6HCJ5mFgJkuk5cwAvMBKiiujwa7t45ewP\n-----END PRIVATE KEY-----\n";
struct World {
    peer: Peer,
    sim: Sim,
    server: Pid,
    client: Pid,
    serial: u64,
    passes: u32,
}
impl World {
    fn new(public: [u8; 65]) -> Self {
        let limits = Limits {
            io: skein_io::Limits { sockets: 5, retry: Duration::from_millis(1), ..skein_io::Limits::TEST },
            connections: 2,
            queue: 64,
            plaintext: 32768,
            ciphertext: 32768,
            observations: 0,
            observation_bytes: 0,
        };
        let peer = Peer::new("127.0.0.1:0".parse().expect("loopback"), limits, public, 23).expect("peer bounds");
        let mut sim = Sim::new(23, skein_sim::Config::calm());
        let server = sim.spawn_process();
        let client = sim.spawn_process();
        Self { peer, sim, server, client, serial: 10, passes: 0 }
    }
    fn pass(&mut self) {
        self.passes += 1;
        assert!(self.passes <= PASSES, "push world settles within its pass bound");
        self.sim.reap(self.server, self.peer.completions());
        self.peer.iterate(self.sim.now(), self.sim.wall());
        self.sim.submit(self.server, self.peer.submissions());
        if !self.peer.work_pending(self.sim.now())
            && let Some(next) = [self.sim.next_due(), self.peer.next_deadline()].into_iter().flatten().min()
        {
            self.sim.advance_to(next);
        }
    }
    fn client(&mut self, kind: Op) -> Complete {
        self.serial += 1;
        let op = Token::new(self.serial);
        let mut out = Queue::with_capacity(1);
        let mut input = Queue::with_capacity(1);
        out.push(Submit { op, kind });
        self.sim.submit(self.client, &mut out);
        loop {
            self.sim.reap(self.client, &mut input);
            if let Some(done) = input.pop() {
                assert_eq!(done.op, op);
                return done;
            }
            self.pass();
        }
    }
    fn connect(&mut self) -> Fd {
        let Done::Fd(fd) = call(&mut self.sim, self.client, 1, Op::Socket { family: Family::Ipv4 }) else {
            panic!("socket")
        };
        assert_eq!(
            call(&mut self.sim, self.client, 2, Op::Connect { fd, addr: Listening::address(&self.peer) }),
            Done::Nothing
        );
        fd
    }
    fn exchange(&mut self, request: &[u8]) -> Vec<u8> {
        let fd = self.connect();
        self.send(fd, request);
        self.response(fd)
    }
    fn response(&mut self, fd: Fd) -> Vec<u8> {
        let mut response = Vec::new();
        loop {
            let complete = self.client(Op::Recv { fd, buf: Box::new([0; 1024]) });
            let Ok(Done::Count(count)) = complete.result else { panic!("recv {complete:?}") };
            if count == 0 {
                break;
            }
            let Op::Recv { buf, .. } = complete.kind else { panic!("buffer") };
            response.extend_from_slice(&buf[..usize::try_from(count).expect("count")]);
        }
        assert_eq!(call(&mut self.sim, self.client, 3, Op::Close { fd }), Done::Nothing);
        response
    }
    fn send(&mut self, fd: Fd, bytes: &[u8]) {
        let mut from = 0;
        while usize::try_from(from).expect("offset") < bytes.len() {
            let complete = self.client(Op::Send { fd, bytes: Box::from(bytes), from });
            let Ok(Done::Count(sent)) = complete.result else { panic!("send {complete:?}") };
            assert!(sent > 0);
            from += sent;
        }
    }
    fn settle(&mut self) {
        Listening::shutdown(&mut self.peer);
        while !self.peer.is_empty() {
            self.pass();
        }
        for pid in [self.server, self.client] {
            self.sim.assert_quiescent(pid);
            self.sim.assert_no_open_fds(pid);
        }
    }
}
fn request(world: &World, vapid: &Vapid) -> Vec<u8> {
    let origin = format!("http://{}", Listening::address(&world.peer));
    let subscription = Subscription {
        endpoint: format!("{origin}/push/alice").into_bytes().into_boxed_slice(),
        p256dh: *vapid.public(),
        auth: [7; 16],
    };
    let sealed = skein_push::seal(
        vapid,
        b"mailto:p@example.test",
        &subscription,
        &Push { payload: b"hello", ttl: 60, urgency: Urgency::Low, topic: Some(b"updates") },
        world.sim.wall(),
    )
    .expect("seal");
    let mut request = format!(
        "POST /push/alice HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        Listening::address(&world.peer),
        sealed.body.len()
    )
    .into_bytes();
    for (name, value) in sealed.headers {
        request.extend_from_slice(&name);
        request.extend_from_slice(b": ");
        request.extend_from_slice(&value);
        request.extend_from_slice(b"\r\n");
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(&sealed.body);
    request
}
#[test]
fn pushes_are_recorded_over_sockets_and_gone_is_not() {
    let vapid = Vapid::from_pkcs8_pem(PEM).expect("RFC test key");
    let mut world = World::new(*vapid.public());
    while world.peer.listening_address().is_none() {
        world.pass();
    }
    let request = request(&world, &vapid);
    let response = world.exchange(&request);
    assert!(response.starts_with(b"HTTP/1.1 201 "), "{response:?}");
    let records = world.peer.received();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target.as_ref(), b"/push/alice");
    assert_eq!(records[0].topic.as_deref(), Some(b"updates".as_slice()));
    assert_eq!(records[0].length, 108);
    world.peer.script(Box::from(b"/push/alice".as_slice()), Script::Gone);
    let response = world.exchange(&request);
    assert!(response.starts_with(b"HTTP/1.1 410 "), "{response:?}");
    assert_eq!(world.peer.received().len(), 1);
    world.settle();
}
#[test]
fn connection_faults_reset_before_reading_and_drop_after_recording() {
    use skein_fake_peers::push::Faults;
    let vapid = Vapid::from_pkcs8_pem(PEM).expect("RFC test key");
    for drop in [true, false] {
        let mut world = World::new(*vapid.public());
        while world.peer.listening_address().is_none() {
            world.pass();
        }
        world
            .peer
            .faults(Faults { reset: if drop { 0 } else { 1000 }, drop: if drop { 1000 } else { 0 } })
            .expect("per-mille faults");
        let request = request(&world, &vapid);
        let response = if drop {
            world.exchange(&request)
        } else {
            let fd = world.connect();
            world.response(fd)
        };
        assert!(response.is_empty(), "the fault emits no HTTP answer");
        assert_eq!(world.peer.received().len(), usize::from(drop));
        world.settle();
    }
}
