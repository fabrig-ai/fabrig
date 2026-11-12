//! Checks across standalone's parts before any service is made.
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_link::Problem;
use jig_protocol_config::{self as config, Path, Segment};
use skein_io::Addr;
use skein_lib::{Decimal, Duration, List, Map, Writer};
use temper_engine_domain as domain;
use temper_protocol_worker_wire as payload;

pub(super) use temper_service_core::startup::{against, at, call_answer, own, short};
fn charter_path(engine: &config::Engine, index: usize) -> Path {
    let number = Decimal::of(u64::from(engine.charters.get(index).expect("checked charter index").number));
    let mut name = Writer::new(8_usize.checked_add(number.as_bytes().len()).expect("setting name fits"));
    name.put(b"charter ").expect("setting prefix counted");
    name.put(number.as_bytes()).expect("setting number counted");
    Path::root().member(&name.finish())
}
/// Each default charter leaves its brief room before the core admits work
/// (temper/docs/design/domain/agent/tools.md).
pub fn room(engine: &config::Engine, host: domain::room::HostPart, problems: &mut Problems) {
    for (index, charter) in engine.charters.iter().enumerate() {
        if let Err(short) = host.check(charter) {
            problems.push(charter_path(engine, index), What::Half(crate::startup::EngineWhat::Room(short)));
        }
    }
}
/// The binary's stock fits before any listener opens (protocol/configuration/stock.md).
/// The caller selects `Given::Stock`; worlds size their own defaults.
pub(crate) fn stock(engine: &config::Engine, problems: &mut Problems) {
    if engine.defaults == config::Defaults::Passed
        && let Err(floor) = crate::stock::floors(
            &crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes),
            &engine.limits,
        )
    {
        problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"overrides").member(floor.limit),
            What::Half(crate::startup::EngineWhat::Stock(floor)),
        );
    }
}
/// Named compatibility checks on the envelope runtime and store actually use.
pub fn engine_envelope(root: &domain::Limits, problems: &mut Problems) {
    forge_result(root.forge.client.op_bytes, root.tasks.result_bytes, problems);
    if !jig_domain_core::brief_holds_documents(root.brief.sections, root.tasks.charter_documents) {
        against(
            problems,
            b"root.brief.sections",
            b"engine.limits.overrides.tasks.charter_documents and the core sections",
        );
    }
    if let Err(found) = config::check_effective_limits(&domain::core_limits(root)) {
        let mut path = Path::root().member(b"engine");
        for segment in &found.path.segments {
            path = path.child(segment.clone());
        }
        problems.push(path, What::Jig(found.what));
    }
    let fits = match domain::policy_row_bound(root) {
        Some(bytes) => bytes <= u64::from(root.journal.transcript_bytes),
        None => false,
    };
    if !fits {
        against(
            problems,
            b"root.journal.transcript_bytes",
            b"engine.limits.core.policy_bytes plus its policy row carrier",
        );
    }
}
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
/// Standalone's requirements on jig's parts, including absolute agent paths.
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
/// The root has no hub child: the engine runs no slot, and every charter's
/// runs are placed on the workers (domain/root.md, 2).
pub fn hub(engine: &config::Engine, problems: &mut Problems) {
    let path = Path::root().member(b"engine");
    if engine.limits.core.fleet.engine_slots != 0 {
        problems.push(
            path.member(b"limits").member(b"overrides").member(b"fleet.engine_slots"),
            What::Half(crate::startup::EngineWhat::NoHub),
        );
    }
    for (index, charter) in engine.charters.iter().enumerate() {
        match charter.hosts {
            fleet::Kinds::Workers => {}
            fleet::Kinds::Engine | fleet::Kinds::Both => {
                problems
                    .push(charter_path(engine, index).member(b"hosts"), What::Half(crate::startup::EngineWhat::NoHub));
            }
        }
    }
}
/// The smallest resumed start within the engine's room (temper/docs/design/protocol/configuration/limits.md).
pub fn resumed_start(engine: &config::Engine, problems: &mut Problems) {
    let core = &engine.limits.core;
    let needs = match u32::try_from(core.fleet.call_name_bytes) {
        Ok(name) => match jig_protocol_worker_wire::sizes::smallest_resumed(
            core.call_records,
            name,
            jig_protocol_worker_wire::CEILINGS.call_tool,
            core.call_answer_bytes,
        ) {
            Some(bytes) => Some(u64::from(bytes)),
            None => None,
        },
        Err(_) => None,
    };
    short(
        problems,
        Path::root().member(b"engine").member(b"limits").member(b"io").member(b"output"),
        u64::from(engine.limits.io.largest_room()),
        b"the smallest resumed start",
        needs,
    );
}
/// The deployment's provider is not 0, which the root keeps for the forge's
/// sign-in (domain/root.md, 5).
pub fn provider(engine: &config::Engine, problems: &mut Problems) {
    for (index, provider) in engine.providers.iter().enumerate() {
        match provider.kind {
            config::ProviderKind::Deployment if provider.number == 0 => {
                problems.push(
                    at(index, Path::root().member(b"engine").member(b"providers")).member(b"number"),
                    What::Half(crate::startup::EngineWhat::Provider),
                );
            }
            config::ProviderKind::Deployment | config::ProviderKind::Link | config::ProviderKind::Application => {}
        }
    }
}
/// Each git identity's number is one both the engine's and the worker's
/// accounts tables hold.
pub fn credentials(engine: &config::Engine, workers: &[config::Worker], forges: u32, problems: &mut Problems) {
    let mut bound = engine.limits.core.accounts.credentials;
    for worker in workers {
        bound = bound.min(worker.limits.core.accounts.credentials);
    }
    if forges > bound {
        problems.push(own(b"forge.forges"), What::Half(crate::startup::EngineWhat::Credentials));
    }
}
/// Check the optional listener and the secret paths that require it.
pub fn webhooks(members: &super::application::Members, problems: &mut Problems) {
    let path = Path::root().member(b"application");
    if let Some(webhooks) = &members.webhooks {
        match &webhooks.listener {
            config::Address::Addr(Addr::Ip(socket)) if socket.ip().is_loopback() => {
                if socket.port() != 0 && webhooks.listener == members.clients.listener {
                    problems.push(path.member(b"webhooks").member(b"listener"), What::Jig(config::What::Unique));
                }
            }
            config::Address::Addr(_) | config::Address::Host { .. } => {
                problems.push(
                    path.member(b"webhooks").member(b"listener"),
                    What::Half(crate::startup::EngineWhat::Loopback),
                );
            }
        }
    }
}
/// Every git rule is checked independently, so one start reports them all.
pub fn limits(
    limits: &limits::Limits,
    #[cfg(feature = "worker")] git: &limits::GitComponent,
    #[cfg(not(feature = "worker"))] _: &limits::GitComponent,
    problems: &mut Problems,
) {
    #[cfg(feature = "worker")]
    super::worker::git_limits(&limits::worker(limits), git, problems);
    let sign_in = limits.sign_in;
    for (name, value) in [
        (b"sign_in.members".as_slice(), sign_in.members),
        (b"sign_in.pending", sign_in.pending),
        (b"sign_in.fetches", sign_in.fetches),
        (b"sign_in.body_bytes", sign_in.body_bytes),
        (b"sign_in.url_bytes", sign_in.url_bytes),
        (b"sign_in.secret_bytes", sign_in.secret_bytes),
    ] {
        if value == 0 {
            against(problems, name, b"a nonzero sign-in bound");
        }
    }
    if sign_in.deadline == Duration::ZERO {
        against(problems, b"sign_in.deadline_ms", b"a nonzero sign-in deadline");
    }
    if limits.forge.judge_projects > temper_client_protocol_wire::DISMISSED / 8 {
        against(problems, b"forge.judge_projects", b"wire::DISMISSED / 8");
    }
    if sign_in.members > limits.forge.bindings {
        against(problems, b"sign_in.members", b"forge.bindings");
    }
    if sign_in.body_bytes > 65_536 {
        against(problems, b"sign_in.body_bytes", b"skein-oauth's response bound");
    }
    if limits.clients.bundle_files == 0 {
        against(problems, b"clients.bundle_files", b"a nonempty bundle");
    }
    if limits.clients.bundle_bytes == 0 {
        against(problems, b"clients.bundle_bytes", b"a nonempty bundle");
    }
}
/// The longest routed remote: base, two separators, owner, name and `.git`.
pub fn longest_remote(machine: &temper_forge_setup::machine::Machine, name_bytes: u32) -> Option<u64> {
    Some(machine_remote(machine, name_bytes)?.max(u64::from(temper_engine_domain_forge::PULL_ADDRESS_BYTES)))
}
pub fn machine_remote(machine: &temper_forge_setup::machine::Machine, name_bytes: u32) -> Option<u64> {
    let mut base = skein_lib::len_u64(crate::system::GITHUB_REMOTE.len());
    if let Some(owned) = &machine.owned {
        base = base.max(skein_lib::len_u64(owned.remote.len()));
    }
    if let Some(fake) = &machine.fake {
        base = base.max(skein_lib::len_u64(fake.remote.len()));
    }
    if let Some(github) = &machine.github {
        base = base.max(skein_lib::len_u64(github.remote.len()));
    }
    remote(base, name_bytes)
}
pub(super) fn remote(base: u64, name_bytes: u32) -> Option<u64> {
    base.checked_add(u64::from(name_bytes).checked_mul(2)?)?.checked_add(6)
}
pub(super) fn names(remote: Option<u64>, limits: &limits::Limits, problems: &mut Problems) {
    if remote.unwrap_or(u64::MAX) > u64::from(limits.checkout.name_bytes) {
        against(problems, b"checkout.name_bytes", b"the longest remote");
    }
    if limits.forge.name_bytes > limits.checkout.name_bytes {
        against(problems, b"checkout.name_bytes", b"forge.name_bytes");
    }
}
#[cfg(not(feature = "worker"))]
fn bounded(bounds: &payload::Bounds, engine: bool, problems: &mut Problems) -> Option<payload::v1::Limits> {
    let ceilings = payload::v1::CEILINGS;
    let repositories: &[u8] = b"checkout.repositories";
    let names: &[u8] = if engine { b"forge.name_bytes" } else { b"checkout.name_bytes" };
    for (name, bytes, bound) in [
        (
            repositories,
            bounds.repositories,
            ceilings.items_repositories.min(ceilings.left_landed).min(ceilings.saved_repositories),
        ),
        (
            names,
            bounds.name_bytes,
            ceilings
                .repository_name
                .min(ceilings.merge_branch)
                .min(ceilings.branch_branch)
                .min(ceilings.writable_push)
                .min(ceilings.writable_landing_some)
                .min(ceilings.repository_save_some),
        ),
        (b"checkout.name_bytes", bounds.remote_bytes, ceilings.repository_remote),
        (b"checkout.conflicts", bounds.conflicts, ceilings.files_files),
        (b"checkout.path_bytes", bounds.path_bytes, ceilings.files_files_item),
        (b"identity.token_bytes", bounds.token_bytes, ceilings.token_token),
        (b"forges.installations", bounds.entries, ceilings.entries_entries),
        (b"forge_component.token_bytes", bounds.entry_token_bytes, ceilings.entry_token),
        (
            b"identity.text_bytes",
            bounds.text_bytes,
            ceilings
                .identity_name
                .min(ceilings.identity_login)
                .min(ceilings.identity_email)
                .min(ceilings.entry_account_login),
        ),
    ] {
        if bytes > bound {
            problems.push(own(name), What::Past { bytes: u64::from(bytes), bound: u64::from(bound) });
        }
    }
    payload::limits(bounds)
}
#[cfg(feature = "worker")]
use super::worker::checks::bounded;
/// Each side's application records fit jig's fields and the worker's admission.
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
    #[cfg(not(feature = "worker"))]
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    let mut sizes = [None, None];
    for (index, side) in [engine, worker].iter().enumerate() {
        if let Some(side) = side {
            #[cfg(feature = "worker")]
            {
                let prefix: &[u8] = if index == 0 { b"engine" } else { b"workers" };
                *sizes.get_mut(index).expect("two sides") = super::worker::checks::largest(side, prefix, problems);
            }
            #[cfg(not(feature = "worker"))]
            {
                *sizes.get_mut(index).expect("two sides") = payload::largest(side);
            }
            #[cfg(not(feature = "worker"))]
            if let Some(largest) = *sizes.get(index).expect("two sides") {
                for (name, bytes, bound) in [
                    (b"workspace_items".as_slice(), largest.items, ceilings.workspace_items),
                    (b"work_left_some", largest.left, ceilings.work_left_some),
                    // Saved's ceiling is checked with the checkout's other carriers.
                    (b"resource_resource_some", largest.resource, ceilings.resource_resource_some),
                    (b"application_value_secret", largest.grant, ceilings.application_value_secret),
                ] {
                    if bytes > bound {
                        let prefix: &[u8] = if index == 0 { b"engine" } else { b"workers" };
                        problems.push(
                            Path::root().member(prefix).member(b"limits").member(name),
                            What::Past { bytes: u64::from(bytes), bound: u64::from(bound) },
                        );
                    }
                }
            }
        }
    }
    if let [Some(engine), Some(worker)] = sizes
        && (!repository_shape || engine.items > worker.items)
    {
        against(problems, b"checkout.repositories", b"the worker's checkout capacity and checkout.name_bytes");
    }
}

