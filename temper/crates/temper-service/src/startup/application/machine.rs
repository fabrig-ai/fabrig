//! Machine forge templates (temper/docs/design/draft/playground-reset.md, 3).
use super::{Problems, What, read, sign_in::transport};
use crate::settings::Transport;
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind, writer};
use temper_engine_domain_forge::{Api, ApiTransport, Bot, ForgeRecord, ForgeState, Kind as ForgeKind, Machine, Mode};
use temper_forge_setup::setup::{Entry, Mode as SetupMode};

pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    limits: &skein_forge_setup::limits::Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Machine> {
    let at = at?;
    let path = Path::root().member(b"application").member(b"machine");
    let mut object = read::object(document, at, &path, problems)?;
    let owned_at = read::required(&mut object, document, b"owned", &path, problems);
    let github_at = read::required(&mut object, document, b"github", &path, problems);
    let program_at = read::required(&mut object, document, b"git_program", &path, problems);
    read::finish(object, document, &path, problems);
    let mut machine = Machine::bare();
    if let Some(at) = owned_at
        && document.token(at)?.kind != Kind::Null
    {
        let path = path.member(b"owned");
        let (entry, text) = body(document, at, &path, limits, directories, problems)?;
        match &entry.mode {
            SetupMode::Owned(_) => {}
            SetupMode::Managed { .. } | SetupMode::Fake(_) => {
                problems.push(path.member(b"mode"), What::Jig(config::What::Form));
                return None;
            }
        }
        machine.owned = Some(owned(&entry, text));
    }
    machine.github = github(document, github_at?, &path.member(b"github"), directories, problems)?;
    if let Some(at) = program_at
        && document.token(at)?.kind != Kind::Null
    {
        let path = path.member(b"git_program");
        let program = match config::read::text(document, at, 4096) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path, What::Jig(what));
                return None;
            }
        };
        if program.is_empty() || program.contains(&0) {
            problems.push(path, What::Jig(config::What::Form));
            return None;
        }
        let program = super::joined(directories, config::Directory::Data, &program, &path, problems)?;
        if !program.starts_with(b"/") || program.len() > 4096 {
            problems.push(path, What::Jig(config::What::Bound));
            return None;
        }
        machine.github.git_program = Some(program);
    }
    Some(machine)
}

