//! Going back and the layout order (protocol/store/layouts.md, 5.6).
use super::*;
use jig_test_layouts::layouts::{BACK_EARLIER, BACK_LATER, BACK_NEWEST, FIRST};
use skein_codec::layout::Refused;

const EARLIER: OwnerSet = OwnerSet { families: &[KeptFamily { current: BACK_EARLIER, ..FAMILY }], ..SET };
const LATER: OwnerSet = OwnerSet { families: &[KeptFamily { current: BACK_LATER, ..FAMILY }], ..SET };
const BEFORE: &[u8] = &[0, 2, 0, 42, 0];
const AT_DEFAULTS: &[u8] = &[0, 2, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn fingerprint(text: &str) -> [u8; 32] {
    let schema = skein_codec::schema::parse(text.as_bytes()).unwrap();
    schema.layout(schema.root(b"Growing", None).unwrap(), &[]).unwrap().digest()
}

fn order_key(number: u16) -> Box<[u8]> {
    crate::keys::order(FAMILY.name, number, fixtures::KEY_LIMIT).unwrap()
}

fn order(store: &mut Store, number: u16) -> Option<Box<[[u8; 32]]>> {
    let bytes = fixtures::get_kv(store, &order_key(number))?;
    let wire = crate::registry_v1::Order::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    Some(wire.layouts.into_boxed())
}

pub(super) fn write(disk: &mut Disk, store: &mut Store, rows: &[(&[u8], &[u8])]) -> u64 {
    let number = store.stores.store.last() + 1;
    let mut writes = vec![fixtures::header(number)];
    for &(key, bytes) in rows {
        writes.push(Write::Put { key: key.into(), value: bytes.into() });
    }
    let events = disk.request(store, fixtures::commit(number, writes.into_boxed_slice()));
    let [Event::Committed { number: committed, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(*committed, number);
    number
}

pub(super) fn get(disk: &mut Disk, store: &mut Store, key: &[u8]) -> Box<[u8]> {
    let events = disk.request(store, Request::Get { owner: Token::new(1), key: Get::Application(key.into()) });
    let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = events.as_ref() else { panic!("{events:?}") };
    value.clone()
}

fn later_value(extra: u32, variant: u8, hint: &[u8]) -> Box<[u8]> {
    let n = 42_u16.to_be_bytes();
    let extra = extra.to_be_bytes();
    let length = u32::try_from(hint.len()).unwrap().to_be_bytes();
    [2_u16.to_be_bytes().as_slice(), &n, &[variant], &extra, &length, hint].concat().into_boxed_slice()
}

fn pending(events: &[Event], previous: u64) -> u64 {
    let [Event::Opened { pending: true, last, .. }, Event::Census { .. }] = events else { panic!("{events:?}") };
    assert!(*last > previous);
    *last
}

fn at_later(set: OwnerSet, previous: OwnerSet, keep_earlier: bool) -> (Disk, Store) {
    let mut disk = Disk::new(&[]);
    let mut store = fresh(&mut disk, previous);
    if keep_earlier {
        write(&mut disk, &mut store, &[(b"a/0", BEFORE)]);
    }
    let last = store.stores.store.last();
    let mut disk = Disk::new(&disk.items());
    let mut later = super::store(&disk, set);
    pending(&disk.open(&mut later), last);
    switch(&mut disk, &mut later);
    (disk, later)
}

#[test]
fn a_missing_order_takes_the_stored_ids_then_appends_the_binary_layout() {
    const BINARY_B: OwnerSet = OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET };
    let mut disk = Disk::new(&base(true));
    let mut previous = store(&disk, BINARY_B);
    assert_opened(&disk.open(&mut previous), 1);
    let key = order_key(0);
    assert_eq!(key.as_ref(), [&[0, 14], FAMILY.name.as_bytes(), &[0, 0], &[0, 0]].concat());
    assert_eq!(crate::keys::decode_order(&key), Some((FAMILY.name.into(), 0)));
    raw(
        &mut disk,
        &mut previous,
        vec![
            skein_kv::Op::Erase { key },
            skein_kv::Op::Erase { key: order_key(1) },
            layout_row(FIRST, 1, FAMILY.name, 0),
            layout_text(FIRST),
        ],
    );
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, SET);
    pending(&disk.open(&mut current), 2);
    assert_eq!(
        order(&mut current, 0).unwrap().as_ref(),
        &[fingerprint(FIRST), fingerprint(KEPT), fingerprint(CURRENT)]
    );
    let stable = row(&mut current, 1, 2).unwrap().layout;
    assert_eq!(order(&mut current, 1).unwrap().as_ref(), &[stable]);
    assert_eq!(get(&mut disk, &mut current, b"a/0").as_ref(), &[0, 2, 0, 42, 7]);
    assert!(current.stores.store.census().keyspaces().iter().any(|space| space.prefix() == [0, 14] && space.rows == 2));
    assert!(crate::keys::KEYSPACES.contains(&(&[0, 14], "component.registry-orders")));
}

#[test]
fn missing_orders_alone_write_once() {
    let mut disk = Disk::new(&[]);
    let mut before = fresh(&mut disk, EARLIER);
    raw(
        &mut disk,
        &mut before,
        vec![skein_kv::Op::Erase { key: order_key(0) }, skein_kv::Op::Erase { key: order_key(1) }],
    );
    let mut disk = Disk::new(&disk.items());
    let mut first = store(&disk, EARLIER);
    let last = pending(&disk.open(&mut first), 2);
    assert_eq!(order(&mut first, 0).unwrap().as_ref(), &[fingerprint(BACK_EARLIER)]);
    switch(&mut disk, &mut first);
    let mut again = store(&disk, EARLIER);
    let events = disk.open(&mut again);
    assert_opened(&events, last);
    assert!(if let Event::Opened { pending: false, .. } = events[0] { true } else { false });
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    assert!(!disk.tree().iter().any(|(path, ..)| path.as_ref() == b"generations/3"));
}

#[test]
fn appending_a_sixty_fifth_layout_drops_only_the_oldest() {
    let mut disk = Disk::new(&[]);
    let mut before = fresh(&mut disk, EARLIER);
    let mut held = skein_lib::List::with_capacity(64);
    for older in 0_u8..63 {
        held.push([older; 32]).unwrap();
    }
    held.push(fingerprint(BACK_EARLIER)).unwrap();
    let wire = crate::registry_v1::Order { layouts: held };
    let value = wire.encode(&crate::registry_v1::CEILINGS).unwrap();
    raw(&mut disk, &mut before, vec![skein_kv::Op::Put { key: order_key(0), value }]);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, LATER);
    pending(&disk.open(&mut current), 2);
    let layouts = order(&mut current, 0).unwrap();
    let expected = [&wire.layouts.as_slice()[1..], &[fingerprint(BACK_LATER)]].concat();
    assert_eq!(layouts.as_ref(), expected);
}

