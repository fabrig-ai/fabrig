//! The shell drives job pages and observes commits over a copy (protocol/store/jobs.md, 5).
#![expect(clippy::arithmetic_side_effects, reason = "small fixed counts in disk step tests")]
use super::*;
use crate::{Converts, Job, JobAnswer, JobPage, JobRefusal, JobWrite, KeyRange, Owner, PatchSet, PatchSets};
use alloc::collections::VecDeque;

const UPSTREAM_JOB: Job = Job {
    name: "notes-to-labels",
    keys: &[KeyRange { from: jig_test_patches_jig::keys::NOTES_FROM, to: jig_test_patches_jig::keys::NOTES_TO }],
    converts: None,
};
const APP_JOB: Job = Job {
    name: "consume-fixtures",
    keys: &[KeyRange { from: b"l", to: b"m" }, KeyRange { from: b"s", to: b"t" }],
    converts: None,
};
const FOLLOW: Job =
    Job { name: "read-the-first-jobs-writes", keys: &[KeyRange { from: b"l", to: b"m" }], converts: None };
const JOBS: PatchSets = PatchSets {
    // Deliberately downstream first: the component chooses Owner's order.
    sets: &[PatchSet { jobs: &[APP_JOB, FOLLOW], ..BASE.sets[0] }, PatchSet { jobs: &[UPSTREAM_JOB], ..BASE.sets[1] }],
};
const PATCHES_AND_JOBS: PatchSets = PatchSets {
    sets: &[PatchSet { jobs: &[APP_JOB, FOLLOW], ..SETS.sets[0] }, PatchSet { jobs: &[UPSTREAM_JOB], ..SETS.sets[1] }],
};
fn store(disk: &Disk, sets: PatchSets) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    store.application.version = 0;
    store.patch_sets(sets);
    store
}
pub(super) fn done(store: &mut Store, owner: Owner, name: &str) -> Option<u64> {
    let key =
        crate::keys::encode_own(&crate::keys::Own::Job { owner, name: name.into() }, store.limits.store.key).unwrap();
    let byte = match owner {
        Owner::Skein => 0,
        Owner::Smith => 1,
        Owner::Jig => 2,
        Owner::Application => 3,
    };
    assert_eq!(key.as_ref(), [&[0, 8, byte], name.as_bytes(), &[0, 0]].concat());
    assert_eq!(crate::keys::decode_own(&key), Some(crate::keys::Own::Job { owner, name: name.into() }));
    let bytes = fixtures::get_kv(store, &key)?;
    let row =
        crate::registry_v1::Job::decode(&crate::registry_v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
    Some(row.ended())
}
fn app(store: &mut Store, key: &[u8]) -> Option<Box<[u8]>> {
    let key = crate::keys::application(key, store.limits.store.key).unwrap();
    fixtures::get_kv(store, &key)
}
type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
fn snapshot(disk: &mut Disk) -> Contents {
    disk.contents()
        .into_iter()
        .filter(|(path, kind, _, bytes)| {
            // Recovery opens a new empty segment; no committed byte in A changes.
            path.starts_with(b"generations/1/") && (*kind != Kind::File || !bytes.is_empty())
        })
        .collect()
}
fn page(events: Box<[Event]>) -> (Token, Owner, Box<str>, JobPage) {
    let mut events = events.into_vec();
    let Event::Job { owner, of, job, page } = events.remove(0) else { panic!("job page: {events:?}") };
    assert!(events.is_empty());
    (owner, of, job, page)
}
fn answer(page: &JobPage, of: Owner, job: &str) -> JobAnswer {
    assert!(page.layout.is_none());
    if page.end {
        assert!(page.rows.is_empty());
        let writes: Box<[JobWrite]> = if job == APP_JOB.name {
            Box::new([JobWrite::Put { key: b"l/finished".as_slice().into(), value: label(1) }])
        } else {
            Box::new([])
        };
        return JobAnswer { writes, through: None };
    }
    let first = page.rows.first().unwrap();
    if of == Owner::Jig {
        assert_eq!(first.key.as_ref(), &[2, b'n', b'/', b'0']);
        return JobAnswer {
            writes: Box::new([
                JobWrite::Put { key: Box::from([2, b'l', b'/', b'0']), value: label(2) },
                JobWrite::Erase { key: first.key.clone() },
            ]),
            through: Some(first.key.clone()),
        };
    }
    if job == FOLLOW.name {
        assert_eq!(first.key.as_ref(), b"l/finished", "a set's later job reads its earlier one's last writes");
        assert_eq!(first.value, label(1));
        return JobAnswer { writes: Box::new([]), through: None };
    }
    if first.key.as_ref() == b"l/0" {
        assert_eq!(first.value, label(2), "jig's writes precede the application's");
    }
    JobAnswer { writes: Box::new([JobWrite::Erase { key: first.key.clone() }]), through: Some(first.key.clone()) }
}
fn drive(disk: &mut Disk, store: &mut Store, first: Box<[Event]>) -> Vec<Event> {
    let mut events = VecDeque::from(first.into_vec());
    let mut progress = Vec::new();
    let mut rows = 0;
    let mut commits = 0;
    let mut last_key: Option<Box<[u8]>> = None;
    for _ in 0_u32..100 {
        let Some(event) = events.pop_front() else { return progress };
        match event {
            Event::Job { owner, of, job, page } => {
                if of == Owner::Jig {
                    store.limits.store.ops = 3;
                }
                assert!(page.rows.len() <= usize::try_from(store.limits.store.ops).unwrap());
                assert!(page.rows.windows(2).all(|r| r[0].key < r[1].key));
                if let Some(first) = page.rows.first() {
                    if let Some(last) = &last_key {
                        assert!(first.key > *last);
                    }
                    last_key = Some(first.key.clone());
                }
                rows += u64::try_from(page.rows.len()).unwrap();
                commits += 1;
                // Context reads remain served while a job page is out.
                let context = store.patch_page(skein_kv::Range::prefix(b"l"), None, 1).unwrap();
                assert!(context.rows.len() <= 1);
                let before = store.stores.store.last();
                let response = answer(&page, of, &job);
                let next = disk.request(store, Request::Job { owner, answer: Ok(response) });
                let sealed = next.iter().any(|event| match event {
                    Event::Opened { .. } => true,
                    _ => false,
                });
                assert_eq!(
                    store.stores.store.last(),
                    before + 1 + u64::from(sealed),
                    "one answer commit, with only the seal after the last job"
                );
                if page.end {
                    assert_eq!(done(store, of, &job), Some(before + 1));
                }
                events.extend(next);
            }
            Event::Ran { of, ref job, rows: found, commits: made, .. } => {
                assert_eq!(found, rows);
                assert_eq!(made, commits);
                assert!(done(store, of, job).is_some_and(|ended| ended <= store.stores.store.last()));
                rows = 0;
                commits = 0;
                last_key = None;
                progress.push(event);
            }
            Event::Site { owner, key, site, at, mut patcher, .. } => {
                let mut context = jig_test_patches::patches::Context { key };
                let (old, new) = patcher.parts();
                let result = jig_test_patches::patches::written(site, old, new, at, &mut context);
                events.extend(disk.request(store, Request::Site { owner, patcher, result }));
            }
            Event::Patched { .. } | Event::Opened { .. } | Event::Census { .. } => progress.push(event),
            other => panic!("unexpected job start answer: {other:?}"),
        }
    }
    panic!("jobs did not end")
}
#[test]
fn a_page_rooms_answer_fits_the_exact_commit_edge_for_whole_and_relative_keys() {
    assert_eq!(JobWrite::Put { key: Box::new([1, 2, 3]), value: Box::new([4, 5, 6, 7]) }.bytes(), 16);
    assert_eq!(JobWrite::Erase { key: Box::new([1, 2, 3]) }.bytes(), 8);
    for of in [Owner::Jig, Owner::Application] {
        let sets = PatchSets {
            sets: Box::leak(Box::new([
                PatchSet { jobs: if of == Owner::Application { &[FOLLOW] } else { &[] }, ..BASE.sets[0] },
                PatchSet { jobs: if of == Owner::Jig { &[UPSTREAM_JOB] } else { &[] }, ..BASE.sets[1] },
            ])),
        };
        let mut disk = Disk::new(&fixture_baseline(None));
        let mut store = store(&disk, sets);
        // Fit three puts, their frame and the done row exactly.
        let writes: Box<[JobWrite]> = (0..3_u8)
            .map(|n| JobWrite::Put {
                key: if of == Owner::Application {
                    vec![b'l', b'/', b'x' + n].into()
                } else {
                    vec![2, b'l', b'/', b'x' + n].into()
                },
                value: label(1),
            })
            .collect();
        let declared = if of == Owner::Jig { UPSTREAM_JOB } else { FOLLOW };
        let done_op = crate::jobs::kept::operation(&store, of, &declared, u64::MAX).unwrap();
        let skein_kv::Op::Put { key, value } = done_op else { panic!("done put") };
        let row = skein_kv::put_bytes(u32::try_from(key.len()).unwrap(), u32::try_from(value.len()).unwrap()).unwrap();
        let bytes = writes.iter().map(JobWrite::bytes).sum::<u64>();
        let tags = if of == Owner::Application { 3 } else { 0 };
        let commit = u32::try_from(u64::from(skein_kv::FRAME_BYTES + row) + bytes + tags).unwrap();
        let mut next = disk.open(&mut store);
        // Recovery reads the baseline's larger frames before the job's tiny edge.
        store.limits.store.commit = commit;
        let (owner, ending) = loop {
            assert!(
                match next.first() {
                    Some(Event::Job { .. }) => true,
                    _ => false,
                },
                "{next:?}"
            );
            let (owner, found, name, page) = page(next);
            assert_eq!((found, name.as_ref()), (of, declared.name));
            if page.end {
                break (owner, page);
            }
            next = disk.request(
                &mut store,
                Request::Job { owner, answer: Ok(JobAnswer { writes: Box::new([]), through: None }) },
            );
        };
        assert_eq!(ending.room.writes, 3);
        assert_eq!(ending.room.bytes, bytes);
        let events = disk.request(&mut store, Request::Job { owner, answer: Ok(JobAnswer { writes, through: None }) });
        assert!(
            events.iter().any(|e| match e {
                Event::Ran { of: found, .. } => *found == of,
                _ => false,
            }),
            "{events:?}"
        );
        assert!(done(&mut store, of, declared.name).is_some());
    }
}
#[test]
fn pending_jobs_page_in_owner_and_set_order_commit_answers_then_seal_and_run_no_more() {
    let mut disk = Disk::new(&fixture_baseline(None));
    let before = snapshot(&mut disk);
    let mut store = store(&disk, JOBS);
    let opened = disk.open(&mut store);
    let events = drive(&mut disk, &mut store, opened);
    let [
        Event::Ran { of: Owner::Jig, job: jig, rows: 1, commits: 2, .. },
        Event::Ran { of: Owner::Application, job: app_job, commits: 8, .. },
        Event::Ran { of: Owner::Application, job: follow, rows: 1, commits: 2, .. },
        Event::Opened { last, pending: true, .. },
        Event::Census { .. },
    ] = events.as_slice()
    else {
        panic!("job progress: {events:?}")
    };
    assert_eq!(jig.as_ref(), UPSTREAM_JOB.name);
    assert_eq!(app_job.as_ref(), APP_JOB.name);
    assert_eq!(follow.as_ref(), FOLLOW.name);
    assert_eq!(*last, FIXTURE_LAST + 13);
    assert_eq!(app(&mut store, b"l/0"), None);
    assert_eq!(app(&mut store, b"l/finished"), Some(label(1)));
    assert_eq!(snapshot(&mut disk), before, "A stays byte-identical until the switch");
    assert_sealed(&mut store, *last);
    let switched = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
    assert!(match switched.as_ref() {
        [Event::Switched { .. }] => true,
        _ => false,
    });
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    disk.crash(17);
    let mut again = self::store(&disk, JOBS);
    let events = disk.open(&mut again);
    assert!(match events.as_ref() {
        [Event::Opened { last: found, pending: false, .. }, Event::Census { .. }] if found == last => true,
        _ => false,
    });
    assert!(!disk.tree().iter().any(|(path, ..)| path.as_ref() == b"generations/3"));
}
#[test]
fn an_empty_stores_first_commit_marks_every_declared_job_done() {
    let mut disk = Disk::new(&[]);
    let mut store = store(&disk, JOBS);
    assert_opened(&disk.open(&mut store), 0);
    assert!(match disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)]))).as_ref() {
        [Event::Committed { number: 1, .. }] => true,
        _ => false,
    });
    for (of, name) in
        [(Owner::Jig, UPSTREAM_JOB.name), (Owner::Application, APP_JOB.name), (Owner::Application, FOLLOW.name)]
    {
        assert_eq!(done(&mut store, of, name), Some(1));
    }
    disk.crash(18);
    let mut again = self::store(&disk, JOBS);
    assert!(match disk.open(&mut again).as_ref() {
        [Event::Opened { last: 1, pending: false, .. }, Event::Census { .. }] => true,
        _ => false,
    });
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
}
#[test]
fn refused_or_out_of_bounds_answers_abandon_the_copy_and_retry_from_the_first_page() {
    for flaw in ["owner refusal", "operations", "bytes", "key", "value", "before first", "not a page key"] {
        let mut disk = Disk::new(&fixture_baseline(None));
        let before = snapshot(&mut disk);
        let mut store = store(&disk, JOBS);
        let (owner, of, job, first) = page(disk.open(&mut store));
        let first_key = first.rows[0].key.clone();
        // A committed answer must still be abandoned with its generation.
        let (owner, _, _, end) =
            page(disk.request(&mut store, Request::Job { owner, answer: Ok(answer(&first, of, &job)) }));
        assert!(end.end);
        let mut response = JobAnswer { writes: Box::new([]), through: None };
        let result = match flaw {
            "owner refusal" => Err(JobRefusal { key: Some(first_key.clone()), why: "owner says no".into() }),
            "operations" => {
                response.writes =
                    (0..store.limits.commits.writes).map(|_| JobWrite::Erase { key: first_key.clone() }).collect();
                Ok(response)
            }
            "bytes" => {
                store.limits.store.commit = 20;
                Ok(response)
            }
            "key" => {
                response.writes = Box::new([JobWrite::Erase {
                    key: vec![1; usize::try_from(store.limits.store.key).unwrap() + 1].into(),
                }]);
                Ok(response)
            }
            "value" => {
                response.writes = Box::new([JobWrite::Put {
                    key: first_key.clone(),
                    value: vec![0; usize::try_from(store.limits.store.value).unwrap() + 1].into(),
                }]);
                Ok(response)
            }
            "before first" => {
                response.through = Some(Box::new([]));
                Ok(response)
            }
            "not a page key" => {
                response.through = Some(first_key.clone());
                Ok(response)
            }
            _ => unreachable!(),
        };
        let events = disk.request(&mut store, Request::Job { owner, answer: result });
        assert!(
            match events.as_ref() {
                [Event::OpenRefused { why: OpenRefusal::Job { owner: Owner::Jig, job, .. }, .. }]
                    if job.as_ref() == UPSTREAM_JOB.name =>
                    true,
                _ => false,
            },
            "{flaw}: {events:?}"
        );
        assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
        assert_eq!(snapshot(&mut disk), before, "{flaw}");
        disk.crash(19);
        let mut again = self::store(&disk, JOBS);
        let (_, _, _, retried) = page(disk.open(&mut again));
        assert_eq!(retried.rows[0].key, first_key, "{flaw}: retry begins at the first page");
    }
}
#[test]
fn a_nonempty_pages_through_must_consume_a_key_of_that_page() {
    for through in [b"".as_slice(), b"n/0", b"z"] {
        let mut disk = Disk::new(&fixture_baseline(None));
        let mut store = store(&disk, JOBS);
        let (owner, _, _, first) = page(disk.open(&mut store));
        assert!(!first.end && !first.rows.is_empty());
        let events = disk.request(
            &mut store,
            Request::Job { owner, answer: Ok(JobAnswer { writes: Box::new([]), through: Some(through.into()) }) },
        );
        assert!(match events.as_ref() {
            [Event::OpenRefused { why: OpenRefusal::Job { .. }, .. }] => true,
            _ => false,
        });
        assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    }
}
#[test]
fn patches_finish_before_pending_jobs_and_the_seal_follows_the_jobs() {
    let mut disk = Disk::new(&fixture_baseline(None));
    let mut store = store(&disk, PATCHES_AND_JOBS);
    let opened = disk.open(&mut store);
    let events = drive(&mut disk, &mut store, opened);
    let steps: Vec<_> = events
        .iter()
        .map(|event| match event {
            Event::Patched { .. } => "patch",
            Event::Ran { .. } => "job",
            Event::Opened { pending: true, .. } => "opened",
            Event::Census { .. } => "census",
            _ => panic!("{event:?}"),
        })
        .collect();
    assert_eq!(steps, ["patch", "patch", "patch", "patch", "job", "job", "job", "opened", "census"]);
    let last = store.stores.store.last();
    assert_sealed(&mut store, last);
}

