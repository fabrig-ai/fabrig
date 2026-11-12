//! Deferred standalone preparation with its original operation count.
mod cue;
mod driven;
pub(crate) use driven::Driven;
mod hooks;
mod watch;
use crate::fixtures;
pub use cue::Cue;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{
    net::{SocketAddr, TcpListener},
    path::PathBuf,
};

/// One fully settled service life, retained across a worker restart.
pub struct Life {
    /// Whether this service drained cooperatively.
    pub drained: bool,
    /// Tasks run during this life, in activation order.
    pub ran: Vec<u64>,
    /// Worker links opened during this life.
    pub linked: usize,
}

const LIVES: usize = 2;
#[derive(Clone, Copy, PartialEq, Eq)]
enum LifeState {
    First,
    Stopping,
    Restarting,
    Second,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "restart, completion, bootstrap and signal ownership are independent"
)]
pub struct ServiceHost {
    pub(crate) pending_streak: crate::patience::PendingStreak,
    restartable: bool,
    start_failures: u32,
    failed_starts: Vec<String>,
    wait_check: bool,
    now: Time,
    lives: Vec<Life>,
    state: LifeState,
    in_flight: usize,
    local_completion: bool,
    life_tasks: usize,
    life_links: usize,
    linked: usize,

    pub(crate) cluster: Option<crate::cluster::Shape>,
    pub(crate) workers_address: Option<SocketAddr>,
    pub(crate) notes: Vec<temper_service::Note>,
    pub(crate) runs: crate::cluster::Runs,
    pub(crate) tries: std::collections::BTreeMap<u64, jig_domain_tasks::Tries>,
    started_at: Option<Time>,
    pub(crate) service: Option<Driven>,
    pub(crate) address: Option<SocketAddr>,
    pub(crate) link: Vec<u8>,
    watch: watch::Watch,
    hook_wire: hooks::Hooks,
    pub(crate) hooks: Option<SocketAddr>,
    pub(crate) forge_address: Option<SocketAddr>,
    pub(crate) blob_address: Option<SocketAddr>,
    pub(crate) fixture: Option<crate::forge::Fixture>,
    pub(crate) seen: crate::forge::Seen,
    github: Option<crate::github::Fixture>,
    records_seeded: bool,
    clients: Option<SocketAddr>,
    clients_listener: Option<TcpListener>,
    pub(crate) github_seen: crate::github::Seen,
    cues: Vec<Cue>,
    links: u32,
    path: PathBuf,
    signals: Fd,
    signals_adopted: bool,
    forge: Option<TcpListener>,
    operations: u32,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl ServiceHost {
    pub(crate) fn new(path: PathBuf, signals: Fd, direct_key: bool) -> Self {
        Self::configured(path, signals, None, direct_key)
    }
    pub(crate) fn configured(
        path: PathBuf,
        signals: Fd,
        fixture: Option<crate::forge::Fixture>,
        direct_key: bool,
    ) -> Self {
        Self::with_shape(path, signals, fixture, direct_key, None, None, None)
    }
    pub(crate) fn with_issuer(path: PathBuf, signals: Fd, issuer: SocketAddr) -> Self {
        Self::with_shape(path, signals, None, false, None, None, Some(issuer))
    }
    fn with_shape(
        path: PathBuf,
        signals: Fd,
        fixture: Option<crate::forge::Fixture>,
        direct_key: bool,
        cluster: Option<crate::cluster::Shape>,
        github: Option<crate::github::Fixture>,
        issuer: Option<SocketAddr>,
    ) -> Self {
        let clients_listener = github.map(|_| {
            let [listener] = temper_fixtures::ports::reserve::<1>(&[]);
            listener
        });
        let clients =
            clients_listener.as_ref().map(|listener| listener.local_addr().expect("reserved clients address"));
        let forge = (fixture.is_none() && github.is_none())
            .then(|| TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("held forge listener"));
        let address = forge
            .as_ref()
            .map_or(([127, 0, 0, 1], 0).into(), |listener| listener.local_addr().expect("held forge address"));
        fixtures::write(&path, address, address).expect("stand-in standalone documents");
        if direct_key {
            fixtures::key_documents(&path).expect("standard key route");
        }
        if let Some(fixture) = fixture {
            crate::forge::write(&path, address, &fixture).expect("forge documents");
        }
        if let Some(fixture) = github {
            crate::github::write(&path, address, clients.expect("fixed clients"), fixture).expect("GitHub documents");
        }
        if fixture.is_none() && github.is_none() {
            let mut value = serde_json::from_slice(&std::fs::read(path.join("temper.json")).expect("deployment"))
                .expect("deployment JSON");
            fixtures::finish_configuration(&path, &mut value).expect("world defaults separated from startup");
        }
        if let Some(shape) = &cluster {
            shape.write(path.parent().expect("cluster scratch"), fixture.is_none() && github.is_none());
        }
        if let Some(address) = issuer {
            fixtures::sign_in_documents(&path, address).expect("local issuer registration");
        }
        let roots = Self::directories(&path, cluster.is_some());
        let config = std::fs::read(roots.document()).expect("standalone document");
        let operations = temper_box_shell::operations_in_with(&[], b"/", &roots, &config, &fixtures::given(&path))
            .expect("standalone operation count");
        let restartable = cluster.is_some();
        let queued = if restartable { operations } else { 1 };
        let records_seeded =
            matches!(cluster, Some(crate::cluster::Shape::Worker { .. })) || fixture.is_none() && github.is_none();
        Self {
            pending_streak: crate::patience::PendingStreak::default(),
            restartable,
            start_failures: 0,
            failed_starts: Vec::new(),
            wait_check: temper_test_env::value(&temper_service_shell::WAIT_CHECK).is_some(),
            now: Time::ZERO,
            lives: Vec::with_capacity(if restartable { LIVES } else { 0 }),
            state: LifeState::First,
            in_flight: 0,
            local_completion: false,
            life_tasks: 0,
            life_links: 0,
            linked: 0,
            cluster,
            workers_address: None,
            notes: Vec::new(),
            runs: crate::cluster::Runs::default(),
            tries: std::collections::BTreeMap::new(),
            started_at: None,
            service: None,
            address: None,
            link: Vec::new(),
            watch: watch::Watch::default(),
            hook_wire: hooks::Hooks::default(),
            hooks: None,
            forge_address: None,
            blob_address: None,
            fixture,
            github,
            records_seeded,
            clients,
            clients_listener,
            github_seen: crate::github::Seen::default(),
            seen: crate::forge::Seen::default(),
            cues: Vec::with_capacity(cue::CUES),
            links: 0,
            path,
            signals,
            signals_adopted: false,
            forge,
            operations,
            completions: Queue::with_capacity(queued),
            submissions: Queue::with_capacity(queued),
        }
    }
}
impl ServiceHost {
    pub(crate) fn cluster(
        path: &std::path::Path,
        signals: Fd,
        shape: crate::cluster::Shape,
        fixture: Option<crate::forge::Fixture>,
    ) -> Self {
        Self::with_shape(path.join("data"), signals, fixture, false, Some(shape), None, None)
    }
    pub(crate) fn github(path: PathBuf, signals: Fd, fixture: crate::github::Fixture) -> Self {
        Self::with_shape(path, signals, None, false, None, Some(fixture), None)
    }
    #[cfg(test)]
    pub(crate) fn clients_occupied_by(&mut self, listener: &TcpListener) {
        let address = listener.local_addr().expect("occupied clients address");
        let path = self.path.join("temper.json");
        let mut config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("config")).expect("JSON");
        config["application"]["clients"]["listener"] = serde_json::json!({"socket":address.to_string()});
        fixtures::private(&path, &serde_json::to_vec(&config).expect("config JSON")).expect("occupied listener config");
        // This endpoint must fail to bind; no reserved listener belongs to its service.
        self.clients_listener = None;
        self.clients = Some(address);
    }
    pub(crate) fn make_github_app(&mut self, peer: &mut skein_fake_peers::forge::Peer) {
        if let Some(fixture) = self.github
            && !fixture.connected
            && !fixture.manifest
        {
            self.github_seen.app =
                Some(crate::github::make_app(peer, self.clients.expect("clients"), &self.path, fixture));
        }
    }
    pub(crate) fn checkpoint_github(
        &mut self,
        peer: &mut skein_fake_peers::forge::Peer,
        calls: &[crate::calls::Called],
    ) {
        if let Some(fixture) = self.github
            && fixture.unaccepted_after_install
        {
            self.github_seen.seed_unaccepted(peer);
        }
        self.github_seen.checkpoint(peer, calls);
    }
    /// Settled lives in order; `ran()` and `linked()` retain their aggregate history.
    #[must_use]
    pub fn lives(&self) -> &[Life] {
        &self.lives
    }
    pub(crate) fn stopping(&self) -> bool {
        matches!(self.state, LifeState::Stopping | LifeState::Restarting)
    }
    pub(crate) fn stop_for_restart(&mut self) {
        assert!(
            self.restartable && self.state == LifeState::First && !self.is_empty(),
            "stop the first live cluster service"
        );
        self.state = LifeState::Stopping;
    }
    pub(crate) fn settled(&self) -> bool {
        self.in_flight == 0
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.service.as_ref().is_some_and(Driven::is_empty)
    }
    pub(crate) fn restart(&mut self) {
        assert!(self.state == LifeState::Stopping && self.settled(), "restart a settled cluster service");
        // The referee requests only; dropping and preparing stay in this host's iteration.
        self.state = LifeState::Restarting;
    }
    pub(crate) fn restart_after_failures(&mut self, failures: u32) {
        self.restart();
        self.start_failures = failures;
    }
    /// Starts refused by the shipped configuration checks, before a service is made.
    #[must_use]
    pub fn failed_starts(&self) -> &[String] {
        &self.failed_starts
    }
    fn refuse_starts(&mut self) {
        if self.start_failures == 0 {
            return;
        }
        assert!(matches!(self.cluster, Some(crate::cluster::Shape::Engine { .. })));
        let directories = Self::directories(&self.path, true);
        let document = directories.document();
        let original = std::fs::read(&document).expect("engine document");
        let mut invalid: serde_json::Value = serde_json::from_slice(&original).expect("engine JSON");
        invalid["engine"]["limits"]["profile"] = serde_json::json!("unknown-profile");
        fixtures::private(&document, &serde_json::to_vec(&invalid).expect("refused engine JSON"))
            .expect("misconfigured start");
        for _ in 0..self.start_failures {
            let Err(failure) = temper_box_shell::prepare_in_with(&[], b"/", &directories, &fixtures::given(&self.path))
            else {
                panic!("unknown engine profile must refuse the start")
            };
            assert!(failure.contains("engine.limits.profile"), "{failure}");
            self.failed_starts.push(failure);
        }
        fixtures::private(&document, &original).expect("corrected engine document");
        self.start_failures = 0;
    }
    fn begin_next_life(&mut self) {
        self.life_tasks = self.runs.tasks.len();
        self.life_links = self.linked();
        self.service = None;
        self.state = LifeState::Second;
    }
    fn record_life(&mut self) {
        if self.restartable && self.settled() && self.lives.len() == usize::from(self.state == LifeState::Second) {
            assert!(self.lives.len() < LIVES, "bounded service lives");
            self.lives.push(Life {
                drained: matches!(
                    self.end(),
                    Some(
                        crate::End::Engine(temper_service::EngineEnd::Drained)
                            | crate::End::Worker(temper_service::worker_alone::End::Drained)
                    )
                ),
                ran: self.runs.tasks[self.life_tasks..].to_vec(),
                linked: self.linked() - self.life_links,
            });
        }
    }
    /// Forward under the host's span, including a stopping life's local close.
    fn forward_submissions(&mut self) {
        if !self.restartable {
            return;
        }
        if let Some(service) = &mut self.service {
            while let Some(submit) = service.submissions().pop() {
                if self.state == LifeState::Stopping && matches!(submit.kind, Op::Close { fd } if fd == self.signals) {
                    self.local_completion = true;
                    // The supervisor owns this pipe across both service lives.
                    // Complete the inner close, retaining the registered reader.
                    service.completions().push(Complete {
                        op: submit.op,
                        kind: submit.kind,
                        result: Ok(Done::Nothing),
                    });
                } else {
                    self.in_flight += 1;
                    self.submissions.push(submit);
                }
            }
        }
    }
    fn directories(path: &std::path::Path, cluster: bool) -> temper_box_shell::directories::Directories {
        if cluster {
            let path = path.parent().expect("cluster scratch");
            crate::cluster::directories_in([&path.join("config"), &path.join("data"), &path.join("cache")])
        } else {
            directories(path)
        }
    }
    fn finish_bootstrap(&mut self) {
        let Some(service) = &mut self.service else { return };
        if !self.signals_adopted
            && service.end().is_some_and(|end| {
                !matches!(
                    end,
                    crate::End::Engine(temper_service::EngineEnd::Drained)
                        | crate::End::Worker(temper_service::worker_alone::End::Drained)
                )
            })
        {
            // A failed bootstrap will not reopen: close its retained signal
            // reader through the ended service's ordinary io lifecycle.
            service.adopt_signals(self.signals).expect("failed bootstrap signal pipe");
            self.signals_adopted = true;
        }
        if !self.records_seeded
            && service.is_empty()
            && matches!(
                service.end(),
                Some(
                    crate::End::Engine(temper_service::EngineEnd::Drained)
                        | crate::End::Worker(temper_service::worker_alone::End::Drained)
                )
            )
        {
            self.service = None;
            fixtures::seed(&self.path, self.forge_address.expect("forge peer address"), self.github);
            self.records_seeded = true;
        }
    }
    fn ready(&self) -> bool {
        self.cluster.as_ref().is_none_or(crate::cluster::Shape::ready)
            && ((self.fixture.is_none() && self.github.is_none()) || self.forge_address.is_some())
            && (self.github.is_none() || self.blob_address.is_some())
    }
    fn prepare_service(&self) -> Driven {
        use std::os::fd::{AsRawFd, IntoRawFd};
        let prepared = temper_box_shell::prepare_in_with(
            &[],
            b"/",
            &Self::directories(&self.path, self.cluster.is_some()),
            &fixtures::given(&self.path),
        )
        .expect("standalone preparation");
        let mut service = Driven::new(prepared);
        if let Some(blob) = self.blob_address
            && let temper_box_shell::Service::Joined(joined) = &mut service.service
        {
            configure_job_logs(joined, self.forge_address.expect("GitHub API"), blob);
        }
        if let Some(listener) = &self.clients_listener {
            // Keep the original bound across the bootstrap service's shutdown
            // and the configured App's second start at the same address.
            let listener = listener.try_clone().expect("clients listener descriptor");
            service
                .adopt_listener(Fd::new(listener.as_raw_fd()), listener.local_addr().expect("clients address"))
                .expect("held clients listener");
            let _ = listener.into_raw_fd();
        }
        service
    }
    #[must_use]
    pub fn end(&self) -> Option<&crate::End> {
        self.service.as_ref().and_then(Driven::end)
    }
    pub(crate) fn early_exit(&self, process: usize) -> String {
        const STDERR_TAIL: usize = 4096;
        let service = self.service.as_ref().expect("an ended service was prepared");
        let directories = Self::directories(&self.path, self.cluster.is_some());
        let (stderr, code) = temper_box_shell::service_ending(
            service.end(),
            false,
            &directories,
            &directories.data,
            service.unsettled(),
        );
        let tail = &stderr.as_bytes()[stderr.len().saturating_sub(STDERR_TAIL)..];
        format!(
            "service {process} exited before caller release: {:?}; end={:?}\nstderr tail:\n{}",
            skein_io::kernel::Exit::Code(code),
            service.end(),
            String::from_utf8_lossy(tail)
        )
    }
    #[must_use]
    pub fn started_at(&self) -> Option<Time> {
        self.started_at
    }
    #[must_use]
    pub fn assigned_at(&self) -> Option<Time> {
        self.runs.assigned_at
    }
    #[must_use]
    pub fn ran(&self) -> &[u64] {
        &self.runs.tasks
    }
    /// Assignments decoded at the worker's own wire, retained across lives.
    #[must_use]
    pub fn assignments(&self) -> &[jig_protocol_worker_wire::Assignment] {
        &self.runs.assignments
    }
    /// Worker openings retained from the shipped service's diagnostics.
    #[must_use]
    pub fn linked(&self) -> usize {
        self.linked
    }
    /// Links printed by this service start, independent of the scratch's old file.
    #[must_use]
    pub fn links(&self) -> u32 {
        self.links
    }
    pub(crate) fn fixture_ready(&self) -> bool {
        self.fixture.is_none()
            || self.service.as_ref().and_then(Driven::engine_domain).is_some_and(|domain| {
                matches!(domain.forge_connection(1), temper_engine_domain_forge::Connection::Connected { .. })
            })
    }
    pub(crate) fn arm_watch(&mut self, call: usize) {
        if self.watch.call != Some(call) {
            self.watch = watch::Watch::default();
            self.watch.call = Some(call);
        }
        self.watch.armed = true;
    }
    pub fn arm(&mut self, key: skein_lib::RequestKey) {
        assert!(self.cues.len() < cue::CUES, "bounded held cues");
        self.cues.push(Cue::new(key));
    }
    #[must_use]
    pub fn held(&self) -> bool {
        !self.cues.is_empty() && self.cues.iter().all(Cue::held)
    }
    pub(crate) fn held_at(&self, index: usize) -> bool {
        self.cues.get(index).is_some_and(Cue::held)
    }
    pub(crate) fn watching(&self) -> bool {
        self.watch.armed
    }
    pub(crate) fn delivered(&self) -> bool {
        self.watch.delivered
    }
}

