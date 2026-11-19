//! Machine and client settings, read strictly (temper/docs/design/protocol/configuration/engine.md).
//! A box's sign-in is exclusive (temper/docs/design/protocol/openid-provider.md, 9).
pub mod push;
mod read;
pub mod sign_in;
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::{List, bytes};
pub use temper_engine_service_vocabulary::startup::application::*;

#[derive(Debug)]
pub struct Members {
    pub clients: Clients,
    pub sign_in: Option<sign_in::SignIn>,
    pub push: Option<push::Push>,
    pub webhooks: Option<Webhooks>,
    pub test_github: Option<temper_forge_setup::machine::GitHubMachine>,
    pub demo: Option<settings::Demo>,
    pub limits: limits::Limits,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
    pub workers: Box<[Worker]>,
    /// Position kept for the worker's reader; an engine alone refuses the member.
    pub environment: Option<u32>,
}
/// The optional webhook listener (protocol/configuration/engine.md).
#[derive(Debug)]
pub struct Webhooks {
    pub listener: config::Address,
}
/// Read a socket listener, and the clients' bundle and public origin when asked.
#[must_use]
pub fn clients(document: &Document, at: u32, path: &Path, bundle: bool, problems: &mut Problems) -> Option<Clients> {
    let mut object =
        read::object(document, at, if bundle { "application-clients" } else { "webhooks" }, path, problems)?;
    let at = read::required(&mut object, document, b"listener", path, problems);
    let mut directory = None;
    if bundle
        && let Some(at) = read::required(&mut object, document, b"bundle", path, problems)
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 1_048_576) {
            Ok(value) => {
                if !value.starts_with(b"/") {
                    problems.push(path.member(b"bundle"), What::Absolute);
                }
                directory = Some(value);
            }
            Err(what) => problems.push(path.member(b"bundle"), What::Jig(what)),
        }
    }
    let mut public = None;
    if bundle
        && let Some(at) = read::nullable(&mut object, document, b"public", path, problems)
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 267) {
            Ok(value) => match Origin::parse(&value) {
                Some(origin) => public = Some(origin),
                None => problems.push(path.member(b"public"), What::Jig(config::What::Form)),
            },
            Err(what) => problems.push(path.member(b"public"), What::Jig(what)),
        }
    }
    read::finish(object, document, path, problems);
    let path = path.member(b"listener");
    let mut found = config::Problems::empty();
    let listener = config::address::read(document, at?, &path, &mut found);
    problems.append(found, &Path::root());
    let listener = listener?;
    let socket = match listener {
        config::Address::Addr(skein_io::Addr::Ip(_)) => true,
        config::Address::Addr(skein_io::Addr::InProcess(_)) | config::Address::Host { .. } => false,
    };
    read::form(socket, path, problems);
    Some(Clients { listener, bundle: directory, public })
}
#[derive(Debug)]
pub struct Forge {
    pub records_only: bool,
    pub number: u16,
    pub kind: settings::Kind,
    pub api: config::Address,
    pub transport: Transport,
    pub git: Box<[u8]>,
    pub git_program: Option<Box<[u8]>>,
    pub web: Option<Box<[u8]>>,
    pub app_read_only: bool,
    pub user: User,
    pub identity: u32,
}
/// How a forge's account id is supplied.
#[derive(Debug, PartialEq, Eq)]
pub enum User {
    /// An explicit positive account id.
    Id(u64),
    /// A private file holding the decimal account id.
    File(Box<[u8]>),
}
/// Twenty decimal digits and one final line feed.
pub const USER_BYTES: u32 = 21;
#[derive(Debug)]
pub enum Transport {
    Plaintext,
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}
#[derive(Debug)]
pub struct Identity {
    pub number: u32,
    pub forge: Option<u16>,
    pub token: Box<[u8]>,
    pub token_configured: bool,
    pub login: Box<[u8]>,
    pub author: settings::Author,
}
#[derive(Debug)]
pub struct Worker {
    pub root: Box<[u8]>,
}
// Runtime starts and offline store readers judge the same fixed tier carriers.
fn tier_bounds(limits: &limits::Limits, problems: &mut Problems) {
    let bound = temper_protocol_store::forge_changes_bound().expect("schema tier rows hold at least one landing");
    if limits.forge.changes > bound {
        problems.push(
            super::checks::own(b"forge.changes"),
            What::Half(crate::startup::EngineWhat::TierChanges {
                count: limits.forge.changes,
                bound,
                ceiling: temper_protocol_store::tier_landings_ceiling(),
            }),
        );
    }
    let bound = temper_engine_domain_forge::FindingSignature::IDS;
    if limits.forge.tiers.failing_tests > bound {
        problems.push(
            super::checks::own(b"forge.tiers.failing_tests"),
            What::Half(crate::startup::EngineWhat::TierFailingTests { count: limits.forge.tiers.failing_tests, bound }),
        );
    }
    let ceiling = temper_protocol_store::v25::CEILINGS.landing_log_landings;
    let bound = ceiling.saturating_sub(limits.forge.landings_kept);
    if limits.forge.landings_pending > bound {
        problems.push(
            super::checks::own(b"forge.landings_pending"),
            What::Half(crate::startup::EngineWhat::LandingPending {
                count: limits.forge.landings_pending,
                bound,
                kept: limits.forge.landings_kept,
                ceiling,
            }),
        );
    }
    let ceiling = temper_protocol_store::v25::CEILINGS.tier_posting_comments;
    let bound = ceiling
        .saturating_sub(limits.forge.changes)
        .checked_div(limits.forge.landing_members)
        .unwrap_or(0)
        .saturating_sub(limits.forge.landings_kept);
    if limits.forge.landings_pending > bound {
        problems.push(
            super::checks::own(b"forge.landings_pending"),
            What::Half(crate::startup::EngineWhat::LandingComments {
                count: limits.forge.landings_pending,
                bound,
                ceiling,
            }),
        );
    }
    let bound = temper_engine_domain_forge::FindingSignature::ID_BYTES;
    if limits.forge.tiers.test_id_bytes > bound {
        problems.push(
            super::checks::own(b"forge.tiers.test_id_bytes"),
            What::Half(crate::startup::EngineWhat::TestIdBytes { bytes: limits.forge.tiers.test_id_bytes, bound }),
        );
    }
}
fn workers(document: &Document, directories: &config::Directories, problems: &mut Problems) -> List<Worker> {
    let count = match super::member(document, 0, b"workers") {
        Some(at) => match config::read::list(document, at, document.len()) {
            Ok(ats) => ats.len(),
            Err(_) => 0,
        },
        None => 0,
    };
    let mut workers = List::with_capacity(count);
    for _ in 0..count {
        match directories.join(config::Directory::Data, b"workspaces") {
            Ok(root) => workers.push(Worker { root }).expect("bounded workers"),
            Err(what) => problems.push(Path::root().member(b"application").member(b"workers"), What::Jig(what)),
        }
    }
    workers
}
fn webhooks(document: &Document, at: Option<u32>, path: &Path, problems: &mut Problems) -> Option<Webhooks> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let value = clients(document, at, &path.member(b"webhooks"), false, problems)?;
    Some(Webhooks { listener: value.listener })
}
pub fn parse(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Members> {
    parse_over(document, at, jig, directories, temper_service_core::derived::STOCK, problems)
}
pub fn parse_over(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    directories: &config::Directories,
    sources: temper_service_core::derived::Sources,
    problems: &mut Problems,
) -> Option<Members> {
    let stock = limits::stock();
    let mut bounds = match limits::base(stock.live_goals, stock.tool_payload, sources) {
        Ok((_jig, own)) => own,
        Err(found) => {
            problems.push(found.path, What::Jig(found.what));
            return None;
        }
    };
    if let Some(slots) = temper_service_core::derived::slots(document) {
        bounds.checkout.workspaces = slots;
        bounds.git_component.operations = slots;
    }
    parse_with_limits(document, at, jig, directories, &bounds, problems)
}
/// Parse the application against a world's typed limits.
pub fn parse_with_limits(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    directories: &config::Directories,
    bounds: &limits::Limits,
    problems: &mut Problems,
) -> Option<Members> {
    tier_bounds(bounds, problems);
    let path = Path::root().member(b"application");
    let mut object = read::object(document, at, "application", &path, problems)?;
    let overrides = List::with_capacity(0);
    if object.member(document, b"machine").is_some() {
        problems.push(path.member(b"machine"), What::Jig(config::What::Fixed));
    }
    let test_at = object.member(document, b"test");
    if object.member(document, b"demo").is_some() {
        problems.push(path.member(b"demo"), What::Jig(config::What::Fixed));
    }
    if object.member(document, b"limits").is_some() {
        problems.push(path.member(b"limits"), What::Jig(config::What::Fixed));
    }
    if object.member(document, b"workers").is_some() {
        problems.push(path.member(b"workers"), What::Jig(config::What::Fixed));
    }
    if object.member(document, b"charter").is_some() {
        problems.push(path.member(b"charter"), What::Half(crate::startup::EngineWhat::CharterStored));
    }
    let clients_at = read::required(&mut object, document, b"clients", &path, problems);
    let clients = match clients_at {
        Some(at) => clients(document, at, &path.member(b"clients"), true, problems),
        None => None,
    };
    let sign_in_at = read::nullable(&mut object, document, b"sign_in", &path, problems);
    let sign_in = sign_in::parse(document, sign_in_at, jig, bounds.sign_in, directories, problems);
    let push_at = object.member(document, b"push");
    let push = push::parse(document, push_at, directories, problems);
    if object.member(document, b"landing").is_some() {
        problems.push(path.member(b"landing"), What::Jig(config::What::Fixed));
    }
    if object.member(document, b"welcome").is_some() {
        problems.push(path.member(b"welcome"), What::Jig(config::What::Fixed));
    }
    let at = read::required(&mut object, document, b"webhooks", &path, problems);
    let webhooks = webhooks(document, at, &path, problems);
    let workers = workers(document, directories, problems);
    let environment = if workers.is_empty() { None } else { object.member(document, b"environment") };
    read::finish(object, document, &path, problems);
    let mut found = config::Problems::empty();
    let test_github = temper_forge_setup::machine::read_test(document, test_at, directories, &mut found);
    problems.append(found, &Path::root());
    if let Some(sign_in) = &sign_in {
        sign_in::check_forges(&sign_in.forges, jig, problems);
    }
    if !problems.is_empty() {
        return None;
    }
    Some(Members {
        clients: clients?,
        sign_in,
        push,
        webhooks,
        test_github,
        demo: None,
        workers: workers.into_boxed(),
        environment,
        limits: *bounds,
        overrides: overrides.into_boxed(),
    })
}
/// The startup files named by the application (temper/docs/design/protocol/configuration/engine.md).
#[must_use]
pub fn files(members: &Members) -> Box<[config::File]> {
    named_files(members.sign_in.as_ref(), members.push.as_ref(), &members.limits)
}
#[must_use]
pub fn named_files(
    sign_in: Option<&sign_in::SignIn>,
    push: Option<&push::Push>,
    limits: &limits::Limits,
) -> Box<[config::File]> {
    let mut files = List::with_capacity(
        limits
            .forge
            .forges
            .checked_mul(4)
            .expect("bounded forge files")
            .checked_add(3)
            .expect("sign-in and push files"),
    );
    if let Some(sign_in) = sign_in {
        sign_in::files(sign_in, limits.sign_in, &mut files);
    }
    if let Some(push) = push {
        files.push(push::file(push)).expect("push key file");
    }
    files.into_boxed()
}
#[derive(Debug)]
pub struct Resolved {
    pub sign_in: Option<settings::SignIn>,
    pub push: Option<settings::Push>,
    pub webhooks: Option<skein_forge_connection::hooks::Webhooks>,
}
pub(super) fn contents(
    files: &settings::Files,
    name: &[u8],
    max: u32,
    at: Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    for (path, value) in &files.entries {
        if path.as_ref() == name {
            if value.expose().len() > usize::try_from(max).expect("file bound") {
                problems.push(at, What::Jig(config::What::Bound));
                return None;
            }
            return Some(bytes::copy_of(value.expose()));
        }
    }
    problems.push(at, What::Jig(config::What::Unreadable));
    None
}
fn joined(
    directories: &config::Directories,
    kind: config::Directory,
    value: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    match directories.join(kind, value) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            None
        }
    }
}

