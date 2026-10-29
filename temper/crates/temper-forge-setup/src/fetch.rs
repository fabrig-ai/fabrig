//! Streaming pinned binary fetch (protocol/setup.md, 6.6).
use crate::{
    check::joined,
    setup::{Origin, Owned},
};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::{self, Binary};
use skein_lib::{Decimal, List, Queue, Writer, sha256::Sha256};

/// One pinned release at its resolved cache path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pin {
    pub origin: Origin,
    pub path: Box<[u8]>,
    pub digest: [u8; 32],
    pub file: Box<[u8]>,
}
/// Why the fetch was refused; no existing binary is replaced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Architecture(Box<[u8]>),
    Present { file: Box<[u8]> },
    Status { url: Box<[u8]>, status: u16 },
    Bytes { url: Box<[u8]> },
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
fn digest(binary: Binary, arch: &str) -> [u8; 32] {
    let mut result = [0; 32];
    let text = pin::digest(binary, arch).expect("pinned architecture").as_bytes();
    for (slot, digits) in result.iter_mut().zip(text.chunks_exact(2)) {
        let mut value = 0_u8;
        for digit in digits {
            let n = if digit.is_ascii_digit() {
                digit.checked_sub(b'0').expect("digit")
            } else {
                digit.checked_sub(b'a').expect("hex digit").checked_add(10).expect("hex digit")
            };
            value = value.checked_mul(16).expect("byte").checked_add(n).expect("byte");
        }
        *slot = value;
    }
    result
}
/// Selects release paths and digests for the host's architecture.
pub fn pins(owned: &Owned, machine: &str) -> Result<Box<[Pin]>, Failure> {
    let arch = match machine {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(Failure::Architecture(machine.as_bytes().into())),
    };
    let mut pins = List::with_capacity(2);
    for binary in [Binary::Forgejo, Binary::Runner] {
        let (fetch, version, name, default, file) = match binary {
            Binary::Forgejo => {
                (owned.fetch.forgejo, pin::VERSION, b"forgejo".as_slice(), b"codeberg.org".as_slice(), &owned.forgejo)
            }
            Binary::Runner => {
                if !owned.fetch.runner {
                    continue;
                }
                (
                    true,
                    pin::RUNNER_VERSION,
                    b"forgejo-runner".as_slice(),
                    b"code.forgejo.org".as_slice(),
                    &owned.runner.as_ref().expect("fetched runner exists").binary,
                )
            }
        };
        if !fetch {
            continue;
        }
        let project = match binary {
            Binary::Forgejo => b"forgejo".as_slice(),
            Binary::Runner => b"runner".as_slice(),
        };
        let origin = match &owned.fetch.mirror {
            Some(mirror) => match binary {
                Binary::Forgejo => mirror.forgejo.clone(),
                Binary::Runner => mirror.runner.clone(),
            },
            None => Origin { host: default.into(), port: 443 },
        };
        let path = joined(&[
            b"/forgejo/",
            project,
            b"/releases/download/v",
            version.as_bytes(),
            b"/",
            name,
            b"-",
            version.as_bytes(),
            b"-linux-",
            arch.as_bytes(),
        ]);
        pins.push(Pin { origin, path, digest: digest(binary, arch), file: file.clone() }).expect("two pins");
    }
    Ok(pins.into_boxed())
}
/// Answers from the host; file effects finish before the next demand.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Begin,
    Done,
    Hashed(Option<[u8; 32]>),
    Answered { status: u16, length: Option<u64> },
    Chunk(Box<[u8]>),
    Ended,
}
/// One effect, answered by one event, or the terminal answer.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Hash { file: Box<[u8]> },
    Get { origin: Origin, path: Box<[u8]> },
    Open { part: Box<[u8]> },
    Read,
    Write { offset: u64, bytes: Box<[u8]> },
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
    Finish,
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
    hash: Sha256,
    failure: Option<Failure>,
}
impl Fetch {
    #[must_use]
    pub fn new(pins: Box<[Pin]>, fetch_bytes: u64) -> Self {
        Self { pins, index: 0, bound: fetch_bytes, phase: Phase::Begin, total: 0, hash: Sha256::new(), failure: None }
    }
    fn pin(&self) -> &Pin {
        self.pins.get(self.index).expect("current pin")
    }
    fn part(&self) -> Box<[u8]> {
        joined(&[&self.pin().file, b".part"])
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
            let Event::Answered { status, length } = event else { unreachable!("response head") };
            if status != 200 {
                machine.end(Err(Failure::Status { url: machine.url(), status }))
            } else if match length {
                Some(length) => length > machine.bound,
                None => false,
            } {
                machine.end(Err(Failure::Bytes { url: machine.url() }))
            } else {
                machine.phase = Phase::Open;
                Request::Open { part: machine.part() }
            }
        }
        Phase::Read => match event {
            Event::Chunk(bytes) => {
                let length = u64::try_from(bytes.len()).expect("chunk length fits");
                if length > machine.bound.saturating_sub(machine.total) {
                    machine.remove(Failure::Bytes { url: machine.url() })
                } else {
                    let offset = machine.total;
                    machine.total = offset.checked_add(length).expect("admitted length");
                    machine.hash.update(&bytes);
                    machine.phase = Phase::Write;
                    Request::Write { offset, bytes }
                }
            }
            Event::Ended => {
                let got = core::mem::replace(&mut machine.hash, Sha256::new()).finish();
                if got == machine.pin().digest {
                    machine.phase = Phase::Finish;
                    Request::Finish { part: machine.part(), file: machine.pin().file.clone() }
                } else {
                    machine.remove(Failure::Digest { url: machine.url(), pinned: machine.pin().digest, got })
                }
            }
            Event::Begin | Event::Done | Event::Hashed(_) | Event::Answered { .. } => unreachable!("body answer"),
        },
        phase @ (Phase::Open | Phase::Write | Phase::Finish | Phase::Remove) => {
            let Event::Done = event else { unreachable!("file effect done") };
            match phase {
                Phase::Open => {
                    machine.total = 0;
                    machine.hash = Sha256::new();
                    machine.phase = Phase::Read;
                    Request::Read
                }
                Phase::Write => {
                    machine.phase = Phase::Read;
                    Request::Read
                }
                Phase::Finish => machine.advance(),
                Phase::Remove => {
                    let failure = machine.failure.take().expect("removal refusal");
                    machine.end(Err(failure))
                }
                Phase::Begin | Phase::Hash | Phase::Head | Phase::Read | Phase::End => unreachable!("handled above"),
            }
        }
        Phase::End => unreachable!("fetch ended"),
    };
    out.push(request);
}
