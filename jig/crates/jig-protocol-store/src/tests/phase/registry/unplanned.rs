//! Written jobs over every underived layout (protocol/store/jobs.md, 4 and 6).
use super::*;
use crate::{Converts, Job, JobAnswer, JobWrite, Layouts};
use jig_test_layouts::layouts::WIDE;

fn written_job(from: Layouts) -> Job {
    Job {
        name: "unplanned",
        keys: FAMILY.types[0].keys,
        converts: Some(Converts { family: FAMILY.name, value_type: 0, from }),
        counts: &[],
    }
}
fn with_job(set: OwnerSet) -> OwnerSet {
    OwnerSet { jobs: Box::leak(Box::new([written_job(Layouts::Unplanned)])), ..set }
}
fn assert_opened_pending(events: &[Event]) {
    assert!(
        events.iter().any(|event| if let Event::Opened { pending: true, .. } = event { true } else { false }),
        "{events:?}"
    );
}

fn two_layout_store() -> Disk {
    let mut disk = Disk::new(&base(false));
    let mut previous = store(&disk, OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }], ..SET });
    assert_opened(&disk.open(&mut previous), 1);
    let mut ops = vec![
        old_row(3, FAMILY.name, 0),
        old_text(),
        layout_row(WIDE, 5, FAMILY.name, 0),
        layout_text(WIDE),
        layout_row(WRITTEN, 7, FAMILY.name, 0),
        layout_text(WRITTEN),
    ];
    for (key, value) in [
        (b"a/0", vec![0, 5, 0, 0, 0, 40, 8]),
        (b"a/1", vec![0, 3, 0, 41]),
        (b"a/2", vec![0, 3, 0, 42]),
        (b"a/3", vec![0, 3, 0, 43]),
        (b"a/4", vec![0, 3, 0, 44]),
        (b"a/5", vec![0, 5, 0, 0, 0, 45, 8]),
        (b"a/6", vec![0, 7, 0, 46, 8]),
    ] {
        ops.push(skein_kv::Op::Put {
            key: crate::keys::application(key, 1024).unwrap(),
            value: value.into_boxed_slice(),
        });
    }
    raw(&mut disk, &mut previous, ops);
    Disk::new(&disk.items())
}

#[test]
fn unplanned_jobs_walk_each_underived_layout_with_its_texts_and_retire_after_conversion() {
    let mut disk = two_layout_store();
    let set = with_job(written_set());
    let mut store = store(&disk, set);
    let mut events = disk.open(&mut store);
    let mut consumed = Vec::new();
    let mut ids = Vec::new();
    let mut pages = 0_u32;
    for _ in 0_u32..20 {
        let Some(Event::Job { owner, of, job, page }) = events.first() else { break };
        assert_eq!((*of, job.as_ref()), (Owner::Application, "unplanned"));
        let layout = page.layout.as_ref().expect("every walk's page has its layout");
        assert_eq!(layout.root.as_ref(), "Growing");
        assert_eq!(layout.variant, None);
        assert!(layout.held.is_empty());
        let id = if layout.text.as_ref() == KEPT.as_bytes() {
            3
        } else {
            assert_eq!(layout.text.as_ref(), WIDE.as_bytes());
            5
        };
        if ids.last() != Some(&id) {
            ids.push(id);
        }
        pages += 1;
        let (writes, through): (Box<[JobWrite]>, Option<Box<[u8]>>) = if page.end {
            assert!(page.rows.is_empty());
            assert_eq!(id, 5, "only the last walk ends the job");
            (Box::new([]), None)
        } else {
            assert!(!page.rows.is_empty());
            for row in &page.rows {
                assert_eq!(crate::registry::head(&row.value), Some(id));
                if id == 3 {
                    assert_eq!(row.value.len(), 4);
                } else {
                    assert_eq!(row.value.len(), 7);
                }
            }
            // Consume just one row, so every next page must keep the same
            // layout until that walk is exhausted, then reset to the first key.
            let row = &page.rows[0];
            consumed.push((id, row.key.clone()));
            let n = if id == 3 { row.value[3] } else { row.value[5] };
            (
                Box::new([JobWrite::Put { key: row.key.clone(), value: Box::from([0, 2, 0, n, 8]) }]),
                Some(row.key.clone()),
            )
        };
        events = disk.request(
            &mut store,
            Request::Job { owner: *owner, answer: Ok(JobAnswer { counts: Box::new([]), writes, through }) },
        );
    }
    assert_eq!(ids, [3, 5], "{events:?}");
    assert_eq!(pages, 7);
    assert_eq!(
        consumed,
        [(3, b"a/1"), (3, b"a/2"), (3, b"a/3"), (3, b"a/4"), (5, b"a/0"), (5, b"a/5")]
            .map(|(id, key)| (id, Box::from(key.as_slice())))
    );
    assert_opened_pending(&events);
    for (key, n) in [(b"a/0", 40), (b"a/1", 41), (b"a/2", 42), (b"a/3", 43), (b"a/4", 44), (b"a/5", 45), (b"a/6", 46)] {
        assert_eq!(app(&mut store, key).unwrap().as_ref(), &[0, 7, 0, n, 8]);
    }
    assert!(row(&mut store, 0, 3).is_some());
    assert!(row(&mut store, 0, 5).is_some());
    switch(&mut disk, &mut store);
    let mut disk = Disk::new(&disk.items());
    let set = OwnerSet { jobs: Box::leak(Box::new([set.jobs[0], NOOP])), ..set };
    let mut store = self::store(&disk, set);
    let events = disk.open(&mut store);
    let events = drive_empty(&mut disk, &mut store, events);
    assert_opened_pending(&events);
    assert!(row(&mut store, 0, 3).is_none());
    assert!(row(&mut store, 0, 5).is_none());
    assert_eq!(text(&mut store, hash(WIDE)), None);
    assert!(row(&mut store, 0, 7).is_some());
}

