//! The engine component seam (temper/docs/design/protocol/services.md, 4).
use crate::system::{accounts_driver, core_config, deployment_owned, machine_config, playground_config, start_problem};
use crate::{AccountsRuntime, accounts, clients, engine, limits, settings, store, values, workers};
use crate::{End, ForgeRuntime, Stop, forge, forges, route::box_link};
use alloc::boxed::Box;
use jig_protocol_config::{Problems, What};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{List, Queue, Time, Token};
impl engine::Engine {
    pub fn prepared(
        mut settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: skein_forge_connection::Settings,
        box_line_bytes: Option<u32>,
        program_login: Option<Box<[u8]>>,
    ) -> Result<Self, Problems> {
        let mut limits =
            limits::engine(&settings.jig.limits, &settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        limits.memory_bytes = settings.jig.memory;
        limits.host_reply_bytes = settings.host_reply_bytes;
        if !temper_engine_domain::valid(&limits) {
            return Err(start_problem(b"memory_bytes", What::Bound));
        }
        let mut forge_settings = forges;
        let shared = forge::shared(&settings.limits, &settings.jig.limits.io);
        settings.jig.limits.clients.wire.keyed_value_some =
            limits::client_value_bytes(&settings.jig.limits, &settings.limits);
        let store_limits = store::limits(&limits, &limits.forge, &settings.jig.limits.store)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let accounts = accounts_driver(&settings, seeds, accounts, &store_limits.secrets)?;
        let mut values =
            values::Values::new(settings.jig.limits.driver.accounts, values::credential_bytes(&settings.jig));
        values.fill(&settings);
        let (forge_config, identities) = forge::config(&settings, seeds.deployment);
        let mut first_periods = List::with_capacity(limits.authority.projects);
        for project in &settings.jig.projects {
            first_periods
                .push((project.number, project.policy.period_spend))
                .or(Err(start_problem(b"projects", What::Bound)))?;
        }
        let mut clients = clients::Clients::new(
            &settings.directories,
            &settings.jig,
            &settings.clients,
            seeds,
            &limits.journal,
            settings.sign_in.as_ref(),
            settings.machine.owned.as_ref(),
            settings.files.entries.as_slice(),
        )?;
        clients.program_environment.clone_from(&settings.program_environment);
        clients.deployment = crate::family::Deployment {
            program_login,
            welcome: settings.welcome,
            owned: deployment_owned(settings.machine.owned.is_some(), box_line_bytes.is_some()),
        };
        if let Some(sign_in) = &settings.sign_in {
            clients.configure_google(sign_in, settings.limits.sign_in, seeds, &settings.jig);
        }
        clients.configure_forge_sign_in(&settings);
        let setup = crate::setup::prepare(&settings, &mut forge_settings, &seeds.crypto, box_line_bytes.is_some())?;
        let mut forge = forge::Layer::new(forge_settings, settings.limits.forge_component, &shared, &[])
            .expect("checked forge limits");
        forge.log_origin_limit = settings.limits.forges.log_origins;
        let notes = Queue::with_capacity(settings.limits.forge.forges);
        let workers = workers::Workers::configured(&mut settings, seeds.deployment)?;
        let mut core = core_config(&settings.jig, seeds);
        core.kinds = crate::stock::kinds(&limits, *core.connectors.first().expect("checked forge connector"));
        core.jobs = limits::jobs(&settings.jig.limits);
        let config = temper_engine_domain::Config {
            host_part: settings.host_part,
            kinds_stock: settings.kinds_stock,
            ways_stock: settings.ways_stock,
            period_span: crate::stock::PERIOD_SPAN,
            core,
            landings: settings.landings,
            forge: forge_config,
            identities,
            machine: Some(machine_config(&settings.machine)),
            forges: Box::new([]),
            under_box: box_line_bytes.is_some(),
            playground: playground_config(settings.demo.as_ref(), settings.welcome, settings.limits.setup.deadline),
        };
        let store = store::Store::new(&limits, &store_limits, &config.core.settings.models, seeds.installation);
        let demo = settings.demo;
        let own = settings.limits;
        let mut engine = engine::Engine::new(
            config,
            &limits,
            store,
            engine::Children { accounts, workers, clients, demo, forge, setup, notes, limits: own },
            values,
            first_periods,
        )?;
        if let Some(line_bytes) = box_line_bytes {
            engine.box_link = box_link::Link::new(line_bytes, false);
        }
        Ok(engine)
    }
    pub fn take_handover_read(&mut self) -> Option<crate::HandoverRead> {
        let engine = &mut *self;
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
        {
            let engine = &mut *self;
            if engine.clients.owns_playground_read(run) {
                engine.clients.playground_read_answer(run, answer);
            } else {
                crate::setup::lifecycle::read(engine, forge, run, answer);
            }
        }
    }
    pub fn take_handover_removal(&mut self) -> Option<crate::HandoverRemoval> {
        crate::setup::lifecycle::take_removal(self)
    }
    pub fn handover_removed(&mut self, forge: u16, run: Token, result: Result<(), i32>) {
        {
            let engine = &mut *self;
            crate::setup::lifecycle::removed(engine, forge, run, result);
        }
    }
    #[must_use]
    pub fn setup_page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        self.setup.page(forge, state, now)
    }
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        self.store.adopt(root, effective_user)
    }
    pub fn adopt_bundle(&mut self, bundle: clients::bundle::Bundle) {
        self.clients.adopt_bundle(bundle);
    }
    #[must_use]
    pub fn unsettled(&self) -> &[temper_engine_domain::Unsettled] {
        if let Some(drain) = &self.drain { drain.unsettled.as_slice() } else { &[] }
    }
    pub fn configure_google_endpoints(&mut self, runtime: clients::fetch::Runtime) {
        {
            let engine = &mut *self;
            engine.clients.google_endpoints(runtime.endpoints);
        }
    }
}
impl engine::Engine {
    /// Machine inputs loaded once by the shell (temper/docs/design/protocol/configuration/engine.md).
    pub fn configure_forge_runtime(&mut self, github: ForgeRuntime, roots: skein_tls::Config) {
        {
            let engine = &mut *self;
            assert!(engine.forge.serving.is_none(), "machine inputs precede serving");
            engine.forge.github = Some(github);
            engine.forge.roots = Some(roots);
        }
    }

