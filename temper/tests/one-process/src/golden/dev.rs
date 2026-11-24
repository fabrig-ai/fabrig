//! Start the deployed build's immutable image (temper/docs/design/protocol/floor.md, 5).
use super::{Golden, Proc, UPGRADED, files, load_reader};
use crate::{
    ctl, fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, Story},
};
use jig_domain_core::{CoreRecord, Record};
use jig_domain_tasks::Stored;
use skein_io::kernel::ServiceSignal;
use skein_lib::Time;
use std::{cell::RefCell, collections::BTreeMap, path::Path, rc::Rc};
use temper_engine_domain::Record as Row;
use temper_playground::{
    Links,
    machine::{self, Machine, SignalHost},
    programs,
};

type Image = BTreeMap<Vec<u8>, Vec<u8>>;
const ENDED: [u64; 3] = [2, 3, 4];

struct Source {
    files: Image,
    extents: BTreeMap<Vec<u8>, usize>,
    layouts: Image,
}
impl Source {
    fn read(disk: &mut skein_fake_machine::Machine) -> Self {
        let files = generation(disk, b"generations/1/");
        let source = crate::upgrade::maps::Maps::generation(disk, 1);
        let extents: BTreeMap<Vec<u8>, usize> = source
            .store
            .iter()
            .filter(|(key, _)| key.starts_with(&jig_protocol_store::logs::EXTENTS))
            .map(|(key, value)| {
                let log = u64::from_be_bytes(key[2..].try_into().expect("log number"));
                let end = usize::try_from(u64::from_be_bytes(value.as_slice().try_into().expect("log extent")))
                    .expect("small log");
                (format!("generations/1/store/logs/{:016x}/{log:016x}", log >> 12).into_bytes(), end)
            })
            .collect();
        for (path, end) in &extents {
            assert_eq!(
                files.get(path).expect("each source log exists").len(),
                *end,
                "kept log ends at its committed extent"
            );
        }
        Self { files, extents, layouts: layouts(&source.store) }
    }
    fn unchanged(&self, disk: &mut skein_fake_machine::Machine) {
        let image = files::image_root(disk, 0);
        let after = generation(disk, b"generations/1/");
        for (path, bytes) in &self.files {
            let found = after.get(path).expect("every source file still present");
            if found != bytes {
                let (facts, _) = &image[&(0, path.clone())];
                assert!(facts.links > 1 && self.extents.contains_key(path), "only a shared log may append");
                let suffix = path.strip_prefix(b"generations/1/").expect("source path");
                let copy = [b"generations/2/".as_slice(), suffix].concat();
                assert!(&image[&(0, copy)].1 == found, "source log is shared with the started generation");
            }
            if let Some(end) = self.extents.get(path) {
                // The source reads its extent; the linked log may append beyond it (jig's logs.md, 7).
                assert!(
                    found.starts_with(bytes.get(..*end).expect("source log covers its extent")),
                    "source log prefix unchanged: {}",
                    String::from_utf8_lossy(path)
                );
            } else {
                assert!(found == bytes, "source bytes unchanged: {}", String::from_utf8_lossy(path));
            }
        }
        for (path, bytes) in after.iter().filter(|(path, _)| !self.files.contains_key(*path)) {
            assert!(
                bytes.is_empty()
                    && (path.starts_with(b"generations/1/store/log-")
                        || path.starts_with(b"generations/1/secrets/log-")),
                "only empty recovery segments added to source: {}",
                String::from_utf8_lossy(path)
            );
        }
    }
}

