//! One process, joined through io's workers channel (protocol/README.md, 6).
use crate::{
    AccountsRuntime, End, accounts, clients, engine, settings, store,

};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{List, Queue, Time, Token, Wall};
pub(crate) const ACCOUNT_APPLICATION: &[u8] = b"temper";

/// The engine root, its protocol children and the kernel queues.
#[expect(missing_debug_implementations, reason = "the service owns account token records")]
pub struct Service {
    pub(crate) engine: Option<engine::Engine>,
    pub(crate) process: temper_service_core::process::Process,
    // Keep the generational handles even after claim: a failed restore must
    // close any listener whose component never reached its Listen.
    held_listeners: List<Token>,
    listeners_closing: u32,
    links: Queue<Box<[u8]>>,
    local_links: Queue<Box<[u8]>>,
}
impl Service {
    /// Payload-free closure and queue state at the shell's wait boundary.
    #[must_use]
    pub fn wait_state(&self) -> crate::diagnostics::Wait {
        crate::diagnostics::Wait { engine: match &self.engine { Some(engine) => Some(engine.wait_state()), None => None },
            held_listeners: self.held_listeners.len(), listener_closes: self.listeners_closing,
            signals_closing: self.process.signals_closing(), operations: self.process.has_operations(), submissions: self.process.submission_count() }
    }

    /// Check the real ring's empty, unbounded wait shape without another iteration.
    pub fn check_wait_boundary(&self, now: Time) {
        assert!(
            self.is_empty()
                || self.work_pending(now)
                || self.next_deadline().is_some()
                || self.process.has_operations()
                || self.process.submission_count() != 0,
            "service would wait forever with nothing in flight: end={:?}; state={:?}; pending={:?}",
            self.end(),
            self.wait_state(),
            self.pending(now)
        );
    }

    /// The engine domain, for a world to inspect before requesting shutdown.
    #[must_use]
    pub fn engine_domain(&self) -> Option<&temper_engine_domain::Domain> {
        match &self.engine {
            Some(engine) => Some(&engine.domain),
            None => None,
        }
    }

