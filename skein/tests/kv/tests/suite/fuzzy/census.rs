//! Recount rows and the period's operations, retaining only overflow assignments.

use std::collections::{BTreeMap, BTreeSet};

use skein_kv::{CENSUS_HEADS, CENSUS_KEYSPACES, Census, CensusRule, Event, Limits, Op, Page, Request, Store};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

const OWNER: Token = Token::new(1);
const SEEDS: u64 = 16;
const COMMITS: u64 = 160;
type Counts = BTreeMap<Vec<u8>, ([u64; 6], BTreeMap<Option<u16>, u64>)>;

fn prefix(key: &[u8]) -> Vec<u8> {
    key[..key.len().min(2)].to_vec()
}

fn head(value: &[u8]) -> Option<u16> {
    if value.len() < 2 { None } else { Some(u16::from_be_bytes([value[0], value[1]])) }
}

struct Row {
    value: Vec<u8>,
    space: Vec<u8>,
    head: Option<u16>,
}

struct Model {
    since: u64,
    rows: BTreeMap<Vec<u8>, Row>,
    // Entries outlive their rows until the next snapshot starts.
    spaces: BTreeSet<Vec<u8>>,
    // Winning operations retain their assignment; counters are recounted on read.
    operations: Vec<(Vec<u8>, [u64; 3])>,
}

impl Model {
    fn new(since: u64) -> Self {
        Self { since, rows: BTreeMap::new(), spaces: BTreeSet::new(), operations: Vec::new() }
    }

    fn apply(&mut self, ops: &[Op], writes: bool) {
        for (index, op) in ops.iter().enumerate() {
            if ops[index + 1..].iter().any(|later| later.key() == op.key()) {
                continue;
            }
            let mut space = prefix(op.key());
            if !self.spaces.contains(&space) {
                let named = self.spaces.iter().filter(|key| !key.is_empty()).count();
                let folded = self.rows.iter().any(|(key, row)| row.space != prefix(key));
                if named == CENSUS_KEYSPACES || folded {
                    space.clear();
                }
            }
            self.spaces.insert(space.clone());
            if writes {
                let count = match op {
                    Op::Put { key, value } => [1, 0, u64::try_from(key.len() + value.len()).expect("written bytes")],
                    Op::Erase { key } => [0, 1, u64::try_from(key.len()).expect("written bytes")],
                };
                self.operations.push((space.clone(), count));
            }
            self.rows.remove(op.key());
            match op {
                Op::Put { key, value } => {
                    // Recount present heads after removing the overwritten row.
                    // Short values share None but are not bound overflow.
                    let heads: BTreeSet<_> = self
                        .rows
                        .values()
                        .filter(|row| row.space == space && row.head.is_some())
                        .map(|row| row.head)
                        .collect();
                    let folded = self.rows.values().any(|row| row.space == space && row.head != head(&row.value));
                    let mut assigned = head(value);
                    if !heads.contains(&assigned) && (heads.len() == CENSUS_HEADS - 1 || folded) {
                        assigned = None;
                    }
                    self.rows.insert(key.to_vec(), Row { value: value.to_vec(), space, head: assigned });
                }
                Op::Erase { .. } => {}
            }
        }
    }

    fn snapshot(&mut self, since: u64) {
        self.since = since;
        self.spaces.retain(|space| self.rows.values().any(|row| row.space == *space));
        self.operations.clear();
    }

    fn raw(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.rows.iter().map(|(key, row)| (key.clone(), row.value.clone())).collect()
    }

    fn counts(&self) -> Counts {
        let mut counts = Counts::new();
        for space in &self.spaces {
            counts.insert(space.clone(), ([0; 6], BTreeMap::new()));
        }
        for (space, writes) in &self.operations {
            let (count, _) = counts.get_mut(space).expect("operation's assigned keyspace");
            count[3] += writes[0];
            count[4] += writes[1];
            count[5] += writes[2];
        }
        for (key, row) in &self.rows {
            let (count, heads) = counts.get_mut(&row.space).expect("row's assigned keyspace");
            count[0] += 1;
            count[1] += u64::try_from(key.len()).expect("key bytes");
            count[2] += u64::try_from(row.value.len()).expect("value bytes");
            *heads.entry(row.head).or_default() += 1;
        }
        counts
    }

    fn check(&self, census: &Census, seed: u64, step: u64) {
        let actual: Vec<_> = census
            .keyspaces()
            .iter()
            .map(|space| {
                let heads: Vec<_> = space.heads().iter().map(|entry| (entry.head, entry.rows)).collect();
                (
                    space.prefix().to_vec(),
                    ([space.rows, space.keys, space.values, space.puts, space.erases, space.written], heads),
                )
            })
            .collect();
        let expected: Vec<_> = self
            .counts()
            .into_iter()
            .map(|(space, (count, heads))| (space, (count, heads.into_iter().collect::<Vec<_>>())))
            .collect();
        assert_eq!(census.since(), self.since, "seed {seed}, step {step}");
        assert_eq!(actual, expected, "seed {seed}, step {step}");
    }
}

fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn put(key: &[u8], value: &[u8]) -> Op {
    Op::Put { key: Box::from(key), value: Box::from(value) }
}

fn limits() -> Limits {
    Limits {
        key: 8,
        value: 8,
        ops: 256,
        commit: 8192,
        queued: 4,
        queued_bytes: 32768,
        budget: 1_048_576,
        segment: 4096,
        snapshot_after: u64::MAX,
        chunk: 256,
        page: Page { rows: 4, bytes: 64 },
        deadline: Duration::from_secs(1),
    }
}

fn open(world: &mut World) {
    world.request_no_fire(Request::Open { owner: OWNER, root: world.root });
    assert!(matches!(world.pop(), Event::Opened { .. }));
}

fn close(world: &mut World) {
    world.request_no_fire(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    assert_eq!(world.io.open_files(), 0);
}

fn random_ops(seed: &mut u64) -> Vec<Op> {
    let mut ops = Vec::new();
    for _ in 0..=draw(seed) % 4 {
        let space = if draw(seed).is_multiple_of(3) { 0 } else { u8::try_from(draw(seed) % 136).expect("space") };
        let row = if draw(seed).is_multiple_of(2) { 0 } else { u8::try_from(draw(seed) % 12).expect("row") };
        let key = match draw(seed) % 32 {
            0 => vec![],
            1 => vec![2],
            2 => vec![3, space],
            _ => vec![2, space, row],
        };
        let op = if draw(seed).is_multiple_of(3) {
            Op::Erase { key: key.into_boxed_slice() }
        } else {
            let mut value = vec![0; usize::try_from(draw(seed) % 7).expect("length")];
            if value.len() >= 2 {
                value[1] = u8::try_from(draw(seed) % 13).expect("head");
            }
            put(&key, &value)
        };
        ops.push(op);
    }
    ops
}

fn crossing_bounds(seed: &mut u64) -> Vec<Op> {
    let mut spaces: Vec<_> = (0..=CENSUS_KEYSPACES).map(|space| u8::try_from(space).expect("space byte")).collect();
    for index in (1..spaces.len()).rev() {
        let other = usize::try_from(draw(seed) % u64::try_from(index + 1).expect("length")).expect("index");
        spaces.swap(index, other);
    }
    let mut ops = Vec::new();
    for space in spaces {
        ops.push(put(&[2, space, 0], &[0, 0]));
    }
    for head in 1..=10_u8 {
        ops.push(put(&[2, 0, head], &[0, head]));
    }
    ops
}

#[test]
fn generated_commits_snapshots_and_reopens_match_the_census_recount() {
    for initial in 1..=SEEDS {
        let mut seed = initial;
        let mut world = World::untraced(limits(), &[]);
        world.store.count(CensusRule { prefix: [2; 256] });
        open(&mut world);
        let mut model = Model::new(1);
        let mut snapshot = BTreeMap::new();
        let mut since = 1;
        let mut log = Vec::new();
        let mut snapshots = 0;
        let mut reopens = 0;
        for number in 1..=COMMITS {
            let ops = if number == 1 { crossing_bounds(&mut seed) } else { random_ops(&mut seed) };
            world.request_no_fire(Request::Commit { owner: OWNER, ops: ops.clone().into_boxed_slice() });
            assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
            model.apply(&ops, true);
            log.push(ops);
            model.check(world.store.census(), initial, number);
            if number == 1 {
                assert_eq!(world.store.census().keyspaces().len(), CENSUS_KEYSPACES + 1);
                assert!(world.store.census().keyspaces().iter().any(|space| space.heads().len() == CENSUS_HEADS));
            }
            if draw(&mut seed).is_multiple_of(17) {
                // No concurrent commits: this independent snapshot is the map
                // at S; replay below uses exactly the later operations.
                snapshot = model.raw();
                since = number + 1;
                world.env.limits.snapshot_after = 1;
                world.fire_once();
                world.drive(true);
                world.env.limits.snapshot_after = u64::MAX;
                model.snapshot(since);
                log.clear();
                model.check(world.store.census(), initial, number);
                snapshots += 1;
            }
            if draw(&mut seed).is_multiple_of(19) {
                close(&mut world);
                world.store = Store::new(&world.env.limits);
                world.store.count(CensusRule { prefix: [2; 256] });
                open(&mut world);
                let before = model.raw();
                model = Model::new(since);
                for (key, value) in &snapshot {
                    model.apply(&[put(key, value)], false);
                }
                for ops in &log {
                    model.apply(ops, true);
                }
                assert_eq!(model.raw(), before, "recovery model, seed {initial}, number {number}");
                assert_eq!(world.store.last(), number);
                model.check(world.store.census(), initial, number);
                reopens += 1;
            }
        }
        assert!(snapshots > 0 && reopens > 0, "seed {initial} crosses both lifecycle transitions");
        close(&mut world);
    }
}