fn record_tries(tries: &mut std::collections::BTreeMap<u64, jig_domain_tasks::Tries>, service: &Driven) {
    if let Some(engine) = service.engine_domain() {
        for row in &engine.tasks().view_tasks() {
            let task = engine.tasks().task(row.number).expect("view task");
            let seen = tries.entry(row.number).or_insert(jig_domain_tasks::Tries::NONE);
            seen.transient = seen.transient.max(task.tries.transient);
            seen.permanent = seen.permanent.max(task.tries.permanent);
            seen.run = seen.run.max(task.tries.run);
            seen.agent = seen.agent.max(task.tries.agent);
            seen.lost = seen.lost.max(task.tries.lost);
            seen.invalid = seen.invalid.max(task.tries.invalid);
        }
    }
}

fn retain_listener(data: &std::path::Path, listener: temper_service::Listening, addr: SocketAddr) {
    let path = ServiceHost::directories(data, true).document();
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("engine document")).expect("engine JSON");
    let socket = serde_json::json!({"socket":addr.to_string()});
    match listener {
        temper_service::Listening::Workers => document["engine"]["hosts"]["listener"] = serde_json::json!([socket]),
        temper_service::Listening::Clients => document["application"]["clients"]["listener"] = socket,
        temper_service::Listening::Webhooks => document["application"]["webhooks"]["listener"] = socket,
    }
    fixtures::private(&path, &serde_json::to_vec(&document).expect("engine JSON")).expect("retained engine address");
}

