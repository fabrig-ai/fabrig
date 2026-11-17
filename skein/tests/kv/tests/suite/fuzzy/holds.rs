//! Seeded holds race snapshots, rollovers, payload drops and file-operation crashes (data/kv.md, 9).

use super::super::logs::{self, OWNER};
use skein_kv::{Event, Op, Range, Request};
use skein_kv_tests::world::World;
use skein_lib::Token;
use std::collections::BTreeMap;

type Rows = BTreeMap<Vec<u8>, Vec<u8>>;

#[derive(Clone, Default)]
struct Model {
    rows: Rows,
    logs: BTreeMap<u64, Vec<Vec<u8>>>,
    extents: BTreeMap<u64, u64>,
}

fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn admit(world: &mut World, versions: &mut Vec<Model>, seed: &mut u64) {
    let mut model = versions.last().expect("initial model").clone();
    let log = draw(seed) % 4;
    let mut extent_key = b"e/".to_vec();
    extent_key.extend_from_slice(&log.to_be_bytes());
    let payload = if draw(seed).is_multiple_of(4) {
        model.logs.remove(&log);
        model.extents.remove(&log);
        model.rows.remove(&extent_key);
        Op::Drop { log }
    } else {
        let record = vec![
            u8::try_from(draw(seed) % 256).expect("byte");
            1 + usize::try_from(draw(seed) % 16).expect("record length")
        ];
        let frame = world.store.frame(&record).expect("bounded record");
        let end = model.extents.entry(log).or_default();
        *end += frame.len();
        model.rows.insert(extent_key, end.to_be_bytes().to_vec());
        model.logs.entry(log).or_default().push(record);
        Op::Append { log, frame }
    };
    let key = vec![b'k', u8::try_from(draw(seed) % 8).expect("key")];
    let row = if draw(seed).is_multiple_of(3) {
        model.rows.remove(&key);
        Op::Erase { key: key.into_boxed_slice() }
    } else {
        let value = draw(seed).to_be_bytes().to_vec();
        model.rows.insert(key.clone(), value.clone());
        Op::Put { key: key.into_boxed_slice(), value: value.into_boxed_slice() }
    };
    world.start(Request::Commit { owner: OWNER, ops: Box::new([row, payload]) });
    versions.push(model);
}

fn rows(world: &mut World) -> Rows {
    world.start(Request::Load {
        owner: OWNER,
        range: Range { start: Box::new([]), end: None },
        max: world.env.limits.page,
    });
    let Event::Loaded { rows, next: None, .. } = world.pop() else { panic!("whole map fits one page") };
    rows.into_vec().into_iter().map(|row| (row.key.into_vec(), row.value.into_vec())).collect()
}

fn check(world: &mut World, model: &Model, context: &str) {
    assert_eq!(rows(world), model.rows, "{context}: map including extents");
    for log in 0..4 {
        assert_eq!(world.store.log_end(log), model.extents.get(&log).copied().unwrap_or(0), "{context}: extent {log}");
        assert_eq!(
            logs::records(world, log),
            model.logs.get(&log).cloned().unwrap_or_default(),
            "{context}: payload {log}"
        );
    }
}

