//! The document's machine parts and the box's kinds (temper/docs/design/protocol/configuration.md, 3; temper/docs/design/protocol/setup.md, 7.1).
use crate::{
    check::joined,
    problem::Problems,
    setup::{Account, Entry, Fake, Fetch, Mirror, Mode, Origin, Owned, Repository, Runner},
};
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::{self as config, Path, read};
use skein_forge_setup::{
    given::{email, login},
    limits::Limits,
};
use skein_json::{Document, document::Kind};
use skein_lib::{Decimal, List, bytes};

/// The configuration table's shapes and rows (temper/docs/design/protocol/documentation.md, 8).
pub use config::members::{Member, Shape};

/// An object whose requested names belong to one configuration shape.
#[derive(Debug)]
pub struct MemberObject {
    object: read::Object,
    shape: &'static str,
    rows: &'static [Member],
}
impl MemberObject {
    pub fn shaped(d: &Document, at: u32, shape: &'static str, rows: &'static [Member]) -> Result<Self, config::What> {
        Ok(Self { object: read::Object::new(d, at)?, shape, rows })
    }
    pub fn member(&mut self, d: &Document, name: &[u8]) -> Option<u32> {
        #[cfg(debug_assertions)]
        {
            let mut found = false;
            for row in self.rows {
                found |= row.shape == self.shape && row.name.as_bytes() == name;
            }
            assert!(found, "configuration member missing from table: {} {name:?}", self.shape);
        }
        self.object.member(d, name)
    }
    pub fn required(&mut self, d: &Document, name: &[u8], p: &Path, ps: &mut config::Problems) -> Option<u32> {
        let at = self.member(d, name);
        if at.is_none() {
            ps.push(p.member(name), config::What::Missing);
        }
        at
    }
    pub fn finish(self, d: &Document, p: &Path, ps: &mut config::Problems) {
        self.object.finish(d, p, ps);
    }
}

/// What a box runs a forge's group as (temper/docs/design/protocol/setup.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxKind {
    Owned,
    Fake,
}
impl BoxKind {
    #[must_use]
    pub fn word(self) -> &'static [u8] {
        match self {
            Self::Owned => b"owned",
            Self::Fake => b"fake",
        }
    }
    #[must_use]
    pub fn parse(word: &[u8]) -> Option<Self> {
        if word == b"owned" {
            Some(Self::Owned)
        } else if word == b"fake" {
            Some(Self::Fake)
        } else {
            None
        }
    }
}
/// A box's owned Forgejo's machine part: its port and what temper init chose.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnedMachine {
    pub listen: SocketAddr,
    pub remote: Box<[u8]>,
    pub person: Account,
    pub sign_in: Option<Box<[u8]>>,
    pub public: Option<Origin>,
    pub fetch: Fetch,
    pub forgejo: Box<[u8]>,
    pub runner: Option<Runner>,
}
/// The playground's fake forge's machine part.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FakeMachine {
    pub listen: SocketAddr,
    pub remote: Box<[u8]>,
    pub git: Box<[u8]>,
}
/// GitHub's place, none for GitHub itself; `--fake-backends` names its fake.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GitHubMachine {
    pub git: Option<Box<[u8]>>,
    pub api: config::Address,
    pub tls: bool,
    pub remote: Box<[u8]>,
    pub web: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Machine {
    pub owned: Option<OwnedMachine>,
    pub fake: Option<FakeMachine>,
    pub github: Option<GitHubMachine>,
}

