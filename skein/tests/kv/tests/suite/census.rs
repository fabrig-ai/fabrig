//! The map's census, including its log period on the simulated disk.

use skein_kv::{
    CENSUS_HEADS, CENSUS_KEYSPACES, CENSUS_PREFIX, Census, CensusRule, Event, Limits, Op, Page, Request, Store,
};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const OWNER: Token = Token::new(1);

fn limits() -> Limits {
    Limits {
        key: 64,
        value: 64,
        ops: 8,
        commit: 1024,
        queued: 8,
        queued_bytes: 8192,
        budget: 1_048_576,
        segment: 4096,
        snapshot_after: u64::MAX,
        chunk: 256,
        page: Page { rows: 16, bytes: 1024 },
        deadline: Duration::from_secs(1),
    }
}

fn rule() -> CensusRule {
    let mut prefix = [1; 256];
    prefix[1] = 3;
    CensusRule { prefix }
}

fn put(key: &[u8], value: &[u8]) -> Op {
    Op::Put { key: Box::from(key), value: Box::from(value) }
}

fn erase(key: &[u8]) -> Op {
    Op::Erase { key: Box::from(key) }
}

fn commit(store: &mut Store, ops: Vec<Op>) {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let number = store.last() + 1;
    skein_kv::down(store, &env, Request::Commit { owner: OWNER, ops: ops.into_boxed_slice() }, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Committed { owner: OWNER, number }));
    assert!(above.is_empty() && below.is_empty());
}

// Owned observations survive resetting the store for recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
struct View {
    prefix: Vec<u8>,
    counts: [u64; 6],
    heads: Vec<(Option<u16>, u64)>,
}

fn view(census: &Census) -> (u64, Vec<View>) {
    (
        census.since(),
        census
            .keyspaces()
            .iter()
            .map(|space| View {
                prefix: space.prefix().to_vec(),
                counts: [space.rows, space.keys, space.values, space.puts, space.erases, space.written],
                heads: space.heads().iter().map(|entry| (entry.head, entry.rows)).collect(),
            })
            .collect(),
    )
}

#[test]
fn rows_bytes_heads_and_writes_follow_put_and_erase() {
    let mut store = Store::memory(&limits());
    store.count(rule());
    let key = [1, 1, 10, 7];
    commit(&mut store, vec![put(&key, &[0, 3, 255])]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: vec![1, 1, 10], counts: [1, 4, 3, 1, 0, 7], heads: vec![(Some(3), 1)] }])
    );
    commit(&mut store, vec![erase(&key)]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: vec![1, 1, 10], counts: [0, 0, 0, 1, 1, 11], heads: vec![] }])
    );
}

#[test]
fn overwrites_short_values_missing_erases_and_last_ops_are_counted() {
    let mut store = Store::memory(&limits());
    store.count(rule());
    commit(&mut store, vec![put(b"aa", &[0, 11, 7])]);
    commit(&mut store, vec![put(b"aa", &[0, 12, 8, 9])]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: b"a".to_vec(), counts: [1, 2, 4, 2, 0, 11], heads: vec![(Some(12), 1)] }])
    );
    commit(&mut store, vec![erase(b"absent"), put(b"ab", b"x"), put(b"ac", b"")]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: b"a".to_vec(), counts: [3, 6, 5, 4, 1, 22], heads: vec![(None, 2), (Some(12), 1)] }])
    );
    commit(
        &mut store,
        vec![put(b"aa", b"ignored"), erase(b"aa"), put(b"aa", &[0, 13]), put(b"ad", b"ignored"), erase(b"ad")],
    );
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: b"a".to_vec(), counts: [3, 6, 3, 5, 2, 28], heads: vec![(None, 2), (Some(13), 1)] }])
    );
}

#[test]
fn no_rule_leaves_the_census_empty() {
    let mut store = Store::memory(&limits());
    commit(&mut store, vec![put(b"a", b"value"), erase(b"absent")]);
    assert_eq!(store.len(), 1);
    assert_eq!(view(store.census()), (1, vec![]));
    commit(&mut store, vec![put(b"a", b"new"), erase(b"a")]);
    assert_eq!(view(store.census()), (1, vec![]));
}

