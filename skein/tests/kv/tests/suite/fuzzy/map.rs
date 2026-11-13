//! Seeded model checks for commit, get and ordered pages.

use std::collections::BTreeMap;

use skein_kv::{CensusRule, Event, Limits, Op, Page, Range, Request, Store};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

fn limits() -> Limits {
    Limits {
        key: 8,
        value: 8,
        ops: 4,
        commit: 256,
        queued: 4,
        queued_bytes: 1024,
        budget: 4096,
        segment: 1024,
        snapshot_after: 2048,
        chunk: 128,
        page: Page { rows: 4, bytes: 64 },
        deadline: Duration::from_secs(1),
    }
}

fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[test]
fn generated_commits_and_pages_match_the_model() {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    for initial in 1..=64_u64 {
        let mut seed = initial;
        let mut store = Store::memory(&env.limits);
        let mut model = BTreeMap::new();
        let mut above = Queue::with_capacity(4);
        let mut below = Queue::with_capacity(1);
        for step in 0..100_u64 {
            let key = [u8::try_from(draw(&mut seed) % 24).expect("key byte")];
            let value = [u8::try_from(draw(&mut seed) % 256).expect("value byte")];
            let erase = draw(&mut seed).is_multiple_of(4);
            let op = if erase {
                Op::Erase { key: Box::from(key) }
            } else {
                Op::Put { key: Box::from(key), value: Box::from(value) }
            };
            skein_kv::down(
                &mut store,
                &env,
                Request::Commit { owner: Token::new(step + 1), ops: Box::new([op]) },
                &mut above,
                &mut below,
            );
            match above.pop().expect("one commit outcome") {
                Event::Committed { .. } => {
                    if erase {
                        model.remove(key.as_slice());
                    } else {
                        model.insert(key.to_vec(), value.to_vec());
                    }
                }
                Event::Refused { .. } => {}
                event => panic!("unexpected commit outcome: {event:?}"),
            }
            skein_kv::down(
                &mut store,
                &env,
                Request::Get { owner: Token::new(101), key: Box::from(key) },
                &mut above,
                &mut below,
            );
            let Event::Got { value: found, .. } = above.pop().expect("get answers") else {
                panic!("get terminal");
            };
            assert_eq!(found.as_deref(), model.get(key.as_slice()).map(Vec::as_slice), "seed {initial}, step {step}");
        }
        let mut start = Box::from(&b""[..]);
        let mut found = Vec::new();
        loop {
            skein_kv::down(
                &mut store,
                &env,
                Request::Load {
                    owner: Token::new(102),
                    range: Range { start, end: None },
                    max: Page { rows: 4, bytes: 64 },
                },
                &mut above,
                &mut below,
            );
            let Event::Loaded { rows, next, .. } = above.pop().expect("load answers") else {
                panic!("load terminal");
            };
            found.extend(rows.iter().map(|row| (row.key.to_vec(), row.value.to_vec())));
            let Some(next) = next else { break };
            start = next;
        }
        assert_eq!(found, model.into_iter().collect::<Vec<_>>(), "seed {initial}");
        assert!(below.is_empty());
    }
}

fn keyspace(key: &[u8]) -> Vec<u8> {
    let length = if key.first() == Some(&1) { 3 } else { 1 };
    key[..key.len().min(length)].to_vec()
}

type ModelCensus = BTreeMap<Vec<u8>, ([u64; 6], BTreeMap<Option<u16>, u64>)>;

#[test]
fn generated_commits_keep_the_census_equal_to_a_recount() {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    for initial in 1..=64_u64 {
        let mut seed = initial;
        let mut store = Store::memory(&env.limits);
        let mut prefix = [1; 256];
        prefix[1] = 3;
        store.count(CensusRule { prefix });
        let mut model = BTreeMap::<Vec<u8>, Vec<u8>>::new();
        let mut writes = BTreeMap::<Vec<u8>, [u64; 3]>::new();
        let mut above = Queue::with_capacity(1);
        let mut below = Queue::with_capacity(1);
        for step in 1..=100_u64 {
            let mut ops = Vec::new();
            for _ in 0..=draw(&mut seed) % 4 {
                let key = if draw(&mut seed).is_multiple_of(24) {
                    vec![]
                } else {
                    vec![
                        u8::try_from(1 + draw(&mut seed) % 2).expect("tag"),
                        u8::try_from(draw(&mut seed) % 3).expect("keyspace"),
                        u8::try_from(draw(&mut seed) % 4).expect("row"),
                    ]
                };
                let op = if draw(&mut seed).is_multiple_of(4) {
                    Op::Erase { key: key.into_boxed_slice() }
                } else {
                    let mut value = vec![
                        u8::try_from(draw(&mut seed) % 256).expect("value");
                        usize::try_from(draw(&mut seed) % 7).expect("length")
                    ];
                    if value.len() >= 2 {
                        value[0] = 0;
                        value[1] = u8::try_from(11 + draw(&mut seed) % 4).expect("head");
                    }
                    Op::Put { key: key.into_boxed_slice(), value: value.into_boxed_slice() }
                };
                ops.push(op);
            }
            skein_kv::down(
                &mut store,
                &env,
                Request::Commit { owner: Token::new(step), ops: ops.clone().into_boxed_slice() },
                &mut above,
                &mut below,
            );
            assert_eq!(
                above.pop(),
                Some(Event::Committed { owner: Token::new(step), number: step }),
                "seed {initial}, step {step}"
            );
            for (index, op) in ops.iter().enumerate() {
                if ops[index + 1..].iter().any(|later| later.key() == op.key()) {
                    continue;
                }
                let count = writes.entry(keyspace(op.key())).or_default();
                match op {
                    Op::Put { key, value } => {
                        model.insert(key.to_vec(), value.to_vec());
                        count[0] += 1;
                        count[2] += u64::try_from(key.len() + value.len()).expect("written");
                    }
                    Op::Erase { key } => {
                        model.remove(key.as_ref());
                        count[1] += 1;
                        count[2] += u64::try_from(key.len()).expect("written");
                    }
                }
            }
            let mut expected = ModelCensus::new();
            for (prefix, count) in &writes {
                expected.insert(prefix.clone(), ([0, 0, 0, count[0], count[1], count[2]], BTreeMap::new()));
            }
            for (key, value) in &model {
                let (count, heads) = expected.get_mut(&keyspace(key)).expect("row has an applied put");
                count[0] += 1;
                count[1] += u64::try_from(key.len()).expect("key length");
                count[2] += u64::try_from(value.len()).expect("value length");
                let head = if value.len() < 2 { None } else { Some(u16::from_be_bytes([value[0], value[1]])) };
                *heads.entry(head).or_default() += 1;
            }
            let actual: ModelCensus = store
                .census()
                .keyspaces()
                .iter()
                .map(|space| {
                    (
                        space.prefix().to_vec(),
                        (
                            [space.rows, space.keys, space.values, space.puts, space.erases, space.written],
                            space.heads().iter().map(|head| (head.head, head.rows)).collect(),
                        ),
                    )
                })
                .collect();
            assert_eq!(actual, expected, "seed {initial}, step {step}");
            assert_eq!(store.census().since(), 1);
            assert!(above.is_empty() && below.is_empty());
        }
    }
}
