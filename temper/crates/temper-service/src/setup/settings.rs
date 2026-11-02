//! Mapping the configured setup forges to the pinned host (protocol/setup.md, 5.1).
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_forge_setup::{github, host};
use skein_lib::{CryptoRng, List, Writer};

pub(super) fn fixed(
    config: &crate::settings::Engine,
    _forges: &skein_forge_connection::Settings,
) -> Result<(host::Settings, host::Limits), Problems> {
    let own = &config.limits;
    check(&own.setup)?;
    let configured = List::with_capacity(own.forge.forges);
    let f = own.forge_component;
    let limits = host::Limits {
        forges: own.forge.forges,
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
pub(super) fn github(
    forge: u16,
    read_only: bool,
    web: Option<&[u8]>,
    url: Box<[u8]>,
    own: &crate::limits::Limits,
) -> Result<github::given::Given, Problems> {
    let pure = own.setup.plan();
    let number = skein_lib::Decimal::of(u64::from(forge));
    let given = github::given::Given {
        forge,
        // A person's installations are kept; a shared App's hook is left as read (setup.md, 17.7).
        repair_hook: !read_only,
        uninstall_strays: false,
        word: Box::from(b"temper".as_slice()),
        web: match web {
            Some(web) => Box::from(web),
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
            // At most one installation per page, plus the terminal empty page.
            pages: own.forges.installations.checked_add(1).ok_or(problem(b"forges.installations"))?,
            state_life: own.setup.state_life,
            deadline: pure.deadline,
            backoff: pure.backoff,
            backoff_max: pure.backoff_max,
        },
    };
    if let Err(refused) = github::given::check(&given) {
        let mut found = Problems::empty();
        problems(host::Unusable::GitHubGiven { forge, refused }, &mut found);
        return Err(found);
    }
    Ok(given)
}
pub(super) fn connected(number: u16, own: &crate::limits::Limits) -> skein_forge_setup::given::Given {
    // A connected Forgejo runs only read plans here. The kit requires the writing
    // plan's names too; use temper's valid constants, never the connected user's login.
    let mut limits = own.setup.plan();
    limits.reachable = own.picker.page;
    limits.repositories = own.forge.client.repositories;
    skein_forge_setup::given::Given {
        forge: number,
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
    }
}
pub(crate) fn prepare(
    config: &crate::settings::Engine,
    forges: &mut skein_forge_connection::Settings,
    seed: &skein_lib::Secret,
    _under_box: bool,
) -> Result<super::Layer, Problems> {
    let (settings, limits) = fixed(config, forges)?;
    let own = &config.limits;
    let crypto = CryptoRng::new(seed, 5).ok_or(problem(b"setup"))?;
    let runtime = super::runtime::Runtime::new(config, &settings);
    let mut layer = super::Layer::new(
        settings,
        limits,
        crypto,
        own.setup.state_life,
        own.records.value_bytes,
        own.forge_component.installations,
    );
    layer.runtime = Some(runtime);
    resolved(&mut layer, forges);
    Ok(layer)
}
fn resolved(layer: &mut super::Layer, forges: &skein_forge_connection::Settings) {
    for endpoint in &forges.forges {
        match endpoint.address {
            skein_io::Addr::Ip(address) => layer.resolved(crate::ForgeRuntime {
                log_origins: endpoint.log_origins.clone(),
                number: endpoint.number,
                address,
                transport: match &endpoint.transport {
                    skein_forge_connection::settings::Transport::Plaintext { name } => {
                        skein_forge_connection::settings::Transport::Plaintext { name: name.clone() }
                    }
                    skein_forge_connection::settings::Transport::Tls { server_name, trust } => {
                        skein_forge_connection::settings::Transport::Tls {
                            server_name: server_name.clone(),
                            trust: trust.clone(),
                        }
                    }
                },
            }),
            skein_io::Addr::InProcess(_) => {}
        }
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
pub(super) fn listener(address: &jig_protocol_config::Address) -> Result<Box<[u8]>, Problems> {
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