/// Load roots only: restoring and reading history needs no fake-forge image or ledger.
pub fn read_dev(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let (mut disk, configuration) = load_reader(directory);
    assert_eq!(files::active(&mut disk), b"1\n");
    let source = Source::read(&mut disk);
    let store = Rc::new(RefCell::new(Store::new()));
    files::transcripts(directory, &mut store.borrow_mut());
    let mut previous = None;
    for pass in 0..2 {
        UPGRADED.with(|lines| lines.borrow_mut().clear());
        store.borrow_mut().recovering();
        let (started, reads) = start(disk, &configuration, &store);
        disk = started;
        assert_eq!(files::active(&mut disk), b"2\n", "one registration, no generation on the second start");
        source.unchanged(&mut disk);
        let generations: Vec<_> = files::image_root(&mut disk, 0)
            .keys()
            .filter(|(_, path)| path.strip_prefix(b"generations/").is_some_and(|tail| !tail.contains(&b'/')))
            .map(|(_, path)| path.clone())
            .collect();
        assert_eq!(generations, [b"generations/1".to_vec(), b"generations/2".to_vec()]);
        let jobs = UPGRADED.with(|lines| lines.borrow().concat());
        if pass == 0 {
            assert_eq!(
                jobs,
                "jig escalation-deciders 3 2 tasks=0\ntemper test-ids-landing 0 1\ntemper test-ids-receipt 0 1 many=0\ntemper test-ids-tier 0 1 many=0 flakes=0 posted=0\ntemper test-ids-verification 0 1 many=0\ntemper deleted-repository-forge-rows 17 2 dropped=0 BranchHead=0 PullState=0 Ci=0 Client=0 Receipt=0 Tier=0 Verification=0 LandingLog=0 Application=0 Names=0\n",
                "only the kept story's pending jobs"
            );
            let registered = layouts(&crate::upgrade::maps::Maps::read(&mut disk).store);
            assert!(registered.keys().any(|key| !source.layouts.contains_key(key)), "this build registers new layouts");
            previous = Some((reads, registered));
        } else {
            assert!(jobs.is_empty(), "no job pending on the second start: {jobs}");
            let registered = layouts(&crate::upgrade::maps::Maps::read(&mut disk).store);
            assert_eq!(Some((reads, registered)), previous, "the same kept history and registry on both starts");
        }
    }
    world::lower_store(None);
}

/// The restart sweep uses the same loader, bridge, census and kept-token reads as `read_dev`.
pub(crate) fn restart_faults(
    directory: &Path,
    settings: world::Settings,
    injector: &Rc<RefCell<crate::restarts::Injector>>,
) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let (mut disk, configuration) = load_reader(directory);
    let source = Source::read(&mut disk);
    let store = Rc::new(RefCell::new(Store::new()));
    files::transcripts(directory, &mut store.borrow_mut());
    let passes = if injector.borrow().wants_signal() { 2 } else { 1 };
    let mut previous = None;
    for _ in 0..passes {
        UPGRADED.with(|lines| lines.borrow_mut().clear());
        store.borrow_mut().recovering();
        let (started, reads) = start_with(disk, &configuration, &store, settings, Some(injector));
        disk = started;
        source.unchanged(&mut disk);
        if let Some(previous) = previous {
            assert_eq!(reads, previous, "unit restart serves the same kept history");
        }
        previous = Some(reads);
    }
    world::lower_store(None);
}

