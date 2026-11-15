//! Seeded file-operation crashes keep extents on whole durable frames (data/kv.md, 9).

use super::super::logs::{self, OWNER};
use skein_kv::{Direction, Event, Op, Request};
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct Model {
    logs: BTreeMap<u64, Vec<Vec<u8>>>,
    rows: BTreeMap<u8, Vec<u8>>,
}
fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[test]
fn generated_payload_batches_crash_and_recover_against_the_model() {
    for initial in 1..=16 {
        let mut seed = initial;
        let mut world = logs::world();
        let mut model = Model::default();
        for round in 0..12 {
            let before = world.store.last();
            let mut versions = vec![model.clone()];
            for commit in 0..3 {
                let mut next = versions.last().expect("prior commit").clone();
                let log = draw(&mut seed) % 4;
                let mut ops = Vec::new();
                if draw(&mut seed).is_multiple_of(4) {
                    ops.push(Op::Drop { log });
                    next.logs.remove(&log);
                } else {
                    let record = vec![
                        u8::try_from(draw(&mut seed) % 256).expect("byte");
                        1 + usize::try_from(draw(&mut seed) % 16).expect("length")
                    ];
                    // Two frames in one commit exercise placement and batch coalescing.
                    for _ in 0..2 {
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
            assert!(last >= acknowledged && last <= before + 3, "seed {initial}, round {round}: contiguous prefix");
            model = versions
                .get(usize::try_from(last - before).expect("prefix index"))
                .expect("a whole admitted commit")
                .clone();
            for log in 0..4 {
                let expected = model.logs.get(&log).cloned().unwrap_or_default();
                let (records, next) = logs::read(&mut world, log, 0, Direction::Forward, 1024);
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
    }
}