#[test]
fn prefixes_fit_the_rule_the_key_and_the_constant() {
    let mut store = Store::memory(&limits());
    let mut prefix = [0; 256];
    prefix[1] = u8::MAX;
    store.count(CensusRule { prefix });
    commit(&mut store, vec![put(&[], &[]), put(&[0, 2], &[]), put(&[1, 2], &[]), put(&[1; CENSUS_PREFIX + 1], &[])]);
    assert_eq!(
        view(store.census()),
        (
            1,
            vec![
                View { prefix: vec![], counts: [1, 0, 0, 1, 0, 0], heads: vec![(None, 1)] },
                View { prefix: vec![0], counts: [1, 2, 0, 1, 0, 2], heads: vec![(None, 1)] },
                View { prefix: vec![1; CENSUS_PREFIX], counts: [1, 9, 0, 1, 0, 9], heads: vec![(None, 1)] },
                View { prefix: vec![1, 2], counts: [1, 2, 0, 1, 0, 2], heads: vec![(None, 1)] },
            ]
        )
    );
}

#[test]
fn keyspaces_overflow_after_the_named_bound() {
    let mut store = Store::memory(&limits());
    store.count(CensusRule { prefix: [1; 256] });
    for tag in (0..CENSUS_KEYSPACES).rev() {
        commit(&mut store, vec![put(&[u8::try_from(tag).expect("tag")], b"v")]);
    }
    assert_eq!(store.census().keyspaces().len(), CENSUS_KEYSPACES);
    let mut expected: Vec<_> = (0..CENSUS_KEYSPACES)
        .map(|tag| View {
            prefix: vec![u8::try_from(tag).expect("tag")],
            counts: [1, 1, 1, 1, 0, 2],
            heads: vec![(None, 1)],
        })
        .collect();
    assert_eq!(view(store.census()), (1, expected.clone()));
    let overflow = u8::try_from(CENSUS_KEYSPACES).expect("overflow tag");
    commit(&mut store, vec![put(&[overflow], &[0, 12])]);
    expected.insert(0, View { prefix: vec![], counts: [1, 1, 2, 1, 0, 3], heads: vec![(Some(12), 1)] });
    assert_eq!(view(store.census()), (1, expected.clone()));
    // Updating and removing folded rows must never decrement a named keyspace.
    commit(&mut store, vec![put(&[overflow], &[0, 13, 4]), erase(&[0])]);
    expected[0] = View { prefix: vec![], counts: [1, 1, 3, 2, 0, 7], heads: vec![(Some(13), 1)] };
    expected[1] = View { prefix: vec![0], counts: [0, 0, 0, 1, 1, 3], heads: vec![] };
    assert_eq!(view(store.census()), (1, expected.clone()));
    commit(&mut store, vec![erase(&[overflow])]);
    expected[0] = View { prefix: vec![], counts: [0, 0, 0, 2, 1, 8], heads: vec![] };
    assert_eq!(view(store.census()), (1, expected), "empty spaces stay until a snapshot starts");
}