    /// Construct the root, its deployment store and its configured accounts.
    pub fn engine(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
    ) -> Result<Self, Problems> {
        let io = settings.jig.limits.io;
        Self::new(settings, seeds, accounts, forges, io)
    }
    /// Construct the engine on the supplied io policy.
    pub(crate) fn new(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
        io_limits: io::Limits,
    ) -> Result<Self, Problems> {
        let forges = crate::forge::prepare(&settings, forges)?;
        Self::prepared(settings, seeds, accounts, forges, io_limits, None, None)
    }
    pub(crate) fn prepared(
        settings: settings::Engine, seeds: &settings::Seeds, accounts: Option<AccountsRuntime>,
        forges: skein_forge_connection::Settings, io_limits: io::Limits, box_line_bytes: Option<u32>, program_login: Option<Box<[u8]>>,
    ) -> Result<Self, Problems> {
        let sockets = io_limits.sockets;
        let notes = settings.limits.sign_in.pending;
        let engine = engine::Engine::prepared(settings, seeds, accounts, forges, box_line_bytes, program_login)?;
        let operations = startup_operations(&io_limits).ok_or(start_problem(b"limits", What::Bound))?;
        Ok(Self { engine: Some(engine), process: temper_service_core::process::Process::new(io_limits, operations, operations),
            held_listeners: List::with_capacity(sockets), listeners_closing: 0,
            links: Queue::with_capacity(notes), local_links: Queue::with_capacity(notes) })
    }
    pub fn take_handover_read(&mut self) -> Option<crate::HandoverRead> {
        let engine = self.engine.as_mut()?;
        match engine.clients.playground_read() {
            Some(read) => Some(read),
            None => crate::setup::lifecycle::take_read(engine),
        }
    }
    pub fn handover_read(
        &mut self,
        forge: u16,
        run: Token,
        answer: Result<Option<skein_lib::Secret>, crate::HandoverUnread>,
    ) {
        if let Some(engine) = &mut self.engine {
            if engine.clients.owns_playground_read(run) {
                engine.clients.playground_read_answer(run, answer);
            } else {
                crate::setup::lifecycle::read(engine, forge, run, answer);
            }
        }
    }
    pub fn take_handover_removal(&mut self) -> Option<crate::HandoverRemoval> {
        crate::setup::lifecycle::take_removal(self.engine.as_mut()?)
    }
    pub fn handover_removed(&mut self, forge: u16, run: Token, result: Result<(), i32>) {
        if let Some(engine) = &mut self.engine {
            crate::setup::lifecycle::removed(engine, forge, run, result);
        }
    }
    /// A manifest page can be reloaded until its own setup clock or capacity ends it.
    #[must_use]
    pub fn setup_page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        self.engine.as_ref()?.setup.page(forge, state, now)
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        match &mut self.engine {
            Some(engine) => engine.store.adopt(root, effective_user),
            None => Err(root),
        }
    }
    /// Hold the bundle the binary read at start, served on the clients' listener.
    pub fn adopt_bundle(&mut self, bundle: clients::bundle::Bundle) {
        self.engine.as_mut().expect("engine bundle").clients.adopt_bundle(bundle);
    }
    /// Earliest deadline of the root and its children.
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> { self.engine.as_ref()?.next_deadline() }
    /// Whether an iteration has work on the supplied clock.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        (match &self.engine { Some(engine) => engine.work_pending(now), None => false })
            || self.process.work_pending(now)
            || self.end().is_some() && self.listeners_closing < self.held_listeners.len()
            || self.end().is_some() && self.process.signals_pending()
    }
    /// Snapshot current readiness without advancing any component or exposing payloads.
    /// Only diagnostics callers pay for walking and copying the pending event tags.
    #[must_use]
    pub fn pending(&self, now: Time) -> crate::diagnostics::Pending {
        crate::diagnostics::Pending { work_pending: self.work_pending(now),
            io_ready: self.process.io().is_ready(), io_due: self.process.io().is_due(now),
            completions: self.process.completion_count(), told: self.process.told_count(), raw_submissions: self.process.raw_submission_count(),
            ended: self.end().is_some(), signals: self.process.has_signals(), signals_closing: self.process.signals_closing(),
            engine: match &self.engine { Some(engine) => Some(engine.pending(now)), None => None } }
    }

    /// A first call drains; another aborts, keeping a failure's reason.
    pub fn shutdown(&mut self) { if let Some(engine) = &mut self.engine { engine.shutdown(); } }
    /// How this service ended, once it has.
    #[must_use]
    pub fn end(&self) -> Option<&End> {
        self.engine.as_ref().expect("joined service has an engine").end.as_ref()
    }
    /// Sent forge entries left without their outcome when this stop reached its bound.
    #[must_use]
    pub fn unsettled(&self) -> &[temper_engine_domain::Unsettled] {
        if let Some(engine) = &self.engine
            && let Some(drain) = &engine.drain
        {
            drain.unsettled.as_slice()
        } else {
            &[]
        }
    }

    /// Adopt a held listener before the first down pass (protocol/setup/box.md, 1).
    /// A Listen for its address claims it; refusal leaves the descriptor with the caller.
    pub fn adopt_listener(&mut self, fd: Fd, addr: core::net::SocketAddr) -> Result<(), Fd> {
        if self.held_listeners.room() == 0 {
            return Err(fd);
        }
        match self.process.io_mut().adopt_listener(fd, addr) {
            Ok(entity) => {
                self.held_listeners.push(entity).expect("one handle per admitted listener");
                Ok(())
            }
            Err(fd) => Err(fd),
        }
    }
    /// Adopt the shell's termination-signal source.
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        self.process.adopt_signals(descriptor)
    }
    /// Kernel completions, filled by the shell.
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        self.process.completions()
    }
    /// Kernel submissions, taken by the shell.
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        self.process.submissions()
    }
    /// Printed sign-in links, taken by the binary.
    pub const fn links(&mut self) -> &mut Queue<Box<[u8]>> {
        &mut self.links
    }
    /// Local sign-in links, taken by the binary to keep in the engine's link file, never printed.
    pub const fn local_links(&mut self) -> &mut Queue<Box<[u8]>> {
        &mut self.local_links
    }
    /// Bound listeners, taken by the binary before other diagnostics.
    pub fn listening(&mut self) -> &mut Queue<crate::Note> {
        &mut self.engine.as_mut().expect("joined service has an engine").listening
    }
    /// Operator diagnostics, taken by the binary.
    pub fn notes(&mut self) -> &mut Queue<crate::Note> { &mut self.engine.as_mut().expect("engine service").notes }
    /// Root, component and I/O deadlines on the injected clock.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.next_policy_deadline(), self.process.io().next_deadline())
    }
    /// Maximum simultaneous kernel operations, for the shell's ring.
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.process.operations()
    }
    /// Every owned child has closed and every queue and kernel route is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool { self.end().is_some() && match &self.engine { Some(engine) => engine.is_empty(), None => true } && self.process.is_empty() }
    fn route_components(&mut self) { if let Some(engine) = &mut self.engine { engine.route(&mut self.process); } }
    /// Format only the roots for a world's secrets check.
    pub fn fmt_roots(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut roots = formatter.debug_list();
        if let Some(engine) = &self.engine {
            roots.entry(&engine.domain);
        }
        
        roots.finish()
    }
    #[cfg(test)]
    pub(crate) fn publish_staged(&mut self) {
        self.process.publish();
    }
    #[cfg(test)]
    pub(crate) fn complete_staged(&mut self, complete: Complete) -> Complete {
        self.process.complete(complete)
    }
    fn complete(&mut self, complete: Complete) { match self.engine.as_mut().expect("engine completion").complete(complete) { Ok(()) => {}, Err(complete) => self.process.up(complete) } }
}
/// Core settings with the application's private tools (domain/agent/ask-temper.md, 4).
pub(crate) fn core_config(config: &jig_protocol_config::Engine, seeds: &settings::Seeds) -> jig_domain_core::Config {
    let mut core = jig_protocol_config::core(config, seeds.deployment, seeds.seed);
    let mut tools = List::with_capacity(1);
    for tool in temper_engine_smith::PRIVATE_TOOLS {
        tools.push(Box::from(tool)).expect("private tool names");
    }
    core.settings.private_tools = tools.into_boxed();
    core
}

