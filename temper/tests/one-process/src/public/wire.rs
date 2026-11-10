//! Socket programs on the world's kernel (temper/docs/design/testing/README.md, 2.5).
use super::http::{self, Answer};
use skein_http::Method;
use skein_io::kernel::{Complete, Done, Family, Fd, Op, Submit};
use skein_lib::{Queue, Token};
use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
};

const OPERATIONS: u32 = 256;
const PIECE: usize = 4096;
pub(super) enum Event {
    Request { id: u64, peer: SocketAddr, bytes: Box<[u8]> },
    Answer { id: u64, answer: Option<Answer> },
}
struct Connection {
    fd: Option<Fd>,
    peer: SocketAddr,
    method: Method,
    outgoing: Option<Box<[u8]>>,
    incoming: Vec<u8>,
    client: bool,
    abandon: bool,
}
pub(super) struct Wire {
    pub completions: Queue<Complete>,
    pub submissions: Queue<Submit>,
    pub events: VecDeque<Event>,
    connections: BTreeMap<u64, Connection>,
    pending: BTreeMap<Token, Option<u64>>,
    serial: u64,
    next: u64,
    listener: Option<Fd>,
    address: Option<SocketAddr>,
    accepting: Option<Token>,
    pub ready: bool,
    stopping: bool,
}
impl Wire {
    pub fn new(address: Option<SocketAddr>) -> Self {
        let mut wire = Self {
            completions: Queue::with_capacity(OPERATIONS),
            submissions: Queue::with_capacity(OPERATIONS),
            events: VecDeque::new(),
            connections: BTreeMap::new(),
            pending: BTreeMap::new(),
            serial: 0,
            next: 0,
            listener: None,
            address,
            accepting: None,
            ready: address.is_none(),
            stopping: false,
        };
        if let Some(addr) = address {
            wire.submit(None, Op::Socket { family: Family::of(&addr) });
        }
        wire
    }
    fn submit(&mut self, owner: Option<u64>, kind: Op) -> Token {
        self.serial += 1;
        let op = Token::new(self.serial);
        assert!(
            self.pending.len() < usize::try_from(OPERATIONS).expect("operations"),
            "bounded public program operations"
        );
        self.pending.insert(op, owner);
        self.submissions.push(Submit { op, kind });
        op
    }
    fn accept(&mut self) {
        self.accepting = Some(self.submit(None, Op::Accept { fd: self.listener.expect("listener") }));
    }
    pub fn start(&mut self, address: SocketAddr, method: Method, bytes: Box<[u8]>) -> u64 {
        let id = self.next;
        self.next += 1;
        self.connections.insert(
            id,
            Connection {
                fd: None,
                peer: address,
                method,
                outgoing: Some(bytes),
                incoming: Vec::new(),
                client: true,
                abandon: false,
            },
        );
        self.submit(Some(id), Op::Socket { family: Family::of(&address) });
        id
    }
    pub fn abandon(&mut self, id: u64) {
        self.connections.get_mut(&id).expect("new client request").abandon = true;
    }
    fn recv(&mut self, id: u64, fd: Fd) {
        self.submit(Some(id), Op::Recv { fd, buf: vec![0; PIECE].into() });
    }
    pub fn respond(&mut self, id: u64, bytes: Box<[u8]>) {
        let fd = self.connections.get(&id).expect("waiting downstream").fd.expect("connected downstream");
        self.submit(Some(id), Op::Send { fd, bytes, from: 0 });
    }
    pub fn stop(&mut self) {
        if self.stopping {
            return;
        }
        self.stopping = true;
        if let Some(target) = self.accepting {
            self.submit(None, Op::Cancel { target });
        } else if let Some(fd) = self.listener.take() {
            self.submit(None, Op::Close { fd });
        }
    }
    pub fn empty(&self) -> bool {
        self.pending.is_empty() && self.connections.is_empty() && self.listener.is_none()
    }
    pub fn idle(&self) -> bool {
        self.connections.is_empty()
    }
    pub fn iterate(&mut self) {
        while let Some(complete) = self.completions.pop() {
            let owner = self.pending.remove(&complete.op).expect("owned public operation");
            if let Some(id) = owner {
                self.connection(id, complete);
            } else {
                self.listening(complete);
            }
        }
    }
    fn listening(&mut self, complete: Complete) {
        if let Op::Accept { .. } = complete.kind {
            self.accepting = None;
            if self.stopping {
                if let Ok(Done::Accepted { fd, .. }) = complete.result {
                    self.submit(None, Op::Close { fd });
                }
                if let Some(fd) = self.listener.take() {
                    self.submit(None, Op::Close { fd });
                }
                return;
            }
        }
        match (complete.kind, complete.result.expect("public listener operation")) {
            (Op::Socket { .. }, Done::Fd(fd)) => {
                self.listener = Some(fd);
                self.submit(None, Op::Bind { fd, addr: self.address.expect("listen address") });
            }
            (Op::Bind { fd, .. }, Done::Bound(_)) => {
                self.submit(None, Op::Listen { fd, backlog: 128 });
            }
            (Op::Listen { .. }, Done::Nothing) => {
                self.ready = true;
                self.accept();
            }
            (Op::Accept { .. }, Done::Accepted { fd, peer }) => {
                let id = self.next;
                self.next += 1;
                self.connections.insert(
                    id,
                    Connection {
                        fd: Some(fd),
                        peer,
                        method: Method::Get,
                        outgoing: None,
                        incoming: Vec::new(),
                        client: false,
                        abandon: false,
                    },
                );
                self.recv(id, fd);
                self.accept();
            }
            (Op::Close { .. } | Op::Cancel { .. }, Done::Nothing) => {}
            other => panic!("public listener completion: {other:?}"),
        }
    }
    fn connection(&mut self, id: u64, complete: Complete) {
        let mut conn = self.connections.remove(&id).expect("live connection");
        let Ok(done) = complete.result else {
            if conn.client {
                self.events.push_back(Event::Answer { id, answer: None });
            }
            if let Some(fd) = conn.fd.take() {
                self.submit(None, Op::Close { fd });
            }
            return;
        };
        let mut keep = true;
        match (complete.kind, done) {
            (Op::Socket { .. }, Done::Fd(fd)) => {
                conn.fd = Some(fd);
                self.submit(Some(id), Op::Connect { fd, addr: conn.peer });
            }
            (Op::Connect { fd, .. }, Done::Nothing) => {
                self.submit(Some(id), Op::Send { fd, bytes: conn.outgoing.take().expect("outgoing request"), from: 0 });
            }
            (Op::Send { fd, bytes, from }, Done::Count(n)) => {
                assert!(n > 0, "send progresses");
                let from = from + n;
                if usize::try_from(from).expect("sent length") != bytes.len() {
                    self.submit(Some(id), Op::Send { fd, bytes, from });
                } else if conn.client && !conn.abandon {
                    self.recv(id, fd);
                } else {
                    if conn.abandon {
                        self.events.push_back(Event::Answer { id, answer: None });
                    }
                    self.submit(Some(id), Op::Close { fd });
                }
            }
            (Op::Recv { fd, buf }, Done::Count(n)) => {
                conn.incoming.extend_from_slice(&buf[..usize::try_from(n).expect("received length")]);
                assert!(conn.incoming.len() <= http::MESSAGE, "bounded public message");
                if conn.client {
                    if let Some(answer) = http::answer(&conn.incoming, conn.method) {
                        self.events.push_back(Event::Answer { id, answer: Some(answer) });
                        self.submit(Some(id), Op::Close { fd });
                    } else if n == 0 {
                        self.events.push_back(Event::Answer { id, answer: None });
                        self.submit(Some(id), Op::Close { fd });
                    } else {
                        self.recv(id, fd);
                    }
                } else if let Some((_, end)) = http::request(&conn.incoming) {
                    self.events.push_back(Event::Request { id, peer: conn.peer, bytes: conn.incoming[..end].into() });
                } else if n == 0 {
                    self.submit(Some(id), Op::Close { fd });
                } else {
                    self.recv(id, fd);
                }
            }
            (Op::Close { .. }, Done::Nothing) => keep = false,
            other => panic!("public connection completion: {other:?}"),
        }
        if keep {
            self.connections.insert(id, conn);
        }
    }
}
