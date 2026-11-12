//! Drivers lifecycle over the simulator and crashing filesystem (protocol/store/README.md, 8).
use super::{Drivers, Hosted, Lane, NotReady, Submission, check_read_page, max_read};
use crate::{Event, Store, tests::fixtures};
use skein_fake_machine::{How, Machine, serve};
use skein_io::kernel::{Complete, Error, Fd, Op};
use skein_lib::{Queue, Token};
use skein_sim::{Config, Handle, Pid, Sim};

struct World {
    host: Drivers,
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
        let host = Drivers::new(&fixtures::store_limits(), 1);
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
fn adoption_modes_open_rehearse_and_go_back_without_an_implicit_open() {
    let mut world = World::new();
    // The default Open must still create a deployment before the other modes can use it.
    world.open();
    world.close();
    let source = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(source.raw()));
    let scratch = world.machine.lay(&[]);
    let scratch = world.sim.root(world.pid, Handle::new(scratch.raw()));
    world.host = Drivers::new(&fixtures::store_limits(), 2);
    world.store = Store::on_directory(&fixtures::store_limits(), fixtures::APPLICATION, &[], world.host.seeded());
    world.host.adopt_mode(world.fd, 0, Token::new(41), fixtures::INSTALLATION, super::Adoption::Rehearse { scratch }).unwrap();
    world.settle();
    let Some(Event::Rehearsing { .. }) = world.above.pop() else { panic!("no implicit Open before Rehearsing") };
    let Some(Event::Opened { .. }) = world.above.pop() else { panic!("scratch opens") };
    assert!(world.above.is_empty());
    world.host.down(&mut world.store, &fixtures::env(), crate::Request::Close { owner: Token::new(41) }, &mut world.above);
    world.settle();
    let Some(Event::Closed { removed: true, .. }) = world.above.pop() else { panic!("scratch emptied") };
    world.close();
    let source = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(source.raw()));
    world.host = Drivers::new(&fixtures::store_limits(), 3);
    world.store = Store::on_directory(&fixtures::store_limits(), fixtures::APPLICATION, &[], world.host.seeded());
    world.host.adopt_mode(world.fd, 0, Token::new(41), fixtures::INSTALLATION, super::Adoption::Back).unwrap();
    world.settle();
    let Some(Event::BackRefused { why: crate::BackRefusal::NoPrevious, .. }) = world.above.pop() else { panic!("Back, not Open") };
    assert!(world.above.is_empty());
    world.close();
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
    world.host = Drivers::new(&fixtures::store_limits(), 2);
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

#[test]
fn failed_adoption_answers_once_and_can_still_close() {
    let mut world = World::new();
    world.adopt();
    world.host.iterate(&mut world.store, &fixtures::env(), &mut world.above);
    assert_eq!(world.host.submissions(), 2);
    while let Some(Submission { lane, submit }) = world.host.pop_submission() {
        world.host.complete(lane, Complete { op: submit.op, kind: submit.kind, result: Err(Error::Permission) });
    }
    world.settle();
    let Some(Event::OpenRefused { owner, why: crate::OpenRefusal::Failed }) = world.above.pop() else {
        panic!("failed root clone refuses adoption");
    };
    assert_eq!(owner, Token::new(41));
    assert!(world.above.is_empty(), "one terminal for the adoption");
    world.close();
    let mut host = Drivers::new(&fixtures::store_limits(), 1);
    host.close();
    assert!(host.closed());
    assert_eq!(host.adopt(Fd::new(999), 0, Token::new(42), [0; 16]), Err(Fd::new(999)));
}

#[test]
fn a_get_not_ready_follows_its_value() {
    const OWNER: Token = Token::new(9);
    for (not_ready, in_pass) in [(NotReady::InPass, true), (NotReady::Above, false)] {
        let mut hosted = Hosted::new(&fixtures::store_limits(), fixtures::APPLICATION, &[], 1, not_ready);
        let answer = hosted.secrets(OWNER, crate::Secrets::Get { owner: crate::SecretOwner::SignIn, number: 1 });
        assert_eq!(answer.is_some(), in_pass);
        assert_eq!(hosted.above.is_empty(), in_pass);
        let answer = answer.or(hosted.above.pop());
        let Some(Event::SecretsNotReady { owner }) = answer else { panic!("get is not ready before the mark") };
        assert_eq!(owner, OWNER);
        assert!(hosted.above.is_empty());
    }
}