#[test]
fn an_unplanned_job_without_underived_layouts_has_one_empty_end_page_without_a_layout() {
    let mut disk = Disk::new(&base(true));
    let mut store = store(&disk, with_job(SET));
    let events = disk.open(&mut store);
    let [Event::Job { owner, page, .. }] = events.as_ref() else { panic!("{events:?}") };
    assert!(page.rows.is_empty());
    assert!(page.end);
    assert!(page.layout.is_none(), "a reachable older layout is not an unplanned layout");
    let events = disk.request(
        &mut store,
        Request::Job {
            owner: *owner,
            answer: Ok(JobAnswer { counts: Box::new([]), writes: Box::new([]), through: None }),
        },
    );
    assert_opened_pending(&events);
    assert!(!events.iter().any(|event| if let Event::Job { .. } = event { true } else { false }));
}

#[test]
fn a_done_unplanned_job_does_not_exempt_a_later_underived_layout() {
    let mut disk = Disk::new(&base(false));
    let set = with_job(written_set());
    let mut store = store(&disk, set);
    let events = disk.open(&mut store);
    let events = drive_empty(&mut disk, &mut store, events);
    assert_opened_pending(&events);
    switch(&mut disk, &mut store);
    raw(
        &mut disk,
        &mut store,
        vec![
            layout_row(WIDE, 5, FAMILY.name, 0),
            layout_text(WIDE),
            skein_kv::Op::Put { key: b"\x02a/0".as_slice().into(), value: Box::from([0, 5, 0, 0, 0, 42, 8]) },
        ],
    );
    let before = snapshot(&mut disk);
    let mut store = self::store(&disk, set);
    let events = disk.open(&mut store);
    let [
        Event::OpenRefused {
            why: OpenRefusal::Layout { family, value_type: 0, id: 5, values: Some(1), why: Unplanned::Underived(_), .. },
            ..
        },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), FAMILY.name);
    assert_eq!(snapshot(&mut disk), before);
}

#[test]
fn written_job_admission_refuses_duplicate_unplanned_and_duplicate_fingerprints() {
    for from in [Layouts::Unplanned, Layouts::One([7; 32])] {
        let job = written_job(from);
        let set = OwnerSet { jobs: Box::leak(Box::new([job, Job { name: "second", ..job }])), ..written_set() };
        let mut disk = Disk::new(&[]);
        let mut store = store(&disk, set);
        let events = disk.open(&mut store);
        assert!(
            if let [Event::OpenRefused { why: OpenRefusal::Set { owner: Owner::Application }, .. }] = events.as_ref() {
                true
            } else {
                false
            },
            "{from:?}: {events:?}"
        );
        // The jobs' admission compares sets too, independently of their
        // families' other admission rules.
        let split = OwnerSets {
            sets: Box::leak(Box::new([
                OwnerSet { jobs: Box::leak(Box::new([job])), ..set },
                OwnerSet { owner: Owner::Jig, jobs: Box::leak(Box::new([Job { name: "second", ..job }])), ..set },
            ])),
        };
        assert_eq!(crate::jobs::admission::check(&split), Err(Owner::Jig));
    }
    // Different fingerprints, and One beside Unplanned, make distinct promises.
    let one = written_job(Layouts::One([7; 32]));
    let jobs = [
        one,
        Job { name: "other", converts: Some(Converts { from: Layouts::One([8; 32]), ..one.converts.unwrap() }), ..one },
        Job { name: "all", ..written_job(Layouts::Unplanned) },
        Job {
            name: "sibling",
            keys: FAMILY.types[1].keys,
            converts: Some(Converts { value_type: 1, ..one.converts.unwrap() }),
            ..one
        },
    ];
    let mut disk = Disk::new(&[]);
    let set = OwnerSet { jobs: Box::leak(Box::new(jobs)), ..written_set() };
    let mut store = store(&disk, set);
    assert_opened(&disk.open(&mut store), 0);
}
