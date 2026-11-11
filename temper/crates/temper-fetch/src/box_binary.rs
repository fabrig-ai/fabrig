//! Finding this build's temper-box (temper/docs/design/client/box-binary.md, 2, 3, 6).
use crate::{
    Origin,
    fetch::{hex, url},
    joined,
};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Queue, sha256::Sha256};

pub const SUMS_BYTES: u64 = 65_536;
pub const SERVED_BYTES: u64 = 134_217_728;
pub const UNPACKED_BYTES: u64 = 536_870_912;

/// The exact stdout of this build's candidate.
#[must_use]
pub fn identity(version: &str, commit: &str) -> Box<[u8]> {
    joined(&[b"temper-box ", version.as_bytes(), b" (", commit.as_bytes(), b")\n"])
}
/// Only a release build name may download.
#[must_use]
pub fn clean(commit: &[u8]) -> bool {
    if commit.len() != 12 {
        return false;
    }
    for byte in commit {
        if !b"0123456789abcdef".contains(byte) {
            return false;
        }
    }
    true
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sums {
    Missing,
    Doubled,
    Malformed,
    TooLong,
}

fn digest(text: &[u8]) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut out = [0; 32];
    for (pair, byte) in text.chunks_exact(2).zip(out.iter_mut()) {
        let mut value = 0_u8;
        for digit in pair {
            let n = match digit {
                b'0'..=b'9' => digit.checked_sub(b'0')?,
                b'a'..=b'f' => digit.checked_sub(b'a')?.checked_add(10)?,
                _ => return None,
            };
            value = value.checked_mul(16)?.checked_add(n)?;
        }
        *byte = value;
    }
    Some(out)
}
/// Exactly one sha256sum row names this file; unrelated digests are not parsed.
pub fn sum(sums: &[u8], name: &[u8]) -> Result<[u8; 32], Sums> {
    if skein_lib::len_u64(sums.len()) > SUMS_BYTES {
        return Err(Sums::TooLong);
    }
    let mut found = None;
    let mut start = 0;
    for end in 0..=sums.len() {
        if end != sums.len() && sums.get(end) != Some(&b'\n') {
            continue;
        }
        let line = sums.get(start..end).expect("line within sums");
        start = end.saturating_add(1);
        let mut space = None;
        for (at, byte) in line.iter().enumerate() {
            if byte.is_ascii_whitespace() {
                space = Some(at);
                break;
            }
        }
        let Some(at) = space else { continue };
        let tail = line.get(at..).expect("position in line").trim_ascii_start();
        let tail = tail.strip_prefix(b"*").unwrap_or(tail);
        if tail != name {
            continue;
        }
        if found.is_some() {
            return Err(Sums::Doubled);
        }
        if line.get(64..66) != Some(b"  ") && line.get(64..66) != Some(b" *") {
            return Err(Sums::Malformed);
        }
        if line.get(66..) != Some(name) {
            return Err(Sums::Malformed);
        }
        let text = line.get(..64).ok_or(Sums::Malformed)?;
        let parsed = digest(text).ok_or(Sums::Malformed)?;
        found = Some(parsed);
    }
    found.ok_or(Sums::Missing)
}

