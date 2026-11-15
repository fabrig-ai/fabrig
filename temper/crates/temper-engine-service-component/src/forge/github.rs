//! GitHub names resolved on demand while serving (protocol/forge/README.md, 2.6).
use super::HELD_ADDRESS;
use crate::{engine::Engine, forges};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Api {
    Pending,
    Ready,
    Failed,
}

#[derive(Debug)]
pub(crate) struct Waiting {
    pub(crate) adopted: Option<skein_forge_connection::Access>,
    pub(crate) plans: Queue<Plan>,
}
#[derive(Debug)]
pub(crate) enum Plan {
    Requested { run: Token, plan: skein_forge_setup::host::Plan },
    Running { run: Token, plan: skein_forge_setup::host::Plan },
}
impl Waiting {
    pub(crate) fn new() -> Self {
        Self { adopted: None, plans: Queue::uncapped() }
    }
}

pub(crate) fn unresolved(engine: &Engine) -> Option<(Box<[u8]>, u16)> {
    if engine.forge.serving.is_none()
        || engine.end.is_some()
        || engine.drain.is_some()
        || engine.forge.github_api == Api::Failed
    {
        return None;
    }
    if let Some((number, _)) = engine.forge.github_waiting.iter().next() {
        let configured = engine.workers.forges.get(*number)?;
        if let jig_protocol_config::Address::Host { name, port } = &configured.api {
            return Some((name.clone(), *port));
        }
    }
    let github = engine.forge.github.as_ref()?;
    if github.address == HELD_ADDRESS {
        // A previously resolved kept name can supply the API address without another lookup.
        let mut known = false;
        for (name, _) in &engine.forge.names {
            if forges::github_host(engine, name) {
                known = true;
                break;
            }
        }
        if !known {
            return None;
        }
    }
    let mut wanted = false;
    for forge in engine.workers.forges.iter() {
        if forge.kind == crate::settings::Kind::GitHub && forge.log_origins.is_none() {
            wanted = true;
            break;
        }
    }
    if !wanted {
        return None;
    }
    let entry = github.log_origins.get(origin_index(engine)?)?;
    Some((entry.origin.host.clone(), entry.origin.port))
}

pub(crate) fn resolved(engine: &mut Engine, name: &[u8], address: Option<core::net::SocketAddr>) {
    let Some((wanted, _)) = unresolved(engine) else { return };
    if wanted.as_ref() != name {
        return;
    }
    if !engine.forge.github_waiting.is_empty() {
        api(engine, name, address);
        return;
    }
    let index = origin_index(engine).expect("wanted origin");
    let github = engine.forge.github.as_mut().expect("wanted GitHub origin");
    let entry = github.log_origins.get_mut(index).expect("wanted origin");
    if let Some(address) = address {
        entry.address = skein_io::Addr::Ip(address);
    } else {
        engine.notes.push(crate::Note::GitHubLogOriginUnresolved { host: entry.origin.host.clone() });
    }
    engine.forge.github_origins_tried = index.checked_add(1).expect("bounded origins");
    if engine.forge.github_origins_tried == github.log_origins.len() {
        let mut any = false;
        for entry in &github.log_origins {
            if entry.address != skein_io::Addr::Ip(HELD_ADDRESS) {
                any = true;
            }
        }
        if !any {
            engine.notes.push(crate::Note::GitHubLogOriginsUnresolved);
        }
    }
}

#[expect(clippy::manual_find, reason = "step code uses explicit bounded loops, not closures")]
fn origin_index(engine: &Engine) -> Option<u32> {
    let github = engine.forge.github.as_ref()?;
    for index in engine.forge.github_origins_tried..github.log_origins.len() {
        if github.log_origins.get(index).expect("origin index").address == skein_io::Addr::Ip(HELD_ADDRESS) {
            return Some(index);
        }
    }
    None
}

fn api(engine: &mut Engine, name: &[u8], address: Option<core::net::SocketAddr>) {
    if let Some(address) = address {
        engine.forge.github_api = Api::Ready;
        engine.forge.github.as_mut().expect("GitHub API runtime").address = address;
        engine.forge.names.insert(Box::from(name), address.ip()).expect("kept host capacity");
    } else {
        engine.forge.github_api = Api::Failed;
    }
    for _ in 0..engine.forge.github_waiting.len() {
        let number = *engine.forge.github_waiting.iter().next().expect("waiting forge").0;
        let mut waiting = engine.forge.github_waiting.remove(&number).expect("waiting forge");
        if let Some(address) = address {
            let configured = engine.workers.forges.get(number).expect("kept waiting forge");
            let port = match &configured.api {
                jig_protocol_config::Address::Host { port, .. } => *port,
                jig_protocol_config::Address::Addr(_) => unreachable!("waiting forge names GitHub's host"),
            };
            forges::resolved(engine, number, core::net::SocketAddr::new(address.ip(), port), waiting.adopted);
        } else {
            engine.forge.held.insert(number, Box::from(name)).expect("kept forge capacity");
            engine.notes.push(crate::Note::ForgeNotConnected {
                number,
                why: crate::NotConnected::Unreachable { host: Box::from(name) },
            });
        }
        for _ in 0..waiting.plans.len() {
            let plan = waiting.plans.pop().expect("waiting plan count");
            match plan {
                Plan::Requested { run, plan } => crate::setup::request_plan(engine, run, plan),
                Plan::Running { run, plan } => crate::setup::run_plan(engine, run, plan),
            }
        }
    }
}
