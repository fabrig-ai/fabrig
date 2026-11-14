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

fn heads(census: &Census, index: usize) -> Vec<(Option<u16>, u64)> {
    census.keyspaces()[index].heads().iter().map(|entry| (entry.head, entry.rows)).collect()
}

// Owned observations survive resetting the store for recovery.
#[derive(Debug, PartialEq, Eq)]
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
    let key = [1, 1, 10, 0, 7];
    commit(&mut store, vec![put(&key, &[0, 12, 65])]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: vec![1, 1, 10], counts: [1, 5, 3, 1, 0, 8], heads: vec![(Some(12), 1)] }])
    );
    commit(&mut store, vec![erase(&key)]);
    assert_eq!(
        view(store.census()),
        (1, vec![View { prefix: vec![1, 1, 10], counts: [0, 0, 0, 1, 1, 13], heads: vec![] }])
    );
}

#[test]
fn overwrites_short_values_missing_erases_and_last_ops_are_counted() {
    let mut store = Store::memory(&limits());
    store.count(rule());
    commit(&mut store, vec![put(b"aa", &[0, 11, 7])]);
    commit(&mut store, vec![put(b"aa", &[0, 12, 8, 9])]);
    let space = &store.census().keyspaces()[0];
    assert_eq!([space.rows, space.keys, space.values, space.puts, space.written], [1, 2, 4, 2, 11]);
    assert_eq!(heads(store.census(), 0), vec![(Some(12), 1)]);
    commit(&mut store, vec![erase(b"absent"), put(b"ab", b"x"), put(b"ac", b"")]);
    assert_eq!(heads(store.census(), 0), vec![(None, 2), (Some(12), 1)]);
    let space = &store.census().keyspaces()[0];
    assert_eq!([space.rows, space.keys, space.values, space.puts, space.erases, space.written], [3, 6, 5, 4, 1, 22]);
    commit(
        &mut store,
        vec![put(b"aa", b"ignored"), erase(b"aa"), put(b"aa", &[0, 13]), put(b"ad", b"ignored"), erase(b"ad")],
    );
    let space = &store.census().keyspaces()[0];
    assert_eq!([space.rows, space.keys, space.values, space.puts, space.erases, space.written], [3, 6, 3, 5, 2, 28]);
    assert_eq!(heads(store.census(), 0), vec![(None, 2), (Some(13), 1)]);
}

#[test]
fn no_rule_leaves_the_census_empty() {
    let mut store = Store::memory(&limits());
    commit(&mut store, vec![put(b"a", b"value"), erase(b"absent")]);
    assert_eq!(store.len(), 1);
    assert_eq!(store.census().keyspaces(), []);
}

#[test]
fn prefixes_fit_the_rule_the_key_and_the_constant() {
    let mut store = Store::memory(&limits());
    let mut prefix = [0; 256];
    prefix[1] = u8::MAX;
    store.count(CensusRule { prefix });
    commit(&mut store, vec![put(&[], &[]), put(&[0, 2], &[]), put(&[1, 2], &[]), put(&[1; CENSUS_PREFIX + 1], &[])]);
    let prefixes: Vec<_> = store.census().keyspaces().iter().map(skein_kv::Keyspace::prefix).collect();
    assert_eq!(prefixes, vec![&[][..], &[0][..], &[1; CENSUS_PREFIX][..], &[1, 2][..]]);
}

#[test]
fn keyspaces_overflow_after_the_named_bound() {
    let mut store = Store::memory(&limits());
    store.count(CensusRule { prefix: [1; 256] });
    for tag in (0..CENSUS_KEYSPACES).rev() {
        commit(&mut store, vec![put(&[u8::try_from(tag).expect("tag")], b"v")]);
    }
    assert_eq!(store.census().keyspaces().len(), CENSUS_KEYSPACES);
    assert!(store.census().keyspaces().iter().all(|space| space.prefix().len() == 1));
    let overflow = u8::try_from(CENSUS_KEYSPACES).expect("overflow tag");
    commit(&mut store, vec![put(&[overflow], &[0, 12])]);
    assert_eq!(store.census().keyspaces().len(), CENSUS_KEYSPACES + 1);
    let first = &store.census().keyspaces()[0];
    assert_eq!(first.prefix(), &[]);
    assert_eq!([first.rows, first.keys, first.values, first.puts, first.written], [1, 1, 2, 1, 3]);
    // Updating and removing folded rows must never decrement a named keyspace.
    commit(&mut store, vec![put(&[overflow], &[0, 13, 4]), erase(&[0])]);
    assert_eq!(heads(store.census(), 0), vec![(Some(13), 1)]);
    assert_eq!(store.census().keyspaces()[1].rows, 0);
    commit(&mut store, vec![erase(&[overflow])]);
    assert_eq!(store.census().keyspaces()[0].rows, 0);
    assert_eq!(store.census().keyspaces()[0].heads(), []);
    assert_eq!(store.census().keyspaces().len(), CENSUS_KEYSPACES + 1, "empty spaces stay until a snapshot starts");
}

