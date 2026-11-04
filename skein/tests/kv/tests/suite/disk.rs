//! Retained log follows the live map (skein/docs/design/kv.md, 5, 7 and 9).

use std::collections::BTreeMap;

use skein_kv::{Event, Limits, Op, Page, Request, SNAPSHOT_FLOOR, SNAPSHOT_MULTIPLE};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

fn limits() -> Limits {
    Limits {
        key: 8,
        value: 64,
        ops: 16,
        commit: 2048,
        queued: 8,
        queued_bytes: 16_384,
        budget: 262_144,
        segment: 8192,
        snapshot_after: 16_777_216,
        chunk: 1024,
        page: Page { rows: 16, bytes: 1024 },
        deadline: Duration::from_secs(1),
    }
}

fn retained_log(files: &BTreeMap<Box<[u8]>, Box<[u8]>>) -> u64 {
    files
        .iter()
        .filter(|(name, _)| name.starts_with(b"log-"))
        .map(|(_, bytes)| u64::try_from(bytes.len()).expect("small file"))
        .sum()
}

fn snapshot_bound(model: &BTreeMap<Box<[u8]>, Box<[u8]>>, limits: &Limits) -> u64 {
    let live: u64 =
        model.iter().map(|(key, value)| u64::try_from(8 + key.len() + value.len()).expect("small row")).sum();
    SNAPSHOT_MULTIPLE.saturating_mul(live).max(SNAPSHOT_FLOOR).min(limits.snapshot_after)
}

fn commit(world: &mut World, model: &mut BTreeMap<Box<[u8]>, Box<[u8]>>, number: u64, ops: Vec<Op>) {
    world.request(Request::Commit { owner: Token::new(number), ops: ops.clone().into_boxed_slice() });
    assert_eq!(
        world.pop(),
        Event::Committed { owner: Token::new(number), number },
        "commit {number}: retained log {}, bound {}",
        retained_log(&world.files()),
        snapshot_bound(model, &world.env.limits)
    );
    for op in ops {
        match op {
            Op::Put { key, value } => {
                model.insert(key, value);
            }
            Op::Erase { key } => {
                model.remove(&key);
            }
        }
    }
}

#[derive(Default)]
struct Observed {
    most_log: u64,
    bound_at_most: u64,
    snapshots: u64,
    start: u64,
}

impl Observed {
    fn check(&mut self, world: &mut World, model: &BTreeMap<Box<[u8]>, Box<[u8]>>, number: u64) {
        let files = world.files();
        let log = retained_log(&files);
        let bound = snapshot_bound(model, &world.env.limits);
        assert!(
            log <= bound + world.env.limits.segment,
            "commit {number}: retained log {log}, bound {bound}, segment {}",
            world.env.limits.segment
        );
        if log > self.most_log {
            self.most_log = log;
            self.bound_at_most = bound;
        }
        if let Some(snapshot) = files.get(&b"snapshot"[..]) {
            let start = u64::from_be_bytes(snapshot[6..14].try_into().expect("snapshot start"));
            if start != self.start {
                // Ten commits write less than the floor, so each observation sees at most one new snapshot.
                self.snapshots += 1;
                self.start = start;
            }
        }
    }

    fn report(&self) {
        println!(
            "most retained log {} bytes, bound {} bytes plus segment 8192 bytes; {} snapshots; last start {}",
            self.most_log, self.bound_at_most, self.snapshots, self.start
        );
    }
}

#[test]
fn a_tiny_store_keeps_its_log_under_the_floor() {
    let mut world = World::new(limits());
    let mut model = BTreeMap::new();
    let mut observed = Observed::default();
    world.request(Request::Open { owner: Token::new(0), root: world.root });
    assert_eq!(
        world.pop(),
        Event::Opened { owner: Token::new(0), last: 0 },
        "commit 0: retained log {}, bound {}",
        retained_log(&world.files()),
        snapshot_bound(&model, &world.env.limits)
    );
    for number in 1..=2000_u64 {
        let key = u16::try_from((number - 1) % 16).expect("sixteen keys").to_be_bytes();
        let value = [u8::try_from(number % 251).expect("small byte"); 64];
        commit(&mut world, &mut model, number, vec![Op::Put { key: Box::from(key), value: Box::from(value) }]);
        if number % 10 == 0 {
            observed.check(&mut world, &model, number);
        }
    }
    observed.check(&mut world, &model, 2000);
    assert!(
        observed.start > 1000,
        "commit 2000: retained log {}, bound {}, snapshot start {}",
        retained_log(&world.files()),
        snapshot_bound(&model, &world.env.limits),
        observed.start
    );
    observed.report();
}

#[test]
fn a_churning_store_keeps_its_log_within_twice_its_map() {
    let mut world = World::new(limits());
    let mut model = BTreeMap::new();
    let mut observed = Observed::default();
    world.request(Request::Open { owner: Token::new(0), root: world.root });
    assert_eq!(
        world.pop(),
        Event::Opened { owner: Token::new(0), last: 0 },
        "commit 0: retained log {}, bound {}",
        retained_log(&world.files()),
        snapshot_bound(&model, &world.env.limits)
    );
    for number in 1..=32_u64 {
        let ops = (0..16_u64)
            .map(|offset| {
                let key = u16::try_from((number - 1) * 16 + offset).expect("512 keys").to_be_bytes();
                Op::Put { key: Box::from(key), value: Box::from([0; 64]) }
            })
            .collect();
        commit(&mut world, &mut model, number, ops);
        if number % 10 == 0 {
            observed.check(&mut world, &model, number);
        }
    }
    for churn in 1..=1500_u64 {
        let number = 32 + churn;
        let key = u16::try_from((churn - 1) % 512).expect("512 keys").to_be_bytes();
        let value = [u8::try_from(churn % 251).expect("small byte"); 64];
        commit(&mut world, &mut model, number, vec![Op::Put { key: Box::from(key), value: Box::from(value) }]);
        if number % 10 == 0 {
            observed.check(&mut world, &model, number);
        }
    }
    observed.check(&mut world, &model, 1532);
    assert!(
        observed.most_log > SNAPSHOT_FLOOR,
        "commit 1532: most retained log {}, bound {} exceeds floor {}",
        observed.most_log,
        observed.bound_at_most,
        SNAPSHOT_FLOOR
    );
    let log = retained_log(&world.files());
    let bound = snapshot_bound(&model, &world.env.limits);
    for (key, value) in &model {
        world.request(Request::Get { owner: Token::new(1533), key: key.clone() });
        assert_eq!(
            world.pop(),
            Event::Got { owner: Token::new(1533), value: Some(value.clone()) },
            "commit 1532: retained log {log}, bound {bound}, key {key:?}"
        );
    }
    observed.report();
}
