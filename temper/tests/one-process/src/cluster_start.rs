//! Engine alone and two workers alone on one simulated kernel (temper/docs/design/testing/tiers.md, 1.5).
use crate::testing::{Disk, USER};
use jig_domain_core::{CoreRecord, Record, settings};
use jig_protocol_config::{Directories, Files};
use jig_protocol_store::{Event, Request, Write};
use serde_json::json;
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{List, Queue, Secret, Time, Token, Wall};
use skein_shell::Host;
use skein_world::{Memory, Referee, World};
use std::collections::BTreeSet;
use temper_playground::{documents, machine};
use temper_worker_service::{
    fetch::{self, Fetch},
    figures, startup, worker_alone,
};

const WORKERS: [(u32, &str); 2] = [(0, "w1"), (1, "w2")];
const ENGINE: &str = "127.0.0.1:7200";
const PLACES: u32 = 1;
const AGENTS_KIND: u32 = 21;
const SEED_WORKERS: Token = Token::new(1);
const START_STEPS: u32 = 10_000;

#[derive(Debug)]
pub struct Outcome {
    pub engine: Option<temper_service::End>,
    pub figures_sent: [usize; 2],
    pub places_taken: [u32; 2],
    pub links: [usize; 2],
    pub workers: [Option<worker_alone::End>; 2],
    pub settled: bool,
}

/// The worker's next service is built only after its figures connection closes.
struct Taking {
    fetch: Fetch,
    startup: Option<WorkerStart>,
}

struct WorkerStart {
    part: startup::WorkerPart,
    files: Files,
    root: Fd,
    seed: u64,
}

impl Taking {
    fn new(name: &str, root: Fd, seed: u64) -> Self {
        let config = format!("/{name}/config");
        let data = format!("/{name}/data");
        let cache = format!("/{name}/cache");
        let directories =
            Directories::given(config.as_bytes(), data.as_bytes(), cache.as_bytes()).expect("worker directories");
        let document = json!({"workers":[{"engine":{"socket":ENGINE},"name":name,"secret":"worker.secret","slots":PLACES}],"application":{}});
        // No runs need a Git program; the worker still opens and lays out its own checkout root.
        let part = startup::worker_part(document.to_string().as_bytes(), &directories, None, &[])
            .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&startup::lines(&why))));
        let mut entries = List::with_capacity(1);
        entries.push((part.worker.jig.secret.clone(), Secret::new(fixture_secret(name)))).expect("one worker's file");
        let files = Files { entries };
        let fetch = Fetch::new(&part.worker, &files, seed).expect("a figures opening");
        Self { fetch, startup: Some(WorkerStart { part, files, root, seed }) }
    }

    fn iterate(&mut self, now: Time, wall: Wall) -> Option<Worker> {
        fetch::iterate(&mut self.fetch, now, wall);
        while let Some(note) = self.fetch.notes().pop() {
            assert!(!matches!(note, worker_alone::Note::Linked { .. }), "figures are not a run link");
        }
        if !self.fetch.is_empty() {
            return None;
        }
        let figures = self.fetch.take().expect("figures completed before the connection closed");
        let places = figures.places;
        let WorkerStart { mut part, files, root, seed } = self.startup.take().expect("start the worker once");
        figures::apply(&mut part, figures)
            .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&startup::lines(&why))));
        let runtime = temper_service::WorkerRuntime {
            root,
            path: part.worker.root.clone(),
            agent: documents::pinned_agent(),
            wires: None,
        };
        let service = temper_service::startup::worker_alone(part, &files, &documents::agent(), runtime, seed)
            .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
        Some(Worker { service, places, linked: 0 })
    }
}

struct Worker {
    service: worker_alone::Service,
    places: u32,
    linked: usize,
}

struct Engine {
    service: temper_service::Service,
    figures_sent: [usize; 2],
}

enum Process {
    Engine(Box<Engine>),
    Taking(Box<Taking>),
    Worker(Box<Worker>),
}

impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        match self {
            Self::Engine(engine) => {
                let service = &mut engine.service;
                temper_service::iterate(service, now, wall);
                while let Some(note) = service.notes().pop() {
                    if let temper_service::Note::Engine(temper_service::EngineNote::WorkerFiguresTaken { worker }) =
                        note
                    {
                        let at = WORKERS
                            .iter()
                            .position(|(_, name)| name.as_bytes() == worker.as_ref())
                            .expect("figures taken at a configured position");
                        engine.figures_sent[at] += 1;
                    }
                }
                service.listening().clear();
                service.links().clear();
            }
            Self::Taking(taking) => {
                if let Some(worker) = taking.iterate(now, wall) {
                    *self = Self::Worker(Box::new(worker));
                }
            }
            Self::Worker(worker) => {
                worker_alone::iterate(&mut worker.service, now, wall);
                while let Some(note) = worker.service.notes().pop() {
                    worker.linked += usize::from(matches!(note, worker_alone::Note::Linked { .. }));
                }
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        match self {
            Self::Engine(engine) => engine.service.completions(),
            Self::Taking(taking) => taking.fetch.completions(),
            Self::Worker(worker) => worker.service.completions(),
        }
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        match self {
            Self::Engine(engine) => engine.service.submissions(),
            Self::Taking(taking) => taking.fetch.submissions(),
            Self::Worker(worker) => worker.service.submissions(),
        }
    }
    fn work_pending(&self, now: Time) -> bool {
        match self {
            Self::Engine(engine) => engine.service.work_pending(now),
            Self::Taking(taking) => taking.fetch.work_pending(now),
            Self::Worker(worker) => worker.service.work_pending(now),
        }
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        match self {
            Self::Engine(engine) => engine.service.next_deadline(),
            Self::Taking(taking) => taking.fetch.next_deadline(),
            Self::Worker(worker) => worker.service.next_deadline(),
        }
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        match self {
            Self::Engine(engine) => engine.service.next_policy_deadline(),
            Self::Taking(_) => None,
            Self::Worker(worker) => worker.service.next_policy_deadline(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::Engine(engine) => engine.service.is_empty(),
            Self::Taking(_) => false,
            Self::Worker(worker) => worker.service.is_empty(),
        }
    }
    fn operations(&self) -> u32 {
        match self {
            Self::Engine(engine) => engine.service.operations(),
            Self::Taking(taking) => taking.fetch.operations(),
            Self::Worker(worker) => worker.service.operations(),
        }
    }
}

#[derive(Default)]
struct Stop {
    steps: u32,
    admitted: [BTreeSet<Token>; 2],
    ready: bool,
    stopping: bool,
}

impl Referee<Process> for Stop {
    fn watch(&mut self, _now: Time, processes: &[Process]) -> Result<(), String> {
        let [Process::Engine(engine), first, second] = processes else {
            return Err("expected the engine and two worker hosts".into());
        };
        let links = [first, second].map(|process| match process {
            Process::Worker(worker) => worker.linked,
            Process::Taking(_) | Process::Engine(_) => 0,
        });
        if let Some(domain) = engine.service.engine_domain() {
            let channels = WORKERS.map(|(position, _)| domain.worker_channel(position));
            for (seen, channel) in self.admitted.iter_mut().zip(channels) {
                seen.extend(channel);
                if seen.len() > 1 {
                    return Err(format!("a worker was admitted more than once: {seen:?}"));
                }
            }
            if let [Some(first), Some(second)] = channels {
                self.ready = first != second && links.iter().all(|linked| *linked > 0);
            }
        }
        if !self.stopping {
            self.steps += 1;
            let engine_end = engine.service.end();
            let worker_ends = [first, second].map(|process| match process {
                Process::Worker(worker) => worker.service.end(),
                Process::Taking(_) | Process::Engine(_) => None,
            });
            if engine_end.is_some() || worker_ends.iter().any(Option::is_some) {
                return Err(format!("service ended before stop: {engine_end:?}; {worker_ends:?}"));
            }
            if self.steps >= START_STEPS {
                return Err(format!(
                    "no two run links after {START_STEPS} steps: channels {:?}; figures {:?}; links {links:?}",
                    self.admitted, engine.figures_sent
                ));
            }
        }
        Ok(())
    }

    fn act(&mut self, _now: Time, processes: &mut [Process]) {
        if self.ready
            && !self.stopping
            && let [Process::Engine(engine), Process::Worker(first), Process::Worker(second)] = processes
        {
            self.stopping = true;
            engine.service.shutdown();
            first.service.shutdown();
            second.service.shutdown();
        }
    }

    fn passed(&self) -> bool {
        self.stopping
    }
}

fn fixture_secret(name: &str) -> Box<[u8]> {
    format!("{name}-fixture-secret").into_bytes().into_boxed_slice()
}

fn engine(disk: &mut Disk, seed: u64) -> temper_service::Service {
    let mut document = documents::world_document();
    document["engine"]["hosts"]["listener"] = json!([{"socket": ENGINE}]);
    document["workers"] = json!([]);
    document.jig_over("fleet.workers", WORKERS.len());
    document.jig_over("fleet.attempts", WORKERS.len());
    let mut given = documents::given(&document);
    let temper_service::stock::Given::Own { forge, .. } = &mut given else { panic!("the world's typed settings") };
    let limits = temper_service::limits::engine(&document.jig, &document.limits).expect("engine limits");
    let agents = temper_service::stock::forge(&limits)
        .into_vec()
        .into_iter()
        .find(|setting| setting.kind == AGENTS_KIND)
        .expect("the stock agents setting");
    let mut settings = std::mem::take(forge).into_vec();
    settings.push(agents);
    *forge = settings.into_boxed_slice();
    document.as_object_mut().expect("deployment").remove("world_settings");
    let bytes = serde_json::to_vec(&document.document).expect("engine alone document");
    let seeds = documents::seeds(seed).service();
    let files = documents::files();
    let mut checked = temper_service::startup::checked_start_with_limits(
        &bytes,
        &Directories::working(),
        &files,
        None,
        &seeds,
        documents::forges(),
        None,
        &given,
        None,
        &document.jig,
        &document.limits,
    )
    .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
    seed_workers(disk, checked.engine_mut(), &seeds);
    temper_service::startup::from_checked(checked, &files, None, &seeds, None, None, None, None, Box::new([]))
        .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))))
}

