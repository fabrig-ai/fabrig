//! Streaming pinned binary fetch (protocol/setup.md, 6.6).
use crate::{
    check::joined,
    setup::{Origin, Owned},
};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::{self, Binary, Form, Platform};
use skein_lib::{Decimal, List, Queue, Writer, sha256::Sha256};

/// One pinned release at its resolved cache path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pin {
    pub origin: Origin,
    pub path: Box<[u8]>,
    pub digest: [u8; 32],
    pub file: Box<[u8]>,
    pub bottle: Option<Bottle>,
}
/// The verified archive and its one binary member (protocol/setup.md, 6.6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bottle {
    pub digest: [u8; 32],
    pub member: Box<[u8]>,
    pub token: bool,
}
/// Why the fetch was refused; no existing binary is replaced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Architecture(Box<[u8]>),
    Present { file: Box<[u8]> },
    Status { url: Box<[u8]>, status: u16 },
    Redirect { url: Box<[u8]> },
    Gone { url: Box<[u8]>, status: u16 },
    Extraction { url: Box<[u8]> },
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
            Self::Redirect { url } => joined(&[url, b": invalid or repeated redirect"]),
            Self::Gone { url, status } => joined(&[
                url,
                b": HTTP status ",
                Decimal::of(u64::from(*status)).as_bytes(),
                b": give the binary by path, or a mirror",
            ]),
            Self::Extraction { url } => joined(&[url, b": extraction failed"]),
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
fn digest(binary: Binary, platform: Platform, arch: &str, form: Form) -> [u8; 32] {
    let mut result = [0; 32];
    let text = pin::digest(binary, platform, arch, form).expect("pinned architecture").as_bytes();
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
pub fn pins(owned: &Owned, platform: Platform, machine: &str) -> Result<Box<[Pin]>, Failure> {
    let arch = match machine {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(Failure::Architecture(machine.as_bytes().into())),
    };
    if platform == Platform::Macos && arch != "arm64" {
        return Err(Failure::Architecture(machine.as_bytes().into()));
    }
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
            None => Origin {
                host: match platform {
                    Platform::Linux => default.into(),
                    Platform::Macos => b"ghcr.io".as_slice().into(),
                },
                port: 443,
            },
        };
        let linux_path = joined(&[
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
        let (path, bottle) = match platform {
            Platform::Linux => (linux_path, None),
            Platform::Macos => {
                let archive = digest(binary, platform, arch, Form::Archive);
                let path = joined(&[b"/v2/homebrew/core/", name, b"/blobs/sha256:", &hex(&archive)]);
                let member = joined(&[name, b"/", version.as_bytes(), b"/bin/", name]);
                (path, Some(Bottle { digest: archive, member, token: owned.fetch.mirror.is_none() }))
            }
        };
        pins.push(Pin {
            origin,
            path,
            digest: digest(binary, platform, arch, Form::Binary),
            file: file.clone(),
            bottle,
        })
        .expect("two pins");
    }
    Ok(pins.into_boxed())
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
    Extracted(Extraction),
}
/// The bounded extraction's result.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Extraction {
    Digest([u8; 32]),
    Bytes,
    Failed,
}
/// One effect, answered by one event, or the terminal answer.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Hash { file: Box<[u8]> },
    Get { origin: Origin, path: Box<[u8]>, authorization: Option<Box<[u8]>> },
    Open { part: Box<[u8]> },
    Read,
    Write { offset: u64, bytes: Box<[u8]> },
    Extract { archive: Box<[u8]>, member: Box<[u8]>, part: Box<[u8]>, bound: u64 },
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
    Extract,
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
    hash: Sha256,
    failure: Option<Failure>,
    redirected: bool,
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
            hash: Sha256::new(),
            failure: None,
            redirected: false,
        }
    }
    fn pin(&self) -> &Pin {
        self.pins.get(self.index).expect("current pin")
    }
    fn part(&self) -> Box<[u8]> {
        joined(&[&self.pin().file, b".part"])
    }
    fn archive(&self) -> Box<[u8]> {
        joined(&[&self.pin().file, b".archive"])
    }
    fn download(&self) -> Box<[u8]> {
        match &self.pin().bottle {
            Some(_) => self.archive(),
            None => self.part(),
        }
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
            self.redirected = false;
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
    fn head(&mut self, status: u16, length: Option<u64>, location: Option<Box<[u8]>>) -> Request {
        if self.pin().bottle.is_some() && [301, 302, 303, 307, 308].contains(&status) && location.is_some() {
            if !self.redirected
                && let Some(location) = location
                && let Some((origin, path)) = redirect(&location)
            {
                self.redirected = true;
                return Request::Get { origin, path, authorization: None };
            }
            return self.end(Err(Failure::Redirect { url: self.url() }));
        }
        if status != 200 {
            let failure = match self.pin().bottle {
                Some(_) => Failure::Gone { url: self.url(), status },
                None => Failure::Status { url: self.url(), status },
            };
            self.end(Err(failure))
        } else if match length {
            Some(length) => length > self.bound,
            None => false,
        } {
            self.end(Err(Failure::Bytes { url: self.url() }))
        } else {
            self.phase = Phase::Open;
            Request::Open { part: self.download() }
        }
    }
    fn body(&mut self, event: Event) -> Request {
        match event {
            Event::Chunk(bytes) => {
                let length = skein_lib::len_u64(bytes.len());
                if length > self.bound.saturating_sub(self.total) {
                    self.remove(Failure::Bytes { url: self.url() })
                } else {
                    let offset = self.total;
                    self.total = offset.checked_add(length).expect("admitted length");
                    self.hash.update(&bytes);
                    self.phase = Phase::Write;
                    Request::Write { offset, bytes }
                }
            }
            Event::Ended => {
                let got = core::mem::replace(&mut self.hash, Sha256::new()).finish();
                let pinned = match &self.pin().bottle {
                    Some(bottle) => bottle.digest,
                    None => self.pin().digest,
                };
                if got != pinned {
                    self.remove(Failure::Digest { url: self.url(), pinned, got })
                } else if let Some(bottle) = &self.pin().bottle {
                    let request = Request::Extract {
                        archive: self.archive(),
                        member: bottle.member.clone(),
                        part: self.part(),
                        bound: self.bound,
                    };
                    self.phase = Phase::Extract;
                    request
                } else {
                    self.phase = Phase::Finish;
                    Request::Finish { part: self.part(), file: self.pin().file.clone() }
                }
            }
            Event::Begin | Event::Done | Event::Hashed(_) | Event::Answered { .. } | Event::Extracted(_) => {
                unreachable!("body answer")
            }
        }
    }
    fn extracted(&mut self, extraction: Extraction) -> Request {
        let failure = match extraction {
            Extraction::Digest(got) if got == self.pin().digest => {
                self.phase = Phase::Finish;
                return Request::Finish { part: self.part(), file: self.pin().file.clone() };
            }
            Extraction::Digest(got) => Failure::Digest { url: self.url(), pinned: self.pin().digest, got },
            Extraction::Bytes => Failure::Bytes { url: self.url() },
            Extraction::Failed => Failure::Extraction { url: self.url() },
        };
        self.failure = Some(failure);
        self.phase = Phase::RemovePart;
        Request::Remove { part: self.part() }
    }
}
fn redirect(location: &[u8]) -> Option<(Origin, Box<[u8]>)> {
    let rest = location.strip_prefix(b"https://")?;
    let slash = skein_lib::bytes::find(rest, b"/")?;
    for byte in rest.get(..slash)? {
        if b"?#".contains(byte) {
            return None;
        }
    }
    let parsed = skein_forge::github::redirect::parse(location, u32::MAX, u32::MAX).ok()?;
    if parsed.origin.port == 0 {
        return None;
    }
    Some((Origin { host: parsed.origin.host, port: parsed.origin.port }, rest.get(slash..)?.into()))
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
                    Request::Get {
                        origin: machine.pin().origin.clone(),
                        path: machine.pin().path.clone(),
                        authorization: match &machine.pin().bottle {
                            Some(bottle) if bottle.token => Some(b"Bearer QQ==".as_slice().into()),
                            Some(_) | None => None,
                        },
                    }
                }
            }
        }
        Phase::Head => {
            let Event::Answered { status, length, location } = event else { unreachable!("response head") };
            machine.head(status, length, location)
        }
        Phase::Extract => {
            let Event::Extracted(extraction) = event else { unreachable!("extraction answer") };
            machine.extracted(extraction)
        }
        Phase::Read => machine.body(event),
        phase @ (Phase::Open | Phase::Write | Phase::Finish | Phase::RemovePart | Phase::Remove) => {
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
                Phase::Finish => match machine.pin().bottle {
                    Some(_) => {
                        machine.phase = Phase::Remove;
                        Request::Remove { part: machine.archive() }
                    }
                    None => machine.advance(),
                },
                Phase::RemovePart => {
                    machine.phase = Phase::Remove;
                    Request::Remove { part: machine.archive() }
                }
                Phase::Remove => match machine.failure.take() {
                    Some(failure) => machine.end(Err(failure)),
                    None => machine.advance(),
                },
                Phase::Begin | Phase::Hash | Phase::Head | Phase::Read | Phase::Extract | Phase::End => {
                    unreachable!("handled above")
                }
            }
        }
        Phase::End => unreachable!("fetch ended"),
    };
    out.push(request);
}
