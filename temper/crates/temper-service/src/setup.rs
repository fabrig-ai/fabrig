//! Durable setup needs and held manifest pages (protocol/forge.md, 2.6).
pub(crate) mod diagnostics;
pub(crate) mod lifecycle;
pub(crate) mod public;
mod routes;
pub(crate) use routes::{MANIFEST_SCRIPT_HASH, MANIFEST_STYLE_HASH, PAGE_STYLE_HASH, busy};
pub(crate) mod settings;
use alloc::boxed::Box;
use jig_protocol_store::Event as StoreEvent;
use lifecycle::{Kind, Purpose, Run};
pub(crate) use settings::prepare;
use skein_forge_setup::{host, reconcile};
use skein_lib::{CryptoRng, Duration, Env, Map, Queue, Secret, Time, Token, Wall};
use temper_engine_domain as root;
use temper_protocol_records::Item;

/// Plans are staged in the protocol layer: the root names them, never their credentials.
#[expect(clippy::struct_excessive_bools, reason = "the setup layer retains independent lifecycle and listener facts")]
pub(crate) struct Layer {
    component: Option<host::Host>,
    pending: Option<(host::Settings, CryptoRng)>,
    pub(crate) context: Option<settings::Context>,
    waiting_for_webhooks: bool,
    webhooks_bound: bool,
    pub(crate) env: Env<host::Limits>,
    pub(crate) above: Queue<host::Event>,
    pub(crate) io: Queue<skein_io::Request>,
    pub(crate) closed: bool,
    closing: bool,
    served: Map<u16, ()>,
    unserved: Map<u16, &'static [u8]>,
    pub(crate) staged: Map<Token, host::Plan>,
    runs: Map<Token, Run>,
    public: Map<Token, public::Caller>,
    backs: Queue<public::ReturnTo>,
    pastes: Map<u64, u16>,
    file_api: Map<u16, Secret>,
    erases: Map<Token, u16>,
    pub(crate) lifecycle: lifecycle::Lifecycle,
    writes: Map<Token, Write>,
    next_write: u64,
    pages: Queue<Page>,
    waiting: Queue<Waiting>,
    held: Queue<Held>,
    listener: Box<[u8]>,
    life: Duration,
    value_bytes: u32,
    local_limits: skein_forge_setup::limits::Limits,
    under_box: bool,
    #[cfg(test)]
    pub(crate) answers: Queue<(Token, host::Answer)>,
}
struct Write {
    purpose: Purpose,
    number: u64,
    value: Secret,
}
struct Page {
    forge: u16,
    state: Box<[u8]>,
    bytes: Box<[u8]>,
    until: Time,
}
/// GitHub's setup visit for a forge busy with its one run, held until that run ends (client/forge.md, 9.1).
struct Waiting {
    forge: u16,
    call: Token,
    installation: core::num::NonZeroU64,
    state: Box<[u8]>,
    back: Option<Box<[u8]>>,
    until: Time,
}
/// A person's Install that met only the engine's own run on its forge, held until that run ends
/// (protocol/setup.md, 5.1): the forge's one person's request, so at most one per forge.
struct Held {
    run: Token,
    forge: u16,
    until: Time,
}

