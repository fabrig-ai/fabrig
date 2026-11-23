//! Prepared and fresh schemas over independent copies of the same store (protocol/store/layouts.md, 8).
use super::*;

type OpenedAnswer = (Token, u64, [u8; 16], bool);
fn opened(event: &Event) -> OpenedAnswer {
    let Event::Opened { owner, last, installation, pending } = event else { panic!("{event:?}") };
    (*owner, *last, *installation, *pending)
}

fn registry_rows(store: &mut Store) -> Vec<skein_kv::Row> {
    // The fixture has at most three layout rows and three text rows.
    const ROWS: u32 = 6;
    let ranges = [skein_kv::Range { start: Box::from([0, 6]), end: Some(Box::from([0, 8])) }];
    let page = crate::pages::reader::walk_stored(store, &ranges, None, ROWS).expect("fixture registry is readable");
    assert!(page.next.is_none(), "compare the whole registry");
    page.rows.into_iter().map(|item| item.row).collect()
}

struct Case<'a> {
    items: &'a [Item],
    binary: OwnerSet,
    prepared: &'static [Schema],
    pending: bool,
    reads: &'a [(&'a [u8], &'a [u8])],
}
fn assert_same_open(case: Case<'_>) {
    let mut fresh_disk = Disk::new(case.items);
    let mut prepared_disk = Disk::new(case.items);
    let mut fresh = store(&fresh_disk, case.binary);
    let mut prepared = store(&prepared_disk, case.binary);
    prepared.prepared_schemas(case.prepared);

    let fresh_events = fresh_disk.open(&mut fresh);
    let prepared_events = prepared_disk.open(&mut prepared);
    let [fresh_event, Event::Census { .. }] = fresh_events.as_ref() else { panic!("{fresh_events:?}") };
    let [prepared_event, Event::Census { .. }] = prepared_events.as_ref() else { panic!("{prepared_events:?}") };
    let fresh_answer = opened(fresh_event);
    let prepared_answer = opened(prepared_event);
    assert_eq!(prepared_answer, fresh_answer);
    assert_eq!(prepared_answer.3, case.pending);
    if case.pending {
        switch(&mut fresh_disk, &mut fresh);
        switch(&mut prepared_disk, &mut prepared);
    }
    assert_eq!(registry_rows(&mut prepared), registry_rows(&mut fresh));
    for &(key, value) in case.reads {
        let read = |store: &mut Store| {
            let key = crate::keys::application(key, store.limits.store.key).expect("fixture key fits");
            crate::open::get(store, Token::new(1), key).expect("fixture value reads")
        };
        assert_eq!(read(&mut fresh).as_deref(), Some(value));
        assert_eq!(read(&mut prepared).as_deref(), Some(value));
    }
}

#[test]
fn prepared_and_fresh_opens_write_and_read_the_same_rows() {
    let historical = base(true);
    let current = base_at(CURRENT, Some(&[0, 2, 0, 42, 7]));
    let altered = Box::leak(CURRENT.replacen("= 7", "= 8", 1).into_boxed_str());
    assert_eq!(CURRENT.bytes().zip(altered.bytes()).filter(|(old, new)| old != new).count(), 1);
    let altered = OwnerSet { families: Box::leak(Box::new([KeptFamily { current: altered, ..FAMILY }])), ..SET };
    let held = held_source();
    let with_history = fixtures::prepared(sets(SET), &[KEPT]);
    for case in [
        Case {
            items: &current,
            binary: SET,
            prepared: fixtures::prepared(sets(SET), &[]),
            pending: false,
            reads: &[(b"a/0", &[0, 2, 0, 42, 7]), (b"c/0", &[0, 2, 1])],
        },
        Case {
            items: &historical,
            binary: SET,
            prepared: with_history,
            pending: true,
            reads: &[(b"a/0", &[0, 2, 0, 42, 7]), (b"c/0", &[0, 2, 1])],
        },
        Case {
            items: &historical,
            binary: altered,
            prepared: with_history,
            pending: true,
            reads: &[(b"a/0", &[0, 2, 0, 42, 8]), (b"c/0", &[0, 2, 1])],
        },
        Case {
            items: &held,
            binary: HELD_CURRENT_SET,
            prepared: fixtures::prepared(sets(HELD_CURRENT_SET), &[CHILD]),
            pending: true,
            reads: &[(b"h/0", &[0, 1, 0, 0, 0, 0, 4, 0, 1, 42, 7]), (b"j/0", &[0, 1, 1])],
        },
    ] {
        assert_same_open(case);
    }
}

fn rehearse(items: &[Item], set: OwnerSet, prepared: &'static [Schema]) -> ([u64; 2], OpenedAnswer) {
    let mut disk = Disk::new(items);
    let before = disk.contents();
    let mut store = store(&disk, set);
    store.prepared_schemas(prepared);
    let directory = disk.root();
    let scratch = disk.scratch_root();
    let events = disk.request(
        &mut store,
        Request::Rehearse {
            owner: Token::new(1),
            directory,
            scratch,
            installation: fixtures::INSTALLATION,
            reach: crate::Reach::Whole,
        },
    );
    // A writing rehearsal decodes both the source generation and its upgrade copy.
    let [
        Event::Rehearsing { .. },
        Event::Decoded { records: original, .. },
        Event::Decoded { records: copy, .. },
        opened_event,
        Event::Census { .. },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    let closed = disk.request(&mut store, Request::Close { owner: Token::new(1) });
    let [Event::Closed { removed: true, .. }] = closed.as_ref() else { panic!("{closed:?}") };
    assert_eq!(disk.contents(), before, "rehearsal leaves the source untouched");
    ([*original, *copy], opened(opened_event))
}

#[test]
fn prepared_and_fresh_rehearsals_decode_both_kinds_of_family() {
    const CHILD: KeptFamily = KeptFamily { keys: &[Keys { from: b"m", to: b"n", root: "Child" }], ..CHILD_FAMILY };
    const OLD: OwnerSet = OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }, CHILD], ..SET };
    const BINARY: OwnerSet = OwnerSet { families: &[FAMILY, CHILD], ..SET };
    let mut disk = Disk::new(&[]);
    let mut source = fresh(&mut disk, OLD);
    raw(
        &mut disk,
        &mut source,
        vec![
            skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 2, 0, 42]) },
            skein_kv::Op::Put { key: b"\x02m/0".as_slice().into(), value: Box::from([0, 1, 42]) },
        ],
    );
    let items = disk.items();
    let fresh = rehearse(&items, BINARY, &[]);
    let prepared = rehearse(&items, BINARY, fixtures::prepared(sets(BINARY), &[KEPT]));
    assert_eq!(prepared, fresh);
    assert_eq!(prepared.0, [2, 2], "decode the registered Growing and unregistered Child records");
}
