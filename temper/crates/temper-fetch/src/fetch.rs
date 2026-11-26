//! Streaming pinned binary fetch (protocol/setup/owned.md, 6).
use crate::{Origin, joined};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::Binary;
use skein_lib::{Decimal, Queue, Writer, sha256::Sha256};

/// A pinned binary's progress name (protocol/setup/owned.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Named {
    Forge(Binary),
    ClaudeCode,
}
/// The fetch's progress on stdout (protocol/setup/owned.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fetched {
    pub binary: Named,
    pub done: u64,
    pub total: Option<u64>,
}
impl Fetched {
    /// The progress line, its line feed included.
    #[must_use]
    pub fn line(&self) -> Box<[u8]> {
        let done = Decimal::of(self.done);
        match self.total {
            Some(total) => joined(&[
                b"fetched ",
                self.word(),
                b" ",
                done.as_bytes(),
                b" of ",
                Decimal::of(total).as_bytes(),
                b" bytes\n",
            ]),
            None => joined(&[b"fetched ", self.word(), b" ", done.as_bytes(), b" bytes\n"]),
        }
    }
    /// The binary name in this line.
    #[must_use]
    pub fn word(&self) -> &'static [u8] {
        match self.binary {
            Named::Forge(Binary::Forgejo) => b"forgejo",
            Named::Forge(Binary::Runner) => b"runner",
            Named::ClaudeCode => b"claude-code",
        }
    }
    /// Reads a progress line with an optional line feed, refusing invalid totals.
    #[must_use]
    pub fn read(line: &[u8]) -> Option<Fetched> {
        let line = match line.strip_suffix(b"\n") {
            Some(line) => line,
            None => line,
        };
        let rest = line.strip_prefix(b"fetched ")?.strip_suffix(b" bytes")?;
        let (binary, numbers) = if let Some(numbers) = rest.strip_prefix(b"forgejo ") {
            (Binary::Forgejo, numbers)
        } else {
            (Binary::Runner, rest.strip_prefix(b"runner ")?)
        };
        let (done, total) = match skein_lib::bytes::find(numbers, b" of ") {
            Some(at) => {
                let done = skein_lib::bytes::decimal(numbers.get(..at)?)?;
                let after = at.checked_add(b" of ".len())?;
                let total = skein_lib::bytes::decimal(numbers.get(after..)?)?;
                if total == 0 || done > total {
                    return None;
                }
                (done, Some(total))
            }
            None => (skein_lib::bytes::decimal(numbers)?, None),
        };
        Some(Self { binary: Named::Forge(binary), done, total })
    }
}

