//! The box's group file (temper/docs/plan/playground-reset-r5/notes/forge-records.md, 4;
//! temper/docs/design/protocol/setup.md, 7.1–7.3 and 8).
use crate::{problem::Problems, setup};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_forge_setup::limits::Limits;
use skein_json::{Document, writer::Encoder};

/// The largest encoded group file.
pub const GROUP_BYTES: u32 = 8192;

/// The processes the box runs for a forge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupKind {
    Owned,
    Fake { llm: bool },
}

/// One forge's complete setup entry and its box group kind.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupFile {
    pub forge: u16,
    pub kind: GroupKind,
    pub entry: setup::Entry,
}

fn account(e: &mut Encoder, value: &setup::Account) {
    e.object_start();
    e.key(b"login");
    e.string(&value.login);
    e.key(b"email");
    e.string(&value.email);
    e.object_end();
}

fn origin(e: &mut Encoder, value: &setup::Origin) {
    let port = skein_lib::Decimal::of(u64::from(value.port));
    e.string(&crate::check::joined(&[b"https://", &value.host, b":", port.as_bytes()]));
}

fn owned(e: &mut Encoder, value: &setup::Owned) {
    e.object_start();
    e.key(b"forgejo");
    // The sentinel retains the fetch step as well as the resolved pinned cache path.
    e.string(if value.fetch.forgejo { b"fetched" } else { &value.forgejo });
    e.key(b"runner");
    match &value.runner {
        None => e.null(),
        Some(runner) => {
            e.object_start();
            e.key(b"binary");
            e.string(if value.fetch.runner { b"fetched" } else { &runner.binary });
            e.key(b"labels");
            e.array_start();
            for label in &runner.labels {
                e.string(label);
            }
            e.array_end();
            e.key(b"capacity");
            e.unsigned(u64::from(runner.capacity));
            e.key(b"path");
            e.string(&runner.path);
            e.object_end();
        }
    }
    e.key(b"listen");
    listener(e, value.listen);
    e.key(b"person");
    account(e, &value.person);
    if let Some(public) = &value.public {
        e.key(b"public");
        origin(e, public);
    }
    if let Some(mirror) = &value.fetch.mirror {
        e.key(b"mirror");
        e.object_start();
        e.key(b"forgejo");
        origin(e, &mirror.forgejo);
        e.key(b"runner");
        origin(e, &mirror.runner);
        e.object_end();
    }
    e.object_end();
}

fn listener(e: &mut Encoder, address: core::net::SocketAddr) {
    e.object_start();
    e.key(b"socket");
    e.string(&crate::ini::socket(address));
    e.object_end();
}

fn fake(e: &mut Encoder, value: &setup::Fake) {
    e.object_start();
    e.key(b"listen");
    listener(e, value.listen);
    e.key(b"git");
    e.string(&value.git);
    e.key(b"owner");
    e.string(&value.owner);
    e.key(b"repositories");
    e.array_start();
    for repository in &value.repositories {
        e.object_start();
        e.key(b"name");
        e.string(&repository.name);
        e.key(b"landing");
        e.string(&repository.landing);
        e.object_end();
    }
    e.array_end();
    e.object_end();
}

fn emit(e: &mut Encoder, file: &GroupFile) {
    e.object_start();
    e.key(b"forge");
    e.unsigned(u64::from(file.forge));
    e.key(b"kind");
    match file.kind {
        GroupKind::Owned => e.string(b"owned"),
        GroupKind::Fake { llm } => {
            e.string(b"fake");
            e.key(b"llm");
            e.boolean(llm);
        }
    }
    let entry = &file.entry;
    e.key(b"setup");
    e.object_start();
    e.key(b"forge");
    e.unsigned(u64::from(entry.forge));
    e.key(b"directory");
    e.string(&entry.directory);
    e.key(b"bot");
    account(e, &entry.bot);
    if entry.asked {
        e.key(b"start");
        e.string(b"asked");
    }
    e.key(b"mode");
    e.object_start();
    match &entry.mode {
        setup::Mode::Owned(value) => {
            e.key(b"owned");
            owned(e, value);
        }
        setup::Mode::Fake(value) => {
            e.key(b"fake");
            fake(e, value);
        }
        setup::Mode::Managed { .. } => unreachable!("a group runs an owned or fake forge"),
    }
    e.object_end();
    e.object_end();
    e.object_end();
}