    /// Kept names awaiting the shell's startup resolution (temper/docs/design/protocol/forge/engine.md, 2).
    #[must_use]
    #[expect(clippy::type_complexity, reason = "the startup resolution contract returns numbered host and port tuples")]
    pub fn unresolved(&self) -> Box<[(u16, Box<[u8]>, u16)]> {
        let engine = self;
        if engine.forge.serving.is_some() {
            return Box::new([]);
        }
        let mut names = List::with_capacity(engine.workers.forges.len());
        for forge in engine.workers.forges.iter() {
            if engine.forge.resolving.contains_key(&forge.number)
                && let jig_protocol_config::Address::Host { name, port } = &forge.api
            {
                names.push((forge.number, name.clone(), *port)).expect("kept forges");
            }
        }
        names.into_boxed()
    }

    /// Re-add a kept forge with the shell's answer before serving (temper/docs/design/protocol/forge/engine.md, 2).
    pub fn resolved(&mut self, forge: u16, address: Option<core::net::SocketAddr>) {
        let engine = self;
        if engine.forge.serving.is_some() || engine.forge.unreachable(forge).is_none() {
            return;
        }
        engine.forge.resolving.remove(&forge);
        let Some(address) = address else { return };
        let Some(configured) = engine.workers.forges.remove(forge) else { return };
        engine.forge.down(skein_forge_connection::Request::Remove { forge });
        if let jig_protocol_config::Address::Host { name, .. } = &configured.api {
            engine.forge.names.insert(name.clone(), address.ip()).expect("kept host capacity");
        }
        let Some(mut endpoint) = forges::endpoint(engine, &configured) else {
            assert!(engine.workers.forges.insert(configured), "removed forge has table room");
            return;
        };
        endpoint.address = skein_io::Addr::Ip(address);
        engine.forge.endpoints.insert(forge, endpoint).expect("kept forge capacity");
        if forges::keep(engine, configured) {
            engine.setup.replace_endpoint(forge, address);
            if engine.forge_records.done() {
                forge::records::connect(engine, forge);
            }
        }
    }
}