pub(crate) fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// The accounts' driver, when the engine configures an account; its records
/// are kept in the secrets' store, at most `secret_bytes` each.
pub(crate) fn accounts_driver(
    settings: &settings::Engine,
    seeds: &settings::Seeds,
    runtime: Option<AccountsRuntime>,
    secrets: &skein_kv::Limits,
) -> Result<Option<accounts::Accounts>, Problems> {
    if settings.jig.llm.providers.is_empty() {
        return Ok(None);
    }
    Ok(Some(accounts::Accounts::new(
        ACCOUNT_APPLICATION.into(),
        &settings.jig,
        &seeds.crypto,
        &seeds.installation,
        runtime,
        secrets,
    )?))
}
/// Drive store terminals, account events, one root input and released outputs.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    for stage in STAGES {
        iterate_stage(service, stage, now, wall);
    }
}
pub use temper_service_core::{STAGES, Stage};

/// Run one stage with the pass's times (protocol/README.md, 6.1).
pub fn iterate_stage(service: &mut Service, stage: Stage, now: Time, wall: Wall) {
    match stage {
        Stage::Io => {
            service.process.clock(now, wall);
            if let Some(engine) = &mut service.engine {
                engine::clock(engine, now, wall);
            }
            service.process.resume();
            if !service.process.io().is_ready() {
                for _ in 0..service.process.completion_count() {
                    let Some(complete) = service.process.take_completion() else { break };
                    service.complete(complete);
                }
                service.process.fire();
            }
        }
        Stage::Route => {
            route_events(service);
        }
        Stage::Engine => { if let Some(engine) = &mut service.engine { engine.iterate(now, wall); } }
        Stage::Worker => {}
        Stage::Out => {
            close_sources(service);
            if let Some(engine) = &mut service.engine {
                engine.abort_if_ended();
            }
            service.route_components();
            if let Some(engine) = &mut service.engine { engine.take_links(&mut service.links, &mut service.local_links); }
            service.process.publish();
        }
    }
}
fn close_sources(service: &mut Service) {
    if service.end().is_some() {
        for _ in service.listeners_closing..service.held_listeners.len() {
            if service.process.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let entity = *service.held_listeners.get(service.listeners_closing).expect("held listener");
            service.listeners_closing = service.listeners_closing.checked_add(1).expect("one close per held listener");
            service.process.down(io::Request::Close { entity });
        }
    }
    if service.end().is_some() {
        service.process.close_signals();
    }
}