#[test]
fn heads_include_no_head_and_do_not_readmit_while_folded_rows_remain() {
    let mut store = Store::memory(&limits());
    store.count(CensusRule { prefix: [1; 256] });
    for head in (1..CENSUS_HEADS).rev() {
        let head = u8::try_from(head).expect("head byte");
        commit(&mut store, vec![put(&[b'a', head], &[0, head])]);
    }
    let named = |first, last| (first..=last).map(|head| (Some(head), 1)).collect::<Vec<_>>();
    let check = |store: &Store, counts, heads| {
        assert_eq!(view(store.census()), (1, vec![View { prefix: b"a".to_vec(), counts, heads }]));
    };
    check(&store, [7, 14, 14, 7, 0, 28], named(1, 7));
    commit(&mut store, vec![put(b"a8", &[0, 8]), put(b"a9", &[0, 9])]);
    let mut expected = vec![(None, 2)];
    expected.extend(named(1, 7));
    check(&store, [9, 18, 18, 9, 0, 36], expected);
    commit(&mut store, vec![erase(&[b'a', 1]), put(b"a10", &[0, 10])]);
    let mut expected = vec![(None, 3)];
    expected.extend(named(2, 7));
    check(&store, [9, 19, 18, 10, 1, 43], expected);
    // Keep the short value in this keyspace; it shares no-head but is not folded by the bound.
    commit(&mut store, vec![put(b"as", b"x"), erase(b"a8"), erase(b"a9"), erase(b"a10")]);
    let mut expected = vec![(None, 1)];
    expected.extend(named(2, 7));
    check(&store, [7, 14, 13, 11, 4, 53], expected.clone());
    commit(&mut store, vec![put(b"a11", &[0, 11])]);
    expected.push((Some(11), 1));
    check(&store, [8, 17, 15, 12, 4, 58], expected.clone());
    commit(&mut store, vec![erase(b"as")]);
    expected.remove(0);
    check(&store, [7, 15, 14, 12, 5, 60], expected);
}

#[test]
fn snapshot_pruning_does_not_split_folded_keyspaces() {
    let mut world = World::new(Limits { snapshot_after: 1, ..limits() });
    world.store.count(CensusRule { prefix: [1; 256] });
    open(&mut world);
    for tag in 0..=CENSUS_KEYSPACES {
        disk_commit(
            &mut world,
            vec![put(&[u8::try_from(tag).expect("tag")], &[0, 12])],
            u64::try_from(tag + 1).expect("number"),
        );
    }
    let overflow = u8::try_from(CENSUS_KEYSPACES).expect("overflow tag");
    let last = u64::try_from(CENSUS_KEYSPACES + 2).expect("number");
    disk_commit(&mut world, vec![erase(&[0])], last);
    world.fire_once();
    let mut expected = vec![View { prefix: vec![], counts: [1, 1, 2, 0, 0, 0], heads: vec![(Some(12), 1)] }];
    expected.extend((1..CENSUS_KEYSPACES).map(|tag| View {
        prefix: vec![u8::try_from(tag).expect("tag")],
        counts: [1, 1, 2, 0, 0, 0],
        heads: vec![(Some(12), 1)],
    }));
    assert_eq!(view(world.store.census()), (last + 1, expected.clone()), "pruning frees a named slot");
    let next = overflow + 1;
    disk_commit(&mut world, vec![put(&[next], &[0, 13])], last + 1);
    assert_eq!(world.store.census().keyspaces()[0].rows, 2, "a free slot cannot split folded rows");
    expected[0] = View { prefix: vec![], counts: [2, 2, 4, 1, 0, 3], heads: vec![(Some(12), 1), (Some(13), 1)] };
    assert_eq!(view(world.store.census()), (last + 1, expected.clone()));
    disk_commit(&mut world, vec![erase(&[overflow]), put(&[next], &[0, 14, 65])], last + 2);
    expected[0] = View { prefix: vec![], counts: [1, 1, 3, 2, 1, 8], heads: vec![(Some(14), 1)] };
    assert_eq!(view(world.store.census()), (last + 1, expected.clone()));
    disk_commit(&mut world, vec![erase(&[next]), put(&[next + 1], &[0, 15])], last + 3);
    expected[0] = View { prefix: vec![], counts: [0, 0, 0, 2, 2, 9], heads: vec![] };
    expected.push(View { prefix: vec![next + 1], counts: [1, 1, 2, 1, 0, 3], heads: vec![(Some(15), 1)] });
    assert_eq!(view(world.store.census()), (last + 1, expected), "admission resumes once folded rows are gone");
    close(&mut world);
}

fn open(world: &mut World) {
    world.request_no_fire(Request::Open { owner: OWNER, root: world.root });
    assert!(matches!(world.pop(), Event::Opened { .. }));
}