pub fn resolve(members: &Members, files: &settings::Files, problems: &mut Problems) -> Option<Resolved> {
    resolve_parts(
        members.sign_in.as_ref(),
        members.push.as_ref(),
        members.webhooks.as_ref(),
        &members.limits,
        files,
        problems,
    )
}
pub fn resolve_parts(
    sign_in: Option<&sign_in::SignIn>,
    push: Option<&push::Push>,
    webhooks: Option<&Webhooks>,
    limits: &limits::Limits,
    files: &settings::Files,
    problems: &mut Problems,
) -> Option<Resolved> {
    let sign_in = match sign_in {
        Some(sign_in) => sign_in::resolve(sign_in, limits.sign_in, files, problems),
        None => None,
    };
    let push = match push {
        Some(push) => push::resolve(push, files, problems),
        None => None,
    };
    let webhooks = match webhooks {
        Some(value) => match value.listener {
            config::Address::Addr(listener) => Some(skein_forge_connection::hooks::Webhooks { listener }),
            config::Address::Host { .. } => None,
        },
        None => None,
    };
    if problems.is_empty() { Some(Resolved { sign_in, push, webhooks }) } else { None }
}

fn transport(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Transport> {
    if document.token(at)?.kind == Kind::String {
        let value = match config::read::text(document, at, 16) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.clone(), What::Jig(what));
                return None;
            }
        };
        read::form(value.as_ref() == b"plaintext", path.clone(), problems);
        return Some(Transport::Plaintext);
    }
    let mut outer = read::object(document, at, "application-transport", path, problems)?;
    let tls = read::required(&mut outer, document, b"tls", path, problems);
    read::finish(outer, document, path, problems);
    let path = path.member(b"tls");
    let mut object = read::object(document, tls?, "application-tls", &path, problems)?;
    let server_name = read::text(&mut object, document, b"server_name", &path, problems);
    let at = read::required(&mut object, document, b"trust", &path, problems);
    let mut trust = None;
    if let Some(at) = at
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 1_048_576) {
            Ok(value) => trust = Some(value),
            Err(what) => problems.push(path.member(b"trust"), What::Jig(what)),
        }
    }
    read::finish(object, document, &path, problems);
    Some(Transport::Tls { server_name: server_name?, trust })
}

fn resolved_transport(
    transport: &Transport,
    files: &settings::Files,
    path: &Path,
    problems: &mut Problems,
) -> settings::Transport {
    match transport {
        Transport::Plaintext => settings::Transport::Plaintext,
        Transport::Tls { server_name, trust } => {
            let trust = match trust {
                Some(name) => contents(
                    files,
                    name,
                    1_048_576,
                    path.member(b"transport").member(b"tls").member(b"trust"),
                    problems,
                ),
                None => None,
            };
            settings::Transport::Tls { server_name: server_name.clone(), trust }
        }
    }
}
