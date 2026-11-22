//! Drive a deployment store, or its owning service, on the simulated kernel.
use skein_fake_machine::Machine;
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::Queue;
use skein_sim::{Config, Handle, Pid, Sim};
use temper_engine_service::store::Store;

pub use crate::upgrade::check::pre_settings::pre_settings_refusals_leave_the_whole_directory_unchanged;
pub use crate::upgrade::rehearsal::live::a_live_change_and_uncertain_write_reach_the_opening_without_network;

pub const USER: u32 = 1000;

pub struct Disk {
    pub machine: Machine,
    pub sim: Sim,
    pid: Pid,
    completed: Queue<Complete>,
}

impl Disk {
    #[must_use]
    pub fn new(machine: Machine) -> Self {
        let mut sim = Sim::new(0, Config::calm());
        let pid = sim.spawn_process();
        Self { machine, sim, pid, completed: Queue::uncapped() }
    }

    pub fn root(&mut self, index: u32) -> Fd {
        self.sim.root(self.pid, Handle::new(self.machine.reopen_root(index).raw()))
    }

    pub fn pass(&mut self, submitted: &mut Queue<Submit>, pending: bool) {
        self.sim.submit(self.pid, submitted);
        skein_fake_machine::serve(&mut self.machine, &mut self.sim);
        if !pending && self.sim.ready(self.pid) == 0 {
            let _ = self.sim.advance();
        }
        self.sim.reap(self.pid, &mut self.completed);
    }

    pub fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completed
    }

    pub fn tick(&mut self, store: &mut Store) {
        store.clock(self.sim.now(), self.sim.wall());
        store.iterate();
        let pending = store.work_pending(self.sim.now());
        self.pass(store.submissions(), pending);
        while let Some(complete) = self.completed.pop() {
            store.complete(complete);
        }
    }

    pub fn event(&mut self, store: &mut Store) -> jig_protocol_store::Event {
        use jig_protocol_store::Event;
        for _ in 0..10_000 {
            self.tick(store);
            while let Some(event) = store.events().pop() {
                match event {
                    Event::Census { .. } | Event::Ran { .. } => {}
                    event @ (Event::Job { .. } | Event::Site { .. }) => assert!(store.opening_site(event)),
                    event => return event,
                }
            }
        }
        panic!("fixture store request settles");
    }

    pub fn closed(&self) {
        self.sim.assert_no_open_fds(self.pid);
    }

    pub fn close(&mut self, mut store: Store) {
        store.close();
        for _ in 0..10_000 {
            self.tick(&mut store);
            if store.closed() {
                self.closed();
                return;
            }
        }
        panic!("fixture store closes");
    }
}
