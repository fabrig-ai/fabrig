//! Kept forges join the setup host and lifecycle (temper/docs/design/draft/playground-reset.md, 3 and 6.4).
use super::{lifecycle, settings};
use alloc::boxed::Box;
use skein_forge_setup::host;
use skein_lib::{List, Map, Secret};
use temper_engine_domain_forge::{self as domain, ForgeRecord};
use temper_forge_setup::{machine, setup};

pub(super) struct Runtime {
    machine: machine::Machine,
    directories: jig_protocol_config::Directories,
    limits: crate::limits::Limits,
    url: Option<Box<[u8]>>,
    webhooks: Option<core::net::SocketAddr>,
    endpoints: Map<u16, host::Forge>,
    resolved: Map<u16, Endpoint>,
    pub(super) initial: List<ForgeRecord>,
}
#[derive(Debug)]
struct Endpoint {
    address: skein_io::Addr,
    transport: host::Transport,
}
impl Runtime {
    pub(super) fn new(config: &crate::settings::Engine, settings: &host::Settings) -> Self {
        let mut endpoints = Map::with_capacity(config.limits.forge.forges);
        for forge in &settings.forges {
            endpoints.insert(forge.number(), forge.clone()).expect("document setup endpoints");
        }
        let initial = List::with_capacity(config.limits.forge.forges);
        Self {
            machine: config.machine.clone(),
            directories: config.directories.clone(),
            limits: config.limits,
            url: match &config.clients.public {
                Some(origin) => Some(origin.base()),
                None => settings::listener(&config.clients.listener).ok(),
            },
            webhooks: match &config.webhooks {
                Some(hooks) => match hooks.listener {
                    skein_io::Addr::Ip(address) => Some(address),
                    skein_io::Addr::InProcess(_) => None,
                },
                None => None,
            },
            endpoints,
            resolved: Map::with_capacity(config.limits.forge.forges),
            initial,
        }
    }
    #[expect(clippy::manual_map, reason = "the configuration boundary uses explicit matches")]
    pub(super) fn git_program(&self, mode: domain::Mode) -> Option<Box<[u8]>> {
        match mode {
            domain::Mode::Fake => match &self.machine.fake {
                Some(fake) => Some(fake.git.clone()),
                None => None,
            },
            domain::Mode::Connected | domain::Mode::Managed | domain::Mode::Owned => None,
        }
    }
    pub(super) fn resolved(&mut self, endpoint: crate::ForgeRuntime) {
        let transport = match endpoint.transport {
            skein_forge_connection::settings::Transport::Plaintext { name } => host::Transport::Plaintext { name },
            skein_forge_connection::settings::Transport::Tls { server_name, trust } => {
                host::Transport::Tls { server_name, trust }
            }
        };
        self.resolved
            .insert(endpoint.number, Endpoint { address: skein_io::Addr::Ip(endpoint.address), transport })
            .expect("admitted setup endpoint");
    }
    pub(super) fn listening(&mut self, address: core::net::SocketAddr) {
        self.webhooks = Some(address);
        let mut numbers = List::with_capacity(self.endpoints.len());
        for (number, _) in &self.endpoints {
            numbers.push(*number).expect("setup endpoint numbers");
        }
        for number in &numbers {
            let forge = self.endpoints.get_mut(number).expect("held setup endpoint");
            match &mut forge.kind {
                host::Kind::Forgejo(given) => {
                    if given.listener.is_some() {
                        given.listener = Some(address);
                    }
                }
                host::Kind::GitHub(_) => {}
            }
        }
    }
    pub(super) fn forget(&mut self, forge: u16) {
        self.endpoints.remove(&forge);
        self.resolved.remove(&forge);
        let capacity = self.initial.capacity();
        let initial = core::mem::replace(&mut self.initial, List::with_capacity(capacity));
        for record in initial.into_boxed() {
            if record.number != forge {
                self.initial.push(record).expect("remaining pending forges");
            }
        }
    }
    fn entry(&self, record: &ForgeRecord) -> Option<setup::Entry> {
        let mut entry = match record.mode {
            domain::Mode::Owned | domain::Mode::Fake => {
                let kind =
                    if record.mode == domain::Mode::Owned { machine::BoxKind::Owned } else { machine::BoxKind::Fake };
                machine::entry(&self.machine, record.number, kind, &self.directories)?
            }
            domain::Mode::Managed => setup::Entry {
                forge: record.number,
                directory: Box::new([]),
                bot: setup::Account { login: record.bot.login.clone(), email: record.bot.email.clone() },
                orgs: Box::new([]),
                mode: setup::Mode::Managed { admin_token: None },
                asked: false,
            },
            domain::Mode::Connected => return None,
        };
        entry.bot = setup::Account { login: record.bot.login.clone(), email: record.bot.email.clone() };
        entry.asked = record.asked;
        Some(entry)
    }
    fn forge(
        &self,
        record: &ForgeRecord,
        source: &crate::settings::Forge,
        entry: Option<&setup::Entry>,
    ) -> Option<host::Forge> {
        if let Some(forge) = self.endpoints.get(&record.number) {
            return Some(forge.clone());
        }
        let (address, transport) = match self.resolved.get(&record.number) {
            Some(endpoint) => (endpoint.address, endpoint.transport.clone()),
            None => (skein_io::Addr::Ip(direct(source)?), host::Transport::Plaintext { name: None }),
        };
        match &transport {
            host::Transport::Plaintext { .. } => match address {
                skein_io::Addr::Ip(address) if address.ip().is_loopback() => {}
                skein_io::Addr::Ip(_) | skein_io::Addr::InProcess(_) => return None,
            },
            host::Transport::Tls { .. } => {}
        }
        let kind = match record.kind {
            domain::Kind::GitHub => host::Kind::GitHub(
                settings::github(
                    record.number,
                    record.read_only,
                    record.place.web.as_deref(),
                    self.url.as_ref()?.clone(),
                    &self.limits,
                )
                .ok()?,
            ),
            domain::Kind::Forgejo => {
                let given = match entry {
                    Some(entry) => {
                        let mut limits = self.limits.setup.plan();
                        limits.repositories = self.limits.forge.client.repositories;
                        limits.reachable = self.limits.picker.page;
                        temper_forge_setup::given::values(
                            entry,
                            if record.mode == domain::Mode::Managed { None } else { self.webhooks },
                            &limits,
                        )
                    }
                    None => settings::connected(record.number, &self.limits),
                };
                host::Kind::Forgejo(given)
            }
        };
        Some(host::Forge { kind, address, transport })
    }
    fn local(&self, record: &ForgeRecord, entry: &setup::Entry, restored: bool) -> lifecycle::Local {
        lifecycle::Local {
            source: if record.mode == domain::Mode::Managed {
                lifecycle::Source::Managed
            } else {
                lifecycle::Source::Prepared
            },
            first: intent(record.first.as_ref()),
            first_source: if record.first.is_some() {
                lifecycle::IntentSource::Record
            } else {
                lifecycle::IntentSource::Request
            },
            repositories: match &entry.mode {
                setup::Mode::Fake(fake) => u64::try_from(fake.repositories.len()).expect("bounded fake repositories"),
                setup::Mode::Owned(_) | setup::Mode::Managed { .. } => 0,
            },
            pending: true,
            restored,
            limits: {
                let mut limits = self.limits.setup.plan();
                limits.repositories = self.limits.forge.client.repositories;
                limits
            },
            webhooks: self.webhooks.is_some() && record.mode != domain::Mode::Managed,
            bot_login: record.bot.login.clone(),
            identity: crate::settings::Identity {
                number: u32::from(record.number),
                forge: record.number,
                token: Secret::new(Box::new([])),
                login: record.bot.login.clone(),
                author: crate::settings::Author { name: record.bot.name.clone(), email: record.bot.email.clone() },
            },
            handover: None,
            path: temper_forge_setup::place::Place { directory: entry.directory.clone(), user: Box::new([]) }
                .path(temper_forge_setup::place::File::AdminToken),
            file: lifecycle::FileStep::Idle,
            trigger: if record.mode == domain::Mode::Fake {
                lifecycle::Trigger::StartFake
            } else if record.asked {
                lifecycle::Trigger::StartAsked
            } else {
                lifecycle::Trigger::Start
            },
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "setup protocol interprets a checked numeric API host")]
fn direct(source: &crate::settings::Forge) -> Option<core::net::SocketAddr> {
    match &source.transport {
        crate::settings::Transport::Plaintext => {}
        crate::settings::Transport::Tls { .. } => return None,
    }
    match &source.api {
        jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => Some(*address),
        jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => None,
        jig_protocol_config::Address::Host { name, port } => {
            let ip = core::str::from_utf8(name).ok()?.parse().ok()?;
            Some(core::net::SocketAddr::new(ip, *port))
        }
    }
}
#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
fn intent(first: Option<&domain::FirstIntent>) -> Option<lifecycle::Intent> {
    match first {
        Some(first) => Some(lifecycle::Intent {
            playground: first.playground,
            org: first.org.clone(),
            repository: first.repository.clone(),
            label: first.label.clone(),
        }),
        None => None,
    }
}

/// The service table holds the forge; named and TLS APIs use `Layer::resolved`.
pub(crate) fn keep(engine: &mut crate::engine::Engine, record: &ForgeRecord) {
    if record.state != domain::ForgeState::Kept {
        super::forget(engine, record.number);
        return;
    }
    if engine.setup.closing || engine.setup.closed {
        return;
    }
    let Some(runtime) = &engine.setup.runtime else { return };
    let Some(source) = engine.workers.forges.get(record.number) else { return };
    let document = runtime.endpoints.contains_key(&record.number);
    let entry = runtime.entry(record);
    let Some(forge) = runtime.forge(record, source, entry.as_ref()) else { return };
    match &forge.kind {
        host::Kind::GitHub(given) => engine.setup.listener.clone_from(&given.url),
        host::Kind::Forgejo(_) => {}
    }
    let local = match entry {
        Some(entry) if !document => Some(runtime.local(record, &entry, engine.setup.lifecycle.startup_restored())),
        Some(_) | None => None,
    };
    if !engine.setup.served.contains_key(&record.number) {
        if let Some((settings, _)) = &mut engine.setup.pending {
            if let host::Kind::Forgejo(given) = &forge.kind
                && let Some(listener) = given.listener
                && listener.port() == 0
            {
                engine.setup.waiting_for_webhooks = true;
            }
            settings.forges.push(forge).expect("admitted setup forge");
            if let Some(runtime) = &mut engine.setup.runtime {
                runtime.initial.push(record.clone()).expect("pending kept forges");
            }
        } else if engine.setup.component.is_some() {
            engine.setup.down(host::Request::Add(forge));
        } else {
            return;
        }
        engine.setup.served.insert(record.number, ()).expect("admitted setup forge");
        engine.setup.unserved.remove(&record.number);
    }
    if let Some(local) = engine.setup.lifecycle.local_mut(record.number)
        && local.first_source == lifecycle::IntentSource::Request
        && record.first.is_some()
    {
        local.first = intent(record.first.as_ref());
        local.first_source = lifecycle::IntentSource::Record;
    }
    if !engine.setup.lifecycle.reconcile.contains_key(&record.number) {
        let pending = match local {
            Some(local) => lifecycle::Reconcile::Local(local),
            None if record.kind == domain::Kind::GitHub && !document => lifecycle::Reconcile::GitHub,
            None => return,
        };
        engine.setup.lifecycle.reconcile.insert(record.number, pending).expect("one lifecycle per forge");
    }
}
