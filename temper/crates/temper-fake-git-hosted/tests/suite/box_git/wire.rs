//! The box's Git core, stdio and forge on one simulated kernel.
use super::{SEED, text};
use skein_fake_machine::{self as disk, How, Item, Machine, Opened};
use skein_fake_peers::forge::{Forge, Peer};
use skein_io::kernel::{Complete, Done, Exit, Family, Fd, LONGEST_NAME, Op, Spawn, Submit};
use skein_lib::{Queue, Token};
use skein_shell::Host;
use skein_sim::{Config, Handle, Pid, Sim};
use std::{cell::RefCell, net::SocketAddr, rc::Rc};
use temper_fake::forge;
use temper_fake_git::{Answer, Git, Route, Start};

const FORGE_PORT: u16 = 3000;
const FIXTURE_PORT: u16 = 3001;
// Each operation or settlement takes far fewer turns at the calm kernel's fixed bounds.
const TURNS: usize = 2048;
// One receive buffer for the seeded HTTP exchanges; reads continue through EOF or the framed request.
const RECEIVE_BYTES: usize = 4096;

#[derive(Clone, Copy)]
pub(super) enum Endpoint {
    Forge,
    Reply { status: &'static str, body: &'static [u8] },
    Closed,
}

impl Endpoint {
    pub(super) fn address(self) -> SocketAddr {
        let port = match self {
            Self::Forge => FORGE_PORT,
            Self::Reply { .. } | Self::Closed => FIXTURE_PORT,
        };
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    pub(super) fn remote(self) -> String {
        format!("http://{}/fake/widget.git", self.address())
    }
}

pub(super) struct Hosted {
    sim: Sim,
    machine: Machine,
    root: Opened,
    server: Pid,
    peer: Peer,
    shared: Rc<RefCell<Forge>>,
}

impl Hosted {
    pub(super) fn new(token: &[u8; 40]) -> Self {
        let document = serde_json::json!({
            "listen": {"socket": Endpoint::Forge.address().to_string()},
            "admin_token": "/forge/admin_token",
            "owner": "fake",
            "repositories": [{"name": "widget", "landing": "main", "ci": null}]
        });
        let document = forge::Document::decode(document.to_string().as_bytes()).expect("forge document");
        let (peer, shared) = forge::peer(&document, token, SEED).expect("seeded forge");
        let mut sim = Sim::new(SEED, Config::calm());
        let server = sim.spawn_process();
        let mut machine = Machine::new();
        let root = machine.lay(&[Item::directory(b"home"), Item::directory(b"work")]);
        let mut hosted = Self { sim, machine, root, server, peer, shared };
        for _ in 0..TURNS {
            hosted.turn_forge();
            if hosted.peer.address().is_some() {
                let path = "/api/v1/repos/fake/widget/collaborators/temper-admin";
                let grant = hosted.request("PUT", path, token, br#"{"permission":"admin"}"#);
                assert!(grant.starts_with(b"HTTP/1.1 204"), "grant: {}", text(&grant));
                return hosted;
            }
        }
        panic!("forge comes ready");
    }

    fn turn_forge(&mut self) {
        turn(&mut self.sim, self.server, &mut self.peer);
        self.sim.advance();
    }

    fn request(&mut self, method: &str, path: &str, token: &[u8; 40], body: &[u8]) -> Vec<u8> {
        let address = Endpoint::Forge.address();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {address}\r\nAuthorization: token {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            text(token),
            body.len()
        );
        let server = self.server;
        let peer = &mut self.peer;
        let mut progress = |sim: &mut Sim| turn(sim, server, peer);
        let client = self.sim.spawn_process();
        let fd = socket(&mut self.sim, client, &mut progress);
        call(&mut self.sim, client, Op::Connect { fd, addr: address }, &mut progress);
        send(&mut self.sim, client, fd, &[request.as_bytes(), body].concat(), &mut progress);
        let mut response = Vec::new();
        loop {
            let bytes = receive(&mut self.sim, client, fd, &mut progress);
            if bytes.is_empty() {
                break;
            }
            response.extend(bytes);
        }
        call(&mut self.sim, client, Op::Close { fd }, &mut progress);
        self.sim.assert_no_open_fds(client);
        self.sim.assert_quiescent(client);
        response
    }

    pub(super) fn branch(&mut self, token: &[u8; 40]) -> String {
        let response = self.request("GET", "/api/v1/repos/fake/widget/branches/work", token, b"");
        assert!(response.starts_with(b"HTTP/1.1 200"), "branch: {}", text(&response));
        let (_, body) = text(&response).split_once("\r\n\r\n").expect("branch response body");
        let branch: serde_json::Value = serde_json::from_str(body).expect("branch JSON");
        branch["commit"]["id"].as_str().expect("branch commit id").into()
    }

    pub(super) fn check_message(&self, message: &[u8]) {
        // The seed owns object 1. This is the first commit after it.
        let stored = 2;
        let forge = self.shared.borrow();
        let commit = forge.domain.object(stored).expect("stored commit");
        assert_eq!(commit.message.as_ref(), message);
    }

    pub(super) fn run(&mut self, endpoint: Endpoint, input: &[u8], invocation: impl FnOnce(Fd) -> Spawn) -> Answer {
        let pid = self.sim.spawn_process();
        let worker = Handle::new(self.machine.reopen_root(0).raw());
        let root = self.sim.root(pid, worker);
        let [stdin, stdout, stderr] =
            [self.sim.open_inherited_read(pid), self.sim.open_inherited_write(pid), self.sim.open_inherited_write(pid)];
        assert_eq!(self.sim.peer_feed(pid, stdin, input), input.len());
        self.sim.peer_close(pid, stdin);
        let start = Start {
            root,
            pipes: Box::new([(0, stdin), (1, stdout), (2, stderr)]),
            owned: Box::new([root, stdin, stdout, stderr]),
        };
        let mut git = Git::new(&invocation(root), start, Route::new(forge::git_limits().git()));
        if let Endpoint::Reply { status, body } = endpoint {
            let head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            let response = [head.as_bytes(), body].concat();
            let machine = &mut self.machine;
            script(&mut self.sim, endpoint.address(), &response, &mut |sim| {
                turn(sim, pid, &mut git);
                disk::serve(machine, sim);
            });
        }
        for _ in 0..TURNS {
            turn(&mut self.sim, pid, &mut git);
            disk::serve(&mut self.machine, &mut self.sim);
            self.turn_forge();
            if let Some(Exit::Code(exit)) = git.exit() {
                self.sim.assert_no_open_fds(pid);
                self.sim.assert_quiescent(pid);
                let answer = Answer {
                    exit,
                    stdout: self.sim.peer_drain(pid, stdout, usize::MAX),
                    stderr: self.sim.peer_drain(pid, stderr, usize::MAX),
                };
                assert!(self.sim.peer_eof(pid, stdout) && self.sim.peer_eof(pid, stderr));
                self.sim.peer_close(pid, stdout);
                self.sim.peer_close(pid, stderr);
                return answer;
            }
        }
        panic!("{}", self.sim.report("git must settle"));
    }

    pub(super) fn work_is_empty(&mut self) -> bool {
        let work = self.machine.open(self.root, b"work", How::DirectoryNoFollow).expect("working directory");
        let room = u32::try_from(LONGEST_NAME).expect("a name fits a u32");
        let entries = self.machine.list(work, 1, room).expect("work entries");
        self.machine.close(work);
        entries.is_empty()
    }

    pub(super) fn write(&mut self, at: &[u8], path: &[u8], content: &[u8]) {
        let path = absolute_join(at, path);
        let file = self.machine.open(self.root, &path, How::CreateNoFollow { mode: 0o600 }).expect("worker edit");
        self.machine.write(file, 0, content).expect("write fixture");
        self.machine.close(file);
    }

    pub(super) fn read(&mut self, at: &[u8], path: &[u8]) -> Vec<u8> {
        let path = absolute_join(at, path);
        let file = self.machine.open(self.root, &path, How::ReadNoFollow).expect("checkout file");
        let size = u32::try_from(self.machine.stat(file).size).expect("fixture size");
        let bytes = self.machine.read(file, 0, size).expect("read fixture");
        self.machine.close(file);
        bytes
    }

    pub(super) fn finish(mut self) {
        self.peer.shutdown();
        for _ in 0..TURNS {
            self.turn_forge();
            if self.peer.is_empty() {
                self.sim.assert_no_open_fds(self.server);
                self.sim.assert_quiescent(self.server);
                self.machine.close(self.root);
                assert_eq!(self.machine.open_handles(), 0);
                return;
            }
        }
        panic!("{}", self.sim.report("forge must settle"));
    }
}

fn absolute_join(at: &[u8], path: &[u8]) -> Vec<u8> {
    [at.strip_prefix(b"/").expect("absolute fixture path"), b"/", path].concat()
}

fn turn(sim: &mut Sim, pid: Pid, host: &mut impl Host) {
    sim.reap(pid, host.completions());
    host.iterate(sim.now(), sim.wall());
    sim.submit(pid, host.submissions());
}

fn call(sim: &mut Sim, pid: Pid, kind: Op, progress: &mut impl FnMut(&mut Sim)) -> Complete {
    let mut out = Queue::with_capacity(1);
    let mut incoming = Queue::with_capacity(1);
    out.push(Submit { op: Token::new(1), kind });
    sim.submit(pid, &mut out);
    for _ in 0..TURNS {
        sim.reap(pid, &mut incoming);
        if let Some(completed) = incoming.pop() {
            assert!(
                completed.result.is_ok(),
                "{}",
                sim.report(&format!("fixture {:?}: {:?}", completed.kind.shape(), completed.result))
            );
            return completed;
        }
        progress(sim);
        sim.advance();
    }
    panic!("{}", sim.report("fixture operation must settle"));
}

fn socket(sim: &mut Sim, pid: Pid, progress: &mut impl FnMut(&mut Sim)) -> Fd {
    let Ok(Done::Fd(fd)) = call(sim, pid, Op::Socket { family: Family::Ipv4 }, progress).result else {
        panic!("socket descriptor");
    };
    fd
}

fn receive(sim: &mut Sim, pid: Pid, fd: Fd, progress: &mut impl FnMut(&mut Sim)) -> Vec<u8> {
    let completed = call(sim, pid, Op::Recv { fd, buf: vec![0; RECEIVE_BYTES].into() }, progress);
    let (Op::Recv { buf, .. }, Ok(Done::Count(count))) = (completed.kind, completed.result) else {
        panic!("received bytes");
    };
    buf[..skein_lib::usize_of(count)].to_vec()
}

fn send(sim: &mut Sim, pid: Pid, fd: Fd, bytes: &[u8], progress: &mut impl FnMut(&mut Sim)) {
    let mut from = 0;
    while skein_lib::usize_of(from) < bytes.len() {
        let Ok(Done::Count(count)) = call(sim, pid, Op::Send { fd, bytes: bytes.into(), from }, progress).result else {
            panic!("sent bytes");
        };
        assert_ne!(count, 0, "the write progresses");
        from += count;
    }
}

fn script(sim: &mut Sim, address: SocketAddr, response: &[u8], git: &mut impl FnMut(&mut Sim)) {
    let server = sim.spawn_process();
    let listener = socket(sim, server, git);
    call(sim, server, Op::Bind { fd: listener, addr: address }, git);
    call(sim, server, Op::Listen { fd: listener, backlog: 1 }, git);
    let Ok(Done::Accepted { fd, .. }) = call(sim, server, Op::Accept { fd: listener }, git).result else {
        panic!("git connection");
    };
    let mut request = Vec::new();
    loop {
        let bytes = receive(sim, server, fd, git);
        assert!(!bytes.is_empty(), "git sends the complete request before the reply");
        request.extend(bytes);
        if let Some((head, body)) = text(&request).split_once("\r\n\r\n") {
            let length: usize = head
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .expect("request length")
                .parse()
                .expect("decimal length");
            if body.len() == length {
                break;
            }
        }
    }
    send(sim, server, fd, response, git);
    call(sim, server, Op::Close { fd }, git);
    call(sim, server, Op::Close { fd: listener }, git);
    sim.assert_no_open_fds(server);
    sim.assert_quiescent(server);
}