impl Layer {
    /// Retain machine-wide settings after checking the document's seed.
    pub(crate) fn empty(&mut self, config: &crate::settings::Engine, under_box: bool) {
        let bound = self.env.limits.forges;
        if let Some((settings, _)) = &mut self.pending {
            settings.forges = skein_lib::List::with_capacity(bound);
        }
        self.served = Map::with_capacity(bound);
        self.unserved = Map::with_capacity(bound);
        self.file_api = Map::with_capacity(bound);
        self.lifecycle = lifecycle::Lifecycle::new(bound, config.limits.forges.installations);
        self.local_limits = config.limits.setup.plan();
        self.local_limits.repositories = config.limits.forge.client.repositories;
        self.under_box = under_box;
        self.webhooks_bound = config.webhooks.is_none();
    }
    pub(crate) fn hold_record(
        &mut self,
        record: &temper_engine_domain_forge::Forge,
        source: &crate::settings::Forge,
        mut identity: crate::settings::Identity,
        handover: Option<Secret>,
        restored: bool,
    ) -> Result<(), crate::forge::table::NotKept> {
        if record.files.is_some() {
            if let Some(entry) = crate::forge::table::setup_entry(record)
                && let temper_forge_setup::setup::Mode::Fake(fake) = entry.mode
                && let Some(access) = crate::forge::file_access(source)
            {
                self.lifecycle
                    .adoptions
                    .insert(
                        record.number,
                        lifecycle::Adoption {
                            next: core::num::NonZeroU64::new(1).expect("first repository"),
                            repositories: u64::try_from(fake.repositories.len()).expect("bounded repositories"),
                            files: Some(lifecycle::AdoptionFiles {
                                admin_token: handover.expect("checked fake handover"),
                                access,
                            }),
                        },
                    )
                    .expect("admitted forge");
            }
            return Ok(());
        }
        let Some(entry) = crate::forge::table::setup_entry(record) else { return Ok(()) };
        let managed = match &entry.mode {
            temper_forge_setup::setup::Mode::Managed { .. } => true,
            temper_forge_setup::setup::Mode::Owned(_) | temper_forge_setup::setup::Mode::Fake(_) => false,
        };
        if identity.login.is_empty() {
            identity.login.clone_from(&entry.bot.login);
        }
        let local = lifecycle::Local {
            source: if managed { lifecycle::Source::Managed } else { lifecycle::Source::Prepared },
            first: None,
            repositories: match &entry.mode {
                temper_forge_setup::setup::Mode::Fake(fake) => {
                    u64::try_from(fake.repositories.len()).expect("bounded repositories")
                }
                temper_forge_setup::setup::Mode::Managed { .. } | temper_forge_setup::setup::Mode::Owned(_) => 0,
            },
            pending: true,
            restored,
            limits: self.local_limits.clone(),
            webhooks: match &self.context {
                Some(context) => context.webhooks.is_some() && !managed,
                None => false,
            },
            bot_login: entry.bot.login.clone(),
            identity,
            handover,
            path: temper_forge_setup::place::Place { directory: entry.directory, user: Box::new([]) }
                .path(temper_forge_setup::place::File::AdminToken),
            file: lifecycle::FileStep::Idle,
            trigger: if record.playground && self.under_box {
                lifecycle::Trigger::StartDemo
            } else if entry.asked {
                lifecycle::Trigger::StartAsked
            } else {
                lifecycle::Trigger::Start
            },
        };
        self.lifecycle.add_forge(record.number, lifecycle::Reconcile::Local(local))
    }
    /// Add a forge to the pending settings or the running host.
    pub(crate) fn add_forge(
        &mut self,
        number: u16,
        forge: Result<host::Forge, &'static [u8]>,
        file_api: Option<Secret>,
    ) -> Result<(), crate::forge::table::NotKept> {
        use crate::forge::table::NotKept;
        if self.closing || self.closed {
            return Err(NotKept::Started);
        }
        if self.served.contains_key(&number) || self.unserved.contains_key(&number) {
            return Ok(());
        }
        if self.served.len().checked_add(self.unserved.len()).ok_or(NotKept::Bound)? >= self.env.limits.forges {
            return Err(NotKept::Bound);
        }
        match forge {
            Ok(forge) => {
                if forge.number() != number {
                    return Err(NotKept::Unresolved);
                }
                let listener = match &forge.kind {
                    host::Kind::GitHub(given) => Some(given.url.clone()),
                    host::Kind::Forgejo(_) => None,
                };
                let wait = match &forge.kind {
                    host::Kind::Forgejo(given) => given.listener.is_some() && !self.webhooks_bound,
                    host::Kind::GitHub(_) => false,
                };
                if let Some((settings, _)) = &mut self.pending {
                    settings.forges.push(forge).or(Err(NotKept::Bound))?;
                } else if let Some(host) = &mut self.component {
                    if let Err(why) = host.add(&self.env.limits, forge) {
                        return Err(NotKept::Host(why));
                    }
                } else {
                    return Err(NotKept::Started);
                }
                if let Some(listener) = listener {
                    self.listener = listener;
                }
                if self.pending.is_some() && wait {
                    self.waiting_for_webhooks = true;
                }
                self.served.insert(number, ()).expect("admitted setup forge");
            }
            Err(reason) => {
                self.unserved.insert(number, reason).expect("admitted setup forge");
            }
        }
        if let Some(token) = file_api {
            self.file_api.insert(number, token).expect("admitted setup forge");
        }
        Ok(())
    }
    /// Drop the forge's settings, runs and callbacks at once.
    #[expect(clippy::too_many_lines, reason = "removal clears every forge association and queued output together")]
    pub(crate) fn remove_forge(&mut self, forge: u16) {
        if let Some((settings, _)) = &mut self.pending {
            let mut kept = skein_lib::List::with_capacity(settings.forges.capacity());
            for held in core::mem::replace(&mut settings.forges, skein_lib::List::with_capacity(0)).into_boxed() {
                if held.number() != forge {
                    kept.push(held).expect("configured capacity");
                }
            }
            settings.forges = kept;
        }
        if let Some(host) = &mut self.component {
            host.remove(forge);
        }
        self.served.remove(&forge);
        self.unserved.remove(&forge);
        self.file_api.remove(&forge);
        self.lifecycle.drop_forge(forge);
        let mut runs = skein_lib::List::with_capacity(self.runs.capacity());
        for (token, run) in &self.runs {
            if run.forge == forge {
                runs.push(*token).expect("run capacity");
            }
        }
        for _ in 0..self.above.len() {
            let event = self.above.pop().expect("host output count");
            let removed = match &event {
                host::Event::Need { run, .. } | host::Event::Done { run, .. } | host::Event::Stop { run, .. } => {
                    runs.as_slice().contains(run)
                }
                host::Event::Closed => false,
            };
            if !removed {
                self.above.push(event);
            }
        }
        for token in runs.into_boxed() {
            self.runs.remove(&token);
            self.staged.remove(&token);
            self.public.remove(&token);
        }
        let mut staged = Map::with_capacity(self.staged.capacity());
        let mut previous = core::mem::replace(&mut self.staged, Map::with_capacity(0));
        while let Some((token, _)) = previous.first() {
            let token = *token;
            let plan = previous.remove(&token).expect("held entry");
            if self::forge(&plan) != forge {
                staged.insert(token, plan).expect("plan capacity");
            }
        }
        self.staged = staged;
        let mut public = Map::with_capacity(self.public.capacity());
        let mut previous = core::mem::replace(&mut self.public, Map::with_capacity(0));
        while let Some((token, _)) = previous.first() {
            let token = *token;
            let caller = previous.remove(&token).expect("held entry");
            let (public::Caller::Manifest { forge: number, .. }
            | public::Caller::Redirect { forge: number, .. }
            | public::Caller::Connect { forge: number, .. }
            | public::Caller::Install { forge: number, .. }
            | public::Caller::AcceptAsked { forge: number, .. }
            | public::Caller::Accept { forge: number, .. }
            | public::Caller::Reachable { forge: number, .. }) = &caller;
            if *number != forge {
                public.insert(token, caller).expect("caller capacity");
            }
        }
        self.public = public;
        let mut pastes = Map::with_capacity(self.pastes.capacity());
        let mut previous = core::mem::replace(&mut self.pastes, Map::with_capacity(0));
        while let Some((token, _)) = previous.first() {
            let token = *token;
            let number = previous.remove(&token).expect("held entry");
            if number != forge {
                pastes.insert(token, number).expect("paste capacity");
            }
        }
        self.pastes = pastes;
        let mut erases = Map::with_capacity(self.erases.capacity());
        let mut previous = core::mem::replace(&mut self.erases, Map::with_capacity(0));
        while let Some((token, _)) = previous.first() {
            let token = *token;
            let number = previous.remove(&token).expect("held entry");
            if number != forge {
                erases.insert(token, number).expect("erase capacity");
            }
        }
        self.erases = erases;
        let mut writes = Map::with_capacity(self.writes.capacity());
        let mut previous = core::mem::replace(&mut self.writes, Map::with_capacity(0));
        while let Some((token, _)) = previous.first() {
            let token = *token;
            let write = previous.remove(&token).expect("held entry");
            if match temper_protocol_records::parse(write.number) {
                Some((_, number, _)) => number != forge,
                None => true,
            } {
                assert!(writes.insert(token, write).is_ok(), "write capacity");
            }
        }
        self.writes = writes;
        for _ in 0..self.pages.len() {
            let value = self.pages.pop().expect("page count");
            if value.forge != forge {
                self.pages.push(value);
            }
        }
        for _ in 0..self.waiting.len() {
            let value = self.waiting.pop().expect("waiting count");
            if value.forge != forge {
                self.waiting.push(value);
            }
        }
        for _ in 0..self.held.len() {
            let value = self.held.pop().expect("held count");
            if value.forge != forge {
                self.held.push(value);
            }
        }
        for _ in 0..self.backs.len() {
            let value = self.backs.pop().expect("back count");
            if value.forge != forge {
                self.backs.push(value);
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn startup_restored(&self) -> bool {
        self.lifecycle.startup_restored()
    }
    /// Consume the listener's typed bind result before any setup plan can copy Given.
    pub(crate) fn webhooks_listening(&mut self, addr: core::net::SocketAddr) {
        if let Some(context) = &mut self.context {
            context.webhooks = Some(skein_io::Addr::Ip(addr));
        }
        if let Some((settings, _)) = &mut self.pending {
            for index in 0..settings.forges.len() {
                if let host::Kind::Forgejo(given) = &mut settings.forges.get_mut(index).expect("configured forge").kind
                    && given.listener.is_some()
                {
                    given.listener = Some(addr);
                }
            }
        }
        self.waiting_for_webhooks = false;
        self.webhooks_bound = true;
    }
    pub(crate) fn adopting(&self, forge: u16) -> bool {
        self.lifecycle.adoptions.contains_key(&forge)
    }
    pub(crate) fn records_only(&self, forge: u16) -> bool {
        self.lifecycle.local(forge).is_some()
    }
    pub(crate) fn new(
        settings: host::Settings,
        limits: host::Limits,
        crypto: CryptoRng,
        life: Duration,
        value_bytes: u32,
        installations: u32,
    ) -> Self {
        let mut listener: Box<[u8]> = Box::new([]);
        let mut served = Map::with_capacity(limits.forges);
        for forge in &settings.forges {
            served.insert(forge.number(), ()).expect("configured host forges");
            if let host::Kind::GitHub(given) = &forge.kind {
                listener.clone_from(&given.url);
            }
        }
        Self {
            listener,
            component: None,
            pending: Some((settings, crypto)),
            context: None,
            waiting_for_webhooks: false,
            webhooks_bound: false,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::uncapped(),
            io: Queue::uncapped(),
            closed: false,
            closing: false,
            served,
            unserved: Map::with_capacity(limits.forges),
            staged: Map::with_capacity(limits.runs),
            runs: Map::with_capacity(limits.runs),
            public: Map::with_capacity(limits.runs),
            backs: Queue::with_capacity(limits.states),
            pastes: Map::with_capacity(limits.forges),
            file_api: Map::with_capacity(limits.forges),
            erases: Map::with_capacity(limits.forges),
            lifecycle: lifecycle::Lifecycle::new(limits.forges, installations),
            writes: Map::with_capacity(limits.runs.checked_add(limits.forges).expect("setup writes")),
            next_write: 1,
            pages: Queue::with_capacity(limits.states),
            waiting: Queue::with_capacity(limits.states),
            held: Queue::with_capacity(limits.forges),
            life,
            value_bytes,
            local_limits: crate::limits::profile(crate::limits::Profile::Small).setup.plan(),
            under_box: false,
            #[cfg(test)]
            answers: Queue::with_capacity(64),
        }
    }
    pub(crate) fn down(&mut self, request: host::Request) {
        #[cfg(test)]
        if let host::Request::Answer { run, answer } = &request {
            self.answers.push((*run, answer.clone()));
        }
        self.component.as_mut().expect("host started after records load").down(
            &self.env,
            request,
            &mut self.above,
            &mut self.io,
        );
    }
    pub(crate) fn up(&mut self, event: skein_io::Event) {
        self.component.as_mut().expect("host owns IO").up(
            &self.env,
            crate::accounts::untag(event),
            &mut self.above,
            &mut self.io,
        );
    }
    pub(crate) fn close(&mut self) {
        if self.closing || self.closed {
            return;
        }
        self.closing = true;
        self.lifecycle.adoptions = Map::with_capacity(self.env.limits.forges);
        self.lifecycle.reconcile = Map::with_capacity(self.env.limits.forges);
        self.pages = Queue::with_capacity(self.env.limits.states);
        self.waiting = Queue::with_capacity(self.env.limits.states);
        self.held = Queue::with_capacity(self.env.limits.forges);
        self.backs = Queue::with_capacity(self.env.limits.states);
        self.public = Map::with_capacity(self.env.limits.runs);
        self.file_api = Map::with_capacity(self.env.limits.forges);
        self.staged = Map::with_capacity(self.env.limits.runs);
        if self.component.is_some() {
            self.down(host::Request::Close);
        } else {
            self.pending = None;
            self.closed = true;
        }
    }
    #[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let host = match &self.component {
            Some(host) => host.next_deadline(),
            None => None,
        };
        let mut next = crate::accounts::earliest(
            host,
            match self.pages.iter().next() {
                Some(page) => Some(page.until),
                None => None,
            },
        );
        for waiting in &self.waiting {
            next = crate::accounts::earliest(next, Some(waiting.until));
        }
        for held in &self.held {
            next = crate::accounts::earliest(next, Some(held.until));
        }
        next
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        !self.above.is_empty() || !self.io.is_empty() || due(self.next_deadline(), now) || self.waiting_ready()
    }
    fn waiting_ready(&self) -> bool {
        for waiting in &self.waiting {
            if !self.busy(waiting.forge) {
                return true;
            }
        }
        for held in &self.held {
            if !self.busy(held.forge) {
                return true;
            }
        }
        false
    }
    /// The forge's one run: a live run of it, or its removal (protocol/setup.md, 6.5).
    pub(crate) fn busy(&self, forge: u16) -> bool {
        if self.lifecycle.deleting.contains_key(&forge) {
            return true;
        }
        for (_, run) in &self.runs {
            if run.forge == forge && run.kind != Kind::Reachable {
                return true;
            }
        }
        false
    }
    pub(super) fn reading(&self, forge: u16) -> bool {
        for (_, run) in &self.runs {
            if run.forge == forge && run.kind == Kind::Reachable {
                return true;
            }
        }
        false
    }
    /// Busy with the engine's own runs alone: the reconcile after a connect, at a start or on a hint.
    pub(super) fn busy_internally(&self, forge: u16) -> bool {
        if self.lifecycle.deleting.contains_key(&forge) {
            return false;
        }
        let mut any = false;
        for (_, run) in &self.runs {
            if run.forge == forge && run.kind != Kind::Reachable {
                if !run.internal {
                    return false;
                }
                any = true;
            }
        }
        any
    }
    pub(crate) fn lifecycle_ready(&self, engine: &crate::engine::Engine) -> bool {
        !self.closing
            && self.component.is_some()
            && self.lifecycle.ready(&self.runs, self.env.limits.runs, &engine.domain)
    }
    /// A served page stays available for reload, until its own clock/count evicts it.
    pub(crate) fn page(&self, forge: u16, state: &[u8], now: Time) -> Option<&[u8]> {
        for page in &self.pages {
            if page.forge == forge && page.state.as_ref() == state && now < page.until {
                return Some(&page.bytes);
            }
        }
        None
    }
    fn take_page(&mut self, forge: u16, state: &[u8], now: Time) -> bool {
        let mut found = false;
        for _ in 0..self.pages.len() {
            let page = self.pages.pop().expect("page count");
            if page.until <= now {
                continue;
            }
            if page.forge == forge && page.state.as_ref() == state {
                found = true;
            } else {
                self.pages.push(page);
            }
        }
        found
    }
    fn hold(&mut self, forge: u16, state: Box<[u8]>, bytes: Box<[u8]>) {
        if self.pages.room() == 0 {
            let _oldest = self.pages.pop();
        }
        self.pages.push(Page { forge, state, bytes, until: self.env.now.saturating_add(self.life) });
    }
}
/// Name the forge before gating, including plans whose connected path is staged later.
fn forge(plan: &host::Plan) -> u16 {
    match plan {
        host::Plan::Manifest { forge, .. }
        | host::Plan::Convert { forge, .. }
        | host::Plan::Install { forge }
        | host::Plan::Accept { forge, .. }
        | host::Plan::Reconcile { forge, .. }
        | host::Plan::Adopt { forge, .. }
        | host::Plan::Create { forge, .. }
        | host::Plan::Paste { forge, .. }
        | host::Plan::Check { forge }
        | host::Plan::TakeOver { forge, .. }
        | host::Plan::Reachable { forge, .. } => *forge,
    }
}
pub(crate) fn request(engine: &mut crate::engine::Engine, run: Token) {
    let plan = engine.setup.staged.remove(&run).expect("root names a staged setup plan");
    let number = forge(&plan);
    if crate::forge::ended(engine, number) {
        ended(engine, run, root::SetupEnd::Stopped { forge: number, reason: Box::from(b"forge ended".as_slice()) });
    } else if engine.drain.is_some() || engine.setup.closing || engine.setup.component.is_none() {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped { forge: number, reason: Box::from(b"setup host is not running".as_slice()) },
        );
    } else if let Some(host) = engine.forge.unreachable(number) {
        ended(engine, run, root::SetupEnd::Stopped { forge: number, reason: crate::forge::unreachable_text(&host) });
    } else if !engine.setup.served.contains_key(&number) {
        let reason = match engine.setup.unserved.get(&number) {
            Some(reason) => *reason,
            None => b"setup host has no entry for this forge",
        };
        ended(engine, run, root::SetupEnd::Stopped { forge: number, reason: Box::from(reason) });
    } else if matches_app_plan(&plan) && lifecycle::has_app(engine, number) {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped {
                forge: number,
                reason: Box::from(b"forge holds App records; Forget before making a new App".as_slice()),
            },
        );
    } else if is_install(&plan) && engine.setup.busy_internally(number) && engine.setup.held.room() > 0 {
        // An Install's link writes nothing: it waits for the engine's own run, never a person's.
        let until = engine.env.now.saturating_add(engine.setup.life);
        engine.setup.held.push(Held { run, forge: number, until });
    } else if (if let host::Plan::Reachable { .. } = &plan {
        engine.setup.reading(number)
    } else {
        lifecycle::busy(engine, number)
    }) || engine.setup.runs.len() == engine.setup.env.limits.runs
    {
        ended(
            engine,
            run,
            root::SetupEnd::Stopped { forge: number, reason: Box::from(b"setup host: busy".as_slice()) },
        );
    } else {
        let kind = lifecycle::kind(&plan);
        engine
            .setup
            .runs
            .insert(
                run,
                Run {
                    first: None,
                    forge: number,
                    kind,
                    internal: false,
                    previous: engine.domain.forge_connection(number),
                },
            )
            .expect("admitted host run");
        if kind == Kind::Convert {
            lifecycle::state(engine, number, temper_engine_domain_forge::Connection::Connecting);
        }
        run_plan(engine, run, plan);
    }
}
/// Run each held Install whose forge is free, or end it busy past the state's life.
fn release(engine: &mut crate::engine::Engine) {
    for _ in 0..engine.setup.held.len() {
        if engine.events.room() < 4 {
            break;
        }
        let held = engine.setup.held.pop().expect("held count");
        if !engine.setup.busy(held.forge) {
            engine
                .setup
                .staged
                .insert(held.run, host::Plan::Install { forge: held.forge })
                .expect("held run was staged");
            request(engine, held.run);
        } else if engine.env.now < held.until && engine.setup.busy_internally(held.forge) {
            engine.setup.held.push(held);
        } else {
            let reason = Box::from(b"setup host: busy".as_slice());
            ended(engine, held.run, root::SetupEnd::Stopped { forge: held.forge, reason });
        }
    }
}
fn is_install(plan: &host::Plan) -> bool {
    match plan {
        host::Plan::Install { .. } => true,
        host::Plan::Manifest { .. }
        | host::Plan::Convert { .. }
        | host::Plan::Accept { .. }
        | host::Plan::Reconcile { .. }
        | host::Plan::Paste { .. }
        | host::Plan::Check { .. }
        | host::Plan::TakeOver { .. }
        | host::Plan::Adopt { .. }
        | host::Plan::Create { .. }
        | host::Plan::Reachable { .. } => false,
    }
}
fn matches_app_plan(plan: &host::Plan) -> bool {
    match plan {
        host::Plan::Manifest { .. } | host::Plan::Convert { .. } => true,
        host::Plan::Install { .. }
        | host::Plan::Accept { .. }
        | host::Plan::Reconcile { .. }
        | host::Plan::Paste { .. }
        | host::Plan::Check { .. }
        | host::Plan::TakeOver { .. }
        | host::Plan::Adopt { .. }
        | host::Plan::Create { .. }
        | host::Plan::Reachable { .. } => false,
    }
}
fn ended(engine: &mut crate::engine::Engine, run: Token, end: root::SetupEnd) {
    if public::ended(engine, run, &end) {
        return;
    }
    engine.events.push(root::Event::Setup { run, end });
}
fn begin(engine: &mut crate::engine::Engine) {
    if !engine.forge_records.loading()
        && !engine.setup.waiting_for_webhooks
        && engine.end.is_none()
        && engine.drain.is_none()
        && let Some((settings, crypto)) = engine.setup.pending.take()
    {
        match host::Host::new(&engine.setup.env.limits, settings, crypto) {
            Ok(host) => engine.setup.component = Some(host),
            Err(why) => {
                let mut found = jig_protocol_config::Problems::empty();
                settings::problems(why, &mut found);
                engine.end = Some(crate::End::Stopped(crate::Stop::Start(found)));
            }
        }
    }
}
#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
pub(crate) fn advance(engine: &mut crate::engine::Engine) {
    engine.setup.env.now = engine.env.now;
    engine.setup.env.wall = engine.env.wall;
    begin(engine);
    release(engine);
    routes::advance(engine);
    for _ in 0..engine.setup.pages.len() {
        if !due(
            match engine.setup.pages.iter().next() {
                Some(page) => Some(page.until),
                None => None,
            },
            engine.env.now,
        ) {
            break;
        }
        let _expired = engine.setup.pages.pop();
    }
    if let Some(host) = &mut engine.setup.component
        && due(host.next_deadline(), engine.env.now)
    {
        host.fire(&engine.setup.env, &mut engine.setup.above, &mut engine.setup.io);
    }
    for _ in 0..engine.setup.above.len() {
        if engine.events.room() < engine.forge_records.bounds.installations.checked_add(4).expect("setup event room")
            || engine.notes.room() == 0
        {
            break;
        }
        let event = engine.setup.above.pop().expect("host output count");
        if (engine.setup.closing || engine.end.is_some()) && event != host::Event::Closed {
            continue;
        }
        match event {
            host::Event::Need { run, need } => {
                let forge = engine.setup.runs.get(&run).expect("live setup run").forge;
                match need {
                    host::Need::Read(item) => {
                        let value = read_item(engine, run, forge, item);
                        engine.setup.down(host::Request::Answer { run, answer: host::Answer::Read(value) });
                    }
                    host::Need::Write { item, bytes } => {
                        lifecycle::keep(
                            engine,
                            forge,
                            item_record(item).expect("host never writes Pasted"),
                            bytes.expose(),
                            Purpose::Host(run),
                        );
                    }
                }
            }
            host::Event::Done { run, outcome } => lifecycle::done(engine, run, outcome),
            host::Event::Stop { run, failure } => lifecycle::stopped(engine, run, failure),
            host::Event::Closed => engine.setup.closed = true,
        }
        if engine.end.is_some() {
            break;
        }
    }
    lifecycle::advance(engine);
}
pub(crate) fn answer(engine: &mut crate::engine::Engine, event: StoreEvent) {
    if let StoreEvent::Erased { owner } = event {
        if let Some(forge) = engine.setup.erases.remove(&owner)
            && !engine.setup.closing
            && engine.drain.is_none()
            && engine.end.is_none()
        {
            lifecycle::connect(engine, forge);
        }
        return;
    }
    if let StoreEvent::Kept { owner } = event {
        let Some(write) = engine.setup.writes.remove(&owner) else { return };
        engine.forge_records.rows.push(write.number, write.value, engine.setup.value_bytes);
        if !engine.setup.closing {
            lifecycle::kept(engine, write.purpose);
        }
    } else {
        lifecycle::writes_failed(engine);
        crate::stop::unavailable(engine, b"forge records");
    }
}
fn item_record(item: skein_forge_setup::item::Item) -> Option<Item> {
    use skein_forge_setup::item::Item as I;
    Some(match item {
        I::AdminToken => Item::AdminToken,
        I::ApiToken => Item::ApiToken,
        I::GitToken => Item::GitToken,
        I::User => Item::User,
        I::WebhookSecret => Item::WebhookSecret,
        I::AppId => Item::AppId,
        I::AppSlug => Item::AppSlug,
        I::AppKey => Item::AppKey,
        I::Accepted => Item::Accepted,
        I::Pasted => return None,
    })
}
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "failure has no credential; this is the diagnostic boundary"
)]
fn failure_text(failure: &reconcile::Failure) -> Box<[u8]> {
    alloc::format!(
        "setup {:?} {}: {:?}",
        failure.step,
        core::str::from_utf8(&failure.subject).unwrap_or("subject"),
        failure.why
    )
    .into_bytes()
    .into_boxed_slice()
}
fn due(at: Option<Time>, now: Time) -> bool {
    match at {
        Some(at) => at <= now,
        None => false,
    }
}
pub(crate) fn tag(mut request: skein_io::Request) -> skein_io::Request {
    if let skein_io::Request::Connect { owner, .. } = &mut request {
        assert!(owner.raw() < (1_u64 << 60_u32), "setup token fits prefix");
        *owner = Token::new(owner.raw() | crate::tokens::SETUP);
    }
    request
}
#[cfg(test)]
#[path = "tests/setup_scopes.rs"]
mod scopes_tests;
#[cfg(test)]
#[path = "tests/setup.rs"]
mod tests;