fn layouts(rows: &Image) -> Image {
    rows.iter()
        .filter(|(key, _)| {
            matches!(jig_protocol_store::keys::decode_own(key), Some(jig_protocol_store::keys::Own::Layout { .. }))
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn generation(disk: &mut skein_fake_machine::Machine, prefix: &[u8]) -> Image {
    files::image_root(disk, 0)
        .into_iter()
        .filter(|((_, path), (facts, _))| path.starts_with(prefix) && facts.is == skein_fake_machine::Is::File)
        .map(|((_, path), (_, bytes))| (path, bytes))
        .collect()
}

fn start(
    disk: skein_fake_machine::Machine,
    configuration: &temper_playground::documents::Configuration,
    store: &Rc<RefCell<Store>>,
) -> (skein_fake_machine::Machine, Vec<serde_json::Value>) {
    start_with(disk, configuration, store, fixtures::calm(fixtures::SEED), None)
}

fn start_with(
    disk: skein_fake_machine::Machine,
    configuration: &temper_playground::documents::Configuration,
    store: &Rc<RefCell<Store>>,
    settings: world::Settings,
    injector: Option<&Rc<RefCell<crate::restarts::Injector>>>,
) -> (skein_fake_machine::Machine, Vec<serde_json::Value>) {
    world::lower_store(Some(store.clone()));
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let machine = Machine::new(disk);
    let params = programs::Parameters {
        deployment: configuration.clone().into(),
        seed: start.seed,
        links: Links::Keep,
        forge: start.fakes.forge.clone(),
        route: temper_playground::story::route(configuration),
        disk: machine.0.clone(),
    };
    let outcome = programs::with(params, || {
        let mut world =
            skein_world::World::new_controlled(settings.seed, settings.config, settings.memory, |controls| {
                if let Some(injector) = &injector {
                    injector.borrow_mut().attach(controls.clone());
                }
                Judge {
                    controls,
                    injector: injector.cloned(),
                    stopped: false,
                    checked: false,
                    finding: None,
                    store: store.clone(),
                    requests: Vec::new(),
                    seen: 0,
                    transport: start.fakes.forge.clone(),
                }
            })
            .with_machine(machine);
        for (program, roots) in programs::programs::<Golden>() {
            world.host_roots(program, roots);
        }
        world.spawn_restartable(programs::service_roots(), programs::service::<Golden>);
        world.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        world.spawn_restartable(
            vec![skein_world::StartupRoot { name: b"private".as_slice().into(), path: b"client".as_slice().into() }],
            |inherited| Proc::Other(HarnessProcess::Calls(Box::new(ctl::Caller::scripted_restartable(inherited)))),
        );
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(super::llm_peer(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(start.fakes.peer, None))));
        if let Some(injector) = &injector {
            world.run_with_inputs(|sim, pids, procs| injector.borrow_mut().inputs(sim, pids, procs))
        } else {
            world.run()
        }
    });
    if let Some(injector) = &injector {
        injector.borrow_mut().end_phase(outcome.end);
    }
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "dev reader drains every process");
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty());
    assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_engine_service::End::Drained)));
    let caller = outcome
        .procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Other(HarnessProcess::Calls(caller)) => Some(caller),
            _ => None,
        })
        .expect("kept-token ctl caller");
    let reads = caller
        .answers
        .iter()
        .enumerate()
        .map(|(index, answer)| {
            assert_eq!(
                answer.exit,
                skein_io::kernel::Exit::Code(0),
                "read {index}: {}",
                String::from_utf8_lossy(&answer.error)
            );
            serde_json::from_slice(&answer.output).expect("ctl JSON")
        })
        .collect::<Vec<serde_json::Value>>();
    assert_eq!(reads[0]["party"]["number"], 1);
    assert_eq!(reads[0]["party"]["login"]["bytes"], "owner");
    let chat = &reads[1]["activity"];
    assert_eq!(chat["cut"], false);
    assert!(chat["before"].is_null() && chat["after"].is_null(), "whole kept chat");
    let turns = chat["rows"]
        .as_array()
        .expect("chat activity")
        .iter()
        .filter_map(|row| {
            let name = row.pointer("/event/owner/decoded/name")?;
            Some((name["attempt"].as_u64().expect("attempt"), name["number"].as_u64().expect("turn")))
        })
        .collect::<Vec<_>>();
    assert_eq!(turns, [(1, 1), (1, 2), (4, 1)], "all kept chat turns served");
    assert!(serde_json::to_string(&reads[1]).expect("chat JSON").contains(super::CHAT_REPLY));
    let history = history(&reads);
    (std::mem::replace(&mut outcome.machine.0.borrow_mut(), machine::fresh()), history)
}

fn history(reads: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut kept = vec![reads[0].clone(), reads[1]["activity"].clone()];
    for (task, read) in ENDED.iter().zip(reads.iter().skip(2)) {
        let chip = &read["task"]["chip"];
        assert_eq!(chip["task"], *task);
        assert_eq!(chip["phase"], "ended");
        assert!(!chip.pointer("/details/owner/details/ending").expect("ended task's result").is_null());
        kept.push(chip.clone());
    }
    let mut positions: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for read in reads.iter().skip(2) {
        let page = read.get("activity").unwrap_or(read);
        assert_eq!(page["cut"], false);
        let rows = page["rows"].as_array().expect("task activity rows");
        for row in rows {
            let task =
                row.pointer("/event/owner/source/task").and_then(serde_json::Value::as_u64).expect("activity's task");
            positions.entry(task).or_default().push(row["position"].as_u64().expect("activity position"));
        }
        kept.push(page.clone());
    }
    assert_eq!(
        positions,
        BTreeMap::from([(2, vec![4, 3, 2, 1]), (3, vec![7, 6, 5, 4, 3, 2, 1]), (4, vec![5, 4, 3, 2, 1])]),
        "every kept ended task's complete history"
    );
    kept
}

