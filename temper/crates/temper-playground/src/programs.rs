//! The composition's registry and harness seam (temper/docs/design/playground.md, 3–4).
use crate::{
    documents,
    machine::{self, SignalHost},
    service::{Observe, ServiceHost},
    story::Links,
};
use skein_io::kernel::{Complete, Exit, Spawn, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_sim_world::{HostedProgram, Inherited, StartupRoot, StartupRoots};
use std::{cell::RefCell, rc::Rc};
mod shell;
pub use shell::Shell;

/// Instrumentation supplied by a world's harness, absent from the playground.
pub trait Harness: 'static {
    type Service: Observe;
    type Process: Host;
    /// Whether git uses the authenticated HTTP route and prepared agent roots,
    /// read at each spawn so one harness may serve worlds of both kinds.
    #[must_use]
    fn github() -> bool {
        false
    }
    fn configure(_document: &mut serde_json::Value) {}
    #[must_use]
    fn agent_profile() -> smith_agent_service::profile::Profile {
        documents::agent_profile()
    }
    #[must_use]
    fn agent_settings() -> temper_service::settings::Agent {
        documents::agent()
    }
    /// Override sets for a test phase's open; the playground keeps production's sets.
    #[must_use]
    fn patch_sets() -> Option<jig_protocol_store::PatchSets> {
        None
    }
    /// A world's worker may speak chosen versions; the playground keeps the build's ranges.
    #[must_use]
    fn worker_wires() -> Option<temper_service::worker_settings::Wires> {
        None
    }
    fn spawned(_role: Role, _inherited: &Inherited, _base: &[u8]) {}
    fn complete(_role: Role, _complete: &Complete) {}
    /// A routed git world's protocol bounds.
    #[must_use]
    fn git_limits() -> skein_fake_forge_http::git::Limits {
        crate::fakes::forge_limits().git()
    }
    /// A harness may keep its service's independent seed inputs.
    #[must_use]
    fn seeds(one: u64) -> temper_service::settings::Seeds {
        documents::seeds(one).service()
    }
    #[must_use]
    fn agent_seed(one: u64) -> u64 {
        documents::seeds(one).agent
    }
    /// The scripted provider a restartable LLM peer is made, and made again, from.
    #[must_use]
    fn provider(one: u64) -> skein_fake_peers::llm::Peer {
        crate::fakes::scripted_provider(Box::new([]), documents::seeds(one).llm)
    }
    /// The seed a restartable forge peer is made, and made again, with.
    #[must_use]
    fn forge_seed(one: u64) -> u64 {
        documents::seeds(one).forge
    }
    #[must_use]
    fn files() -> temper_service::settings::Files {
        documents::files()
    }
    #[must_use]
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        documents::given(document)
    }
}
impl Harness for () {
    type Service = ();
    type Process = Never;
}
/// An uninhabited process when there is no harness.
pub enum Never {}
impl Host for Never {
    fn iterate(&mut self, _: Time, _: Wall) {
        match *self {}
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        match *self {}
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        match *self {}
    }
    fn work_pending(&self, _: Time) -> bool {
        match *self {}
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        match *self {}
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        match *self {}
    }
    fn is_empty(&self) -> bool {
        match *self {}
    }
    fn operations(&self) -> u32 {
        match *self {}
    }
}
/// Roles whose startup descriptors and completions a harness may observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Service,
    Agent,
    Git,
    Shell,
    Provider,
    Forge,
    Visitor,
    Other,
}

/// Parameters shared by function-pointer factories during one world's steps.
#[derive(Clone)]
pub struct Parameters {
    pub deployment: Box<[u8]>,
    pub seed: u64,
    pub links: Links,
    pub forge: temper_fake_git::Typed,
    pub route: Option<skein_fake_forge_http::git::Limits>,
    pub disk: Rc<RefCell<skein_fake_machine::Machine>>,
}
thread_local! { static PARAMETERS: RefCell<Option<Parameters>> = const { RefCell::new(None) }; }
/// Install factory parameters for the worker's one world, with no end.
pub fn install(parameters: Parameters) {
    PARAMETERS.with(|at| *at.borrow_mut() = Some(parameters));
}

/// Scope factory parameters around construction or steps; restore them even on unwind.
pub fn with<R>(parameters: Parameters, run: impl FnOnce() -> R) -> R {
    struct Reset(Option<Parameters>);
    impl Drop for Reset {
        fn drop(&mut self) {
            PARAMETERS.with(|at| *at.borrow_mut() = self.0.take());
        }
    }
    let reset = Reset(PARAMETERS.with(|at| at.replace(Some(parameters))));
    let result = run();
    drop(reset);
    result
}
fn parameters() -> Parameters {
    PARAMETERS.with(|at| at.borrow().as_ref().expect("factory parameters scoped around the world's steps").clone())
}