#[test]
fn debug_of_the_hosted_store_counts_its_terminals() {
    let mut hosted = Hosted::new(&fixtures::store_limits(), fixtures::APPLICATION, &[], 1, NotReady::InPass);
    hosted.above.push(Event::Secret {
        owner: Token::new(9),
        value: Some(crate::SecretValue::Digest { expiry: 0, digest: [171; 32] }),
    });
    let debug = alloc::fmt::format(format_args!("{hosted:?}"));
    assert!(!debug.contains("171"));
    assert!(debug.contains("above: 1"));
}

#[test]
fn a_store_whose_reads_are_below_its_page_is_refused() {
    let mut limits = fixtures::store_limits();
    let page = limits.store.page.bytes;
    let below = page.checked_sub(1).unwrap();
    // Synthetic fields check admission without constructing an invalid KV store.
    limits.store.chunk = below;
    limits.core.resume_bytes = below;
    limits.core.load_bytes = u64::from(below);
    assert_eq!(max_read(&limits), below);
    assert_eq!(check_read_page(&limits), None);
    for (chunk, resume, load) in [(page, below, below), (below, page, below), (below, below, page)] {
        limits.store.chunk = chunk;
        limits.core.resume_bytes = resume;
        limits.core.load_bytes = u64::from(load);
        assert_eq!(max_read(&limits), page);
        assert_eq!(check_read_page(&limits), Some(()));
    }
}

#[test]
fn a_yielded_patch_walk_keeps_the_host_runnable_without_a_timer() {
    const BASE: crate::PatchSet = crate::PatchSet {
        owner: crate::Owner::Application,
        name: "fixture",
        families: &[crate::KeptFamily {
            name: "fixture-label",
            texts: &[jig_test_patches::patches::kept::fixture_label::TEXTS[0]],
            map: crate::KeptMap::Store,
            keys: &[],
            frames: None,
            flights: &[],
        }],
        patches: &[],
        fresh: Some("fresh"),
    };
    const BEFORE: crate::PatchSets = crate::PatchSets { sets: &[BASE] };
    const AFTER: crate::PatchSets =
        crate::PatchSets { sets: &[crate::PatchSet { patches: &["patch 1\nowner fixture 1\nmark fresh\n"], ..BASE }] };
    let mut world = World::new();
    world.store.patch_sets(BEFORE);
    world.open();
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        fixtures::commit(1, Box::new([fixtures::header(1)])),
        &mut world.above,
    );
    world.settle();
    world.above = Queue::uncapped();
    world.close();
    let handle = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(handle.raw()));
    world.host = Drivers::new(&fixtures::store_limits(), 2);
    world.store = Store::on_directory(&fixtures::store_limits(), fixtures::APPLICATION, &[], world.host.seeded());
    world.store.patch_sets(AFTER);
    world.adopt();
    let mut yielded = false;
    for _ in 0_u32..1000 {
        world.step();
        if crate::patching::phase::ready(&world.store) && !world.host.work_pending(fixtures::env().now) {
            let hosted = Hosted {
                component: world.store,
                drivers: world.host,
                above: Queue::uncapped(),
                secrets: Queue::uncapped(),
                sources: Queue::uncapped(),
                env: fixtures::env(),
                not_ready: NotReady::Above,
                closing: false,
            };
            assert!(
                hosted.work_pending(fixtures::env().now),
                "a yielded patch pass must wake the host without IO or a timer"
            );
            world.store = hosted.component;
            world.host = hosted.drivers;
            yielded = true;
            break;
        }
    }
    assert!(yielded, "the marked patch exercises a yielded pass: {:?}", world.above);
    world.settle();
    let mut opened = false;
    for event in &world.above {
        if let Event::Opened { pending: true, .. } = event {
            opened = true;
        }
    }
    assert!(opened, "copy opens before the switch");
    world.close();
}
