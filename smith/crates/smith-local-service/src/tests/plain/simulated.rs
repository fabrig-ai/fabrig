//! Plain capture bounds and replacement over the fake machine (shell.md, section 4).

use super::{PLAIN_CHILDREN, PLAIN_FILE_BYTES, TOO_MANY_PLAIN_CHILDREN};
use crate::process::ProcessAdapter;
use skein_fake_machine::{How, Item, Machine, Opened};
use skein_lib::{List, Queue, Time, Token};
use smith_local_domain as local;

const SEEDS: [u64; 2] = [8, 19];
const OWNER: Token = Token::new(7);
const DEADLINE: Time = Time::from_nanos(u64::MAX);

struct PlainWorld {
    process: ProcessAdapter,
    machine: Machine,
    root: Opened,
    sim: skein_sim::Sim,
    pid: skein_sim::Pid,
    events: Queue<local::Event>,
}

impl PlainWorld {
    fn new(seed: u64, file_bytes: usize, children: u32) -> Self {
        let mut items = vec![Item::file(b"file", &vec![b'x'; file_bytes])];
        for child in 1..children {
            items.push(Item::file(format!("file-{child}").as_bytes(), b""));
        }
        let mut machine = Machine::new();
        let root = machine.lay(&items);
        let mut sim = skein_sim::Sim::new(seed, skein_sim::Config::calm());
        let pid = sim.spawn_process();
        let descriptor = sim.root(pid, skein_sim::Handle::new(root.raw()));
        let config = crate::tests::config();
        let mut process =
            ProcessAdapter::new(&config.limits.process, config.launch, config.endpoints, 1).unwrap();
        process.adopt_delivery_roots(Box::new([descriptor]), Box::new([]));
        Self { process, machine, root, sim, pid, events: Queue::uncapped() }
    }

    fn settle(&mut self) {
        crate::tests::settle_file_io(
            &mut self.process,
            &mut self.sim,
            &mut self.machine,
            self.pid,
            &mut self.events,
        );
    }

    fn capture(&mut self) -> Option<bool> {
        let mut directories = List::with_capacity(1);
        directories.push(0).unwrap();
        self.process.capture_plain(directories, DEADLINE);
        self.settle();
        self.process.take_capture_done()
    }

    fn resize(&mut self, bytes: usize) {
        let file = self.machine.open(self.root, b"file", How::Write).unwrap();
        self.machine.truncate(file, u64::try_from(bytes).unwrap()).unwrap();
        self.machine.close(file);
    }

    fn status(&mut self) -> (bool, local::Event) {
        self.process.plain_status(OWNER, 0, DEADLINE, &mut self.events);
        let immediate = !self.events.is_empty();
        self.settle();
        let event = self.events.pop().expect("one plain status terminal");
        assert!(self.events.is_empty(), "status terminates once");
        (immediate, event)
    }

    fn close(&mut self) {
        self.process.close();
        self.settle();
        assert!(self.process.closed());
        assert!(self.events.is_empty(), "close emits no additional status terminal");
        assert_eq!(self.machine.open_handles(), 0, "all scan handles and the adopted root close");
    }
}

fn assert_status(event: local::Event, expected: bool) {
    let local::Event::PlainStatus { owner, changed } = event else { panic!("plain status: {event:?}") };
    assert_eq!(owner, OWNER);
    assert_eq!(changed, expected);
}

/// Lower test of smith-local-service's tests::plain::an_unknown_plain_capture_completes_and_counts_as_changed
/// (top tiers test wiring only).
#[test]
fn an_unknown_capture_completes_and_answers_changed_immediately() {
    for seed in SEEDS {
        for (bytes, children) in [(PLAIN_FILE_BYTES + 1, 1), (0, PLAIN_CHILDREN + 1), (0, TOO_MANY_PLAIN_CHILDREN)] {
            let mut world = PlainWorld::new(seed, bytes, children);
            assert_eq!(world.capture(), Some(true), "seed {seed}, {bytes} bytes, {children} children");
            let (immediate, event) = world.status();
            assert!(immediate, "unknown capture skips the comparison scan");
            assert_status(event, true);
            world.close();
        }
    }
}

/// Lower test of smith-local-service's tests::plain::a_plain_tree_outgrowing_its_capture_counts_as_changed
/// (top tiers test wiring only).
#[test]
fn a_tree_outgrowing_its_capture_counts_as_changed() {
    for seed in SEEDS {
        let mut world = PlainWorld::new(seed, PLAIN_FILE_BYTES, 1);
        assert_eq!(world.capture(), Some(true));
        world.resize(PLAIN_FILE_BYTES + 1);
        assert_status(world.status().1, true);
        world.close();
    }
}

/// Lower test of smith-local-service's tests::plain::an_untouched_plain_tree_at_the_bounds_is_unchanged
/// (top tiers test wiring only).
#[test]
fn an_untouched_tree_at_both_bounds_is_unchanged() {
    for seed in SEEDS {
        let mut world = PlainWorld::new(seed, PLAIN_FILE_BYTES, PLAIN_CHILDREN);
        assert_eq!(world.capture(), Some(true));
        assert_status(world.status().1, false);
        world.close();
    }
}

/// Lower test of smith-local-service's tests::plain::a_fresh_capture_replaces_an_unknown_plain_tree
/// (top tiers test wiring only).
#[test]
fn a_fresh_capture_replaces_an_unknown_tree() {
    for seed in SEEDS {
        let mut world = PlainWorld::new(seed, PLAIN_FILE_BYTES + 1, 1);
        let first = world.capture();
        let (_, unknown) = world.status();
        world.resize(b"small".len());
        let fresh = world.capture();
        let (_, unchanged) = world.status();
        assert_eq!([first, fresh], [Some(true), Some(true)]);
        assert_status(unknown, true);
        assert_status(unchanged, false);
        world.close();
    }
}
