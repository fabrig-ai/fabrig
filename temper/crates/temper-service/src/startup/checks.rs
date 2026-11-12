//! The standalone start's paired checks (protocol/services.md, 6).
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use temper_engine_domain as domain;
use jig_protocol_config::{self as config, Path};
use skein_io::Addr;
use temper_protocol_worker_wire as payload;
use temper_service_core::agent::Declaration;
pub use temper_engine_service::startup::checks::*;
pub(super) use temper_service_core::startup::{against, at, call_answer, own, short};
use super::worker::checks::bounded;
pub use super::worker::worker;
fn in_process(address: &config::Address) -> bool {
    match address {
        config::Address::Addr(Addr::InProcess(name)) => name.as_bytes() == b"workers",
        config::Address::Addr(Addr::Ip(_)) | config::Address::Host { .. } => false,
    }
}
pub(super) fn loopback(address: &config::Address) -> bool {
    match address {
        config::Address::Addr(Addr::Ip(socket)) => socket.ip().is_loopback(),
        config::Address::Addr(Addr::InProcess(_)) | config::Address::Host { .. } => false,
    }
}
fn engine_alone(engine: &config::Engine, workers: &[config::Worker], problems: &mut Problems) {
    let hosts = Path::root().member(b"engine").member(b"hosts");
    if !workers.is_empty() {
        problems.push(Path::root().member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone));
    }
    if engine.hosts.listener.len() != 1 {
        problems.push(hosts.member(b"listener"), What::Half(crate::startup::EngineWhat::EngineAlone));
    }
    for (index, address) in engine.hosts.listener.iter().enumerate() {
        if !loopback(address) {
            problems.push(at(index, hosts.member(b"listener")), What::Half(crate::startup::EngineWhat::EngineAlone));
        }
    }
    if engine.hosts.workers.is_empty() {
        problems.push(hosts.member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone));
    }
    for (index, worker) in engine.hosts.workers.iter().enumerate() {
        for previous in engine.hosts.workers.iter().take(index) {
            if previous.digest == worker.digest {
                problems.push(
                    at(index, hosts.member(b"workers")).member(b"digest"),
                    What::Half(crate::startup::EngineWhat::EngineAlone),
                );
                break;
            }
        }
    }
}
pub fn standalone(
    engine: &config::Engine,
    clients: &super::application::Clients,
    workers: &[config::Worker],
    problems: &mut Problems,
) {
    let root = Path::root();
    let alone = match engine.hosts.listener.first() {
        Some(config::Address::Addr(Addr::Ip(_))) => true,
        Some(config::Address::Addr(Addr::InProcess(_)) | config::Address::Host { .. }) | None => false,
    };
    if alone {
        engine_alone(engine, workers, problems);
    } else {
        if workers.len() != 1 {
            problems.push(root.member(b"workers"), What::Standalone);
        }
        if engine.hosts.listener.len() != 1 || !in_process(engine.hosts.listener.first().expect("parsed listeners")) {
            problems.push(root.member(b"engine").member(b"hosts").member(b"listener"), What::Standalone);
        }
        let hosts_match = match (engine.hosts.workers.first(), workers.first()) {
            (Some(host), Some(worker)) => host.name == worker.name,
            (None, _) | (_, None) => false,
        };
        if engine.hosts.workers.len() != 1 || !hosts_match {
            problems.push(root.member(b"engine").member(b"hosts").member(b"workers"), What::Standalone);
        }
    }
    match clients.listener {
        config::Address::Addr(Addr::Ip(socket)) if socket.ip().is_loopback() => {}
        config::Address::Addr(Addr::Ip(_) | Addr::InProcess(_)) | config::Address::Host { .. } => {
            problems.push(
                root.member(b"application").member(b"clients").member(b"listener"),
                What::Half(crate::startup::EngineWhat::Loopback),
            );
        }
    }
    if engine.connectors.as_ref() != [1] {
        problems.push(root.member(b"engine").member(b"connectors"), What::Half(crate::startup::EngineWhat::Connectors));
    }
    if alone {
        return;
    }
    for (index, worker) in workers.iter().enumerate() {
        let path = at(index, root.member(b"workers"));
        if worker.slots > engine.limits.core.fleet.slots {
            problems.push(
                path.member(b"slots"),
                What::Against { other: b"engine.limits.overrides.fleet.slots, at least the slots" },
            );
        }
        if !in_process(&worker.engine) {
            problems.push(path.member(b"engine"), What::Standalone);
        }
        let command = match (worker.agent.arguments.first(), worker.agent.arguments.get(1)) {
            (Some(first), Some(second)) => first.as_ref() == b"internal" && second.as_ref() == b"agent",
            _ => false,
        };
        let old_command = match worker.agent.arguments.first() {
            Some(first) => first.as_ref() == b"agent",
            None => false,
        };
        if old_command {
            problems.push(
                path.member(b"agent").member(b"arguments"),
                What::Half(crate::startup::EngineWhat::OldAgentCommand),
            );
        } else if command {
            let absolute = match worker.agent.arguments.get(2) {
                Some(value) => value.starts_with(b"/"),
                None => false,
            };
            if worker.agent.arguments.len() != 3 || !absolute {
                problems.push(path.member(b"agent").member(b"arguments"), What::Absolute);
            }
        } else {
            for (index, value) in worker.agent.arguments.iter().enumerate() {
                if !value.starts_with(b"/") {
                    problems.push(at(index, path.member(b"agent").member(b"arguments")), What::Absolute);
                }
            }
        }
        for (name, value) in
            [(b"program".as_slice(), &worker.agent.program), (b"configuration", &worker.agent.configuration)]
        {
            if !value.starts_with(b"/") {
                problems.push(path.member(b"agent").member(name), What::Absolute);
            }
        }
    }
}
pub fn credentials(engine: &config::Engine, workers: &[config::Worker], forges: u32, problems: &mut Problems) {
    let mut bound = engine.limits.core.accounts.credentials;
    for worker in workers {
        bound = bound.min(worker.limits.core.accounts.credentials);
    }
    if forges > bound {
        problems.push(own(b"forge.forges"), What::Half(crate::startup::EngineWhat::Credentials));
    }
}
pub fn channel(
    engine: Option<&payload::Bounds>,
    worker: Option<&payload::Bounds>,
    machine: Option<&temper_forge_setup::machine::Machine>,
    limits: &limits::Limits,
    problems: &mut Problems,
) {
    if engine.is_some()
        && let Some(machine) = machine
    {
        names(machine_remote(machine, limits.forge.name_bytes), limits, problems);
    }
    // Both sides carry checkout repositories; forge watches need no directories.
    let repository_shape = match (engine, worker) {
        (Some(engine), Some(worker)) => engine.repositories == worker.repositories,
        _ => true,
    };
    let engine = match engine {
        Some(engine) => bounded(engine, true, problems),
        None => None,
    };
    let worker = match worker {
        Some(worker) => bounded(worker, false, problems),
        None => None,
    };
    
    let mut sizes = [None, None];
    for (index, side) in [engine, worker].iter().enumerate() {
        if let Some(side) = side {
            
            {
                let prefix: &[u8] = if index == 0 { b"engine" } else { b"workers" };
                *sizes.get_mut(index).expect("two sides") = super::worker::checks::largest(side, prefix, problems);
            }
            
            
        }
    }
    if let [Some(engine), Some(worker)] = sizes
        && (!repository_shape || engine.items > worker.items)
    {
        against(problems, b"checkout.repositories", b"the worker's checkout capacity and checkout.name_bytes");
    }
}
pub fn grants(engine: &config::Engine, worker: &config::Limits, agent: &settings::Agent, problems: &mut Problems) {
    let credential = temper_service_core::values::token_credential_bytes(worker.token);
    if crate::values::credential_bytes(engine) > credential {
        problems.push(worker_limit(b"accounts.token"), What::Against { other: b"engine.limits accounts.token" });
    }
    let bodies = &agent.limits.bodies;
    if u32::try_from(temper_service_core::values::SOURCE_NAME_BYTES).expect("bounded source name") > bodies.program_value_name {
        problems.push(
            at(0, Path::root().member(b"workers")).member(b"agent").member(b"configuration"),
            What::Against { other: b"the agent's program_value_name" },
        );
    }
    if credential > bodies.credential_value_bytes {
        problems.push(worker_limit(b"accounts.token"), What::Against { other: b"the agent's credential_value_bytes" });
    }
}
pub(super) fn worker_limit(member: &[u8]) -> Path {
    at(0, Path::root().member(b"workers")).member(b"limits").member(b"overrides").member(member)
}
pub fn turns(
    engine: Option<&config::Limits>,
    workers: &[config::Worker],
    own_limits: &limits::Limits,
    agent: Option<&Declaration>,
    problems: &mut Problems,
) {
    let engine_path = Path::root().member(b"engine").member(b"limits").member(b"overrides");
    if let Some(engine) = engine {
        forge_result(own_limits.forge.client.op_bytes, engine.core.tasks.result_bytes, problems);
    }
    if let Some(agent) = agent {
        let turn = u64::from(agent.turn_body);
        if let Some(engine) = engine {
            short(
                problems,
                engine_path.member(b"hub.outcome_bytes"),
                engine.hub.outcome_bytes,
                b"the agent's outcome (agent.json profile.declared.tool_payload)",
                Some(agent.outcome_bytes),
            );
            short(
                problems,
                engine_path.member(b"hub.turn_bytes"),
                engine.hub.turn_bytes,
                b"the agent's largest turn",
                Some(turn),
            );
            short(
                problems,
                engine_path.member(b"core.resume_bytes"),
                u64::from(engine.core.resume_bytes),
                b"the agent's largest turn with its length",
                turn.checked_add(4),
            );
            let loads = &own_limits.root.loads;
            short(
                problems,
                own(b"root.loads.transcript_bytes"),
                u64::from(loads.transcript_bytes),
                b"the agent's largest turn",
                Some(turn),
            );
            short(
                problems,
                own(b"root.loads.bytes"),
                u64::from(loads.bytes),
                b"the agent's largest turn with a loaded row",
                turn.checked_add(u64::try_from(size_of::<domain::Record>()).expect("record slot")),
            );
        }
        for (index, worker) in workers.iter().enumerate() {
            let path = at(index, Path::root().member(b"workers")).member(b"limits").member(b"overrides");
            short(
                problems,
                path.member(b"hub.outcome_bytes"),
                worker.limits.hub.outcome_bytes,
                b"the agent's outcome (agent.json profile.declared.tool_payload)",
                Some(agent.outcome_bytes),
            );
            short(
                problems,
                path.member(b"hub.turn_bytes"),
                worker.limits.hub.turn_bytes,
                b"the agent's largest turn",
                Some(turn),
            );
        }
    }
    if let Some(engine) = engine {
        let window = resume_window(engine.core.resume_bytes);
        short(
            problems,
            engine_path.member(b"hub.transcript_bytes"),
            engine.hub.transcript_bytes,
            b"the engine's resume window",
            window,
        );
        for (index, worker) in workers.iter().enumerate() {
            let path = at(index, Path::root().member(b"workers")).member(b"limits").member(b"overrides");
            short(
                problems,
                path.member(b"hub.transcript_bytes"),
                worker.limits.hub.transcript_bytes,
                b"the engine's resume window",
                window,
            );
        }
    }
}
fn resume_window(bytes: u32) -> Option<u64> {
    let bytes = u64::from(bytes);
    let carrier = u64::try_from(size_of::<Box<[u8]>>()).ok()?;
    bytes.checked_add(carrier.checked_sub(4)?.checked_mul(bytes / 4)?)
}
fn forge_result(bytes: u32, result: u32, problems: &mut Problems) {
    short(problems, own(b"forge.client.op_bytes"), u64::from(bytes), b"tasks.result_bytes", Some(u64::from(result)));
}
pub(super) fn sockets(
    engine: &config::Engine,
    worker: &config::Worker,
    git: &limits::GitComponent,
    members: &super::application::Members,
    problems: &mut Problems,
) {
    match (socket_count(engine, worker, git, members), process_io(&engine.limits.io, Some(&worker.limits.io))) {
        (Some(bytes), Some(io)) if bytes <= u64::from(io.sockets) => {}
        (Some(bytes), Some(io)) => problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"io").member(b"sockets"),
            What::Past { bytes, bound: u64::from(io.sockets) },
        ),
        (None, _) | (_, None) => problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"io").member(b"sockets"),
            What::Jig(config::What::Bound),
        ),
    }
}
fn socket_count(
    engine: &config::Engine,
    worker: &config::Worker,
    git: &limits::GitComponent,
    members: &super::application::Members,
) -> Option<u64> {
    let mut required = u64::from(limits::server(&engine.limits).connections).checked_add(1)?.checked_add(2)?;
    if members.webhooks.is_some() {
        required = required.checked_add(1)?.checked_add(u64::from(members.limits.forge_component.hooks))?;
    }
    if members.sign_in.is_some() {
        required = required.checked_add(u64::from(members.limits.sign_in.fetches))?;
    }
    let mut sign_in = false;
    for provider in &engine.llm.providers {
        sign_in |= provider.sign_in.is_some();
    }
    if sign_in {
        required = required
            .checked_add(u64::from(engine.limits.driver.accounts))?
            .checked_add(u64::from(engine.limits.driver.exchanges))?
            .checked_add(u64::from(engine.limits.driver.listeners))?;
    }
    required
        .checked_add(1)?
        .checked_add(u64::from(worker.slots).checked_mul(4)?)?
        .checked_add(u64::from(git.operations).checked_mul(4)?)
}
pub(crate) fn process_io(engine: &skein_io::Limits, worker: Option<&skein_io::Limits>) -> Option<skein_io::Limits> {
    let mut io = *engine;
    if let Some(worker) = worker {
        io.names = io.names.checked_add(worker.names)?;
        io.intake = io.intake.max(worker.intake);
        io.receive = io.receive.max(worker.receive);
        io.output = io.output.max(worker.output);
        io.sends = io.sends.max(worker.sends);
    }
    io.sockets = limits::SOCKETS;
    Some(io)
}
pub(super) fn raw_worker(document: &skein_json::Document, at: u32, path: &Path, problems: &mut Problems) {
    if let Ok(mut worker) = config::read::Object::new(document, at)
        && let Some(agent_at) = worker.member(document, b"agent")
        && let Ok(mut agent) = config::read::Object::new(document, agent_at)
        && let Some(at) = agent.member(document, b"configuration")
        && let Ok(value) = config::read::text(document, at, 4096)
        && !value.starts_with(b"/")
    {
        problems.push(path.member(b"agent").member(b"configuration"), What::Absolute);
    }
}
pub(super) fn names(remote: Option<u64>, limits: &limits::Limits, problems: &mut Problems) {
    if remote.unwrap_or(u64::MAX) > u64::from(limits.checkout.name_bytes) {
        against(problems, b"checkout.name_bytes", b"the longest remote");
    }
    if limits.forge.name_bytes > limits.checkout.name_bytes {
        against(problems, b"checkout.name_bytes", b"forge.name_bytes");
    }
}

pub(super) fn port(address: &config::Address) -> Option<u16> {
    match address {
        config::Address::Addr(address) => Some(address.socket()?.port()),
        config::Address::Host { port, .. } => Some(*port),
    }
}
pub fn limits(limits: &limits::Limits, git: &limits::GitComponent, problems: &mut Problems) {
    super::worker::git_limits(&limits::worker(limits), git, problems);
    temper_engine_service::startup::checks::limits(limits, git, problems);
}
pub fn webhooks(members: &super::application::Members, problems: &mut Problems) { webhook_parts(&members.clients, members.webhooks.as_ref(), problems); }