/// Reads `application.machine`, recording paths and resolving data and cache paths as setup does.
pub fn read(
    d: &Document,
    at: u32,
    _limits: &Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Machine> {
    let mut ps = config::Problems::empty();
    let result = read_config(d, at, directories, &mut ps);
    problems.append(ps);
    result
}
/// The application retains each typed problem's full member path.
pub fn read_config(
    d: &Document,
    at: u32,
    directories: &config::Directories,
    ps: &mut config::Problems,
) -> Option<Machine> {
    let p = Path::root().member(b"application").member(b"machine");
    let mut result = machine(d, at, &p, ps);
    if ps.is_empty()
        && let Some(machine) = &mut result
    {
        if let Some(owned) = &mut machine.owned {
            resolve_path(
                directories,
                config::Directory::Cache,
                &mut owned.forgejo,
                p.member(b"owned").member(b"forgejo"),
                ps,
            );
            if let Some(runner) = &mut owned.runner {
                resolve_path(
                    directories,
                    config::Directory::Cache,
                    &mut runner.binary,
                    p.member(b"owned").member(b"runner").member(b"binary"),
                    ps,
                );
            }
        }
        if let Some(github) = &mut machine.github
            && let Some(git) = &mut github.git
        {
            resolve_path(directories, config::Directory::Data, git, p.member(b"github").member(b"git"), ps);
        }
        if let Some(fake) = &mut machine.fake {
            resolve_path(
                directories,
                config::Directory::Data,
                &mut fake.git,
                p.member(b"fake").member(b"git"),
                ps,
            );
        }
    }
    if ps.is_empty() { result } else { None }
}
fn machine(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Machine> {
    let mut o = object(d, at, "machine", p, ps)?;
    let owned_at = o.member(d, b"owned");
    let fake_at = o.member(d, b"fake");
    let github_at = o.member(d, b"github");
    o.finish(d, p, ps);
    let owned = match owned_at {
        Some(at) => Some(owned(d, at, &p.member(b"owned"), ps)?),
        None => None,
    };
    let fake = match fake_at {
        Some(at) => Some(fake(d, at, &p.member(b"fake"), ps)?),
        None => None,
    };
    let github = match github_at {
        Some(at) if d.token(at)?.kind != Kind::Null => Some(github(d, at, &p.member(b"github"), ps)?),
        Some(_) | None => None,
    };
    Some(Machine { owned, fake, github })
}
/// The entry a kind's forge runs with (protocol/setup.md, 8), if its machine part exists.
#[must_use]
pub fn entry(machine: &Machine, forge: u16, kind: BoxKind, directories: &config::Directories) -> Option<Entry> {
    let mode = match kind {
        BoxKind::Owned => {
            let o = machine.owned.as_ref()?;
            Mode::Owned(Owned {
                public: o.public.clone(),
                fetch: o.fetch.clone(),
                forgejo: o.forgejo.clone(),
                runner: o.runner.clone(),
                listen: o.listen,
                person: o.person.clone(),
            })
        }
        BoxKind::Fake => {
            let f = machine.fake.as_ref()?;
            Mode::Fake(Fake {
                listen: f.listen,
                git: f.git.clone(),
                owner: bytes::copy_of(b"playground"),
                repositories: Box::new([Repository {
                    name: bytes::copy_of(b"widget"),
                    landing: bytes::copy_of(b"main"),
                }]),
            })
        }
    };
    Some(Entry {
        forge,
        directory: directories.join(config::Directory::Data, &directory(forge, kind)).ok()?,
        bot: Account { login: bytes::copy_of(b"temper"), email: bytes::copy_of(b"temper@localhost") },
        mode,
        asked: kind == BoxKind::Owned,
    })
}
/// The forge's directory beneath the data directory.
#[must_use]
pub fn directory(forge: u16, kind: BoxKind) -> Box<[u8]> {
    let prefix = match kind {
        BoxKind::Owned => b"forge-".as_slice(),
        BoxKind::Fake => b"playground-".as_slice(),
    };
    joined(&[prefix, Decimal::of(u64::from(forge)).as_bytes()])
}
/// The group's kind word and a line feed (temper/docs/design/protocol/setup.md, 7.1).
#[must_use]
pub fn group_file(kind: BoxKind) -> Box<[u8]> {
    joined(&[kind.word(), b"\n"])
}
#[must_use]
pub fn parse_group_file(bytes: &[u8]) -> Option<BoxKind> {
    BoxKind::parse(bytes.strip_suffix(b"\n")?)
}
/// `box/groups/N`, relative to the data directory.
#[must_use]
pub fn group_path(forge: u16) -> Box<[u8]> {
    joined(&[b"box/groups/", Decimal::of(u64::from(forge)).as_bytes()])
}

fn fake(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<FakeMachine> {
    let mut o = object(d, at, "machine-fake", p, ps)?;
    let listen_at = o.required(d, b"listen", p, ps);
    let remote = text(d, &mut o, b"remote", p, ps);
    let git = text(d, &mut o, b"git", p, ps);
    o.finish(d, p, ps);
    let listen = listen(d, listen_at?, &p.member(b"listen"), ps)?;
    let remote = remote?;
    let git = git?;
    valid(git_base(&remote), p.member(b"remote"), ps);
    valid(path(&git), p.member(b"git"), ps);
    Some(FakeMachine { listen, remote, git })
}
fn github(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<GitHubMachine> {
    let mut o = object(d, at, "machine-github", p, ps)?;
    let api_at = o.required(d, b"api", p, ps);
    let transport_at = o.required(d, b"transport", p, ps);
    let remote = text(d, &mut o, b"remote", p, ps);
    let web = text(d, &mut o, b"web", p, ps);
    let git = match o.member(d, b"git") {
        Some(at) => match read::text(d, at, 4096) {
            Ok(value) => {
                valid(path(&value), p.member(b"git"), ps);
                Some(value)
            }
            Err(what) => {
                ps.push(p.member(b"git"), what);
                return None;
            }
        },
        None => None,
    };
    o.finish(d, p, ps);
    let api = config::address::read(d, api_at?, &p.member(b"api"), ps)?;
    let tls = transport(d, transport_at?, &p.member(b"transport"), ps)?;
    let socket = match &api {
        config::Address::Addr(a) => {
            let socket = a.socket();
            valid(socket.is_some(), p.member(b"api"), ps);
            socket
        }
        config::Address::Host { .. } => None,
    };
    if !tls {
        valid(
            match socket {
                Some(a) => a.ip().is_loopback(),
                None => false,
            },
            p.member(b"transport"),
            ps,
        );
    }
    let remote = remote?;
    let web = web?;
    valid(git_base(&remote), p.member(b"remote"), ps);
    valid(git_base(&web), p.member(b"web"), ps);
    Some(GitHubMachine { git, api, tls, remote, web })
}
fn transport(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<bool> {
    if d.token(at)?.kind == Kind::String {
        let value = match read::text(d, at, 16) {
            Ok(v) => v,
            Err(w) => {
                ps.push(p.clone(), w);
                return None;
            }
        };
        valid(value.as_ref() == b"plaintext", p.clone(), ps);
        return Some(false);
    }
    let mut o = object(d, at, "machine-transport", p, ps)?;
    let tls_at = o.required(d, b"tls", p, ps);
    o.finish(d, p, ps);
    let p = p.member(b"tls");
    let mut o = object(d, tls_at?, "machine-tls", &p, ps)?;
    let server_name = text(d, &mut o, b"server_name", &p, ps);
    let trust = o.required(d, b"trust", &p, ps);
    o.finish(d, &p, ps);
    let _server_name = server_name?;
    let trust = trust?;
    if d.token(trust)?.kind != Kind::Null
        && let Err(w) = read::text(d, trust, 1_048_576)
    {
        ps.push(p.member(b"trust"), w);
    }
    Some(true)
}

fn object(d: &Document, at: u32, shape: &'static str, p: &Path, ps: &mut config::Problems) -> Option<MemberObject> {
    match MemberObject::shaped(d, at, shape, &MEMBERS) {
        Ok(o) => Some(o),
        Err(w) => {
            ps.push(p.clone(), w);
            None
        }
    }
}
fn text(d: &Document, o: &mut MemberObject, name: &[u8], p: &Path, ps: &mut config::Problems) -> Option<Box<[u8]>> {
    bounded_text(d, o, name, 4096, p, ps)
}
fn bounded_text(
    d: &Document,
    o: &mut MemberObject,
    name: &[u8],
    max: u32,
    p: &Path,
    ps: &mut config::Problems,
) -> Option<Box<[u8]>> {
    let at = o.required(d, name, p, ps)?;
    match read::text(d, at, max) {
        Ok(v) => Some(v),
        Err(w) => {
            ps.push(p.member(name), w);
            None
        }
    }
}
fn valid(ok: bool, p: Path, ps: &mut config::Problems) {
    if !ok {
        ps.push(p, config::What::Form);
    }
}
fn path(b: &[u8]) -> bool {
    !b.is_empty() && !b.contains(&0)
}
fn account(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Account> {
    let o = object(d, at, "machine-account", p, ps)?;
    account_members(d, o, p, ps)
}
fn account_members(d: &Document, mut o: MemberObject, p: &Path, ps: &mut config::Problems) -> Option<Account> {
    let login_value = text(d, &mut o, b"login", p, ps);
    let email_value = text(d, &mut o, b"email", p, ps);
    o.finish(d, p, ps);
    let login_value = login_value?;
    let email_value = email_value?;
    valid(login(&login_value), p.member(b"login"), ps);
    valid(email(&email_value), p.member(b"email"), ps);
    Some(Account { login: login_value, email: email_value })
}
fn strings(
    d: &Document,
    at: u32,
    max: u32,
    names: bool,
    p: &Path,
    ps: &mut config::Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let items = match read::list(d, at, max) {
        Ok(v) => v,
        Err(w) => {
            ps.push(p.clone(), w);
            return None;
        }
    };
    valid(!items.is_empty(), p.clone(), ps);
    let mut values = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let here = p.child(config::Segment::Index(u32::try_from(i).expect("bounded list")));
        match read::text(d, *at, if names { 40 } else { 128 }) {
            Ok(v) => {
                valid(if names { login(&v) } else { !v.is_empty() && !v.contains(&0) }, here, ps);
                values.push(v).expect("list capacity");
            }
            Err(w) => ps.push(here, w),
        }
    }
    Some(values.into_boxed())
}
fn runner(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Runner> {
    let mut o = object(d, at, "machine-runner", p, ps)?;
    let binary = text(d, &mut o, b"binary", p, ps);
    let labels_at = o.required(d, b"labels", p, ps);
    let cap_at = o.required(d, b"capacity", p, ps);
    let path_value = text(d, &mut o, b"path", p, ps);
    o.finish(d, p, ps);
    let capacity = match read::integer(d, cap_at?, 64) {
        Ok(n) => n,
        Err(w) => {
            ps.push(p.member(b"capacity"), w);
            return None;
        }
    };
    valid(capacity > 0, p.member(b"capacity"), ps);
    let binary = binary?;
    let path_value = path_value?;
    valid(path(&binary), p.member(b"binary"), ps);
    valid(path(&path_value), p.member(b"path"), ps);
    Some(Runner {
        binary,
        labels: strings(d, labels_at?, 8, false, &p.member(b"labels"), ps)?,
        capacity: u32::try_from(capacity).expect("capacity bounded"),
        path: path_value,
    })
}
fn owned(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<OwnedMachine> {
    let mut o = object(d, at, "machine-owned", p, ps)?;
    let remote = text(d, &mut o, b"remote", p, ps);
    let forgejo = text(d, &mut o, b"forgejo", p, ps);
    let runner_at = o.required(d, b"runner", p, ps);
    let listen_at = o.required(d, b"listen", p, ps);
    let person_at = o.required(d, b"person", p, ps);
    let mirror_at = o.member(d, b"mirror");
    let public_at = o.member(d, b"public");
    let sign_in_at = o.member(d, b"sign_in");
    o.finish(d, p, ps);
    let runner_at = runner_at?;
    let mut runner = if d.token(runner_at)?.kind == Kind::Null {
        None
    } else {
        Some(runner(d, runner_at, &p.member(b"runner"), ps)?)
    };
    let listen = listen(d, listen_at?, &p.member(b"listen"), ps)?;
    let mut forgejo = forgejo?;
    valid(path(&forgejo), p.member(b"forgejo"), ps);
    let fetched_forgejo = forgejo.as_ref() == b"fetched";
    let mut fetched_runner = false;
    if fetched_forgejo {
        forgejo = joined(&[b"forgejo/", skein_forge::forgejo::pin::VERSION.as_bytes(), b"/forgejo"]);
    }
    if let Some(runner) = &mut runner {
        fetched_runner = runner.binary.as_ref() == b"fetched";
        if fetched_runner {
            runner.binary =
                joined(&[b"forgejo-runner/", skein_forge::forgejo::pin::RUNNER_VERSION.as_bytes(), b"/forgejo-runner"]);
        }
    }
    let mirror = match mirror_at {
        Some(at) => Some(mirror(d, at, &p.member(b"mirror"), ps)?),
        None => None,
    };
    let public = match public_at {
        Some(at) => {
            let path = p.member(b"public");
            let Ok(value) = read::text(d, at, 4096) else {
                ps.push(path, config::What::Form);
                return None;
            };
            Some(origin(&value, &path, ps)?)
        }
        None => None,
    };
    let fetch = Fetch { forgejo: fetched_forgejo, runner: fetched_runner, mirror };
    let person = account(d, person_at?, &p.member(b"person"), ps)?;
    let sign_in = match sign_in_at {
        Some(at) if d.token(at)?.kind != Kind::Null => match read::text(d, at, 256) {
            Ok(value) => Some(value),
            Err(what) => {
                ps.push(p.member(b"sign_in"), what);
                None
            }
        },
        Some(_) | None => None,
    };
    if sign_in.is_some() {
        valid(!crate::sign_in::refused_person(&person.login, b""), p.member(b"person").member(b"login"), ps);
        valid(!crate::sign_in::refused_person(b"", &person.email), p.member(b"person").member(b"email"), ps);
    }
    let remote = remote?;
    valid(git_base(&remote), p.member(b"remote"), ps);
    Some(OwnedMachine { listen, remote, person, sign_in, public, fetch, forgejo, runner })
}
fn mirror(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Mirror> {
    let mut o = object(d, at, "machine-mirror", p, ps)?;
    let forgejo = text(d, &mut o, b"forgejo", p, ps);
    let runner = text(d, &mut o, b"runner", p, ps);
    o.finish(d, p, ps);
    let forgejo = origin(&forgejo?, &p.member(b"forgejo"), ps);
    let runner = origin(&runner?, &p.member(b"runner"), ps);
    Some(Mirror { forgejo: forgejo?, runner: runner? })
}
fn origin(value: &[u8], p: &Path, ps: &mut config::Problems) -> Option<Origin> {
    // The shared location parser handles authorities; origins forbid every tail.
    let authority = value.strip_prefix(b"https://");
    let mut clean = authority.is_some();
    if let Some(rest) = authority {
        for byte in rest {
            if b"/?#".contains(byte) {
                clean = false;
            }
        }
    }
    if clean
        && let Ok(parsed) = skein_forge::github::redirect::parse(value, 4096, 1)
        && parsed.origin.port != 0
    {
        return Some(Origin { host: parsed.origin.host, port: parsed.origin.port });
    }
    ps.push(p.clone(), config::What::Form);
    None
}
fn listen(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<SocketAddr> {
    let address = config::address::read(d, at, p, ps)?;
    let listen = match address {
        config::Address::Addr(a) => a.socket(),
        config::Address::Host { .. } => None,
    };
    let listen = match listen {
        Some(a) if a.ip().is_loopback() => a,
        Some(_) | None => {
            ps.push(p.clone(), config::What::Form);
            return None;
        }
    };
    Some(listen)
}

#[expect(clippy::disallowed_methods, reason = "the configuration protocol parses URL hosts and ports")]
fn git_base(input: &[u8]) -> bool {
    let (rest, plain) = if let Some(rest) = input.strip_prefix(b"https://") {
        (rest, false)
    } else if let Some(rest) = input.strip_prefix(b"http://") {
        (rest, true)
    } else {
        return false;
    };
    if rest.is_empty() || rest.ends_with(b"/") {
        return false;
    }
    for byte in rest {
        if byte.is_ascii_control() || byte.is_ascii_whitespace() || b"@?#\\".contains(byte) {
            return false;
        }
    }
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' {
            end = index;
            break;
        }
    }
    let Ok(authority) = core::str::from_utf8(rest.get(..end).expect("host prefix")) else { return false };
    let (host, port) = if let Some(ipv6) = authority.strip_prefix('[') {
        let Some((host, suffix)) = ipv6.split_once(']') else { return false };
        if suffix.is_empty() {
            (host, None)
        } else if let Some(port) = suffix.strip_prefix(':') {
            (host, Some(port))
        } else {
            return false;
        }
    } else if let Some((host, port)) = authority.split_once(':') {
        (host, Some(port))
    } else {
        (authority, None)
    };
    if host.is_empty() {
        return false;
    }
    if let Some(port) = port
        && port.parse::<u16>().is_err()
    {
        return false;
    }
    match host.parse::<core::net::IpAddr>() {
        Ok(ip) => !plain || ip.is_loopback(),
        Err(_) => {
            for byte in host.bytes() {
                if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.' {
                    return false;
                }
            }
            !plain || host.eq_ignore_ascii_case("localhost")
        }
    }
}
fn resolve_path(
    directories: &config::Directories,
    kind: config::Directory,
    value: &mut Box<[u8]>,
    path: Path,
    problems: &mut config::Problems,
) {
    match directories.join(kind, value) {
        Ok(resolved) => *value = resolved,
        Err(what) => problems.push(path, what),
    }
}

/// The machine shapes, re-exported in the application’s configuration reference.
pub const SHAPES: [Shape; 9] = [
    Shape {
        name: "machine",
        paths: &["application.machine"],
        line: "The machine places offered when a forge is made.",
    },
    Shape {
        name: "machine-owned",
        paths: &["application.machine.owned"],
        line: "The owned Forgejo and its runner on this machine.",
    },
    Shape {
        name: "machine-fake",
        paths: &["application.machine.fake"],
        line: "The playground’s fake forge on this machine.",
    },
    Shape {
        name: "machine-github",
        paths: &["application.machine.github"],
        line: "A GitHub test peer’s addresses and transport.",
    },
    Shape {
        name: "machine-runner",
        paths: &["application.machine.owned.runner"],
        line: "The owned Forgejo’s runner program and capacity.",
    },
    Shape {
        name: "machine-account",
        paths: &["application.machine.owned.person"],
        line: "The person account made on the owned forge.",
    },
    Shape {
        name: "machine-mirror",
        paths: &["application.machine.owned.mirror"],
        line: "HTTPS origins used to fetch pinned binaries.",
    },
    Shape {
        name: "machine-transport",
        paths: &["application.machine.github.transport"],
        line: "A TLS transport object, or plaintext as text.",
    },
    Shape {
        name: "machine-tls",
        paths: &["application.machine.github.transport.tls"],
        line: "TLS identity and trust for the GitHub test peer.",
    },
];

/// The machine members read by this decoder and printed by the application.
pub const MEMBERS: [Member; 30] = [
    Member {
        shape: "machine-owned",
        name: "sign_in",
        kind: "text or null",
        bound: "a client of engine.clients.provider; at most 256 bytes",
        required: false,
        line: "The client the box’s owned Forgejo signs its person in through, null for none.",
        gone: None,
    },
    Member {
        shape: "machine",
        name: "owned",
        kind: "machine-owned",
        bound: "",
        required: false,
        line: "The box’s owned Forgejo place, absent when none is offered.",
        gone: None,
    },
    Member {
        shape: "machine",
        name: "fake",
        kind: "machine-fake",
        bound: "",
        required: false,
        line: "The playground’s fake forge place, absent when none is offered.",
        gone: None,
    },
    Member {
        shape: "machine",
        name: "github",
        kind: "machine-github or null",
        bound: "",
        required: false,
        line: "GitHub’s place, null for the public service.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "remote",
        kind: "text",
        bound: "at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash",
        required: true,
        line: "The base of the owned forge’s repository remotes.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "forgejo",
        kind: "text",
        bound: "cache path or fetched; at most 4,096 bytes",
        required: true,
        line: "The Forgejo binary, fetched at the pinned version when requested.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "runner",
        kind: "machine-runner or null",
        bound: "",
        required: true,
        line: "The runner to prepare, or null for none.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "listen",
        kind: "address",
        bound: "loopback socket, unique among listeners",
        required: true,
        line: "The owned Forgejo listener.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "person",
        kind: "machine-account",
        bound: "",
        required: true,
        line: "The person account made at setup.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "mirror",
        kind: "machine-mirror",
        bound: "",
        required: false,
        line: "Alternate HTTPS origins for fetching the pinned binaries.",
        gone: None,
    },
    Member {
        shape: "machine-owned",
        name: "public",
        kind: "text",
        bound: "HTTPS origin; at most 4,096 bytes",
        required: false,
        line: "The browser origin of the owned forge behind its proxy.",
        gone: None,
    },
    Member {
        shape: "machine-fake",
        name: "listen",
        kind: "address",
        bound: "loopback socket, unique among listeners",
        required: true,
        line: "The fake forge listener.",
        gone: None,
    },
    Member {
        shape: "machine-fake",
        name: "remote",
        kind: "text",
        bound: "at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash",
        required: true,
        line: "The base of the fake forge’s repository remotes.",
        gone: None,
    },
    Member {
        shape: "machine-fake",
        name: "git",
        kind: "text",
        bound: "data path; at most 4,096 bytes",
        required: true,
        line: "The fake git program for the playground’s repositories.",
        gone: None,
    },
    Member {
        shape: "machine-github",
        name: "api",
        kind: "address",
        bound: "socket or host and port",
        required: true,
        line: "The GitHub API address.",
        gone: None,
    },
    Member {
        shape: "machine-github",
        name: "transport",
        kind: "machine-transport or text",
        bound: "plaintext on loopback, or TLS",
        required: true,
        line: "The transport to the GitHub API.",
        gone: None,
    },
    Member {
        shape: "machine-github",
        name: "remote",
        kind: "text",
        bound: "at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash",
        required: true,
        line: "The base of the GitHub repository remotes.",
        gone: None,
    },
    Member {
        shape: "machine-github",
        name: "web",
        kind: "text",
        bound: "at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash",
        required: true,
        line: "The base of GitHub’s browser pages.",
        gone: None,
    },
    Member {
        shape: "machine-github",
        name: "git",
        kind: "text",
        bound: "data path; at most 4,096 bytes",
        required: false,
        line: "The git program for repositories on the test peer.",
        gone: None,
    },
    Member {
        shape: "machine-runner",
        name: "binary",
        kind: "text",
        bound: "cache path or fetched; at most 4,096 bytes",
        required: true,
        line: "The runner binary, fetched at the pinned version when requested.",
        gone: None,
    },
    Member {
        shape: "machine-runner",
        name: "labels",
        kind: "list of text",
        bound: "1 to 8 labels, each at most 128 bytes",
        required: true,
        line: "The labels offered to the forge’s jobs.",
        gone: None,
    },
    Member {
        shape: "machine-runner",
        name: "capacity",
        kind: "integer",
        bound: "1 to 64",
        required: true,
        line: "The number of jobs the runner may execute at once.",
        gone: None,
    },
    Member {
        shape: "machine-runner",
        name: "path",
        kind: "text",
        bound: "at most 4,096 bytes; nonempty without NUL",
        required: true,
        line: "The PATH the runner gives its jobs.",
        gone: None,
    },
    Member {
        shape: "machine-account",
        name: "login",
        kind: "text",
        bound: "valid forge login; at most 4,096 bytes",
        required: true,
        line: "The person’s forge login.",
        gone: None,
    },
    Member {
        shape: "machine-account",
        name: "email",
        kind: "text",
        bound: "valid forge email; at most 4,096 bytes",
        required: true,
        line: "The person’s forge email address.",
        gone: None,
    },
    Member {
        shape: "machine-mirror",
        name: "forgejo",
        kind: "text",
        bound: "HTTPS origin; at most 4,096 bytes",
        required: true,
        line: "The origin serving the pinned Forgejo binary.",
        gone: None,
    },
    Member {
        shape: "machine-mirror",
        name: "runner",
        kind: "text",
        bound: "HTTPS origin; at most 4,096 bytes",
        required: true,
        line: "The origin serving the pinned runner binary.",
        gone: None,
    },
    Member {
        shape: "machine-transport",
        name: "tls",
        kind: "machine-tls",
        bound: "",
        required: true,
        line: "The API’s TLS server name and trust roots.",
        gone: None,
    },
    Member {
        shape: "machine-tls",
        name: "server_name",
        kind: "text",
        bound: "at most 4,096 bytes",
        required: true,
        line: "The server name checked during the TLS handshake.",
        gone: None,
    },
    Member {
        shape: "machine-tls",
        name: "trust",
        kind: "text or null",
        bound: "at most 1,048,576 bytes",
        required: true,
        line: "The trust roots, or null for the machine’s roots.",
        gone: None,
    },
];