/// Processes common to every world, plus the harness's own.
pub enum Process<H: Harness> {
    Service(Box<ServiceHost<H>>),
    Agent(Box<machine::Rooted<skein_sim_world::PipeHost<smith_agent_shell::Agent>>>),
    List(Box<smith_agent_shell::list::Process>),
    GitRoute(Box<temper_fake_git::Git<temper_fake_git::Route>>),
    Git(Box<temper_fake_git_hosted::Git<temper_fake_git_hosted::Typed>>),
    Shell(Box<Shell>),
    Peer(Box<SignalHost<skein_fake_peers::llm::Peer>>),
    Forge(Box<SignalHost<skein_fake_peers::forge::Peer>>),
    Visitor(Box<skein_playground::visitor::Visitor>),
    Other(H::Process),
}
impl<H: Harness> Process<H> {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Service(h) => h.as_ref(),
            Self::Agent(h) => h.as_ref(),
            Self::List(h) => h.as_ref(),
            Self::GitRoute(h) => h.as_ref(),
            Self::Git(h) => h.as_ref(),
            Self::Shell(h) => h.as_ref(),
            Self::Peer(h) => h.as_ref(),
            Self::Forge(h) => h.as_ref(),
            Self::Visitor(h) => h.as_ref(),
            Self::Other(h) => h,
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Service(h) => h.as_mut(),
            Self::Agent(h) => h.as_mut(),
            Self::List(h) => h.as_mut(),
            Self::GitRoute(h) => h.as_mut(),
            Self::Git(h) => h.as_mut(),
            Self::Shell(h) => h.as_mut(),
            Self::Peer(h) => h.as_mut(),
            Self::Forge(h) => h.as_mut(),
            Self::Visitor(h) => h.as_mut(),
            Self::Other(h) => h,
        }
    }
    fn role(&self) -> Role {
        match self {
            Self::Service(_) => Role::Service,
            Self::Agent(_) | Self::List(_) => Role::Agent,
            Self::GitRoute(_) | Self::Git(_) => Role::Git,
            Self::Shell(_) => Role::Shell,
            Self::Peer(_) => Role::Provider,
            Self::Forge(_) => Role::Forge,
            Self::Visitor(_) => Role::Visitor,
            Self::Other(_) => Role::Other,
        }
    }
}
impl<H: Harness> Host for Process<H> {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host().state_digest()
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        let role = self.role();
        for complete in self.host_mut().completions().iter() {
            H::complete(role, complete);
        }
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
}
/// The service owns independent state and worker directories.
#[must_use]
pub fn service_roots() -> Vec<StartupRoot> {
    vec![
        StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() },
        StartupRoot { name: b"worker".as_slice().into(), path: b"worker".as_slice().into() },
    ]
}
/// Start the same service from deployment bytes as `temper serve`.
#[must_use]
pub fn service<H: Harness>(inherited: &Inherited) -> Process<H> {
    let parameters = parameters();
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("state root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    H::spawned(Role::Service, inherited, b"state");
    let mut document: serde_json::Value = serde_json::from_slice(&parameters.deployment).expect("deployment JSON");
    H::configure(&mut document);
    let given = H::given(&document);
    documents::settings::strip_models(&mut document);
    let bytes = serde_json::to_vec(&document).expect("deployment JSON");
    let mut service = temper_service::startup::with_setup_given(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &H::files(),
        Some(&H::agent_settings()),
        &H::seeds(parameters.seed),
        None,
        Some(temper_service::WorkerRuntime {
            wires: H::worker_wires(),
            root: worker,
            path: b"/worker".as_slice().into(),
            agent: documents::pinned_agent(),
        }),
        documents::forges(),
        None,
        None,
        None,
        &given,
        Box::new([]),
    )
    .unwrap_or_else(|problems| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems))));
    if let Some(sets) = H::patch_sets() {
        service.patch_sets(sets);
    }
    service.adopt_store(root, 1000).expect("deployment store");
    service.adopt_signals(inherited.signal).expect("service signal");
    Process::Service(Box::new(ServiceHost::new(service, H::Service::started(root, inherited), parameters)))
}
/// Host smith with its own configuration and a product diagnostic pipe.
#[must_use]
pub fn agent<H: Harness>(spawn: &Spawn, inherited: &Inherited) -> Process<H> {
    H::spawned(Role::Agent, inherited, b"worker/agent");
    let profile = H::agent_profile();
    if list_arguments(&spawn.args) {
        return Process::List(Box::new(smith_agent_shell::list::Process::with_endpoints(
            documents::agent_configuration_for(&profile).llm_endpoints,
            smith_agent_shell::list::Faces {
                stdin: inherited.pipes.iter().find(|(child, _)| *child == 0).expect("stdin").1,
                stdout: inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1,
                stderr: Some(inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1),
                signal: inherited.signal,
            },
        )));
    }
    let declarations = documents::agent_declarations_for(&profile);
    let stderr = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = skein_sim_world::PipeWriter::new(stderr, 4096);
    let agent = smith_agent_shell::Agent::new(
        smith_agent_shell::config::Configuration {
            service: documents::agent_configuration_for(&profile),
            memory: profile.declared.memory,
            trace: None,
            profile,
            declarations,
            program: None,
            program_endpoints: std::collections::BTreeSet::new(),
        },
        smith_agent_shell::Resources {
            input: inherited.pipes.iter().find(|(child, _)| *child == 0).expect("stdin").1,
            output: inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1,
            error: Some(stderr),
            signals: inherited.signal,
            seed: H::agent_seed(parameters().seed),
            pid: 101,
            trace: None,
            roots: Some(inherited.roots.iter().map(|(_, fd)| *fd).collect()),
            programs: None,
        },
        Box::new(errors),
    )
    .expect("hosted agent");
    Process::Agent(Box::new(machine::Rooted::new(capture.host(agent), inherited)))
}
#[must_use]
pub fn git<H: Harness>(spawn: &Spawn, inherited: &Inherited) -> Process<H> {
    H::spawned(Role::Git, inherited, b"worker");
    let route = if H::github() { Some(H::git_limits()) } else { parameters().route };
    if let Some(limits) = route {
        Process::GitRoute(Box::new(temper_fake_git::Git::new(
            spawn,
            temper_fake_git_hosted::start(inherited),
            temper_fake_git::Route::new(limits),
        )))
    } else {
        Process::Git(Box::new(temper_fake_git_hosted::git(spawn, inherited, parameters().forge)))
    }
}
#[must_use]
pub fn shell<H: Harness>(spawn: &Spawn, inherited: &Inherited) -> Process<H> {
    H::spawned(Role::Shell, inherited, b"worker/shell");
    Process::Shell(Box::new(Shell::new(spawn, inherited)))
}
#[must_use]
pub fn worker_root(_spawn: &Spawn) -> Vec<StartupRoot> {
    vec![StartupRoot { name: b"worker".as_slice().into(), path: b"worker".as_slice().into() }]
}
fn list_arguments(arguments: &[Box<[u8]>]) -> bool {
    let arguments = match arguments {
        [flag, rest @ ..] if flag.as_ref() == b"--list" => rest,
        _ => arguments,
    };
    match arguments {
        [internal, list, _] => internal.as_ref() == b"internal" && list.as_ref() == b"agent-list",
        _ => false,
    }
}
fn agent_roots<H: Harness>(spawn: &Spawn) -> Vec<StartupRoot> {
    if list_arguments(&spawn.args) { Vec::new() } else { machine::agent_roots::<H>(spawn) }
}
#[must_use]
pub fn agent_root<H: Harness>(spawn: &Spawn) -> Vec<StartupRoot> {
    if H::github() {
        vec![StartupRoot { name: b"worker".as_slice().into(), path: b"agent".as_slice().into() }]
    } else {
        worker_root(spawn)
    }
}
#[must_use]
pub fn llm_peer<H: Harness>(peer: skein_fake_peers::llm::Peer, inherited: &Inherited) -> Process<H> {
    Process::Peer(Box::new(SignalHost::new(peer, Some(inherited.signal))))
}
#[must_use]
pub fn fake_forge<H: Harness>(peer: skein_fake_peers::forge::Peer, inherited: &Inherited) -> Process<H> {
    Process::Forge(Box::new(SignalHost::new(peer, Some(inherited.signal))))
}
/// A restartable LLM peer: a power loss makes it again from the harness's provider.
#[must_use]
pub fn llm<H: Harness>(inherited: &Inherited) -> Process<H> {
    llm_peer(H::provider(parameters().seed), inherited)
}
/// A restartable forge peer on the world's forge, which git shares.
#[must_use]
pub fn forge<H: Harness>(inherited: &Inherited) -> Process<H> {
    let parameters = parameters();
    fake_forge(crate::fakes::forge_peer(&parameters.forge, H::forge_seed(parameters.seed)), inherited)
}
/// Register the agent, git (typed or routed) and scripted shell.
#[must_use]
pub fn programs<H: Harness>() -> Vec<(HostedProgram<Process<H>>, StartupRoots)> {
    vec![
        (
            HostedProgram {
                program: documents::PINNED_AGENT_PROGRAM.into(),
                make: agent::<H>,
                instances: documents::AGENTS,
                operations: (documents::agent_limits().routes + 3).max(smith_agent_shell::list::operations()),
            },
            agent_roots::<H>,
        ),
        (
            HostedProgram { program: b"/usr/bin/git".as_slice().into(), make: git::<H>, instances: 1, operations: 1 },
            worker_root,
        ),
        (
            HostedProgram { program: b"/bin/sh".as_slice().into(), make: shell::<H>, instances: 1, operations: 1 },
            agent_root::<H>,
        ),
    ]
}

#[cfg(test)]
#[path = "tests/programs.rs"]
mod tests;

#[cfg(test)]
#[path = "programs/spawn_tests.rs"]
mod spawn_tests;
