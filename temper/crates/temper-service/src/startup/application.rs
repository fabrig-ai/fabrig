//! Machine and client settings, read strictly (temper/docs/design/protocol/configuration/engine.md).
//! A box's sign-in is exclusive (temper/docs/design/protocol/openid-provider.md, 9).
mod demo;
mod landing;
mod public;
mod read;
pub mod sign_in;
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
pub use public::Origin;
#[cfg(test)]
pub use public::Scheme;
use skein_json::{Document, document::Kind};
use skein_lib::{List, bytes};

#[derive(Debug)]
pub struct Members {
    pub clients: Clients,
    pub sign_in: Option<sign_in::SignIn>,
    pub webhooks: Option<Webhooks>,
    pub machine: temper_forge_setup::machine::Machine,
    pub demo: Option<settings::Demo>,
    pub welcome: bool,
    pub landings: skein_lib::Map<u32, temper_engine_domain::ProjectLanding>,
    /// The landing's projects in the document's order, which names a refused one's path.
    pub landing_projects: Box<[u32]>,
    pub limits: limits::Limits,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
    pub workers: Box<[Worker]>,
}
/// The application's clients listener (protocol/configuration/engine.md).
#[derive(Debug)]
pub struct Clients {
    /// The socket address used for the listener, Host guard and printed links.
    pub listener: config::Address,
    /// `application.clients.bundle`: the bundle's directory; `None` serves no browser.
    pub bundle: Option<Box<[u8]>>,
    /// The browser origin and jig public name (protocol/sign-in.md, 2.2).
    pub public: Option<Origin>,
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
/// The forge's browser origin (protocol/openid-provider.md, 1 and 7).
pub(crate) fn forge_origin(
    git: &[u8],
    web: Option<&[u8]>,
    public: Option<&temper_forge_setup::setup::Origin>,
) -> Box<[u8]> {
    let Some(public) = public else { return bytes::copy_of(web.unwrap_or(git)) };
    let port = skein_lib::Decimal::of(u64::from(public.port));
    let mut parts = List::with_capacity(4);
    parts.push(b"https://".as_slice()).expect("four origin pieces");
    parts.push(public.host.as_ref()).expect("four origin pieces");
    if public.port != 443 {
        parts.push(b":".as_slice()).expect("four origin pieces");
        parts.push(port.as_bytes()).expect("four origin pieces");
    }
    let mut length = 0_usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("bounded origin");
    }
    let mut out = skein_lib::Writer::new(length);
    for part in &parts {
        out.put(part).expect("measured origin");
    }
    out.finish()
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
    #[cfg(feature = "worker")]
    pub git: Option<Box<[u8]>>,
}
fn configured_limits(
    document: &Document,
    at: u32,
    path: &Path,
    problems: &mut Problems,
    handed: &mut List<(Box<[u8]>, u64)>,
) -> Option<limits::Limits> {
    let mut object = read::object(document, at, "application-limits", path, problems)?;
    let profile = read::text(&mut object, document, b"profile", path, problems);
    let overrides = read::required(&mut object, document, b"overrides", path, problems);
    read::finish(object, document, path, problems);
    let profile = match profile.as_deref() {
        Some(b"standard") => limits::Profile::Standard,
        Some(b"small") | None => limits::Profile::Small,
        Some(_) => {
            problems.push(path.member(b"profile"), What::Profile);
            limits::Profile::Small
        }
    };
    let mut limits = limits::profile(profile);
    let path = path.member(b"overrides");
    if let Some(at) = overrides
        && let Some(mut object) = read::overrides(document, at, &path, problems)
    {
        let end = config::read::after(document, at).ok()?;
        for index in at..end {
            let Some(token) = document.token(index) else { continue };
            if token.kind != Kind::Key {
                continue;
            }
            let name = document.text(token).expect("object key text");
            let Some(value_at) = object.member(document, name) else { continue };
            match config::read::integer(document, value_at, u64::MAX) {
                Ok(value) => {
                    if limits::handed_on(name) {
                        handed.push((bytes::copy_of(name), value)).expect("bounded overrides");
                    }
                    if (!limits::handed_on(name) || limits::host_setup(name))
                        && let Err(what) = limits::apply(&mut limits, name, value)
                    {
                        problems.push(path.member(name), What::Override(what));
                    }
                }
                Err(config::What::Bound) => problems.push(path.member(name), What::Override(limits::Refused::Range)),
                Err(what) => problems.push(path.member(name), What::Jig(what)),
            }
        }
        let mut found = config::Problems::empty();
        object.finish(document, &path, &mut found);
        problems.append(found, &Path::root());
    }
    // Both runtime startup and offline store readers pass here before sizing the store.
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
    // Pass the resolved client capacity once, including the profile default.
    handed
        .push((bytes::copy_of(b"forge.client.repositories"), u64::from(limits.forge.client.repositories)))
        .expect("bounded setup inputs");
    Some(limits)
}
fn workers(
    document: &Document,
    ats: Option<List<u32>>,
    count: u32,
    directories: &config::Directories,
    path: &Path,
    problems: &mut Problems,
) -> List<Worker> {
    let mut workers = List::with_capacity(count);
    if let Some(ats) = ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"workers"), index);
            let Some(mut object) = read::object(document, *at, "application-worker", &path, problems) else { continue };
            let root = read::text(&mut object, document, b"root", &path, problems);
            let at = read::required(&mut object, document, b"git", &path, problems);
            let mut git = None;
            if let Some(at) = at {
                let null = match document.token(at) {
                    Some(token) => token.kind == Kind::Null,
                    None => false,
                };
                if !null {
                    match config::read::text(document, at, 1_048_576) {
                        Ok(value) => git = Some(value),
                        Err(what) => problems.push(path.member(b"git"), What::Jig(what)),
                    }
                }
            }
            read::finish(object, document, &path, problems);
            if let Some(root) = root {
                if let Some(git) = &git
                    && !git.starts_with(b"/")
                {
                    problems.push(path.member(b"git"), What::Absolute);
                }
                if let Some(root) = joined(directories, config::Directory::Data, &root, &path.member(b"root"), problems)
                {
                    workers
                        .push(Worker {
                            root,
                            #[cfg(feature = "worker")]
                            git,
                        })
                        .expect("bounded workers");
                }
            }
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
    let path = Path::root().member(b"application");
    let mut object = read::object(document, at, "application", &path, problems)?;
    let mut overrides = List::with_capacity(document.len());
    let machine_at = read::required(&mut object, document, b"machine", &path, problems);
    let demo_at = read::required(&mut object, document, b"demo", &path, problems);
    let limit_at = read::required(&mut object, document, b"limits", &path, problems);
    let limits = match limit_at {
        Some(at) => configured_limits(document, at, &path.member(b"limits"), problems, &mut overrides),
        None => None,
    };
    let bounds = limits.unwrap_or(limits::profile(limits::Profile::Small));
    let workers_ats = read::list(&mut object, document, b"workers", jig.limits.core.fleet.workers, &path, problems);
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
    let landing_at = object.member(document, b"landing");
    let welcome = match object.member(document, b"welcome") {
        Some(at) => match config::read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"welcome"), What::Jig(what));
                false
            }
        },
        None => false,
    };
    let at = read::required(&mut object, document, b"webhooks", &path, problems);
    let webhooks = webhooks(document, at, &path, problems);
    read::finish(object, document, &path, problems);
    let machine = match machine_at {
        Some(at) => {
            let mut found = config::Problems::empty();
            let machine = temper_forge_setup::machine::read_config(document, at, directories, &mut found);
            problems.append(found, &Path::root());
            machine
        }
        None => None,
    };
    if let Some(machine) = &machine
        && let Some(owned) = &machine.owned
        && let Some(client) = &owned.sign_in
    {
        let mut found = false;
        if let Some(provider) = &jig.clients.provider {
            for configured in &provider.clients {
                if configured.id == *client {
                    found = true;
                    break;
                }
            }
        }
        read::form(sign_in::visible(client), path.member(b"machine").member(b"owned").member(b"sign_in"), problems);
        if !found {
            problems.push(path.member(b"machine").member(b"owned").member(b"sign_in"), What::Jig(config::What::Refers));
        }
        if sign_in.is_some() {
            problems.push(
                path.member(b"machine").member(b"owned").member(b"sign_in"),
                What::Half(crate::startup::EngineWhat::BoxSignIn),
            );
        }
    }
    if let Some(sign_in) = &sign_in {
        sign_in::check_forges(&sign_in.forges, jig, problems);
    }
    let demo = demo::parse(document, demo_at, directories, problems);
    let workers = workers(document, workers_ats, jig.limits.core.fleet.workers, directories, &path, problems);
    let landing = match landing_at {
        Some(at) => landing::parse(document, at, jig, &bounds, problems),
        None => Some((skein_lib::Map::with_capacity(0), Box::default())),
    };
    if !problems.is_empty() {
        return None;
    }
    let (landings, landing_projects) = landing?;
    Some(Members {
        clients: clients?,
        sign_in,
        webhooks,
        machine: machine?,
        demo,
        welcome,
        workers: workers.into_boxed(),
        limits: limits?,
        overrides: overrides.into_boxed(),
        landings,
        landing_projects,
    })
}
/// The startup files named by the application (temper/docs/design/protocol/configuration/engine.md).
pub fn files(members: &Members) -> Box<[config::File]> {
    let mut files = List::with_capacity(
        members.limits.forge.forges.checked_mul(4).expect("bounded forge files").checked_add(2).expect("sign-in files"),
    );
    if let Some(sign_in) = &members.sign_in {
        sign_in::files(sign_in, members.limits.sign_in, &mut files);
    }
    files.into_boxed()
}
#[derive(Debug)]
pub struct Resolved {
    pub sign_in: Option<settings::SignIn>,
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
    let sign_in = match &members.sign_in {
        Some(sign_in) => sign_in::resolve(sign_in, members.limits.sign_in, files, problems),
        None => None,
    };
    let webhooks = match &members.webhooks {
        Some(value) => match value.listener {
            config::Address::Addr(listener) => Some(skein_forge_connection::hooks::Webhooks { listener }),
            config::Address::Host { .. } => None,
        },
        None => None,
    };
    if problems.is_empty() { Some(Resolved { sign_in, webhooks }) } else { None }
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
