//! Seeded file-operation crashes keep extents on whole durable frames (skein/docs/design/data/kv.md, 9).
//! Cases take their seeds as in skein/docs/design/testing/sweeps.md, 4.

use super::super::logs::{self, OWNER};
use skein_kv::{Direction, Event, LogRecord, Op, Request};
use skein_kv_tests::world::World;
use skein_world::{sweep::cases, this_test};
use std::{collections::BTreeMap, convert::Infallible};

const ROUNDS: u32 = 12;
const COMMITS: u32 = 3;
const FRAMES_PER_APPEND: u32 = 2;
const RECORD_BYTES: u32 = 16;
// All commits can append to the same log without a drop.
const LOG_BYTES: u32 = ROUNDS * COMMITS * FRAMES_PER_APPEND * RECORD_BYTES;

#[derive(Clone, Default)]
struct Model {
    logs: BTreeMap<u64, Vec<Vec<u8>>>,
    rows: BTreeMap<u8, Vec<u8>>,
}
pub(super) fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn draw_record(seed: &mut u64) -> Vec<u8> {
    vec![
        u8::try_from(draw(seed) % 256).expect("byte");
        1 + usize::try_from(draw(seed) % u64::from(RECORD_BYTES)).expect("length")
    ]
}

fn world() -> World {
    let mut limits = logs::limits();
    limits.page.bytes = LOG_BYTES;
    let mut world = World::new(limits);
    world.logs(b"e/");
    logs::open(&mut world, 0);
    world
}

fn read_whole_log(world: &mut World, log: u64) -> (Box<[LogRecord]>, Option<u64>) {
    logs::read(world, log, 0, Direction::Forward, LOG_BYTES)
}

#[test]
fn whole_log_read_fits_the_workload_bound_with_a_full_width_draw() {
    let mut seed = u64::MAX;
    let record = draw_record(&mut seed);
    assert_eq!(record.len(), usize::try_from(RECORD_BYTES).expect("length"));

    let mut world = world();
    let log = 0;
    for number in 1..=u64::from(ROUNDS * COMMITS) {
        let ops = (0..FRAMES_PER_APPEND).map(|_| logs::append(&world, log, &record)).collect();
        logs::commit(&mut world, ops, number);
    }
    let (records, next) = read_whole_log(&mut world, log);
    assert_eq!(next, None, "the full workload fits one read");
    assert_eq!(records.len(), usize::try_from(ROUNDS * COMMITS * FRAMES_PER_APPEND).expect("frame count"));
    assert!(records.iter().all(|found| found.record.as_deref() == Some(record.as_slice())));
}

#[test]
fn generated_payload_batches_crash_and_recover_against_the_model() {
    let test = this_test!("generated_payload_batches_crash_and_recover_against_the_model");
    cases::<_, Infallible, _>(test, 1..=16, |initial| {
        let mut seed = initial;
        let mut world = world();
        let mut model = Model::default();
        for round in 0..ROUNDS {
            let before = world.store.last();
            let mut versions = vec![model.clone()];
            for commit in 0..COMMITS {
                let mut next = versions.last().expect("prior commit").clone();
                let log = draw(&mut seed) % 4;
                let mut ops = Vec::new();
                if draw(&mut seed).is_multiple_of(4) {
                    ops.push(Op::Drop { log });
                    next.logs.remove(&log);
                } else {
                    let record = draw_record(&mut seed);
                    // Two frames in one commit exercise placement and batch coalescing.
                    for _ in 0..FRAMES_PER_APPEND {
                        ops.push(logs::append(&world, log, &record));
                        next.logs.entry(log).or_default().push(record.clone());
                    }
                }
                let key = u8::try_from(draw(&mut seed) % 4).expect("key");
                if draw(&mut seed).is_multiple_of(3) {
                    ops.push(Op::Erase { key: Box::from([b'k', key]) });
                    next.rows.remove(&key);
                } else {
                    let value = vec![u8::try_from(commit + round).expect("small value")];
                    ops.push(Op::Put { key: Box::from([b'k', key]), value: value.clone().into_boxed_slice() });
                    next.rows.insert(key, value);
                }
                world.start(Request::Commit { owner: OWNER, ops: ops.into_boxed_slice() });
                assert!(world.above.is_empty(), "seed {initial}, round {round}: admission");
                versions.push(next);
            }
            let steps = usize::try_from(draw(&mut seed) % 120).expect("step count");
            for _ in 0..steps {
                if !world.step(false) {
                    break;
                }
            }
            let acknowledged = world.store.last();
            world.crash(draw(&mut seed));
            world.request(Request::Open { owner: OWNER, root: world.root });
            let Event::Opened { last, .. } = world.pop() else { panic!("seed {initial}, round {round}: recovery") };
            assert!(
                last >= acknowledged && last <= before + u64::from(COMMITS),
                "seed {initial}, round {round}: contiguous prefix"
            );
            model = versions
                .get(usize::try_from(last - before).expect("prefix index"))
                .expect("a whole admitted commit")
                .clone();
            for log in 0..4 {
                let expected = model.logs.get(&log).cloned().unwrap_or_default();
                let (records, next) = read_whole_log(&mut world, log);
                assert_eq!(next, None, "seed {initial}, round {round}");
                let mut end = 0;
                for (record, expected) in records.iter().zip(&expected) {
                    assert_eq!(record.at, end);
                    assert_eq!(record.length, u32::try_from(expected.len()).expect("length"));
                    assert_eq!(
                        record.record.as_deref(),
                        Some(expected.as_slice()),
                        "seed {initial}, round {round}, log {log}"
                    );
                    end += world.store.frame(expected).expect("frame").len();
                    assert_eq!(record.end, end);
                }
                assert_eq!(records.len(), expected.len(), "seed {initial}, round {round}, log {log}");
                assert_eq!(world.store.log_end(log), end, "extent names exactly the model's whole frames");
            }
            for key in 0..4 {
                world.request(Request::Get { owner: OWNER, key: Box::from([b'k', key]) });
                let Event::Got { value, .. } = world.pop() else { panic!("map read") };
                assert_eq!(value.as_deref(), model.rows.get(&key).map(Vec::as_slice), "seed {initial}, round {round}");
            }
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
