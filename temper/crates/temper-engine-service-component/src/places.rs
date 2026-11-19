//! Routing's chosen and kept forge setups (temper/docs/design/protocol/setup/place.md, 2, 3 and 6).
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Token, bytes};
use temper_engine_domain_forge::{self as domain, BoxKind, ForgeRecord};
use temper_forge_setup::setup_file::{self, SetupFile};

/// Effects hosted by the shell; worlds supply ports and files at this boundary.
#[derive(Debug)]
pub enum PlaceIo {
    Choose { request: Token },
    Read { forge: u16, kind: temper_forge_setup::machine::BoxKind, directories: jig_protocol_config::Directories },
    Write { forge: u16, setup: Box<SetupFile>, directories: jig_protocol_config::Directories },
}
#[derive(Debug)]
struct Choosing {
    kind: BoxKind,
    login: Box<[u8]>,
    tries: u8,
}
pub(crate) struct Places {
    directories: jig_protocol_config::Directories,
    provider: bool,
    owned_forgejo: Option<Box<[u8]>>,
    owned_runner: Option<Box<[u8]>>,
    blocked: List<u16>,
    choosing: Map<Token, Choosing>,
    chosen: Map<u16, SetupFile>,
    pub(crate) files: Map<u16, SetupFile>,
    records: Map<u16, ForgeRecord>,
    registered: Map<u16, ()>,
    starts: Map<u16, (BoxKind, Token)>,
    io: Queue<PlaceIo>,
}
impl Places {
    pub(crate) fn listening(&mut self, address: core::net::SocketAddr) {
        if !self.blocked.as_slice().contains(&address.port()) {
            self.blocked.push(address.port()).expect("deployment listener");
        }
    }
    pub(crate) fn new(settings: &crate::settings::Engine) -> Self {
        let mut blocked = List::uncapped();
        for address in &settings.jig.hosts.listener {
            if let jig_protocol_config::Address::Addr(skein_io::Addr::Ip(a)) = address {
                blocked.push(a.port()).expect("counted listeners");
            }
        }
        if let jig_protocol_config::Address::Addr(skein_io::Addr::Ip(a)) = &settings.clients.listener {
            blocked.push(a.port()).expect("clients listener");
        }
        if let Some(hooks) = &settings.webhooks
            && let skein_io::Addr::Ip(a) = hooks.listener
        {
            blocked.push(a.port()).expect("webhooks listener");
        }
        for provider in &settings.jig.llm.providers {
            if let Some(sign_in) = &provider.sign_in
                && let Some(port) = sign_in.redirect_port
            {
                blocked.push(port).expect("counted redirects");
            }
        }
        if let Some(demo) = &settings.demo {
            for provider in &settings.jig.llm.providers {
                if provider.number == demo.provider
                    && let Some(name) = &provider.endpoint
                {
                    for endpoint in &settings.jig.llm.endpoints {
                        if endpoint.name == *name
                            && let jig_protocol_config::EndpointAddress::Socket(a) = &endpoint.address
                        {
                            blocked.push(a.port()).expect("demo listener");
                        }
                    }
                }
            }
        }
        let capacity = settings.limits.forge.forges;
        let mut files = Map::with_capacity(capacity);
        for (forge, file) in &settings.setups {
            files.insert(*forge, file.clone()).expect("world setup files");
        }
        Self {
            directories: settings.directories.clone(),
            provider: settings.jig.clients.provider.is_some(),
            owned_forgejo: settings.owned_forgejo.clone(),
            owned_runner: settings.owned_runner.clone(),
            blocked,
            choosing: Map::with_capacity(settings.limits.forge.applications),
            chosen: Map::with_capacity(capacity),
            files,
            registered: Map::with_capacity(capacity),
            records: Map::with_capacity(capacity),
            starts: Map::with_capacity(capacity),
            io: Queue::uncapped(),
        }
    }
}
impl crate::engine::Engine {
    pub fn take_place_io(&mut self) -> Option<PlaceIo> {
        match &mut self.places {
            Some(p) => p.io.pop(),
            None => None,
        }
    }
    /// One kernel choice. Collisions request another, eight failures answer busy.
    pub fn place_port(&mut self, request: Token, port: Option<u16>) {
        let Some(p) = &mut self.places else { return };
        let Some(choosing) = p.choosing.get_mut(&request) else { return };
        choosing.tries = choosing.tries.checked_add(1).expect("eight attempts");
        let port = match port {
            Some(port) if port != 0 && !p.blocked.as_slice().contains(&port) => Some(port),
            Some(_) | None => None,
        };
        if port.is_none() && choosing.tries < 8 {
            p.io.push(PlaceIo::Choose { request });
            return;
        }
        let choosing = p.choosing.remove(&request).expect("held choice");
        let place = match port {
            Some(port) => {
                match setup_file::chosen(setup_kind(choosing.kind), port, &choosing.login, p.provider, &p.directories) {
                    Ok(mut setup) => {
                        match &mut setup {
                            SetupFile::Owned(owned) => {
                                if let Some(forgejo) = &p.owned_forgejo {
                                    owned.forgejo.clone_from(forgejo);
                                    owned.fetch.forgejo = false;
                                }
                                if let Some(binary) = &p.owned_runner {
                                    owned.runner.as_mut().expect("stock runner").binary.clone_from(binary);
                                    owned.fetch.runner = false;
                                }
                            }
                            SetupFile::Fake(_) => {}
                        }
                        let remote = match &setup {
                            SetupFile::Owned(o) => o.remote.clone(),
                            SetupFile::Fake(f) => f.remote.clone(),
                        };
                        p.chosen.insert(port, setup).expect("admitted local forge");
                        Some(domain::Place {
                            api: domain::Address { host: bytes::copy_of(b"127.0.0.1"), port, tls: None },
                            git: remote,
                            web: None,
                        })
                    }
                    Err(_) => None,
                }
            }
            None => None,
        };
        let git_program = match choosing.kind {
            BoxKind::Owned => None,
            BoxKind::Fake => p.directories.join(jig_protocol_config::Directory::Data, b"demo/bin/git").ok(),
        };
        self.events.push(temper_engine_domain::Event::Forge(Box::new(domain::Event::PlaceChosen {
            request,
            place,
            git_program,
        })));
    }
    /// Complete the one immutable write or kept-file read for a forge.
    pub fn place_file(&mut self, forge: u16, file: Result<SetupFile, Box<[u8]>>) {
        let Some(p) = &mut self.places else { return };
        let Some(record) = p.records.remove(&forge) else { return };
        match file {
            Ok(file) if file.listen().port() == record.place.api.port => {
                p.files.insert(forge, file.clone()).expect("kept local setups");
                crate::setup::set_file(&mut self.setup, forge, file.clone());
                crate::forges::record(self, record);
                let start = self.places.as_mut().expect("configured places").starts.remove(&forge);
                if let Some((kind, request)) = start {
                    start_group(self, forge, kind, request);
                }
            }
            Ok(_) => failed(self, forge, bytes::copy_of(b"setup differs from the kept forge's port")),
            Err(why) => failed(self, forge, why),
        }
    }
}
pub(crate) fn choose(engine: &mut crate::engine::Engine, request: Token, kind: BoxKind, login: Box<[u8]>) {
    let Some(p) = &mut engine.places else { return };
    p.choosing.insert(request, Choosing { kind, login, tries: 0 }).expect("admitted place choices");
    p.io.push(PlaceIo::Choose { request });
}
/// Gate the setup host and box start on the immutable file's successful write/read.
pub(crate) fn record(engine: &mut crate::engine::Engine, record: &ForgeRecord) -> bool {
    let kind = match record.mode {
        domain::Mode::Owned => BoxKind::Owned,
        domain::Mode::Fake => BoxKind::Fake,
        domain::Mode::Managed | domain::Mode::Connected => return true,
    };
    let Some(p) = &mut engine.places else { return true };
    if let Some(file) = p.files.get(&record.number) {
        if !p.registered.contains_key(&record.number) {
            let file = file.clone();
            p.registered.insert(record.number, ()).expect("one client per forge");
            register(engine, record.number, &file);
        }
        return true;
    }
    if p.records.contains_key(&record.number) {
        return false;
    }
    p.records.insert(record.number, record.clone()).expect("kept forge files");
    match p.chosen.remove(&record.place.api.port) {
        Some(setup) => p.io.push(PlaceIo::Write {
            forge: record.number,
            setup: Box::new(setup),
            directories: p.directories.clone(),
        }),
        None => p.io.push(PlaceIo::Read {
            forge: record.number,
            kind: setup_kind(kind),
            directories: p.directories.clone(),
        }),
    }
    false
}
pub(crate) fn start_group(engine: &mut crate::engine::Engine, forge: u16, kind: BoxKind, request: Token) {
    if let Some(p) = &mut engine.places
        && !p.files.contains_key(&forge)
    {
        p.starts.insert(forge, (kind, request)).expect("one start per forge");
        return;
    }
    let mut out = Queue::with_capacity(3);
    engine.box_link.group(forge, kind, request, &mut out);
    crate::route::box_link::drain(engine, &mut out);
}
fn failed(engine: &mut crate::engine::Engine, forge: u16, why: Box<[u8]>) {
    engine.notes.push(crate::Note::StartupWarning { line: why });
    if let Some(p) = &mut engine.places
        && let Some((_, request)) = p.starts.remove(&forge)
    {
        crate::setup::lifecycle::set_up_refused(engine, forge, request, domain::SetUpRefusal::Busy);
    }
}

