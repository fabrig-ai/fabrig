//! Directory opens and restarts on the fake machine (jig protocol/store/jobs.md, 2–3).
use crate::{Service, store::Store};
use jig_protocol_store::{Event, Get, PatchSet, PatchSets, Request, Row};
use skein_fake_machine::{Machine, serve};
use skein_lib::{Queue, Time, Token, Wall};
use skein_sim::{Config, Handle, Pid, Sim};

pub const JOB: &str = "deleted-repository-forge-rows";
const PASSES: u32 = 10_000; // Bounded calm directory operations, including generation copying.

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary Rust fixture builds a static patch set with only the pending job omitted"
)]
fn old_sets() -> PatchSets {
    PatchSets {
        sets: Box::leak(
            crate::store::PATCH_SETS
                .sets
                .iter()
                .map(|set| {
                    let jobs = Box::leak(
                        set.jobs.iter().copied().filter(|job| job.name != JOB).collect::<Vec<_>>().into_boxed_slice(),
                    );
                    PatchSet { jobs, ..*set }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}

pub struct Directory {
    machine: Machine,
    sim: Sim,
    pid: Pid,
    fd: skein_io::kernel::Fd,
}
impl Directory {
    pub fn new() -> Self {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(&[]);
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        Self { machine, sim, pid, fd }
    }
    pub fn service(&self) -> Service {
        self.service_with_sets(None)
    }
    fn service_with_sets(&self, sets: Option<PatchSets>) -> Service {
        let settings = super::start_settings(super::engine(), &[(1, 100)]);
        let mut service = Service::engine(settings, &super::seeds(), None, super::forge_runtime()).expect("engine");
        let store = &mut service.engine.as_mut().expect("engine").store;
        if let Some(sets) = sets {
            store.patch_sets(sets);
        }
        store.adopt(self.fd, 0).expect("directory adopted");
        service
    }
    pub fn turn(&mut self, service: &mut Service) {
        let engine = service.engine.as_mut().expect("engine");
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
        self.io(&mut engine.store);
    }
    fn io(&mut self, store: &mut Store) {
        store.iterate();
        self.sim.submit(self.pid, &mut store.submitted);
        serve(&mut self.machine, &mut self.sim);
        let mut completions = Queue::uncapped();
        self.sim.reap(self.pid, &mut completions);
        while let Some(complete) = completions.pop() {
            store.complete(complete);
        }
    }
    /// Complete a first start under the old job list, append the story's rows, then close.
    pub fn seed(&mut self, rows: Box<[temper_engine_domain::Write]>) {
        let mut service = self.service_with_sets(Some(old_sets()));
        for _ in 0..PASSES {
            self.turn(&mut service);
            assert_eq!(service.end(), None, "first start");
            if service.engine.as_ref().expect("engine").domain.ready() {
                break;
            }
        }
        // Let the component finish the secret bootstrap that follows its startup mark.
        for _ in 0_u32..256 {
            self.turn(&mut service);
            assert_eq!(service.end(), None, "first start bootstrap");
        }
        let engine = service.engine.as_mut().expect("engine");
        assert!(engine.domain.ready(), "completed first start");
        assert!(engine.store.host.mark_durable());
        super::commit(&mut engine.store, 1, rows);
        let mut committed = false;
        for _ in 0..PASSES {
            self.io(&mut engine.store);
            if let Some(event) = engine.store.host.above.pop() {
                committed = true;
                let Event::Committed { .. } = event else { panic!("fixture commit: {event:?}") };
                break;
            }
        }
        assert!(committed, "fixture commit answered");
        assert!(engine.store.idle(), "fixture commit settled");
        engine.store.close();
        for _ in 0..PASSES {
            self.io(&mut engine.store);
            if engine.store.closed() {
                break;
            }
        }
        assert!(engine.store.closed(), "directory closed");
        drop(service);
        self.machine.crash(1);
        let handle = self.machine.reopen_root(0);
        self.sim = Sim::new(1, Config::calm());
        self.pid = self.sim.spawn_process();
        self.fd = self.sim.root(self.pid, Handle::new(handle.raw()));
    }
    pub fn get(&mut self, store: &mut Store, key: &temper_engine_domain_forge::Key) -> Option<Row> {
        let owner = Token::new(991);
        let key = temper_protocol_store::keys::encode(key, u32::MAX).expect("fixture key");
        store.host.ask(Request::Get { owner, key: Get::Application(key) });
        for _ in 0..PASSES {
            self.io(store);
            for _ in 0..store.host.above.len() {
                let event = store.host.above.pop().expect("store answer");
                if let Event::Got { owner: found, .. } = &event
                    && *found == owner
                {
                    let Event::Got { row, .. } = event else { unreachable!("matched get") };
                    return row;
                }
                store.host.above.push(event);
            }
        }
        panic!("directory get answered");
    }
}