#[test]
fn going_back_keeps_later_values_and_discards_only_the_droppable_hint() {
    let (mut disk, mut later) = at_later(LATER, EARLIER, true);
    let expected = [fingerprint(BACK_EARLIER), fingerprint(BACK_LATER)];
    assert_eq!(order(&mut later, 0).unwrap().as_ref(), &expected);
    let value = later_value(0, 0, b"a hint");
    let another = later_value(0, 0, b"another hint");
    let last = write(&mut disk, &mut later, &[(b"a/1", &value), (b"a/2", &another)]);
    assert_eq!(app(&mut later, b"a/1").unwrap().as_ref(), &[&[0, 3], &value[2..]].concat());

    let mut disk = Disk::new(&disk.items());
    let mut earlier = store(&disk, EARLIER);
    let events = disk.open(&mut earlier);
    assert_opened(&events, last);
    assert!(if let Event::Opened { pending: false, .. } = events[0] { true } else { false });
    assert_eq!(order(&mut earlier, 0).unwrap().as_ref(), &expected);
    let a = get(&mut disk, &mut earlier, b"a/1");
    let b = get(&mut disk, &mut earlier, b"a/2");
    assert_eq!(a.as_ref(), BEFORE);
    assert_eq!(b.as_ref(), BEFORE);
    let last = write(&mut disk, &mut earlier, &[(b"a/1", &a), (b"a/2", &b)]);
    assert_eq!(app(&mut earlier, b"a/1").unwrap().as_ref(), BEFORE);
    assert_eq!(app(&mut earlier, b"a/2").unwrap().as_ref(), BEFORE);

    let mut disk = Disk::new(&disk.items());
    let mut later = store(&disk, LATER);
    assert_opened(&disk.open(&mut later), last);
    assert_eq!(get(&mut disk, &mut later, b"a/1").as_ref(), AT_DEFAULTS);
    assert_eq!(get(&mut disk, &mut later, b"a/2").as_ref(), AT_DEFAULTS);
}