/// Encodes a checked group, retaining fetched binaries' pinned-cache spelling.
#[must_use]
pub fn encode(file: &GroupFile) -> Box<[u8]> {
    let limits = skein_json::writer::Limits { depth: 8, length: GROUP_BYTES };
    let mut measure = Encoder::measure(&limits);
    emit(&mut measure, file);
    let mut write = Encoder::write(measure.measured().expect("bounded group file"), &limits);
    emit(&mut write, file);
    write.finish()
}

fn parse(
    d: &Document,
    limits: &Limits,
    directories: &config::Directories,
    ps: &mut config::Problems,
) -> Option<GroupFile> {
    let p = Path::root();
    let mut o = match read::Object::new(d, 0) {
        Ok(o) => o,
        Err(what) => {
            ps.push(p, what);
            return None;
        }
    };
    let forge_at = o.required(d, b"forge", &p, ps);
    let kind_at = o.required(d, b"kind", &p, ps);
    let setup_at = o.required(d, b"setup", &p, ps);
    let llm_at = o.member(d, b"llm");
    o.finish(d, &p, ps);
    let forge = match read::integer(d, forge_at?, u64::from(u16::MAX)) {
        Ok(n) => u16::try_from(n).expect("bounded forge"),
        Err(what) => {
            ps.push(p.member(b"forge"), what);
            return None;
        }
    };
    let kind = match read::text(d, kind_at?, 5) {
        Ok(kind) => kind,
        Err(what) => {
            ps.push(p.member(b"kind"), what);
            return None;
        }
    };
    let kind = match kind.as_ref() {
        b"owned" if llm_at.is_none() => GroupKind::Owned,
        b"fake" => {
            let llm = match llm_at {
                None => false,
                Some(at) => match read::boolean(d, at) {
                    Ok(llm) => llm,
                    Err(what) => {
                        ps.push(p.member(b"llm"), what);
                        return None;
                    }
                },
            };
            GroupKind::Fake { llm }
        }
        _ => {
            ps.push(p.member(b"kind"), config::What::Form);
            return None;
        }
    };
    let mut entry = setup::entry(d, setup_at?, limits, &p.member(b"setup"), ps)?;
    if entry.forge != forge {
        ps.push(p.member(b"forge"), config::What::Form);
    }
    match (kind, &entry.mode) {
        (GroupKind::Owned, setup::Mode::Owned(_)) | (GroupKind::Fake { .. }, setup::Mode::Fake(_)) => {}
        (GroupKind::Owned, setup::Mode::Fake(_))
        | (GroupKind::Fake { .. }, setup::Mode::Owned(_))
        | (GroupKind::Owned | GroupKind::Fake { .. }, setup::Mode::Managed { .. }) => {
            ps.push(p.member(b"setup").member(b"mode"), config::What::Form);
        }
    }
    if ps.is_empty() {
        setup::resolve_entry(&mut entry, directories, &p.member(b"setup"), ps);
    }
    Some(GroupFile { forge, kind, entry })
}

/// Reads only a bounded, whole group file using setup's entry and directory rules.
pub fn decode(bytes: &[u8], limits: &Limits, directories: &config::Directories) -> Result<GroupFile, Problems> {
    let mut ps = config::Problems::empty();
    let d = config::document(bytes, &config::DocumentLimits { bytes: GROUP_BYTES, tokens: GROUP_BYTES, depth: 8 });
    let file = match d {
        Ok(d) => parse(&d, limits, directories, &mut ps),
        Err(problem) => {
            ps.push(problem.path, problem.what);
            None
        }
    };
    if file.is_none() && ps.is_empty() {
        ps.push(Path::root(), config::What::Form);
    }
    let mut problems = Problems::empty();
    problems.append(ps);
    match file {
        Some(file) if problems.is_empty() => Ok(file),
        Some(_) | None => Err(problems),
    }
}