/// All paths are native host bytes; the machine never consults an environment.
#[derive(Debug)]
pub struct Build {
    pub beside: Box<[u8]>,
    pub cache: Box<[u8]>,
    pub site: Origin,
    pub version: Box<str>,
    pub commit: Box<str>,
    pub platform: Box<[u8]>,
    pub beside_exists: bool,
}
/// Candidate stdout is bounded by the host even when its exit is not successful.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Candidate {
    pub line: Box<[u8]>,
    pub success: bool,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Checkout { version: Box<str>, commit: Box<str>, beside: Box<[u8]> },
    Platform(Box<[u8]>),
    Identity { file: Box<[u8]>, line: Box<[u8]> },
    Download { url: Box<[u8]>, why: Box<[u8]> },
}
impl Failure {
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        match self {
            Self::Checkout { version, commit, beside } => joined(&[
                b"no temper-box of this build (",
                version.as_bytes(),
                b" (",
                commit.as_bytes(),
                b")) beside ",
                beside,
                b"/temper; a build from a checkout downloads none: build it with cargo build -p temper-box",
            ]),
            Self::Platform(platform) => joined(&[b"no temper-box download for target ", platform]),
            Self::Identity { file, line } => joined(&[file, b": temper-box identity differs: ", line]),
            Self::Download { url, why } => joined(&[b"temper-box: ", url, b": ", why]),
        }
    }
}
/// Progress has the forge fetch's cadence and spelling, but a different binary name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Progress {
    pub done: u64,
    pub total: Option<u64>,
}
impl Progress {
    #[must_use]
    pub fn line(&self) -> Box<[u8]> {
        let done = Decimal::of(self.done);
        match self.total {
            Some(total) => {
                joined(&[b"fetched temper-box ", done.as_bytes(), b" of ", Decimal::of(total).as_bytes(), b" bytes\n"])
            }
            None => joined(&[b"fetched temper-box ", done.as_bytes(), b" bytes\n"]),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unpacked {
    File { digest: [u8; 32], bytes: u64 },
    TooLong,
    Invalid,
}
/// One answer per request. Chunk bytes have not yet been written.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Begin,
    Done,
    Identity(Candidate),
    Cached { file: Option<[u8; 32]>, sum: Option<[u8; 32]> },
    Answered { status: u16, length: Option<u64>, location: Option<Box<[u8]>> },
    Chunk(Box<[u8]>),
    Ended,
    Unpacked(Unpacked),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Identity { file: Box<[u8]> },
    Lock { directory: Box<[u8]> },
    Clean { directory: Box<[u8]> },
    Cached { file: Box<[u8]> },
    RemoveCached { file: Box<[u8]> },
    Get { origin: Origin, path: Box<[u8]> },
    Open { file: Box<[u8]> },
    Read,
    Write(Box<[u8]>),
    Say(Progress),
    Unpack { archive: Box<[u8]>, part: Box<[u8]>, bound: u64 },
    WriteSum { file: Box<[u8]>, digest: [u8; 32] },
    RenameSum { file: Box<[u8]> },
    Publish { file: Box<[u8]> },
    End(Result<Box<[u8]>, Failure>),
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Begin,
    Beside,
    Lock,
    Clean,
    Cached,
    CacheIdentity,
    RemoveCached,
    Head,
    Read,
    Open,
    Write,
    SayRead,
    SayEnd,
    Unpack,
    PartIdentity,
    WriteSum,
    RenameSum,
    Publish,
    Refuse,
    End,
}
/// One chunk in flight, with bounded sums retained until their end.
#[derive(Debug)]
pub struct Find {
    build: Build,
    directory: Box<[u8]>,
    file: Box<[u8]>,
    phase: Phase,
    failure: Option<Failure>,
    sums: List<u8>,
    fetching_sums: bool,
    path: Box<[u8]>,
    redirects: u8,
    total: u64,
    length: Option<u64>,
    said: u64,
    hash: Sha256,
    served: [u8; 32],
    unpacked: [u8; 32],
}
impl Find {
    #[must_use]
    pub fn new(build: Build) -> Self {
        let directory = joined(&[&build.cache, b"/bin/", build.commit.as_bytes(), b"/", &build.platform]);
        let file = joined(&[&directory, b"/temper-box"]);
        Self {
            build,
            directory,
            file,
            phase: Phase::Begin,
            failure: None,
            sums: List::with_capacity(65_536),
            fetching_sums: true,
            path: Box::new([]),
            redirects: 0,
            total: 0,
            length: None,
            said: 0,
            hash: Sha256::new(),
            served: [0; 32],
            unpacked: [0; 32],
        }
    }
    fn end(&mut self, answer: Result<Box<[u8]>, Failure>) -> Request {
        self.phase = Phase::End;
        Request::End(answer)
    }
    fn refuse(&mut self, why: Failure) -> Request {
        self.failure = Some(why);
        self.phase = Phase::Refuse;
        Request::Clean { directory: self.directory.clone() }
    }
    fn download_failure(&mut self, why: Box<[u8]>) -> Request {
        self.refuse(Failure::Download { url: url(&self.build.site, &self.path), why })
    }
    fn cache(&mut self) -> Request {
        if !clean(self.build.commit.as_bytes()) {
            return self.end(Err(Failure::Checkout {
                version: self.build.version.clone(),
                commit: self.build.commit.clone(),
                beside: self.build.beside.clone(),
            }));
        }
        self.phase = Phase::Lock;
        Request::Lock { directory: self.directory.clone() }
    }
    fn get(&mut self) -> Request {
        self.phase = Phase::Head;
        Request::Get { origin: self.build.site.clone(), path: self.path.clone() }
    }
    fn fetch(&mut self, sums: bool) -> Request {
        if ![b"Linux-x86_64".as_slice(), b"Linux-aarch64", b"Darwin-arm64", b"Darwin-x86_64"]
            .contains(&self.build.platform.as_ref())
        {
            return self.refuse(Failure::Platform(self.build.platform.clone()));
        }
        self.fetching_sums = sums;
        self.redirects = 0;
        let major = self.build.version.split('.').next().expect("version component");
        let root = joined(&[b"/temper/v", major.as_bytes(), b"/", self.build.commit.as_bytes(), b"/"]);
        self.path =
            if sums { joined(&[&root, b"SHA256SUMS"]) } else { joined(&[&root, &self.build.platform, b"/temper-box"]) };
        self.get()
    }
    fn head(&mut self, status: u16, length: Option<u64>, location: Option<Box<[u8]>>) -> Request {
        if [301, 302, 303, 307, 308].contains(&status) {
            if let Some(location) = location {
                let current = url(&self.build.site, &self.path);
                if self.redirects < 3
                    && let Some(path) = same_origin(&self.build.site, &location)
                {
                    self.redirects = self.redirects.checked_add(1).expect("three redirects");
                    self.path = path;
                    return self.get();
                }
                return self.download_failure(joined(&[b"redirect refused from ", &current, b" to ", &location]));
            }
            return self.download_failure(b"redirect without Location".as_slice().into());
        }
        if status != 200 {
            return self.download_failure(joined(&[b"HTTP status ", Decimal::of(u64::from(status)).as_bytes()]));
        }
        let bound = self.bound();
        if let Some(length) = length
            && length > bound
        {
            return self.download_failure(b"Content-Length past byte bound".as_slice().into());
        }
        self.total = 0;
        self.said = 0;
        self.length = length;
        self.hash = Sha256::new();
        if self.fetching_sums {
            self.read()
        } else {
            self.phase = Phase::Open;
            Request::Open { file: joined(&[&self.file, b".gz.part"]) }
        }
    }
    fn bound(&self) -> u64 {
        if self.fetching_sums { SUMS_BYTES } else { SERVED_BYTES }
    }
    fn read(&mut self) -> Request {
        self.phase = Phase::Read;
        Request::Read
    }
    fn say(&mut self, end: bool) -> Request {
        self.phase = if end { Phase::SayEnd } else { Phase::SayRead };
        self.said = self.total;
        Request::Say(Progress {
            done: self.total,
            total: match self.length {
                Some(0) | None => None,
                Some(n) => Some(n),
            },
        })
    }
    fn figure(&self, done: u64) -> u64 {
        match self.length {
            Some(total) if total != 0 => {
                done.checked_mul(100).expect("bounded body").checked_div(total).expect("nonzero")
            }
            Some(_) | None => done.checked_div(1_048_576).expect("MiB"),
        }
    }
    fn body(&mut self, event: Event) -> Request {
        match event {
            Event::Chunk(bytes) => {
                let n = skein_lib::len_u64(bytes.len());
                if n > self.bound().saturating_sub(self.total) {
                    return self.download_failure(b"body past byte bound".as_slice().into());
                }
                if let Some(length) = self.length
                    && n > length.saturating_sub(self.total)
                {
                    return self.download_failure(b"body past its Content-Length".as_slice().into());
                }
                self.total = self.total.checked_add(n).expect("bounded body");
                if self.fetching_sums {
                    for byte in bytes.as_ref() {
                        self.sums.push(*byte).expect("sums admitted within bound");
                    }
                    self.read()
                } else {
                    self.hash.update(&bytes);
                    self.phase = Phase::Write;
                    Request::Write(bytes)
                }
            }
            Event::Ended => {
                if self.fetching_sums {
                    let name = joined(&[&self.build.platform, b"/temper-box"]);
                    match sum(self.sums.as_slice(), &name) {
                        Ok(digest) => {
                            self.served = digest;
                            self.sums.clear();
                            self.fetch(false)
                        }
                        Err(why) => self.download_failure(sums_words(why)),
                    }
                } else if self.total != self.said {
                    self.say(true)
                } else {
                    self.verify()
                }
            }
            Event::Begin
            | Event::Done
            | Event::Identity(_)
            | Event::Cached { .. }
            | Event::Answered { .. }
            | Event::Unpacked(_) => unreachable!("body event"),
        }
    }
    fn verify(&mut self) -> Request {
        let got = core::mem::replace(&mut self.hash, Sha256::new()).finish();
        if got != self.served {
            return self.download_failure(joined(&[
                b"digest differs: wanted ",
                &hex(&self.served),
                b", got ",
                &hex(&got),
            ]));
        }
        self.phase = Phase::Unpack;
        Request::Unpack {
            archive: joined(&[&self.file, b".gz.part"]),
            part: joined(&[&self.file, b".part"]),
            bound: UNPACKED_BYTES,
        }
    }
    fn candidate(&self, candidate: &Candidate) -> bool {
        candidate.success && candidate.line == identity(&self.build.version, &self.build.commit)
    }
    fn wrong_identity(file: Box<[u8]>, candidate: Candidate) -> Failure {
        Failure::Identity { file, line: candidate.line }
    }
}
fn sums_words(why: Sums) -> Box<[u8]> {
    Box::from(match why {
        Sums::Missing => b"SHA256SUMS: missing temper-box line".as_slice(),
        Sums::Doubled => b"SHA256SUMS: doubled temper-box line",
        Sums::Malformed => b"SHA256SUMS: malformed temper-box line",
        Sums::TooLong => b"SHA256SUMS: past 64 KiB",
    })
}
fn same_origin(origin: &Origin, location: &[u8]) -> Option<Box<[u8]>> {
    if location.starts_with(b"/") && !location.starts_with(b"//") {
        return Some(location.into());
    }
    let rest = location.strip_prefix(b"https://")?;
    let slash = skein_lib::bytes::find(rest, b"/")?;
    let authority = rest.get(..slash)?;
    let wanted = url(origin, b"");
    let wanted = wanted.strip_prefix(b"https://")?;
    let default = joined(&[&origin.host, b":443"]);
    if !(authority.eq_ignore_ascii_case(wanted) || origin.port == 443 && authority.eq_ignore_ascii_case(&default)) {
        return None;
    }
    Some(rest.get(slash..)?.into())
}
/// Consume one host event and emit one request, including at refusal and success.
#[expect(clippy::too_many_lines, reason = "one exhaustive transition table keeps each effect beside its next phase")]
pub fn step(machine: &mut Find, event: Event, out: &mut Queue<Request>) {
    let request = match machine.phase {
        Phase::Begin => {
            let Event::Begin = event else { unreachable!("begin once") };
            if machine.build.beside_exists {
                machine.phase = Phase::Beside;
                Request::Identity { file: joined(&[&machine.build.beside, b"/temper-box"]) }
            } else {
                machine.cache()
            }
        }
        Phase::Beside => {
            let Event::Identity(candidate) = event else { unreachable!("identity answer") };
            if machine.candidate(&candidate) {
                machine.end(Ok(joined(&[&machine.build.beside, b"/temper-box"])))
            } else {
                machine.cache()
            }
        }
        Phase::Cached => {
            let Event::Cached { file, sum } = event else { unreachable!("cache answer") };
            if file.is_some() && file == sum {
                machine.phase = Phase::CacheIdentity;
                Request::Identity { file: machine.file.clone() }
            } else if file.is_some() || sum.is_some() {
                machine.phase = Phase::RemoveCached;
                Request::RemoveCached { file: machine.file.clone() }
            } else {
                machine.fetch(true)
            }
        }
        Phase::CacheIdentity => {
            let Event::Identity(candidate) = event else { unreachable!("identity answer") };
            if machine.candidate(&candidate) {
                machine.end(Ok(machine.file.clone()))
            } else {
                machine.failure = Some(Find::wrong_identity(machine.file.clone(), candidate));
                machine.phase = Phase::RemoveCached;
                Request::RemoveCached { file: machine.file.clone() }
            }
        }
        Phase::Head => {
            let Event::Answered { status, length, location } = event else { unreachable!("head answer") };
            machine.head(status, length, location)
        }
        Phase::Read => machine.body(event),
        Phase::Unpack => {
            let Event::Unpacked(result) = event else { unreachable!("unpack answer") };
            match result {
                Unpacked::File { digest, bytes } if bytes <= UNPACKED_BYTES => {
                    machine.unpacked = digest;
                    machine.phase = Phase::PartIdentity;
                    Request::Identity { file: joined(&[&machine.file, b".part"]) }
                }
                Unpacked::File { .. } | Unpacked::TooLong => {
                    machine.download_failure(b"unpacked file past 512 MiB".as_slice().into())
                }
                Unpacked::Invalid => machine.download_failure(b"invalid gzip stream".as_slice().into()),
            }
        }
        Phase::PartIdentity => {
            let Event::Identity(candidate) = event else { unreachable!("identity answer") };
            if machine.candidate(&candidate) {
                machine.phase = Phase::WriteSum;
                Request::WriteSum { file: machine.file.clone(), digest: machine.unpacked }
            } else {
                machine.download_failure(joined(&[b"temper-box identity differs: ", &candidate.line]))
            }
        }
        phase @ (Phase::Lock
        | Phase::Clean
        | Phase::RemoveCached
        | Phase::Open
        | Phase::Write
        | Phase::SayRead
        | Phase::SayEnd
        | Phase::WriteSum
        | Phase::RenameSum
        | Phase::Publish
        | Phase::Refuse) => {
            let Event::Done = event else { unreachable!("effect answer") };
            match phase {
                Phase::Lock => {
                    machine.phase = Phase::Clean;
                    Request::Clean { directory: machine.directory.clone() }
                }
                Phase::Clean => {
                    machine.phase = Phase::Cached;
                    Request::Cached { file: machine.file.clone() }
                }
                Phase::RemoveCached => match machine.failure.take() {
                    Some(why) => machine.refuse(why),
                    None => machine.fetch(true),
                },
                Phase::Open => machine.say(false),
                Phase::Write => {
                    if machine.figure(machine.total) > machine.figure(machine.said) {
                        machine.say(false)
                    } else {
                        machine.read()
                    }
                }
                Phase::SayRead => machine.read(),
                Phase::SayEnd => machine.verify(),
                Phase::WriteSum => {
                    machine.phase = Phase::RenameSum;
                    Request::RenameSum { file: machine.file.clone() }
                }
                Phase::RenameSum => {
                    machine.phase = Phase::Publish;
                    Request::Publish { file: machine.file.clone() }
                }
                Phase::Publish => machine.end(Ok(machine.file.clone())),
                Phase::Refuse => {
                    let why = machine.failure.take().expect("refusal retained");
                    machine.end(Err(why))
                }
                Phase::Begin
                | Phase::Beside
                | Phase::Cached
                | Phase::CacheIdentity
                | Phase::Head
                | Phase::Read
                | Phase::Unpack
                | Phase::PartIdentity
                | Phase::End => unreachable!("handled above"),
            }
        }
        Phase::End => unreachable!("machine ended"),
    };
    out.push(request);
}

#[cfg(test)]
mod tests;