fn configure_job_logs(service: &mut temper_service::Service, api: SocketAddr, blob: SocketAddr) {
    let roots = skein_shell::read_trust_roots(
        &skein_shell::Trust::Machine { ssl_cert_file: None },
        &skein_shell::RootBounds { bundle_bytes: 1_048_576, certificates: 1024, certificate_bytes: 65_536 },
    )
    .expect("machine roots");
    let roots = skein_tls::Config::from_der(&roots.certificates, &[]).expect("TLS roots").0;
    let mut log_origins = temper_box_shell::github_logs::resolve(None, |_| Ok(([127, 0, 0, 1], 443).into()))
        .expect("production default with fake DNS");
    // The fake serves plaintext on loopback; keep the production host policy.
    for at in 0..log_origins.len() {
        let entry = log_origins.get_mut(at).expect("default origin");
        entry.origin.tls = false;
        entry.origin.port = blob.port();
        entry.address = skein_io::Addr::Ip(blob);
    }
    service.configure_forge_runtime(
        temper_service::ForgeRuntime {
            number: 0,
            address: api,
            transport: skein_forge_connection::settings::Transport::Plaintext { name: None },
            log_origins,
        },
        roots,
    );
}
impl Host for ServiceHost {
    fn drain(&mut self) {
        if self.wait_check
            && let Some(service) = &self.service
        {
            service.check_wait_boundary(self.now);
        }
        self.forward_submissions();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        if self.state == LifeState::Restarting {
            self.refuse_starts();
            self.begin_next_life();
        }
        if self.service.is_none()
            && self.ready()
            && let Some(address) = self.address
        {
            let forge = self
                .forge_address
                .unwrap_or_else(|| self.forge.as_ref().expect("held forge").local_addr().expect("forge address"));
            if self.state != LifeState::Second {
                fixtures::bind(&self.path, address, forge).expect("bound standalone documents");
                if let Some(shape) = &self.cluster {
                    shape.write(
                        self.path.parent().expect("cluster scratch"),
                        self.fixture.is_none() && self.github.is_none(),
                    );
                }
            }
            let mut service = self.prepare_service();
            if self.cluster.is_some() {
                assert!(service.operations() <= self.operations, "cluster operations fit admission");
            } else {
                assert_eq!(service.operations(), self.operations, "preparation preserves operations");
            }
            if self.records_seeded {
                service.adopt_signals(self.signals).expect("service signal pipe");
                self.signals_adopted = true;
                self.started_at.get_or_insert(now);
            }
            self.service = Some(service);
        }
        if let Some(service) = &mut self.service {
            self.local_completion = false;
            if self.restartable {
                while let Some(complete) = self.completions.pop() {
                    self.in_flight = self.in_flight.checked_sub(1).expect("a completion has a submitted operation");
                    service.completions().push(complete);
                }
            }
            for complete in service.completions().iter() {
                if self.fixture.is_some() {
                    self.hook_wire.complete(complete, self.hooks, &mut self.seen.responses);
                }
                if let Some(crate::cluster::Shape::Worker { engine: Some(engine), .. }) = self.cluster {
                    self.runs.complete(complete, engine, now);
                }
                self.watch.complete(complete);
                for cue in &mut self.cues {
                    cue.complete(complete);
                }
            }
            service.iterate(now, wall);
            record_tries(&mut self.tries, service);
            while let Some(note) = service.take_listening() {
                if let temper_service::Note::Engine(temper_service::EngineNote::Listening { listener, addr }) = note {
                    if self.records_seeded && matches!(self.cluster, Some(crate::cluster::Shape::Engine { .. })) {
                        retain_listener(&self.path, listener, addr);
                    }
                    match listener {
                        temper_service::Listening::Workers => {
                            if self.records_seeded {
                                self.workers_address = Some(addr);
                            }
                        }
                        // The durable mark can leave the workers listening before webhooks.
                        temper_service::Listening::Webhooks => {
                            if self.records_seeded {
                                self.hooks = Some(addr);
                            }
                        }
                        temper_service::Listening::Clients => {}
                    }
                }
                self.linked += usize::from(matches!(
                    note,
                    temper_service::Note::Worker(temper_service::worker_alone::Note::Linked { .. })
                ));
                self.notes.push(note);
            }
            while let Some(note) = service.take_note() {
                self.linked += usize::from(matches!(
                    note,
                    temper_service::Note::Worker(temper_service::worker_alone::Note::Linked { .. })
                ));
                self.notes.push(note);
            }
            while let Some(link) = service.take_link() {
                if !self.records_seeded {
                    // The first start seeds settings and owners before its durable
                    // mark permits fixture records. Drain it before seeding the
                    // forge and opening the store for the story.
                    service.shutdown();
                    continue;
                }
                self.links += 1;
                self.link.clear();
                self.link.extend_from_slice(&link);
                self.link.push(b'\n');
                fixtures::private(&self.path.join("link"), &self.link).expect("private printed link");
            }
            if service.is_empty() && self.records_seeded && self.state != LifeState::Stopping {
                self.forge = None;
                self.clients_listener = None;
            }
        }
        self.finish_bootstrap();
        self.record_life();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        if !self.restartable
            && let Some(service) = &mut self.service
        {
            service.completions()
        } else {
            &mut self.completions
        }
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        if !self.restartable {
            return match &mut self.service {
                Some(service) => service.submissions(),
                None => &mut self.submissions,
            };
        }
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        (self.restartable
            && (self.state == LifeState::Restarting || self.local_completion || !self.completions.is_empty()))
            || self.service.as_ref().map_or(self.address.is_some() && self.ready(), |service| service.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.service.as_ref().and_then(Driven::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.as_ref().and_then(Driven::next_policy_deadline)
    }
    fn is_empty(&self) -> bool {
        if self.restartable {
            !self.stopping() && self.settled()
        } else {
            self.service.as_ref().is_some_and(Driven::is_empty)
        }
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}

/// All directories are explicit and belong to this fixture's scratch.
#[must_use]
pub fn directories(directory: &std::path::Path) -> temper_box_shell::directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let values = [
        b"--config".as_slice(),
        directory.as_os_str().as_bytes(),
        b"--data",
        directory.as_os_str().as_bytes(),
        b"--cache",
        directory.as_os_str().as_bytes(),
    ];
    temper_box_shell::directories::resolve(&values.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("fixture directories")
        .0
}
