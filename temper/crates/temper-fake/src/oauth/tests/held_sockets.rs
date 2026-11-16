//! The fake issuer's held sockets (testing/process-tier.md, The fake box keeps two walks).
use super::{DOCUMENT, Document, SEED, VERIFIER, credential, refill, token_part};
use crate::oauth::CONNECTIONS;
use serde_json::Value;
use skein_fake_peers::oauth::Peer;
use skein_io::kernel::{Complete, Done, Family, Fd, Op, Submit};
use skein_lib::{Queue, Token};
use skein_shell::Host;
use skein_sim::{Pid, Sim};

struct World {
    sim: Sim,
    server: Pid,
    client: Pid,
    peer: Peer,
    exchanges: usize,
}
impl World {
    fn new() -> Self {
        let mut sim = Sim::new(SEED, skein_sim::Config::calm());
        let (server, client) = (sim.spawn_process(), sim.spawn_process());
        let peer = Document::decode(DOCUMENT).unwrap().peer(SEED).unwrap();
        let mut world = Self { sim, server, client, peer, exchanges: 0 };
        for _ in 0..100 {
            world.pass();
        }
        world
    }
    fn pass(&mut self) {
        self.sim.reap(self.server, self.peer.completions());
        refill(&mut self.peer, SEED).expect("production refill before each pass");
        let plans = self.peer.granted().len();
        self.peer.iterate(self.sim.now(), self.sim.wall());
        // The standalone peer retains no observations; consumed plans count exchanges.
        let consumed = plans.checked_sub(self.peer.granted().len()).expect("pass consumes plans");
        self.exchanges = self.exchanges.checked_add(consumed).expect("bounded exchanges");
        self.sim.submit(self.server, self.peer.submissions());
        self.sim.advance();
    }
    fn call(&mut self, kind: Op) -> Complete {
        let mut submissions = Queue::with_capacity(1);
        let mut completions = Queue::with_capacity(1);
        submissions.push(Submit { op: Token::new(1), kind });
        self.sim.submit(self.client, &mut submissions);
        for _ in 0..1000 {
            self.sim.reap(self.client, &mut completions);
            if let Some(complete) = completions.pop() {
                return complete;
            }
            self.pass();
        }
        panic!("the socket operation did not complete");
    }
    fn connect(&mut self) -> Fd {
        let Done::Fd(fd) = self.call(Op::Socket { family: Family::Ipv4 }).result.unwrap() else {
            panic!("socket descriptor")
        };
        assert_eq!(
            self.call(Op::Connect { fd, addr: self.peer.address().expect("issuer listener") }).result.unwrap(),
            Done::Nothing
        );
        for _ in 0..100 {
            self.pass();
        }
        fd
    }
    fn fetch(&mut self, fd: Fd, request: &[u8]) -> Vec<u8> {
        let mut from = 0;
        while usize::try_from(from).unwrap() < request.len() {
            let Done::Count(sent) = self.call(Op::Send { fd, bytes: request.into(), from }).result.unwrap() else {
                panic!("send count")
            };
            assert!(sent > 0, "request progresses");
            from = from.checked_add(sent).expect("bounded request");
        }
        let mut reply = Vec::new();
        loop {
            let complete = self.call(Op::Recv { fd, buf: Box::new([0; 4096]) });
            let Done::Count(count) = complete.result.expect("issuer read succeeds") else { panic!("read count") };
            assert!(count > 0, "issuer answers instead of turning the sign-in away");
            let Op::Recv { buf, .. } = complete.kind else { panic!("receive buffer") };
            reply.extend_from_slice(&buf[..usize::try_from(count).unwrap()]);
            if let Some(end) = reply.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = core::str::from_utf8(&reply[..end]).unwrap();
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .expect("issuer frames its response");
                let total = end.checked_add(4).and_then(|end| end.checked_add(length)).expect("bounded response");
                if reply.len() == total {
                    return reply;
                }
            }
        }
    }
    fn close(&mut self, fd: Fd) {
        assert_eq!(self.call(Op::Close { fd }).result.unwrap(), Done::Nothing);
    }
    fn settle(mut self) {
        self.peer.shutdown();
        for _ in 0..1000 {
            if self.peer.is_empty() {
                break;
            }
            self.pass();
        }
        assert!(self.peer.is_empty());
        assert_eq!(self.exchanges, 1, "exactly one code exchange");
        for pid in [self.server, self.client] {
            self.sim.assert_quiescent(pid);
            self.sim.assert_no_open_fds(pid);
        }
    }
}

fn request(method: &str, target: &str, body: &str) -> Vec<u8> {
    format!("{method} {target} HTTP/1.1\r\nhost: localhost\r\nconnection: keep-alive\r\ncontent-type: application/x-www-form-urlencoded\r\ncontent-length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

#[test]
fn a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away() {
    let mut world = World::new();
    let challenge = skein_oauth::challenge(VERIFIER).unwrap();
    let target = format!(
        "/api/accounts/authorize?response_type=code&client_id=dynamic_agent_client&redirect_uri=http://127.0.0.1:1455/auth/callback&state=0123456789abcdef&nonce=fake-nonce&resource=https://api.openai.com/v1&code_challenge_method=S256&code_challenge={}",
        core::str::from_utf8(&challenge).unwrap()
    );
    let authorize = world.connect();
    let reply = world.fetch(authorize, &request("GET", &target, ""));
    assert!(reply.starts_with(b"HTTP/1.1 302"), "authorize grants at the browser's visit");
    let code = reply.windows(b"?code=".len()).position(|part| part == b"?code=").expect("redirect code");
    let code = reply[code + b"?code=".len()..].split(|byte| *byte == b'&').next().unwrap();
    // Keep the authorize socket, then open its spare: not two sockets before the visit.
    let spare = world.connect();
    let body = format!(
        "grant_type=authorization_code&client_id=issued-client&redirect_uri=http://127.0.0.1:1455/auth/callback&code={}&code_verifier={}",
        core::str::from_utf8(code).unwrap(),
        core::str::from_utf8(VERIFIER).unwrap()
    );
    let token = world.connect();
    let reply = world.fetch(token, &request("POST", "/api/accounts/oauth/token", &body));
    assert!(reply.starts_with(b"HTTP/1.1 200"), "token POST completes beside the browser's two held sockets");
    let end = reply.windows(4).position(|part| part == b"\r\n\r\n").unwrap();
    let tokens: Value = serde_json::from_slice(&reply[end + 4..]).unwrap();
    assert_eq!(
        tokens["access_token"],
        core::str::from_utf8(&credential::credential(SEED, credential::added_key(0))).unwrap()
    );
    assert_eq!(tokens["refresh_token"], "fake-refresh");
    let parts: Vec<_> = tokens["id_token"].as_str().expect("account ID token").split('.').collect();
    let [_header, claims, _signature] = parts.as_slice() else { panic!("signed ID token") };
    let claims = token_part(claims);
    assert_eq!(claims["sub"], "fake-person");
    assert_eq!(claims["email"], "sam@example.com");
    assert_eq!(claims["email_verified"], true);
    assert_eq!(world.exchanges, 1, "exactly one code exchange");
    assert_eq!(world.peer.rejected(), 0, "the sign-in was admitted");
    world.close(token);

    let further: Vec<_> = (0..=CONNECTIONS).map(|_| world.connect()).collect();
    assert!(world.peer.rejected() > 0, "the production issuer turns away sockets past its cap");
    for fd in further.into_iter().chain([authorize, spare]) {
        world.close(fd);
    }
    world.settle();
}
