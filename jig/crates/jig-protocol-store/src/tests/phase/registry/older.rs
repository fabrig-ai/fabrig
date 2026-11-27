//! Older map layouts are read, with a check before any write (jig/docs/design/protocol/store/layouts.md, 5.6).
use super::{
    back::{get, write},
    *,
};
use crate::{Converts, Job, JobAnswer, JobWrite, Layouts, Refusals};
use jig_test_layouts::layouts::{READ_BINARY, READ_OLDER};
use skein_codec::layout::{Refused, Why};

const OLDER: OwnerSet = OwnerSet { families: &[KeptFamily { current: READ_OLDER, ..FAMILY }], ..SET };
const BINARY: OwnerSet = OwnerSet { families: &[KeptFamily { current: READ_BINARY, ..FAMILY }], ..SET };
const KEPT_VARIANT: &[u8] = &[0, 2, 0];
const AT_BOUND: &[u8] = &[0, 2, 0, 255];
const PAST_BOUND: &[u8] = &[0, 2, 1, 0];
const DROPPED_VARIANT: &[u8] = &[0, 2, 1];
const ANOTHER_VARIANT: &[u8] = &[0, 2, 2];
const OLDER_ID: u16 = 2; // The family's version, used for its first layout.
const GOOD_AND_REFUSING: u64 = 3; // One original value and two refusing values.
const REFUSING: u64 = 2;
const PAGE_BYTES: u32 = 512; // Room for every registry row but not two keys with tails.
const KEY_TAIL_BYTES: usize = 256;

fn older(rows: &[(&[u8], &[u8])]) -> Disk {
    let mut disk = Disk::new(&[]);
    let mut previous = fresh(&mut disk, OLDER);
    write(&mut disk, &mut previous, &[(b"a/0", KEPT_VARIANT), (b"c/0", AT_BOUND)]);
    if !rows.is_empty() {
        write(&mut disk, &mut previous, rows);
    }
    Disk::new(&disk.items())
}

