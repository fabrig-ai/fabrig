//! Mapping the configured setup forges to the pinned host (protocol/setup.md, 5.1).
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_forge_setup::{github, host};
use skein_lib::{CryptoRng, List, Writer};
use temper_forge_setup::setup::Mode;

/// Values needed to build a forge's host settings after startup (temper/docs/design/protocol/setup.md, 5.1).
pub(crate) struct Statics {
    pub(super) clients: jig_protocol_config::Address,
    pub(super) public: Option<crate::startup::application::Origin>,
    pub(super) webhooks: Option<core::net::SocketAddr>,
    pub(super) webhooks_bound: bool,
    pub(super) limits: crate::limits::Limits,
    pub(super) directories: jig_protocol_config::Directories,
    pub(super) under_box: bool,
}
impl Statics {
    fn new(config: &crate::settings::Engine, under_box: bool) -> Self {
        Self {
            clients: config.clients.listener.clone(),
            public: config.clients.public.clone(),
            webhooks: match &config.webhooks {
                Some(hooks) => match hooks.listener {
                    skein_io::Addr::Ip(address) => Some(address),
                    skein_io::Addr::InProcess(_) => unreachable!("checked webhooks listener"),
                },
                None => None,
            },
            webhooks_bound: config.webhooks.is_none(),
            limits: config.limits,
            directories: config.directories.clone(),
            under_box,
        }
    }
}
pub(super) fn fixed(
    config: &crate::settings::Engine,
) -> Result<(host::Settings, host::Limits), Problems> {
    let statics = Statics::new(config, false);
    let own = &statics.limits;
    check(&own.setup)?;
    let configured = List::with_capacity(own.forge_component.forges);
    let f = own.forge_component;
    let limits = host::Limits {
        forges: f.forges,
        runs: own.setup.runs,
        states: own.setup.states,
        token_bytes: f.token_bytes,
        head_bytes: f.head_bytes,
        headers: f.headers,
        body_bytes_in: f.body_bytes_in,
        depth: f.depth,
        chunk: f.chunk,
        string_bytes: f.string_bytes.max(f.key_bytes.checked_mul(2).ok_or(problem(b"forge_component.key_bytes"))?),
        body_bytes: f.body_bytes,
        inner: f.inner.max(own.picker.page),
        target_bytes: f.target_bytes,
        io: config.jig.limits.io,
        tls: f.tls,
    };
    Ok((host::Settings { forges: configured, user_agent: Box::from(crate::forge::USER_AGENT.as_bytes()) }, limits))
}
pub(super) fn host_forge(
    statics: &Statics,
    source: &crate::settings::Forge,
    entry: Option<&temper_forge_setup::setup::Entry>,
    endpoint: (skein_io::Addr, host::Transport),
) -> Result<host::Forge, Problems> {
    let own = &statics.limits;
    let pure = own.setup.plan();
    let kind = match source.kind {
        crate::settings::Kind::Forgejo => host::Kind::Forgejo(forgejo(statics, source, entry)?),
        crate::settings::Kind::GitHub => {
            let url = match &statics.public {
                Some(origin) => origin.base(),
                None => listener(&statics.clients)?,
            };
            let number = skein_lib::Decimal::of(u64::from(source.number));
            let given = github::given::Given {
                forge: source.number,
                read_only: source.app_read_only,
                word: Box::from(b"temper".as_slice()),
                web: match &source.web {
                    Some(web) => web.clone(),
                    None => Box::from(b"https://github.com".as_slice()),
                },
                url: url.clone(),
                redirect_url: joined(&[&url, b"forges/", number.as_bytes(), b"/github/redirect"]),
                setup_url: joined(&[&url, b"forges/", number.as_bytes(), b"/github/setup"]),
                limits: github::limits::Limits {
                    key_bytes: own.forge_component.key_bytes,
                    name_bytes: own.forge.name_bytes,
                    page_bytes: own.forge_component.body_bytes,
                    installations: own.forges.installations,
                    reachable: own.picker.page,
                    pages: own.forges.installations.checked_add(1).ok_or(problem(b"forges.installations"))?,
                    state_life: own.setup.state_life,
                    deadline: pure.deadline,
                    backoff: pure.backoff,
                    backoff_max: pure.backoff_max,
                },
            };
            if let Err(refused) = github::given::check(&given) {
                let mut found = Problems::empty();
                problems(host::Unusable::GitHubGiven { forge: source.number, refused }, &mut found);
                return Err(found);
            }
            host::Kind::GitHub(given)
        }
    };
    Ok(host::Forge { kind, address: endpoint.0, transport: endpoint.1 })
}
fn forgejo(
    statics: &Statics,
    source: &crate::settings::Forge,
    entry: Option<&temper_forge_setup::setup::Entry>,
) -> Result<skein_forge_setup::given::Given, Problems> {
    let mut limits = statics.limits.setup.plan();
    limits.reachable = statics.limits.picker.page;
    limits.repositories = statics.limits.forge.client.repositories;
    let given = match entry {
        Some(entry) => {
            let listener = if admin_connect(&entry.mode) { None } else { statics.webhooks };
            temper_forge_setup::given::values(entry, listener, &limits)
        }
        None => skein_forge_setup::given::Given {
            forge: source.number,
            bot_login: b"temper".as_slice().into(),
            bot_email: b"temper@localhost".as_slice().into(),
            admin_name: b"temper-admin".as_slice().into(),
            api_name: b"temper-api".as_slice().into(),
            git_name: b"temper-git".as_slice().into(),
            orgs: Box::new([]),
            org_owner: None,
            adopted: Box::new([]),
            listener: None,
            word: b"temper".as_slice().into(),
            limits,
        },
    };
    if let Err(refused) = skein_forge_setup::given::check(&given) {
        let mut found = Problems::empty();
        problems(host::Unusable::Given { forge: source.number, refused }, &mut found);
        return Err(found);
    }
    Ok(given)
}
pub(crate) fn prepare(
    config: &crate::settings::Engine,
    seed: &skein_lib::Secret,
    under_box: bool,
) -> Result<super::Layer, Problems> {
    let (settings, limits) = fixed(config)?;
    let own = &config.limits;
    let crypto = CryptoRng::new(seed, 5).ok_or(problem(b"setup"))?;
    let layer = super::Layer::new(
        Statics::new(config, under_box),
        settings,
        limits,
        crypto,
        own.setup.state_life,
        own.records.value_bytes,
        own.forge_component.installations,
    );
    Ok(layer)
}
pub(crate) fn local(
    record: &temper_engine_domain_forge::ForgeRecord,
    entry: &temper_forge_setup::setup::Entry,
    statics: &Statics,
    under_box: bool,
    restored: bool,
) -> super::lifecycle::Local {
    let trigger = if record.playground {
        super::lifecycle::Trigger::StartDemo
    } else if admin_connect(&entry.mode) {
        super::lifecycle::Trigger::Start
    } else if under_box {
        super::lifecycle::Trigger::StartAsked
    } else {
        super::lifecycle::Trigger::Start
    };
    let mut local = local_entry(entry, statics, trigger, restored, crate::forges::identity(record));
    local.bot_login.clone_from(&record.bot.login);
    local
}
fn local_entry(
    entry: &temper_forge_setup::setup::Entry,
    statics: &Statics,
    trigger: super::lifecycle::Trigger,
    restored: bool,
    identity: crate::settings::Identity,
) -> super::lifecycle::Local {
    let managed = admin_connect(&entry.mode);
    let mut limits = statics.limits.setup.plan();
    limits.repositories = statics.limits.forge.client.repositories;
    super::lifecycle::Local {
        first: None,
        source: if managed { super::lifecycle::Source::Managed } else { super::lifecycle::Source::Prepared },
        repositories: match &entry.mode {
            Mode::Fake(fake) => u64::try_from(fake.repositories.len()).expect("checked repositories"),
            Mode::Owned(_) | Mode::Managed { .. } => 0,
        },
        pending: !restored,
        restored,
        handover: None,
        path: temper_forge_setup::place::Place { directory: entry.directory.clone(), user: Box::new([]) }
            .path(temper_forge_setup::place::File::AdminToken),
        file: super::lifecycle::FileStep::Idle,
        trigger,
        limits,
        webhooks: statics.webhooks.is_some() && !managed,
        bot_login: entry.bot.login.clone(),
        identity,
    }
}
fn check(own: &temper_forge_setup::limits::Host) -> Result<(), Problems> {
    let pure = own.plan();
    if pure.orgs == 0
        || own.runs == 0
        || own.states == 0
        || own.state_life == skein_lib::Duration::ZERO
        || own.state_life > github::limits::STATE_LIFE_MAX
    {
        return Err(problem(b"setup"));
    }
    if let Err(why) = skein_forge_setup::limits::check(&pure) {
        let mut found = Problems::empty();
        for place in [Some(why.place), why.other].into_iter().flatten() {
            use skein_forge_setup::given::Place;
            let name = match place {
                Place::Backoff => b"setup.backoff_ms".as_slice(),
                Place::BackoffMax => b"setup.backoff_max_ms",
                Place::Deadline => b"setup.deadline_ms",
                Place::FilesLimit
                | Place::FileBytesLimit
                | Place::CreateOrg
                | Place::CreateRepository
                | Place::Files
                | Place::FilePath(_)
                | Place::FileContent(_)
                | Place::Plan => unreachable!("Create inputs are admitted by the domain and fixed starter profile"),
                Place::Word
                | Place::BotLogin
                | Place::BotEmail
                | Place::AdminName
                | Place::ApiName
                | Place::GitName
                | Place::Orgs
                | Place::Org(_)
                | Place::OrgOwner
                | Place::Repositories
                | Place::Repository(_)
                | Place::OrgsLimit
                | Place::RepositoriesLimit => b"setup.orgs",
                Place::ReachableLimit | Place::Page => b"picker.page",
            };
            found.push(limit_path(name), What::Bound);
        }
        return Err(found);
    }
    Ok(())
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for part in parts {
        len = len.checked_add(part.len()).expect("bounded setup URL");
    }
    let mut writer = Writer::new(len);
    for part in parts {
        writer.put(part).expect("measured setup URL");
    }
    writer.finish()
}
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "configuration protocol renders a checked listener address"
)]
fn listener(address: &jig_protocol_config::Address) -> Result<Box<[u8]>, Problems> {
    let value = match address {
        jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => alloc::format!("http://{address}/"),
        jig_protocol_config::Address::Host { name, port } => {
            alloc::format!("http://{}:{port}/", core::str::from_utf8(name).expect("checked listener name"))
        }
        jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => return Err(problem(b"setup")),
    };
    Ok(value.into_bytes().into_boxed_slice())
}
pub(super) fn limit_path(name: &[u8]) -> Path {
    Path::root().member(b"application").member(b"limits").member(b"overrides").member(name)
}
fn problem(name: &[u8]) -> Problems {
    let mut found = Problems::empty();
    found.push(limit_path(name), What::Bound);
    found
}
/// All pinned host refusals name the members which feed that bound.
pub(crate) fn problems(why: host::Unusable, found: &mut Problems) {
    use host::{Stacked, Unusable};
    let names: &[&[u8]] = match why {
        Unusable::Zero => &[b"setup.runs", b"setup.states", b"forge_component", b"io"],
        Unusable::TokenBytes { .. } => &[b"forge_component.token_bytes"],
        Unusable::Documents(_) | Unusable::GitHubDocuments(_) => {
            &[b"forge_component.depth", b"forge_component.body_bytes_in", b"forge_component.chunk"]
        }
        Unusable::GitHubMetadata { .. } => &[b"forge_component.inner", b"forge_component.string_bytes"],
        Unusable::Stack { stacked, .. } => match stacked {
            Stacked::HttpOverIo => &[b"forge_component.chunk", b"io.intake", b"io.output"],
            Stacked::HttpOverTls => {
                &[b"forge_component.chunk", b"forge_component.tls.read", b"forge_component.tls.send"]
            }
            Stacked::TlsOverIo => {
                &[b"forge_component.tls.read", b"forge_component.tls.send", b"io.intake", b"io.output"]
            }
            Stacked::JsonOverHttp => &[b"forge_component.chunk"],
        },
        Unusable::Forges { .. } => &[b"forge_component.forges"],
        Unusable::Duplicate { forge }
        | Unusable::Address { forge }
        | Unusable::PlaintextName { forge }
        | Unusable::Request { forge }
        | Unusable::Given { forge, .. }
        | Unusable::GitHubGiven { forge, .. }
        | Unusable::Reachable { forge, .. } => {
            // Forge numbers, not array indices, are retained in the host diagnostic.
            let number = skein_lib::Decimal::of(u64::from(forge));
            found.push(Path::root().member(b"application").member(b"forges").member(number.as_bytes()), What::Bound);
            match why {
                Unusable::GitHubGiven { refused, .. } => {
                    for place in [Some(refused.place), refused.other].into_iter().flatten() {
                        found.push(limit_path(github_member(place)), What::Bound);
                    }
                    return;
                }
                Unusable::Reachable { .. } => &[b"picker.page", b"forge_component.inner"],
                Unusable::Zero
                | Unusable::TokenBytes { .. }
                | Unusable::Documents(_)
                | Unusable::GitHubDocuments(_)
                | Unusable::GitHubMetadata { .. }
                | Unusable::Stack { .. }
                | Unusable::Forges { .. }
                | Unusable::Duplicate { .. }
                | Unusable::Address { .. }
                | Unusable::PlaintextName { .. }
                | Unusable::Request { .. }
                | Unusable::Given { .. } => &[b"forge_component", b"setup"],
            }
        }
    };
    for name in names {
        found.push(limit_path(name), What::Bound);
    }
}
fn github_member(place: github::given::Place) -> &'static [u8] {
    use github::given::Place;
    match place {
        Place::KeyBytes => b"forge_component.key_bytes",
        Place::NameBytes => b"forge.name_bytes",
        Place::PageBytes => b"forge_component.body_bytes",
        Place::Installations | Place::Pages => b"forges.installations",
        Place::StateLife => b"setup.state_life_ms",
        Place::Deadline => b"setup.deadline_ms",
        Place::Backoff => b"setup.backoff_ms",
        Place::BackoffMax => b"setup.backoff_max_ms",
        Place::Reachable | Place::Page => b"picker.page",
        Place::Word
        | Place::Target
        | Place::State
        | Place::Web
        | Place::Url
        | Place::RedirectUrl
        | Place::SetupUrl
        | Place::Code => b"setup",
    }
}

fn admin_connect(mode: &Mode) -> bool {
    match mode {
        Mode::Managed { admin_token: None } => true,
        Mode::Managed { admin_token: Some(_) } | Mode::Owned(_) | Mode::Fake(_) => false,
    }
}
