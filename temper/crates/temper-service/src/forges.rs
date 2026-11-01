//! The deployment's forges at runtime (draft playground-reset.md, section 3; notes contracts T5).
use crate::{engine::Engine, forge::Endpoint, settings};
use alloc::boxed::Box;
use core::net::{IpAddr, SocketAddr};
use jig_protocol_config::Address;
use skein_lib::{List, Secret};
use temper_engine_domain_forge::{ApiTransport, ForgeRecord, Kind};

pub(crate) fn settings(record: &ForgeRecord) -> settings::Forge {
    let api = match ip(&record.api.host) {
        Some(ip) => Address::Addr(skein_io::Addr::Ip(SocketAddr::new(ip, record.api.port))),
        None => Address::Host { name: record.api.host.clone(), port: record.api.port },
    };
    settings::Forge {
        mode: record.mode,
        number: record.number,
        kind: match record.kind {
            Kind::Forgejo => settings::Kind::Forgejo,
            Kind::GitHub => settings::Kind::GitHub,
        },
        api,
        transport: match &record.api.transport {
            ApiTransport::Plaintext => settings::Transport::Plaintext,
            ApiTransport::Tls { server_name, trust } => {
                settings::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
            }
        },
        git: record.git.clone(),
        git_program: record.git_program.clone(),
        web: record.web.clone(),
        app_read_only: record.read_only,
        identity: u32::from(record.number),
    }
}
#[expect(clippy::disallowed_methods, reason = "the service protocol layer parses a record host as an IP literal")]
fn ip(host: &[u8]) -> Option<IpAddr> {
    core::str::from_utf8(host).ok()?.parse().ok()
}
pub(crate) fn identity(record: &ForgeRecord) -> settings::Identity {
    settings::Identity {
        number: u32::from(record.number),
        forge: record.number,
        token: Secret::new(Box::new([])),
        login: record.bot.login.clone(),
        author: settings::Author { name: record.bot.name.clone(), email: record.bot.email.clone() },
    }
}
pub(crate) fn register(engine: &mut Engine, record: ForgeRecord) {
    let number = record.number;
    let mut index = None;
    for (at, forge) in engine.workers.forges.iter().enumerate() {
        if forge.number == number {
            index = Some(u32::try_from(at).expect("forge index"));
            break;
        }
    }
    if let Some(at) = index {
        let previous = engine.records.get(&number).expect("registered forge");
        if previous.kind != record.kind || previous.api != record.api {
            engine.notes.push(crate::Note::ForgeRecordChanged { number });
            return;
        }
        *engine.workers.forges.get_mut(at).expect("registered forge") = settings(&record);
    } else {
        let configured = settings(&record);
        let endpoint = match (&configured.api, &record.api.transport) {
            (Address::Addr(skein_io::Addr::Ip(address)), ApiTransport::Plaintext) => Endpoint::Reachable(*address),
            _ => Endpoint::Held { host: record.api.host.clone() },
        };
        if !engine.forge.add(&configured, endpoint) {
            engine.notes.push(crate::Note::ForgeNotConnected {
                number,
                why: crate::NotConnected::Unreachable { host: record.api.host.clone() },
            });
            return;
        }
        engine.workers.forges.push(configured).expect("connection admits forge");
        engine.forge_records.rows.register(number);
    }
    if record.kind == Kind::GitHub {
        let mut identity = identity(&record);
        identity.login = Box::new([]);
        engine.values.tokens.add(number, &identity, engine.forge_records.bounds.installations);
    }
    crate::setup::register(engine, &record);
    engine.records.insert(number, record).expect("connection admits forge");
    ask(engine, number);
}
pub(crate) fn unregister(engine: &mut Engine, forge: u16) {
    cancel(engine, forge);
    crate::setup::unregister(engine, forge);
    engine.forge.remove(forge);
    let capacity = engine.workers.forges.capacity();
    let forges = core::mem::replace(&mut engine.workers.forges, List::with_capacity(capacity));
    for configured in forges.into_boxed() {
        if configured.number == forge {
            engine.values.identities.remove(configured.identity);
        } else {
            engine.workers.forges.push(configured).expect("remaining forges");
        }
    }
    engine.records.remove(&forge);
    engine.forge_records.rows.unregister(forge);
    engine.values.identities.remove(u32::from(forge));
    engine.values.tokens.remove(forge);
    crate::setup::lifecycle::remove(engine, forge);
}

fn ask(engine: &mut Engine, number: u16) {
    if engine.forge.unreachable(number).is_none() || engine.resolving.contains_key(&number) {
        return;
    }
    let record = engine.records.get(&number).expect("registered resolve");
    let tls = match &record.api.transport {
        ApiTransport::Plaintext => None,
        ApiTransport::Tls { server_name, trust } => {
            Some(crate::Tls { server_name: server_name.clone(), trust: trust.clone() })
        }
    };
    engine.resolves.push(crate::Resolve { forge: number, host: record.api.host.clone(), port: record.api.port, tls });
    engine.resolving.insert(number, ()).expect("registered resolve");
}
fn cancel(engine: &mut Engine, number: u16) {
    engine.resolving.remove(&number);
    for _ in 0..engine.resolves.len() {
        let resolve = engine.resolves.pop().expect("queued resolve");
        if resolve.forge != number {
            engine.resolves.push(resolve);
        }
    }
}
pub(crate) fn resolved(engine: &mut Engine, number: u16, answer: Result<crate::ForgeRuntime, Box<[u8]>>) {
    if !engine.records.contains_key(&number)
        || engine.forge.unreachable(number).is_none()
        || !engine.resolving.contains_key(&number)
    {
        return;
    }
    cancel(engine, number);
    let record = engine.records.get(&number).expect("registered resolve").clone();
    if let Ok(runtime) = answer
        && runtime.number == number
        && engine.forge.resolve(&settings(&record), runtime)
    {
        crate::setup::resolved(engine, &record);
        connect_after_load(engine, number);
        return;
    }
    engine.notes.push(crate::Note::ForgeNotConnected {
        number,
        why: crate::NotConnected::Unreachable { host: record.api.host },
    });
}
fn connect_after_load(engine: &mut Engine, number: u16) {
    for (index, forge) in engine.workers.forges.iter().enumerate() {
        if forge.number == number {
            if engine.forge_records.passed(u32::try_from(index).expect("forge index")) {
                crate::forge::records::connect(engine, number);
            }
            return;
        }
    }
}