impl engine::Engine {
    pub fn route(&mut self, process: &mut temper_service_core::process::Process) {
        let engine = self;

        for _ in 0..engine.store.submitted.len() {
            if process.room() == 0 {
                break;
            }
            process.raw_submissions().push(engine.store.submitted.pop().expect("store submissions"));
        }
        if engine.store.opened || engine.end.is_some() {
            for _ in 0..engine.setup.io.len() {
                if process.room() < io::MAX_OUT_DOWN.submissions || !process.io().takes() {
                    break;
                }
                let request = engine.setup.io.pop().expect("setup IO requests");
                process.down(crate::setup::tag(request));
            }
            for _ in 0..engine.forge.io.len() {
                if process.room() < io::MAX_OUT_DOWN.submissions || !process.io().takes() {
                    break;
                }
                let request = engine.forge.io.pop().expect("forge io requests");
                process.down(forge::tag(request));
            }
            for _ in 0..engine.clients.io.len() {
                if process.room() < io::MAX_OUT_DOWN.submissions || !process.io().takes() {
                    break;
                }
                let request = engine.clients.io.pop().expect("client io requests");
                process.down(request);
            }
            for _ in 0..engine.workers.host.io.len() {
                if process.room() < io::MAX_OUT_DOWN.submissions || !process.io().takes() {
                    break;
                }
                let request = engine.workers.host.io.pop().expect("worker io requests");
                process.down(request);
            }
        }
        if (engine.store.opened || engine.end.is_some())
            && let Some(accounts) = &mut engine.accounts
        {
            for _ in 0..accounts.io.len() {
                if process.room() < io::MAX_OUT_DOWN.submissions || !process.io().takes() {
                    break;
                }
                let request = accounts::tag(accounts.io.pop().expect("account io requests"));
                process.down(request);
            }
        }
    }
    pub fn up(&mut self, now: Time, event: io::Event) {
        let engine = self;
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
        let error = if let io::Event::Failed { error, .. } = &event { Some(*error) } else { None };
        if let Some(owner) = owner
            && crate::tokens::prefix(owner) == 11
        {
            engine.setup.up(event);
        } else if let Some(owner) = owner
            && crate::tokens::prefix(owner) == 5
        {
            engine.forge.up(event);
        } else if let Some(owner) = owner
            && clients::owns(owner)
        {
            if !engine.clients.up(event) && engine.end.is_none() {
                engine.end = Some(End::Stopped(Stop::Listener {
                    listener: crate::Listening::Clients,
                    address: Some(engine.clients.address),
                    error,
                }));
            }
        } else if let Some(owner) = owner
            && accounts::owns(owner)
        {
            if let Some(accounts) = &mut engine.accounts {
                accounts.up(accounts::untag(event));
            }
        } else if !engine.workers.up(now, event, &mut engine.events) && engine.end.is_none() {
            let address = engine.workers.host.listener_address(owner);
            engine.workers.notes.push(crate::Note::WorkerListenerFailed { address, error });
            engine.end = Some(End::Stopped(Stop::Listener { listener: crate::Listening::Workers, address, error }));
        }
        engine.drain_worker_notes();
        for (listener, addr) in [
            (crate::Listening::Clients, engine.clients.listening.take()),
            (crate::Listening::Workers, engine.workers.host.listening.take()),
        ] {
            if let Some(addr) = addr {
                engine
                    .listening
                    .try_push(crate::Note::Listening { listener, addr })
                    .expect("one note per listener bind");
            }
        }
    }
    #[must_use]
    pub fn wait_state(&self) -> crate::diagnostics::WaitEngine {
        use crate::diagnostics::{WaitEngine, WaitOwner};
        let engine = self;
        WaitEngine {
            restart: engine.domain.start_phase(),
            mark_durable: engine.store.host.mark_durable(),
            store_closed: engine.store.closed(),
            clients: WaitOwner { closed: engine.clients.closed, io: engine.clients.io.len() },
            workers: WaitOwner { closed: engine.workers.host.closed(), io: engine.workers.host.io.len() },
            setup: WaitOwner { closed: engine.setup.closed, io: engine.setup.io.len() },
            setup_above: engine.setup.above.len(),
            forge: WaitOwner { closed: engine.forge.closed, io: engine.forge.io.len() },
            forge_above: engine.forge.above.len(),
            accounts: match &engine.accounts {
                Some(accounts) => Some(accounts::wait_state(accounts, engine.env.now)),
                None => None,
            },
        }
    }

