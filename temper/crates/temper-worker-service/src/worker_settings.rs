//! The worker's checked construction bounds and payload sizes (protocol/README.md, 5).
use crate::settings;
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_worker_link as link;
use skein_io as io;
use skein_lib::Token;
use smith_host_protocol as host;
use temper_protocol_worker_wire::{self as payload, v1};
use temper_service_core::limits;
pub use temper_service_core::values::{grant_bytes, token_credential_bytes as credential_bytes};
use temper_worker_domain as root;

/// Git's operations, job files, source inspection and the worker root's close. Git's operation
/// count covers the checkout workspaces, which cover the worker's slots.
pub(crate) fn operations(git: &temper_worker_protocol_git::Limits) -> Option<u32> {
    temper_worker_protocol_git::limits::operations(git)?.checked_add(3)?.checked_add(git.operations)
}

/// Operations reserved by the process for the configured git component.
pub(crate) fn configured_operations(git: &limits::GitComponent) -> Option<u32> {
    operations(&git_component(*git))
}

/// Every queue and component bound established before constructing a worker.
#[derive(Debug)]
pub struct Prepared {
    pub root: root::Limits,
    pub link: link::Limits,
    pub payload: v1::Limits,
    pub prefix: u64,
    pub address: io::Addr,
}

pub(crate) fn problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"workers").child(config::Segment::Index(0)).member(member), what);
    problems
}

/// Check the root, host adapter and link before any state is allocated.
pub fn prepare(worker: &settings::Worker, agent: &Agent, own: &Limits) -> Result<Prepared, Problems> {
    if !io::valid(&worker.jig.limits.io) {
        return Err(problem(b"limits", What::Bound));
    }
    let address = match worker.jig.engine {
        config::Address::Addr(address) => address,
        config::Address::Host { .. } => return Err(problem(b"engine", What::Form)),
    };
    if !programs_valid(&agent.programs) {
        return Err(problem(b"agent", What::Bound));
    }
    let domain_limits = root_limits(worker, agent, own);
    if !root::valid(&domain_limits) {
        return Err(problem(b"limits", What::Bound));
    }
    host::Process::new(Token::new(0), &agent.limits, agent.host_limits.detail_bytes, agent.endpoints.clone())
        .or(Err(problem(b"agent", What::Bound)))?;
    let payload = payload_limits(own).ok_or(problem(b"limits", What::Bound))?;
    let link = link_limits(&worker.jig, &payload, own.peer.forges).ok_or(problem(b"limits", What::Bound))?;
    if !link::valid(&link) {
        return Err(problem(b"limits", What::Bound));
    }
    let prefix = crate::tokens::worker(0).ok_or(problem(b"slots", What::Bound))?;
    worker.jig.slots.checked_mul(4).ok_or(problem(b"slots", What::Bound))?;
    worker.jig.limits.core.accounts.accounts.checked_add(1).ok_or(problem(b"limits", What::Bound))?;
    let bodies = &agent.limits.bodies;
    if credential_bytes(worker.jig.limits.token) > bodies.credential_value_bytes
        || u32::try_from(smith_host_domain::SourceName::BYTES).expect("bounded source name") > bodies.program_value_name
    {
        return Err(problem(b"limits", What::Bound));
    }
    Ok(Prepared { root: domain_limits, link, payload, prefix, address })
}

#[must_use]
pub fn root_limits(worker: &settings::Worker, agent: &Agent, own: &Limits) -> root::Limits {
    let mut host = worker.jig.limits.hub;
    host.slots = worker.jig.slots;
    root::Limits {
        tooling: tooling(worker.jig.slots),
        host,
        checkout: checkout(own.checkout),
        agent: smith_host_domain::Limits { agents: worker.jig.slots, ..agent.host_limits },
        reply_bytes: agent.limits.bodies.host_reply_text,
        bounds: agent.bounds.clone(),
        stop_bound: worker.jig.stop_bound,
        drain: worker.jig.graces.drain,
        after_grace: skein_lib::Duration::from_nanos(
            worker
                .jig
                .stop_bound
                .as_nanos()
                .saturating_sub(worker.jig.limits.link.ping.as_nanos())
                .saturating_sub(worker.jig.limits.link.silence.as_nanos())
                .saturating_sub(worker.jig.graces.grace.as_nanos()),
        ),
        watermark: root::WATERMARK,
    }
}

/// The worker accepts repository and path bytes only up to its checkout limits.
#[must_use]
pub fn payload_limits(own: &Limits) -> Option<v1::Limits> {
    let mut limits = payload::limits(&own.bounds())?;
    limits.job_spec_command = limits::TOOLING_PATH_BYTES;
    limits.job_spec_request = limits::TOOLING_FILE_BYTES;
    Some(limits)
}