#[test]
fn heads_include_no_head_and_do_not_readmit_while_folded_rows_remain() {
    let mut store = Store::memory(&limits());
    store.count(CensusRule { prefix: [1; 256] });
    for head in (1..CENSUS_HEADS).rev() {
        let head = u8::try_from(head).expect("head byte");
        commit(&mut store, vec![put(&[b'a', head], &[0, head])]);
    }
    assert_eq!(heads(store.census(), 0), (1..8).map(|head| (Some(head), 1)).collect::<Vec<_>>());
    commit(&mut store, vec![put(b"a8", &[0, 8]), put(b"a9", &[0, 9])]);
    assert_eq!(heads(store.census(), 0).len(), CENSUS_HEADS);
    assert_eq!(heads(store.census(), 0)[0], (None, 2));
    commit(&mut store, vec![erase(&[b'a', 1]), put(b"a10", &[0, 10])]);
    assert!(!heads(store.census(), 0).iter().any(|(head, _)| *head == Some(1)), "zero-row head leaves immediately");
    assert_eq!(heads(store.census(), 0)[0], (None, 3), "a free slot cannot split a folded head");
    // Keep the short value in this keyspace; it shares no-head but is not folded by the bound.
    commit(&mut store, vec![put(b"as", b"x"), erase(b"a8"), erase(b"a9"), erase(b"a10")]);
    assert_eq!(heads(store.census(), 0)[0], (None, 1));
    commit(&mut store, vec![put(b"a11", &[0, 11])]);
    assert!(
        heads(store.census(), 0).contains(&(Some(11), 1)),
        "admission resumes when folded rows are gone, even with a short value"
    );
    commit(&mut store, vec![erase(b"as")]);
    assert!(heads(store.census(), 0).iter().all(|(head, _)| head.is_some()));
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
    assert_eq!(world.store.census().since(), last + 1);
    assert_eq!(world.store.census().keyspaces().len(), CENSUS_KEYSPACES, "pruning frees a named slot");
    let next = overflow + 1;
    disk_commit(&mut world, vec![put(&[next], &[0, 13])], last + 1);
    assert_eq!(world.store.census().keyspaces()[0].rows, 2, "a free slot cannot split folded rows");
    assert!(!world.store.census().keyspaces().iter().any(|space| space.prefix() == [next]));
    disk_commit(&mut world, vec![erase(&[overflow]), put(&[next], &[0, 14, 65])], last + 2);
    let folded = &world.store.census().keyspaces()[0];
    assert_eq!([folded.rows, folded.keys, folded.values], [1, 1, 3]);
    assert_eq!(heads(world.store.census(), 0), vec![(Some(14), 1)]);
    disk_commit(&mut world, vec![erase(&[next]), put(&[next + 1], &[0, 15])], last + 3);
    assert_eq!(world.store.census().keyspaces()[0].rows, 0);
    assert!(
        world.store.census().keyspaces().iter().any(|space| space.prefix() == [next + 1] && space.rows == 1),
        "admission resumes once folded rows are gone"
    );
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
    assert_eq!(world.store.census().since(), 3);
    assert!(
        world.store.census().keyspaces().iter().all(|space| space.puts == 0 && space.erases == 0 && space.written == 0)
    );
    assert!(!world.store.census().keyspaces().iter().any(|space| space.prefix() == b"z"));
    assert_eq!(world.store.len(), 3, "snapshot start preserves rows");
    // This commit is logged from S, while the snapshot is still running.
    disk_commit(&mut world, vec![put(b"b", &[0, 14, 65]), erase(b"a")], 3);
    let b = world.store.census().keyspaces().iter().find(|space| space.prefix() == b"b").expect("b space");
    assert_eq!([b.rows, b.keys, b.values, b.puts, b.erases, b.written], [1, 1, 3, 1, 0, 4]);
    // A keyspace erased at S remains reconstructible from its erase frame.
    let before = view(world.store.census());
    close(&mut world);
    world.store = Store::new(&world.env.limits);
    world.store.count(rule());
    open(&mut world);
    assert_eq!(view(world.store.census()), before, "snapshot rows add no writes; replay starts at S");
    close(&mut world);
}
