//! One client exchange over the hosted process's simulated loopback.
use super::Script;
use skein_io::kernel::{Complete, Done, Family, Fd, Op, Spawn};
use std::net::SocketAddr;

/// Where a hosted child reads its peer's address.
#[derive(Clone, Copy, Debug)]
pub enum Address {
    Argument(usize),
    Environment(&'static [u8]),
}

impl Address {
    pub(super) fn read(self, script: &Script) -> SocketAddr {
        let bytes = match self {
            Self::Argument(index) => &script.args[index],
            Self::Environment(name) => script
                .env()
                .iter()
                .find_map(|entry| entry.strip_prefix(name)?.strip_prefix(b"="))
                .expect("explicit environment value"),
        };
        let addr: SocketAddr = std::str::from_utf8(bytes).expect("address text").parse().expect("substituted address");
        assert!(addr.ip().is_loopback(), "scripted exchanges are local");
        addr
    }
}

#[derive(Debug)]
pub(super) struct Exchanges {
    pub env: Box<[Box<[u8]>]>,
    pub replies: Vec<Vec<u8>>,
    current: Option<Exchange>,
}

impl Exchanges {
    pub fn new(spawn: &Spawn) -> Self {
        Self { env: spawn.env.clone(), replies: Vec::new(), current: None }
    }

    pub fn active(&self) -> bool {
        self.current.is_some()
    }

    pub fn start(&mut self, addr: SocketAddr, request: &'static [u8]) -> Op {
        assert!(!self.active(), "exchanges are sequential");
        self.current = Some(Exchange { addr, request, reply: Vec::new() });
        Op::Socket { family: if addr.is_ipv4() { Family::Ipv4 } else { Family::Ipv6 } }
    }

    pub fn advance(&mut self, complete: Complete) -> Option<Op> {
        let mut current = self.current.take().expect("an exchange is in flight");
        if let Some(next) = current.advance(complete) {
            self.current = Some(current);
            Some(next)
        } else {
            self.replies.push(current.reply);
            None
        }
    }
}

#[derive(Debug)]
struct Exchange {
    addr: SocketAddr,
    request: &'static [u8],
    reply: Vec<u8>,
}

impl Exchange {
    fn recv(fd: Fd) -> Op {
        Op::Recv { fd, buf: Box::new([0; 1024]) }
    }

    fn advance(&mut self, complete: Complete) -> Option<Op> {
        let op = match (complete.kind, complete.result) {
            (Op::Socket { .. }, Ok(Done::Fd(fd))) => Op::Connect { fd, addr: self.addr },
            (Op::Connect { fd, .. }, Ok(Done::Nothing)) => Op::Send { fd, bytes: self.request.into(), from: 0 },
            (Op::Send { fd, bytes, from }, Ok(Done::Count(count))) if count > 0 => {
                let from = from + count;
                if usize::try_from(from).expect("sent count fits") < bytes.len() {
                    Op::Send { fd, bytes, from }
                } else {
                    Self::recv(fd)
                }
            }
            (Op::Recv { fd, .. }, Ok(Done::Count(0))) => Op::Close { fd },
            (Op::Recv { fd, buf }, Ok(Done::Count(count))) => {
                self.reply.extend_from_slice(&buf[..usize::try_from(count).expect("received count fits")]);
                assert!(self.reply.len() <= 8192, "a bounded fixture reply");
                Self::recv(fd)
            }
            (Op::Close { .. }, Ok(Done::Nothing)) => return None,
            (op, result) => panic!("scripted exchange to {}: {op:?} answered {result:?}", self.addr),
        };
        Some(op)
    }
}
