//! A forge's immutable setup file (temper/docs/design/protocol/setup/place.md, 3).
use crate::{
    check::joined,
    machine::{self, BoxKind, FakeMachine, OwnedMachine},
    setup::Entry,
};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::writer::{Encoder, Limits};
use skein_lib::Decimal;

/// Processes' place and programs, chosen once for a forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SetupFile {
    Owned(OwnedMachine),
    Fake(FakeMachine),
}
#[derive(Debug)]
pub enum Refused {
    Json,
    Version(u64),
    Members(crate::problem::Problems),
}
impl SetupFile {
    #[must_use]
    pub fn kind(&self) -> BoxKind {
        match self {
            Self::Owned(_) => BoxKind::Owned,
            Self::Fake(_) => BoxKind::Fake,
        }
    }
    #[must_use]
    pub fn listen(&self) -> core::net::SocketAddr {
        match self {
            Self::Owned(o) => o.listen,
            Self::Fake(f) => f.listen,
        }
    }
    #[must_use]
    pub fn started_notice(&self) -> Box<[u8]> {
        match self {
            Self::Owned(o) => joined(&[
                b"Forgejo: api http://",
                &crate::ini::socket(o.listen),
                b"/api/v1\nForgejo: person ",
                &o.person.login,
                b"\n",
            ]),
            Self::Fake(_) => Box::new([]),
        }
    }
    #[must_use]
    pub fn entry(&self, forge: u16, directories: &config::Directories) -> Option<Entry> {
        use crate::setup::{Account, Fake, Mode, Owned, Repository};
        let mode = match self {
            Self::Owned(o) => Mode::Owned(Owned {
                public: o.public.clone(),
                fetch: o.fetch.clone(),
                forgejo: o.forgejo.clone(),
                runner: o.runner.clone(),
                listen: o.listen,
                person: o.person.clone(),
            }),
            Self::Fake(f) => Mode::Fake(Fake {
                listen: f.listen,
                git: f.git.clone(),
                owner: Box::from(b"playground".as_slice()),
                repositories: Box::new([Repository {
                    name: Box::from(b"widget".as_slice()),
                    landing: Box::from(b"main".as_slice()),
                }]),
            }),
        };
        Some(Entry {
            forge,
            directory: directories.join(config::Directory::Data, &machine::directory(forge, self.kind())).ok()?,
            bot: Account { login: Box::from(b"temper".as_slice()), email: Box::from(b"temper@localhost".as_slice()) },
            mode,
            asked: self.kind() == BoxKind::Owned,
        })
    }
}
/// Relative to the data directory.
#[must_use]
pub fn path(forge: u16, kind: BoxKind) -> Box<[u8]> {
    joined(&[&machine::directory(forge, kind), b"/setup"])
}
/// The stock place chosen by routing for the box (protocol/setup/place.md, 2).
pub fn chosen(
    kind: BoxKind,
    port: u16,
    login: &[u8],
    provider: bool,
    directories: &config::Directories,
) -> Result<SetupFile, Refused> {
    use crate::setup::{Account, Fetch, Runner};
    let listen = core::net::SocketAddr::from(([127, 0, 0, 1], port));
    let remote = joined(&[b"http://", &crate::ini::socket(listen)]);
    let file = match kind {
        BoxKind::Owned => SetupFile::Owned(OwnedMachine {
            listen,
            remote,
            person: Account { login: Box::from(login), email: joined(&[login, b"@localhost"]) },
            public: None,
            sign_in: if provider { Some(Box::from(b"forgejo".as_slice())) } else { None },
            forgejo: Box::from(b"fetched".as_slice()),
            fetch: Fetch { forgejo: true, runner: true, mirror: None },
            runner: Some(Runner::stock(
                Box::from(b"fetched".as_slice()),
                Box::new([Box::from(b"temper-host:host".as_slice())]),
            )),
        }),
        BoxKind::Fake => SetupFile::Fake(FakeMachine { listen, remote, git: Box::from(b"demo/bin/git".as_slice()) }),
    };
    decode(&encode(&file), directories)
}