fn assert_refusal(events: &[Event], refused: u64, why: Refused, declaration: &str, part: &str, values: u64) {
    let Some(Event::OpenRefused { why: found, .. }) = events.last() else { panic!("{events:?}") };
    assert_eq!(
        found,
        &OpenRefusal::Layout {
            family: FAMILY.name.into(),
            value_type: 0,
            id: 3,
            values: Some(values),
            why: Unplanned::Unkept(crate::Refusals {
                refused,
                why,
                declaration: declaration.into(),
                part: part.into()
            }),
        }
    );
}

#[test]
fn going_back_counts_every_refusal_across_pages_and_reports_the_first_in_key_order() {
    const NEWEST: OwnerSet = OwnerSet { families: &[KeptFamily { current: BACK_NEWEST, ..FAMILY }], ..SET };
    let (mut disk, mut later) = at_later(LATER, EARLIER, true);
    let field = later_value(5, 0, b"");
    let variant = later_value(0, 1, &[b'x'; 64]);
    let good = later_value(0, 0, &[b'x'; 64]);
    write(&mut disk, &mut later, &[(b"a/2", &field), (b"a/3", &good), (b"a/4", &good)]);
    let last = write(&mut disk, &mut later, &[(b"a/5", &good), (b"a/6", &good), (b"a/7", &variant)]);
    let mut disk = Disk::new(&disk.items());
    let mut newest = store(&disk, NEWEST);
    pending(&disk.open(&mut newest), last);
    switch(&mut disk, &mut newest);
    // The lowest rejecting map key is at a later registry id: report id 3, not 4.
    let newest_value = [AT_DEFAULTS, &[1]].concat();
    write(&mut disk, &mut newest, &[(b"a/1", &newest_value)]);
    let items = disk.items();
    for rehearsing in [false, true] {
        let mut disk = Disk::new(&items);
        let before = disk.contents();
        let mut earlier = store(&disk, EARLIER);
        // Room for any registry row, but not the whole type's values.
        earlier.limits.store.page.bytes = 320;
        let events = if rehearsing {
            let scratch = disk.scratch_root();
            let directory = disk.root();
            disk.request(
                &mut earlier,
                Request::Rehearse {
                    owner: Token::new(1),
                    directory,
                    scratch,
                    installation: fixtures::INSTALLATION,
                    reach: crate::Reach::Whole,
                },
            )
        } else {
            disk.open(&mut earlier)
        };
        assert_refusal(&events, 2, Refused::Field, "Growing", "extra", 6);
        assert_unchanged_except_recovery(&before, &mut disk);
    }
}

#[test]
fn going_back_refuses_the_added_variant_on_its_own() {
    let (mut disk, mut later) = at_later(LATER, EARLIER, true);
    let variant = later_value(0, 1, b"");
    write(&mut disk, &mut later, &[(b"a/1", &variant)]);
    let mut disk = Disk::new(&disk.items());
    let before = disk.contents();
    let mut earlier = store(&disk, EARLIER);
    assert_refusal(&disk.open(&mut earlier), 1, Refused::Variant, "Choice", "added", 1);
    assert_unchanged_except_recovery(&before, &mut disk);
}