/// One pinned release at its resolved cache path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pin {
    pub binary: Named,
    pub origin: Origin,
    pub path: Box<[u8]>,
    pub digest: [u8; 32],
    pub file: Box<[u8]>,
    pub packed: bool,
}
/// Why the fetch was refused; no existing binary is replaced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Architecture(Box<[u8]>),
    Present { file: Box<[u8]> },
    Status { url: Box<[u8]>, status: u16 },
    Truncated { url: Box<[u8]> },
    Damaged { url: Box<[u8]> },
    UnpackedBytes { url: Box<[u8]> },
    Trailing { url: Box<[u8]> },
    Bytes { url: Box<[u8]> },
    ClaudeBytes { url: Box<[u8]> },
    Size { url: Box<[u8]> },
    Digest { url: Box<[u8]>, pinned: [u8; 32], got: [u8; 32] },
}
impl Failure {
    /// Diagnostic words following the forge command's prefix.
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        match self {
            Self::Architecture(arch) => joined(&[b"no pinned binaries for architecture ", arch]),
            Self::Present { file } => joined(&[file, b": does not match the pinned digest"]),
            Self::Status { url, status } => {
                joined(&[url, b": HTTP status ", Decimal::of(u64::from(*status)).as_bytes()])
            }
            Self::Truncated { url } => joined(&[url, b": truncated zstd frame"]),
            Self::Damaged { url } => joined(&[url, b": damaged zstd frame"]),
            Self::UnpackedBytes { url } => joined(&[url, b": unpacked file past setup.fetch_bytes"]),
            Self::Trailing { url } => joined(&[url, b": zstd frame followed by other bytes"]),
            Self::Size { url } => joined(&[url, b": body past its Content-Length"]),
            Self::ClaudeBytes { url } => joined(&[url, b": body past the pinned size"]),
            Self::Bytes { url } => joined(&[url, b": body past setup.fetch_bytes"]),
            Self::Digest { url, pinned, got } => {
                joined(&[url, b": digest differs: pinned ", &hex(pinned), b", got ", &hex(got)])
            }
        }
    }
}
/// Lowercase SHA-256 text.
#[must_use]
pub fn hex(digest: &[u8; 32]) -> Box<[u8]> {
    let mut out = Writer::new(64);
    for byte in digest {
        for digit in [byte >> 4_u8, byte & 15] {
            out.put(&[*b"0123456789abcdef".get(usize::from(digit)).expect("hex digit")]).expect("64 hex digits");
        }
    }
    out.finish()
}
/// The whole URL, with a port only when it differs from HTTPS's default.
#[must_use]
pub fn url(origin: &Origin, path: &[u8]) -> Box<[u8]> {
    if origin.port == 443 {
        joined(&[b"https://", &origin.host, path])
    } else {
        joined(&[b"https://", &origin.host, b":", Decimal::of(u64::from(origin.port)).as_bytes(), path])
    }
}
/// Answers from the host; file effects finish before the next demand.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Begin,
    Done,
    Hashed(Option<[u8; 32]>),
    Answered { status: u16, length: Option<u64>, location: Option<Box<[u8]>> },
    Chunk(Box<[u8]>),
    Ended,
    Unpacked(Unpacked),
}
/// The result of unpacking a bounded zstd frame (temper/docs/design/protocol/setup/owned.md, 6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unpacked {
    File { digest: [u8; 32], bytes: u64 },
    Truncated,
    Damaged,
    TooLong,
    Trailing,
}
/// One effect, answered by one event, or the terminal answer.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Hash { file: Box<[u8]> },
    Get { origin: Origin, path: Box<[u8]> },
    Open { part: Box<[u8]> },
    Read,
    Say(Fetched),
    Write { offset: u64, bytes: Box<[u8]> },
    Unpack { packed: Box<[u8]>, part: Box<[u8]>, bound: u64 },
    Finish { part: Box<[u8]>, file: Box<[u8]> },
    Remove { part: Box<[u8]> },
    End(Result<(), Failure>),
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Begin,
    Hash,
    Head,
    Open,
    Read,
    Write,
    SayRead,
    SayEnd,
    Unpack,
    Finish,
    RemovePart,
    Remove,
    End,
}
/// The fetch's opaque state; a single chunk is moved into its write request.
#[derive(Debug)]
pub struct Fetch {
    pins: Box<[Pin]>,
    index: usize,
    bound: u64,
    phase: Phase,
    total: u64,
    length: Option<u64>,
    said: u64,
    hash: Sha256,
    failure: Option<Failure>,
}
impl Fetch {
    #[must_use]
    pub fn new(pins: Box<[Pin]>, fetch_bytes: u64) -> Self {
        Self {
            pins,
            index: 0,
            bound: fetch_bytes,
            phase: Phase::Begin,
            total: 0,
            length: None,
            said: 0,
            hash: Sha256::new(),
            failure: None,
        }
    }
    fn pin(&self) -> &Pin {
        self.pins.get(self.index).expect("current pin")
    }
    fn part(&self) -> Box<[u8]> {
        joined(&[&self.pin().file, b".part"])
    }
    fn packed(&self) -> Box<[u8]> {
        joined(&[&self.pin().file, b".zst.part"])
    }
    fn download(&self) -> Box<[u8]> {
        if self.pin().packed { self.packed() } else { self.part() }
    }
    fn url(&self) -> Box<[u8]> {
        url(&self.pin().origin, &self.pin().path)
    }
    fn end(&mut self, result: Result<(), Failure>) -> Request {
        self.phase = Phase::End;
        Request::End(result)
    }
    fn next(&mut self) -> Request {
        if self.index == self.pins.len() {
            self.end(Ok(()))
        } else {
            self.phase = Phase::Hash;
            Request::Hash { file: self.pin().file.clone() }
        }
    }
    fn advance(&mut self) -> Request {
        self.index = self.index.checked_add(1).expect("pin count");
        self.next()
    }
    fn remove(&mut self, failure: Failure) -> Request {
        self.failure = Some(failure);
        self.phase = Phase::Remove;
        Request::Remove { part: self.download() }
    }
    fn bytes_failure(&self) -> Failure {
        match self.pin().binary {
            Named::Forge(Binary::Forgejo | Binary::Runner) => Failure::Bytes { url: self.url() },
            Named::ClaudeCode => Failure::ClaudeBytes { url: self.url() },
        }
    }
    fn head(&mut self, status: u16, length: Option<u64>) -> Request {
        if status != 200 {
            self.end(Err(Failure::Status { url: self.url(), status }))
        } else if match length {
            Some(length) => length > self.bound,
            None => false,
        } {
            self.end(Err(self.bytes_failure()))
        } else {
            self.length = match length {
                Some(0) | None => None,
                Some(length) => Some(length),
            };
            self.phase = Phase::Open;
            Request::Open { part: self.download() }
        }
    }
    fn body(&mut self, event: Event) -> Request {
        match event {
            Event::Chunk(bytes) => {
                let length = skein_lib::len_u64(bytes.len());
                if length > self.bound.saturating_sub(self.total) {
                    self.remove(self.bytes_failure())
                } else if match self.length {
                    Some(total) => length > total.saturating_sub(self.total),
                    None => false,
                } {
                    self.remove(Failure::Size { url: self.url() })
                } else {
                    let offset = self.total;
                    self.total = offset.checked_add(length).expect("admitted length");
                    if !self.pin().packed {
                        self.hash.update(&bytes);
                    }
                    self.phase = Phase::Write;
                    Request::Write { offset, bytes }
                }
            }
            Event::Ended => {
                if self.total == self.said {
                    self.verdict()
                } else {
                    self.say(Phase::SayEnd)
                }
            }
            Event::Begin | Event::Done | Event::Hashed(_) | Event::Answered { .. } | Event::Unpacked(_) => {
                unreachable!("body answer")
            }
        }
    }
    fn say(&mut self, phase: Phase) -> Request {
        self.said = self.total;
        self.phase = phase;
        Request::Say(Fetched { binary: self.pin().binary, done: self.total, total: self.length })
    }
    fn figure(&self, done: u64) -> u128 {
        match self.length {
            Some(total) => u128::from(done)
                .checked_mul(100)
                .expect("u64 percentage fits u128")
                .checked_div(u128::from(total))
                .expect("nonzero size"),
            None => u128::from(done.checked_div(1_048_576).expect("one MiB")),
        }
    }
    fn read(&mut self) -> Request {
        self.phase = Phase::Read;
        Request::Read
    }
    fn verdict(&mut self) -> Request {
        if self.pin().packed {
            self.phase = Phase::Unpack;
            return Request::Unpack { packed: self.packed(), part: self.part(), bound: self.bound };
        }
        let got = core::mem::replace(&mut self.hash, Sha256::new()).finish();
        let pinned = self.pin().digest;
        if got == pinned { self.finish() } else { self.remove(Failure::Digest { url: self.url(), pinned, got }) }
    }
    fn finish(&mut self) -> Request {
        self.phase = Phase::Finish;
        Request::Finish { part: self.part(), file: self.pin().file.clone() }
    }
    fn unpacked(&mut self, unpacked: Unpacked) -> Request {
        let pinned = self.pin().digest;
        let failure = match unpacked {
            Unpacked::File { digest, bytes } if bytes <= self.bound => {
                if digest == pinned {
                    return self.finish();
                }
                Failure::Digest { url: self.url(), pinned, got: digest }
            }
            Unpacked::File { .. } | Unpacked::TooLong => Failure::UnpackedBytes { url: self.url() },
            Unpacked::Truncated => Failure::Truncated { url: self.url() },
            Unpacked::Damaged => Failure::Damaged { url: self.url() },
            Unpacked::Trailing => Failure::Trailing { url: self.url() },
        };
        self.failure = Some(failure);
        self.phase = Phase::RemovePart;
        Request::Remove { part: self.part() }
    }
}
/// Consumes a host answer and emits exactly one next request.
pub fn step(machine: &mut Fetch, event: Event, out: &mut Queue<Request>) {
    let request = match machine.phase {
        Phase::Begin => {
            let Event::Begin = event else { unreachable!("fetch begins once") };
            machine.next()
        }
        Phase::Hash => {
            let Event::Hashed(digest) = event else { unreachable!("hash answer") };
            match digest {
                Some(digest) if digest == machine.pin().digest => machine.advance(),
                Some(_) => machine.end(Err(Failure::Present { file: machine.pin().file.clone() })),
                None => {
                    machine.phase = Phase::Head;
                    Request::Get { origin: machine.pin().origin.clone(), path: machine.pin().path.clone() }
                }
            }
        }
        Phase::Head => {
            let Event::Answered { status, length, .. } = event else { unreachable!("response head") };
            machine.head(status, length)
        }
        Phase::Unpack => {
            let Event::Unpacked(unpacked) = event else { unreachable!("unpack answer") };
            machine.unpacked(unpacked)
        }
        Phase::Read => machine.body(event),
        phase @ (Phase::Open
        | Phase::Write
        | Phase::SayRead
        | Phase::SayEnd
        | Phase::Finish
        | Phase::RemovePart
        | Phase::Remove) => {
            let Event::Done = event else { unreachable!("file effect done") };
            match phase {
                Phase::Open => {
                    machine.total = 0;
                    machine.hash = Sha256::new();
                    machine.say(Phase::SayRead)
                }
                Phase::Write => {
                    if machine.figure(machine.total) > machine.figure(machine.said) {
                        machine.say(Phase::SayRead)
                    } else {
                        machine.read()
                    }
                }
                Phase::SayRead => machine.read(),
                Phase::SayEnd => machine.verdict(),
                Phase::Finish if !machine.pin().packed => machine.advance(),
                Phase::Finish | Phase::RemovePart => {
                    machine.phase = Phase::Remove;
                    Request::Remove { part: machine.packed() }
                }
                Phase::Remove => match machine.failure.take() {
                    Some(failure) => machine.end(Err(failure)),
                    None => machine.advance(),
                },
                Phase::Begin | Phase::Hash | Phase::Head | Phase::Read | Phase::Unpack | Phase::End => {
                    unreachable!("handled above")
                }
            }
        }
        Phase::End => unreachable!("fetch ended"),
    };
    out.push(request);
}
