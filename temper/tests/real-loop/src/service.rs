//! Deferred standalone preparation with its original operation count.
mod cue;
mod driven;
pub(crate) use driven::Driven;
mod watch;
use crate::fixtures;
pub use cue::Cue;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{
    net::{SocketAddr, TcpListener},
    os::unix::ffi::OsStrExt,
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
    pub(crate) notes: Vec<(Time, temper_service::Note)>,
    pub(crate) runs: crate::cluster::Runs,
    pub(crate) tries: std::collections::BTreeMap<u64, jig_domain_tasks::Tries>,
    tries_revision: Option<u64>,
    pub(crate) cancel_history: crate::cancel_kills::History,
    started_at: Option<Time>,
    pub(crate) service: Option<Driven>,
    fetch: Option<temper_worker_service::fetch::Fetch>,
    figures: Option<jig_protocol_worker_wire::Figures>,
    pub(crate) address: Option<SocketAddr>,
    pub(crate) link: Vec<u8>,
    watch: watch::Watch,
    pub(crate) hooks: Option<SocketAddr>,
    pub(crate) forge_address: Option<SocketAddr>,
    pub(crate) blob_address: Option<SocketAddr>,
    pub(crate) fixture: Option<crate::forge::Fixture>,
    pub(crate) seen: crate::forge::Seen,
    github: bool,
    records_seeded: bool,
    clients: Option<SocketAddr>,
    clients_listener: Option<TcpListener>,
    cues: Vec<Cue>,
    links: u32,
    path: PathBuf,
    pub(crate) configuration: temper_playground::documents::Configuration,
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
        Self::with_shape(path, signals, fixture, direct_key, None, false, None)
    }
    pub(crate) fn with_issuer(path: PathBuf, signals: Fd, issuer: SocketAddr) -> Self {
        Self::with_shape(path, signals, None, false, None, false, Some(issuer))
    }
    fn with_shape(
        path: PathBuf,
        signals: Fd,
        fixture: Option<crate::forge::Fixture>,
        direct_key: bool,
        cluster: Option<crate::cluster::Shape>,
        github: bool,
        issuer: Option<SocketAddr>,
    ) -> Self {
        let clients_listener = github.then(|| {
            let [listener] = temper_fixtures::ports::reserve::<1>(&[]);
            listener
        });
        let clients =
            clients_listener.as_ref().map(|listener| listener.local_addr().expect("reserved clients address"));
        let forge = (fixture.is_none() && !github)
            .then(|| TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("held forge listener"));
        let address = forge.as_ref().map_or(([127, 0, 0, 1], if github { 9 } else { 0 }).into(), |listener| {
            listener.local_addr().expect("held forge address")
        });
        let mut configuration = fixtures::write(&path, address, address).expect("stand-in standalone documents");
        if direct_key {
            fixtures::key_documents(&path, &mut configuration).expect("standard key route");
        }
        if let Some(fixture) = fixture {
            crate::forge::write(&path, address, &fixture, &mut configuration).expect("forge documents");
        }
        if github {
            crate::github::write(&path, address, clients.expect("fixed clients"), &mut configuration)
                .expect("GitHub documents");
        }
        if let Some(shape) = &cluster {
            shape.write(path.parent().expect("cluster scratch"), fixture.is_none() && !github, &mut configuration);
        }
        if let Some(address) = issuer {
            fixtures::sign_in_documents(&path, address, &mut configuration).expect("local issuer registration");
        }
        let roots = Self::directories(&path, cluster.is_some());
        let shape = match &cluster {
            None => temper_config_shell::Shape::Standalone,
            Some(crate::cluster::Shape::Engine { .. }) => temper_config_shell::Shape::EngineAlone,
            Some(crate::cluster::Shape::Worker { .. }) => temper_config_shell::Shape::WorkerAlone,
        };
        let config =
            fixtures::file::read(std::path::Path::new(std::ffi::OsStr::from_bytes(&roots.config)), shape).document;
        let given = fixtures::given(&path, &configuration);
        let operations = match shape {
            temper_config_shell::Shape::Standalone => temper_box_shell::operations_in_with_limits(
                "temper-box",
                &[(b"PATH".as_slice().into(), b"/usr/bin:/bin".as_slice().into())],
                b"/",
                &roots,
                &config,
                &given,
                &configuration.jig,
                &configuration.limits,
            ),
            temper_config_shell::Shape::EngineAlone | temper_config_shell::Shape::WorkerAlone => {
                temper_service::startup::operations_with_worker(
                    &config,
                    &temper_service_shell::jig_directories(&roots),
                    &given,
                    Some((&configuration.jig, &configuration.limits)),
                    None,
                )
                .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())
            }
        }
        .expect("shape's operation count");
        let mut host = Self::waiting(path, signals, configuration, operations, cluster.is_some());
        host.wait_check = temper_test_env::value(&temper_service_shell::WAIT_CHECK).is_some();
        host.records_seeded =
            matches!(cluster, Some(crate::cluster::Shape::Worker { .. })) || fixture.is_none() && !github;
        host.cluster = cluster;
        host.fixture = fixture;
        host.github = github;
        host.clients = clients;
        host.clients_listener = clients_listener;
        host.forge = forge;
        host
    }

    fn waiting(
        path: PathBuf,
        signals: Fd,
        configuration: temper_playground::documents::Configuration,
        operations: u32,
        restartable: bool,
    ) -> Self {
        let queued = if restartable { operations } else { 1 };
        Self {
            pending_streak: crate::patience::PendingStreak::default(),
            restartable,
            wait_check: false,
            now: Time::ZERO,
            lives: Vec::with_capacity(if restartable { LIVES } else { 0 }),
            state: LifeState::First,
            in_flight: 0,
            local_completion: false,
            life_tasks: 0,
            life_links: 0,
            linked: 0,
            cluster: None,
            workers_address: None,
            notes: Vec::new(),
            runs: crate::cluster::Runs::default(),
            tries: std::collections::BTreeMap::new(),
            tries_revision: None,
            cancel_history: crate::cancel_kills::History::default(),
            started_at: None,
            service: None,
            fetch: None,
            figures: None,
            address: None,
            link: Vec::new(),
            watch: watch::Watch::default(),
            hooks: None,
            forge_address: None,
            blob_address: None,
            fixture: None,
            github: false,
            records_seeded: false,
            clients: None,
            clients_listener: None,
            seen: crate::forge::Seen::default(),
            cues: Vec::with_capacity(cue::CUES),
            links: 0,
            path,
            configuration,
            signals,
            signals_adopted: false,
            forge: None,
            operations,
            completions: Queue::with_capacity(queued),
            submissions: Queue::with_capacity(queued),
        }
    }
}
impl ServiceHost {
    /// A prepared fixture with no native admission or outstanding kernel requests.
    #[cfg(test)]
    pub(crate) fn held_startup(
        configuration: temper_playground::documents::Configuration,
        service: temper_service::Service,
    ) -> Self {
        let mut host = Self::waiting(PathBuf::new(), Fd::new(0), configuration, service.operations(), false);
        host.service = Some(Driven::new(service));
        host
    }

