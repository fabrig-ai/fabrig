//! The forges this service serves, kept while it runs (temper/docs/design/protocol/forge.md, 2.2).
use crate::settings::Forge;
use alloc::boxed::Box;
use skein_lib::{List, Map};

#[derive(Debug)]
pub struct Forges {
    forges: Map<u16, Forge>,
    // Record bots distinguish a login rewrite from an author change.
    bots: Map<u16, temper_engine_domain_forge::Bot>,
}

impl Forges {
    #[must_use]
    pub const fn with_capacity(capacity: u32) -> Self {
        Self { forges: Map::with_capacity(capacity), bots: Map::with_capacity(capacity) }
    }

    #[must_use]
    pub fn get(&self, forge: u16) -> Option<&Forge> {
        self.forges.get(&forge)
    }

    pub(crate) fn bot_login(&self, forge: u16) -> Option<&[u8]> {
        match self.bots.get(&forge) {
            Some(bot) => Some(&bot.login),
            None => None,
        }
    }

    #[must_use]
    pub fn get_mut(&mut self, forge: u16) -> Option<&mut Forge> {
        self.forges.get_mut(&forge)
    }

    /// The forges in number order.
    pub fn iter(&self) -> impl Iterator<Item = &Forge> {
        Iter { entries: self.forges.iter() }
    }

    /// The record loader's positional cursor, in number order.
    #[must_use]
    pub fn nth(&self, index: usize) -> Option<&Forge> {
        self.iter().nth(index)
    }

    #[must_use]
    pub fn numbers(&self) -> List<u16> {
        let mut numbers = List::with_capacity(self.len());
        for (number, _) in &self.forges {
            numbers.push(*number).expect("one number per forge");
        }
        numbers
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.forges.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.forges.is_empty()
    }

    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.forges.capacity()
    }

    /// Refuse a full table or a number already present, preserving its forge.
    pub fn insert(&mut self, forge: Forge) -> bool {
        if self.forges.contains_key(&forge.number) {
            return false;
        }
        self.forges.insert(forge.number, forge).is_ok()
    }

    pub fn remove(&mut self, forge: u16) -> Option<Forge> {
        self.bots.remove(&forge);
        self.forges.remove(&forge)
    }
}

struct Iter<'a> {
    entries: alloc::collections::btree_map::Iter<'a, u16, Forge>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Forge;

    fn next(&mut self) -> Option<Self::Item> {
        match self.entries.next() {
            Some((_, forge)) => Some(forge),
            None => None,
        }
    }
}

/// Record-driven runtime membership (temper/docs/design/protocol/forge.md, 2.2).
pub(crate) fn record(engine: &mut crate::engine::Engine, record: temper_engine_domain_forge::ForgeRecord) {
    if !engine.store.component.mark_durable() {
        engine.forge_records.pending.push(record);
        return;
    }
    let kept = record.clone();
    let login_only = match engine.workers.forges.bots.get(&record.number) {
        Some(bot) => bot.name == record.bot.name && bot.email == record.bot.email,
        None => false,
    };
    let (forge, identity) = record_settings(engine, record);
    if let Some(held) = engine.workers.forges.get(forge.number)
        && held.kind == forge.kind
        && held.mode == forge.mode
        && held.api == forge.api
        && held.transport == forge.transport
        && held.git == forge.git
        && held.web == forge.web
        && held.app_read_only == forge.app_read_only
        && held.git_program == forge.git_program
        && (engine.values.tokens.same_identity(&identity) || login_only)
    {
        if !engine.values.tokens.same_identity(&identity) {
            engine.values.tokens.keep(&identity, engine.forge.env.limits.installations);
        }
        engine.workers.forges.bots.insert(kept.number, kept.bot.clone()).expect("kept forge capacity");
        if engine.setup.serves(forge.number) {
            crate::setup::keep(engine, &kept);
        }
        return;
    }
    // A kept record replaces its previous runtime endpoint.
    engine.forge.file_access.remove(&kept.number);
    engine.forge.startup_access.remove(&kept.number);
    if engine.workers.forges.remove(kept.number).is_some() {
        engine.forge.down(skein_forge_connection::Request::Remove { forge: kept.number });
    }
    if keep(engine, forge) {
        engine.workers.forges.bots.insert(kept.number, kept.bot.clone()).expect("kept forge capacity");
        engine.values.tokens.keep(&identity, engine.forge.env.limits.installations);
        crate::setup::keep(engine, &kept);
        if engine.forge_records.done() {
            crate::forge::records::connect(engine, kept.number);
        }
    }
}