fn setup_kind(kind: BoxKind) -> temper_forge_setup::machine::BoxKind {
    match kind {
        BoxKind::Owned => temper_forge_setup::machine::BoxKind::Owned,
        BoxKind::Fake => temper_forge_setup::machine::BoxKind::Fake,
    }
}

pub(crate) fn forget(engine: &mut crate::engine::Engine, forge: u16) {
    if let Some(p) = &mut engine.places {
        if let Some(SetupFile::Owned(owned)) = p.files.remove(&forge)
            && let Some(id) = owned.sign_in
        {
            engine.clients.provider_client(
                skein_lib::ReplyTo::new(Token::new(u64::from(forge))),
                jig_protocol_engine_clients::ProviderClientChange::Remove { id },
            );
        }
        p.registered.remove(&forge);
        p.records.remove(&forge);
        p.starts.remove(&forge);
    }
}

fn register(engine: &mut crate::engine::Engine, forge: u16, file: &SetupFile) {
    if let SetupFile::Owned(owned) = file {
        engine.events.push(temper_engine_domain::Event::Forge(Box::new(domain::Event::SetupPerson {
            forge,
            login: owned.person.login.clone(),
        })));
        if let Some(id) = &owned.sign_in {
            let origin = match &owned.public {
                Some(public) => setup_file::origin(public),
                None => owned.remote.clone(),
            };
            let tail = b"/user/oauth2/temper/callback";
            let mut w = skein_lib::Writer::new(origin.len().checked_add(tail.len()).expect("bounded redirect"));
            w.put(&origin).expect("measured origin");
            w.put(tail).expect("measured callback");
            let redirect = w.finish();
            engine.clients.forge_sign_in.owned(forge, owned, &redirect);
            engine.clients.provider_client(
                skein_lib::ReplyTo::new(Token::new(u64::from(forge))),
                jig_protocol_engine_clients::ProviderClientChange::Add(skein_oauth_issuer::Client {
                    id: id.clone(),
                    secret: None,
                    redirect_uri: redirect,
                }),
            );
        }
    }
}
