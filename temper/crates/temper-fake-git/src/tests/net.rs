//! The scripted remote decodes the route's actual request and writes server answers.
use super::*;
use crate::connection::{Connection, Failed, Reply};
use skein_fake_forge_protocol::git::{Answer, Call, Name, server};
use skein_json::tokenizer;
use skein_lib::{
    Env, Queue,
    stream::{Down, Held},
};
use std::collections::VecDeque;
#[derive(Clone)]
pub(super) enum Response {
    Answer(Answer),
    Error(u16, api::Error),
    Raw(u16, Vec<u8>),
    Reset,
    Close,
}
pub(super) struct Net {
    script: VecDeque<(Call, Response)>,
    request: Vec<u8>,
    response: VecDeque<u8>,
    reset: bool,
    held: bool,
    pub seen: Vec<Call>,
    pub connected: Vec<k::Addr>,
    pub refused: bool,
    pub piece: usize,
}
impl Net {
    pub const FD: Fd = Fd::new(10004);
    pub fn new(script: Vec<(Call, Response)>) -> Self {
        Self {
            script: script.into(),
            request: Vec::new(),
            response: VecDeque::new(),
            reset: false,
            held: false,
            seen: Vec::new(),
            connected: Vec::new(),
            refused: false,
            piece: 127,
        }
    }
    fn request(&mut self) {
        let Some(end) = self.request.windows(4).position(|b| b == b"\r\n\r\n").map(|n| n + 4) else { return };
        let length = self.request[..end]
            .split(|b| *b == b'\n')
            .find_map(|line| line.strip_prefix(b"Content-Length: "))
            .map(|length| {
                std::str::from_utf8(length.strip_suffix(b"\r").expect("CRLF"))
                    .expect("ASCII")
                    .parse::<usize>()
                    .expect("length")
            })
            .expect("length header");
        if self.request.len() < end + length {
            return;
        }
        assert_eq!(self.request.len(), end + length, "one HTTP request at a time");
        let target = self.request.split(|b| *b == b' ').nth(1).expect("target");
        let name = git::name(target).expect("git prefix").expect("git call");
        let call = decode(name, self.request[end..].into());
        let authorization = self.request[..end]
            .split(|b| *b == b'\n')
            .find_map(|line| line.strip_prefix(b"Authorization: ").map(|v| v.strip_suffix(b"\r").expect("CRLF")));
        assert_eq!(authorization, name.repository().then_some(AUTHORIZATION), "{name:?}: repository credentials only");
        let (expected, response) = self.script.pop_front().unwrap_or_else(|| panic!("unexpected route call {call:?}"));
        assert_eq!(call, expected, "scripted call order and write count");
        self.seen.push(call);
        self.request.clear();
        let (status, body, length) = match response {
            Response::Answer(answer) => {
                (200, server::encode(&answer, &limits()).expect("answer encoding").into_vec(), None)
            }
            Response::Error(status, error) => {
                (status, server::error(error, &limits()).expect("error encoding").into_vec(), None)
            }
            Response::Raw(status, body) => (status, body, None),
            Response::Reset => {
                self.reset = true;
                (200, b"cut".to_vec(), Some(10))
            }
            Response::Close => (200, b"cut".to_vec(), Some(10)),
        };
        let length = length.unwrap_or(body.len());
        self.response.extend(format!("HTTP/1.1 {status} Answer\r\nContent-Length: {length}\r\n\r\n").as_bytes());
        self.response.extend(body);
    }
    pub fn step(&mut self, op: &mut Op) -> Result<Done, k::Error> {
        match op {
            Op::Socket { .. } => {
                assert!(!self.held, "one connection per command");
                self.held = true;
                Ok(Done::Fd(Self::FD))
            }
            Op::Connect { addr, .. } => {
                self.connected.push(*addr);
                if self.refused { Err(k::Error::Refused) } else { Ok(Done::Nothing) }
            }
            Op::Send { bytes, from, .. } => {
                let bytes = &bytes[usize::try_from(*from).expect("offset")..];
                let count = bytes.len().min(13);
                self.request.extend_from_slice(&bytes[..count]);
                self.request();
                Ok(Done::Count(u32::try_from(count).expect("short send")))
            }
            Op::Recv { buf, .. } => {
                if self.response.is_empty() && self.reset {
                    return Err(k::Error::Reset);
                }
                let count = buf.len().min(self.response.len()).min(self.piece);
                for slot in buf.iter_mut().take(count) {
                    *slot = self.response.pop_front().expect("receive count");
                }
                Ok(Done::Count(u32::try_from(count).expect("receive count")))
            }
            Op::Shutdown { .. } => Ok(Done::Nothing),
            Op::Close { fd } => {
                assert_eq!(*fd, Self::FD);
                assert!(self.held);
                self.held = false;
                Ok(Done::Nothing)
            }
            Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Open { .. }
            | Op::Read { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Lock { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Window { .. }
            | Op::ReadSignal { .. }
            | Op::Cancel { .. }
            | Op::Usage
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. } => panic!("script accepts socket jobs only"),
        }
    }
    pub fn closed(&self) {
        assert!(!self.held, "socket closed");
        assert!(self.script.is_empty(), "all calls made once: {:?}", self.script.front().map(|(call, _)| call));
    }
    pub fn exchange(&mut self, connection: &mut Connection) -> Result<Reply, Failed> {
        for _ in 0..10000 {
            let job = connection.next_job();
            if let Some(answer) = connection.take() {
                return answer;
            }
            if let Some(mut job) = job {
                let result = self.step(&mut job);
                connection.complete(job, result);
            }
        }
        panic!("exchange settles");
    }
    pub fn close(&mut self, connection: &mut Connection) {
        connection.close();
        for _ in 0..4 {
            if let Some(mut job) = connection.next_job() {
                let result = self.step(&mut job);
                connection.complete(job, result);
            }
        }
        assert!(connection.is_closed());
        self.closed();
    }
}
fn decode(name: Name, bytes: Box<[u8]>) -> Call {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits().documents.json };
    let mut json = tokenizer::Tokenizer::new(&env.limits);
    let mut decoder = server::decoder(name, &limits());
    let mut above = Queue::with_capacity(2);
    let mut below = Queue::with_capacity(2);
    let mut held = Held::new(bytes);
    tokenizer::down(&mut json, &env, server::body_first(&decoder), &mut above, &mut below);
    for _ in 0..10000 {
        if let Some(event) = above.pop() {
            match server::body(&mut decoder, &limits(), &event) {
                server::BodyStep::Demand(request) => tokenizer::down(&mut json, &env, request, &mut above, &mut below),
                server::BodyStep::Decoded(call) => return call,
                server::BodyStep::Malformed => panic!("malformed request"),
            }
        } else if let Some(Down::Demand { read, .. }) = below.pop() {
            tokenizer::up(&mut json, &env, held.answer(read).expect("body demand"), &mut above, &mut below);
        } else {
            panic!("request decoder progresses");
        }
    }
    panic!("bounded request decoder");
}