fn held_listener_closed(service: &Service, event: &io::Event) -> bool {
    let io::Event::Closed { owner } = event else { return false };
    for entity in &service.held_listeners {
        if entity == owner {
            return true;
        }
    }
    false
}
fn route_events(service: &mut Service) {
    for _ in 0..service.process.told_count() {
        let event = service.process.told().pop().expect("io event count");
        if let io::Event::Shutdown { .. } = event {
            service.shutdown();
            continue;
        }
        let owner = match &event {
            io::Event::Listening { owner, .. }
            | io::Event::Accepted { owner, .. }
            | io::Event::Connecting { owner, .. }
            | io::Event::Connected { owner }
            | io::Event::Stream { owner, .. }
            | io::Event::Output { owner, .. }
            | io::Event::Spawned { owner, .. }
            | io::Event::Exited { owner, .. }
            | io::Event::Usage { owner, .. }
            | io::Event::Window { owner, .. }
            | io::Event::Failed { owner, .. }
            | io::Event::Closed { owner } => Some(*owner),
            io::Event::Resized | io::Event::Shutdown { .. } => None,
        };
        if owner == Some(Token::new(1)) {
            if let io::Event::Closed { .. } = event {
                service.process.signals_closed();
            }
            continue;
        }
        if held_listener_closed(service, &event) {
            // An unclaimed listener's token owns Closed. Once claimed, its
            // component's owner receives that terminal through the normal route.
            continue;
        }
        if let Some(engine) = &mut service.engine { engine.up(service.process.env().now, event); }
    }
}

#[must_use]
pub fn startup_operations(io: &io::Limits) -> Option<u32> {
    temper_service_core::process::Process::operations_for(io, 0)?.checked_add(store::KERNEL_HANDOFF_ENTRIES)
}






impl Service {
    /// Start-time endpoints only; the browser and the issuer never choose a socket.
    pub fn configure_google_endpoints(&mut self, runtime: clients::fetch::Runtime) {
        if let Some(engine) = &mut self.engine {
            engine.clients.google_endpoints(runtime.endpoints);
        }
    }
}

/// Whether the deployment can set up an owned forge (temper/docs/design/client/forge/records.md).
pub(crate) const fn deployment_owned(document_owned: bool, under_box: bool) -> bool {
    document_owned && under_box
}

#[must_use]
pub fn playground_config(
    demo: Option<&settings::Demo>,
    welcome: bool,
    deadline: skein_lib::Duration,
) -> Option<temper_engine_domain_forge::PlaygroundConfig> {
    if !welcome {
        return None;
    }
    match demo {
        Some(demo) => Some(temper_engine_domain_forge::PlaygroundConfig { provider: demo.provider, deadline }),
        None => None,
    }
}

/// github.com's remotes' base (temper/docs/design/protocol/configuration/engine.md).
pub(crate) const GITHUB_REMOTE: &[u8] = b"https://github.com";

