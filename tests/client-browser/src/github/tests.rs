//! The browser fixture's admission, with a setup close completion held on Sim.
use skein_fake_peers::forge::{Observation, Peer};
use skein_io::kernel::{Complete, Done, Family, Fd, Op, Submit};
use skein_lib::{Queue, Token};
use skein_sim::{Pid, Sim};
use skein_world::Host;

const PASSES: u32 = 1000;

struct World {
    peer: Peer,
    sim: Sim,
    server: Pid,
    client: Pid,
    serial: u64,
    passes: u32,
    hold_close: bool,
    held: Vec<Complete>,
}
impl World {
    fn new() -> Self {
        let mut sim = Sim::new(super::SEED, skein_sim::Config::calm());
        let server = sim.spawn_process();
        let client = sim.spawn_process();
        let mut world = Self {
            peer: super::peer(([127, 0, 0, 1], 0).into())
                .keeping(skein_fake_peers::forge::Kept { heads: true, ..skein_fake_peers::forge::Kept::NONE }),
            sim,
            server,
            client,
            serial: 0,
            passes: 0,
            hold_close: false,
            held: Vec::new(),
        };
        while world.peer.address().is_none() || world.peer.blob_address().is_none() {
            world.pass();
        }
        world
    }
    fn pass(&mut self) {
        self.passes += 1;
        assert!(self.passes <= PASSES, "fixture admission settles within its pass bound");
        let mut completions = Queue::with_capacity(self.peer.operations());
        self.sim.reap(self.server, &mut completions);
        while let Some(complete) = completions.pop() {
            if self.hold_close && matches!(complete.kind, Op::Close { .. }) {
                self.held.push(complete);
            } else {
                self.peer.completions().push(complete);
            }
        }
        self.peer.iterate(self.sim.now(), self.sim.wall());
        self.sim.submit(self.server, self.peer.submissions());
        if !self.peer.work_pending(self.sim.now())
            && let Some(next) = [self.sim.next_due(), self.peer.next_deadline()].into_iter().flatten().min()
        {
            self.sim.advance_to(next);
        }
    }
    fn call(&mut self, kind: Op) -> Complete {
        self.serial += 1;
        let mut submissions = Queue::with_capacity(1);
        let mut completions = Queue::with_capacity(1);
        submissions.push(Submit { op: Token::new(self.serial), kind });
        self.sim.submit(self.client, &mut submissions);
        loop {
            self.sim.reap(self.client, &mut completions);
            if let Some(complete) = completions.pop() {
                return complete;
            }
            self.pass();
        }
    }
    fn open(&mut self) -> Fd {
        let Done::Fd(fd) = self.call(Op::Socket { family: Family::Ipv4 }).result.expect("socket") else {
            panic!("client socket");
        };
        let addr = self.peer.address().expect("bound peer");
        assert_eq!(self.call(Op::Connect { fd, addr }).result, Ok(Done::Nothing));
        fd
    }
    fn answer(&mut self, fd: Fd) {
        // The fixture owns this App and bot. Keep this admission probe separate
        // from Chromium's incidental requests and the fake's error wording.
        let config = super::github_config();
        let slug = std::str::from_utf8(&config.app.slug).expect("fixture App slug");
        let request = format!(
            "GET /users/{slug}%5Bbot%5D HTTP/1.1\r\nHost: localhost\r\nUser-Agent: fixture\r\nX-GitHub-Api-Version: {}\r\n\r\n",
            skein_forge::github::pin::API_VERSION
        )
        .into_bytes();
        let mut from = 0;
        while usize::try_from(from).expect("send offset") < request.len() {
            let complete = self.call(Op::Send { fd, bytes: request.clone().into_boxed_slice(), from });
            let Ok(Done::Count(sent)) = complete.result else { panic!("request sends") };
            assert!(sent > 0);
            from += sent;
        }
        let mut answer = Vec::new();
        loop {
            if let Some(end) = answer.windows(4).position(|part| part == b"\r\n\r\n") {
                let header = std::str::from_utf8(&answer[..end]).expect("HTTP headers");
                assert!(header.starts_with("HTTP/1.1 200 "), "the fixture's App bot answers: {header}");
                let length = header
                    .split("\r\n")
                    .skip(1)
                    .find_map(|line| {
                        let (name, value) = line.split_once(':').expect("HTTP header");
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().expect("Content-Length"))
                    })
                    .expect("the fake frames the bot with Content-Length");
                if let Some(body) = answer.get(end + 4..end + 4 + length) {
                    let bot: serde_json::Value = serde_json::from_slice(body).expect("App bot JSON");
                    assert_eq!(bot["id"], config.bot);
                    assert_eq!(bot["login"], format!("{slug}[bot]"));
                    break;
                }
            }
            let complete = self.call(Op::Recv { fd, buf: Box::new([0; 1024]) });
            let Ok(Done::Count(count)) = complete.result else {
                panic!("the next setup connection is admitted: {:?}", complete.result);
            };
            assert!(count > 0, "the next setup connection receives an answer");
            let Op::Recv { buf, .. } = complete.kind else { panic!("receive buffer") };
            answer.extend_from_slice(&buf[..usize::try_from(count).expect("received bytes")]);
        }
    }
    fn close(&mut self, fd: Fd) {
        assert_eq!(self.call(Op::Close { fd }).result, Ok(Done::Nothing));
    }
    fn settle(mut self) {
        self.hold_close = false;
        for complete in self.held.drain(..) {
            self.peer.completions().push(complete);
        }
        self.peer.shutdown();
        while !self.peer.is_empty() {
            self.pass();
        }
        for pid in [self.server, self.client] {
            self.sim.assert_quiescent(pid);
            self.sim.assert_no_open_fds(pid);
        }
    }
}

#[test]
fn the_picker_is_admitted_before_the_previous_setup_connection_finishes_closing() {
    let mut world = World::new();
    let browser = world.open();
    world.answer(browser);
    let forge = world.open();
    world.answer(forge);
    let previous_setup = world.open();
    world.answer(previous_setup);

    // Replay the miss: EOF and the next Accept arrive before the old setup's
    // Close completion. The peer still owns that old connection's slot.
    world.hold_close = true;
    world.close(previous_setup);
    while world.held.is_empty() {
        world.pass();
    }
    let picker = world.open();
    world.answer(picker);
    assert_eq!(
        world.peer.observations().iter().filter(|record| matches!(record, Observation::Head { .. })).count(),
        4,
        "all four connections reach HTTP"
    );
    world.close(picker);
    world.close(forge);
    world.close(browser);
    world.settle();
}