fn commits(world: &mut World) {
    while let Some(event) = world.above.pop() {
        assert!(matches!(event, Event::Committed { .. }), "commit accepted: {event:?}");
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one seeded story captures a hold, changes or crashes its source, then recovers its copy"
)]
fn copied_holds_recover_the_model_while_the_serving_store_runs_or_crashes() {
    let mut snapshots = 0;
    let mut linked_segments = 0;
    let mut copied_segments = 0;
    let mut crashes = 0;
    for initial in 1..=32 {
        let mut seed = initial;
        let mut limits = logs::limits();
        limits.segment = 80 + draw(&mut seed) % 160;
        limits.snapshot_after = 160 + draw(&mut seed) % 400;
        let mut world = World::new(limits);
        world.logs(b"e/");
        logs::open(&mut world, 0);
        let mut versions = vec![Model::default()];
        for _ in 0..12 {
            admit(&mut world, &mut versions, &mut seed);
            world.drive(draw(&mut seed).is_multiple_of(3));
            commits(&mut world);
            // Leave a snapshot at a randomly drawn chunk or replacement phase.
            for _ in 0..draw(&mut seed) % 4 {
                if world.ready() {
                    world.fire_once();
                }
            }
        }
        let before = draw(&mut seed) % 4;
        for _ in 0..before {
            admit(&mut world, &mut versions, &mut seed);
        }
        world.start(Request::Hold { owner: OWNER });
        let expected_last = u64::try_from(versions.len() - 1).expect("last admitted before hold");
        for _ in 0..3 {
            admit(&mut world, &mut versions, &mut seed);
        }
        let mut held = None;
        for _ in 0..1000 {
            while let Some(event) = world.above.pop() {
                match event {
                    Event::Committed { number, .. } => assert!(number <= expected_last, "seed {initial}: barrier"),
                    Event::Held { last, snapshot, segments, .. } => {
                        assert_eq!(last, expected_last, "seed {initial}");
                        assert_eq!(world.store.last(), last, "copy before later commits are answered");
                        snapshots += u32::from(snapshot);
                        linked_segments += segments.iter().filter(|s| s.end.is_none()).count();
                        copied_segments += segments.iter().filter(|s| s.end.is_some()).count();
                        let model = &versions[usize::try_from(last).expect("version")];
                        assert_eq!(rows(&mut world), model.rows, "seed {initial}: map at Held");
                        let extents: Vec<_> = model.extents.iter().map(|(&log, &end)| (log, end)).collect();
                        world.copy_held(snapshot, &segments, &extents);
                        held = Some(last);
                    }
                    other => panic!("seed {initial}: {other:?}"),
                }
            }
            if held.is_some() {
                break;
            }
            assert!(world.step(true), "seed {initial}: hold makes progress");
        }
        let last = held.expect("hold answered");
        world.drive(true);
        commits(&mut world);
        // The source continues changing well past the map captured above.
        for _ in 0..12 {
            admit(&mut world, &mut versions, &mut seed);
            world.drive(draw(&mut seed).is_multiple_of(2));
            commits(&mut world);
            for _ in 0..draw(&mut seed) % 3 {
                if world.ready() {
                    world.fire_once();
                }
            }
        }
        if draw(&mut seed).is_multiple_of(2) {
            for _ in 0..3 {
                admit(&mut world, &mut versions, &mut seed);
            }
            for _ in 0..draw(&mut seed) % 60 {
                if !world.step(true) {
                    break;
                }
            }
            if !world.file_pending() {
                admit(&mut world, &mut versions, &mut seed);
            }
            assert!(world.file_pending(), "seed {initial}: crash at a pending file operation");
            let acknowledged = world.store.last();
            world.crash(draw(&mut seed));
            world.request(Request::Open { owner: OWNER, root: world.root });
            let Event::Opened { last: recovered, .. } = world.pop() else { panic!("seed {initial}: source recovery") };
            assert!(recovered >= acknowledged && recovered < u64::try_from(versions.len()).expect("versions"));
            check(
                &mut world,
                &versions[usize::try_from(recovered).expect("version")],
                &format!("seed {initial}: source after crash"),
            );
            crashes += 1;
        } else {
            world.start(Request::Release { owner: OWNER });
            assert_eq!(world.pop(), Event::Released { owner: OWNER });
            world.drive(true);
            check(&mut world, versions.last().expect("latest model"), &format!("seed {initial}: source after release"));
        }
        world.request(Request::Close { owner: OWNER });
        assert_eq!(world.pop(), Event::Closed { owner: OWNER });
        world.open_copy();
        assert_eq!(world.pop(), Event::Opened { owner: Token::new(1), last }, "seed {initial}: copy recovered to hold");
        check(&mut world, &versions[usize::try_from(last).expect("version")], &format!("seed {initial}: copy"));
        world.request(Request::Close { owner: OWNER });
        assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    }
    assert!(
        snapshots > 0 && linked_segments > 0 && copied_segments > 0 && crashes > 0,
        "draws cover both copy forms, snapshots and crashes"
    );
}