/// Machine places passed to the root (temper/docs/design/protocol/configuration/engine.md).
#[must_use]
pub fn machine_config(machine: &temper_forge_setup::machine::Machine) -> temper_engine_domain_forge::Machine {
    use temper_engine_domain_forge::{Address, Machine, Place};
    let github = match &machine.github {
        Some(github) => {
            let (host, port) = match &github.api {
                jig_protocol_config::Address::Host { name, port } => (name.clone(), *port),
                jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => {
                    (socket_host(*address), address.port())
                }
                jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                    unreachable!("checked machine socket")
                }
            };
            Place {
                api: Address { tls: if github.tls { Some(host.clone()) } else { None }, host, port },
                git: github.remote.clone(),
                web: Some(github.web.clone()),
            }
        }
        None => Place {
            api: Address {
                host: skein_lib::bytes::copy_of(b"api.github.com"),
                port: 443,
                tls: Some(skein_lib::bytes::copy_of(b"api.github.com")),
            },
            git: skein_lib::bytes::copy_of(GITHUB_REMOTE),
            web: Some(skein_lib::bytes::copy_of(b"https://github.com")),
        },
    };
    Machine {
        owned_person: match &machine.owned {
            Some(owned) => owned.person.login.clone(),
            None => Box::new([]),
        },
        owned: match &machine.owned {
            Some(owned) => Some(local_place(owned.listen, &owned.remote)),
            None => None,
        },
        fake: match &machine.fake {
            Some(fake) => Some(local_place(fake.listen, &fake.remote)),
            None => None,
        },
        github,
        github_git_program: match &machine.github {
            Some(github) => github.git.clone(),
            None => None,
        },
    }
}
#[expect(clippy::disallowed_macros, reason = "startup renders a checked IP address for the domain place")]
#[must_use]
pub fn socket_host(address: core::net::SocketAddr) -> Box<[u8]> {
    alloc::format!("{}", address.ip()).into_bytes().into_boxed_slice()
}
fn local_place(address: core::net::SocketAddr, remote: &[u8]) -> temper_engine_domain_forge::Place {
    temper_engine_domain_forge::Place {
        api: temper_engine_domain_forge::Address { host: socket_host(address), port: address.port(), tls: None },
        git: skein_lib::bytes::copy_of(remote),
        web: None,
    }
}





#[cfg(test)]
mod box_tests {
    use crate::tests::fixtures;

    #[test]
    fn machine_context_carries_only_the_owned_persons_login() {
        let mut machine = fixtures::machine();
        assert!(super::machine_config(&machine).owned_person.is_empty());
        let temper_forge_setup::setup::Mode::Owned(owned) = fixtures::owned_setup(1).entry.mode else {
            panic!("owned machine fixture")
        };
        machine.owned = Some(temper_forge_setup::machine::OwnedMachine {
            sign_in: None,
            listen: owned.listen,
            remote: b"http://forge.example".as_slice().into(),
            person: owned.person,
            public: owned.public,
            fetch: owned.fetch,
            forgejo: owned.forgejo,
            runner: owned.runner,
        });
        assert_eq!(super::machine_config(&machine).owned_person.as_ref(), b"owner");
    }

}

impl Service {
    pub fn engine_component(&mut self) -> &mut engine::Engine { self.engine.as_mut().expect("engine service") }
    pub fn configure_forge_runtime(&mut self, github: crate::ForgeRuntime, roots: skein_tls::Config) { self.engine_component().configure_forge_runtime(github, roots); }
    #[must_use]
    #[expect(clippy::type_complexity, reason = "startup resolution returns numbered host and port tuples")]
    pub fn unresolved(&self) -> Box<[(u16, Box<[u8]>, u16)]> { self.engine.as_ref().expect("engine service").unresolved() }
    pub fn resolved(&mut self, forge: u16, address: Option<core::net::SocketAddr>) { self.engine_component().resolved(forge, address); }
}