#[test]
fn older_values_read_at_the_binary_layout_without_a_job() {
    let mut disk = older(&[]);
    let mut current = store(&disk, BINARY);
    let events = disk.open(&mut current);
    let [Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(get(&mut disk, &mut current, b"a/0").as_ref(), KEPT_VARIANT);
    assert_eq!(get(&mut disk, &mut current, b"c/0").as_ref(), &[0, 2, 255]);
}

fn refusal(value_type: u16, values: u64, refused: u64, why: Refused, declaration: &str, part: &str) -> OpenRefusal {
    OpenRefusal::Layout {
        family: FAMILY.name.into(),
        value_type,
        id: OLDER_ID,
        values: Some(values),
        why: Unplanned::Unread(Refusals { refused, why, declaration: declaration.into(), part: part.into() }),
    }
}

#[test]
fn older_refusals_count_across_pages_before_open_or_rehearsal_writes() {
    let cases = [
        ([(b"a/1", DROPPED_VARIANT), (b"a/2", ANOTHER_VARIANT)], 0, Refused::Variant, "Choice", "dropped"),
        ([(b"c/1", PAST_BOUND), (b"c/2", PAST_BOUND)], 1, Refused::Narrowed, "Stable", "n"),
    ];
    for (rows, value_type, why, declaration, part) in cases {
        let [(first, a), (second, b)] =
            rows.map(|(key, value)| ([key.as_slice(), &[b'x'; KEY_TAIL_BYTES]].concat(), value));
        let items = older(&[(&first, a), (&second, b)]).items();
        let expected = refusal(value_type, GOOD_AND_REFUSING, REFUSING, why, declaration, part);
        for rehearsing in [false, true] {
            let mut disk = Disk::new(&items);
            let before = disk.contents();
            let mut current = store(&disk, BINARY);
            current.limits.store.page.bytes = PAGE_BYTES;
            let events = if rehearsing {
                let scratch = disk.scratch_root();
                let directory = disk.root();
                disk.request(
                    &mut current,
                    Request::Rehearse {
                        owner: Token::new(1),
                        directory,
                        scratch,
                        installation: fixtures::INSTALLATION,
                        reach: crate::Reach::Whole,
                    },
                )
            } else {
                disk.open(&mut current)
            };
            let Some(Event::OpenRefused { why, .. }) = events.last() else { panic!("{events:?}") };
            assert_eq!(why, &expected);
            assert_unchanged_except_recovery(&before, &mut disk);
        }
    }
}

fn with_job(from: Layouts) -> OwnerSet {
    let converts = Converts { family: FAMILY.name, value_type: 0, from };
    let job = Job { name: "convert-older", counts: &[], keys: FAMILY.types[0].keys, converts: Some(converts) };
    OwnerSet { jobs: Box::leak(Box::new([job])), ..BINARY }
}

fn open_with_job(disk: &mut Disk, current: &mut Store) -> Vec<Event> {
    let events = disk.open(current);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert!(!page.end);
    assert_eq!(page.layout.as_ref().unwrap().text.as_ref(), READ_OLDER.as_bytes());
    assert_eq!(
        page.rows.iter().map(|row| (row.key.as_ref(), row.value.as_ref())).collect::<Vec<_>>(),
        [(b"a/0".as_slice(), KEPT_VARIANT), (b"a/1".as_slice(), DROPPED_VARIANT)]
    );
    let writes =
        page.rows.iter().map(|row| JobWrite::Put { key: row.key.clone(), value: KEPT_VARIANT.into() }).collect();
    let answer = Ok(JobAnswer { counts: Box::new([]), writes, through: None });
    let events = disk.request(current, Request::Job { owner: *owner, answer });
    drive_empty(disk, current, events)
}

#[test]
fn written_jobs_take_the_older_layout_as_stored() {
    let schema = skein_codec::schema::parse(READ_OLDER.as_bytes()).unwrap();
    let fingerprint = schema.layout(schema.root(b"Growing", None).unwrap(), &[]).unwrap().digest();
    for from in [Layouts::One(fingerprint), Layouts::Unplanned] {
        let mut disk = older(&[(b"a/1", DROPPED_VARIANT)]);
        let mut current = store(&disk, with_job(from));
        let events = open_with_job(&mut disk, &mut current);
        let [Event::Ran { .. }, Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_slice() else {
            panic!("{events:?}");
        };
        assert_eq!(get(&mut disk, &mut current, b"a/1").as_ref(), KEPT_VARIANT);
    }
}

#[test]
fn a_done_unplanned_job_does_not_take_a_fallible_read_layout() {
    let mut disk = older(&[(b"a/1", DROPPED_VARIANT)]);
    let set = with_job(Layouts::Unplanned);
    let mut current = store(&disk, set);
    open_with_job(&mut disk, &mut current);
    switch(&mut disk, &mut current);
    // Put the refused value back at its original layout after the job is done.
    raw(
        &mut disk,
        &mut current,
        vec![skein_kv::Op::Put {
            key: crate::keys::application(b"a/1", fixtures::KEY_LIMIT).unwrap(),
            value: DROPPED_VARIANT.into(),
        }],
    );
    let mut disk = Disk::new(&disk.items());
    let before = disk.contents();
    let mut current = store(&disk, set);
    let events = disk.open(&mut current);
    let [Event::OpenRefused { why, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(*why, refusal(0, 1, 1, Refused::Variant, "Choice", "dropped"));
    assert_unchanged_except_recovery(&before, &mut disk);
}

fn frame_set(current: &'static str) -> OwnerSet {
    let family = KeptFamily {
        current,
        frames: Some(crate::Frames { root: "Growing", texts: &[], number: 0 }),
        keys: &[],
        types: &[],
        ..FAMILY
    };
    OwnerSet { owner: Owner::Smith, families: Box::leak(Box::new([family])), ..SET }
}

#[test]
fn dropping_an_older_log_or_frame_variant_is_underived() {
    let log_older = Box::leak(READ_OLDER.replace(FAMILY.name, "jig-store-tasks").into_boxed_str());
    let log_binary = Box::leak(READ_BINARY.replace(FAMILY.name, "jig-store-tasks").into_boxed_str());
    for (older, binary, name) in [
        (history_set(log_older), history_set(log_binary), "jig-store-tasks"),
        (frame_set(READ_OLDER), frame_set(READ_BINARY), FAMILY.name),
    ] {
        let mut disk = Disk::new(&[]);
        fresh(&mut disk, older);
        let mut disk = Disk::new(&disk.items());
        let mut current = store(&disk, binary);
        let events = disk.open(&mut current);
        let [
            Event::OpenRefused {
                why:
                    OpenRefusal::Layout {
                        family,
                        value_type: 0,
                        id: OLDER_ID,
                        values: None,
                        why: Unplanned::Underived(why),
                    },
                ..
            },
        ] = events.as_ref()
        else {
            panic!("{events:?}")
        };
        assert_eq!((family.as_ref(), why.why), (name, Why::VariantDropped));
    }
}