fn record_settings(
    engine: &crate::engine::Engine,
    record: temper_engine_domain_forge::ForgeRecord,
) -> (Forge, crate::settings::Identity) {
    let api = record_address(record.place.api.host, record.place.api.port);
    let (login, author) = match record.kind {
        temper_engine_domain_forge::Kind::Forgejo => {
            (record.bot.login, crate::settings::Author { name: record.bot.name, email: record.bot.email })
        }
        temper_engine_domain_forge::Kind::GitHub => {
            (Box::default(), crate::settings::Author { name: Box::new([]), email: Box::new([]) })
        }
    };
    let identity = crate::settings::Identity {
        number: u32::from(record.number),
        forge: record.number,
        token: skein_lib::Secret::new(Box::new([])),
        login,
        author,
    };
    let forge = Forge {
        log_origins: match engine.workers.forges.get(record.number) {
            Some(forge) => forge.log_origins.clone(),
            None => None,
        },
        number: record.number,
        kind: match record.kind {
            temper_engine_domain_forge::Kind::Forgejo => crate::settings::Kind::Forgejo,
            temper_engine_domain_forge::Kind::GitHub => crate::settings::Kind::GitHub,
        },
        mode: record.mode,
        api,
        transport: match record.place.api.tls {
            Some(server_name) => crate::settings::Transport::Tls { server_name, trust: None },
            None => crate::settings::Transport::Plaintext,
        },
        git: record.place.git,
        web: record.place.web,
        git_program: match record.git_program {
            Some(program) => Some(program),
            None => engine.setup.git_program(record.mode),
        },
        app_read_only: record.read_only,
        identity: identity.number,
        user: 0,
        api_token: None,
        webhook_secret: None,
    };
    (forge, identity)
}

#[expect(clippy::disallowed_methods, reason = "the service protocol layer parses the record's API host")]
fn record_address(host: Box<[u8]>, port: u16) -> jig_protocol_config::Address {
    let ip = match core::str::from_utf8(&host) {
        Ok(host) => host.parse::<core::net::IpAddr>().ok(),
        Err(_) => None,
    };
    match ip {
        Some(ip) => jig_protocol_config::Address::Addr(skein_io::Addr::Ip(core::net::SocketAddr::new(ip, port))),
        None => jig_protocol_config::Address::Host { name: host, port },
    }
}

/// A kept record admits its endpoint (temper/docs/design/protocol/forge.md, 2.2).
pub(crate) fn keep(engine: &mut crate::engine::Engine, forge: Forge) -> bool {
    let number = forge.number;
    let host = match &forge.api {
        jig_protocol_config::Address::Host { name, .. } => name.clone(),
        jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => {
            skein_forge::host::host(core::net::SocketAddr::new(address.ip(), 80), None).expect("IP host")
        }
        jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => Box::new([]),
    };
    if engine.workers.forges.get(number).is_some() {
        return true;
    }
    let endpoint = match engine.forge.endpoints.remove(&number) {
        Some(endpoint) => Some(endpoint),
        None => endpoint(engine, &forge),
    };
    let Some(mut endpoint) = endpoint else {
        engine.notes.push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
        return false;
    };
    let held = endpoint.address == skein_io::Addr::Ip(crate::forge::HELD_ADDRESS);
    if !engine.workers.forges.insert(forge) {
        engine.notes.push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
        return false;
    }
    if let skein_io::Addr::Ip(address) = endpoint.address {
        engine.setup.resolved(crate::ForgeRuntime {
            log_origins: endpoint.log_origins.clone(),
            number,
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
        });
    }
    endpoint.access = None;
    engine.forge.down(skein_forge_connection::Request::Add(endpoint));
    if engine.forge.component.start(number).is_none() {
        engine.workers.forges.remove(number);
        engine.notes.push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
        return false;
    }

    if held {
        engine.forge.held.insert(number, host).expect("kept forge capacity");
        if engine.forge.serving.is_none() {
            engine.forge.resolving.insert(number, ()).expect("kept forge capacity");
        }
    } else {
        engine.forge.held.remove(&number);
    }
    true
}