fn disk_commit(world: &mut World, ops: Vec<Op>, number: u64) {
    world.request_no_fire(Request::Commit { owner: OWNER, ops: ops.into_boxed_slice() });
    assert_eq!(world.pop(), Event::Committed { owner: OWNER, number });
}

fn close(world: &mut World) {
    world.request(Request::Close { owner: OWNER });
    assert_eq!(world.pop(), Event::Closed { owner: OWNER });
    assert_eq!(world.io.open_files(), 0);
}

#[test]
fn recovery_recounts_commits_and_keeps_the_rule_when_the_map_restarts() {
    let mut world = World::new(limits());
    world.store.count(rule());
    open(&mut world);
    disk_commit(&mut world, vec![put(&[1, 1, 10, 0, 7], &[0, 11]), put(b"a", b"x")], 1);
    disk_commit(&mut world, vec![put(&[1, 1, 10, 0, 7], &[0, 12, 65]), erase(b"absent")], 2);
    let before = view(world.store.census());
    assert_eq!(
        before,
        (
            1,
            vec![
                View { prefix: vec![1, 1, 10], counts: [1, 5, 3, 2, 0, 15], heads: vec![(Some(12), 1)] },
                View { prefix: b"a".to_vec(), counts: [1, 1, 1, 1, 1, 8], heads: vec![(None, 1)] },
            ]
        )
    );
    close(&mut world);
    // Open recovers into a new map but must keep this Store's rule.
    open(&mut world);
    assert_eq!(view(world.store.census()), before);
    close(&mut world);
}

#[test]
fn snapshot_start_resets_writes_prunes_empty_spaces_and_recovery_keeps_its_period() {
    let mut world = World::new(Limits { snapshot_after: 90, ..limits() });
    world.store.count(rule());
    open(&mut world);
    disk_commit(&mut world, vec![put(b"a", &[0, 11]), erase(b"z")], 1);
    disk_commit(&mut world, vec![put(b"b", &[0, 12]), put(&[1, 1, 10, 0, 7], &[0, 13])], 2);
    assert_eq!(world.store.census().since(), 1);
    assert!(world.store.census().keyspaces().iter().any(|space| space.prefix() == b"z"));
    // Fire until the starting segment's directory sync announces S, before any snapshot rows are written.
    for _ in 0..8 {
        if world.store.census().since() != 1 {
            break;
        }
        world.fire_once();
    }
    assert_eq!(
        view(world.store.census()),
        (
            3,
            vec![
                View { prefix: vec![1, 1, 10], counts: [1, 5, 2, 0, 0, 0], heads: vec![(Some(13), 1)] },
                View { prefix: b"a".to_vec(), counts: [1, 1, 2, 0, 0, 0], heads: vec![(Some(11), 1)] },
                View { prefix: b"b".to_vec(), counts: [1, 1, 2, 0, 0, 0], heads: vec![(Some(12), 1)] },
            ]
        )
    );
    // This commit is logged from S, while the snapshot is still running.
    disk_commit(&mut world, vec![put(b"b", &[0, 14, 65]), erase(b"a")], 3);
    // A keyspace erased at S remains reconstructible from its erase frame.
    let before = view(world.store.census());
    assert_eq!(
        before,
        (
            3,
            vec![
                View { prefix: vec![1, 1, 10], counts: [1, 5, 2, 0, 0, 0], heads: vec![(Some(13), 1)] },
                View { prefix: b"a".to_vec(), counts: [0, 0, 0, 0, 1, 1], heads: vec![] },
                View { prefix: b"b".to_vec(), counts: [1, 1, 3, 1, 0, 4], heads: vec![(Some(14), 1)] },
            ]
        )
    );
    close(&mut world);
    world.store = Store::new(&world.env.limits);
    world.store.count(rule());
    open(&mut world);
    assert_eq!(view(world.store.census()), before, "snapshot rows add no writes; replay starts at S");
    close(&mut world);
}