/// Jig's link profile, with every temper part priced by the generated codec.
#[must_use]
pub fn link_limits(worker: &config::Worker, payload: &v1::Limits, forges: u32) -> Option<link::Limits> {
    let jig = &worker.limits;
    let core = jig.core;
    let hub = jig.hub;
    let profile = jig.link;
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    let largest = payload::largest(payload)?;
    let sizes = jig_protocol_worker_wire::sizes::Sizes {
        program_sources: core.fleet.program_sources,
        piece: hub.piece_bytes,
        charter: u32::try_from(hub.charter_bytes).ok()?,
        transcript: u32::try_from(hub.transcript_bytes).ok()?,
        turn: u32::try_from(hub.turn_bytes).ok()?,
        message: core.tasks.message_bytes,
        sender: ceilings.message_sender,
        call_name: u32::try_from(core.fleet.call_name_bytes).ok()?,
        tool_name: ceilings.call_tool,
        call_input: core.call_answer_bytes,
        answer: core.call_answer_bytes,
        result: u32::try_from(hub.outcome_bytes).ok()?,
        detail: hub.detail_bytes,
        grant: grant_bytes(credential_bytes(jig.token), largest.grant)?,
        name: profile.name,
        secret: profile.secret,

        items: largest.items,
        left: largest.left,
        saved: largest.saved,
        evidence: u32::try_from(hub.delivery_evidence_bytes).ok()?,
        resource: largest.resource,
        slots: worker.slots,
        calls: hub.run_calls,
        turns: hub.turns,
        inbox: core.tasks.run_inbox,
        answered: core.call_records,
        models: ceilings.bounds_endpoints,
        workstreams: core.fleet.workstreams,
        accounts: core.accounts.accounts,
        credentials: forges,
        facts: profile.facts,
    };
    Some(link::Limits {
        sizes,
        chunk: profile.chunk,
        room: worker.limits.io.largest_room(),
        handshake: profile.handshake,
        ping: profile.ping,
        silence: profile.silence,
        stall: profile.stall,
        backoff: profile.backoff,
        ceiling: profile.ceiling,
        proved: profile.proved,
        grace: worker.graces.grace,
        busy: profile.busy,
    })
}

/// One retained declaration per recognized adapter, with each path at smith's bound.
fn programs_valid(programs: &[settings::Program]) -> bool {
    if programs.len() > 1 {
        return false;
    }
    for program in programs {
        for path in [&program.path, &program.directory] {
            if path.len() > settings::PROGRAM_PATH_BYTES || !path.starts_with(b"/") || path.contains(&0) {
                return false;
            }
        }
    }
    true
}

/// The application's wire is part of every worker's statement (protocol/wires.md, 4).
#[must_use]
pub fn credential(worker: &settings::Worker, secret: &[u8]) -> link::Credential {
    let mut wires = skein_lib::List::with_capacity(1);
    wires
        .push(jig_protocol_worker_wire::Spoken::current(
            payload::wire::FAMILY,
            payload::wire::VERSION,
            payload::wire::layout(),
        ))
        .expect("one application wire");
    link::Credential { name: worker.jig.name.clone(), secret: config::secret(secret.into()), wires }
}

fn tooling(slots: u32) -> temper_worker_domain::tooling::Limits {
    temper_worker_domain::tooling::Limits {
        file_bytes: limits::TOOLING_FILE_BYTES,
        tail_bytes: limits::TOOLING_LOG_BYTES,
        path_bytes: limits::TOOLING_PATH_BYTES,
        jobs: slots,
        path: limits::TOOLING_PATH.into(),
    }
}

/// Translate the shared checkout envelope to the worker's domain (protocol/services.md, 5).
#[must_use]
pub fn checkout(limits: limits::Checkout) -> temper_worker_domain_checkout::Limits {
    temper_worker_domain_checkout::Limits {
        workspaces: limits.workspaces,
        preparations: limits.preparations,
        repositories: limits.repositories,
        name_bytes: limits.name_bytes,
        message_bytes: limits.message_bytes,
        conflicts: limits.conflicts,
        path_bytes: limits.path_bytes,
        remote_timeout: limits.remote_timeout,
        local_timeout: limits.local_timeout,
        facts: limits.facts,
    }
}

/// Translate the shared git envelope to the worker's protocol (protocol/services.md, 5).
#[must_use]
pub fn git_component(limits: limits::GitComponent) -> temper_worker_protocol_git::Limits {
    temper_worker_protocol_git::Limits {
        operations: limits.operations,
        stdout_bytes: limits.stdout_bytes,
        stderr_bytes: limits.stderr_bytes,
        read_bytes: limits.read_bytes,
        sweep_entries: limits.sweep_entries,
        sweep_bytes: limits.sweep_bytes,
    }
}

