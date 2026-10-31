//! Host lifecycle over the simulator and crashing filesystem (store.md, 13).
use super::{Host, Lane, Submission};
use crate::{Event, Store, tests::fixtures};
use skein_fake_machine::{How, Machine, serve};
use skein_io::kernel::{Fd, Op};
use skein_lib::{Queue, Token};
use skein_sim::{Config, Handle, Pid, Sim};

struct World {
    host: Host,
    store: Store,
    machine: Machine,
    sim: Sim,
    pid: Pid,
    fd: Fd,
    above: Queue<Event>,
}
impl World {
    fn new() -> Self {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(&[]);
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let host = Host::new(&fixtures::store_limits(), 1);
        let store = Store::on_directory(&fixtures::store_limits(), fixtures::APPLICATION, &[], host.seeded());
        Self { host, store, machine, sim, pid, fd, above: Queue::uncapped() }
    }
    fn adopt(&mut self) {
        assert_eq!(self.host.adopt(self.fd, 0, Token::new(41), fixtures::INSTALLATION), Ok(()));
    }
    fn step(&mut self) {
        self.host.iterate(&mut self.store, &fixtures::env(), &mut self.above);
        let mut submitted = Queue::with_capacity(super::KERNEL_HANDOFF_ENTRIES);
        while let Some(Submission { lane, mut submit }) = self.host.pop_submission() {
            let tag = u8::try_from(lane.index().checked_add(1).expect("three lanes")).expect("small lane");
            submit.op = Token::tagged(tag, submit.op.raw()).expect("local token");
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::tagged(tag, target.raw()).expect("local target");
            }
            submitted.push(submit);
        }
        self.sim.submit(self.pid, &mut submitted);
        serve(&mut self.machine, &mut self.sim);
        let mut completed = Queue::with_capacity(super::KERNEL_HANDOFF_ENTRIES);
        self.sim.reap(self.pid, &mut completed);
        while let Some(mut complete) = completed.pop() {
            let index = usize::from(complete.op.tag()).checked_sub(1).expect("lane tag");
            let lane = *Lane::ALL.get(index).expect("three lanes");
            complete.op = Token::new(complete.op.untagged());
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.untagged());
            }
            self.host.complete(lane, complete);
        }
    }
    fn settle(&mut self) {
        for _ in 0_u32..1000 {
            self.step();
            if self.host.idle() && !self.host.work_pending(fixtures::env().now) {
                return;
            }
        }
        panic!("calm host settles");
    }
    fn open(&mut self) {
        self.adopt();
        self.settle();
        let Some(Event::Opened { owner, last: 0, .. }) = self.above.pop() else { panic!("empty store opens") };
        assert_eq!(owner, Token::new(41));
        assert!(self.above.is_empty());
    }
    fn close(&mut self) {
        self.host.close();
        self.settle();
        assert!(self.host.closed());
        self.sim.assert_no_open_fds(self.pid);
        assert_eq!(self.machine.open_handles(), 0);
    }
}

#[test]
fn a_commit_waits_for_its_driver_and_is_recovered_after_close() {
    let mut world = World::new();
    world.open();
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        fixtures::commit(1, Box::new([fixtures::header(1)])),
        &mut world.above,
    );
    assert!(world.above.is_empty());
    world.settle();
    let Some(Event::Committed { number: 1, .. }) = world.above.pop() else { panic!("first commit is durable") };
    world.close();
    let handle = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(handle.raw()));
    world.host = Host::new(&fixtures::store_limits(), 2);
    world.store = Store::on_directory(&fixtures::store_limits(), fixtures::APPLICATION, &[], world.host.seeded());
    world.adopt();
    world.settle();
    let Some(Event::Opened { last: 1, .. }) = world.above.pop() else { panic!("committed map recovers") };
    world.close();
}

#[test]
fn generation_creation_uses_the_seeded_host_drivers() {
    let mut world = World::new();
    world.open();
    let root = world.machine.reopen_root(0);
    let active = world.machine.open(root, b"active", How::Read).expect("active generation");
    assert_eq!(world.machine.read(active, 0, 16).expect("active bytes"), b"1\n");
    world.machine.close(active);
    let manifest = world.machine.open(root, b"generations/1/manifest", How::Read).expect("generation manifest");
    let bytes = world.machine.read(manifest, 0, super::max_whole(&fixtures::store_limits())).expect("manifest bytes");
    assert_eq!(crate::codec::component::Manifest::decode(&bytes).expect("manifest").generation, 1);
    world.machine.close(manifest);
    world.machine.close(root);
    world.close();
}

#[test]
fn adoption_is_single_use_and_closes_the_owned_root() {
    let mut world = World::new();
    world.open();
    assert_eq!(world.host.adopt(Fd::new(999), 0, Token::new(42), [0; 16]), Err(Fd::new(999)));
    world.close();
    assert_eq!(world.host.adopt(Fd::new(999), 0, Token::new(42), [0; 16]), Err(Fd::new(999)));
}

#[test]
fn close_drains_inflight_adoption_before_closing_the_root() {
    let mut world = World::new();
    world.adopt();
    world.step();
    assert!(!world.host.idle());
    world.close();
    assert!(world.above.is_empty());
}