#[test]
fn going_back_after_retirement_still_considers_the_newer_layout_later() {
    let (mut disk, mut later) = at_later(LATER, EARLIER, false);
    let expected = [fingerprint(BACK_EARLIER), fingerprint(BACK_LATER)];
    assert!(row(&mut later, 0, 2).is_none(), "the newer open retired the earlier row");
    assert_eq!(order(&mut later, 0).unwrap().as_ref(), &expected);
    let refused = later_value(5, 0, b"");
    write(&mut disk, &mut later, &[(b"a/1", &refused)]);
    let mut back = Disk::new(&disk.items());
    let before = back.contents();
    let mut earlier = store(&back, EARLIER);
    assert_refusal(&back.open(&mut earlier), 1, Refused::Field, "Growing", "extra", 1);
    assert_unchanged_except_recovery(&before, &mut back);

    let keepable = later_value(0, 0, b"may drop");
    let last = write(&mut disk, &mut later, &[(b"a/1", &keepable)]);
    let mut disk = Disk::new(&disk.items());
    let mut earlier = store(&disk, EARLIER);
    pending(&disk.open(&mut earlier), last);
    assert_eq!(row(&mut earlier, 0, 4).unwrap().layout, expected[0]);
    assert_eq!(order(&mut earlier, 0).unwrap().as_ref(), &expected);
    assert_eq!(get(&mut disk, &mut earlier, b"a/1").as_ref(), BEFORE);
}

#[test]
fn a_job_that_names_a_later_layout_takes_its_values_before_the_keep_check() {
    let (mut disk, mut later) = at_later(LATER, EARLIER, true);
    let unkept = later_value(5, 0, b"");
    write(&mut disk, &mut later, &[(b"a/1", &unkept)]);
    let mut disk = Disk::new(&disk.items());
    let job = crate::Job {
        name: "convert-later",
        counts: &[],
        keys: FAMILY.types[0].keys,
        converts: Some(crate::Converts {
            family: FAMILY.name,
            value_type: 0,
            from: crate::Layouts::One(fingerprint(BACK_LATER)),
        }),
    };
    let set = OwnerSet { jobs: Box::leak(Box::new([job])), ..EARLIER };
    let mut earlier = store(&disk, set);
    let events = disk.open(&mut earlier);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(page.layout.as_ref().unwrap().text.as_ref(), BACK_LATER.as_bytes());
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].key.as_ref(), b"a/1");
    assert_eq!(page.rows[0].value.as_ref(), &[&[0, 3], &unkept[2..]].concat());
    let writes = Box::new([crate::JobWrite::Put { key: b"a/1".as_slice().into(), value: BEFORE.into() }]);
    let end = disk.request(
        &mut earlier,
        Request::Job { owner: *owner, answer: Ok(crate::JobAnswer { writes, through: None, counts: Box::new([]) }) },
    );
    let events = drive_empty(&mut disk, &mut earlier, end);
    assert!(events.iter().any(|event| if let Event::Opened { pending: true, .. } = event { true } else { false }));
    assert_eq!(get(&mut disk, &mut earlier, b"a/1").as_ref(), BEFORE);
}

#[test]
fn a_later_log_held_layout_reads_without_screening_its_values_at_open() {
    let older = Box::leak(BACK_EARLIER.replace(FAMILY.name, "jig-store-tasks").into_boxed_str());
    let newer = Box::leak(BACK_LATER.replace(FAMILY.name, "jig-store-tasks").into_boxed_str());
    let (mut disk, mut later) = at_later(history_set(newer), history_set(older), false);
    // Send the log-held type's bytes through the shared read edge, including a
    // value Read accepts but Keep refuses, and one even Read cannot convert.
    let field = later_value(5, 0, b"not needed on read");
    let variant = later_value(0, 1, b"");
    let last = write(&mut disk, &mut later, &[(b"a/1", &field), (b"a/2", &variant)]);
    let mut disk = Disk::new(&disk.items());
    let mut earlier = store(&disk, history_set(older));
    assert_opened(&disk.open(&mut earlier), last);
    assert_eq!(get(&mut disk, &mut earlier, b"a/1").as_ref(), BEFORE);
    assert_eq!(get(&mut disk, &mut earlier, b"a/2"), app(&mut earlier, b"a/2").unwrap());
}