fn assert_census(store: &Store) {
    let mut tasks = store
        .rows
        .values()
        .filter_map(|row| match row {
            Row::Core(Record::Tasks(Stored::Live(task) | Stored::Ended(task))) => {
                Some((task.number, matches!(task.phase, jig_domain_tasks::Phase::Ended(_))))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    tasks.sort_unstable();
    assert_eq!(tasks, [(1, false), (2, true), (3, true), (4, true), (5, false), (6, false)], "kept task census");
    let ended = store
        .rows
        .values()
        .filter_map(|row| match row {
            Row::Core(Record::Tasks(Stored::Ended(task))) => Some(task.number),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ended, ENDED, "ended tasks from the kept story");
    let refused = store.rows.values().filter(|row| matches!(row,
        Row::Core(Record::Core(CoreRecord::Call(call))) if matches!(call.part, jig_domain_core::CallPart::DelegationRefused(_)))).count();
    assert_eq!(refused, 0, "the kept image has no refused delegation");
}

struct Judge {
    controls: skein_world::Controls,
    injector: Option<Rc<RefCell<crate::restarts::Injector>>>,
    stopped: bool,
    checked: bool,
    finding: Option<String>,
    store: Rc<RefCell<Store>>,
    requests: Vec<Option<u64>>,
    seen: usize,
    transport: temper_fake_git_hosted::Typed,
}
impl Judge {
    #[expect(
        clippy::too_many_lines,
        reason = "the reader drains only after the complete kept history and optional fault"
    )]
    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { return Err("service first".into()) };
        if !self.stopped
            && let Some(end) = service.service.end()
        {
            return Err(format!("dev image start refused: {end:?}"));
        }
        if !self.checked {
            let mut forge = self.transport.forge.borrow_mut();
            assert_eq!(forge.domain.observations_lost(), 0, "all restore-time forge observations kept");
            while let Some(observation) = forge.domain.pop_observation() {
                assert!(
                    matches!(
                        observation,
                        skein_fake_forge_domain::Observation::Refused { .. }
                            | skein_fake_forge_domain::Observation::Rejected { .. }
                    ),
                    "restoration needs no forge mutation: {observation:?}"
                );
            }
        }
        if !self.checked && service.service.engine_domain().is_some_and(temper_engine_domain::Domain::ready) {
            if let Some(injector) = &self.injector {
                injector.borrow_mut().opening(now);
                crate::referee::Referee::check_roots(&service.service)?;
            }
            assert_census(&self.store.borrow());
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    caller.commands.push_back(ctl::Command::new(&[b"me", b"--json"]));
                    caller.commands.push_back(ctl::Command::new(&[b"chat", b"show", b"1", b"--json"]));
                    for task in ENDED {
                        caller.commands.push_back(ctl::Command::new(&[
                            b"task",
                            b"show",
                            task.to_string().as_bytes(),
                            b"--activity",
                            b"32",
                            b"--json",
                        ]));
                    }
                }
            }
            self.requests = [None, None].into_iter().chain(ENDED.into_iter().map(Some)).collect();
            self.checked = true;
        }
        if !self.stopped && self.checked {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    while self.seen < caller.answers.len() {
                        let answer = &caller.answers[self.seen];
                        if answer.exit != skein_io::kernel::Exit::Code(0) {
                            return Err(format!("ctl read {}: {}", self.seen, String::from_utf8_lossy(&answer.error)));
                        }
                        if let Some(task) = self.requests[self.seen] {
                            let value: serde_json::Value =
                                serde_json::from_slice(&answer.output).expect("task history JSON");
                            let page = value.get("activity").unwrap_or(&value);
                            assert_eq!(page["cut"], false, "history page is whole");
                            if let Some(before) = page.pointer("/before/value").and_then(serde_json::Value::as_str) {
                                caller.commands.push_back(ctl::Command::new(&[
                                    b"task",
                                    b"activity",
                                    task.to_string().as_bytes(),
                                    b"--before",
                                    before.as_bytes(),
                                    b"--limit",
                                    b"32",
                                    b"--json",
                                ]));
                                self.requests.push(Some(task));
                            }
                        }
                        self.seen += 1;
                    }
                    if self.seen == self.requests.len() {
                        caller.closed = true;
                        if let Some(injector) = &self.injector {
                            if !injector.borrow_mut().ready() {
                                continue;
                            }
                            if !injector.borrow().unit_active() {
                                self.controls.signal(0, ServiceSignal::Terminate);
                            }
                        } else {
                            self.controls.signal(0, ServiceSignal::Terminate);
                        }
                        if self.injector.is_some() {
                            crate::referee::Referee::check_roots(&service.service)?;
                        }
                        self.stopped = true;
                    }
                }
            }
        }
        if self.stopped {
            for proc in others {
                match proc {
                    Proc::Peer(peer) => peer.shutdown(),
                    Proc::Forge(peer) => peer.shutdown(),
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.inner.finding
            {
                return Err(why.clone());
            }
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.injector.as_ref().is_some_and(|injector| injector.borrow().waiting()) {
            return Some(Time::ZERO);
        }
        (!self.stopped).then_some(Time::from_nanos(60_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| "dev image reader did not settle".into())
    }
    fn passed(&self) -> bool {
        self.stopped && self.checked && self.finding.is_none()
    }
}