pub(super) fn body(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &skein_forge_setup::limits::Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<(Entry, Box<[u8]>)> {
    let Some(text) = value_text(document, at) else {
        problems.push(path.clone(), What::Jig(config::What::Bound));
        return None;
    };
    let mut found = temper_forge_setup::problem::Problems::empty();
    let entry = Entry::body(document, at, path, 0, limits, directories, &mut found);
    if !found.is_empty() {
        problems.push(path.clone(), What::Jig(config::What::Form));
        return None;
    }
    Some((entry?, text))
}

pub(super) fn value_text(document: &Document, at: u32) -> Option<Box<[u8]>> {
    let end = config::read::after(document, at).ok()?;
    let limits = writer::Limits { length: temper_forge_setup::setup::BODY_BYTES, depth: 32 };
    let mut measure = writer::Encoder::measure(&limits);
    emit(&mut measure, document, at, end)?;
    let length = measure.measured().ok()?;
    let mut output = writer::Encoder::write(length, &limits);
    emit(&mut output, document, at, end)?;
    Some(output.finish())
}
fn emit(output: &mut writer::Encoder, document: &Document, at: u32, end: u32) -> Option<()> {
    for index in at..end {
        let token = document.token(index)?;
        let text = document.text(token)?;
        match token.kind {
            Kind::ObjectStart => output.object_start(),
            Kind::ObjectEnd => output.object_end(),
            Kind::ArrayStart => output.array_start(),
            Kind::ArrayEnd => output.array_end(),
            Kind::Key => output.key(text),
            Kind::String => output.string(text),
            Kind::Number => output.number(text),
            Kind::True => output.boolean(true),
            Kind::False => output.boolean(false),
            Kind::Null => output.null(),
            Kind::Long => return None,
        }
    }
    Some(())
}

#[expect(clippy::disallowed_macros, reason = "socket and IP Display supply canonical address text")]
fn local(entry: &Entry, text: Box<[u8]>, playground: bool) -> ForgeRecord {
    let (listen, program, mode) = match &entry.mode {
        SetupMode::Owned(owned) => (owned.listen, None, Mode::Owned),
        SetupMode::Fake(fake) => (fake.listen, Some(fake.git.clone()), Mode::Fake),
        SetupMode::Managed { .. } => unreachable!("a local template is owned or fake"),
    };
    ForgeRecord {
        number: 0,
        kind: ForgeKind::Forgejo,
        mode,
        api: Api {
            host: alloc::format!("{}", listen.ip()).into_bytes().into_boxed_slice(),
            port: listen.port(),
            transport: ApiTransport::Plaintext,
        },
        git: alloc::format!("http://{listen}").into_bytes().into_boxed_slice(),
        git_program: program,
        web: None,
        read_only: false,
        bot: Bot { login: entry.bot.login.clone(), name: b"Temper".as_slice().into(), email: entry.bot.email.clone() },
        setup: Some(text),
        first: None,
        playground,
        state: ForgeState::Kept,
    }
}
pub(super) fn owned(entry: &Entry, text: Box<[u8]>) -> ForgeRecord {
    local(entry, text, false)
}
pub(super) fn playground(entry: &Entry, text: Box<[u8]>) -> ForgeRecord {
    local(entry, text, true)
}

#[expect(clippy::disallowed_macros, reason = "IP Display supplies the address's host text")]
fn github(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<ForgeRecord> {
    let mut object = read::object(document, at, path, problems)?;
    let mut record = ForgeRecord::github_com();
    if let Some(at) = object.member(document, b"app_read_only") {
        record.read_only = match config::read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"app_read_only"), What::Jig(what));
                return None;
            }
        };
    }
    let at = read::required(&mut object, document, b"api", path, problems)?;
    let api = if document.token(at)?.kind == Kind::Null {
        config::Address::Host { name: record.api.host.clone(), port: record.api.port }
    } else {
        let mut found = config::Problems::empty();
        let api = config::address::read(document, at, &path.member(b"api"), &mut found);
        problems.append(found, &Path::root());
        api?
    };
    let at = read::required(&mut object, document, b"transport", path, problems)?;
    let transport = if document.token(at)?.kind == Kind::Null {
        Transport::Tls { server_name: b"api.github.com".as_slice().into(), trust: None }
    } else {
        transport(document, at, &path.member(b"transport"), problems)?
    };
    forge_endpoint(&api, &transport, path, problems);
    let (host, port) = match api {
        config::Address::Host { name, port } => (name, port),
        config::Address::Addr(skein_io::Addr::Ip(socket)) => {
            (alloc::format!("{}", socket.ip()).into_bytes().into_boxed_slice(), socket.port())
        }
        config::Address::Addr(skein_io::Addr::InProcess(_)) => return None,
    };
    let transport = match transport {
        Transport::Plaintext => ApiTransport::Plaintext,
        Transport::Tls { server_name, trust } => {
            let trust = match trust {
                Some(trust) => Some(super::joined(
                    directories,
                    config::Directory::Config,
                    &trust,
                    &path.member(b"transport").member(b"tls").member(b"trust"),
                    problems,
                )?),
                None => None,
            };
            ApiTransport::Tls { server_name, trust }
        }
    };
    record.api = Api { host, port, transport };
    if let Some(git) = super::nullable_text(&mut object, document, b"git", path, problems, false) {
        record.git = git;
    }
    if let Some(web) = super::nullable_text(&mut object, document, b"web", path, problems, false) {
        record.web = Some(web);
    }
    read::form(super::git_base(&record.git), path.member(b"git"), problems);
    if let Some(web) = &record.web {
        read::form(super::git_base(web), path.member(b"web"), problems);
    }
    read::finish(object, document, path, problems);
    Some(record)
}

fn forge_endpoint(api: &config::Address, transport: &Transport, path: &Path, problems: &mut Problems) {
    match api {
        config::Address::Addr(skein_io::Addr::InProcess(_)) => read::form(false, path.member(b"api"), problems),
        config::Address::Addr(skein_io::Addr::Ip(_)) | config::Address::Host { .. } => {}
    }
    match transport {
        Transport::Plaintext => {
            let loopback = match api {
                config::Address::Addr(skein_io::Addr::Ip(socket)) => socket.ip().is_loopback(),
                config::Address::Addr(skein_io::Addr::InProcess(_)) | config::Address::Host { .. } => false,
            };
            if !loopback {
                problems.push(path.member(b"transport"), What::Loopback);
            }
        }
        Transport::Tls { .. } => {}
    }
}