    #[must_use]
    pub fn end(&self) -> Option<&End> {
        self.end.as_ref()
    }
    #[must_use]
    pub fn domain(&self) -> &temper_engine_domain::Domain {
        &self.domain
    }
    pub fn shutdown(&mut self) {
        crate::stop::shutdown(self);
    }
    pub fn agents_ended(&mut self, ended: bool) {
        if let Some(drain) = &mut self.drain {
            drain.agents_ended = ended;
        }
    }
    #[must_use]
    pub fn draining(&self) -> bool {
        match &self.drain {
            Some(drain) => drain.closing,
            None => false,
        }
    }
    #[must_use]
    pub fn ready_for_worker(&self) -> bool {
        self.store.opened && self.workers.host.opened() && self.clients.opened()
    }
    #[must_use]
    pub fn stopping(&self) -> bool {
        self.drain.is_some() || self.end.is_some()
    }
    pub fn complete(&mut self, complete: io::kernel::Complete) -> Result<(), io::kernel::Complete> {
        if store::owns(complete.op) {
            self.store.complete(complete);
            Ok(())
        } else {
            Err(complete)
        }
    }
    pub fn notes(&mut self) -> &mut Queue<crate::Note> {
        &mut self.notes
    }
    pub fn listening(&mut self) -> &mut Queue<crate::Note> {
        &mut self.listening
    }
    #[must_use]
    pub fn notes_room(&self) -> u32 {
        self.notes.room()
    }
    #[must_use]
    pub fn notes_capacity(&self) -> u32 {
        self.notes.capacity()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.setup.closed
            && self.setup.io.is_empty()
            && self.setup.above.is_empty()
            && self.forge.closed
            && self.forge.io.is_empty()
            && self.forge.above.is_empty()
            && self.store.closed()
            && self.workers.host.closed()
            && self.workers.notes.is_empty()
            && self.clients.closed
            && self.clients.io.is_empty()
            && match &self.accounts {
                Some(accounts) => accounts.idle(),
                None => true,
            }
    }
    pub fn take_links(&mut self, links: &mut Queue<Box<[u8]>>, local_links: &mut Queue<Box<[u8]>>) {
        if self.clients.starting.is_none() {
            for _ in 0..self.clients.links.len() {
                if links.room() == 0 {
                    break;
                }
                links.push(self.clients.links.pop().expect("printed sign-in links"));
            }
            for _ in 0..self.clients.local_links.len() {
                if local_links.room() == 0 {
                    break;
                }
                local_links.push(self.clients.local_links.pop().expect("local sign-in links"));
            }
        }
    }
    pub fn box_link(&mut self, bytes: u32, adopted: bool) {
        self.box_link = box_link::Link::new(bytes, adopted);
    }
    pub fn box_read(&mut self, bytes: &[u8]) {
        let mut out = Queue::with_capacity(3);
        self.box_link.read(bytes, &mut out);
        box_link::drain(self, &mut out);
    }
    pub fn box_closed(&mut self) {
        let mut out = Queue::with_capacity(3);
        self.box_link.closed(&mut out);
        box_link::drain(self, &mut out);
    }
    #[must_use]
    pub fn box_writing(&self) -> bool {
        self.box_link.write.is_some()
    }
    pub fn box_write(&mut self) -> Option<Box<[u8]>> {
        self.box_link.write.take()
    }
    pub fn setup_state(&mut self, number: u16, state: temper_engine_domain_forge::Connection) {
        crate::setup::lifecycle::state(self, number, state);
    }
}