/// Each sender's credential fits the receiving table and smith's bodies.
#[cfg(feature = "worker")]
pub fn grants(engine: &config::Engine, worker: &config::Limits, agent: &settings::Agent, problems: &mut Problems) {
    let credential = crate::values::token_credential_bytes(worker.token);
    if crate::values::credential_bytes(engine) > credential {
        problems.push(worker_limit(b"accounts.token"), What::Against { other: b"engine.limits accounts.token" });
    }
    let bodies = &agent.limits.bodies;
    if u32::try_from(crate::values::SOURCE_NAME_BYTES).expect("bounded source name") > bodies.program_value_name {
        problems.push(
            at(0, Path::root().member(b"workers")).member(b"agent").member(b"configuration"),
            What::Against { other: b"the agent's program_value_name" },
        );
    }
    if credential > bodies.credential_value_bytes {
        problems.push(worker_limit(b"accounts.token"), What::Against { other: b"the agent's credential_value_bytes" });
    }
}
#[cfg(feature = "worker")]
pub(super) fn worker_limit(member: &[u8]) -> Path {
    at(0, Path::root().member(b"workers")).member(b"limits").member(b"overrides").member(member)
}
/// The resume window as jig's hub counts it, excluding answered calls
/// (jig's `domain/hosts/README.md`; hosted.rs, `conversation_fits`).
fn resume_window(bytes: u32) -> Option<u64> {
    let bytes = u64::from(bytes);
    let carrier = u64::try_from(size_of::<Box<[u8]>>()).ok()?;
    bytes.checked_add(carrier.checked_sub(4)?.checked_mul(bytes / 4)?)
}
fn forge_result(bytes: u32, result: u32, problems: &mut Problems) {
    short(problems, own(b"forge.client.op_bytes"), u64::from(bytes), b"tasks.result_bytes", Some(u64::from(result)));
}
#[cfg(feature = "worker")]
pub(super) use super::worker::checks::agent_answer;
/// Each process checks the turn bounds its own document can know
/// (protocol/configuration/settings.md, 7). A worker alone knows its agent and hub.
pub fn turns(
    engine: Option<&config::Limits>,
    workers: &[config::Worker],
    own_limits: &limits::Limits,
    agent: Option<&settings::Agent>,
    problems: &mut Problems,
) {
    let engine_path = Path::root().member(b"engine").member(b"limits").member(b"overrides");
    if let Some(engine) = engine {
        forge_result(own_limits.forge.client.op_bytes, engine.core.tasks.result_bytes, problems);
    }
    if let Some(agent) = agent {
        let turn = u64::from(agent.turn_body());
        if let Some(engine) = engine {
            short(
                problems,
                engine_path.member(b"hub.outcome_bytes"),
                engine.hub.outcome_bytes,
                b"the agent's outcome (agent.json profile.declared.tool_payload)",
                Some(agent.outcome_bytes()),
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
        #[cfg(feature = "worker")]
        super::worker::checks::turns(workers, agent, problems);
        #[cfg(not(feature = "worker"))]
        for (index, worker) in workers.iter().enumerate() {
            let path = at(index, Path::root().member(b"workers")).member(b"limits").member(b"overrides");
            short(
                problems,
                path.member(b"hub.outcome_bytes"),
                worker.limits.hub.outcome_bytes,
                b"the agent's outcome (agent.json profile.declared.tool_payload)",
                Some(agent.outcome_bytes()),
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
#[cfg(feature = "worker")]
pub use super::worker::worker;
/// The landing rules' limits, one of which a refused rule breaks.
const LANDING: &[u8] =
    b"root.journal.transcript_bytes, authority.requirements, forge.judge_criteria and forge.change_policy.gates";
/// The index of `number` in a list of numbered entries, which a check found there.
fn index_of(numbers: &[u32], number: u32) -> usize {
    let mut index = 0;
    for (at, listed) in numbers.iter().enumerate() {
        if *listed == number {
            index = at;
        }
    }
    index
}
fn landing_config(engine: &settings::Engine) -> Map<u32, domain::ProjectLanding> {
    let mut landings = Map::with_capacity(engine.landings.capacity());
    for (&project, landing) in &engine.landings {
        landings.insert(project, landing.clone()).expect("the same capacity");
    }
    landings
}

/// What the root's construction asserts, refused before it is made, each
/// naming the configuration that breaks it (domain/root.md, 5).
pub fn root(
    mut core: jig_domain_core::Config,
    engine: &settings::Engine,
    seeds: &settings::Seeds,
    landing_projects: &[u32],
    limits: &domain::Limits,
    problems: &mut Problems,
) {
    let landings = landing_config(engine);
    let (forge, identities) = crate::forge::config(engine, seeds.deployment);
    if let Some(&connector) = core.connectors.first() {
        core.kinds = crate::stock::kinds(limits, connector);
    }
    let config = domain::Config {
        host_part: engine.host_part,
        kinds_stock: engine.kinds_stock.clone(),
        ways_stock: engine.ways_stock.clone(),
        period_span: crate::stock::PERIOD_SPAN,
        playground: crate::system::playground_config(engine.demo.as_ref(), true, engine.limits.setup.deadline),
        core,
        landings,
        forge,
        identities,
        machine: Some(crate::system::machine_config(&engine.machine)),
        forges: Box::new([]),
        under_box: false,
    };
    let mut numbers = List::with_capacity(engine.jig.limits.core.authority.projects);
    for project in &engine.jig.projects {
        numbers.push(project.number).expect("bounded projects");
    }
    let numbers = numbers.into_boxed();
    let engine_path = Path::root().member(b"engine");
    let landing_path = Path::root().member(b"application").member(b"landing");
    for refused in domain::refused(config, limits) {
        let (path, what) = match refused {
            domain::Refused::Workers(problem) => {
                let (path, what) = membership(problem);
                (path, What::Jig(what))
            }
            domain::Refused::Run { charter, bytes } => (
                charter_path(&engine.jig, charter),
                match bytes {
                    Some(bytes) => What::Within {
                        bytes,
                        limit: b"root.journal.run_bytes",
                        bound: u64::from(limits.journal.run_bytes),
                    },
                    None => What::Jig(config::What::Bound),
                },
            ),
            domain::Refused::Resume { bytes: 0 } => (
                engine_path.member(b"limits").member(b"overrides").member(b"core.resume_bytes"),
                What::Against { other: b"root.journal.transcript_bytes, above 0" },
            ),
            domain::Refused::Resume { bytes } => (
                own(b"root.journal.transcript_bytes"),
                What::Short {
                    bytes: u64::from(limits.journal.transcript_bytes),
                    of: b"core.resume_bytes",
                    needs: u64::from(bytes),
                },
            ),
            domain::Refused::JudgeProjects => {
                (own(b"forge.judge_projects"), What::Against { other: b"authority.projects in engine.limits" })
            }
            domain::Refused::JudgeCriteria => {
                (own(b"forge.judge_criteria"), What::Against { other: b"authority.requirements in engine.limits" })
            }
            domain::Refused::Escalation { project } => (
                at(index_of(&numbers, project), engine_path.member(b"projects"))
                    .member(b"policy")
                    .member(b"escalation_role"),
                What::Half(crate::startup::EngineWhat::Escalation),
            ),
            domain::Refused::Permissions { project, bytes } => (
                at(index_of(&numbers, project), engine_path.member(b"projects")).member(b"roles"),
                match bytes {
                    Some(bytes) => What::Within {
                        bytes,
                        limit: b"root.journal.transcript_bytes",
                        bound: u64::from(limits.journal.transcript_bytes),
                    },
                    None => What::Jig(config::What::Bound),
                },
            ),
            domain::Refused::Policy { project, bytes } => (
                at(index_of(&numbers, project), engine_path.member(b"projects")).member(b"policy").member(b"bytes"),
                match bytes {
                    Some(bytes) => What::Within { bytes, limit: b"core.policy_bytes", bound: limits.policy_bytes },
                    None => What::Jig(config::What::Bound),
                },
            ),
            domain::Refused::Landing { project: None } => {
                (landing_path.member(b"deployment"), What::Against { other: LANDING })
            }
            domain::Refused::Landing { project: Some(project) } => (
                at(index_of(landing_projects, project), landing_path.member(b"projects")).member(b"rules"),
                What::Against { other: LANDING },
            ),
            domain::Refused::Forges => (
                Path::root().member(b"application").member(b"forges"),
                What::Against { other: b"forge.repositories and forge.client.op_bytes" },
            ),
        };
        problems.push(path, what);
    }
}
/// Endpoint names, and every charter model, agree with the agent.
pub fn agent(engine: &config::Engine, agent: &settings::Agent, problems: &mut Problems) {
    for (index, endpoint) in engine.llm.endpoints.iter().enumerate() {
        let agrees = agent.resolves_endpoint(&endpoint.name);
        if !agrees {
            problems.push(
                at(index, Path::root().member(b"engine").member(b"llm").member(b"endpoints")),
                What::Half(crate::startup::EngineWhat::Agent),
            );
        }
    }
    for (index, provider) in engine.llm.providers.iter().enumerate() {
        for (offered, name) in provider.programs.iter().enumerate() {
            let mut declared = false;
            for program in agent.programs() {
                declared |= program.adapter.name() == name.as_ref();
            }
            if !declared {
                problems.push(
                    at(
                        offered,
                        at(index, Path::root().member(b"engine").member(b"llm").member(b"providers"))
                            .member(b"sources")
                            .member(b"programs"),
                    ),
                    What::Half(crate::startup::EngineWhat::Adapter),
                );
            }
        }
    }
}
/// Charter settings warn; deployment receiving bounds still refuse
/// (temper/docs/design/protocol/configuration/README.md, 3).
pub fn admission(
    core: &jig_domain_core::Config,
    engine: &config::Engine,
    agent: &settings::Agent,
    problems: &mut Problems,
) -> Problems {
    let mut needs = jig_domain_core::settings::fleet_needs::configured(core, &engine.limits.core);
    let warnings = admission_needs(&needs, engine, agent, problems);
    if !warnings.is_empty() || !problems.is_empty() {
        return warnings;
    }
    // Credential sources must fit the same charter models on the provider's direct way.
    for charter in &mut needs.charters {
        for need in &mut charter.models {
            for model in &engine.llm.models {
                if model.model.name != need.name {
                    continue;
                }
                for provider in &engine.llm.providers {
                    if provider.number == model.model.provider
                        && let Some(direct) = &provider.direct
                    {
                        for endpoint in &engine.llm.endpoints {
                            if endpoint.name == *direct {
                                need.endpoint = endpoint.number;
                            }
                        }
                    }
                }
            }
        }
    }
    admission_needs(&needs, engine, agent, problems)
}
fn admission_needs(
    needs: &fleet::Needs,
    engine: &config::Engine,
    agent: &settings::Agent,
    problems: &mut Problems,
) -> Problems {
    use fleet::AgentShortfall;
    let bounds = agent.bounds();
    let mut warnings = Problems::empty();
    // A charter warning must not mask a deployment's receiving shortfall.
    for (why, offered, required) in [
        (AgentShortfall::Message, bounds.message_bytes, needs.message_bytes),
        (AgentShortfall::Inbox, bounds.inbox, needs.start_messages),
        (AgentShortfall::Unacknowledged, bounds.unacknowledged, needs.unacknowledged),
    ] {
        if offered < required {
            problems.push(
                Path::root().member(b"engine").member(b"limits"),
                What::Half(crate::startup::EngineWhat::Admission {
                    why,
                    offered: u64::from(offered),
                    required: u64::from(required),
                }),
            );
            break;
        }
    }
    // Report the first charter shortfall, as hello admission does, without filling the notes queue.
    for charter in &needs.charters {
        if !charter.kinds.allows(fleet::HostKind::Worker) {
            continue;
        }
        for (model, need) in charter.models.iter().enumerate() {
            let mut found = false;
            for bound in &bounds.endpoints {
                found |= bound.endpoint == need.endpoint;
            }
            if !found {
                warnings.push(
                    model_path(
                        charter_index(engine, charter.version.charter),
                        u32::try_from(model).expect("configured model"),
                    ),
                    What::Half(crate::startup::EngineWhat::Agent),
                );
                return warnings;
            }
        }
    }
    let Some(why) = fleet::shortfall(needs, bounds, fleet::HostKind::Worker) else { return warnings };
    let engine_path = Path::root().member(b"engine");
    let path = match why {
        AgentShortfall::Model { charter, model } => {
            warnings
                .push(model_path(charter_index(engine, charter), model), What::Half(crate::startup::EngineWhat::Agent));
            return warnings;
        }
        AgentShortfall::Window { charter, model } | AgentShortfall::Output { charter, model } => {
            model_path(charter_index(engine, charter), model)
        }
        AgentShortfall::Turns { charter } => admission_charter_path(charter_index(engine, charter)).member(b"turns"),
        AgentShortfall::Time { charter } => admission_charter_path(charter_index(engine, charter)).member(b"time_ms"),
        AgentShortfall::Waiting { charter } => {
            admission_charter_path(charter_index(engine, charter)).member(b"waiting_ms")
        }
        AgentShortfall::Checks { charter } => {
            admission_charter_path(charter_index(engine, charter)).member(b"reserve_time_ms")
        }
        AgentShortfall::Spend => engine_path.member(b"rules").member(b"maximum_run_spend"),
        AgentShortfall::Message | AgentShortfall::Inbox | AgentShortfall::Unacknowledged => return warnings,
    };
    let figures = fleet::figures(needs, bounds, fleet::HostKind::Worker, why).expect("selected numerical shortfall");
    warnings.push(
        path,
        What::Half(crate::startup::EngineWhat::Admission { why, offered: figures.offered, required: figures.required }),
    );
    warnings
}
fn admission_charter_path(index: usize) -> Path {
    at(index, Path::root().member(b"engine").member(b"charters"))
}
fn model_path(index: usize, model: u32) -> Path {
    let path = admission_charter_path(index);
    match model.checked_sub(1) {
        None => path.member(b"model"),
        Some(alternative) => {
            at(usize::try_from(alternative).expect("configured alternative index"), path.member(b"alternatives"))
        }
    }
}
fn charter_index(engine: &config::Engine, number: u32) -> usize {
    for (index, charter) in engine.charters.iter().enumerate() {
        if charter.number == number {
            return index;
        }
    }
    unreachable!("canonical shortfall names a configured charter")
}
/// One io holds engine links, the worker's link, and four entities per child.
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
/// Compose engine and worker io exactly as jig's reference startup does.
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
/// Compare port numbers, also across IPv4/IPv6. Absent or ephemeral listeners reserve none.
pub fn redirect_ports(
    redirects: &[super::RedirectPort],
    port: Option<u16>,
    listener: super::Listener,
    problems: &mut Problems,
) {
    for redirect in redirects {
        if let Some(port) = port
            && port != 0
            && port == redirect.port
        {
            problems.push(redirect.path.clone(), What::Half(crate::startup::EngineWhat::Redirect(listener)));
        }
    }
}
pub(super) fn port(address: &config::Address) -> Option<u16> {
    match address {
        config::Address::Addr(address) => Some(address.socket()?.port()),
        config::Address::Host { port, .. } => Some(*port),
    }
}

/// Check standalone's raw agent path before jig joins a configuration path.
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

/// Encoded active writes; `AppId`/`User` are 9 bytes, dominated by admitted `AppKey`.
pub(crate) fn record_writes(limits: &limits::Limits) -> List<(Option<u64>, &'static [u8], Option<usize>)> {
    let capacity = limits.forge.forges.checked_add(5).expect("five scalar writes");
    let mut writes = List::with_capacity(capacity);
    let accepted = match u64::from(limits.forges.installations).checked_mul(8) {
        Some(bytes) => bytes.checked_add(7),
        None => None,
    };
    // All rows use a u16 version and an enum tag; variable fields add a u32 length.
    let tokens = limits.forge_component.token_bytes.max(limits.forge_component.hook_secret_bytes);
    for (bytes, source) in [
        (u64::from(tokens).checked_add(7), b"tokens (forge_component.token_bytes and hook_secret_bytes)".as_slice()),
        (Some(11), b"User and AppId"),
        (u64::from(limits.forge_component.key_bytes).checked_add(7), b"AppKey (forge_component.key_bytes)".as_slice()),
        (accepted, b"Accepted (forges.installations)"),
        (u64::from(limits.forge.name_bytes).checked_add(7), b"AppSlug (forge.name_bytes)"),
    ] {
        writes.push((bytes, source, None)).expect("five scalar writes");
    }
    for _ in 0..limits.forge.forges {
        // Origin carries a 253-byte host, separator and five-digit port, with its codec headers.
        writes.push((Some(267), b"Origin", None)).expect("kept forge origins");
    }
    writes
}
/// Application records fit the store and the largest encoded active write.
pub fn records(jig: &config::Engine, limits: &limits::Limits, problems: &mut Problems) {
    let component = &limits.forge_component;
    for (name, positive, other) in [
        (b"forges.log_origins".as_slice(), limits.forges.log_origins > 0, b"forges.log_origins (above 0)".as_slice()),
        (b"forge_component.log_origins", component.log_origins > 0, b"forge_component.log_origins (above 0)"),
        (b"forge_component.log_poll_ms", component.log_poll > Duration::ZERO, b"forge_component.log_poll_ms (above 0)"),
        (
            b"forge_component.log_catch_up_ms",
            component.log_catch_up > Duration::ZERO,
            b"forge_component.log_catch_up_ms (above 0)",
        ),
        (b"forge_component.log_pages", component.log_pages > 0, b"forge_component.log_pages (above 0)"),
        (b"forge_component.delivery_bytes", component.delivery_bytes > 0, b"forge_component.delivery_bytes (above 0)"),
    ] {
        if !positive {
            against(problems, name, other);
        }
    }
    if limits.forges.log_origins > 32 {
        against(problems, b"forges.log_origins", b"forges.log_origins (at most 32)");
    }
    if limits.forges.log_origins > component.log_origins {
        against(problems, b"forges.log_origins", b"forge_component.log_origins");
    }
    if component.log_catch_up < component.log_poll {
        against(problems, b"forge_component.log_catch_up_ms", b"forge_component.log_poll_ms");
    }
    if component.log_catch_up > Duration::from_secs(86_400) {
        against(
            problems,
            b"forge_component.log_catch_up_ms",
            b"forge_component.log_catch_up_ms (at most 86400000 ms / 24 h)",
        );
    }
    if limits.picker.page == 0 || limits.picker.page > 100 {
        against(problems, b"picker.page", b"picker.page (1 to 100)");
    }
    if limits.routes.query_bytes == 0 {
        against(problems, b"routes.query_bytes", b"positive query bound");
    }
    let value = limits.records.value_bytes;
    match secret_value_bound(jig, limits) {
        Ok(bound) => {
            if value == 0 || value > bound {
                against(problems, b"records.value_bytes", b"engine.limits.store.value");
            }
        }
        Err(refused) => {
            let found = task_store_refusal(jig, refused);
            for problem in &found.problems {
                problems.push(problem.path.clone(), problem.what);
            }
        }
    }
    let mut largest = (0, b"AppKey (forge_component.key_bytes)".as_slice(), None);
    for (bytes, source, index) in record_writes(limits).into_boxed() {
        if bytes.unwrap_or(u64::MAX) > largest.0 {
            largest = (bytes.unwrap_or(u64::MAX), source, index);
        }
    }
    if u64::from(value) < largest.0 {
        against(problems, b"records.value_bytes", largest.1);
        if let Some(index) = largest.2 {
            problems.push(
                at(index, Path::root().member(b"application").member(b"forges")).member(b"api"),
                What::Against { other: b"Origin (records.value_bytes)" },
            );
        }
    }
    // Every observed installation must fit the deployment's client document (client/forge/records.md).
    let bound = temper_client_protocol_wire::INSTALLATIONS;
    if limits.forge_component.installations > bound {
        problems.push(
            own(b"forge_component.installations"),
            What::Past { bytes: u64::from(limits.forge_component.installations), bound: u64::from(bound) },
        );
    }
    if limits.forges.installations == 0 || limits.forges.installations > limits.forge_component.installations {
        against(problems, b"forges.installations", b"forge_component.installations");
    }
}

pub(crate) fn secret_value_bound(jig: &config::Engine, own: &limits::Limits) -> Result<u32, crate::store::Refusal> {
    let root = limits::engine(&jig.limits, own)
        .ok_or(crate::store::Refusal::Map(jig_protocol_store::sizes::SizeRefusal::Ceiling))?;
    Ok(crate::store::layout(&root, &root.forge, &jig.limits.store)?.secrets.value)
}

fn task_record_binds(codec: &jig_protocol_store::tasks_v10::Limits) -> bool {
    match jig_protocol_store::tasks_v10::Value::encoded_bound(codec) {
        Some(bytes) => jig_protocol_store::tasks_v10::TaskRecord::encoded_bound(codec) == bytes.checked_sub(1),
        None => false,
    }
}

fn map_value(refused: crate::store::Refusal) -> Option<(u32, u32)> {
    use crate::store::Refusal;
    use jig_protocol_store::sizes::SizeRefusal;
    match refused {
        Refusal::Map(SizeRefusal::Value { needs, has }) => Some((needs, has)),
        Refusal::Map(
            SizeRefusal::Key { .. }
            | SizeRefusal::Ops { .. }
            | SizeRefusal::Commit { .. }
            | SizeRefusal::Page { .. }
            | SizeRefusal::Queue { .. }
            | SizeRefusal::QueuedBytes { .. }
            | SizeRefusal::Whole { .. }
            | SizeRefusal::Ceiling,
        )
        | Refusal::ReadRoom { .. }
        | Refusal::Transcript { .. }
        | Refusal::Page { .. } => None,
    }
}

fn task_store_refusal(engine: &config::Engine, refused: crate::store::Refusal) -> Problems {
    if let Some((needs, has)) = map_value(refused)
        && let Some(codec) = jig_protocol_store::codec::tasks::limits(&engine.limits.core.tasks)
        && jig_protocol_store::tasks_v10::Row::encoded_bound(&codec) == Some(u64::from(needs))
        && task_record_binds(&codec)
    {
        let mut problems = Problems::empty();
        problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"overrides").member(b"tasks.batch_bytes"),
            What::Half(crate::startup::EngineWhat::StoreTask {
                needs,
                has,
                batch_bytes: engine.limits.core.tasks.batch_bytes,
            }),
        );
        return problems;
    }
    store_refusal(refused)
}

pub(crate) fn store_refusal(refused: crate::store::Refusal) -> Problems {
    let (path, what) = match refused {
        crate::store::Refusal::ReadRoom { read, needs } => {
            (own(b"root.loads.read_bytes"), What::Short { bytes: read, of: b"a read's room", needs })
        }
        crate::store::Refusal::Page { transcript, frame, page, ceiling } => (
            own(b"root.loads.transcript_bytes"),
            What::Half(crate::startup::EngineWhat::StorePage { transcript, frame, page, ceiling }),
        ),
        crate::store::Refusal::Transcript { bytes, ceiling } => (
            own(b"root.loads.transcript_bytes"),
            What::Within {
                bytes: u64::from(bytes),
                limit: b"the transcript codec's ceiling",
                bound: u64::from(ceiling),
            },
        ),
        crate::store::Refusal::Map(why) => (
            Path::root().member(b"engine").member(b"limits").member(b"store"),
            What::Half(crate::startup::EngineWhat::StoreMap(why)),
        ),
    };
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}

/// Translate the kit's configured membership refusal into its startup path.
pub(crate) fn membership(problem: Problem) -> (Path, config::What) {
    let hosts = Path::root().member(b"engine").member(b"hosts").member(b"workers");
    match problem {
        Problem::Workers { .. } => (hosts, config::What::Bound),
        Problem::WorkerName { worker, .. } => {
            (hosts.child(Segment::Index(worker)).member(b"name"), config::What::Bound)
        }
    }
}

#[cfg(not(feature = "worker"))]
/// Only an agent running locally must hold the answer in its tool payload.
pub(super) fn agent_answer(agent: &settings::Agent, problems: &mut Problems) {
    short(
        problems,
        at(0, Path::root().member(b"workers"))
            .member(b"agent")
            .member(b"profile")
            .member(b"declared")
            .member(b"tool_payload"),
        u64::from(agent.tool_payload()),
        b"temper's documentation answer (read_docs)",
        Some(u64::from(temper_service_core::DOC_ANSWER)),
    );
}