/// The engine alone restores operator-owned worker settings, not worker defaults.
fn seed_workers(disk: &mut Disk, settings: &temper_service::settings::Engine, seeds: &temper_service::settings::Seeds) {
    let limits = temper_service::limits::engine(&settings.jig.limits, &settings.limits).expect("engine limits");
    let application =
        temper_protocol_store::application(&limits.forge, limits.tasks.spec_bytes).expect("store application");
    let checked = jig_protocol_store::runtime_limits_reusing(
        settings.store_limits.as_ref().expect("store envelope"),
        &temper_engine_domain::core_limits(&limits),
        application,
        limits.journal.transcript_bytes.max(limits.loads.transcript_bytes),
        None,
    )
    .expect("store limits");
    let mut store = temper_engine_service::store::Store::new(&limits, &checked, &[], seeds);
    store.prepared_schemas(temper_playground::programs::prepared_schemas());
    store.adopt(disk.root(0), USER).expect("fixture store root");
    let Event::Opened { last, pending, .. } = disk.event(&mut store) else { panic!("fixture store opened") };
    if pending {
        store.ask(Request::Switch { owner: SEED_WORKERS });
        assert!(matches!(disk.event(&mut store), Event::Switched { .. }));
    }
    assert_eq!(last, 0, "new deployment");
    let mut header = jig_domain_core::Counters::bootstrap(seeds.deployment).deployment();
    let mut writes = Vec::new();
    for (position, name) in WORKERS {
        let number = position + 1;
        let worker = settings::WorkerSetting {
            number,
            name: name.as_bytes().into(),
            places: PLACES,
            digest: Some(skein_lib::sha256::digest(&fixture_secret(name))),
            withdrawn: false,
        };
        writes.push(Write::Save(Record::Core(CoreRecord::Settings(settings::Record::Changed {
            setting: settings::Setting::Worker(number),
            changed: settings::Changed { value: Some(settings::Value::Worker(Box::new(worker))), revision: 1 },
        }))));
    }
    header.commits = last + 1;
    writes.push(Write::Save(Record::Core(CoreRecord::Deployment(header))));
    store.ask(Request::Commit { number: last + 1, writes: writes.into_boxed_slice() });
    assert!(matches!(disk.event(&mut store), Event::Committed { number, .. } if number == last + 1));
    disk.close(store);
}

/// Run the cluster with no client, agents or chats; the referee stops it on admission.
#[must_use]
pub fn run(seed: u64) -> Outcome {
    let mut setup = Disk::new(machine::fresh());
    let mut engine = engine(&mut setup, seed);
    let mut disk = setup.machine;
    let state = disk.reopen_root(0);
    let roots = WORKERS.map(|_| {
        disk.lay(&[skein_fake_machine::Item::directory(b"workspaces"), skein_fake_machine::Item::directory(b"trash")])
    });
    let mut world = World::new(seed, skein_sim::Config::calm(), Stop::default(), Memory::Unchecked)
        .with_machine(machine::Machine::new(disk));
    world.spawn_root(skein_sim::Handle::new(state.raw()), |root| {
        engine.adopt_store(root, USER).expect("the engine's own root");
        Process::Engine(Box::new(Engine { service: engine, figures_sent: [0; 2] }))
    });
    for ((position, name), root) in WORKERS.into_iter().zip(roots) {
        world.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
            Process::Taking(Box::new(Taking::new(name, root, seed ^ u64::from(position))))
        });
    }
    let outcome = world.run();
    let [Process::Engine(engine), Process::Worker(first), Process::Worker(second)] = outcome.procs.as_slice() else {
        panic!("engine and both workers settled");
    };
    Outcome {
        engine: engine.service.end().cloned(),
        figures_sent: engine.figures_sent,
        places_taken: [first.places, second.places],
        links: [first.linked, second.linked],
        workers: [first.service.end().cloned(), second.service.end().cloned()],
        settled: outcome.procs.iter().all(Host::is_empty),
    }
}