/// Every setup path, including internal follow-ups, crosses the same unresolved-host gate.
fn run_plan(engine: &mut crate::engine::Engine, run: Token, plan: host::Plan) {
    let number = forge(&plan);
    if crate::forge::ended(engine, number) {
        lifecycle::refused(engine, run, number, Box::from(b"forge ended".as_slice()));
    } else if let Some(host) = engine.forge.unreachable(number) {
        lifecycle::refused(engine, run, number, crate::forge::unreachable_text(&host));
    } else {
        engine.setup.down(host::Request::Run { run, plan });
    }
}

/// Connected Forgejo's legacy file input remains a host-local credential.
fn file_api_token(engine: &crate::engine::Engine, forge: u16) -> Option<&[u8]> {
    match engine.setup.file_api.get(&forge) {
        Some(value) => Some(value.expose()),
        None => None,
    }
}

#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
fn read_item(
    engine: &crate::engine::Engine,
    run: Token,
    forge: u16,
    item: skein_forge_setup::item::Item,
) -> Option<Secret> {
    if let Some(public::Caller::Connect { pasted, .. }) = engine.setup.public.get(&run) {
        match pasted {
            Some(public::Pasted::App(value)) if item == skein_forge_setup::item::Item::Pasted => {
                return Some(value.clone());
            }
            Some(public::Pasted::Admin(value)) if item == skein_forge_setup::item::Item::AdminToken => {
                return Some(value.clone());
            }
            Some(public::Pasted::App(_) | public::Pasted::Admin(_)) | None => {}
        }
        if item == skein_forge_setup::item::Item::AdminToken {
            return None; // A caller's admin read never falls back to records or adoptions.
        }
    }
    if item == skein_forge_setup::item::Item::Pasted {
        return None;
    }
    let item = item_record(item).expect("record item");
    let files = match engine.setup.lifecycle.adoptions.get(&forge) {
        Some(adoption) => adoption.files.as_ref(),
        None => None,
    };
    let value = match (files, item) {
        (Some(adoption), Item::AdminToken) => Some(adoption.admin_token.expose()),
        (Some(adoption), Item::WebhookSecret) => match &adoption.access.webhook_secret {
            Some(secret) => Some(secret.expose()),
            None => None,
        },
        _ => match engine.forge_records.rows.forge(forge) {
            Some(rows) => rows.get(item).ok(),
            None => None,
        },
    };
    let value = match value {
        Some(value) => Some(value),
        None if item == Item::ApiToken => file_api_token(engine, forge),
        None => None,
    };
    match value {
        Some(bytes) => Some(Secret::new(Box::from(bytes))),
        None => None,
    }
}