/// The worker's host profile and its declaration to the engine
/// (temper/docs/design/protocol/services.md, 3 and 5).
#[derive(Debug)]
pub struct Agent {
    /// Receiving run bounds, including the protocol's measured schema costs.
    pub run_limits: smith_domain_run::Limits,
    pub tool_payload: u32,
    pub programs: Box<[settings::Program]>,
    pub endpoints: smith_protocol_channel::Endpoints,
    pub limits: smith_host_protocol::Limits,
    pub bounds: jig_domain_link::Bounds,
    pub host_limits: smith_host_domain::Limits,
}

const _: () = assert!(
    temper_service_core::values::SOURCE_NAME_BYTES == smith_host_domain::SourceName::BYTES,
    "engine and worker source name bounds agree"
);

impl Agent {
    pub(crate) fn tool_payload(&self) -> u32 {
        self.tool_payload
    }
    pub(crate) fn host_ask_input(&self) -> u32 {
        self.limits.bodies.host_ask_input
    }
    pub(crate) fn host_reply_text(&self) -> u32 {
        self.limits.bodies.host_reply_text
    }
    pub(crate) fn turn_body(&self) -> u32 {
        self.limits.bodies.turn_body
    }
    pub(crate) fn outcome_bytes(&self) -> u64 {
        self.host_limits.outcome_bytes
    }
    /// The allowances and names checked by the engine (protocol/services.md, 3).
    #[must_use]
    pub fn declaration(&self) -> temper_service_core::agent::Declaration {
        let mut programs =
            skein_lib::List::with_capacity(u32::try_from(self.programs.len()).expect("bounded program declarations"));
        for program in &self.programs {
            programs.push(program.adapter.name().into()).expect("one name per program");
        }
        let mut endpoints = skein_lib::List::with_capacity(self.endpoints.entries().len());
        for endpoint in self.endpoints.entries() {
            endpoints.push(endpoint.name.clone()).expect("one name per endpoint");
        }
        temper_service_core::agent::Declaration {
            charter_bytes: self.host_limits.charter_bytes,
            tool_payload: self.tool_payload,
            host_ask_input: self.limits.bodies.host_ask_input,
            host_reply_text: self.limits.bodies.host_reply_text,
            turn_body: self.limits.bodies.turn_body,
            outcome_bytes: self.host_limits.outcome_bytes,
            run_limits: self.run_limits,
            programs: programs.into_boxed(),
            endpoints: endpoints.into_boxed(),
            bounds: self.bounds.clone(),
        }
    }
}

/// The worker's aggregate on the shared envelopes (protocol/services.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub checkout: limits::Checkout,
    pub git_component: limits::GitComponent,
    pub peer: limits::Peer,
}
impl Limits {
    #[must_use]
    pub fn shared(shared: limits::Shared) -> Self {
        Self { checkout: shared.checkout, git_component: shared.git_component, peer: shared.peer }
    }
    #[must_use]
    pub fn bounds(&self) -> payload::Bounds {
        payload::Bounds {
            repositories: self.checkout.repositories,
            name_bytes: self.checkout.name_bytes,
            remote_bytes: self.checkout.name_bytes,
            conflicts: self.checkout.conflicts,
            path_bytes: self.checkout.path_bytes,
            token_bytes: v1::CEILINGS.token_token,
            entries: self.peer.installations,
            entry_token_bytes: self.peer.token_bytes,
            text_bytes: v1::CEILINGS.identity_name,
        }
    }
}
/// The worker's mirror of the close kind (protocol/services.md, 2).
pub const CLOSE_KIND: u32 = 4;
#[must_use]
pub fn close_bound() -> jig_domain_link::JobBound {
    jig_domain_link::JobBound {
        kind: jig_domain_link::JobKind(CLOSE_KIND),
        spec: limits::CLOSE_SPEC_BYTES,
        result: limits::CLOSE_RESULT_BYTES,
        deadline: limits::CLOSE_DEADLINE,
    }
}
/// The worker's declared kinds, ascending (protocol/services.md, 5).
#[must_use]
pub fn job_kinds() -> Box<[temper_worker_domain::tooling::Kind]> {
    Box::new([
        temper_worker_domain::tooling::Kind {
            bound: limits::prepare_bound(),
            verb: b"prepare".as_slice().into(),
            delivers: true,
        },
        temper_worker_domain::tooling::Kind {
            bound: limits::merge_bound(),
            verb: b"merge".as_slice().into(),
            delivers: true,
        },
        temper_worker_domain::tooling::Kind { bound: close_bound(), verb: b"close".as_slice().into(), delivers: true },
    ])
}