#[test]
fn job_admission_checks_names_ranges_map_and_written_family_on_both_owners() {
    for (base, job) in [(BASE.sets[0], APP_JOB), (BASE.sets[1], UPSTREAM_JOB)] {
        let first = job.keys[0];
        let half: &'static [u8] = if base.owner == Owner::Application { b"l/x" } else { &[2, b'n', b'/', b'x'] };
        let ordered: &'static [KeyRange] =
            Box::leak(Box::new([KeyRange { from: first.from, to: half }, KeyRange { from: half, to: first.to }]));
        let secret_family = crate::KeptFamily { map: crate::KeptMap::Secrets, ..base.families[0] };
        let with_secret = PatchSet { families: Box::leak(vec![secret_family].into_boxed_slice()), ..base };
        let mut families = base.families.to_vec();
        if base.owner == Owner::Jig {
            families.push(BASE.sets[0].families[2]);
        }
        let with_written_secret = PatchSet { families: Box::leak(families.into_boxed_slice()), ..base };
        let converts = Some(Converts { family: base.families[0].name, value_type: 1, from: [0; 32] });
        let bound = Box::leak("n".repeat(64).into_boxed_str());
        let past = Box::leak("n".repeat(65).into_boxed_str());
        for (label, set, jobs, accepted) in [
            ("one name", base, vec![job], true),
            ("names unique", base, vec![job, job], false),
            ("name at 64 bytes", base, vec![Job { name: bound, ..job }], true),
            ("name past 64 bytes", base, vec![Job { name: past, ..job }], false),
            ("adjacent ranges", base, vec![Job { keys: ordered, ..job }], true),
            ("empty ranges", base, vec![Job { keys: &[], ..job }], false),
            (
                "empty interval",
                base,
                vec![Job { keys: Box::leak(Box::new([KeyRange { from: first.from, to: first.from }])), ..job }],
                false,
            ),
            (
                "reversed interval",
                base,
                vec![Job { keys: Box::leak(Box::new([KeyRange { from: first.to, to: first.from }])), ..job }],
                false,
            ),
            ("overlapping intervals", base, vec![Job { keys: Box::leak(Box::new([first, first])), ..job }], false),
            (
                "unordered intervals",
                base,
                vec![Job { keys: Box::leak(Box::new([ordered[1], ordered[0]])), ..job }],
                false,
            ),
            ("outside owner", base, vec![Job { keys: &[KeyRange { from: &[3], to: &[4] }], ..job }], false),
            ("in secrets alone", with_secret, vec![Job { keys: Box::leak(Box::new([first])), ..job }], false),
            ("written unregistered family", base, vec![Job { converts, ..job }], false),
            (
                "written family in secrets",
                with_written_secret,
                vec![Job { converts: Some(Converts { family: "fixture-token", value_type: 1, from: [0; 32] }), ..job }],
                false,
            ),
            (
                "written family missing",
                base,
                vec![Job { converts: Some(Converts { family: "absent", value_type: 1, from: [0; 32] }), ..job }],
                false,
            ),
        ] {
            let sets = PatchSets {
                sets: Box::leak(vec![PatchSet { jobs: Box::leak(jobs.into_boxed_slice()), ..set }].into_boxed_slice()),
            };
            let mut disk = Disk::new(&[]);
            let mut store = store(&disk, sets);
            let events = disk.open(&mut store);
            if accepted {
                assert_opened(&events, 0);
            } else {
                assert!(
                    match events.as_ref() {
                        [Event::OpenRefused { why: OpenRefusal::Set { owner, patch: 0 }, .. }]
                            if *owner == base.owner =>
                            true,
                        _ => false,
                    },
                    "{label} {:?}: {events:?}",
                    base.owner
                );
            }
        }
    }
}
