//! Group files read beneath the box root through the ring
//! (temper/docs/design/draft/playground-reset.md, 3).
use alloc::boxed::Box;
use skein_io::kernel::{Complete, Done, Error, Fd, Op, OpenHow, Submit};
use skein_lib::{List, Queue, Token, bytes};
use temper_forge_setup::group;
type Answer = (u16, Result<Box<[u8]>, Error>);
const OP: Token = Token::new(u64::MAX - 10);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Idle,
    Open,
    Read(Fd),
    Close(Fd),
}
#[derive(Debug)]
pub(crate) struct Groups {
    phase: Phase,
    forge: u16,
    waiting: bool,
    bytes: List<u8>,
    error: Option<Error>,
    answer: Option<Answer>,
}
impl Groups {
    pub fn new() -> Self {
        Self { phase: Phase::Idle, forge: 0, waiting: false, bytes: List::with_capacity(0), error: None, answer: None }
    }
    pub fn owns(token: Token) -> bool {
        token == OP
    }
    pub fn ready(&self) -> bool {
        self.phase == Phase::Idle && self.answer.is_none()
    }
    pub fn pending(&self) -> bool {
        !self.waiting && !self.ready()
    }
    pub fn load(&mut self, forge: u16) {
        assert!(self.ready(), "one group load");
        self.forge = forge;
        self.bytes = List::with_capacity(group::FILE_BYTES.checked_add(1).expect("oversize sentinel"));
        self.error = None;
        self.phase = Phase::Open;
    }
    pub fn take(&mut self) -> Option<Answer> {
        self.answer.take()
    }
    pub fn tick(&mut self, root: Fd, out: &mut Queue<Submit>) {
        if self.waiting || self.answer.is_some() || out.room() == 0 {
            return;
        }
        let kind = match self.phase {
            Phase::Idle => return,
            Phase::Open => Op::Open { root, path: group::path(self.forge), how: OpenHow::Read },
            Phase::Read(fd) => Op::Read {
                fd,
                buf: bytes::zeroed(usize::try_from(self.bytes.room()).expect("read bound")),
                at: u64::from(self.bytes.len()),
            },
            Phase::Close(fd) => Op::Close { fd },
        };
        out.push(Submit { op: OP, kind });
        self.waiting = true;
    }
    pub fn complete(&mut self, complete: Complete) {
        self.waiting = false;
        match (self.phase, complete.result) {
            (Phase::Open, Ok(Done::Fd(fd))) => self.phase = Phase::Read(fd),
            (Phase::Open, Err(error)) => {
                self.error = Some(error);
                self.finish();
            }
            (Phase::Read(fd), Ok(Done::Count(count))) => {
                let Op::Read { buf, .. } = complete.kind else { unreachable!("read buffer") };
                for byte in buf.get(..usize::try_from(count).expect("read count")).expect("read bound") {
                    self.bytes.push(*byte).expect("admitted read");
                }
                if count == 0 || self.bytes.room() == 0 {
                    self.phase = Phase::Close(fd);
                }
            }
            (Phase::Read(fd), Err(error)) => {
                self.error = Some(error);
                self.phase = Phase::Close(fd);
            }
            (Phase::Close(_), result) => {
                if let Err(error) = result
                    && self.error.is_none()
                {
                    self.error = Some(error);
                }
                self.finish();
            }
            _ => unreachable!("group file completion shape"),
        }
    }
    fn finish(&mut self) {
        let bytes = core::mem::replace(&mut self.bytes, List::with_capacity(0)).into_boxed();
        let result = match self.error.take() {
            Some(error) => Err(error),
            None => Ok(bytes),
        };
        self.answer = Some((self.forge, result));
        self.phase = Phase::Idle;
    }
}

/// A socket's public spelling in the load refusal, bounded by its fixed address.
pub(crate) fn unheld(address: core::net::SocketAddr) -> Box<[u8]> {
    use skein_lib::Decimal;
    let mut parts = List::with_capacity(32);
    parts.push(bytes::copy_of(b"listener ")).expect("address pieces");
    match address {
        core::net::SocketAddr::V4(v4) => {
            for (index, octet) in v4.ip().octets().into_iter().enumerate() {
                if index > 0 {
                    parts.push(bytes::copy_of(b".")).expect("separator");
                }
                parts.push(bytes::copy_of(Decimal::of(u64::from(octet)).as_bytes())).expect("octet");
            }
        }
        core::net::SocketAddr::V6(v6) => {
            parts.push(bytes::copy_of(b"[")).expect("bracket");
            let segments = v6.ip().segments();
            let mut run = 0_usize;
            let mut first = 0_usize;
            let mut longest = 1_usize;
            let mut start = 0_usize;
            for (index, segment) in segments.iter().enumerate() {
                if *segment == 0 {
                    if run == 0 {
                        first = index;
                    }
                    run = run.checked_add(1).expect("eight segments");
                    if run > longest {
                        longest = run;
                        start = first;
                    }
                } else {
                    run = 0;
                }
            }
            let end = start.checked_add(longest).expect("eight segments");
            for (index, segment) in segments.iter().enumerate() {
                if longest > 1 && index >= start && index < end {
                    if index == start {
                        parts.push(bytes::copy_of(b"::")).expect("compression");
                    }
                    continue;
                }
                if index > 0 && !(longest > 1 && index == end) {
                    parts.push(bytes::copy_of(b":")).expect("separator");
                }
                let mut digits = List::with_capacity(4);
                for shift in [12_u32, 8, 4, 0] {
                    let digit = usize::from((segment >> shift) & 15);
                    if digit != 0 || !digits.is_empty() || shift == 0 {
                        digits.push(*b"0123456789abcdef".get(digit).expect("hex digit")).expect("four digits");
                    }
                }
                parts.push(digits.into_boxed()).expect("segment");
            }
            if v6.scope_id() > 0 {
                parts.push(bytes::copy_of(b"%")).expect("scope");
                parts.push(bytes::copy_of(Decimal::of(u64::from(v6.scope_id())).as_bytes())).expect("scope");
            }
            parts.push(bytes::copy_of(b"]")).expect("bracket");
        }
    }
    parts.push(bytes::copy_of(b":")).expect("port separator");
    parts.push(bytes::copy_of(Decimal::of(u64::from(address.port())).as_bytes())).expect("port");
    parts.push(bytes::copy_of(b" is not held by the box")).expect("refusal");
    let borrowed = parts.into_boxed();
    let mut length = 0_usize;
    for part in &borrowed {
        length = length.checked_add(part.len()).expect("bounded address refusal");
    }
    let mut writer = skein_lib::Writer::new(length);
    for part in &borrowed {
        writer.put(part).expect("address measured");
    }
    writer.finish()
}