#[expect(clippy::disallowed_methods, reason = "the service protocol layer validates the record's TLS name")]
pub(crate) fn endpoint(
    engine: &crate::engine::Engine,
    forge: &Forge,
) -> Option<skein_forge_connection::settings::Forge> {
    use skein_forge_connection::settings::{Kind, Transport};
    let (address, name) = match &forge.api {
        jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => (*address, None),
        jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => return None,
        jig_protocol_config::Address::Host { name, port } => {
            let mut address = match engine.forge.names.get(name) {
                Some(ip) => core::net::SocketAddr::new(*ip, *port),
                None => crate::forge::HELD_ADDRESS,
            };
            if let Some(github) = &engine.forge.github {
                let github_name = match &github.transport {
                    Transport::Tls { server_name, .. } => match server_name.server() {
                        skein_tls::Server::Dns(name) => Some(name.as_bytes()),
                        skein_tls::Server::Ip(_) => None,
                    },
                    Transport::Plaintext { name } => name.as_deref(),
                };
                if github_name == Some(name.as_ref()) && github.address != crate::forge::HELD_ADDRESS {
                    address = core::net::SocketAddr::new(github.address.ip(), *port);
                }
            }
            (address, Some(name.clone()))
        }
    };
    let transport = match &forge.transport {
        crate::settings::Transport::Plaintext => Transport::Plaintext { name },
        crate::settings::Transport::Tls { server_name, .. } => Transport::Tls {
            server_name: skein_tls::Name::new(core::str::from_utf8(server_name).ok()?)?,
            trust: engine.forge.roots.as_ref()?.clone(),
        },
    };
    Some(skein_forge_connection::settings::Forge {
        number: forge.number,
        kind: match forge.kind {
            crate::settings::Kind::Forgejo => Kind::Forgejo,
            crate::settings::Kind::GitHub => Kind::GitHub,
        },
        address: skein_io::Addr::Ip(address),
        transport,
        access: None,
        log_origins: log_origins(engine, forge)?,
    })
}

fn log_origins(
    engine: &crate::engine::Engine,
    forge: &Forge,
) -> Option<List<skein_forge_connection::settings::LogOrigin>> {
    let origins = match &forge.log_origins {
        Some(origins) => origins.clone(),
        None => match forge.kind {
            crate::settings::Kind::GitHub => match &engine.forge.github {
                Some(github) => github.log_origins.clone(),
                None => List::with_capacity(0),
            },
            crate::settings::Kind::Forgejo => List::with_capacity(0),
        },
    };
    if origins.len() > engine.forge.log_origin_limit {
        return None;
    }
    Some(origins)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forge(number: u16) -> Forge {
        let mut forges = crate::tests::fixtures::forges().into_iter();
        let mut forge = forges.next().expect("fixture forge");
        forge.number = number;
        forge
    }

    #[test]
    fn insert_refuses_a_taken_number_and_a_full_table() {
        let mut forges = Forges::with_capacity(2);
        assert_eq!(forges.capacity(), 2);
        assert!(forges.insert(forge(7)));
        let mut duplicate = forge(7);
        duplicate.user = 99;
        assert!(!forges.insert(duplicate));
        assert_eq!(forges.get(7).unwrap().user, 42);
        assert!(forges.insert(forge(3)));
        assert!(!forges.insert(forge(9)));
        assert_eq!(forges.len(), 2);
        assert!(forges.get(9).is_none());
        assert_eq!(forges.remove(7).unwrap().number, 7);
        assert!(forges.remove(7).is_none());
        assert!(forges.insert(forge(9)));
        forges.get_mut(9).unwrap().user = 17;
        assert_eq!(forges.get(9).unwrap().user, 17);
    }

    #[test]
    fn iteration_is_in_number_order() {
        let mut forges = Forges::with_capacity(4);
        for number in [9, 3, 7] {
            assert!(forges.insert(forge(number)));
        }
        let mut iter = forges.iter();
        assert_eq!(iter.next().unwrap().number, 3);
        assert_eq!(iter.next().unwrap().number, 7);
        assert_eq!(iter.next().unwrap().number, 9);
        assert!(iter.next().is_none());
        assert_eq!(forges.numbers().as_slice(), &[3, 7, 9]);
        assert_eq!(forges.nth(1).unwrap().number, 7);
        assert!(forges.nth(3).is_none());
    }
}