    pub(crate) fn cluster(
        path: &std::path::Path,
        signals: Fd,
        shape: crate::cluster::Shape,
        fixture: Option<crate::forge::Fixture>,
    ) -> Self {
        Self::with_shape(path.join("data"), signals, fixture, false, Some(shape), false, None)
    }
    pub(crate) fn github(path: PathBuf, signals: Fd) -> Self {
        Self::with_shape(path, signals, None, false, None, true, None)
    }
    pub(crate) fn clients_occupied_by(&mut self, listener: &TcpListener) {
        let address = listener.local_addr().expect("occupied clients address");
        let mut file = fixtures::file::read(&self.path, temper_config_shell::Shape::Standalone).file;
        file.listen = Some(address);
        fixtures::file::write(&self.path, &file, temper_config_shell::Shape::Standalone)
            .expect("occupied listener file");
        // This endpoint must fail to bind; no reserved listener belongs to its service.
        self.clients_listener = None;
        self.clients = Some(address);
    }
    /// Settled lives in order; `ran()` and `linked()` retain their aggregate history.
    #[must_use]
    pub fn lives(&self) -> &[Life] {
        &self.lives
    }
    pub(crate) fn stopping(&self) -> bool {
        matches!(self.state, LifeState::Stopping | LifeState::Restarting)
    }
    pub(crate) fn stop_for_restart(&mut self) -> Result<(), String> {
        if !(self.restartable && self.state == LifeState::First && !self.is_empty()) {
            return Err("stop the first live cluster service".into());
        }
        self.state = LifeState::Stopping;
        Ok(())
    }
    pub(crate) fn settled(&self) -> bool {
        self.in_flight == 0
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.service.as_ref().is_some_and(Driven::is_empty)
    }
    pub(crate) fn restart(&mut self) -> Result<(), String> {
        if !(self.state == LifeState::Stopping && self.settled()) {
            return Err("restart a settled cluster service".into());
        }
        // The referee requests only; dropping and preparing stay in this host's iteration.
        self.state = LifeState::Restarting;
        Ok(())
    }
    fn begin_next_life(&mut self) {
        self.life_tasks = self.runs.tasks.len();
        self.life_links = self.linked();
        self.service = None;
        self.figures = None;
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
        if let Some(fetch) = &mut self.fetch {
            while let Some(submit) = fetch.submissions().pop() {
                self.in_flight += 1;
                self.submissions.push(submit);
            }
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
            if self.fixture.is_some() || self.github {
                fixtures::seed(&self.path, self.forge_address.expect("forge peer address"), self.github, Vec::new());
            }
            self.records_seeded = true;
        }
    }
    fn ready(&self) -> bool {
        self.cluster.as_ref().is_none_or(crate::cluster::Shape::ready)
            && ((self.fixture.is_none() && !self.github) || self.forge_address.is_some())
            && (!self.github || self.blob_address.is_some())
    }
    fn prepare_service(&self) -> Driven {
        use std::{
            os::fd::{AsRawFd, IntoRawFd},
            os::unix::ffi::OsStrExt,
        };
        let prepared = if let Some(figures) = &self.figures {
            let directories = Self::directories(&self.path, true);
            let mut part = temper_worker_service::startup::worker_part(
                &fixtures::file::read(
                    &self.path.parent().expect("cluster scratch").join("config"),
                    temper_config_shell::Shape::WorkerAlone,
                )
                .document,
                &temper_service_shell::jig_directories(&directories),
                Some(b"/usr/bin/git"),
                &fixtures::environment(&self.path),
            )
            .expect("worker part");
            if let Some(shape) = &self.cluster {
                let agent: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(self.path.join("world-agent.json")).expect("world agent"))
                        .expect("world agent JSON");
                part.worker.jig.graces = shape.graces(&agent);
            }
            part.named.program = self.path.join("temper-agent").as_os_str().as_bytes().into();
            part.named.arguments = Box::new([part.named.agent.clone()]);
            let files =
                temper_service_shell::read_files(core::mem::take(&mut part.files), skein_shell::effective_user())
                    .expect("worker files");
            Driven::worker(
                temper_worker_shell::worker::from_figures(
                    part,
                    figures.clone(),
                    &files,
                    &directories,
                    &mut Vec::new(),
                    &fixtures::environment(&self.path),
                )
                .expect("worker built from figures"),
            )
        } else if matches!(self.cluster, Some(crate::cluster::Shape::Engine { .. })) {
            let (given, projects) = fixtures::given_and_projects(&self.path, &self.configuration);
            Driven::engine(
                temper_engine_shell::serve::prepare_in_with_inputs_and_limits(
                    "temper-engine",
                    &fixtures::environment(&self.path),
                    b"/",
                    &Self::directories(&self.path, true),
                    &given,
                    &projects,
                    Some((&self.configuration.jig, &self.configuration.limits)),
                )
                .expect("engine preparation"),
            )
        } else {
            let (given, projects) = fixtures::given_and_projects(&self.path, &self.configuration);
            Driven::new(
                temper_box_shell::prepare_in_with_inputs_and_limits(
                    "temper",
                    &fixtures::environment(&self.path),
                    b"/",
                    &Self::directories(&self.path, self.cluster.is_some()),
                    &given,
                    Some(&self.path.join("temper-agent")),
                    &projects,
                    Some((&self.configuration.jig, &self.configuration.limits)),
                )
                .expect("standalone preparation"),
            )
        };
        let mut service = prepared;
        if let Some(blob) = self.blob_address
            && let driven::Service::Joined(joined) = &mut service.service
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
    pub(crate) fn early_exit(&self, process: usize) -> Result<String, String> {
        const STDERR_TAIL: usize = 4096;
        let service = self.service.as_ref().ok_or("an ended service was prepared")?;
        let directories = Self::directories(&self.path, self.cluster.is_some());
        let (stderr, code) = temper_box_shell::service_ending(
            service.end(),
            false,
            &directories,
            &directories.data,
            service.unsettled(),
        );
        let tail = &stderr.as_bytes()[stderr.len().saturating_sub(STDERR_TAIL)..];
        Ok(format!(
            "service {process} exited before caller release: {:?}; end={:?}\nstderr tail:\n{}",
            skein_io::kernel::Exit::Code(code),
            service.end(),
            String::from_utf8_lossy(tail)
        ))
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
    pub fn arm(&mut self, key: skein_lib::RequestKey) -> Result<(), String> {
        if self.cues.len() >= cue::CUES {
            return Err("bounded held cues".into());
        }
        self.cues.push(Cue::new(key));
        self.seed_log_cues();
        Ok(())
    }
    fn seed_log_cues(&mut self) {
        if let Some(driven) = &mut self.service {
            for fd in driven.log_descriptors() {
                for cue in &mut self.cues {
                    cue.opened_log(*fd);
                }
            }
        }
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

fn record_tries(
    tries: &mut std::collections::BTreeMap<u64, jig_domain_tasks::Tries>,
    history: &mut crate::cancel_kills::History,
    service: &Driven,
    now: Time,
) {
    use jig_domain_tasks::{Ending, Executor, Party, Phase, Was};
    if let Some(engine) = service.engine_domain() {
        for row in &engine.tasks().view_tasks() {
            let task = engine.tasks().task(row.number).expect("view task");
            history.tasks.entry(row.number).or_insert_with(|| crate::cancel_kills::Task {
                requester: match task.requester {
                    Party::Task(number) => Some(number),
                    Party::Person(_) | Party::Deployment { .. } => None,
                },
                agent: matches!(task.executor, Executor::Agent { .. }),
            });
            if let Phase::Closing(closing) | Phase::Held { was: Was::Closing(closing), .. } = &task.phase
                && let Ending::Cancelled { by, .. } = &closing.ending
            {
                history
                    .cancellations
                    .entry(row.number)
                    .or_insert(crate::cancel_kills::Cancellation { by: *by, at: now });
            }
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
    let directory = data.parent().expect("cluster scratch").join("config");
    let mut file = fixtures::file::read(&directory, temper_config_shell::Shape::EngineAlone).file;
    match listener {
        temper_service::Listening::Workers => file.workers = Some(addr),
        temper_service::Listening::Clients => file.listen = Some(addr),
        temper_service::Listening::Webhooks => file.webhooks = Some(addr),
    }
    fixtures::file::write(&directory, &file, temper_config_shell::Shape::EngineAlone).expect("retained engine address");
}

fn configure_job_logs(service: &mut temper_service::Service, api: SocketAddr, blob: SocketAddr) {
    let roots = skein_shell::read_trust_roots(
        &skein_shell::Trust::Machine { ssl_cert_file: None },
        &skein_shell::RootBounds { bundle_bytes: 1_048_576, certificates: 1024, certificate_bytes: 65_536 },
    )
    .expect("machine roots");
    let roots = skein_tls::Config::from_der(&roots.certificates, &[]).expect("TLS roots").0;
    let mut log_origins = temper_box_shell::github_logs::defaults();
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
    #[expect(
        clippy::too_many_lines,
        reason = "the host turn keeps both halves' terminal matches beside restart and completion routing"
    )]
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        if self.state == LifeState::Restarting {
            self.begin_next_life();
        }
        if self.service.is_none()
            && self.fetch.is_none()
            && self.ready()
            && let Some(address) = self.address
        {
            let forge = self
                .forge_address
                .unwrap_or_else(|| self.forge.as_ref().expect("held forge").local_addr().expect("forge address"));
            if self.state != LifeState::Second {
                fixtures::bind(&self.path, address, forge, &mut self.configuration)
                    .expect("bound standalone documents");
                if let Some(shape) = &self.cluster {
                    shape.write(
                        self.path.parent().expect("cluster scratch"),
                        self.fixture.is_none() && !self.github,
                        &mut self.configuration,
                    );
                }
            }
            if matches!(self.cluster, Some(crate::cluster::Shape::Worker { .. })) && self.figures.is_none() {
                let directories = Self::directories(&self.path, true);
                let mut part = temper_worker_service::startup::worker_part(
                    &fixtures::file::read(
                        &self.path.parent().expect("cluster scratch").join("config"),
                        temper_config_shell::Shape::WorkerAlone,
                    )
                    .document,
                    &temper_service_shell::jig_directories(&directories),
                    Some(b"/usr/bin/git"),
                    &fixtures::environment(&self.path),
                )
                .expect("worker part");
                let files =
                    temper_service_shell::read_files(core::mem::take(&mut part.files), skein_shell::effective_user())
                        .expect("worker files");
                self.fetch = Some(
                    temper_worker_service::fetch::Fetch::new(&part.worker, &files, fixtures::SEED)
                        .expect("figures fetch"),
                );
            } else {
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
                self.seed_log_cues();
                self.tries_revision = None;
            }
        }
        if let Some(fetch) = &mut self.fetch {
            while let Some(complete) = self.completions.pop() {
                self.in_flight = self.in_flight.checked_sub(1).expect("fetch operation completed");
                fetch.completions().push(complete);
            }
            temper_worker_service::fetch::iterate(fetch, now, wall);
            while let Some(note) = fetch.notes().pop() {
                self.notes.push((now, note.into()));
            }
            if fetch.is_empty() {
                let figures = fetch.take().expect("fetch took figures");
                self.figures = Some(figures);
                self.fetch = None;
            }
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
                if let Some(crate::cluster::Shape::Worker { engine: Some(engine), .. }) = self.cluster {
                    self.runs.complete(complete, engine, now);
                }
                self.watch.complete(complete);
                for cue in &mut self.cues {
                    cue.complete(complete);
                }
            }
            service.iterate(now, wall);
            if let Some(engine) = service.engine_domain()
                && self.tries_revision != Some(engine.revision())
            {
                self.tries_revision = Some(engine.revision());
                record_tries(&mut self.tries, &mut self.cancel_history, service, now);
            }
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
                self.notes.push((now, note));
            }
            while let Some(note) = service.take_note() {
                self.linked += usize::from(matches!(
                    note,
                    temper_service::Note::Worker(temper_service::worker_alone::Note::Linked { .. })
                ));
                self.notes.push((now, note));
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
        if let Some(fetch) = &self.fetch {
            return fetch.work_pending(now) || !self.completions.is_empty();
        }
        (self.restartable
            && (self.state == LifeState::Restarting || self.local_completion || !self.completions.is_empty()))
            || self.service.as_ref().map_or(self.address.is_some() && self.ready(), |service| service.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.fetch
            .as_ref()
            .and_then(temper_worker_service::fetch::Fetch::next_deadline)
            .or_else(|| self.service.as_ref().and_then(Driven::next_deadline))
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