/// Reads version 1, checking the same member forms as the setup entry.
#[expect(clippy::manual_map, reason = "step code spells optional values with match")]
pub fn decode(bytes: &[u8], directories: &config::Directories) -> Result<SetupFile, Refused> {
    let d = config::document(bytes, &config::DocumentLimits { bytes: 65536, tokens: 512, depth: 8 })
        .or(Err(Refused::Json))?;
    let mut ps = config::Problems::empty();
    let p = Path::root();
    let mut o = read::Object::new(&d, 0).or(Err(Refused::Json))?;
    let at = o.required(&d, b"version", &p, &mut ps).ok_or(Refused::Json)?;
    let version = read::integer(&d, at, u64::MAX).or(Err(Refused::Json))?;
    if version != 1 {
        return Err(Refused::Version(version));
    }
    let at = o.required(&d, b"kind", &p, &mut ps).ok_or(Refused::Json)?;
    let kind = read::text(&d, at, 16).or(Err(Refused::Json))?;
    let result = match BoxKind::parse(&kind) {
        Some(BoxKind::Owned) => match machine::owned(&d, 0, &p, &mut ps) {
            Some(o) => Some(SetupFile::Owned(o)),
            None => None,
        },
        Some(BoxKind::Fake) => match machine::fake(&d, 0, &p, &mut ps) {
            Some(f) => Some(SetupFile::Fake(f)),
            None => None,
        },
        None => {
            ps.push(p.member(b"kind"), config::What::Form);
            None
        }
    };
    let Some(mut result) = result else {
        return Err(members(ps));
    };
    match &mut result {
        SetupFile::Owned(o) => {
            machine::resolve_path(directories, config::Directory::Cache, &mut o.forgejo, p.member(b"forgejo"), &mut ps);
            if let Some(r) = &mut o.runner {
                machine::resolve_path(
                    directories,
                    config::Directory::Cache,
                    &mut r.binary,
                    p.member(b"runner").member(b"binary"),
                    &mut ps,
                );
            }
        }
        SetupFile::Fake(f) => {
            machine::resolve_path(directories, config::Directory::Data, &mut f.git, p.member(b"git"), &mut ps);
        }
    }
    if ps.is_empty() { Ok(result) } else { Err(members(ps)) }
}
fn emit(e: &mut Encoder, file: &SetupFile) {
    e.object_start();
    e.key(b"version");
    e.number(b"1");
    e.key(b"kind");
    e.string(file.kind().word());
    e.key(b"listen");
    e.object_start();
    e.key(b"socket");
    e.string(&crate::ini::socket(file.listen()));
    e.object_end();
    e.key(b"remote");
    match file {
        SetupFile::Fake(f) => {
            e.string(&f.remote);
            e.key(b"git");
            e.string(&f.git);
        }
        SetupFile::Owned(o) => {
            e.string(&o.remote);
            e.key(b"person");
            e.object_start();
            e.key(b"login");
            e.string(&o.person.login);
            e.key(b"email");
            e.string(&o.person.email);
            e.object_end();
            e.key(b"forgejo");
            e.string(if o.fetch.forgejo { b"fetched" } else { &o.forgejo });
            e.key(b"runner");
            match &o.runner {
                None => e.null(),
                Some(r) => {
                    e.object_start();
                    e.key(b"binary");
                    e.string(if o.fetch.runner { b"fetched" } else { &r.binary });
                    e.key(b"labels");
                    e.array_start();
                    for label in &r.labels {
                        e.string(label);
                    }
                    e.array_end();
                    e.key(b"capacity");
                    e.number(Decimal::of(u64::from(r.capacity)).as_bytes());
                    e.key(b"path");
                    e.string(&r.path);
                    e.object_end();
                }
            }
            if let Some(public) = &o.public {
                e.key(b"public");
                e.string(&origin(public));
            }
            if let Some(mirror) = &o.fetch.mirror {
                e.key(b"mirror");
                e.object_start();
                e.key(b"forgejo");
                e.string(&origin(&mirror.forgejo));
                e.key(b"runner");
                e.string(&origin(&mirror.runner));
                e.object_end();
            }
            e.key(b"sign_in");
            match &o.sign_in {
                Some(id) => e.string(id),
                None => e.null(),
            }
        }
    }
    e.object_end();
}
#[must_use]
pub fn encode(file: &SetupFile) -> Box<[u8]> {
    let limits = Limits { depth: 4, length: 65536 };
    let mut measure = Encoder::measure(&limits);
    emit(&mut measure, file);
    let mut write = Encoder::write(measure.measured().expect("bounded setup file"), &limits);
    emit(&mut write, file);
    write.finish()
}

/// An origin's canonical text, without a trailing slash.
#[must_use]
pub fn origin(value: &crate::setup::Origin) -> Box<[u8]> {
    let port = Decimal::of(u64::from(value.port));
    joined(&[
        b"https://",
        &value.host,
        if value.port == 443 { b"" } else { b":" },
        if value.port == 443 { b"" } else { port.as_bytes() },
    ])
}

fn members(ps: config::Problems) -> Refused {
    let mut problems = crate::problem::Problems::empty();
    problems.append(ps);
    Refused::Members(problems)
}
